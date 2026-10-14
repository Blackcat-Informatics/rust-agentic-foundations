// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The schema table: the one place that lists the keys of each document type.
//!
//! The writer reads the table for key order, and the validator reads the same
//! table for the closed key set, so the two cannot disagree. The order is the
//! order of the architecture page. The optional keys `tags` and `timestamp`
//! sit directly after `description` in every type, ahead of the keys that are
//! specific to the type.

/// The kind of value a key holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValueKind {
    /// The document type name.
    DocType,
    /// An identifier with the prefix of the document type.
    Id,
    /// A non-empty string.
    Text,
    /// A list of non-empty strings with no duplicate.
    Tags,
    /// An RFC 3339 date and time.
    Timestamp,
    /// The lifecycle state of a capability.
    State,
    /// A map with exactly the keys `code`, `tests` and `examples`, each a list.
    Bindings,
    /// A list of capability identifiers.
    CapabilityIds,
    /// A list of `cap:<slug>#<section>` entries.
    Affects,
    /// `none` or `changes`.
    ContractEffect,
    /// A list of maps with a claim and an optional outcome.
    Items,
}

/// One key of a document type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeySpec {
    /// The key.
    pub key: &'static str,
    /// Whether every document of the type must carry the key.
    pub required: bool,
    /// The kind of value the key holds.
    pub value: ValueKind,
}

const fn required(key: &'static str, value: ValueKind) -> KeySpec {
    KeySpec {
        key,
        required: true,
        value,
    }
}

const fn optional(key: &'static str, value: ValueKind) -> KeySpec {
    KeySpec {
        key,
        required: false,
        value,
    }
}

const CAPABILITY: [KeySpec; 9] = [
    required("type", ValueKind::DocType),
    required("id", ValueKind::Id),
    required("title", ValueKind::Text),
    required("description", ValueKind::Text),
    optional("tags", ValueKind::Tags),
    optional("timestamp", ValueKind::Timestamp),
    required("state", ValueKind::State),
    required("bindings", ValueKind::Bindings),
    required("depends", ValueKind::CapabilityIds),
];

const CHANGE: [KeySpec; 10] = [
    required("type", ValueKind::DocType),
    required("id", ValueKind::Id),
    required("title", ValueKind::Text),
    required("description", ValueKind::Text),
    optional("tags", ValueKind::Tags),
    optional("timestamp", ValueKind::Timestamp),
    required("changes", ValueKind::CapabilityIds),
    required("affects", ValueKind::Affects),
    required("contract_effect", ValueKind::ContractEffect),
    required("items", ValueKind::Items),
];

const ARCHITECTURE: [KeySpec; 6] = [
    required("type", ValueKind::DocType),
    required("id", ValueKind::Id),
    required("title", ValueKind::Text),
    required("description", ValueKind::Text),
    optional("tags", ValueKind::Tags),
    optional("timestamp", ValueKind::Timestamp),
];

/// A document type of the RAF profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocType {
    /// A permanent capability document.
    Capability,
    /// An in-flight change record.
    Change,
    /// A shared architecture page.
    Architecture,
}

impl DocType {
    /// Every document type.
    pub const ALL: [Self; 3] = [Self::Capability, Self::Change, Self::Architecture];

    /// The type named by the value of a `type` key, if it is one of the three.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|doc_type| doc_type.name() == name)
    }

    /// The value of the `type` key.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Capability => "Capability",
            Self::Change => "Change",
            Self::Architecture => "Architecture",
        }
    }

    /// The repository directory that holds documents of this type.
    #[must_use]
    pub fn directory(self) -> &'static str {
        match self {
            Self::Capability => "docs/capabilities",
            Self::Change => "work/changes",
            Self::Architecture => "docs/architecture",
        }
    }

    /// The prefix of the `id` of a document of this type, with its colon.
    #[must_use]
    pub fn id_prefix(self) -> &'static str {
        match self {
            Self::Capability => "cap:",
            Self::Change => "change:",
            Self::Architecture => "arch:",
        }
    }

    /// The keys of this type in the order the writer emits them.
    #[must_use]
    pub fn keys(self) -> &'static [KeySpec] {
        match self {
            Self::Capability => &CAPABILITY,
            Self::Change => &CHANGE,
            Self::Architecture => &ARCHITECTURE,
        }
    }
}
