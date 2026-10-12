// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The resource bounds of the reader.

/// The largest document text, in bytes, that [`crate::Document::parse`]
/// accepts. A text of exactly this size parses.
pub const MAX_TEXT_BYTES: usize = 1024 * 1024;

/// The deepest nesting of collections the frontmatter may have. The top-level
/// mapping is level 1, and each mapping, list or flow list inside it adds one.
pub const MAX_YAML_DEPTH: usize = 32;

/// The deepest nesting of block quotes and list items a body may have. A
/// quote marker and a list marker each add one level.
pub const MAX_BLOCK_NESTING: usize = 64;
