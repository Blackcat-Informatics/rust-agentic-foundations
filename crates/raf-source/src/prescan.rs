// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The token pre-scan: one lexer feeding independent counters.
//!
//! `syn` recurses on more than delimiters, so before a file is parsed this
//! scan walks its tokens once and refuses the file when a counter passes its
//! [`Budget`]. The lexer skips comments (block comments nest) and every kind
//! of string and character literal, so text inside them counts for nothing.
//! The counters are independent of each other and each is defined below.
//!
//! * **Delimiters.** `(`, `[` and `{` open a level and the matching closer
//!   ends it. The count is the number of levels open at once.
//! * **Angles.** Each delimiter level has its own count. `<` adds one unless
//!   it is part of `<=`, and `>` subtracts one down to zero. `->` and `=>` are
//!   single tokens and subtract nothing; `>=` subtracts. The count of a level
//!   returns to zero at `;`, `&&` and `||`, and the count of the enclosing
//!   level returns to zero when a `{ ... }` block closes, so comparisons in
//!   separate statements do not add up.
//! * **Prefix operators.** The length of a run of consecutive tokens that are
//!   `!`, `-`, `*`, `&` or `&&` (a `&&` is one token), with only whitespace and
//!   comments between them. `!=`, `-=`, `*=`, `&=` and `->` are not prefix
//!   operators. A lifetime between two operators does not end the run.
//! * **Closure bars.** A closure head is `||`, or `|` up to the next `|` at the
//!   same level. The count is the length of a chain of heads in which each head
//!   follows the previous one directly, or with `move` between them.
//! * **`else if`.** In one delimiter level, the number of links: an `else`
//!   directly after a `}`, followed by `if`. The count returns to zero at `;`,
//!   at an `else` that is not followed by `if`, and at an identifier other than
//!   `else` directly after a `}`. A block in a condition (`else if {x} {}`)
//!   does not end the chain.
//!
//! * **Recursion.** One additive count of the nesting that the parser, the
//!   syntax tree and its drop build by recursion, summed over every open
//!   delimiter level; it is defined in [`recursion`].
//!
//! * **Tokens.** Every token of the file counts, as the parser's lexer sees
//!   it: an identifier, a keyword, a literal, a lifetime (two: the quote and
//!   the name), each delimiter, and each character of punctuation (`::` and
//!   `->` are two). A doc comment counts as the attribute it stands for: six
//!   tokens for `///` and `/**`, seven for `//!` and `/*!`. Other comments
//!   count for nothing. The count feeds no other counter.
//!
//! The scan does not parse. An unbalanced closer is ignored here and left to
//! the parser to refuse.

use crate::budget::Budget;
use crate::error::{LoadError, LoadErrorKind};
use crate::path::RelPath;

mod recursion;

use recursion::Recursion;

/// The delimiter a group opens with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Delimiter {
    Paren,
    Bracket,
    Brace,
}

/// One token the counters care about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token<'a> {
    Open(Delimiter),
    CloseBrace,
    CloseOther,
    Ident(&'a str),
    Bang,
    Minus,
    Star,
    Amp,
    AmpAmp,
    Pipe,
    PipePipe,
    Lt,
    Gt,
    Semi,
    Comma,
    /// `=>`.
    FatArrow,
    Colon,
    /// `::`.
    PathSep,
    Hash,
    /// Any other operator: `+`, `/`, `%`, `^`, `=`, `?`, `@`, `.`, a range
    /// (`..`, `..=`, `...`), `->`, and the compound forms that end in `=`.
    Operator,
    /// A lifetime or a label such as `'a`.
    Lifetime,
    /// A literal: a string, a character, a byte or a number without a `.`.
    Literal,
    /// A number with a `.`. After a `.` it is two tuple indices (`x.0.1`).
    Float,
    /// Any other punctuation.
    Other,
}

/// Counter state of one delimiter level.
#[derive(Debug, Default, Clone, Copy)]
struct Level {
    angles: usize,
    else_ifs: usize,
    after_brace: bool,
    pending_else: bool,
}

/// All counters.
struct Counters {
    levels: Vec<Level>,
    prefix_run: usize,
    closure_chain: usize,
    closure_pending_end: bool,
    /// The level depth at which an open closure head began, if one is open.
    closure_head_depth: Option<usize>,
}

/// A counter went over its budget at a line.
#[derive(Debug, Clone, Copy)]
struct Exceeded {
    budget: Budget,
    line: usize,
}

impl Counters {
    fn new() -> Self {
        Self {
            levels: vec![Level::default()],
            prefix_run: 0,
            closure_chain: 0,
            closure_pending_end: false,
            closure_head_depth: None,
        }
    }

    /// The level currently open; the stack is never empty.
    fn top(&mut self) -> &mut Level {
        let last = self.levels.len() - 1;
        &mut self.levels[last]
    }

    fn depth(&self) -> usize {
        self.levels.len() - 1
    }

    fn feed(&mut self, token: Token<'_>, line: usize) -> Result<(), Exceeded> {
        self.else_if(token, line)?;
        self.prefix(token, line)?;
        self.closure(token, line)?;
        self.angle_and_delimiter(token, line)
    }

    fn else_if(&mut self, token: Token<'_>, line: usize) -> Result<(), Exceeded> {
        if token == Token::CloseBrace {
            // The closing level is about to be dropped; the enclosing level
            // learns that a block just ended.
            return Ok(());
        }
        let continues = token == Token::Ident("else") && self.top().after_brace;
        let level = self.top();
        let starts_statement = matches!(token, Token::Ident(word) if word != "else");
        if (level.after_brace && starts_statement) || token == Token::Semi {
            level.else_ifs = 0;
        }
        level.after_brace = false;
        if level.pending_else {
            level.pending_else = false;
            if token == Token::Ident("if") {
                level.else_ifs += 1;
                if level.else_ifs > Budget::ElseIf.limit() {
                    return Err(Exceeded {
                        budget: Budget::ElseIf,
                        line,
                    });
                }
            } else {
                level.else_ifs = 0;
            }
        }
        if continues {
            level.pending_else = true;
        }
        Ok(())
    }

    fn prefix(&mut self, token: Token<'_>, line: usize) -> Result<(), Exceeded> {
        if matches!(
            token,
            Token::Bang | Token::Minus | Token::Star | Token::Amp | Token::AmpAmp
        ) {
            self.prefix_run += 1;
            if self.prefix_run > Budget::PrefixOperators.limit() {
                return Err(Exceeded {
                    budget: Budget::PrefixOperators,
                    line,
                });
            }
        } else if token != Token::Lifetime {
            self.prefix_run = 0;
        }
        Ok(())
    }

    fn closure(&mut self, token: Token<'_>, line: usize) -> Result<(), Exceeded> {
        let depth = self.depth();
        if let Some(head_depth) = self.closure_head_depth {
            if depth < head_depth {
                // The level that held the head closed before its second bar.
                self.closure_head_depth = None;
                self.closure_pending_end = false;
                self.closure_chain = 0;
            } else if token == Token::Pipe && depth == head_depth {
                self.closure_head_depth = None;
                self.closure_pending_end = true;
                return Ok(());
            } else {
                return Ok(());
            }
        }
        match token {
            Token::PipePipe | Token::Pipe => {
                self.closure_chain = if self.closure_pending_end {
                    self.closure_chain + 1
                } else {
                    1
                };
                if self.closure_chain > Budget::ClosureBars.limit() {
                    return Err(Exceeded {
                        budget: Budget::ClosureBars,
                        line,
                    });
                }
                if token == Token::Pipe {
                    self.closure_head_depth = Some(depth);
                    self.closure_pending_end = false;
                } else {
                    self.closure_pending_end = true;
                }
            }
            Token::Ident("move") if self.closure_pending_end => {}
            _ => {
                self.closure_pending_end = false;
                self.closure_chain = 0;
            }
        }
        Ok(())
    }

    fn angle_and_delimiter(&mut self, token: Token<'_>, line: usize) -> Result<(), Exceeded> {
        match token {
            Token::Open(_) => {
                self.levels.push(Level::default());
                if self.depth() > Budget::Delimiters.limit() {
                    return Err(Exceeded {
                        budget: Budget::Delimiters,
                        line,
                    });
                }
            }
            Token::CloseBrace | Token::CloseOther => {
                if self.levels.len() > 1 {
                    self.levels.pop();
                    if token == Token::CloseBrace {
                        let parent = self.top();
                        parent.after_brace = true;
                        parent.angles = 0;
                    }
                }
            }
            Token::Lt => {
                let level = self.top();
                level.angles += 1;
                if level.angles > Budget::Angles.limit() {
                    return Err(Exceeded {
                        budget: Budget::Angles,
                        line,
                    });
                }
            }
            Token::Gt => {
                let level = self.top();
                level.angles = level.angles.saturating_sub(1);
            }
            Token::Semi | Token::AmpAmp | Token::PipePipe => self.top().angles = 0,
            _ => {}
        }
        Ok(())
    }
}

fn is_ident_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic() || byte >= 0x80
}

fn is_ident_continue(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric() || byte >= 0x80
}

/// A cursor over the text that counts lines as it moves.
struct Cursor<'a> {
    text: &'a str,
    bytes: &'a [u8],
    at: usize,
    line: usize,
}

impl<'a> Cursor<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            bytes: text.as_bytes(),
            at: 0,
            line: 1,
        }
    }

    fn peek(&self, ahead: usize) -> Option<u8> {
        self.bytes.get(self.at + ahead).copied()
    }

    fn bump(&mut self) {
        if self.peek(0) == Some(b'\n') {
            self.line += 1;
        }
        self.at += 1;
    }

    fn bump_by(&mut self, count: usize) {
        for _ in 0..count {
            self.bump();
        }
    }

    /// The tokens the comment at the cursor stands for: a doc comment is an
    /// attribute, `#[doc = ".."]` (six tokens) or `#![doc = ".."]` (seven).
    fn doc_comment_tokens(&self) -> usize {
        match (self.peek(1), self.peek(2), self.peek(3)) {
            // `///` but not `////`; `/**` but not `/***` or `/**/`.
            (Some(b'/'), Some(b'/'), next) if next != Some(b'/') => 6,
            (Some(b'*'), Some(b'*'), next) if next != Some(b'*') && next != Some(b'/') => 6,
            (Some(b'/' | b'*'), Some(b'!'), _) => 7,
            _ => 0,
        }
    }

    fn skip_line_comment(&mut self) {
        while self.peek(0).is_some_and(|byte| byte != b'\n') {
            self.bump();
        }
    }

    /// Skip a block comment; the cursor is on the opening `/*`.
    fn skip_block_comment(&mut self) {
        self.bump_by(2);
        let mut depth = 1_usize;
        while depth > 0 && self.peek(0).is_some() {
            if self.peek(0) == Some(b'/') && self.peek(1) == Some(b'*') {
                depth += 1;
                self.bump_by(2);
            } else if self.peek(0) == Some(b'*') && self.peek(1) == Some(b'/') {
                depth -= 1;
                self.bump_by(2);
            } else {
                self.bump();
            }
        }
    }

    /// Skip a quoted literal body; the cursor is just after the opening quote.
    fn skip_quoted(&mut self, quote: u8) {
        while let Some(byte) = self.peek(0) {
            self.bump();
            if byte == b'\\' {
                if self.peek(0).is_some() {
                    self.bump();
                }
            } else if byte == quote {
                return;
            }
        }
    }

    /// Skip a raw string; the cursor is on the first `#` or the quote, with
    /// `hashes` already counted and an opening quote known to follow them.
    fn skip_raw_string(&mut self, hashes: usize) {
        self.bump_by(hashes + 1);
        while let Some(byte) = self.peek(0) {
            self.bump();
            if byte == b'"' {
                let closing = (0..hashes).all(|offset| self.peek(offset) == Some(b'#'));
                if closing {
                    self.bump_by(hashes);
                    return;
                }
            }
        }
    }

    /// The number of `#` bytes starting `ahead` bytes from the cursor.
    fn hashes_at(&self, ahead: usize) -> usize {
        let mut count = 0;
        while self.peek(ahead + count) == Some(b'#') {
            count += 1;
        }
        count
    }

    /// Handle a `'`: a character literal is skipped and a lifetime is
    /// consumed. Returns true for a lifetime.
    fn skip_quote_or_lifetime(&mut self) -> bool {
        self.bump();
        match self.peek(0) {
            Some(b'\\') => {
                self.skip_quoted(b'\'');
                false
            }
            Some(first) => {
                let width = match first {
                    0xF0.. => 4,
                    0xE0.. => 3,
                    0xC0.. => 2,
                    _ => 1,
                };
                if self.peek(width) == Some(b'\'') {
                    self.bump_by(width + 1);
                    false
                } else {
                    while self.peek(0).is_some_and(is_ident_continue) {
                        self.bump();
                    }
                    true
                }
            }
            None => false,
        }
    }

    fn ident(&mut self) -> &'a str {
        let start = self.at;
        while self.peek(0).is_some_and(is_ident_continue) {
            self.bump();
        }
        &self.text[start..self.at]
    }
}

/// What the pre-scan of one admitted file counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Scan {
    /// The tokens of the file.
    pub(crate) tokens: usize,
    /// The highest value the recursion counter reached.
    pub(crate) recursion: usize,
}

/// Scan `text` and refuse it when a counter passes its budget or it holds more
/// than `max_tokens` tokens. Returns what the scan counted.
///
/// # Errors
///
/// Returns a [`LoadError`] with [`LoadErrorKind::Budget`] naming `file`, the
/// budget and the line where the count went over.
pub(crate) fn prescan(file: &RelPath, text: &str, max_tokens: usize) -> Result<Scan, LoadError> {
    let mut cursor = Cursor::new(text);
    let mut counters = Counters::new();
    let mut recursion = Recursion::new();
    let mut tokens = 0_usize;
    let over = |budget: Budget, limit: usize, line: usize| {
        LoadError::new(file, Some(line), LoadErrorKind::Budget { budget, limit })
    };
    let mut count = |added: usize, line: usize| {
        tokens += added;
        if tokens > max_tokens {
            Err(over(Budget::FileTokens, max_tokens, line))
        } else {
            Ok(())
        }
    };
    while let Some(byte) = cursor.peek(0) {
        let line = cursor.line;
        let start = cursor.at;
        let token = match byte {
            b if b.is_ascii_whitespace() => {
                cursor.bump();
                continue;
            }
            b'/' => match cursor.peek(1) {
                Some(b'/') => {
                    count(cursor.doc_comment_tokens(), line)?;
                    cursor.skip_line_comment();
                    continue;
                }
                Some(b'*') => {
                    count(cursor.doc_comment_tokens(), line)?;
                    cursor.skip_block_comment();
                    continue;
                }
                _ => {
                    cursor.bump();
                    Token::Operator
                }
            },
            b'"' => {
                cursor.bump();
                cursor.skip_quoted(b'"');
                Token::Literal
            }
            b'\'' => {
                if cursor.skip_quote_or_lifetime() {
                    Token::Lifetime
                } else {
                    Token::Literal
                }
            }
            b if is_ident_start(b) => word(&mut cursor),
            b'0'..=b'9' => {
                if number(&mut cursor) {
                    Token::Float
                } else {
                    Token::Literal
                }
            }
            b'(' | b'[' | b'{' => {
                cursor.bump();
                Token::Open(match byte {
                    b'(' => Delimiter::Paren,
                    b'[' => Delimiter::Bracket,
                    _ => Delimiter::Brace,
                })
            }
            b'}' => {
                cursor.bump();
                Token::CloseBrace
            }
            b')' | b']' => {
                cursor.bump();
                Token::CloseOther
            }
            _ => punctuation(&mut cursor, byte),
        };
        let added = lexed_tokens(token, &text.as_bytes()[start..cursor.at]);
        count(added, line)?;
        counters
            .feed(token, line)
            .map_err(|exceeded| over(exceeded.budget, exceeded.budget.limit(), exceeded.line))?;
        if recursion.feed(token, cursor.at - start) > Budget::Recursion.limit() {
            return Err(over(Budget::Recursion, Budget::Recursion.limit(), line));
        }
    }
    Ok(Scan {
        tokens,
        recursion: recursion.peak(),
    })
}

/// The tokens the parser's lexer reads for `token`, whose text is `lexed`:
/// two for a lifetime (a quote and a name), one per character of
/// punctuation, and one for anything else, a literal of any length included.
fn lexed_tokens(token: Token<'_>, lexed: &[u8]) -> usize {
    match token {
        Token::Lifetime => 2,
        Token::Bang
        | Token::Minus
        | Token::Star
        | Token::Amp
        | Token::AmpAmp
        | Token::Pipe
        | Token::PipePipe
        | Token::Lt
        | Token::Gt
        | Token::Semi
        | Token::Comma
        | Token::FatArrow
        | Token::Colon
        | Token::PathSep
        | Token::Hash
        | Token::Operator
        | Token::Other => lexed.len(),
        Token::Open(_)
        | Token::CloseBrace
        | Token::CloseOther
        | Token::Ident(_)
        | Token::Literal
        | Token::Float => 1,
    }
}

/// Lex one punctuation token; the cursor is on `byte`.
fn punctuation<'a>(cursor: &mut Cursor<'_>, byte: u8) -> Token<'a> {
    let next = cursor.peek(1);
    let (token, width) = match (byte, next) {
        (b'&', Some(b'&')) => (Token::AmpAmp, 2),
        (b'|', Some(b'|')) => (Token::PipePipe, 2),
        (b'=', Some(b'>')) => (Token::FatArrow, 2),
        (b':', Some(b':')) => (Token::PathSep, 2),
        (b'.', Some(b'.')) => (
            Token::Operator,
            if matches!(cursor.peek(2), Some(b'=' | b'.')) {
                3
            } else {
                2
            },
        ),
        // Compound operators that are not prefix operators or brackets.
        (b'&' | b'*' | b'-' | b'!' | b'|' | b'<', Some(b'=')) | (b'-', Some(b'>')) => {
            (Token::Operator, 2)
        }
        (b'&', _) => (Token::Amp, 1),
        (b'|', _) => (Token::Pipe, 1),
        (b'!', _) => (Token::Bang, 1),
        (b'-', _) => (Token::Minus, 1),
        (b'*', _) => (Token::Star, 1),
        (b'<', _) => (Token::Lt, 1),
        (b'>', _) => (Token::Gt, 1),
        (b';', _) => (Token::Semi, 1),
        (b',', _) => (Token::Comma, 1),
        (b':', _) => (Token::Colon, 1),
        (b'#', _) => (Token::Hash, 1),
        (b'+' | b'/' | b'%' | b'^' | b'=' | b'?' | b'@' | b'.', _) => (Token::Operator, 1),
        _ => (Token::Other, 1),
    };
    cursor.bump_by(width);
    token
}

/// Lex an identifier, a keyword, or a literal that starts with a letter: a
/// prefixed string, a raw string or a raw identifier. The cursor is on the
/// first identifier character.
fn word<'a>(cursor: &mut Cursor<'a>) -> Token<'a> {
    let start = cursor.at;
    let word = cursor.ident();
    match (word, cursor.peek(0)) {
        ("b" | "c", Some(b'"')) => {
            cursor.bump();
            cursor.skip_quoted(b'"');
            Token::Literal
        }
        ("b", Some(b'\'')) => {
            cursor.bump();
            cursor.skip_quoted(b'\'');
            Token::Literal
        }
        ("r" | "br" | "cr", Some(b'"' | b'#')) => {
            let hashes = cursor.hashes_at(0);
            if cursor.peek(hashes) == Some(b'"') {
                cursor.skip_raw_string(hashes);
                Token::Literal
            } else if word == "r" && hashes == 1 {
                // A raw identifier such as `r#type`.
                cursor.bump();
                Token::Ident(cursor.ident())
            } else {
                Token::Ident(&cursor.text[start..cursor.at])
            }
        }
        _ => Token::Ident(word),
    }
}

/// Skip a numeric literal and return true when it holds a `.`. `1.0` and
/// `1e5` are one literal; a range `1..2`, a method call `1.max(2)` and the
/// second `.` of `0.0.0` end it before the dot.
fn number(cursor: &mut Cursor<'_>) -> bool {
    let mut dotted = false;
    while cursor
        .peek(0)
        .is_some_and(|byte| is_ident_continue(byte) || byte == b'.')
    {
        if cursor.peek(0) == Some(b'.') {
            if dotted || !cursor.peek(1).is_some_and(|byte| byte.is_ascii_digit()) {
                break;
            }
            dotted = true;
        }
        cursor.bump();
    }
    dotted
}
