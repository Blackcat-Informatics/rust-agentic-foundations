// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The block reader of the frontmatter subset.
//!
//! The reader works on lines one at a time and keeps nothing of the text but
//! the values it builds, so memory grows with the parsed result and a refusal
//! costs next to nothing.

use std::collections::HashSet;

use crate::error::{ErrorKind, ParseError};
use crate::frontmatter::scalar::{Cursor, Fail, Inline, inline_value, plain_refusal};
use crate::frontmatter::value::{
    Comments, Entry, Frontmatter, ItemValue, List, ListForm, ListItem, Map, Scalar, ScalarStyle,
    Value, is_key_char,
};
use crate::limits::{MAX_TEXT_BYTES, MAX_YAML_DEPTH};

/// The line ending a document uses on its fence lines and frontmatter lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineEnding {
    /// `\n`.
    Lf,
    /// `\r\n`.
    CrLf,
}

impl LineEnding {
    /// The text of the line ending.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
        }
    }
}

/// The result of reading the head of a document.
pub(crate) struct Head {
    /// The frontmatter.
    pub(crate) frontmatter: Frontmatter,
    /// The line ending of the fences and the frontmatter lines.
    pub(crate) eol: LineEnding,
    /// Whether the closing fence line ends with a line ending.
    pub(crate) closing_newline: bool,
    /// The byte offset where the body starts.
    pub(crate) body_start: usize,
    /// The 1-based line where the body starts.
    pub(crate) body_line: usize,
}

/// One frontmatter line, without its line ending.
#[derive(Debug, Clone, Copy)]
struct Line<'a> {
    text: &'a str,
    number: usize,
    offset: usize,
    indent: usize,
}

impl<'a> Line<'a> {
    /// The text after the indentation.
    fn content(&self) -> &'a str {
        &self.text[self.indent..]
    }

    /// A blank line or a full-line comment.
    fn is_trivia(&self) -> bool {
        let content = self.content();
        content.is_empty() || content.starts_with('#')
    }

    /// The error for a failure at byte `index` of the line.
    fn error(&self, (kind, index): Fail) -> ParseError {
        self.error_at(kind, index)
    }

    fn error_at(&self, kind: ErrorKind, index: usize) -> ParseError {
        let column = self
            .text
            .char_indices()
            .take_while(|(at, _)| *at < index)
            .count()
            + 1;
        ParseError::at(kind, self.number, column, self.offset + index)
    }
}

/// Split a line from a trailing carriage return.
fn split_cr(line: &str) -> (&str, bool) {
    match line.strip_suffix('\r') {
        Some(stripped) => (stripped, true),
        None => (line, false),
    }
}

/// Read the fences and the frontmatter of `text`.
pub(crate) fn read_head(text: &str) -> Result<Head, ParseError> {
    if text.len() > MAX_TEXT_BYTES {
        let line = 1 + text[..text.floor_char_boundary(MAX_TEXT_BYTES)]
            .matches('\n')
            .count();
        return Err(ParseError::at_offset(
            ErrorKind::TooLarge {
                len: text.len(),
                max: MAX_TEXT_BYTES,
            },
            line,
            MAX_TEXT_BYTES,
        ));
    }
    let eol = if text.starts_with("---\r\n") {
        LineEnding::CrLf
    } else if text.starts_with("---\n") {
        LineEnding::Lf
    } else if text == "---" {
        return Err(ParseError::at(ErrorKind::UnclosedFence, 1, 1, 0));
    } else {
        return Err(ParseError::at(ErrorKind::MissingFence, 1, 1, 0));
    };
    let front_start = 3 + eol.as_str().len();

    // Find the closing fence.
    let mut position = front_start;
    let mut number = 2;
    let (fence_start, closing_newline, body_start) = loop {
        let rest = &text[position..];
        if rest.is_empty() {
            return Err(ParseError::at(ErrorKind::UnclosedFence, 1, 1, 0));
        }
        let (raw, next, had_newline) = match rest.find('\n') {
            Some(index) => (&rest[..index], position + index + 1, true),
            None => (rest, text.len(), false),
        };
        let (line, had_cr) = split_cr(raw);
        if line == "---" {
            let mismatch = if had_newline {
                had_cr != (eol == LineEnding::CrLf)
            } else {
                had_cr
            };
            if mismatch {
                return Err(ParseError::at(
                    ErrorKind::MixedLineEndings,
                    number,
                    4,
                    position + 3,
                ));
            }
            break (position, had_newline, next);
        }
        position = next;
        number += 1;
    };

    let mut reader = Reader {
        text,
        pos: front_start,
        end: fence_start,
        number: 2,
        eol,
        peeked: None,
        pending: Vec::new(),
    };
    let map = reader.top_level(number, fence_start)?;
    let trailing_comments = std::mem::take(&mut reader.pending);
    Ok(Head {
        frontmatter: Frontmatter {
            map,
            trailing_comments,
        },
        eol,
        closing_newline,
        body_start,
        body_line: number + 1,
    })
}

struct Reader<'a> {
    text: &'a str,
    /// Byte offset of the next unread line.
    pos: usize,
    /// Byte offset of the closing fence line.
    end: usize,
    /// Number of the next unread line.
    number: usize,
    eol: LineEnding,
    peeked: Option<Line<'a>>,
    /// Comment and blank lines read but not yet attached to an entry.
    pending: Vec<String>,
}

impl<'a> Reader<'a> {
    /// Read the next raw line of the frontmatter.
    fn read_line(&mut self) -> Result<Option<Line<'a>>, ParseError> {
        if self.pos >= self.end {
            return Ok(None);
        }
        let region = &self.text[self.pos..self.end];
        let number = self.number;
        let offset = self.pos;
        let mismatch = |index: usize| {
            ParseError::at(
                ErrorKind::MixedLineEndings,
                number,
                index + 1,
                offset + index,
            )
        };
        let (raw, advance) = match region.find('\n') {
            Some(index) => (&region[..index], index + 1),
            None => return Err(mismatch(region.chars().count())),
        };
        let (text, had_cr) = split_cr(raw);
        if had_cr != (self.eol == LineEnding::CrLf) {
            return Err(mismatch(text.chars().count()));
        }
        let line = Line {
            text,
            number,
            offset,
            indent: text.len() - text.trim_start_matches(' ').len(),
        };
        if let Some((index, _)) = text.char_indices().find(|(_, c)| c.is_control()) {
            return Err(line.error_at(ErrorKind::ControlCharacter, index));
        }
        if text.ends_with(' ') {
            return Err(line.error_at(ErrorKind::TrailingWhitespace, text.len() - 1));
        }
        self.pos += advance;
        self.number += 1;
        Ok(Some(line))
    }

    /// The next line, without consuming it.
    fn peek(&mut self) -> Result<Option<Line<'a>>, ParseError> {
        if self.peeked.is_none() {
            self.peeked = self.read_line()?;
        }
        Ok(self.peeked)
    }

    /// Consume the line [`Self::peek`] returned.
    fn consume(&mut self) {
        self.peeked = None;
    }

    /// Move blank lines and full-line comments to the pending comments, and
    /// return the next line of content.
    fn next_content(&mut self) -> Result<Option<Line<'a>>, ParseError> {
        loop {
            match self.peek()? {
                Some(line) if line.is_trivia() => {
                    self.pending.push(line.text.to_string());
                    self.consume();
                }
                Some(line) => {
                    if line.indent % 2 != 0 {
                        return Err(line.error_at(ErrorKind::Indentation, line.indent - 1));
                    }
                    return Ok(Some(line));
                }
                None => return Ok(None),
            }
        }
    }

    /// The top-level mapping. `fence_line` and `fence_offset` locate the
    /// closing fence for a frontmatter with no content.
    fn top_level(&mut self, fence_line: usize, fence_offset: usize) -> Result<Map, ParseError> {
        let Some(first) = self.next_content()? else {
            return Err(ParseError::at(
                ErrorKind::NotAMapping,
                fence_line,
                1,
                fence_offset,
            ));
        };
        if first.indent != 0 {
            return Err(first.error_at(ErrorKind::Indentation, 0));
        }
        let content = first.content();
        let dash_item = content == "-" || content.starts_with("- ");
        if dash_item || content.starts_with(['[', '{']) || !content.contains(':') {
            return Err(first.error_at(ErrorKind::NotAMapping, 0));
        }
        self.map(0, 1, None)
    }

    /// A block mapping whose keys sit at column `indent`, at nesting `level`.
    /// `first` is an entry already taken from a `- ` line, with the column
    /// where its key starts.
    fn map(
        &mut self,
        indent: usize,
        level: usize,
        mut first: Option<Line<'a>>,
    ) -> Result<Map, ParseError> {
        let mut entries: Vec<Entry> = Vec::new();
        let mut seen: HashSet<&'a str> = HashSet::new();
        loop {
            let line = if let Some(line) = first.take() {
                line
            } else {
                let Some(line) = self.next_content()? else {
                    break;
                };
                if line.indent < indent {
                    break;
                }
                if line.indent > indent {
                    return Err(line.error_at(ErrorKind::Indentation, indent));
                }
                self.consume();
                line
            };
            let start = indent;
            let (key, entry) = self.entry(line, start, indent, level)?;
            if !seen.insert(key) {
                return Err(line.error_at(
                    ErrorKind::DuplicateKey {
                        key: key.to_string(),
                    },
                    start,
                ));
            }
            entries.push(entry);
        }
        Ok(map_of(entries))
    }

    /// One `key: value` entry whose key starts at byte `start` of `line`.
    /// `indent` is the column of the mapping's keys and `level` its nesting.
    fn entry(
        &mut self,
        line: Line<'a>,
        start: usize,
        indent: usize,
        level: usize,
    ) -> Result<(&'a str, Entry), ParseError> {
        let leading = std::mem::take(&mut self.pending);
        let mut cur = Cursor::new(line.text, start).map_err(|fail| line.error(fail))?;
        let key = read_key(&mut cur).map_err(|fail| line.error(fail))?;
        let (value, trailing) = if cur.rest().is_empty() {
            (None, None)
        } else {
            inline_after_colon(&mut cur, key, level, &line)?
        };
        let value = match value {
            Some(value) => value,
            None => self.block_value(key, indent, level)?,
        };
        Ok((
            key,
            Entry {
                key: key.to_string(),
                value,
                line: line.number,
                comments: Comments { leading, trailing },
            },
        ))
    }

    /// The value of `key:` with nothing after the colon: a block mapping, a
    /// block list, or the empty scalar when no deeper line follows.
    fn block_value(&mut self, key: &str, indent: usize, level: usize) -> Result<Value, ParseError> {
        let Some(next) = self.next_content()? else {
            return Ok(empty_scalar());
        };
        if next.indent <= indent {
            return Ok(empty_scalar());
        }
        if next.indent != indent + 2 {
            return Err(next.error_at(ErrorKind::Indentation, indent + 2));
        }
        if level + 1 > MAX_YAML_DEPTH {
            return Err(next.error_at(
                ErrorKind::DepthExceeded {
                    max: MAX_YAML_DEPTH,
                },
                next.indent,
            ));
        }
        let content = next.content();
        if content == "-" || content.starts_with("- ") {
            self.list(key, indent + 2, level + 1).map(Value::List)
        } else {
            self.consume();
            // The mapping reader takes the first entry from the line already
            // consumed, so peek state stays consistent.
            self.map(indent + 2, level + 1, Some(next)).map(Value::Map)
        }
    }

    /// A block list whose dashes sit at column `indent`, at nesting `level`.
    fn list(&mut self, owner: &str, indent: usize, level: usize) -> Result<List, ParseError> {
        let mut items: Vec<ListItem> = Vec::new();
        let mut maps = false;
        while let Some(line) = self.next_content()? {
            if line.indent < indent {
                break;
            }
            if line.indent > indent {
                return Err(line.error_at(ErrorKind::Indentation, indent));
            }
            let content = line.content();
            if content != "-" && !content.starts_with("- ") {
                return Err(line.error_at(ErrorKind::ExpectedItem, indent));
            }
            self.consume();
            let leading = std::mem::take(&mut self.pending);
            let after = indent + 2;
            if content == "-" {
                return Err(line.error_at(ErrorKind::EmptyItem, indent));
            }
            let mut cur = Cursor::new(line.text, after).map_err(|fail| line.error(fail))?;
            if cur.peek() == Some(' ') {
                return Err(line.error_at(ErrorKind::Spacing, after));
            }
            if cur.rest() == "-" || cur.rest().starts_with("- ") {
                return Err(line.error_at(ErrorKind::NestedSequence, after));
            }
            let is_map = looks_like_entry(cur.rest());
            if !items.is_empty() && is_map != maps {
                return Err(line.error_at(ErrorKind::MixedList, indent));
            }
            maps = is_map;
            let item = if is_map {
                if level + 1 > MAX_YAML_DEPTH {
                    return Err(line.error_at(
                        ErrorKind::DepthExceeded {
                            max: MAX_YAML_DEPTH,
                        },
                        after,
                    ));
                }
                let map = self.map(after, level + 1, Some(line))?;
                ListItem {
                    value: ItemValue::Map(map),
                    line: line.number,
                    comments: Comments {
                        leading,
                        trailing: None,
                    },
                }
            } else {
                let (inline, trailing) =
                    inline_value(&mut cur, owner, level, true).map_err(|fail| line.error(fail))?;
                let scalar = match inline {
                    Inline::Scalar(scalar) => scalar,
                    Inline::Flow(_) => {
                        return Err(line.error_at(ErrorKind::NestedFlow, after));
                    }
                };
                ListItem {
                    value: ItemValue::Scalar(scalar),
                    line: line.number,
                    comments: Comments { leading, trailing },
                }
            };
            items.push(item);
        }
        Ok(list_of(ListForm::Block, items))
    }
}

/// Whether `text` starts a `key:` entry: key characters, then a colon that
/// ends the text or is followed by a space. A merge key counts, so that the
/// entry reader can refuse it by name.
fn looks_like_entry(text: &str) -> bool {
    if text.starts_with("<<:") {
        return true;
    }
    let key_len = match text.find(|c| !is_key_char(c)) {
        Some(index) => index,
        None => text.len(),
    };
    if key_len == 0 {
        return false;
    }
    let after = &text[key_len..];
    after == ":" || after.starts_with(": ")
}

/// Read a key and its colon. Leaves the cursor after the colon.
fn read_key<'a>(cur: &mut Cursor<'a>) -> Result<&'a str, Fail> {
    let start = cur.pos();
    let rest = cur.rest();
    if rest.starts_with("<<") {
        return Err((ErrorKind::MergeKey, start));
    }
    if rest.starts_with(['&', '*', '!'])
        && let Some((kind, _)) = plain_refusal(rest)
    {
        return Err((kind, start));
    }
    let key = cur.take_while(is_key_char);
    if !key.is_empty() && cur.peek() == Some(':') {
        cur.bump();
        return Ok(key);
    }
    // Not a key: name the text before the first colon that would end one.
    let colon = rest
        .match_indices(':')
        .find(|(index, _)| {
            let after = &rest[index + 1..];
            after.is_empty() || after.starts_with(' ')
        })
        .map(|(index, _)| index);
    match colon {
        Some(index) => Err((
            ErrorKind::InvalidKey {
                key: rest[..index].to_string(),
            },
            start,
        )),
        None => Err((ErrorKind::ExpectedKey, start)),
    }
}

/// The part of an entry line after `key:`. Returns the inline value, or
/// `None` when the value is a block (or empty), and the trailing comment.
fn inline_after_colon<'a>(
    cur: &mut Cursor<'a>,
    key: &str,
    level: usize,
    line: &Line<'a>,
) -> Result<(Option<Value>, Option<String>), ParseError> {
    if !cur.rest().starts_with(' ') {
        return Err(line.error_at(ErrorKind::ExpectedKey, cur.pos()));
    }
    cur.bump();
    match cur.peek() {
        Some(' ') => {
            if cur.rest().trim_start_matches(' ').starts_with('#') {
                let comment = format!(" {}", cur.rest());
                return Ok((None, Some(comment)));
            }
            Err(line.error_at(ErrorKind::Spacing, cur.pos()))
        }
        Some('#') => Ok((None, Some(format!(" {}", cur.rest())))),
        Some(_) => {
            let (inline, trailing) =
                inline_value(cur, key, level, false).map_err(|fail| line.error(fail))?;
            let value = match inline {
                Inline::Scalar(scalar) => Value::Scalar(scalar),
                Inline::Flow(scalars) => flow_list(scalars, line.number),
            };
            Ok((Some(value), trailing))
        }
        None => Err(line.error_at(ErrorKind::TrailingWhitespace, cur.pos() - 1)),
    }
}

fn empty_scalar() -> Value {
    Value::Scalar(Scalar {
        text: String::new(),
        style: ScalarStyle::Plain,
    })
}

fn flow_list(scalars: Vec<Scalar>, line: usize) -> Value {
    let items = scalars
        .into_iter()
        .map(|scalar| ListItem {
            value: ItemValue::Scalar(scalar),
            line,
            comments: Comments::default(),
        })
        .collect();
    Value::List(list_of(ListForm::Flow, items))
}

fn list_of(form: ListForm, items: Vec<ListItem>) -> List {
    let depth = 1 + items
        .iter()
        .map(|item| item.value.depth())
        .fold(0, usize::max);
    List { form, items, depth }
}

fn map_of(entries: Vec<Entry>) -> Map {
    let depth = 1 + entries
        .iter()
        .map(|entry| entry.value.depth())
        .fold(0, usize::max);
    Map { entries, depth }
}
