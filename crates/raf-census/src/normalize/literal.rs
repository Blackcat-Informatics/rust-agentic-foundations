// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Literal spellings: integers by value, and the kind a literal abstracts to.

/// The integer suffixes, the longest of a shared tail first.
const SUFFIXES: [&str; 12] = [
    "u128", "i128", "usize", "isize", "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8",
];

/// An integer literal split into its value and its type suffix.
pub(crate) struct Integer<'a> {
    /// The value, or `None` beyond `u128`.
    pub(crate) value: Option<u128>,
    /// The type suffix as written, empty when there is none.
    pub(crate) suffix: &'a str,
}

/// Reads `text` as an integer literal by syntax: any spelling of an integer
/// (`0xcbf2_9ce4_8422_2325`, `14695981039346656037`, `0o17u8`, `0b1010`),
/// whatever its size. `None` for floats and everything that is not a number.
pub(crate) fn parse_integer(text: &str) -> Option<Integer<'_>> {
    let (radix, digits) = if let Some(rest) = text.strip_prefix("0x") {
        (16, rest)
    } else if let Some(rest) = text.strip_prefix("0o") {
        (8, rest)
    } else if let Some(rest) = text.strip_prefix("0b") {
        (2, rest)
    } else if text.starts_with(|first: char| first.is_ascii_digit()) {
        // A float fails below: `.`, `e` and `f` are not decimal digits.
        (10, text)
    } else {
        return None;
    };
    let (digits, suffix) = SUFFIXES
        .iter()
        .find_map(|suffix| digits.strip_suffix(suffix).map(|rest| (rest, *suffix)))
        .unwrap_or((digits, ""));
    let mut value: Option<u128> = Some(0);
    let mut seen = false;
    for character in digits.chars() {
        if character == '_' {
            continue;
        }
        let digit = character.to_digit(radix)?;
        seen = true;
        value = value.and_then(|value| {
            value
                .checked_mul(u128::from(radix))?
                .checked_add(u128::from(digit))
        });
    }
    seen.then_some(Integer { value, suffix })
}

/// The canonical spelling of a literal as the lexer records it: an integer is
/// its decimal value followed by its suffix, so `0xff` and `255` and `1_000`
/// and `1000` are one token. An integer beyond `u128` keeps its digits without
/// underscores. Everything else is as written.
pub(crate) fn respell(text: &str) -> String {
    match parse_integer(text) {
        Some(Integer {
            value: Some(value),
            suffix,
        }) => format!("{value}{suffix}"),
        Some(Integer { value: None, .. }) => text.replace('_', ""),
        None => text.to_owned(),
    }
}

/// The kind a literal is abstracted to.
pub(crate) fn kind(text: &str) -> &'static str {
    let bytes = text.as_bytes();
    match bytes.first() {
        Some(b'"' | b'r') => "$str",
        Some(b'\'') => "$char",
        Some(b'b') if bytes.get(1) == Some(&b'\'') => "$byte",
        Some(b'b') => "$bytes",
        Some(b'c') => "$cstr",
        Some(b'0'..=b'9') => {
            if parse_integer(text).is_some() {
                "$int"
            } else {
                "$float"
            }
        }
        _ => "$lit",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spellings_of_one_value_respell_alike() {
        for spelling in ["255", "0xff", "0xFF", "0o377", "0b1111_1111", "2_5_5"] {
            assert_eq!(respell(spelling), "255", "{spelling}");
        }
        assert_eq!(respell("0xffu8"), "255u8");
        assert_eq!(respell("1usize"), "1usize");
        assert_eq!(kind("1usize"), "$int");
        assert_eq!(respell("1_000"), respell("1000"));
        assert_ne!(respell("255"), respell("256"));
    }

    #[test]
    fn floats_and_strings_are_not_integers() {
        for text in ["1.5", "1e3", "2f32", "\"7\"", "'7'", "x"] {
            assert!(parse_integer(text).is_none(), "{text}");
        }
        assert_eq!(respell("1.5"), "1.5");
        assert_eq!(kind("1.5"), "$float");
        assert_eq!(kind("1e3"), "$float");
    }

    #[test]
    fn an_integer_beyond_u128_is_still_an_int() {
        let huge = "340282366920938463463374607431768211456";
        let parsed = parse_integer(huge).expect("an integer by syntax");
        assert!(parsed.value.is_none());
        assert_eq!(kind(huge), "$int");
        assert_eq!(
            respell("340_282_366_920_938_463_463_374_607_431_768_211_456"),
            huge
        );
    }

    #[test]
    fn each_literal_kind_has_its_name() {
        assert_eq!(kind("\"s\""), "$str");
        assert_eq!(kind("r#\"s\"#"), "$str");
        assert_eq!(kind("'c'"), "$char");
        assert_eq!(kind("b'c'"), "$byte");
        assert_eq!(kind("b\"c\""), "$bytes");
        assert_eq!(kind("c\"c\""), "$cstr");
        assert_eq!(kind("7u8"), "$int");
    }
}
