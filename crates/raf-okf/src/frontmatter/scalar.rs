// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Scalars and flow sequences: the value grammar of the frontmatter subset.
//!
//! Every function works on one line of text through a [`Cursor`] and reports a
//! failure as the kind of error plus the byte index inside that line, which
//! the block reader turns into a positioned [`crate::ParseError`].

use crate::error::ErrorKind;
use crate::frontmatter::value::{Scalar, ScalarStyle};
use crate::limits::MAX_YAML_DEPTH;

/// A failure inside one line: what went wrong and the byte index it points at.
pub(crate) type Fail = (ErrorKind, usize);

type Parsed<T> = Result<T, Fail>;

/// The one escape set of double-quoted scalars, shared by the reader and the
/// writer: the character after the backslash and the character it stands for.
/// Shared ECHAR values come from the lexical library; `0` is the profile's
/// explicit NUL extension. Other ECHAR members remain outside this profile.
pub const ESCAPES: [(char, char); 6] = [
    (
        '\\',
        purrdf_lex::terminals::echar_value(b'\\').expect("backslash is ECHAR"),
    ),
    (
        '"',
        purrdf_lex::terminals::echar_value(b'"').expect("double quote is ECHAR"),
    ),
    (
        'n',
        purrdf_lex::terminals::echar_value(b'n').expect("line feed is ECHAR"),
    ),
    (
        'r',
        purrdf_lex::terminals::echar_value(b'r').expect("carriage return is ECHAR"),
    ),
    (
        't',
        purrdf_lex::terminals::echar_value(b't').expect("tab is ECHAR"),
    ),
    ('0', '\0'),
];

/// Characters the flow syntax reserves. An unquoted scalar that holds one is
/// never written, so it can sit in a block or a flow position alike.
const FLOW_INDICATORS: [char; 5] = [',', '[', ']', '{', '}'];

/// A line of text with a byte position that always lies on a character
/// boundary.
///
/// The cursor holds the unread rest of the line. It is built from a position
/// checked with [`str::get`], and every move after that cuts the rest at an
/// index that comes from `char_indices`, `len_utf8` or a match on the rest
/// itself, which are character boundaries by definition. No operation takes a
/// caller-supplied length, so no input can make the cursor panic.
pub(crate) struct Cursor<'a> {
    line: &'a str,
    rest: &'a str,
}

impl<'a> Cursor<'a> {
    /// A cursor at byte `pos` of `line`.
    ///
    /// # Errors
    ///
    /// An indentation failure at `pos` when `pos` is not a character boundary
    /// of the line; the reader only passes positions after ASCII indentation,
    /// so it does not happen for text the reader produced.
    pub(crate) fn new(line: &'a str, pos: usize) -> Parsed<Self> {
        match line.get(pos..) {
            Some(rest) => Ok(Self { line, rest }),
            None => Err((ErrorKind::Indentation, line.len())),
        }
    }

    /// The byte position.
    pub(crate) fn pos(&self) -> usize {
        self.line.len() - self.rest.len()
    }

    /// The text from the position to the end of the line.
    pub(crate) fn rest(&self) -> &'a str {
        self.rest
    }

    /// The next character, without moving.
    pub(crate) fn peek(&self) -> Option<char> {
        self.rest.chars().next()
    }

    /// Move past the next character and return it.
    pub(crate) fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.cut(c.len_utf8());
        Some(c)
    }

    /// Move to the end of the line.
    pub(crate) fn finish(&mut self) {
        self.cut(self.rest.len());
    }

    /// Move past spaces.
    pub(crate) fn skip_spaces(&mut self) {
        self.cut(self.rest.len() - self.rest.trim_start_matches(' ').len());
    }

    /// Read the longest run of characters that satisfy `keep`.
    pub(crate) fn take_while(&mut self, keep: impl Fn(char) -> bool) -> &'a str {
        let len = self.boundary_where(|c| !keep(c));
        self.take(len)
    }

    /// Read up to the first character in `stops`, or to the end of the line.
    pub(crate) fn take_until(&mut self, stops: &[char]) -> &'a str {
        let len = self.boundary_where(|c| stops.contains(&c));
        self.take(len)
    }

    /// The byte index of the first character that satisfies `stop`, or the
    /// length of the rest.
    fn boundary_where(&self, stop: impl Fn(char) -> bool) -> usize {
        let mut len = self.rest.len();
        for (index, c) in self.rest.char_indices() {
            if stop(c) {
                len = index;
                break;
            }
        }
        len
    }

    fn take(&mut self, len: usize) -> &'a str {
        let (head, tail) = self.rest.split_at(len);
        self.rest = tail;
        head
    }

    /// Drop `len` bytes of the rest. `len` is always a boundary index found
    /// in the rest by this type.
    fn cut(&mut self, len: usize) {
        self.take(len);
    }
}

/// A value that sits on a key's line or after `- `.
pub(crate) enum Inline {
    /// A scalar.
    Scalar(Scalar),
    /// The scalars of a flow sequence.
    Flow(Vec<Scalar>),
}

/// Read the value at the cursor (never a space), with the comment after it.
/// `owner` names the key the value belongs to, `depth` is the number of
/// collections already open, and `in_item` is set for the value of a `- `
/// item, where a flow sequence is not allowed.
pub(crate) fn inline_value(
    cur: &mut Cursor<'_>,
    owner: &str,
    depth: usize,
    in_item: bool,
) -> Parsed<(Inline, Option<String>)> {
    let start = cur.pos();
    match cur.peek() {
        Some('[') => {
            if in_item {
                return Err((ErrorKind::NestedFlow, start));
            }
            let scalars = flow_sequence(cur, owner, depth)?;
            Ok((Inline::Flow(scalars), tail(cur, ErrorKind::TextAfterFlow)?))
        }
        Some('{') => Err((
            ErrorKind::FlowMapping {
                key: owner.to_string(),
            },
            start,
        )),
        Some(quote @ ('"' | '\'')) => {
            let scalar = quoted(cur, quote)?;
            Ok((
                Inline::Scalar(scalar),
                tail(cur, ErrorKind::TextAfterQuote)?,
            ))
        }
        _ => {
            let rest = cur.rest();
            let (text, trailing) = match rest.find(" #") {
                Some(index) => {
                    let text = rest[..index].trim_end_matches(' ');
                    (text, Some(rest[text.len()..].to_string()))
                }
                None => (rest, None),
            };
            let scalar = plain(text, start)?;
            cur.finish();
            Ok((Inline::Scalar(scalar), trailing))
        }
    }
}

/// The text after a value: nothing, or whitespace and a comment.
pub(crate) fn tail(cur: &Cursor<'_>, otherwise: ErrorKind) -> Parsed<Option<String>> {
    let rest = cur.rest();
    if rest.is_empty() {
        return Ok(None);
    }
    if rest.starts_with(' ') && rest.trim_start_matches(' ').starts_with('#') {
        return Ok(Some(rest.to_string()));
    }
    Err((otherwise, cur.pos()))
}

/// The name that follows an anchor, alias or tag indicator.
fn token_name(rest: &str) -> String {
    rest.chars()
        .take_while(|c| !c.is_whitespace() && !matches!(c, ',' | '[' | ']' | '{' | '}'))
        .collect()
}

/// A block scalar header: `>` or `|` with optional indentation and chomping.
fn is_block_header(text: &str) -> bool {
    text.strip_prefix(['>', '|'])
        .is_some_and(|rest| rest.chars().all(|c| matches!(c, '+' | '-' | '0'..='9')))
}

/// Why an unquoted scalar is refused, with the byte index into `text`, or
/// `None` when the reader takes it as it is. The empty text is not judged.
pub(crate) fn plain_refusal(text: &str) -> Option<(ErrorKind, usize)> {
    let mut chars = text.chars();
    let first = chars.next()?;
    let rest = chars.as_str();
    let by_first = match first {
        '&' => Some(ErrorKind::Anchor {
            name: token_name(rest),
        }),
        '*' => Some(ErrorKind::Alias {
            name: token_name(rest),
        }),
        '!' => Some(ErrorKind::Tag {
            name: token_name(rest),
        }),
        '|' | '>' if is_block_header(text) => Some(ErrorKind::BlockScalar {
            header: text.to_string(),
        }),
        '|' | '>' | '%' | '@' | '`' | '#' => {
            Some(ErrorKind::IndicatorCharacter { indicator: first })
        }
        '-' | '?' | ':' if rest.is_empty() || rest.starts_with(' ') => {
            Some(ErrorKind::IndicatorCharacter { indicator: first })
        }
        _ => None,
    };
    if let Some(kind) = by_first {
        return Some((kind, 0));
    }
    if let Some(index) = text.find(": ") {
        return Some((ErrorKind::PlainColonSpace, index));
    }
    if text.ends_with(':') {
        return Some((ErrorKind::PlainColonSpace, text.len() - 1));
    }
    None
}

/// Whether `text` can be written as an unquoted scalar in a block or a flow
/// position, and read back as exactly `text`.
///
/// This is the one predicate the reader and the writer share. It is stricter
/// than what the reader takes in a block position: it also refuses the flow
/// characters `, [ ] { }`, a leading or trailing space, a leading quote, a
/// ` #` and any control character, because those need a quoted style.
#[must_use]
pub fn plain_ok(text: &str) -> bool {
    !text.is_empty()
        && !text.starts_with([' ', '"', '\''])
        && !text.ends_with(' ')
        && !text.contains(" #")
        && !text
            .chars()
            .any(|c| c.is_control() || FLOW_INDICATORS.contains(&c))
        && plain_refusal(text).is_none()
}

/// An unquoted scalar read at byte `at` of its line.
fn plain(text: &str, at: usize) -> Parsed<Scalar> {
    match plain_refusal(text) {
        Some((kind, index)) => Err((kind, at + index)),
        None => Ok(Scalar {
            text: text.to_string(),
            style: ScalarStyle::Plain,
        }),
    }
}

/// A quoted scalar that opens at the cursor; leaves the cursor after the
/// closing quote.
fn quoted(cur: &mut Cursor<'_>, quote: char) -> Parsed<Scalar> {
    let start = cur.pos();
    cur.bump();
    let mut text = String::new();
    loop {
        let at = cur.pos();
        match cur.bump() {
            None => return Err((ErrorKind::UnterminatedQuote, start)),
            Some('\\') if quote == '"' => {
                let Some(escape) = cur.bump() else {
                    return Err((ErrorKind::UnterminatedQuote, start));
                };
                match ESCAPES.iter().find(|(name, _)| *name == escape) {
                    Some((_, value)) => text.push(*value),
                    None => return Err((ErrorKind::UnknownEscape { escape }, at)),
                }
            }
            Some(c) if c == quote => {
                if quote == '\'' && cur.peek() == Some('\'') {
                    cur.bump();
                    text.push('\'');
                } else {
                    let style = if quote == '"' {
                        ScalarStyle::DoubleQuoted
                    } else {
                        ScalarStyle::SingleQuoted
                    };
                    return Ok(Scalar { text, style });
                }
            }
            Some(c) => text.push(c),
        }
    }
}

/// A flow sequence of scalars that opens at the cursor; leaves the cursor
/// after the closing bracket. `depth` collections are already open.
fn flow_sequence(cur: &mut Cursor<'_>, owner: &str, depth: usize) -> Parsed<Vec<Scalar>> {
    let start = cur.pos();
    if depth + 1 > MAX_YAML_DEPTH {
        return Err((
            ErrorKind::DepthExceeded {
                max: MAX_YAML_DEPTH,
            },
            start,
        ));
    }
    cur.bump();
    let mut scalars = Vec::new();
    let mut after_comma = false;
    loop {
        cur.skip_spaces();
        let position = cur.pos();
        let Some(first) = cur.peek() else {
            return Err((ErrorKind::UnterminatedFlow, start));
        };
        match first {
            ']' => {
                if after_comma {
                    return Err((ErrorKind::EmptyFlowEntry, position));
                }
                cur.bump();
                return Ok(scalars);
            }
            ',' => return Err((ErrorKind::EmptyFlowEntry, position)),
            '[' => {
                let run = cur
                    .rest()
                    .chars()
                    .filter(|c| *c != ' ')
                    .take_while(|c| *c == '[')
                    .count();
                return Err(if depth + 1 + run > MAX_YAML_DEPTH {
                    (
                        ErrorKind::DepthExceeded {
                            max: MAX_YAML_DEPTH,
                        },
                        position,
                    )
                } else {
                    (ErrorKind::NestedFlow, position)
                });
            }
            '{' => {
                return Err((
                    ErrorKind::FlowMapping {
                        key: owner.to_string(),
                    },
                    position,
                ));
            }
            quote @ ('"' | '\'') => {
                scalars.push(quoted(cur, quote)?);
                cur.skip_spaces();
                if !matches!(cur.peek(), Some(',' | ']') | None) {
                    return Err((ErrorKind::TextAfterQuote, cur.pos()));
                }
            }
            _ => {
                let raw = cur.take_until(&[',', ']']);
                if raw.contains(" #") {
                    return Err((ErrorKind::UnterminatedFlow, start));
                }
                scalars.push(plain(raw.trim_end_matches(' '), position)?);
            }
        }
        match cur.peek() {
            Some(',') => {
                after_comma = true;
                cur.bump();
            }
            Some(_) => after_comma = false,
            None => return Err((ErrorKind::UnterminatedFlow, start)),
        }
    }
}

/// The escaped spelling of a double-quoted scalar's text.
fn double_quote_into(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match ESCAPES.iter().find(|(_, value)| *value == c) {
            Some((name, _)) => {
                out.push('\\');
                out.push(*name);
            }
            None => out.push(c),
        }
    }
    out.push('"');
}

/// Write a scalar in the style it carries.
pub(crate) fn write_scalar(out: &mut String, scalar: &Scalar) {
    match scalar.style {
        ScalarStyle::Plain => out.push_str(&scalar.text),
        ScalarStyle::SingleQuoted => {
            out.push('\'');
            for c in scalar.text.chars() {
                if c == '\'' {
                    out.push('\'');
                }
                out.push(c);
            }
            out.push('\'');
        }
        ScalarStyle::DoubleQuoted => double_quote_into(out, &scalar.text),
    }
}

/// Whether a plain-looking `text` would be read by a general YAML reader as
/// something other than a string: a reserved word or a number.
fn resolves_to_non_string(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "null" | "~" | "true" | "false" | "yes" | "no" | "on" | "off" | "y" | "n"
    ) {
        return true;
    }
    if matches!(lower.as_str(), ".inf" | "+.inf" | "-.inf" | ".nan") {
        return true;
    }
    let digits = match lower.strip_prefix(['+', '-']) {
        Some(rest) => rest,
        None => lower.as_str(),
    };
    // Rust's floating-point aliases are case-insensitive; retain that quoting
    // subset while delegating numeric interpretation to the native XSD reader.
    let float = match lower.as_str() {
        "inf" | "+inf" | "infinity" | "+infinity" => "INF",
        "-inf" | "-infinity" => "-INF",
        "nan" | "+nan" | "-nan" => "NaN",
        _ => text,
    };
    purrdf_xsd::numeric::parse_double(float).is_ok()
        || ["0x", "0o", "0b"]
            .iter()
            .any(|prefix| digits.starts_with(prefix))
}

/// Choose the style for text built through a public constructor.
///
/// Unquoted when [`plain_ok`] holds and a general YAML reader would also read
/// a string; double-quoted when the text holds control characters, every one
/// of which must have an escape; single-quoted otherwise. Text that no style
/// can write is refused with the first offending character.
pub(crate) fn choose_style(text: &str) -> Result<ScalarStyle, char> {
    if let Some(bad) = text
        .chars()
        .find(|c| c.is_control() && !ESCAPES.iter().any(|(_, value)| value == c))
    {
        return Err(bad);
    }
    if text.chars().any(char::is_control) {
        return Ok(ScalarStyle::DoubleQuoted);
    }
    if plain_ok(text) && !resolves_to_non_string(text) {
        return Ok(ScalarStyle::Plain);
    }
    Ok(ScalarStyle::SingleQuoted)
}
