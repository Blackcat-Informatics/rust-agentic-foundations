// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reader and deterministic writer for Markdown documents with a restricted
//! YAML frontmatter.
//!
//! The crate reads and writes text only. It touches no file and runs no
//! process, so a caller supplies the document text as a string.
//!
//! The frontmatter accepts a small subset of YAML and refuses the rest with a
//! positioned error:
//!
//! * block mappings, nested by two spaces, with keys of letters, digits, `_`
//!   and `-`;
//! * block or flow sequences of scalars, and block sequences of block
//!   mappings;
//! * plain, single-quoted and double-quoted scalars, where the double-quoted
//!   escapes are exactly [`ESCAPES`];
//! * numbers, booleans and null, which are read as the strings that were
//!   written;
//! * full-line comments, and a trailing ` # comment` that is not part of the
//!   value.
//!
//! Flow mappings, duplicate keys at any level, anchors, aliases, tags, merge
//! keys and block scalars are refused, as are texts over [`MAX_TEXT_BYTES`]
//! and nesting over [`MAX_YAML_DEPTH`].
//!
//! ```
//! use raf_okf::Document;
//!
//! let text = "---\n# header\ntype: Architecture\nid: arch:example\ntitle: \"Example\"\ndescription: \"One line.\"\n---\n\n# Example\n";
//! let document = Document::parse(text)?;
//! assert_eq!(document.render(), text);
//! # Ok::<(), raf_okf::ParseError>(())
//! ```

mod body;
mod document;
mod error;
mod frontmatter;
mod limits;
mod profile;
mod span;

pub use body::Body;
pub use document::Document;
pub use error::{ConstructError, ErrorKind, ParseError};
pub use frontmatter::scalar::{ESCAPES, plain_ok};
pub use frontmatter::value::{
    Comments, Entry, Frontmatter, ItemValue, List, ListForm, ListItem, Map, Scalar, ScalarStyle,
    Value,
};
pub use limits::{MAX_TEXT_BYTES, MAX_YAML_DEPTH};
pub use profile::{DocType, KeySpec, TimestampError, ValueKind, validate_timestamp};
pub use span::Span;
