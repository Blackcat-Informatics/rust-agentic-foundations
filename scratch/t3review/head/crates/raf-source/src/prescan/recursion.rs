// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The recursion counter of the pre-scan: one additive count of the nesting
//! that `syn` builds, on the parse thread, by recursion.
//!
//! Each delimiter level has its own count. A counted token adds to the count
//! of the level it is read in. The value of the counter is the sum of the
//! counts of every open level plus one for each open delimiter, so nesting
//! inside a group adds to the nesting around it. A level's count returns to
//! zero where every construct begun in it has ended (see [`Recursion::feed`]),
//! and a level that closes takes its count with it.

use super::{Delimiter, Token};

/// The keywords that open a construct the parser reads, or the syntax tree
/// holds, by recursion: an expression or a closure head, a pattern, or a type.
const COUNTED_KEYWORDS: [&str; 25] = [
    "as", "async", "become", "box", "break", "const", "dyn", "fn", "for", "if", "impl", "let",
    "loop", "match", "mod", "move", "mut", "ref", "return", "static", "try", "unsafe", "where",
    "while", "yield",
];

/// The identifiers after a closing brace that continue the construct the
/// group is part of, so the count of the level does not return to zero:
/// `S {} as T`, `if a {} else {}` and `for S {} in it {}`.
const CONTINUING_WORDS: [&str; 3] = ["as", "else", "in"];

/// Every keyword of the language, strict and reserved. An identifier that is
/// none of them, followed by `!`, names a macro.
const KEYWORDS: [&str; 52] = [
    "Self", "abstract", "as", "async", "await", "become", "box", "break", "const", "continue",
    "crate", "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if",
    "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub",
    "ref", "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// The keywords that end an operand, as a name does.
const OPERAND_KEYWORDS: [&str; 7] = ["Self", "await", "crate", "false", "self", "super", "true"];

/// The keywords of constructs that take a block. One of them between `match`
/// and the next `{` of its level leaves open whose block that `{` starts.
const BLOCK_OWNERS: [&str; 11] = [
    "async", "const", "for", "if", "loop", "match", "move", "static", "try", "unsafe", "while",
];

/// What one `::` of a `use` declaration counts. The index turns a `use` tree
/// into its imports on the caller's thread by recursion, about a kilobyte of
/// stack for each segment in a debug build, so a separator counts enough to
/// keep a `use` tree at the limit within a small part of a small stack.
pub(super) const USE_SEPARATOR_UNITS: usize = 16;

/// Whether a `match` of a level still waits for the `{` of its arms.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Scrutinee {
    /// No `match` is waiting.
    #[default]
    None,
    /// A `match` is waiting, and nothing since it can own a block.
    Clear,
    /// A `match` is waiting, but a construct that takes a block came after it,
    /// so the next `{` may be that construct's.
    Doubt,
}

/// What one character of `&` or `<` counts. A reference type and a list of
/// generic arguments cost the parser about twice the stack of the other
/// counted constructs in a debug build.
const DOUBLED_UNITS: usize = 2;

/// What kind of group a delimiter level is.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// Any group the parser reads token by token.
    #[default]
    Plain,
    /// The delimiters of a macro call, whose tokens the parser keeps as token
    /// trees and nests only at their groups.
    TokenTree,
    /// The arms of a `match`.
    Arms,
}

/// Constructs of a level whose commas do not end what came before them.
#[derive(Debug, Default, Clone, Copy)]
struct Lists {
    /// True between the two bars of a closure head (`|a, b|`), whose commas
    /// separate parameters inside a closure that is still open.
    bar: bool,
    /// True inside a `use` declaration, whose path segments nest.
    use_declaration: bool,
    /// True in a `where` clause, whose commas separate the predicates of an
    /// item that is still open, until the item's body or `;`.
    where_clause: bool,
}

/// The state of one delimiter level.
#[derive(Debug, Default, Clone, Copy)]
struct Level {
    /// The counted tokens read in this level since its count last returned
    /// to zero.
    count: usize,
    /// `<` read in this level and not yet matched by a `>`. While one is open
    /// a `,` may separate generic arguments, so it does not end the
    /// constructs before the `<`.
    angles: usize,
    /// What kind of group the level is.
    kind: Kind,
    /// Closure heads, `use` declarations and `where` clauses that are open.
    lists: Lists,
    /// True just after a `{ ... }` group closed in this level.
    after_brace: bool,
    /// True, in the arms of a `match`, from the start of an arm to its guard
    /// or its `=>`: the arm's pattern, whose alternatives the parser reads in a
    /// loop.
    in_pattern: bool,
    /// A `match` of this level waiting for its arms.
    scrutinee: Scrutinee,
}

/// A mark that one token leaves for the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    /// Nothing.
    None,
    /// `#` or `#!`: the next `[` opens an attribute.
    Attribute,
    /// A lifetime: the next `:` makes it a label.
    Lifetime,
}

/// What came just before a token.
#[derive(Debug, Clone, Copy)]
struct Before {
    /// The mark the previous token left.
    mark: Mark,
    /// A name, a literal or a closing `)` or `]`, which ends an operand.
    operand: bool,
    /// A macro's name and `!`, with an optional name after them: the next
    /// group is the macro's.
    macro_call: bool,
}

/// The recursion counter.
pub(super) struct Recursion {
    levels: Vec<Level>,
    /// The sum of the counts of the open levels plus the open delimiters.
    value: usize,
    /// The highest value so far.
    peak: usize,
    /// True after `#` or `#!`, so the next `[` opens an attribute.
    attribute: bool,
    /// True when the previous token was a lifetime, so a `:` makes it a label.
    after_lifetime: bool,
    /// True when the previous token ended an operand: a name, a literal or a
    /// closing `)` or `]`.
    after_operand: bool,
    /// How much of a macro call was just read: 1 after a name, 2 after its
    /// `!`, 3 after a name following the `!` (`macro_rules! name`).
    macro_call: u8,
}

impl Recursion {
    pub(super) fn new() -> Self {
        Self {
            levels: vec![Level::default()],
            value: 0,
            peak: 0,
            attribute: false,
            after_lifetime: false,
            after_operand: false,
            macro_call: 0,
        }
    }

    /// The highest value the counter reached.
    pub(super) fn peak(&self) -> usize {
        self.peak
    }

    fn top(&mut self) -> &mut Level {
        let last = self.levels.len() - 1;
        &mut self.levels[last]
    }

    /// Add `units` to the count of the open level.
    fn add(&mut self, units: usize) {
        self.top().count += units;
        self.value += units;
        self.peak = self.peak.max(self.value);
    }

    /// Return the count of the open level to zero. `statement` also ends the
    /// open `<`, closure head, `use` declaration, `where` clause, pattern and
    /// waiting `match` of the level.
    fn end(&mut self, statement: bool) {
        let level = self.top();
        let count = level.count;
        level.count = 0;
        if statement {
            level.angles = 0;
            level.lists = Lists::default();
            level.in_pattern = false;
            level.scrutinee = Scrutinee::None;
        }
        self.value -= count;
    }

    /// Open a level for a group that starts with `delimiter`.
    fn open(&mut self, delimiter: Delimiter, before: Before) {
        let level = self.top();
        let inherited = level.kind == Kind::TokenTree;
        let kind = if inherited || before.macro_call {
            Kind::TokenTree
        } else if delimiter == Delimiter::Brace
            && level.scrutinee == Scrutinee::Clear
            && before.operand
        {
            Kind::Arms
        } else {
            Kind::Plain
        };
        let use_declaration = level.lists.use_declaration;
        if delimiter == Delimiter::Brace {
            level.lists.where_clause = false;
            if level.scrutinee != Scrutinee::None {
                level.scrutinee = if kind == Kind::Arms {
                    Scrutinee::None
                } else {
                    Scrutinee::Doubt
                };
            }
        }
        if !(inherited
            || before.macro_call
            || before.mark == Mark::Attribute && delimiter == Delimiter::Bracket)
        {
            self.add(1);
        }
        self.levels.push(Level {
            kind,
            lists: Lists {
                use_declaration,
                ..Lists::default()
            },
            in_pattern: kind == Kind::Arms,
            ..Level::default()
        });
        self.value += 1;
        self.peak = self.peak.max(self.value);
    }

    /// Close the open level, if one is open.
    fn close(&mut self, brace: bool) {
        if self.levels.len() > 1 {
            let closed = self.levels.pop().map_or(0, |level| level.count);
            self.value -= closed + 1;
            if brace {
                self.top().after_brace = true;
            }
        }
    }

    /// Take what the previous tokens said about `token` and record what
    /// `token` says about the next one.
    fn before(&mut self, token: Token<'_>) -> Before {
        let macro_call = std::mem::take(&mut self.macro_call);
        self.macro_call = match token {
            Token::Ident(word) if !KEYWORDS.contains(&word) => {
                if macro_call == 2 {
                    3
                } else {
                    1
                }
            }
            Token::Bang if macro_call == 1 => 2,
            _ => 0,
        };
        let operand = std::mem::replace(
            &mut self.after_operand,
            match token {
                Token::Ident(word) => !KEYWORDS.contains(&word) || OPERAND_KEYWORDS.contains(&word),
                Token::Literal | Token::Float | Token::CloseOther => true,
                _ => false,
            },
        );
        let mark = if std::mem::take(&mut self.attribute) {
            Mark::Attribute
        } else if std::mem::take(&mut self.after_lifetime) {
            Mark::Lifetime
        } else {
            Mark::None
        };
        Before {
            mark,
            operand,
            macro_call: macro_call >= 2,
        }
    }

    /// Feed one token, whose text is `width` bytes long, and return the value
    /// of the counter after it.
    ///
    /// * Counted tokens: an operator (`!`, `-`, `*`, `&`, `|`, `<`, `>`, `+`,
    ///   `/`, `%`, `^`, `=`, `?`, `@`, `.`, a range, `->`, `&&`, `||` and the
    ///   compound forms) counts one for each of its characters, except a `!`
    ///   that follows `#`, and [`DOUBLED_UNITS`] for each `&` or `<` of `&`,
    ///   `&&` or `<`; a number with a `.` counts one (after a `.` it is
    ///   two tuple indices); each keyword of [`COUNTED_KEYWORDS`] counts one; a
    ///   label (a lifetime followed by `:`) counts one; `::` counts
    ///   [`USE_SEPARATOR_UNITS`] inside a `use` declaration; and the `(`, `[`
    ///   or `{` that opens a group counts one in the level around it, except
    ///   the `[` of an attribute and the delimiter of a macro call.
    /// * A macro call is a name that is not a keyword, `!`, an optional name
    ///   and a group. Inside that group nothing counts but the levels that
    ///   its groups open.
    /// * In the arms of a `match` (the first `{` of the `match`'s level, after
    ///   a name, a literal, `)` or `]`, with no construct that takes a block
    ///   between), a `|` of an arm's pattern counts nothing and ends the count
    ///   like a `,`.
    /// * A level's count returns to zero at `;` and `=>`, at a `,` unless a
    ///   `<` of the level, a closure head or a `where` clause before the
    ///   item's `{` or `;` is open, and at the token after a `}` that closed a
    ///   group in the level when that token is a literal, a lifetime, `#`, or
    ///   an identifier other than those of [`CONTINUING_WORDS`].
    pub(super) fn feed(&mut self, token: Token<'_>, width: usize) -> usize {
        let before = self.before(token);
        if self.top().kind == Kind::TokenTree {
            match token {
                Token::Open(delimiter) => self.open(delimiter, before),
                Token::CloseBrace | Token::CloseOther => self.close(false),
                _ => {}
            }
            return self.value;
        }
        let level = self.top();
        if std::mem::take(&mut level.after_brace) && ends_block_statement(token) {
            self.end(true);
            let level = self.top();
            level.in_pattern = level.kind == Kind::Arms;
        }
        let level = self.top();
        if level.scrutinee == Scrutinee::Clear && takes_a_block(token, before) {
            level.scrutinee = Scrutinee::Doubt;
        }
        match token {
            Token::Ident(word) => self.word(word),
            Token::Open(delimiter) => self.open(delimiter, before),
            Token::CloseBrace | Token::CloseOther => self.close(token == Token::CloseBrace),
            _ => self.punctuation(token, width, before),
        }
        self.value
    }

    /// Count a punctuation token or a literal.
    fn punctuation(&mut self, token: Token<'_>, width: usize, before: Before) {
        match token {
            Token::Semi | Token::FatArrow => self.end(true),
            Token::Comma => {
                let level = self.top();
                if level.angles == 0 && !level.lists.bar && !level.lists.where_clause {
                    level.in_pattern = level.kind == Kind::Arms;
                    self.end(false);
                }
            }
            Token::Lt => {
                self.top().angles += 1;
                self.add(DOUBLED_UNITS * width);
            }
            Token::Gt => {
                let level = self.top();
                level.angles = level.angles.saturating_sub(1);
                self.add(width);
            }
            Token::Pipe => {
                let level = self.top();
                if level.in_pattern {
                    self.end(false);
                } else {
                    level.lists.bar = !level.lists.bar;
                    self.add(width);
                }
            }
            Token::Hash => self.attribute = true,
            Token::Bang => {
                if before.mark == Mark::Attribute {
                    self.attribute = true;
                } else {
                    self.add(width);
                }
            }
            Token::Amp | Token::AmpAmp => self.add(DOUBLED_UNITS * width),
            Token::Minus | Token::Star | Token::PipePipe | Token::Operator => self.add(width),
            Token::Float => self.add(1),
            Token::Colon => {
                if before.mark == Mark::Lifetime {
                    self.add(1);
                }
            }
            Token::PathSep => {
                if self.top().lists.use_declaration {
                    self.add(USE_SEPARATOR_UNITS);
                }
            }
            Token::Lifetime => self.after_lifetime = true,
            Token::Ident(_)
            | Token::Open(_)
            | Token::CloseBrace
            | Token::CloseOther
            | Token::Literal
            | Token::Other => {}
        }
    }

    /// Count an identifier or a keyword.
    fn word(&mut self, word: &str) {
        let level = self.top();
        match word {
            "use" => level.lists.use_declaration = true,
            "where" => level.lists.where_clause = true,
            "if" => level.in_pattern = false,
            "match" => {
                level.scrutinee = if level.scrutinee == Scrutinee::None {
                    Scrutinee::Clear
                } else {
                    Scrutinee::Doubt
                };
            }
            _ => {}
        }
        if COUNTED_KEYWORDS.contains(&word) {
            self.add(1);
        }
    }
}

/// True when `token` can start a construct that takes a block, so that a `{`
/// after it may be that construct's and not the arms of a waiting `match`.
fn takes_a_block(token: Token<'_>, before: Before) -> bool {
    match token {
        Token::Ident(word) => BLOCK_OWNERS.contains(&word),
        Token::Pipe | Token::PipePipe => true,
        Token::Colon => before.mark == Mark::Lifetime,
        _ => false,
    }
}

/// True when `token`, read just after a `}` that closed a group, starts a new
/// statement or item rather than continuing the construct the group ended.
fn ends_block_statement(token: Token<'_>) -> bool {
    match token {
        Token::Ident(word) => !CONTINUING_WORDS.contains(&word),
        Token::Literal | Token::Float | Token::Lifetime | Token::Hash => true,
        _ => false,
    }
}
