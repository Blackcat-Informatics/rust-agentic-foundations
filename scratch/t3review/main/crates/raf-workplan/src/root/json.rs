// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The one field of `cargo metadata` the workplan reads.
//!
//! The scanner validates the whole text as JSON (RFC 8259: no comments, no
//! trailing commas, no raw control character in a string, numbers of the
//! grammar, escapes of the set) while it looks for the member `workspace_root`
//! of the top-level object, and keeps nothing else. A `workspace_root` key
//! anywhere deeper, as a package may carry in its own `metadata` table, is not
//! the field. A text that is not JSON, nested deeper than [`MAX_DEPTH`], not an
//! object at the top, without the member, with the member twice, or with a
//! member that is not a string is an error: the scanner never guesses.

use std::fmt;

/// The deepest nesting accepted. `cargo metadata` nests a few levels; a text
/// deeper than this is not its output.
const MAX_DEPTH: usize = 128;

/// The name of the member.
const MEMBER: &str = "workspace_root";

/// Why the text is not the output of `cargo metadata`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum JsonError {
    /// The text is not valid JSON, at this byte offset.
    Malformed {
        /// The offset of the byte the scanner stopped at.
        offset: usize,
        /// What was expected there.
        reason: &'static str,
    },
    /// The top-level value is not an object.
    NotAnObject,
    /// The top-level object has no member `workspace_root`.
    Missing,
    /// The top-level object has the member twice.
    Duplicate,
    /// The member is not a string.
    NotAString,
    /// The string holds an unpaired surrogate escape, so it is not text.
    NotText,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { offset, reason } => {
                write!(f, "not JSON: {reason} at byte {offset}")
            }
            Self::NotAnObject => write!(f, "the top-level value is not an object"),
            Self::Missing => write!(f, "the top-level object has no `{MEMBER}` member"),
            Self::Duplicate => write!(f, "the top-level object has `{MEMBER}` twice"),
            Self::NotAString => write!(f, "`{MEMBER}` is not a string"),
            Self::NotText => write!(f, "`{MEMBER}` holds an unpaired surrogate escape"),
        }
    }
}

/// The value of the top-level member `workspace_root`.
pub(super) fn workspace_root(text: &str) -> Result<String, JsonError> {
    let mut scan = Scan {
        bytes: text.as_bytes(),
        at: 0,
    };
    scan.space();
    if scan.peek() != Some(b'{') {
        // Text that is JSON, but not an object, is the wrong shape; text that
        // is not JSON is reported as that.
        scan.value(0)?;
        scan.space();
        scan.end()?;
        return Err(JsonError::NotAnObject);
    }
    let root = scan.top_object()?;
    scan.space();
    scan.end()?;
    root.ok_or(JsonError::Missing)
}

/// A cursor over the bytes of a UTF-8 text.
struct Scan<'t> {
    bytes: &'t [u8],
    at: usize,
}

impl Scan<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn fail(&self, reason: &'static str) -> JsonError {
        JsonError::Malformed {
            offset: self.at,
            reason,
        }
    }

    fn space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    fn eat(&mut self, byte: u8, reason: &'static str) -> Result<(), JsonError> {
        if self.peek() == Some(byte) {
            self.at += 1;
            Ok(())
        } else {
            Err(self.fail(reason))
        }
    }

    fn end(&self) -> Result<(), JsonError> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(self.fail("the end of the text"))
        }
    }

    /// The members of the top-level object. The value of `workspace_root` is
    /// returned; every other value is validated and dropped.
    fn top_object(&mut self) -> Result<Option<String>, JsonError> {
        self.eat(b'{', "`{`")?;
        let mut found: Option<String> = None;
        self.space();
        if self.peek() == Some(b'}') {
            self.at += 1;
            return Ok(found);
        }
        loop {
            self.space();
            let key = self.string(true)?.unwrap_or_default();
            self.space();
            self.eat(b':', "`:` after a key")?;
            self.space();
            if key == MEMBER {
                if found.is_some() {
                    return Err(JsonError::Duplicate);
                }
                if self.peek() != Some(b'"') {
                    // Validate the value first: a text that is not JSON is
                    // reported as that, whatever the member holds.
                    self.value(1)?;
                    return Err(JsonError::NotAString);
                }
                found = Some(self.string(true)?.ok_or(JsonError::NotText)?);
            } else {
                self.value(1)?;
            }
            self.space();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b'}') => {
                    self.at += 1;
                    return Ok(found);
                }
                _ => return Err(self.fail("`,` or `}`")),
            }
        }
    }

    /// Validate one value at this nesting depth.
    fn value(&mut self, depth: usize) -> Result<(), JsonError> {
        if depth > MAX_DEPTH {
            return Err(self.fail("nesting no deeper than 128"));
        }
        match self.peek() {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => self.string(false).map(drop),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(b't') => self.word(b"true"),
            Some(b'f') => self.word(b"false"),
            Some(b'n') => self.word(b"null"),
            _ => Err(self.fail("a value")),
        }
    }

    fn object(&mut self, depth: usize) -> Result<(), JsonError> {
        self.eat(b'{', "`{`")?;
        self.space();
        if self.peek() == Some(b'}') {
            self.at += 1;
            return Ok(());
        }
        loop {
            self.space();
            self.string(false)?;
            self.space();
            self.eat(b':', "`:` after a key")?;
            self.space();
            self.value(depth + 1)?;
            self.space();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b'}') => {
                    self.at += 1;
                    return Ok(());
                }
                _ => return Err(self.fail("`,` or `}`")),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<(), JsonError> {
        self.eat(b'[', "`[`")?;
        self.space();
        if self.peek() == Some(b']') {
            self.at += 1;
            return Ok(());
        }
        loop {
            self.space();
            self.value(depth + 1)?;
            self.space();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {
                    self.at += 1;
                    return Ok(());
                }
                _ => return Err(self.fail("`,` or `]`")),
            }
        }
    }

    fn word(&mut self, word: &'static [u8]) -> Result<(), JsonError> {
        if self.bytes[self.at..].starts_with(word) {
            self.at += word.len();
            Ok(())
        } else {
            Err(self.fail("`true`, `false` or `null`"))
        }
    }

    /// `-? (0 | [1-9][0-9]*) (. [0-9]+)? ([eE] [+-]? [0-9]+)?`.
    fn number(&mut self) -> Result<(), JsonError> {
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        match self.peek() {
            Some(b'0') => self.at += 1,
            Some(b'1'..=b'9') => self.digits(),
            _ => return Err(self.fail("a digit")),
        }
        if self.peek() == Some(b'.') {
            self.at += 1;
            self.some_digits()?;
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.at += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.at += 1;
            }
            self.some_digits()?;
        }
        Ok(())
    }

    fn digits(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.at += 1;
        }
    }

    fn some_digits(&mut self) -> Result<(), JsonError> {
        if self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.digits();
            Ok(())
        } else {
            Err(self.fail("a digit"))
        }
    }

    /// One string. With `keep` the decoded text is returned, `None` when an
    /// escape pairs no surrogate; without it the string is only validated and
    /// `None` is returned.
    fn string(&mut self, keep: bool) -> Result<Option<String>, JsonError> {
        self.eat(b'"', "`\"`")?;
        let mut out: Vec<u8> = Vec::new();
        let mut text = true;
        loop {
            let Some(byte) = self.peek() else {
                return Err(self.fail("the end of the string"));
            };
            match byte {
                b'"' => {
                    self.at += 1;
                    break;
                }
                0..=0x1f => return Err(self.fail("an escaped control character")),
                b'\\' => {
                    self.at += 1;
                    let Some(escape) = self.peek() else {
                        return Err(self.fail("an escape"));
                    };
                    self.at += 1;
                    let plain = match escape {
                        b'"' | b'\\' | b'/' => escape,
                        b'b' => 0x08,
                        b'f' => 0x0c,
                        b'n' => b'\n',
                        b'r' => b'\r',
                        b't' => b'\t',
                        b'u' => {
                            let unit = self.unit()?;
                            match self.scalar(unit)? {
                                Some(c) => {
                                    if keep {
                                        let mut buffer = [0; 4];
                                        out.extend_from_slice(
                                            c.encode_utf8(&mut buffer).as_bytes(),
                                        );
                                    }
                                }
                                None => text = false,
                            }
                            continue;
                        }
                        _ => {
                            self.at -= 1;
                            return Err(self.fail("one of \" \\ / b f n r t u after a backslash"));
                        }
                    };
                    if keep {
                        out.push(plain);
                    }
                }
                _ => {
                    // A byte of the text itself; the text is UTF-8, so a
                    // multi-byte character passes through byte by byte.
                    if keep {
                        out.push(byte);
                    }
                    self.at += 1;
                }
            }
        }
        if !keep {
            return Ok(None);
        }
        if !text {
            return Ok(None);
        }
        // The raw bytes came from a `str` and each escape was encoded whole.
        String::from_utf8(out)
            .map(Some)
            .map_err(|_| self.fail("UTF-8 text"))
    }

    /// The four hexadecimal digits after `\u`.
    fn unit(&mut self) -> Result<u16, JsonError> {
        let mut value: u16 = 0;
        for _ in 0..4 {
            let digit = self
                .peek()
                .and_then(|byte| char::from(byte).to_digit(16))
                .ok_or_else(|| self.fail("four hexadecimal digits after `\\u`"))?;
            // A digit is below 16, so it fits.
            value = value * 16 + u16::try_from(digit).unwrap_or(0);
            self.at += 1;
        }
        Ok(value)
    }

    /// The character a code unit stands for, pairing a high surrogate with the
    /// `\u` escape that must follow it. `None` for an unpaired surrogate,
    /// which is valid JSON and not text.
    fn scalar(&mut self, unit: u16) -> Result<Option<char>, JsonError> {
        match unit {
            0xd800..=0xdbff => {
                if self.bytes[self.at..].starts_with(b"\\u") {
                    let save = self.at;
                    self.at += 2;
                    let low = self.unit()?;
                    if (0xdc00..=0xdfff).contains(&low) {
                        let high = u32::from(unit) - 0xd800;
                        let low = u32::from(low) - 0xdc00;
                        return Ok(char::from_u32(0x1_0000 + (high << 10) + low));
                    }
                    // Not a pair: the high surrogate is alone, and the escape
                    // that followed is read again as its own.
                    self.at = save;
                }
                Ok(None)
            }
            0xdc00..=0xdfff => Ok(None),
            other => Ok(char::from_u32(u32::from(other))),
        }
    }
}

#[cfg(test)]
mod tests;
