// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The typed form of a document that satisfies the profile.

/// The lifecycle state of a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Purpose and open questions are written; the candidate contract is in
    /// PROPOSED blocks.
    Considered,
    /// The contract is designed and nothing is built.
    Designed,
    /// The contract is designed and the work is under way.
    Implementing,
    /// The contract is built and bound to code and tests.
    Delivered,
}

impl State {
    /// Every state.
    pub const ALL: [Self; 4] = [
        Self::Considered,
        Self::Designed,
        Self::Implementing,
        Self::Delivered,
    ];

    /// The state named by the value of a `state` key.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|state| state.name() == name)
    }

    /// The value of the `state` key.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Considered => "considered",
            Self::Designed => "designed",
            Self::Implementing => "implementing",
            Self::Delivered => "delivered",
        }
    }
}

/// One of the seven second-level sections of a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilitySection {
    /// `Purpose`.
    Purpose,
    /// `Contract`.
    Contract,
    /// `Usage`.
    Usage,
    /// `Examples`.
    Examples,
    /// `Constraints`.
    Constraints,
    /// `Explanation`.
    Explanation,
    /// `Verification`.
    Verification,
}

impl CapabilitySection {
    /// The sections in the order a capability holds them.
    pub const ALL: [Self; 7] = [
        Self::Purpose,
        Self::Contract,
        Self::Usage,
        Self::Examples,
        Self::Constraints,
        Self::Explanation,
        Self::Verification,
    ];

    /// The section named by the part of an `affects` entry after `#`.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|section| section.slug() == slug)
    }

    /// The heading text of the section.
    #[must_use]
    pub fn heading(self) -> &'static str {
        match self {
            Self::Purpose => "Purpose",
            Self::Contract => "Contract",
            Self::Usage => "Usage",
            Self::Examples => "Examples",
            Self::Constraints => "Constraints",
            Self::Explanation => "Explanation",
            Self::Verification => "Verification",
        }
    }

    /// The lowercase name used in anchors and `affects` entries.
    #[must_use]
    pub fn slug(self) -> &'static str {
        match self {
            Self::Purpose => "purpose",
            Self::Contract => "contract",
            Self::Usage => "usage",
            Self::Examples => "examples",
            Self::Constraints => "constraints",
            Self::Explanation => "explanation",
            Self::Verification => "verification",
        }
    }
}

/// The `bindings` map of a capability: what it is bound to.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Bindings {
    /// Symbols of the code that delivers the capability.
    pub code: Vec<String>,
    /// Symbols of the tests that check it.
    pub tests: Vec<String>,
    /// Symbols of the examples that show it.
    pub examples: Vec<String>,
}

/// One `affects` entry of a change: `cap:<slug>#<section>`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Affects {
    /// The capability id, `cap:<slug>`.
    pub capability: String,
    /// The section of that capability.
    pub section: CapabilitySection,
}

/// How an item of a change ended.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Outcome {
    /// The item was built.
    Implemented,
    /// The item was dropped.
    Dropped,
    /// The item moved to another change record; the slug of that change.
    Moved(String),
}

/// One item of a change.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ChangeItem {
    /// The claim, non-empty.
    pub claim: String,
    /// The outcome, once the change is completed.
    pub outcome: Option<Outcome>,
}

/// The effect of a change on the contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractEffect {
    /// The contract is unchanged.
    None,
    /// The contract changes.
    Changes,
}

/// The keys every document type shares.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Header {
    /// The id, with its prefix.
    pub id: String,
    /// The slug of the id, which is the file name without `.md`.
    pub slug: String,
    /// The title, non-empty.
    pub title: String,
    /// The description, non-empty.
    pub description: String,
    /// The tags, when the key is present.
    pub tags: Option<Vec<String>>,
    /// The RFC 3339 timestamp, when the key is present.
    pub timestamp: Option<String>,
}

/// A capability document.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Capability {
    /// The shared keys.
    pub header: Header,
    /// The state.
    pub state: State,
    /// The bindings.
    pub bindings: Bindings,
    /// The capability ids this one depends on.
    pub depends: Vec<String>,
}

/// A change record.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Change {
    /// The shared keys.
    pub header: Header,
    /// The capability ids the change delivers, non-empty.
    pub changes: Vec<String>,
    /// The sections of capabilities the change affects.
    pub affects: Vec<Affects>,
    /// The effect on the contract.
    pub contract_effect: ContractEffect,
    /// The items.
    pub items: Vec<ChangeItem>,
}

/// An architecture page.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Architecture {
    /// The shared keys.
    pub header: Header,
}

/// A document that satisfies the RAF profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RafDocument {
    /// A capability document.
    Capability(Capability),
    /// A change record.
    Change(Change),
    /// An architecture page.
    Architecture(Architecture),
}
