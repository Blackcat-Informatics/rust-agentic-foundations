// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Links: inline links and images, reference definitions, and the `href` and
//! `src` attributes of inline HTML.
//!
//! The inline scanner follows `CommonMark`: a link text is followed
//! immediately by `(`, the parts inside the parentheses are separated by
//! spaces, tabs and line endings only, and a bare destination ends at a
//! space or an ASCII control character, not at any Unicode white space. A
//! link whose text or parentheses span a line break is not read as a link; it
//! is recorded as a split link instead.

use std::error::Error;
use std::fmt;

/// The longest link destination or title the scanner reads, in bytes,
/// not counting the delimiters of a title or of an angle-bracket destination.
pub(crate) const MAX_LINK_PART: usize = 4096;

/// The syntax a link was written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkForm {
    /// `[text](target)`.
    Inline,
    /// `![alt](target)`.
    Image,
    /// `[label]: target`.
    ReferenceDefinition,
    /// An `href="target"` attribute of inline HTML.
    Href,
    /// A `src="target"` attribute of inline HTML.
    Src,
}

/// A link outside fenced code, code spans and HTML comments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    target: String,
    form: LinkForm,
    line: usize,
}

/// A link or image whose text or destination spans a line break. It is not
/// a link; it is recorded so the profile can report it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitLink {
    form: LinkForm,
    line: usize,
}

impl SplitLink {
    /// [`LinkForm::Inline`] or [`LinkForm::Image`].
    #[must_use]
    pub fn form(&self) -> LinkForm {
        self.form
    }

    /// The 1-based line, in the whole text, where the link opens.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }
}

/// What a link target names.
///
/// The three forms are every place a target can lead, so the enum is
/// exhaustive: a consumer matches every variant, and a new one is a breaking
/// change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkTarget {
    /// A target with a scheme (`https:`, `mailto:`) or a network path (`//`).
    External(String),
    /// A heading anchor in the same document; the text after `#`.
    Fragment(String),
    /// A path inside the repository, relative to the root, with an anchor.
    Relative {
        /// The normalised path, `/`-separated, with no `.` or `..`.
        path: String,
        /// The text after `#`, when the target has one.
        anchor: Option<String>,
    },
}

/// Why a link target cannot be resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkError {
    /// The target is empty.
    Empty,
    /// The target is an absolute path.
    Absolute {
        /// The target.
        target: String,
    },
    /// The target climbs above the repository root.
    EscapesRoot {
        /// The target.
        target: String,
    },
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("the link has an empty target"),
            Self::Absolute { target } => write!(f, "link `{target}` is an absolute path"),
            Self::EscapesRoot { target } => {
                write!(f, "link `{target}` leaves the repository")
            }
        }
    }
}

impl Error for LinkError {}

impl Link {
    /// The target as written, with backslash escapes removed.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// The syntax the link was written in.
    #[must_use]
    pub fn form(&self) -> LinkForm {
        self.form
    }

    /// The 1-based line, in the whole text, of the link.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }

    /// The text after the first `#` of the target, when there is one.
    #[must_use]
    pub fn anchor(&self) -> Option<&str> {
        self.target.split_once('#').map(|(_, anchor)| anchor)
    }

    /// What the target names, resolved against the repository-relative path
    /// of the document that holds the link.
    ///
    /// # Errors
    ///
    /// [`LinkError::Empty`] for an empty target, [`LinkError::Absolute`] for
    /// a path that starts with `/`, and [`LinkError::EscapesRoot`] when `..`
    /// segments climb above the root of `doc_path`. A target that leaves the
    /// root is never clamped to it.
    pub fn target_in(&self, doc_path: &str) -> Result<LinkTarget, LinkError> {
        resolve(&self.target, doc_path)
    }
}

fn has_scheme(target: &str) -> bool {
    let mut chars = target.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    for c in chars {
        if c == ':' {
            return true;
        }
        if !(c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-')) {
            return false;
        }
    }
    false
}

fn resolve(target: &str, doc_path: &str) -> Result<LinkTarget, LinkError> {
    if target.is_empty() {
        return Err(LinkError::Empty);
    }
    if has_scheme(target) || target.starts_with("//") {
        return Ok(LinkTarget::External(target.to_string()));
    }
    if let Some(anchor) = target.strip_prefix('#') {
        return Ok(LinkTarget::Fragment(anchor.to_string()));
    }
    if target.starts_with('/') {
        return Err(LinkError::Absolute {
            target: target.to_string(),
        });
    }
    let (file, anchor) = match target.split_once('#') {
        Some((file, anchor)) => (file, Some(anchor.to_string())),
        None => (target, None),
    };
    let escapes = || LinkError::EscapesRoot {
        target: target.to_string(),
    };
    let mut parts: Vec<&str> = doc_path.split('/').collect();
    parts.pop();
    parts.retain(|part| !part.is_empty() && *part != ".");
    for part in file.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(escapes());
                }
            }
            name => parts.push(name),
        }
    }
    Ok(LinkTarget::Relative {
        path: parts.join("/"),
        anchor,
    })
}

/// What the scanner needs to know about one line.
pub(crate) struct LineView<'a> {
    /// A fence line or a line inside fenced code.
    pub(crate) fenced: bool,
    /// A line with nothing but whitespace in the source.
    pub(crate) blank: bool,
    /// An ATX heading line.
    pub(crate) heading: bool,
    /// The line without HTML comments (code spans kept).
    pub(crate) visible: &'a str,
}

/// The links and split links of a body.
pub(crate) struct Found {
    pub(crate) links: Vec<Link>,
    pub(crate) split: Vec<SplitLink>,
}

/// An error from the link scanner: a destination or title over the bound.
pub(crate) struct TooLong {
    /// The 1-based line where the part starts.
    pub(crate) line: usize,
}

/// Scan every line of a body. `first_line` is the whole-text number of line
/// index 0.
pub(crate) fn scan_links(
    views: &[LineView<'_>],
    first_line: usize,
    remove_code: impl Fn(&str) -> String,
) -> Result<Found, TooLong> {
    let mut found = Found {
        links: Vec::new(),
        split: Vec::new(),
    };
    let mut tags = TagState::default();
    let mut chunk: Vec<(usize, String)> = Vec::new();
    for (index, view) in views.iter().enumerate() {
        let number = first_line + index;
        if view.fenced || view.blank {
            tags = TagState::default();
            flush(&mut chunk, &mut found)?;
            continue;
        }
        let text = remove_code(view.visible);
        for (form, target) in html_attribute_targets(&text, &mut tags) {
            found.links.push(Link {
                target,
                form,
                line: number,
            });
        }
        if let Some(target) = definition_target(&text) {
            found.links.push(Link {
                target,
                form: LinkForm::ReferenceDefinition,
                line: number,
            });
        }
        let unquoted = strip_quotes(&text).to_string();
        if view.heading {
            flush(&mut chunk, &mut found)?;
            chunk.push((number, unquoted));
            flush(&mut chunk, &mut found)?;
        } else {
            chunk.push((number, unquoted));
        }
    }
    flush(&mut chunk, &mut found)?;
    found.links.sort_by_key(|link| link.line);
    Ok(found)
}

/// The line without its leading block-quote markers, so that a link inside a
/// quote reads the same on every line of the quote.
fn strip_quotes(line: &str) -> &str {
    let mut rest = line;
    loop {
        let indent = rest.len() - rest.trim_start_matches(' ').len();
        let Some(after) = rest[indent..].strip_prefix('>') else {
            return rest;
        };
        if indent > 3 {
            return rest;
        }
        rest = if let Some(spaced) = after.strip_prefix(' ') {
            spaced
        } else {
            after
        };
    }
}

fn flush(chunk: &mut Vec<(usize, String)>, found: &mut Found) -> Result<(), TooLong> {
    if chunk.is_empty() {
        return Ok(());
    }
    let mut text = String::new();
    let mut starts = Vec::with_capacity(chunk.len());
    for (_, line) in chunk.iter() {
        starts.push(text.len());
        text.push_str(line);
        text.push('\n');
    }
    let line_of = |offset: usize| {
        let at = starts.partition_point(|start| *start <= offset);
        chunk[at.saturating_sub(1)].0
    };
    scan_inline(&text, &line_of, found)?;
    chunk.clear();
    Ok(())
}

struct Opener {
    pos: usize,
    image: bool,
}

fn escaped_at(bytes: &[u8], index: usize) -> bool {
    bytes[..index]
        .iter()
        .rev()
        .take_while(|b| **b == b'\\')
        .count()
        % 2
        == 1
}

fn scan_inline(
    text: &str,
    line_of: &impl Fn(usize) -> usize,
    found: &mut Found,
) -> Result<(), TooLong> {
    let bytes = text.as_bytes();
    let mut stack: Vec<Opener> = Vec::new();
    // Links cannot hold links: after a link, every link opener below this
    // stack depth is dead. Images stay open. Keeping one depth instead of a
    // flag per opener keeps the scan linear.
    let mut dead_below = 0_usize;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'[' => {
                let image = i > 0 && bytes[i - 1] == b'!' && !escaped_at(bytes, i - 1);
                stack.push(Opener { pos: i, image });
                i += 1;
            }
            b']' => {
                let opener = stack.pop();
                let alive = stack.len() >= dead_below;
                dead_below = dead_below.min(stack.len());
                let mut next = i + 1;
                if let Some(opener) = opener
                    && (opener.image || alive)
                    && bytes.get(i + 1) == Some(&b'(')
                    && let Some((target, end)) = parse_inline(text, i + 2, line_of(i))?
                {
                    let form = if opener.image {
                        LinkForm::Image
                    } else {
                        LinkForm::Inline
                    };
                    let line = line_of(opener.pos);
                    if text[opener.pos..end].contains('\n') {
                        found.split.push(SplitLink { form, line });
                    } else {
                        found.links.push(Link { target, form, line });
                    }
                    if !opener.image {
                        dead_below = stack.len();
                    }
                    next = end;
                }
                i = next;
            }
            _ => i += 1,
        }
    }
    Ok(())
}

fn is_link_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

fn skip_space(bytes: &[u8], mut i: usize) -> usize {
    while bytes.get(i).is_some_and(|b| is_link_space(*b)) {
        i += 1;
    }
    i
}

fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(next) = chars.next_if(char::is_ascii_punctuation)
        {
            out.push(next);
        } else {
            out.push(c);
        }
    }
    out
}

/// Read what follows `](` from byte `from`: a destination, an optional title
/// and the closing parenthesis. Returns the unescaped destination and the
/// index after `)`, or `None` when the text is not a link.
fn parse_inline(text: &str, from: usize, line: usize) -> Result<Option<(String, usize)>, TooLong> {
    let bytes = text.as_bytes();
    let mut i = skip_space(bytes, from);
    let destination;
    if bytes.get(i) == Some(&b'<') {
        let mut j = i + 1;
        loop {
            match bytes.get(j) {
                None | Some(b'\n' | b'<') => return Ok(None),
                Some(b'>') => break,
                Some(b'\\') => j += 2,
                Some(_) => j += 1,
            }
            if j - (i + 1) > MAX_LINK_PART {
                return Err(TooLong { line });
            }
        }
        destination = unescape(&text[i + 1..j]);
        i = j + 1;
    } else {
        let mut depth = 0_usize;
        let mut j = i;
        loop {
            match bytes.get(j) {
                None => return Ok(None),
                Some(b'\\') => j += 2,
                Some(b'(') => {
                    depth += 1;
                    j += 1;
                }
                Some(b')') => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    j += 1;
                }
                Some(b) if *b == b' ' || b.is_ascii_control() => break,
                Some(_) => j += 1,
            }
            if j - i > MAX_LINK_PART {
                return Err(TooLong { line });
            }
        }
        destination = unescape(&text[i..j]);
        i = j;
    }
    i = skip_space(bytes, i);
    if let Some(&open) = bytes.get(i)
        && matches!(open, b'"' | b'\'' | b'(')
    {
        let close = if open == b'(' { b')' } else { open };
        let mut j = i + 1;
        loop {
            match bytes.get(j) {
                None => return Ok(None),
                Some(b'\\') => j += 2,
                Some(b) if *b == close => break,
                Some(_) => j += 1,
            }
            if j - (i + 1) > MAX_LINK_PART {
                return Err(TooLong { line });
            }
        }
        i = skip_space(bytes, j + 1);
    }
    if bytes.get(i) == Some(&b')') {
        Ok(Some((destination, i + 1)))
    } else {
        Ok(None)
    }
}

/// The target of a reference definition such as `[label]: target "title"`.
fn definition_target(line: &str) -> Option<String> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let rest = line[indent..].strip_prefix('[')?;
    let (label, after) = rest.split_once("]:")?;
    if label.trim().is_empty() || label.starts_with('^') {
        return None;
    }
    let after = after.trim_start();
    let target = match after.strip_prefix('<') {
        Some(inner) => inner.split('>').next()?,
        None => after.split_whitespace().next()?,
    };
    (!target.is_empty()).then(|| unescape(target))
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// True for a character that can belong to an HTML attribute name.
fn is_attribute_name_char(c: char) -> bool {
    is_word_char(c) || matches!(c, '-' | ':' | '.')
}

/// Where the HTML scan stands between lines: inside a tag, inside a quoted
/// attribute value, or waiting for the value of an `href` or `src`.
#[derive(Default)]
struct TagState {
    in_tag: bool,
    quote: Option<char>,
    awaiting_value: Option<LinkForm>,
}

fn skip_while(chars: &[char], start: usize, keep: impl Fn(char) -> bool) -> usize {
    let mut index = start;
    while chars.get(index).is_some_and(|c| keep(*c)) {
        index += 1;
    }
    index
}

type Targets = Vec<(LinkForm, String)>;

/// Read one attribute at `start` (the first character of its name) and return
/// the index after it, pushing the value of an `href` or `src` attribute.
fn read_attribute(
    chars: &[char],
    start: usize,
    state: &mut TagState,
    targets: &mut Targets,
) -> usize {
    let name_end = skip_while(chars, start, is_attribute_name_char);
    let name: String = chars[start..name_end]
        .iter()
        .collect::<String>()
        .to_ascii_lowercase();
    let after_name = skip_while(chars, name_end, char::is_whitespace);
    if chars.get(after_name) != Some(&'=') {
        return name_end;
    }
    let value_start = skip_while(chars, after_name + 1, char::is_whitespace);
    let wanted = match name.as_str() {
        "href" => Some(LinkForm::Href),
        "src" => Some(LinkForm::Src),
        _ => None,
    };
    read_value(chars, value_start, wanted, state, targets)
}

/// Read the attribute value that starts at `start`, returning the index after
/// it. A value that is not on this line is read from the next one when the
/// attribute is `href` or `src`.
fn read_value(
    chars: &[char],
    start: usize,
    wanted: Option<LinkForm>,
    state: &mut TagState,
    targets: &mut Targets,
) -> usize {
    match chars.get(start) {
        Some(&quote @ ('"' | '\'')) => {
            let inner = start + 1;
            let close = chars[inner..].iter().position(|c| *c == quote);
            if let Some(offset) = close {
                if let Some(form) = wanted {
                    targets.push((form, chars[inner..inner + offset].iter().collect()));
                }
                inner + offset + 1
            } else {
                state.quote = Some(quote);
                chars.len()
            }
        }
        Some(_) => {
            let end = skip_while(chars, start, |c| !c.is_whitespace() && c != '>');
            if let Some(form) = wanted {
                let value: String = chars[start..end].iter().collect();
                targets.push((form, value.trim_end_matches('/').to_string()));
            }
            end
        }
        None => {
            state.awaiting_value = wanted;
            chars.len()
        }
    }
}

/// The `href` and `src` values found on one line, tracking tag state across
/// lines. A tag opens at `<` before a letter or `/` and runs to the next `>`
/// that is not inside quotes. Autolinks such as `<https://x>` are not tags.
fn html_attribute_targets(line: &str, state: &mut TagState) -> Targets {
    let chars: Vec<char> = line.chars().collect();
    let mut targets = Vec::new();
    let mut index = 0;
    if let Some(form) = state.awaiting_value.take()
        && state.in_tag
    {
        let start = skip_while(&chars, 0, char::is_whitespace);
        index = read_value(&chars, start, Some(form), state, &mut targets);
    }
    while let Some(&c) = chars.get(index) {
        if let Some(quote) = state.quote {
            match chars[index..].iter().position(|x| *x == quote) {
                Some(offset) => {
                    state.quote = None;
                    index += offset + 1;
                }
                None => break,
            }
        } else if !state.in_tag {
            let opens = c == '<'
                && chars
                    .get(index + 1)
                    .is_some_and(|n| n.is_alphabetic() || *n == '/');
            index += 1;
            if opens {
                let name_end = skip_while(&chars, index, |x| is_attribute_name_char(x) || x == '/');
                let autolink = chars[index..name_end].contains(&':');
                state.in_tag = !autolink;
                index = name_end;
            }
        } else if c == '>' {
            state.in_tag = false;
            index += 1;
        } else if is_attribute_name_char(c) {
            index = read_attribute(&chars, index, state, &mut targets);
        } else if matches!(c, '"' | '\'') {
            state.quote = Some(c);
            index += 1;
        } else {
            index += 1;
        }
    }
    targets
}
