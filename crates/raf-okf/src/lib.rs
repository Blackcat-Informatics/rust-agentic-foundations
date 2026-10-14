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
//! [`Profile::validate`] checks a document against the RAF profile and returns
//! its typed form or every violation:
//!
//! ```rust
//! use raf_okf::{Document, Profile, RafDocument};
//!
//! const TEXT: &str = "\
//! ---
//! type: Architecture
//! id: arch:example
//! title: \"Example\"
//! description: \"One line.\"
//! ---
//!
//! # Example
//!
//! What this page says.
//! ";
//!
//! let document = Document::parse(TEXT).expect("the text is a document");
//! assert_eq!(document.render(), TEXT);
//! match Profile.validate(&document, "docs/architecture/example.md") {
//!     Ok(RafDocument::Architecture(page)) => assert_eq!(page.header.title, "Example"),
//!     Ok(other) => panic!("not an architecture page: {other:?}"),
//!     Err(violations) => panic!("{violations:?}"),
//! }
//! ```

mod body;
mod document;
mod error;
mod frontmatter;
mod index;
mod limits;
mod profile;
mod span;

pub use body::{
    Body, FencedBlock, GeneratedBlock, Heading, Link, LinkError, LinkForm, LinkTarget,
    ProposedBlock, ProseLine, Section, SplitLink,
};
pub use document::Document;
pub use error::{ConstructError, ErrorKind, ParseError, SpliceError};
pub use frontmatter::scalar::{ESCAPES, plain_ok};
pub use frontmatter::value::{
    Comments, Entry, Frontmatter, ItemValue, List, ListForm, ListItem, Map, Scalar, ScalarStyle,
    Value,
};
pub use index::{Index, is_capability_index};
pub use limits::{MAX_BLOCK_NESTING, MAX_TEXT_BYTES, MAX_YAML_DEPTH};
pub use profile::{
    Affects, Architecture, Bindings, Capability, CapabilitySection, Change, ChangeItem,
    ContractEffect, DocType, Header, KeySpec, Location, Outcome, Profile, RafDocument, Rule, State,
    TimestampError, ValueKind, Violation, validate_timestamp,
};
pub use span::Span;
