// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Violations of the profile.

use std::fmt;

use crate::body::LinkForm;
use crate::profile::schema::DocType;
use crate::profile::timestamp::TimestampError;
use crate::profile::typed::State;

/// Where a violation is.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Location {
    /// A frontmatter key.
    Key(String),
    /// A second-level section, by its heading text.
    Section(String),
    /// A line of the document text.
    Line(usize),
    /// The document as a whole.
    Document,
}

/// The rule a violation breaks. The data of each rule is what its message is
/// made of.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rule {
    /// A required key is missing.
    MissingKey {
        /// The key.
        key: String,
    },
    /// A key outside the closed key set of the type.
    UnknownKey {
        /// The key.
        key: String,
    },
    /// The base OKF key `resource`, which the closed key sets leave out.
    ResourceKey {
        /// The type whose key set it is outside.
        doc_type: DocType,
    },
    /// A `type` other than `Capability`, `Change` or `Architecture`.
    UnknownType {
        /// The value found.
        found: String,
    },
    /// The document is not directly in the directory of its type.
    WrongDirectory {
        /// The declared type.
        declared: DocType,
    },
    /// A value of the wrong shape.
    WrongShape {
        /// What the key must hold.
        expected: &'static str,
    },
    /// An id that is not `<prefix><kebab-slug>`.
    InvalidId {
        /// The prefix the type requires.
        prefix: &'static str,
    },
    /// An id whose slug is not the file name without `.md`.
    IdFileName {
        /// The id.
        id: String,
        /// The file stem.
        stem: String,
    },
    /// An empty or non-string `title` or `description`.
    EmptyText,
    /// A `state` outside the four states.
    BadState {
        /// The value found.
        found: String,
    },
    /// `bindings` without exactly the keys `code`, `tests` and `examples`.
    BindingsKeys,
    /// A binding that is not a list of strings.
    BindingNotAList {
        /// The binding.
        binding: String,
    },
    /// A binding that holds entries in a state that binds nothing.
    BindingNotEmpty {
        /// The state.
        state: State,
        /// The binding.
        binding: String,
    },
    /// A delivered capability with nothing bound where it must bind.
    BindingRequired {
        /// The binding.
        binding: String,
    },
    /// An entry that is not `cap:<kebab-slug>`.
    BadCapabilityId {
        /// The entry.
        entry: String,
    },
    /// An empty `changes` list.
    EmptyChanges,
    /// An `affects` entry that is not `cap:<slug>#<section>`.
    BadAffects {
        /// The entry.
        entry: String,
    },
    /// An `affects` entry with a section outside the seven.
    BadAffectsSection {
        /// The entry.
        entry: String,
        /// The section.
        section: String,
    },
    /// An `affects` entry whose capability is not in `changes`.
    AffectsNotInChanges {
        /// The entry.
        entry: String,
    },
    /// A `contract_effect` other than `none` and `changes`.
    BadContractEffect {
        /// The value found.
        found: String,
    },
    /// An item key other than `claim` and `outcome`.
    ItemUnknownKey {
        /// The key.
        key: String,
    },
    /// An item with no non-empty `claim`.
    ItemClaimEmpty,
    /// An outcome outside `implemented`, `dropped` and `moved:change:<slug>`.
    BadOutcome {
        /// The value found.
        found: String,
    },
    /// An empty tag.
    EmptyTag,
    /// A tag that occurs twice.
    DuplicateTag {
        /// The tag.
        tag: String,
    },
    /// A timestamp that is not RFC 3339.
    BadTimestamp {
        /// Why.
        reason: TimestampError,
    },
    /// A required second-level section is missing.
    SectionMissing,
    /// A second-level section the type does not have.
    SectionUnexpected,
    /// A second-level section that appears more than once.
    SectionDuplicated,
    /// Sections not in the order of the type.
    SectionsOutOfOrder {
        /// The headings in the order found.
        found: Vec<String>,
    },
    /// A section with nothing left after HTML comments and generated blocks
    /// are removed.
    SectionEmpty,
    /// An architecture body with nothing left after HTML comments are removed.
    BodyEmpty,
    /// A PROPOSED block in a document that is not a capability.
    ProposedOutsideCapability {
        /// The change slug the block names.
        slug: String,
    },
    /// A PROPOSED block before the first second-level heading.
    ProposedOutsideSection {
        /// The change slug the block names.
        slug: String,
    },
    /// An unfinished-work token in prose or in a frontmatter value.
    Placeholder {
        /// The token.
        token: String,
    },
    /// A link or image split across lines.
    SplitLink {
        /// The form of the link.
        form: LinkForm,
    },
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey { key } => write!(f, "missing required key `{key}`"),
            Self::UnknownKey { key } => write!(f, "unknown key `{key}`"),
            Self::ResourceKey { doc_type } => {
                let keys: Vec<&str> = doc_type.keys().iter().map(|spec| spec.key).collect();
                write!(
                    f,
                    "`resource` is not a key of a {} document; the keys are {}",
                    doc_type.name(),
                    keys.join(", ")
                )
            }
            Self::UnknownType { found } => write!(
                f,
                "bad type `{found}`; the types are Capability, Change and Architecture"
            ),
            Self::WrongDirectory { declared } => write!(
                f,
                "a {} document belongs directly in {}",
                declared.name(),
                declared.directory()
            ),
            Self::WrongShape { expected } => write!(f, "must be {expected}"),
            Self::InvalidId { prefix } => {
                write!(f, "id must be `{prefix}<slug>` with a lowercase kebab slug")
            }
            Self::IdFileName { id, stem } => {
                write!(f, "id `{id}` does not match the file name `{stem}`")
            }
            Self::EmptyText => f.write_str("must be a non-empty string"),
            Self::BadState { found } => write!(f, "bad state `{found}`"),
            Self::BindingsKeys => {
                f.write_str("bindings must have exactly the keys code, tests and examples")
            }
            Self::BindingNotAList { binding } => {
                write!(f, "binding `{binding}` must be a list of strings")
            }
            Self::BindingNotEmpty { state, binding } => write!(
                f,
                "state {} must have empty bindings, but `{binding}` is not empty",
                state.name()
            ),
            Self::BindingRequired { binding } => {
                write!(
                    f,
                    "state delivered must bind at least one `{binding}` entry"
                )
            }
            Self::BadCapabilityId { entry } => write!(f, "entry `{entry}` is not a cap id"),
            Self::EmptyChanges => f.write_str("must be a non-empty list of cap ids"),
            Self::BadAffects { entry } => {
                write!(f, "affects entry `{entry}` must be `cap:<slug>#<section>`")
            }
            Self::BadAffectsSection { entry, section } => {
                write!(f, "affects entry `{entry}` has a bad section `{section}`")
            }
            Self::AffectsNotInChanges { entry } => write!(
                f,
                "affects entry `{entry}` names a capability missing from `changes`"
            ),
            Self::BadContractEffect { found } => write!(
                f,
                "bad contract_effect `{found}`; it is `none` or `changes`"
            ),
            Self::ItemUnknownKey { key } => write!(f, "unknown item key `{key}`"),
            Self::ItemClaimEmpty => f.write_str("every item needs a non-empty `claim`"),
            Self::BadOutcome { found } => write!(
                f,
                "bad outcome `{found}`; it is `implemented`, `dropped` or `moved:change:<slug>`"
            ),
            Self::EmptyTag => f.write_str("tags must be non-empty strings"),
            Self::DuplicateTag { tag } => write!(f, "duplicate tag `{tag}`"),
            Self::BadTimestamp { reason } => {
                write!(f, "timestamp is not RFC 3339: {reason}")
            }
            Self::SectionMissing => f.write_str("missing section"),
            Self::SectionUnexpected => f.write_str("unexpected section"),
            Self::SectionDuplicated => f.write_str("a section appears more than once"),
            Self::SectionsOutOfOrder { found } => {
                write!(f, "sections out of order: found {}", found.join(", "))
            }
            Self::SectionEmpty => f.write_str("section is empty"),
            Self::BodyEmpty => f.write_str("the body is empty"),
            Self::ProposedOutsideCapability { slug } => {
                write!(f, "PROPOSED block `{slug}` outside a capability")
            }
            Self::ProposedOutsideSection { slug } => {
                write!(f, "PROPOSED block `{slug}` outside a section")
            }
            Self::Placeholder { token } => {
                write!(f, "placeholder `{token}` is not resolved text")
            }
            Self::SplitLink { form } => {
                write!(f, "a link split across lines ({form:?}) is not allowed")
            }
        }
    }
}

/// One violation of the profile: the file, where in it, and the rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub(crate) file: String,
    pub(crate) at: Location,
    pub(crate) rule: Rule,
}

impl Violation {
    /// The path of the document, as given to the validator.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Where in the document.
    #[must_use]
    pub fn at(&self) -> &Location {
        &self.at
    }

    /// The rule broken.
    #[must_use]
    pub fn rule(&self) -> &Rule {
        &self.rule
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.file)?;
        match &self.at {
            Location::Key(key) => write!(f, "`{key}`: ")?,
            Location::Section(name) => write!(f, "section `{name}`: ")?,
            Location::Line(line) => write!(f, "line {line}: ")?,
            Location::Document => {}
        }
        write!(f, "{}", self.rule)
    }
}
