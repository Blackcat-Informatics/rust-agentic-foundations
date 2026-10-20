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
//!
//! The profile has six types: the documents `Capability`, `Change` and
//! `Architecture`, and the records `Evidence`, `Assessment` and `Coverage`
//! ([`DocType::ALL`]). A record states a result in its frontmatter and has no
//! title, description or body; evidence and an assessment share the trace keys
//! `subject`, `inputs`, `verdict` and `by`. A record is built with the
//! constructor of its type, such as [`EvidenceRecord::new`], and written by
//! its `to_document`, whose text the profile reads back as the same record. A
//! capability names the evidence a statement rests on with a claim marker,
//! which [`Body::claims`] reads, and the tests that show a statement with a
//! tested-by marker, which [`Body::tested_by`] reads:
//!
//! ```rust
//! use raf_okf::{Body, Document, EvidenceVerdict, Profile, RafDocument};
//!
//! let record = "\
//! ---
//! type: Evidence
//! id: evidence:memory-bound
//! subject: command
//! inputs:
//!   - kind: symbol
//!     name: raf_okf::Document::parse
//!     digest: 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08
//! verdict: pass
//! by: agent
//! for: [cap:okf-profile]
//! command: \"cargo bench parse\"
//! values:
//!   peak_mib: 41
//! limits:
//!   peak_mib: 64
//! ---
//! ";
//! let document = Document::parse(record).expect("the text is a document");
//! match Profile.validate(&document, "work/evidence/memory-bound.md") {
//!     Ok(RafDocument::Evidence(evidence)) => assert_eq!(evidence.verdict, EvidenceVerdict::Pass),
//!     Ok(other) => panic!("not evidence: {other:?}"),
//!     Err(violations) => panic!("{violations:?}"),
//! }
//!
//! let prose = "Parsing stays under 64 MiB.\n<!-- raf:claim evidence=memory-bound -->\n";
//! let body = Body::parse(prose).expect("a body");
//! assert_eq!(body.claims()[0].evidence(), ["memory-bound"]);
//!
//! let prose = "Parsing stays under 64 MiB.\n<!-- raf:tested-by demo::test_crate::bounds::peak -->\n";
//! let body = Body::parse(prose).expect("a body");
//! assert_eq!(body.tested_by()[0].paths(), ["demo::test_crate::bounds::peak"]);
//! ```

mod body;
mod document;
mod error;
mod frontmatter;
mod index;
mod limits;
mod profile;
mod span;
mod work;

pub use body::{
    Body, Claim, FencedBlock, GeneratedBlock, Heading, Link, LinkError, LinkForm, LinkTarget,
    ProposedBlock, ProseLine, Section, SplitLink, TestedBy, is_kebab,
};
pub use document::Document;
pub use error::{ConstructError, ErrorKind, ErrorTag, ParseError, SpliceError};
pub use frontmatter::scalar::{ESCAPES, plain_ok};
pub use frontmatter::value::{
    Comments, Entry, Frontmatter, ItemValue, List, ListForm, ListItem, Map, Scalar, ScalarStyle,
    Value,
};
pub use index::{Index, is_capability_index};
pub use limits::{MAX_BLOCK_NESTING, MAX_TEXT_BYTES, MAX_YAML_DEPTH};
pub use profile::{
    Actor, Affects, Architecture, AssessmentRecord, AssessmentVerdict, Bindings, Capability,
    CapabilitySection, Change, ChangeItem, ContractEffect, CoverageRecord, DocType, EvidenceRecord,
    EvidenceSubject, EvidenceVerdict, Exclusion, ExclusionItem, Header, Input, InputKind, KeySpec,
    Location, Measure, Number, Outcome, Profile, RafDocument, RecordHeader, Rule, RuleTag,
    SectionRef, State, Strictness, TimestampError, ValueKind, Violation, validate_timestamp,
};
pub use span::Span;
