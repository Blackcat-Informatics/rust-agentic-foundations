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
    if matches!(radix, 10 | 16) {
        // Keep the Rust separators/suffix grammar here. Only significant
        // digits that can fit u128 reach the native numeric reader; larger
        // syntactically valid literals remain integers without a value.
        let limit = if radix == 10 { 39 } else { 32 };
        let mut buffer = [b'0'; 64];
        let mut significant = 0_usize;
        let mut seen = false;
        for byte in digits.bytes().filter(|byte| *byte != b'_') {
            let digit = if radix == 16 {
                purrdf_hash::hex::nibble(byte)?
            } else if byte.is_ascii_digit() {
                byte - b'0'
            } else {
                return None;
            };
            seen = true;
            if digit != 0 || significant != 0 {
                if significant < limit {
                    buffer[significant] = byte;
                }
                significant = (significant + 1).min(limit + 1);
            }
        }
        if !seen {
            return None;
        }
        let value = if significant > limit {
            None
        } else if radix == 10 {
            let text = std::str::from_utf8(&buffer[..significant.max(1)])
                .expect("validated decimal bytes");
            purrdf_xsd::numeric::parse_integer_typed(
                text,
                purrdf_xsd::XsdDatatype::NonNegativeInteger,
            )
            .expect("validated bounded decimal digits must reach native integer parsing")
            .into_bigint()
            .unsigned_abs_u128()
        } else {
            buffer.copy_within(..significant, 64 - significant);
            buffer[..64 - significant].fill(b'0');
            let text = std::str::from_utf8(&buffer).expect("validated hexadecimal bytes");
            let bytes = purrdf_hash::hex::decode_32(text)
                .expect("validated padded hexadecimal digits must reach native decoding");
            Some(u128::from_be_bytes(bytes.as_chunks::<16>().0[1]))
        };
        return Some(Integer { value, suffix });
    }
    based(digits, suffix, radix)
}

/// Rust binary/octal grammar around the native bounded scalar radix reader.
fn based<'a>(digits: &str, suffix: &'a str, radix: u32) -> Option<Integer<'a>> {
    let limit = if radix == 2 { 128 } else { 43 };
    let mut scalar = [b'0'; 130];
    scalar[1] = if radix == 2 { b'b' } else { b'o' };
    let mut significant = 0_usize;
    let mut seen = false;
    for byte in digits.bytes().filter(|byte| *byte != b'_') {
        let digit = purrdf_hash::hex::nibble(byte).filter(|digit| u32::from(*digit) < radix)?;
        seen = true;
        if digit != 0 || significant != 0 {
            if significant < limit {
                scalar[2 + significant] = byte;
            }
            significant = (significant + 1).min(limit + 1);
        }
    }
    if !seen {
        return None;
    }
    if significant > limit {
        return Some(Integer {
            value: None,
            suffix,
        });
    }
    let scalar = std::str::from_utf8(&scalar[..2 + significant.max(1)])
        .expect("validated binary/octal bytes");
    let resolved = purrdf_lex::yaml::read_with(
        scalar,
        purrdf_lex::yaml::Limits {
            max_nodes: 1,
            max_depth: 0,
            aliases: false,
            scalar_keys: false,
        },
    )
    .expect("a validated bounded scalar must reach native radix resolution");
    let value = match &resolved {
        purrdf_lex::json::Value::Number(number) => Some(
            purrdf_xsd::numeric::parse_integer_typed(
                number.lexeme(),
                purrdf_xsd::XsdDatatype::NonNegativeInteger,
            )
            .expect("native radix resolution yields an unsigned decimal")
            .into_bigint()
            .unsigned_abs_u128()
            .expect("native radix values fit u128"),
        ),
        purrdf_lex::json::Value::String(text) => {
            // The native scalar resolver preserves the validated radix text
            // when its magnitude overflows u128.
            assert_eq!(
                text.as_str(),
                scalar,
                "unexpected native radix scalar rewrite"
            );
            None
        }
        _ => panic!("native resolution changed a validated Rust radix scalar's kind"),
    };
    Some(Integer { value, suffix })
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
        let maximum = u128::MAX.to_string();
        for spelling in [
            maximum.clone(),
            format!("0x{}", purrdf_hash::hex::Upper(&u128::MAX.to_be_bytes())),
            format!("0x{}", purrdf_hash::hex::Lower(&u128::MAX.to_be_bytes())),
            format!("{}_{}u128", "0".repeat(256), maximum),
        ] {
            assert_eq!(parse_integer(&spelling).unwrap().value, Some(u128::MAX));
        }
        for spelling in [
            huge.to_owned(),
            "0x1_00000000_00000000_00000000_00000000".to_owned(),
            format!("1{}", "0".repeat(256)),
        ] {
            assert_eq!(parse_integer(&spelling).unwrap().value, None);
            assert_eq!(kind(&spelling), "$int");
        }
        for spelling in [
            "0x_",
            "0xG",
            "0x100000000000000000000000000000000G",
            "1_2e3",
        ] {
            assert!(parse_integer(spelling).is_none(), "{spelling}");
        }
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

    #[test]
    fn binary_and_octal_native_values_keep_exact_width_and_syntax_neighbors() {
        let maximum = u128::MAX.to_string();
        for (prefix, digits, overflow, invalid) in [
            ("0b", "1".repeat(128), format!("1{}", "0".repeat(128)), '2'),
            (
                "0o",
                format!("3{}", "7".repeat(42)),
                format!("4{}", "0".repeat(42)),
                '8',
            ),
        ] {
            let at = format!("{prefix}{digits}");
            assert_eq!(parse_integer(&at).unwrap().value, Some(u128::MAX));
            assert_eq!(respell(&at), maximum);
            let padded = format!("{prefix}{}_{digits}_u128", "0".repeat(256));
            assert_eq!(parse_integer(&padded).unwrap().value, Some(u128::MAX));
            assert_eq!(respell(&padded), format!("{maximum}u128"));
            let beyond = format!("{prefix}{overflow}");
            assert_eq!(parse_integer(&beyond).unwrap().value, None);
            assert_eq!(kind(&beyond), "$int");
            assert_eq!(respell(&beyond), beyond);
            assert!(parse_integer(&format!("{beyond}{invalid}")).is_none());
            assert_eq!(
                parse_integer(&format!("{prefix}___0___")).unwrap().value,
                Some(0)
            );
            assert!(parse_integer(&format!("{prefix}___")).is_none());
        }
    }
}
