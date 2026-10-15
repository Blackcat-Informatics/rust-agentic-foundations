// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The source of the `chunk-parser` crate at each of its five versions: the
//! library, its integration tests and its examples.

pub(crate) const V0_SRC_LIB: &str = r#"//! Reads `key=value` text into entries.

/// One entry of the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The text before the first `=`.
    pub key: String,
    /// The text after it.
    pub value: String,
}

/// Parses a complete text. A line that is not `key=value` is skipped.
///
/// ```
/// let entries = chunk_parser::parse("a=1\nb=2\n");
/// assert_eq!(entries.len(), 2);
/// ```
pub fn parse(input: &str) -> Vec<Entry> {
    input.lines().filter_map(pair).collect()
}

fn pair(text: &str) -> Option<Entry> {
    let (key, value) = text.split_once('=')?;
    if key.is_empty() {
        return None;
    }
    Some(Entry {
        key: key.to_string(),
        value: value.to_string(),
    })
}
"#;

pub(crate) const V0_TESTS_PARSER: &str = r#"use chunk_parser::parse;

#[test]
fn a_whole_text_is_parsed_in_order() {
    let entries = parse("a=1\nb=2\n");
    assert_eq!(entries.len(), 2);
    assert_eq!((entries[0].key.as_str(), entries[0].value.as_str()), ("a", "1"));
    assert_eq!((entries[1].key.as_str(), entries[1].value.as_str()), ("b", "2"));
}

#[test]
fn a_line_that_is_not_a_pair_is_skipped() {
    let entries = parse("a=1\nnot a pair\n=empty key\nb=\n");
    let keys: Vec<&str> = entries.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(keys, ["a", "b"]);
    assert_eq!(entries[1].value, "");
}
"#;

pub(crate) const V0_EXAMPLES_DUMP: &str = r#"fn main() {
    for entry in chunk_parser::parse("a=1\nb=2\n") {
        println!("{}={}", entry.key, entry.value);
    }
}
"#;

pub(crate) const V1_SRC_LIB: &str = r#"//! Reads `key=value` text into entries.

/// One entry of the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The text before the first `=`.
    pub key: String,
    /// The text after it.
    pub value: String,
}

/// Reads text that arrives in chunks.
#[derive(Debug, Default)]
pub struct Parser {
    pending: String,
    entries: Vec<Entry>,
}

impl Parser {
    /// A parser that has read nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes the next chunk, which may end anywhere.
    pub fn push(&mut self, chunk: &str) {
        self.pending.push_str(chunk);
        while let Some(end) = self.pending.find('\n') {
            let line: String = self.pending.drain(..=end).collect();
            if let Some(entry) = pair(line.trim_end_matches(['\n', '\r'])) {
                self.entries.push(entry);
            }
        }
    }

    /// Ends the input and returns the entries.
    #[must_use]
    pub fn finish(mut self) -> Vec<Entry> {
        let rest = std::mem::take(&mut self.pending);
        if let Some(entry) = pair(rest.trim_end_matches('\r')) {
            self.entries.push(entry);
        }
        self.entries
    }
}

/// Parses a complete text. A line that is not `key=value` is skipped.
///
/// ```
/// let entries = chunk_parser::parse("a=1\nb=2\n");
/// assert_eq!(entries.len(), 2);
/// ```
pub fn parse(input: &str) -> Vec<Entry> {
    let mut parser = Parser::new();
    parser.push(input);
    parser.finish()
}

fn pair(text: &str) -> Option<Entry> {
    let (key, value) = text.split_once('=')?;
    if key.is_empty() {
        return None;
    }
    Some(Entry {
        key: key.to_string(),
        value: value.to_string(),
    })
}
"#;

pub(crate) const V1_TESTS_PARSER: &str = r#"use chunk_parser::{parse, Parser};

pub(crate) const TEXT: &str = "a=1\nb=2\nc=3";

fn pairs(entries: &[chunk_parser::Entry]) -> Vec<(&str, &str)> {
    entries.iter().map(|e| (e.key.as_str(), e.value.as_str())).collect()
}

#[test]
fn a_whole_text_is_parsed_in_order() {
    let entries = parse("a=1\nb=2\n");
    assert_eq!(pairs(&entries), [("a", "1"), ("b", "2")]);
}

#[test]
fn a_line_that_is_not_a_pair_is_skipped() {
    let entries = parse("a=1\nnot a pair\n=empty key\nb=\n");
    assert_eq!(pairs(&entries), [("a", "1"), ("b", "")]);
}

#[test]
fn chunks_split_anywhere_give_the_same_entries() {
    let whole = parse(TEXT);
    for split in 0..=TEXT.len() {
        let mut parser = Parser::new();
        parser.push(&TEXT[..split]);
        parser.push(&TEXT[split..]);
        assert_eq!(parser.finish(), whole, "split at {split}");
    }
}

#[test]
fn a_last_line_without_a_line_ending_is_kept() {
    let mut parser = Parser::new();
    parser.push("a=1\nb=");
    assert_eq!(pairs(&parser.finish()), [("a", "1"), ("b", "")]);
}
"#;

pub(crate) const V1_EXAMPLES_DUMP: &str = r#"fn main() {
    for entry in chunk_parser::parse("a=1\nb=2\nc=3\n") {
        println!("{}={}", entry.key, entry.value);
    }
}
"#;

pub(crate) const V1_EXAMPLES_CHUNKS: &str = r#"fn main() {
    let mut parser = chunk_parser::Parser::new();
    for chunk in ["a=1\nb", "=2\nc=", "3\n"] {
        parser.push(chunk);
    }
    for entry in parser.finish() {
        println!("{}={}", entry.key, entry.value);
    }
}
"#;

pub(crate) const V2_SRC_LIB: &str = r#"//! Reads `key=value` text into entries.

use std::fmt;

/// One entry of the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The text before the first `=`.
    pub key: String,
    /// The text after it.
    pub value: String,
}

/// A line that is not `key=value`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// The number of the line, counted from 1.
    pub line: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {} is not key=value", self.line)
    }
}

impl std::error::Error for ParseError {}

/// Reads text that arrives in chunks.
#[derive(Debug, Default)]
pub struct Parser {
    pending: String,
    entries: Vec<Entry>,
    lines: usize,
    failed: Option<ParseError>,
}

impl Parser {
    /// A parser that has read nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes the next chunk, which may end anywhere.
    ///
    /// # Errors
    ///
    /// The first line that is not `key=value`; every later call returns it again.
    pub fn push(&mut self, chunk: &str) -> Result<(), ParseError> {
        if let Some(error) = &self.failed {
            return Err(error.clone());
        }
        self.pending.push_str(chunk);
        while let Some(end) = self.pending.find('\n') {
            let line: String = self.pending.drain(..=end).collect();
            self.take(line.trim_end_matches(['\n', '\r']))?;
        }
        Ok(())
    }

    /// Ends the input and returns the entries.
    ///
    /// # Errors
    ///
    /// The first line that is not `key=value`.
    pub fn finish(mut self) -> Result<Vec<Entry>, ParseError> {
        if let Some(error) = self.failed.take() {
            return Err(error);
        }
        let rest = std::mem::take(&mut self.pending);
        if !rest.is_empty() {
            self.take(rest.trim_end_matches('\r'))?;
        }
        Ok(self.entries)
    }

    fn take(&mut self, text: &str) -> Result<(), ParseError> {
        self.lines += 1;
        if text.is_empty() {
            return Ok(());
        }
        match pair(text) {
            Some(entry) => {
                self.entries.push(entry);
                Ok(())
            }
            None => {
                let error = ParseError { line: self.lines };
                self.failed = Some(error.clone());
                Err(error)
            }
        }
    }
}

/// Parses a complete text.
///
/// ```
/// let entries = chunk_parser::parse("a=1\nb=2\n")?;
/// assert_eq!(entries.len(), 2);
/// # Ok::<(), chunk_parser::ParseError>(())
/// ```
///
/// # Errors
///
/// The first line that is not `key=value`.
pub fn parse(input: &str) -> Result<Vec<Entry>, ParseError> {
    let mut parser = Parser::new();
    parser.push(input)?;
    parser.finish()
}

fn pair(text: &str) -> Option<Entry> {
    let (key, value) = text.split_once('=')?;
    if key.is_empty() {
        return None;
    }
    Some(Entry {
        key: key.to_string(),
        value: value.to_string(),
    })
}
"#;

pub(crate) const V2_TESTS_PARSER: &str = r#"use chunk_parser::{parse, ParseError, Parser};

pub(crate) const TEXT: &str = "a=1\n\nb=2\nc=3";

fn pairs(entries: &[chunk_parser::Entry]) -> Vec<(&str, &str)> {
    entries.iter().map(|e| (e.key.as_str(), e.value.as_str())).collect()
}

#[test]
fn a_whole_text_is_parsed_in_order() {
    let entries = parse("a=1\nb=2\n").expect("parses");
    assert_eq!(pairs(&entries), [("a", "1"), ("b", "2")]);
}

#[test]
fn chunks_split_anywhere_give_the_same_entries() {
    let whole = parse(TEXT).expect("parses");
    for split in 0..=TEXT.len() {
        let mut parser = Parser::new();
        parser.push(&TEXT[..split]).expect("first chunk");
        parser.push(&TEXT[split..]).expect("second chunk");
        assert_eq!(parser.finish().expect("finishes"), whole, "split at {split}");
    }
}

#[test]
fn a_last_line_without_a_line_ending_is_kept() {
    let mut parser = Parser::new();
    parser.push("a=1\nb=").expect("pushes");
    assert_eq!(pairs(&parser.finish().expect("finishes")), [("a", "1"), ("b", "")]);
}

#[test]
fn a_blank_line_is_not_an_entry_and_not_an_error() {
    let entries = parse("a=1\n\n\nb=2\n").expect("parses");
    assert_eq!(pairs(&entries), [("a", "1"), ("b", "2")]);
}

#[test]
fn a_line_that_is_not_a_pair_is_an_error_that_names_its_line() {
    assert_eq!(parse("a=1\nb=2\nnot a pair\nc=3\n"), Err(ParseError { line: 3 }));
    assert_eq!(parse("=no key\n"), Err(ParseError { line: 1 }));
}

#[test]
fn the_first_error_ends_the_parse() {
    let mut parser = Parser::new();
    let error = parser.push("a=1\nbad\nc=3\n").expect_err("fails");
    assert_eq!(error, ParseError { line: 2 });
    assert_eq!(parser.push("d=4\n"), Err(error.clone()));
    assert_eq!(parser.finish(), Err(error));
}
"#;

pub(crate) const V2_EXAMPLES_CHUNKS: &str = r#"fn main() -> Result<(), chunk_parser::ParseError> {
    let mut parser = chunk_parser::Parser::new();
    for chunk in ["a=1\nb", "=2\nc=", "3\n"] {
        parser.push(chunk)?;
    }
    for entry in parser.finish()? {
        println!("{}={}", entry.key, entry.value);
    }
    let mut broken = chunk_parser::Parser::new();
    if let Err(error) = broken.push("a=1\nnot a pair\n") {
        println!("{error}");
    }
    Ok(())
}
"#;

pub(crate) const V3_SRC_LIB: &str = r#"//! Reads `key=value` text into entries.

use std::fmt;

/// One entry of the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The text before the first `=`.
    pub key: String,
    /// The text after it.
    pub value: String,
}

/// A line that is not `key=value`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// The number of the line, counted from 1.
    pub line: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {} is not key=value", self.line)
    }
}

impl std::error::Error for ParseError {}

/// Reads text that arrives in chunks.
#[derive(Debug, Default)]
pub struct Parser {
    pending: String,
    lines: usize,
    failed: Option<ParseError>,
}

impl Parser {
    /// A parser that has read nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes the next chunk, which may end anywhere, and returns the entries
    /// whose lines it completed.
    ///
    /// ```
    /// let mut parser = chunk_parser::Parser::new();
    /// let first = parser.feed("a=1\nb")?;
    /// assert_eq!(first.len(), 1);
    /// let second = parser.feed("=2\n")?;
    /// assert_eq!(second[0].key, "b");
    /// # Ok::<(), chunk_parser::ParseError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The first line that is not `key=value`; every later call returns it again.
    pub fn feed(&mut self, chunk: &str) -> Result<Vec<Entry>, ParseError> {
        if let Some(error) = &self.failed {
            return Err(error.clone());
        }
        self.pending.push_str(chunk);
        let mut found = Vec::new();
        while let Some(end) = self.pending.find('\n') {
            let line: String = self.pending.drain(..=end).collect();
            if let Some(entry) = self.take(line.trim_end_matches(['\n', '\r']))? {
                found.push(entry);
            }
        }
        Ok(found)
    }

    /// Ends the input and returns the entry of a last line that has no line
    /// ending.
    ///
    /// # Errors
    ///
    /// The first line that is not `key=value`.
    pub fn close(mut self) -> Result<Vec<Entry>, ParseError> {
        if let Some(error) = self.failed.take() {
            return Err(error);
        }
        let rest = std::mem::take(&mut self.pending);
        if rest.is_empty() {
            return Ok(Vec::new());
        }
        Ok(self.take(rest.trim_end_matches('\r'))?.into_iter().collect())
    }

    fn take(&mut self, text: &str) -> Result<Option<Entry>, ParseError> {
        self.lines += 1;
        if text.is_empty() {
            return Ok(None);
        }
        match pair(text) {
            Some(entry) => Ok(Some(entry)),
            None => {
                let error = ParseError { line: self.lines };
                self.failed = Some(error.clone());
                Err(error)
            }
        }
    }
}

/// Parses a complete text.
///
/// ```
/// let entries = chunk_parser::parse("a=1\nb=2\n")?;
/// assert_eq!(entries.len(), 2);
/// # Ok::<(), chunk_parser::ParseError>(())
/// ```
///
/// # Errors
///
/// The first line that is not `key=value`.
pub fn parse(input: &str) -> Result<Vec<Entry>, ParseError> {
    let mut parser = Parser::new();
    let mut entries = parser.feed(input)?;
    entries.extend(parser.close()?);
    Ok(entries)
}

fn pair(text: &str) -> Option<Entry> {
    let (key, value) = text.split_once('=')?;
    if key.is_empty() {
        return None;
    }
    Some(Entry {
        key: key.to_string(),
        value: value.to_string(),
    })
}
"#;

pub(crate) const V3_TESTS_PARSER: &str = r#"use chunk_parser::{parse, Entry, ParseError, Parser};

pub(crate) const TEXT: &str = "a=1\n\nb=2\nc=3";

fn pairs(entries: &[Entry]) -> Vec<(&str, &str)> {
    entries.iter().map(|e| (e.key.as_str(), e.value.as_str())).collect()
}

fn run(chunks: &[&str]) -> Result<Vec<Entry>, ParseError> {
    let mut parser = Parser::new();
    let mut all = Vec::new();
    for chunk in chunks {
        all.extend(parser.feed(chunk)?);
    }
    all.extend(parser.close()?);
    Ok(all)
}

#[test]
fn a_whole_text_is_parsed_in_order() {
    let entries = parse("a=1\nb=2\n").expect("parses");
    assert_eq!(pairs(&entries), [("a", "1"), ("b", "2")]);
}

#[test]
fn chunks_split_anywhere_give_the_same_entries() {
    let whole = parse(TEXT).expect("parses");
    for split in 0..=TEXT.len() {
        let entries = run(&[&TEXT[..split], &TEXT[split..]]).expect("parses");
        assert_eq!(entries, whole, "split at {split}");
    }
}

#[test]
fn an_entry_is_returned_by_the_chunk_that_completes_its_line() {
    let mut parser = Parser::new();
    assert_eq!(pairs(&parser.feed("a=1\nb").expect("feeds")), [("a", "1")]);
    assert_eq!(pairs(&parser.feed("=2\nc=3").expect("feeds")), [("b", "2")]);
    assert_eq!(pairs(&parser.close().expect("closes")), [("c", "3")]);
}

#[test]
fn a_blank_line_is_not_an_entry_and_not_an_error() {
    let entries = parse("a=1\n\n\nb=2\n").expect("parses");
    assert_eq!(pairs(&entries), [("a", "1"), ("b", "2")]);
}

#[test]
fn a_line_that_is_not_a_pair_is_an_error_that_names_its_line() {
    assert_eq!(parse("a=1\nb=2\nnot a pair\nc=3\n"), Err(ParseError { line: 3 }));
    assert_eq!(parse("=no key\n"), Err(ParseError { line: 1 }));
}

#[test]
fn the_first_error_ends_the_parse() {
    let mut parser = Parser::new();
    let error = parser.feed("a=1\nbad\nc=3\n").expect_err("fails");
    assert_eq!(error, ParseError { line: 2 });
    assert_eq!(parser.feed("d=4\n"), Err(error.clone()));
    assert_eq!(parser.close(), Err(error));
}
"#;

pub(crate) const V3_EXAMPLES_STREAM: &str = r#"fn main() -> Result<(), chunk_parser::ParseError> {
    let mut parser = chunk_parser::Parser::new();
    for chunk in ["a=1\nb", "=2\nc=", "3\n"] {
        for entry in parser.feed(chunk)? {
            println!("{}={}", entry.key, entry.value);
        }
    }
    for entry in parser.close()? {
        println!("{}={}", entry.key, entry.value);
    }
    let mut broken = chunk_parser::Parser::new();
    if let Err(error) = broken.feed("a=1\nnot a pair\n") {
        println!("{error}");
    }
    Ok(())
}
"#;

pub(crate) const V4_SRC_LIB: &str = r#"//! Reads `key=value` text into entries.

use std::fmt;

/// The longest line, in bytes, when the caller sets no other limit.
pub const DEFAULT_MAX_LINE: usize = 65_536;

/// One entry of the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The text before the first `=`.
    pub key: String,
    /// The text after it.
    pub value: String,
}

/// Why a line is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// The line is not `key=value`.
    Malformed,
    /// The line is longer than the limit.
    TooLong,
}

/// A line that is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// The number of the line, counted from 1.
    pub line: usize,
    /// Why it is refused.
    pub reason: Reason,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reason {
            Reason::Malformed => write!(f, "line {} is not key=value", self.line),
            Reason::TooLong => write!(f, "line {} is over the limit", self.line),
        }
    }
}

impl std::error::Error for ParseError {}

/// Reads text that arrives in chunks.
#[derive(Debug)]
pub struct Parser {
    pending: String,
    lines: usize,
    max_line: usize,
    failed: Option<ParseError>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::with_max_line(DEFAULT_MAX_LINE)
    }
}

impl Parser {
    /// A parser that has read nothing and accepts lines of at most
    /// [`DEFAULT_MAX_LINE`] bytes.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A parser that has read nothing and accepts lines of at most `max_line`
    /// bytes, not counting the line ending.
    #[must_use]
    pub fn with_max_line(max_line: usize) -> Self {
        Self {
            pending: String::new(),
            lines: 0,
            max_line,
            failed: None,
        }
    }

    /// Takes the next chunk, which may end anywhere, and returns the entries
    /// whose lines it completed.
    ///
    /// ```
    /// let mut parser = chunk_parser::Parser::with_max_line(16);
    /// let first = parser.feed("a=1\nb")?;
    /// assert_eq!(first.len(), 1);
    /// let second = parser.feed("=2\n")?;
    /// assert_eq!(second[0].key, "b");
    /// # Ok::<(), chunk_parser::ParseError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The first line that is not `key=value` or is over the limit; every
    /// later call returns it again.
    pub fn feed(&mut self, chunk: &str) -> Result<Vec<Entry>, ParseError> {
        if let Some(error) = &self.failed {
            return Err(error.clone());
        }
        self.pending.push_str(chunk);
        let mut found = Vec::new();
        while let Some(end) = self.pending.find('\n') {
            let line: String = self.pending.drain(..=end).collect();
            if let Some(entry) = self.take(line.trim_end_matches(['\n', '\r']))? {
                found.push(entry);
            }
        }
        if self.pending.trim_end_matches('\r').len() > self.max_line {
            return Err(self.fail(Reason::TooLong, 1));
        }
        Ok(found)
    }

    /// Ends the input and returns the entry of a last line that has no line
    /// ending.
    ///
    /// # Errors
    ///
    /// The first line that is not `key=value` or is over the limit.
    pub fn close(mut self) -> Result<Vec<Entry>, ParseError> {
        if let Some(error) = self.failed.take() {
            return Err(error);
        }
        let rest = std::mem::take(&mut self.pending);
        if rest.is_empty() {
            return Ok(Vec::new());
        }
        Ok(self.take(rest.trim_end_matches('\r'))?.into_iter().collect())
    }

    fn take(&mut self, text: &str) -> Result<Option<Entry>, ParseError> {
        self.lines += 1;
        if text.len() > self.max_line {
            return Err(self.fail(Reason::TooLong, 0));
        }
        if text.is_empty() {
            return Ok(None);
        }
        match pair(text) {
            Some(entry) => Ok(Some(entry)),
            None => Err(self.fail(Reason::Malformed, 0)),
        }
    }

    /// Records the first error. `ahead` is 1 for a partial line, whose number
    /// is the next one.
    fn fail(&mut self, reason: Reason, ahead: usize) -> ParseError {
        let error = ParseError {
            line: self.lines + ahead,
            reason,
        };
        self.failed = Some(error.clone());
        error
    }
}

fn pair(text: &str) -> Option<Entry> {
    let (key, value) = text.split_once('=')?;
    if key.is_empty() {
        return None;
    }
    Some(Entry {
        key: key.to_string(),
        value: value.to_string(),
    })
}
"#;

pub(crate) const V4_TESTS_PARSER: &str = r#"use chunk_parser::{Entry, ParseError, Parser, Reason};

pub(crate) const TEXT: &str = "a=1\n\nb=2\nc=3";

fn pairs(entries: &[Entry]) -> Vec<(&str, &str)> {
    entries.iter().map(|e| (e.key.as_str(), e.value.as_str())).collect()
}

fn run(chunks: &[&str]) -> Result<Vec<Entry>, ParseError> {
    let mut parser = Parser::new();
    let mut all = Vec::new();
    for chunk in chunks {
        all.extend(parser.feed(chunk)?);
    }
    all.extend(parser.close()?);
    Ok(all)
}

#[test]
fn a_whole_text_is_one_chunk() {
    let entries = run(&["a=1\nb=2\n"]).expect("parses");
    assert_eq!(pairs(&entries), [("a", "1"), ("b", "2")]);
}

#[test]
fn chunks_split_anywhere_give_the_same_entries() {
    let whole = run(&[TEXT]).expect("parses");
    for split in 0..=TEXT.len() {
        let entries = run(&[&TEXT[..split], &TEXT[split..]]).expect("parses");
        assert_eq!(entries, whole, "split at {split}");
    }
}

#[test]
fn an_entry_is_returned_by_the_chunk_that_completes_its_line() {
    let mut parser = Parser::new();
    assert_eq!(pairs(&parser.feed("a=1\nb").expect("feeds")), [("a", "1")]);
    assert_eq!(pairs(&parser.feed("=2\nc=3").expect("feeds")), [("b", "2")]);
    assert_eq!(pairs(&parser.close().expect("closes")), [("c", "3")]);
}

#[test]
fn a_blank_line_is_not_an_entry_and_not_an_error() {
    let entries = run(&["a=1\n\n\nb=2\n"]).expect("parses");
    assert_eq!(pairs(&entries), [("a", "1"), ("b", "2")]);
}

#[test]
fn a_line_that_is_not_a_pair_is_an_error_that_names_its_line() {
    let malformed = |line| ParseError { line, reason: Reason::Malformed };
    assert_eq!(run(&["a=1\nb=2\nnot a pair\nc=3\n"]), Err(malformed(3)));
    assert_eq!(run(&["=no key\n"]), Err(malformed(1)));
}

#[test]
fn the_first_error_ends_the_parse() {
    let mut parser = Parser::new();
    let error = parser.feed("a=1\nbad\nc=3\n").expect_err("fails");
    assert_eq!(error, ParseError { line: 2, reason: Reason::Malformed });
    assert_eq!(parser.feed("d=4\n"), Err(error.clone()));
    assert_eq!(parser.close(), Err(error));
}

#[test]
fn a_line_over_the_limit_is_an_error_before_its_line_ending_arrives() {
    let mut parser = Parser::with_max_line(8);
    assert!(parser.feed("abc=1234").is_ok(), "eight bytes are within the limit");
    let error = parser.feed("5").expect_err("nine bytes are over");
    assert_eq!(error, ParseError { line: 1, reason: Reason::TooLong });
    let mut whole = Parser::with_max_line(8);
    let error = whole.feed("a=1\nabc=12345\n").expect_err("fails");
    assert_eq!(error, ParseError { line: 2, reason: Reason::TooLong });
}

#[test]
fn the_default_limit_is_sixty_five_thousand_five_hundred_thirty_six_bytes() {
    assert_eq!(chunk_parser::DEFAULT_MAX_LINE, 65_536);
    let mut parser = Parser::new();
    let value = "x".repeat(65_536 - 2);
    assert!(parser.feed(&format!("k={value}\n")).is_ok());
    let over = "x".repeat(65_536 - 1);
    assert_eq!(
        parser.feed(&format!("k={over}")),
        Err(ParseError { line: 2, reason: Reason::TooLong })
    );
}
"#;

pub(crate) const V4_EXAMPLES_STREAM: &str = r#"fn main() -> Result<(), chunk_parser::ParseError> {
    let mut parser = chunk_parser::Parser::new();
    for chunk in ["a=1\nb", "=2\nc=", "3\n"] {
        for entry in parser.feed(chunk)? {
            println!("{}={}", entry.key, entry.value);
        }
    }
    for entry in parser.close()? {
        println!("{}={}", entry.key, entry.value);
    }
    Ok(())
}
"#;

pub(crate) const V4_EXAMPLES_LIMITS: &str = r#"fn main() {
    let mut parser = chunk_parser::Parser::with_max_line(8);
    match parser.feed("a=1\nabc=123456") {
        Err(error) => println!("{error}"),
        Ok(entries) => panic!("expected the limit to be reached, got {entries:?}"),
    }
}
"#;
