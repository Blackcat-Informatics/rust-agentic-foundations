// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The line structure of a body: lines, fenced code blocks, container
//! nesting, and the views of a line without HTML comments or code spans.

use std::collections::HashMap;

use crate::error::{ErrorKind, ParseError};
use crate::limits::MAX_BLOCK_NESTING;

/// Where a body starts in the whole text of its document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Origin {
    /// Byte offset of the first byte of the body.
    pub(crate) byte: usize,
    /// 1-based line number of the first line of the body.
    pub(crate) line: usize,
}

impl Origin {
    /// The origin of a text that is not part of a longer one.
    pub(crate) const START: Self = Self { byte: 0, line: 1 };
}

/// One line of the body. Offsets are local to the body text.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Line {
    /// Offset of the first byte.
    pub(crate) start: usize,
    /// Offset after the last byte of the text, before any line ending.
    pub(crate) end: usize,
    /// The line is a fence line or sits inside a fenced code block.
    pub(crate) fenced: bool,
}

/// A fenced code block.
#[derive(Debug, Clone)]
pub(crate) struct FenceBlock {
    /// Index of the opening fence line.
    pub(crate) first: usize,
    /// Index of the closing fence line, or of the last line when the block is
    /// never closed.
    pub(crate) last: usize,
    /// Whether a closing fence line ends the block.
    pub(crate) closed: bool,
    /// The info string after the opening fence.
    pub(crate) info: String,
}

/// The line structure of a body.
pub(crate) struct Scan {
    pub(crate) lines: Vec<Line>,
    pub(crate) fences: Vec<FenceBlock>,
}

/// The error for a failure at byte `column_byte` of line `index`.
pub(crate) fn error_at(
    body: &str,
    scan: &[Line],
    origin: Origin,
    index: usize,
    column_byte: usize,
    kind: ErrorKind,
) -> ParseError {
    let line = &scan[index];
    let local = line.start + column_byte;
    let column = body[line.start..local].chars().count() + 1;
    ParseError::at(kind, origin.line + index, column, origin.byte + local)
}

/// True for a line with nothing but whitespace.
pub(crate) fn is_blank(text: &str) -> bool {
    text.trim().is_empty()
}

/// Split `body` into lines, find the fenced code blocks and enforce the
/// nesting bound.
pub(crate) fn scan(body: &str, origin: Origin) -> Result<Scan, ParseError> {
    let mut lines = split_lines(body);
    let mut fences = Vec::new();
    let mut open: Option<OpenFence> = None;
    let mut stack: Vec<usize> = Vec::new();
    let mut previous_blank = true;
    for index in 0..lines.len() {
        let text = &body[lines[index].start..lines[index].end];
        if let Some(fence) = &open {
            lines[index].fenced = true;
            if closes_fence(text, fence, base_indent(&stack)) {
                if let Some(block) = fences.last_mut() {
                    block_close(block, index);
                    block.closed = true;
                }
                open = None;
            } else if let Some(block) = fences.last_mut() {
                block_close(block, index);
            }
            continue;
        }
        if is_blank(text) {
            previous_blank = true;
            continue;
        }
        let prefix = prefix_of(text);
        if let Some(column) = depth_overflow(&prefix, &mut stack, text, previous_blank) {
            return Err(error_at(
                body,
                &lines,
                origin,
                index,
                column,
                ErrorKind::BlockNesting {
                    max: MAX_BLOCK_NESTING,
                },
            ));
        }
        previous_blank = false;
        if let Some(opened) = opens_fence(text, prefix.content, base_indent(&stack)) {
            lines[index].fenced = true;
            fences.push(FenceBlock {
                first: index,
                last: index,
                closed: false,
                info: opened.info.clone(),
            });
            open = Some(opened);
        }
    }
    Ok(Scan { lines, fences })
}

fn block_close(block: &mut FenceBlock, index: usize) {
    block.last = index;
}

fn split_lines(body: &str) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut start = 0;
    while start < body.len() {
        let rest = &body[start..];
        let (raw_end, next) = match rest.find('\n') {
            Some(index) => (start + index, start + index + 1),
            None => (body.len(), body.len()),
        };
        let end = if body[start..raw_end].ends_with('\r') {
            raw_end - 1
        } else {
            raw_end
        };
        lines.push(Line {
            start,
            end,
            fenced: false,
        });
        start = next;
    }
    lines
}

/// An open fenced code block.
#[derive(Debug, Clone)]
struct OpenFence {
    marker: char,
    length: usize,
    info: String,
}

/// Extra indentation the enclosing list items allow before a fence.
fn base_indent(stack: &[usize]) -> usize {
    match stack.last() {
        Some(column) => column + 2,
        None => 0,
    }
}

fn opens_fence(text: &str, content: usize, base: usize) -> Option<OpenFence> {
    let rest = &text[content..];
    let indent = rest.len() - rest.trim_start_matches(' ').len();
    if indent > 3 + base {
        return None;
    }
    let rest = &rest[indent..];
    let marker = rest.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let length = rest.chars().take_while(|c| *c == marker).count();
    if length < 3 {
        return None;
    }
    let info = rest[length..].trim();
    if marker == '`' && info.contains('`') {
        return None;
    }
    Some(OpenFence {
        marker,
        length,
        info: info.to_string(),
    })
}

fn closes_fence(text: &str, fence: &OpenFence, base: usize) -> bool {
    let prefix = prefix_of(text);
    let rest = &text[prefix.content..];
    let indent = rest.len() - rest.trim_start_matches(' ').len();
    if indent > 3 + base {
        return false;
    }
    let rest = &rest[indent..];
    let run = rest.chars().take_while(|c| *c == fence.marker).count();
    run >= fence.length && rest[run..].trim().is_empty()
}

/// The block-quote and list markers at the start of a line.
struct Prefix {
    quotes: usize,
    /// The columns of the list markers, in order.
    markers: Vec<usize>,
    /// Byte offset where the content after the markers starts.
    content: usize,
    /// Byte offset of each marker, quote or list, in order.
    offsets: Vec<usize>,
}

fn is_thematic_break(text: &str) -> bool {
    let compact: Vec<char> = text.chars().filter(|c| *c != ' ').collect();
    compact.len() >= 3
        && compact.iter().all(|c| matches!(c, '-' | '*' | '_'))
        && compact.iter().all(|c| *c == compact[0])
}

fn prefix_of(text: &str) -> Prefix {
    let mut quotes = 0;
    let mut markers = Vec::new();
    let mut offsets = Vec::new();
    let mut pos = 0;
    while quotes + markers.len() <= MAX_BLOCK_NESTING {
        let rest = &text[pos..];
        let spaces = rest.len() - rest.trim_start_matches(' ').len();
        let after = &rest[spaces..];
        if is_thematic_break(after) {
            break;
        }
        if let Some(quoted) = after.strip_prefix('>') {
            quotes += 1;
            offsets.push(pos + spaces);
            let skip = usize::from(quoted.starts_with(' '));
            pos += spaces + 1 + skip;
        } else if let Some(width) = list_marker_width(after) {
            markers.push(pos + spaces);
            offsets.push(pos + spaces);
            let gap = after[width..].len() - after[width..].trim_start_matches(' ').len();
            pos += spaces + width + gap;
        } else {
            break;
        }
    }
    Prefix {
        quotes,
        markers,
        content: pos,
        offsets,
    }
}

/// Whether the line opens a list item: a list marker follows its quote
/// markers, if any.
pub(crate) fn opens_list_item(text: &str) -> bool {
    !prefix_of(text).markers.is_empty()
}

/// The width of a list marker at the start of `text` (`-`, `*`, `+`, or
/// digits and `.` or `)`) that is followed by a space or the end of the line.
fn list_marker_width(text: &str) -> Option<usize> {
    let first = text.chars().next()?;
    let width = if matches!(first, '-' | '*' | '+') {
        1
    } else if first.is_ascii_digit() {
        let digits = text.chars().take_while(char::is_ascii_digit).count();
        let delimiter = text[digits..].chars().next()?;
        if digits > 9 || !matches!(delimiter, '.' | ')') {
            return None;
        }
        digits + 1
    } else {
        return None;
    };
    let after = &text[width..];
    (after.is_empty() || after.starts_with(' ')).then_some(width)
}

/// Update the list stack for a non-blank line and return the byte column of
/// the marker that exceeds the nesting bound, if any.
fn depth_overflow(
    prefix: &Prefix,
    stack: &mut Vec<usize>,
    text: &str,
    previous_blank: bool,
) -> Option<usize> {
    if let Some(first) = prefix.markers.first() {
        while stack.last().is_some_and(|top| top >= first) {
            stack.pop();
        }
        stack.extend(prefix.markers.iter().copied());
    } else if previous_blank {
        let indent = text.len() - text.trim_start_matches(' ').len();
        while stack.last().is_some_and(|top| indent < top + 2) {
            stack.pop();
        }
    }
    let depth = prefix.quotes + stack.len();
    if depth > MAX_BLOCK_NESTING {
        // Point at the first marker of the line that crosses the bound.
        let over = depth - MAX_BLOCK_NESTING;
        let on_line = prefix.offsets.len();
        let index = on_line.saturating_sub(over.min(on_line));
        return Some(match prefix.offsets.get(index) {
            Some(offset) => *offset,
            None => 0,
        });
    }
    None
}

/// A run of backticks in a line.
struct Run {
    start: usize,
    len: usize,
    /// A backslash escapes the first backtick, so the run cannot open a span.
    escaped: bool,
}

/// What `strip` does to a line.
#[derive(Clone, Copy)]
struct Strip {
    /// Remove HTML comments.
    comments: bool,
    /// Replace each code span with one space.
    blank_spans: bool,
}

/// The line with its HTML comments removed. `in_comment` carries a comment
/// that is still open from an earlier line. A comment marker inside a code
/// span is text.
pub(crate) fn remove_comments(line: &str, in_comment: &mut bool) -> String {
    strip(
        line,
        in_comment,
        Strip {
            comments: true,
            blank_spans: false,
        },
    )
}

/// The line with every code span replaced by one space. A code span opens
/// with a run of backticks and closes at the next run of the same length; a
/// run with no match is text.
pub(crate) fn remove_code_spans(line: &str) -> String {
    strip(
        line,
        &mut false,
        Strip {
            comments: false,
            blank_spans: true,
        },
    )
}

fn backtick_runs(line: &str) -> Vec<Run> {
    let mut runs = Vec::new();
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'`' {
            let start = index;
            while bytes.get(index) == Some(&b'`') {
                index += 1;
            }
            let slashes = bytes[..start]
                .iter()
                .rev()
                .take_while(|b| **b == b'\\')
                .count();
            runs.push(Run {
                start,
                len: index - start,
                escaped: slashes % 2 == 1,
            });
        } else {
            index += 1;
        }
    }
    runs
}

/// The first run after run `index` with the same length, found by binary
/// search in the runs of that length.
fn closing_run(by_len: &HashMap<usize, Vec<usize>>, runs: &[Run], index: usize) -> Option<usize> {
    let same = by_len.get(&runs[index].len)?;
    let at = same.partition_point(|candidate| *candidate <= index);
    same.get(at).copied()
}

/// The next thing `strip` acts on.
enum Next {
    /// The next run of backticks.
    Run,
    /// A comment opener at this offset.
    Comment(usize),
}

fn strip(line: &str, in_comment: &mut bool, mode: Strip) -> String {
    let runs = backtick_runs(line);
    let mut by_len: HashMap<usize, Vec<usize>> = HashMap::new();
    for (index, run) in runs.iter().enumerate() {
        by_len.entry(run.len).or_default().push(index);
    }
    let (opens, closes): (Vec<usize>, Vec<usize>) = if mode.comments {
        (
            line.match_indices("<!--").map(|(at, _)| at).collect(),
            line.match_indices("-->").map(|(at, _)| at).collect(),
        )
    } else {
        (Vec::new(), Vec::new())
    };
    let mut out = String::with_capacity(line.len());
    let mut pos = 0;
    while pos < line.len() {
        if *in_comment {
            let at = closes.partition_point(|close| *close < pos);
            match closes.get(at) {
                Some(close) => {
                    *in_comment = false;
                    pos = close + 3;
                }
                None => return out,
            }
            continue;
        }
        let next_run = runs.partition_point(|run| run.start < pos);
        let next_open = opens.partition_point(|open| *open < pos);
        let run_at = runs.get(next_run).map(|run| run.start);
        let open_at = opens.get(next_open).copied();
        let next = match (run_at, open_at) {
            (None, None) => {
                out.push_str(&line[pos..]);
                return out;
            }
            (Some(_), None) => Next::Run,
            (Some(run_start), Some(open)) if run_start < open => Next::Run,
            (_, Some(open)) => Next::Comment(open),
        };
        match next {
            Next::Run => {
                let run = &runs[next_run];
                out.push_str(&line[pos..run.start]);
                let end = run.start + run.len;
                let matching = if run.escaped {
                    None
                } else {
                    closing_run(&by_len, &runs, next_run)
                };
                if let Some(partner) = matching {
                    let close_end = runs[partner].start + runs[partner].len;
                    if mode.blank_spans {
                        out.push(' ');
                    } else {
                        out.push_str(&line[run.start..close_end]);
                    }
                    pos = close_end;
                } else {
                    out.push_str(&line[run.start..end]);
                    pos = end;
                }
            }
            Next::Comment(open) => {
                out.push_str(&line[pos..open]);
                *in_comment = true;
                pos = open + 4;
            }
        }
    }
    out
}
