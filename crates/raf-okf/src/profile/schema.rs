// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The schema table: the one place that lists the keys of each document type.
//!
//! The writer reads the table for key order, and the validator reads the same
//! table for the closed key set, so the two cannot disagree. The order is the
//! order of the architecture page. The optional keys `tags` and `timestamp`
//! sit directly after the shared keys of every type (`description` for the
//! three document types, `id` for the four record types), ahead of the keys
//! that are specific to the type.

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
    /// What a record judges: for evidence `command` or `binding:<path>`, for
    /// an assessment `cap:<slug>#<section>`.
    Subject,
    /// A list of maps with exactly the keys `kind`, `name` and `digest`.
    Inputs,
    /// The verdict of a record, from the set of its type.
    Verdict,
    /// `human` or `agent`.
    Actor,
    /// The command an evidence record ran, a non-empty string.
    Command,
    /// A Git commit id.
    Revision,
    /// A map of measure names to numbers.
    Measures,
    /// A list of crate package names with no duplicate.
    Scope,
    /// A list of maps with exactly the keys `item` and `reason`.
    Exclusions,
    /// A list of maps with `crate`, exactly one of `lines` and `percent`, and
    /// an optional `of`.
    Ceilings,
    /// A list of maps with exactly one of `item` and `file`, and `reason`.
    Exceptions,
    /// A list of maps with exactly the keys `kind` and `subject`.
    DebtFindings,
    /// A list of maps with exactly the keys `item` and `orphans`.
    DebtLines,
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

/// The record types share `type`, `id` and the two optional keys, and carry
/// no `title` or `description`: a record states a result, not prose. Evidence
/// and an assessment share the trace keys `subject`, `inputs`, `verdict` and
/// `by`, in that order, before the keys of their own.
const EVIDENCE: [KeySpec; 13] = [
    required("type", ValueKind::DocType),
    required("id", ValueKind::Id),
    optional("tags", ValueKind::Tags),
    optional("timestamp", ValueKind::Timestamp),
    required("subject", ValueKind::Subject),
    required("inputs", ValueKind::Inputs),
    required("verdict", ValueKind::Verdict),
    required("by", ValueKind::Actor),
    required("for", ValueKind::CapabilityIds),
    optional("command", ValueKind::Command),
    optional("revision", ValueKind::Revision),
    optional("values", ValueKind::Measures),
    optional("limits", ValueKind::Measures),
];

const ASSESSMENT: [KeySpec; 8] = [
    required("type", ValueKind::DocType),
    required("id", ValueKind::Id),
    optional("tags", ValueKind::Tags),
    optional("timestamp", ValueKind::Timestamp),
    required("subject", ValueKind::Subject),
    required("inputs", ValueKind::Inputs),
    required("verdict", ValueKind::Verdict),
    required("by", ValueKind::Actor),
];

const COVERAGE: [KeySpec; 8] = [
    required("type", ValueKind::DocType),
    required("id", ValueKind::Id),
    optional("tags", ValueKind::Tags),
    optional("timestamp", ValueKind::Timestamp),
    required("scope", ValueKind::Scope),
    required("exclusions", ValueKind::Exclusions),
    optional("ceilings", ValueKind::Ceilings),
    optional("exceptions", ValueKind::Exceptions),
];

/// The one slug of the coverage record.
pub(crate) const COVERAGE_SLUG: &str = "default";

const DEBT: [KeySpec; 6] = [
    required("type", ValueKind::DocType),
    required("id", ValueKind::Id),
    optional("tags", ValueKind::Tags),
    optional("timestamp", ValueKind::Timestamp),
    required("findings", ValueKind::DebtFindings),
    required("lines", ValueKind::DebtLines),
];

/// The one slug of the debt record.
pub(crate) const DEBT_SLUG: &str = "default";

/// A document type of the RAF profile.
///
/// The profile closes the set, so the enum is exhaustive: a consumer matches
/// every variant, and a new one is a breaking change. The first three are the
/// documents people write; the last four are records, which state a result
/// in their frontmatter and have no body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocType {
    /// A permanent capability document.
    Capability,
    /// An in-flight change record.
    Change,
    /// A shared architecture page.
    Architecture,
    /// A record of one execution and its verdict.
    Evidence,
    /// A record of one assessment of an authored section.
    Assessment,
    /// The one coverage record of a repository: the crates it covers and
    /// the items it excludes.
    Coverage,
    /// The one debt record of a repository: the findings of strict mode and
    /// the orphan lines it holds as debt. A tool writes it; a person reviews
    /// it.
    Debt,
}

impl DocType {
    /// Every document type.
    pub const ALL: [Self; 7] = [
        Self::Capability,
        Self::Change,
        Self::Architecture,
        Self::Evidence,
        Self::Assessment,
        Self::Coverage,
        Self::Debt,
    ];

    /// The type named by the value of a `type` key, if it is one of
    /// [`DocType::ALL`].
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
            Self::Evidence => "Evidence",
            Self::Assessment => "Assessment",
            Self::Coverage => "Coverage",
            Self::Debt => "Debt",
        }
    }

    /// The indefinite article of [`DocType::name`]: `an` before a vowel,
    /// otherwise `a`.
    ///
    /// ```rust
    /// use raf_okf::DocType;
    ///
    /// assert_eq!(DocType::Capability.article(), "a");
    /// assert_eq!(DocType::Evidence.article(), "an");
    /// ```
    #[must_use]
    pub fn article(self) -> &'static str {
        match self {
            Self::Architecture | Self::Evidence | Self::Assessment => "an",
            Self::Capability | Self::Change | Self::Coverage | Self::Debt => "a",
        }
    }

    /// The repository directory that holds documents of this type.
    #[must_use]
    pub fn directory(self) -> &'static str {
        match self {
            Self::Capability => "docs/capabilities",
            Self::Change => "work/changes",
            Self::Architecture => "docs/architecture",
            Self::Evidence => "work/evidence",
            Self::Assessment => "work/assessments",
            Self::Coverage => "work/coverage",
            Self::Debt => "work/debt",
        }
    }

    /// The prefix of the `id` of a document of this type, with its colon.
    #[must_use]
    pub fn id_prefix(self) -> &'static str {
        match self {
            Self::Capability => "cap:",
            Self::Change => "change:",
            Self::Architecture => "arch:",
            Self::Evidence => "evidence:",
            Self::Assessment => "assessment:",
            Self::Coverage => "coverage:",
            Self::Debt => "debt:",
        }
    }

    /// The one slug a document of this type may have, for a type a
    /// repository holds once: `default` for [`DocType::Coverage`] and
    /// [`DocType::Debt`], so their records are `coverage:default` at
    /// `work/coverage/default.md` and `debt:default` at `work/debt/default.md`.
    /// `None` for every other type.
    #[must_use]
    pub fn singleton(self) -> Option<&'static str> {
        match self {
            Self::Coverage => Some(COVERAGE_SLUG),
            Self::Debt => Some(DEBT_SLUG),
            Self::Capability
            | Self::Change
            | Self::Architecture
            | Self::Evidence
            | Self::Assessment => None,
        }
    }

    /// Whether the type is a record: a result stated in the frontmatter, with
    /// no `title`, no `description` and an empty body.
    #[must_use]
    pub fn is_record(self) -> bool {
        match self {
            Self::Evidence | Self::Assessment | Self::Coverage | Self::Debt => true,
            Self::Capability | Self::Change | Self::Architecture => false,
        }
    }

    /// The keys of this type in the order the writer emits them.
    #[must_use]
    pub fn keys(self) -> &'static [KeySpec] {
        match self {
            Self::Capability => &CAPABILITY,
            Self::Change => &CHANGE,
            Self::Architecture => &ARCHITECTURE,
            Self::Evidence => &EVIDENCE,
            Self::Assessment => &ASSESSMENT,
            Self::Coverage => &COVERAGE,
            Self::Debt => &DEBT,
        }
    }

    /// The names of every type, joined for a message: `Capability, Change,
    /// Architecture, Evidence, Assessment, Coverage or Debt` with `or` as `last`.
    #[must_use]
    pub fn listed(last: &str) -> String {
        let names = Self::ALL.map(Self::name);
        match names.split_last() {
            Some((final_name, rest)) if !rest.is_empty() => {
                format!("{} {last} {final_name}", rest.join(", "))
            }
            _ => names.join(", "),
        }
    }
}
