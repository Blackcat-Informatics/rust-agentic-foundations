// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The closed word lists of the normal form.

/// Rust's strict and reserved keywords, kept verbatim by every pass.
pub(crate) const KEYWORDS: [&str; 51] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
];

/// The reserved word `gen` and the weak keywords, kept verbatim like the
/// strict ones: a body that binds a local named `union` or `raw` keeps the name.
pub const WEAK_KEYWORDS: [&str; 5] = ["gen", "union", "raw", "safe", "macro_rules"];

/// Primitive type names, kept verbatim: `as u32` and `as u64` are different
/// arithmetic.
pub(crate) const PRIMITIVES: [&str; 17] = [
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64", "bool", "char", "str",
];

/// The value adapters the value form drops: a method called with no arguments
/// that changes how a value is held (borrowed, copied, owned) and not what it
/// is. `next.as_str()` and `next` over a `&str` and over a `usize` are one
/// algorithm.
pub const VALUE_ADAPTERS: [&str; 11] = [
    "as_str",
    "as_ref",
    "as_slice",
    "as_mut",
    "as_deref",
    "borrow",
    "borrow_mut",
    "clone",
    "cloned",
    "copied",
    "to_owned",
];

/// Whether `text` is a keyword of either list.
pub(crate) fn is_keyword(text: &str) -> bool {
    KEYWORDS.contains(&text) || WEAK_KEYWORDS.contains(&text)
}
