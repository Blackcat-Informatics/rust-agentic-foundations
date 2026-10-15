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
///
/// The profile closes the set, so the enum is exhaustive: a consumer matches
/// every variant, and a new one is a breaking change.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// The keys every record shares: a record has no `title` and no
/// `description`, because it states a result rather than prose.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RecordHeader {
    /// The id, with its prefix.
    pub id: String,
    /// The slug of the id, which is the file name without `.md`.
    pub slug: String,
    /// The tags, when the key is present.
    pub tags: Option<Vec<String>>,
    /// The RFC 3339 timestamp, when the key is present.
    pub timestamp: Option<String>,
}

/// A section of a capability: `cap:<slug>#<section>`, as the subject of an
/// assessment and the name of a `section` input.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SectionRef {
    /// The capability id, `cap:<slug>`.
    pub capability: String,
    /// The section of that capability.
    pub section: CapabilitySection,
}

impl SectionRef {
    /// The text `cap:<slug>#<section>`.
    #[must_use]
    pub fn text(&self) -> String {
        format!("{}#{}", self.capability, self.section.slug())
    }
}

/// What an input row of a record names.
///
/// The profile closes the set, so the enum is exhaustive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InputKind {
    /// A section of a capability, named `cap:<slug>#<section>`.
    Section,
    /// A symbol of the source, named by its path, such as `raf_okf::Body`.
    Symbol,
    /// An evidence record, named `evidence:<slug>`.
    Record,
}

impl InputKind {
    /// Every kind.
    pub const ALL: [Self; 3] = [Self::Section, Self::Symbol, Self::Record];

    /// The kind named by the value of a `kind` key.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == name)
    }

    /// The value of the `kind` key.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Section => "section",
            Self::Symbol => "symbol",
            Self::Record => "record",
        }
    }
}

/// One input of a record: what it names and the SHA-256 digest of its text
/// when the record was made, as 64 lowercase hexadecimal digits.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Input {
    /// What the row names.
    pub kind: InputKind,
    /// The section, symbol path or evidence id.
    pub name: String,
    /// The digest, 64 lowercase hexadecimal digits.
    pub digest: String,
}

/// Who made a record. The profile names the kind of assessor, never a person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    /// A person.
    Human,
    /// An agent.
    Agent,
}

impl Actor {
    /// Every actor.
    pub const ALL: [Self; 2] = [Self::Human, Self::Agent];

    /// The actor named by the value of a `by` key.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|actor| actor.name() == name)
    }

    /// The value of the `by` key.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Agent => "agent",
        }
    }
}

/// The verdict of an evidence record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceVerdict {
    /// The execution showed what the claims rest on.
    Pass,
    /// It did not.
    Fail,
}

/// The verdict of an assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssessmentVerdict {
    /// The section states the delivered contract clearly.
    Accepted,
    /// It does not.
    Rejected,
}

/// Where the verdict of an evidence record came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceSubject {
    /// The command the `command` key states.
    Command,
    /// A bound test or example, by its path.
    Binding(String),
}

/// A number written in a record: an optional `-`, digits, and an optional
/// fraction of `.` and digits. The text is kept as written, and two numbers
/// are equal when their texts are.
#[derive(Debug, Clone)]
pub struct Number {
    text: String,
    value: f64,
}

impl Number {
    /// The number written as `text`, if the text is one.
    ///
    /// ```rust
    /// use raf_okf::Number;
    ///
    /// assert_eq!(Number::parse("41.5").map(|n| n.value()), Some(41.5));
    /// assert!(Number::parse("-3").is_some());
    /// for text in ["", "1e3", "+1", ".5", "1.", "0x10", "NaN", " 1"] {
    ///     assert!(Number::parse(text).is_none(), "{text:?}");
    /// }
    /// ```
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let unsigned = text.strip_prefix('-').unwrap_or(text);
        let (whole, fraction) = match unsigned.split_once('.') {
            Some((whole, fraction)) => (whole, Some(fraction)),
            None => (unsigned, None),
        };
        let digits = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
        if !digits(whole) || fraction.is_some_and(|part| !digits(part)) {
            return None;
        }
        let value = text.parse::<f64>().ok()?;
        Some(Self {
            text: text.to_string(),
            value,
        })
    }

    /// The text as written.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl Eq for Number {}

/// One named measure of an evidence record: a printed value or its limit.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Measure {
    /// The name.
    pub name: String,
    /// The number.
    pub value: Number,
}

/// An evidence record: one execution, the capabilities it covers, its
/// verdict, and the inputs whose digests it was made against.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct EvidenceRecord {
    /// The shared keys.
    pub header: RecordHeader,
    /// Where the verdict came from.
    pub subject: EvidenceSubject,
    /// The inputs, each a `symbol` row, non-empty.
    pub inputs: Vec<Input>,
    /// The verdict.
    pub verdict: EvidenceVerdict,
    /// Who recorded it.
    pub by: Actor,
    /// The capability ids the evidence covers (the `for` key), non-empty.
    pub covers: Vec<String>,
    /// The command that ran, present exactly when the subject is
    /// [`EvidenceSubject::Command`].
    pub command: Option<String>,
    /// The commit the execution ran at, when it was given.
    pub revision: Option<String>,
    /// The measured values, in author order; empty when the key is absent.
    pub values: Vec<Measure>,
    /// The limits, each naming a measured value; empty when the key is absent.
    pub limits: Vec<Measure>,
}

/// An assessment record: the verdict on one authored section and the inputs
/// it was given.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AssessmentRecord {
    /// The shared keys.
    pub header: RecordHeader,
    /// The section assessed.
    pub subject: SectionRef,
    /// The inputs, one of them the `section` row of the subject.
    pub inputs: Vec<Input>,
    /// The verdict.
    pub verdict: AssessmentVerdict,
    /// Who assessed.
    pub by: Actor,
}

/// What a coverage exclusion takes out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExclusionItem {
    /// One item, by its path, such as `raf_okf::Body`.
    Item(String),
    /// Every item below a module, written `<module path>::*`; the path held
    /// here has no `::*`, and names a module below a crate root.
    Prefix(String),
}

/// One coverage exclusion: what it takes out and why.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Exclusion {
    /// The item or prefix.
    pub item: ExclusionItem,
    /// The reason, non-empty.
    pub reason: String,
}

/// The coverage record of a repository.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CoverageRecord {
    /// The shared keys.
    pub header: RecordHeader,
    /// The package names of the crates whose coverage is enforced; empty to
    /// enforce none.
    pub scope: Vec<String>,
    /// The exclusions, in author order.
    pub exclusions: Vec<Exclusion>,
}

/// A document that satisfies the RAF profile.
///
/// The profile closes the set of types, so the enum is exhaustive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RafDocument {
    /// A capability document.
    Capability(Capability),
    /// A change record.
    Change(Change),
    /// An architecture page.
    Architecture(Architecture),
    /// An evidence record.
    Evidence(EvidenceRecord),
    /// An assessment record.
    Assessment(AssessmentRecord),
    /// The coverage record.
    Coverage(CoverageRecord),
}
