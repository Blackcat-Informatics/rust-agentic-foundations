// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Literal input references in the closed compiler capture profile.

use raf_source::RelPath;

use super::{CaptureError, mismatch};

pub(super) fn imports(
    source: &[u8],
    path: &RelPath,
    mut visit: impl FnMut(&str) -> Result<(), CaptureError>,
) -> Result<(), CaptureError> {
    let fail = |reason| mismatch(format!("schema {path} {reason}"));
    let mut input = std::str::from_utf8(source)
        .map_err(CaptureError::Encoding)?
        .as_bytes();

    loop {
        input = trivia(input);
        let Some((&first, rest)) = input.split_first() else {
            return Ok(());
        };
        input = match first {
            b'"' => {
                quoted(rest)
                    .ok_or_else(|| fail("has an unclosed string"))?
                    .1
            }
            b'`' => line_tail(rest, |byte| matches!(byte, b'\r' | b'\n')),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let (identifier, after) = identifier(input);
                if !matches!(identifier, b"import" | b"embed") {
                    input = after;
                    continue;
                }
                let after = trivia(after);
                match after {
                    [b'"' | b'`', ..] if identifier == b"embed" => {
                        return Err(fail("uses embed, which the capture profile does not admit"));
                    }
                    [b'`', ..] => return Err(fail("must use a double-quoted import path")),
                    [b'"', rest @ ..] => {
                        let (name, after) =
                            quoted(rest).ok_or_else(|| fail("has an unclosed import path"))?;
                        if name.contains(&b'\\') {
                            return Err(fail("must use an unescaped import path"));
                        }
                        visit(std::str::from_utf8(name).map_err(CaptureError::Encoding)?)?;
                        after
                    }
                    _ => after,
                }
            }
            _ => rest,
        };
    }
}

fn trivia(mut input: &[u8]) -> &[u8] {
    loop {
        while let [b'\t'..=b'\r' | b' ', rest @ ..] = input {
            input = rest;
        }
        if let Some(rest) = input.strip_prefix(b"\xef\xbb\xbf") {
            input = rest;
        } else if input.starts_with(b"#") {
            input = line_tail(input, |byte| byte == b'\n');
        } else {
            return input;
        }
    }
}

fn line_tail(input: &[u8], end: impl Fn(u8) -> bool) -> &[u8] {
    input
        .iter()
        .position(|&byte| end(byte))
        .map_or(&[], |at| &input[at..])
}

fn identifier(input: &[u8]) -> (&[u8], &[u8]) {
    let end = input
        .iter()
        .position(|&byte| !byte.is_ascii_alphanumeric() && byte != b'_')
        .unwrap_or(input.len());
    input.split_at(end)
}

fn quoted(input: &[u8]) -> Option<(&[u8], &[u8])> {
    let mut escaped = false;
    let end = input.iter().position(|&byte| {
        let close = byte == b'"' && !escaped;
        escaped = byte == b'\\' && !escaped;
        close
    })?;
    Some((&input[..end], &input[end + 1..]))
}
