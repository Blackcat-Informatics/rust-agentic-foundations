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

/// The rule a violation breaks, without its data.
///
/// One variant per variant of [`Rule`], so a caller can name, list and map the
/// rules without matching their payloads. The set is closed on purpose: a
/// `match` over `RuleTag` is exhaustive, so a rule added to [`Rule`] adds a
/// variant here and breaks every match that must name them all. [`Rule::ALL`]
/// lists every tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RuleTag {
    /// [`Rule::MissingKey`].
    MissingKey,
    /// [`Rule::UnknownKey`].
    UnknownKey,
    /// [`Rule::ResourceKey`].
    ResourceKey,
    /// [`Rule::UnknownType`].
    UnknownType,
    /// [`Rule::WrongDirectory`].
    WrongDirectory,
    /// [`Rule::WrongShape`].
    WrongShape,
    /// [`Rule::InvalidId`].
    InvalidId,
    /// [`Rule::IdFileName`].
    IdFileName,
    /// [`Rule::EmptyText`].
    EmptyText,
    /// [`Rule::BadState`].
    BadState,
    /// [`Rule::BindingsKeys`].
    BindingsKeys,
    /// [`Rule::BindingNotAList`].
    BindingNotAList,
    /// [`Rule::BindingNotEmpty`].
    BindingNotEmpty,
    /// [`Rule::BindingRequired`].
    BindingRequired,
    /// [`Rule::BadCapabilityId`].
    BadCapabilityId,
    /// [`Rule::EmptyChanges`].
    EmptyChanges,
    /// [`Rule::BadAffects`].
    BadAffects,
    /// [`Rule::BadAffectsSection`].
    BadAffectsSection,
    /// [`Rule::AffectsNotInChanges`].
    AffectsNotInChanges,
    /// [`Rule::BadContractEffect`].
    BadContractEffect,
    /// [`Rule::ItemUnknownKey`].
    ItemUnknownKey,
    /// [`Rule::ItemClaimEmpty`].
    ItemClaimEmpty,
    /// [`Rule::BadOutcome`].
    BadOutcome,
    /// [`Rule::EmptyTag`].
    EmptyTag,
    /// [`Rule::DuplicateTag`].
    DuplicateTag,
    /// [`Rule::BadTimestamp`].
    BadTimestamp,
    /// [`Rule::SectionMissing`].
    SectionMissing,
    /// [`Rule::SectionUnexpected`].
    SectionUnexpected,
    /// [`Rule::SectionDuplicated`].
    SectionDuplicated,
    /// [`Rule::SectionsOutOfOrder`].
    SectionsOutOfOrder,
    /// [`Rule::SectionEmpty`].
    SectionEmpty,
    /// [`Rule::BodyEmpty`].
    BodyEmpty,
    /// [`Rule::ProposedOutsideCapability`].
    ProposedOutsideCapability,
    /// [`Rule::ProposedOutsideSection`].
    ProposedOutsideSection,
    /// [`Rule::Placeholder`].
    Placeholder,
    /// [`Rule::SplitLink`].
    SplitLink,
}

impl RuleTag {
    /// The name of the tag: the variant in lowercase kebab-case.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::MissingKey => "missing-key",
            Self::UnknownKey => "unknown-key",
            Self::ResourceKey => "resource-key",
            Self::UnknownType => "unknown-type",
            Self::WrongDirectory => "wrong-directory",
            Self::WrongShape => "wrong-shape",
            Self::InvalidId => "invalid-id",
            Self::IdFileName => "id-file-name",
            Self::EmptyText => "empty-text",
            Self::BadState => "bad-state",
            Self::BindingsKeys => "bindings-keys",
            Self::BindingNotAList => "binding-not-a-list",
            Self::BindingNotEmpty => "binding-not-empty",
            Self::BindingRequired => "binding-required",
            Self::BadCapabilityId => "bad-capability-id",
            Self::EmptyChanges => "empty-changes",
            Self::BadAffects => "bad-affects",
            Self::BadAffectsSection => "bad-affects-section",
            Self::AffectsNotInChanges => "affects-not-in-changes",
            Self::BadContractEffect => "bad-contract-effect",
            Self::ItemUnknownKey => "item-unknown-key",
            Self::ItemClaimEmpty => "item-claim-empty",
            Self::BadOutcome => "bad-outcome",
            Self::EmptyTag => "empty-tag",
            Self::DuplicateTag => "duplicate-tag",
            Self::BadTimestamp => "bad-timestamp",
            Self::SectionMissing => "section-missing",
            Self::SectionUnexpected => "section-unexpected",
            Self::SectionDuplicated => "section-duplicated",
            Self::SectionsOutOfOrder => "sections-out-of-order",
            Self::SectionEmpty => "section-empty",
            Self::BodyEmpty => "body-empty",
            Self::ProposedOutsideCapability => "proposed-outside-capability",
            Self::ProposedOutsideSection => "proposed-outside-section",
            Self::Placeholder => "placeholder",
            Self::SplitLink => "split-link",
        }
    }
}

impl fmt::Display for RuleTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl Rule {
    /// Every tag, in the order of the variants of [`Rule`].
    pub const ALL: [RuleTag; 36] = [
        RuleTag::MissingKey,
        RuleTag::UnknownKey,
        RuleTag::ResourceKey,
        RuleTag::UnknownType,
        RuleTag::WrongDirectory,
        RuleTag::WrongShape,
        RuleTag::InvalidId,
        RuleTag::IdFileName,
        RuleTag::EmptyText,
        RuleTag::BadState,
        RuleTag::BindingsKeys,
        RuleTag::BindingNotAList,
        RuleTag::BindingNotEmpty,
        RuleTag::BindingRequired,
        RuleTag::BadCapabilityId,
        RuleTag::EmptyChanges,
        RuleTag::BadAffects,
        RuleTag::BadAffectsSection,
        RuleTag::AffectsNotInChanges,
        RuleTag::BadContractEffect,
        RuleTag::ItemUnknownKey,
        RuleTag::ItemClaimEmpty,
        RuleTag::BadOutcome,
        RuleTag::EmptyTag,
        RuleTag::DuplicateTag,
        RuleTag::BadTimestamp,
        RuleTag::SectionMissing,
        RuleTag::SectionUnexpected,
        RuleTag::SectionDuplicated,
        RuleTag::SectionsOutOfOrder,
        RuleTag::SectionEmpty,
        RuleTag::BodyEmpty,
        RuleTag::ProposedOutsideCapability,
        RuleTag::ProposedOutsideSection,
        RuleTag::Placeholder,
        RuleTag::SplitLink,
    ];

    /// The tag of this value, which drops its data.
    #[must_use]
    pub fn kind(&self) -> RuleTag {
        match self {
            Self::MissingKey { .. } => RuleTag::MissingKey,
            Self::UnknownKey { .. } => RuleTag::UnknownKey,
            Self::ResourceKey { .. } => RuleTag::ResourceKey,
            Self::UnknownType { .. } => RuleTag::UnknownType,
            Self::WrongDirectory { .. } => RuleTag::WrongDirectory,
            Self::WrongShape { .. } => RuleTag::WrongShape,
            Self::InvalidId { .. } => RuleTag::InvalidId,
            Self::IdFileName { .. } => RuleTag::IdFileName,
            Self::EmptyText => RuleTag::EmptyText,
            Self::BadState { .. } => RuleTag::BadState,
            Self::BindingsKeys => RuleTag::BindingsKeys,
            Self::BindingNotAList { .. } => RuleTag::BindingNotAList,
            Self::BindingNotEmpty { .. } => RuleTag::BindingNotEmpty,
            Self::BindingRequired { .. } => RuleTag::BindingRequired,
            Self::BadCapabilityId { .. } => RuleTag::BadCapabilityId,
            Self::EmptyChanges => RuleTag::EmptyChanges,
            Self::BadAffects { .. } => RuleTag::BadAffects,
            Self::BadAffectsSection { .. } => RuleTag::BadAffectsSection,
            Self::AffectsNotInChanges { .. } => RuleTag::AffectsNotInChanges,
            Self::BadContractEffect { .. } => RuleTag::BadContractEffect,
            Self::ItemUnknownKey { .. } => RuleTag::ItemUnknownKey,
            Self::ItemClaimEmpty => RuleTag::ItemClaimEmpty,
            Self::BadOutcome { .. } => RuleTag::BadOutcome,
            Self::EmptyTag => RuleTag::EmptyTag,
            Self::DuplicateTag { .. } => RuleTag::DuplicateTag,
            Self::BadTimestamp { .. } => RuleTag::BadTimestamp,
            Self::SectionMissing => RuleTag::SectionMissing,
            Self::SectionUnexpected => RuleTag::SectionUnexpected,
            Self::SectionDuplicated => RuleTag::SectionDuplicated,
            Self::SectionsOutOfOrder { .. } => RuleTag::SectionsOutOfOrder,
            Self::SectionEmpty => RuleTag::SectionEmpty,
            Self::BodyEmpty => RuleTag::BodyEmpty,
            Self::ProposedOutsideCapability { .. } => RuleTag::ProposedOutsideCapability,
            Self::ProposedOutsideSection { .. } => RuleTag::ProposedOutsideSection,
            Self::Placeholder { .. } => RuleTag::Placeholder,
            Self::SplitLink { .. } => RuleTag::SplitLink,
        }
    }
}

#[cfg(test)]
mod tag_tests {
    use super::{Rule, RuleTag};
    use crate::body::LinkForm;
    use crate::profile::schema::DocType;
    use crate::profile::timestamp::validate_timestamp;
    use crate::profile::typed::State;

    /// A rule of each tag. The match names every tag, so a tag added to
    /// `RuleTag` stops this from compiling until it has a sample.
    fn sample(tag: RuleTag) -> Rule {
        match tag {
            RuleTag::MissingKey => Rule::MissingKey { key: String::new() },
            RuleTag::UnknownKey => Rule::UnknownKey { key: String::new() },
            RuleTag::ResourceKey => Rule::ResourceKey {
                doc_type: DocType::Capability,
            },
            RuleTag::UnknownType => Rule::UnknownType {
                found: String::new(),
            },
            RuleTag::WrongDirectory => Rule::WrongDirectory {
                declared: DocType::Capability,
            },
            RuleTag::WrongShape => Rule::WrongShape { expected: "" },
            RuleTag::InvalidId => Rule::InvalidId { prefix: "" },
            RuleTag::IdFileName => Rule::IdFileName {
                id: String::new(),
                stem: String::new(),
            },
            RuleTag::EmptyText => Rule::EmptyText,
            RuleTag::BadState => Rule::BadState {
                found: String::new(),
            },
            RuleTag::BindingsKeys => Rule::BindingsKeys,
            RuleTag::BindingNotAList => Rule::BindingNotAList {
                binding: String::new(),
            },
            RuleTag::BindingNotEmpty => Rule::BindingNotEmpty {
                state: State::Designed,
                binding: String::new(),
            },
            RuleTag::BindingRequired => Rule::BindingRequired {
                binding: String::new(),
            },
            RuleTag::BadCapabilityId => Rule::BadCapabilityId {
                entry: String::new(),
            },
            RuleTag::EmptyChanges => Rule::EmptyChanges,
            RuleTag::BadAffects => Rule::BadAffects {
                entry: String::new(),
            },
            RuleTag::BadAffectsSection => Rule::BadAffectsSection {
                entry: String::new(),
                section: String::new(),
            },
            RuleTag::AffectsNotInChanges => Rule::AffectsNotInChanges {
                entry: String::new(),
            },
            RuleTag::BadContractEffect => Rule::BadContractEffect {
                found: String::new(),
            },
            RuleTag::ItemUnknownKey => Rule::ItemUnknownKey { key: String::new() },
            RuleTag::ItemClaimEmpty => Rule::ItemClaimEmpty,
            RuleTag::BadOutcome => Rule::BadOutcome {
                found: String::new(),
            },
            RuleTag::EmptyTag => Rule::EmptyTag,
            RuleTag::DuplicateTag => Rule::DuplicateTag { tag: String::new() },
            RuleTag::BadTimestamp => Rule::BadTimestamp {
                reason: validate_timestamp("x").expect_err("not a timestamp"),
            },
            RuleTag::SectionMissing => Rule::SectionMissing,
            RuleTag::SectionUnexpected => Rule::SectionUnexpected,
            RuleTag::SectionDuplicated => Rule::SectionDuplicated,
            RuleTag::SectionsOutOfOrder => Rule::SectionsOutOfOrder { found: Vec::new() },
            RuleTag::SectionEmpty => Rule::SectionEmpty,
            RuleTag::BodyEmpty => Rule::BodyEmpty,
            RuleTag::ProposedOutsideCapability => Rule::ProposedOutsideCapability {
                slug: String::new(),
            },
            RuleTag::ProposedOutsideSection => Rule::ProposedOutsideSection {
                slug: String::new(),
            },
            RuleTag::Placeholder => Rule::Placeholder {
                token: String::new(),
            },
            RuleTag::SplitLink => Rule::SplitLink {
                form: LinkForm::Inline,
            },
        }
    }

    /// The position of a rule among the variants of `Rule`. The match names
    /// every variant, so a variant added to `Rule` stops this from compiling
    /// until it is placed here and listed in `Rule::ALL`.
    fn position(rule: &Rule) -> usize {
        match rule {
            Rule::MissingKey { .. } => 0,
            Rule::UnknownKey { .. } => 1,
            Rule::ResourceKey { .. } => 2,
            Rule::UnknownType { .. } => 3,
            Rule::WrongDirectory { .. } => 4,
            Rule::WrongShape { .. } => 5,
            Rule::InvalidId { .. } => 6,
            Rule::IdFileName { .. } => 7,
            Rule::EmptyText => 8,
            Rule::BadState { .. } => 9,
            Rule::BindingsKeys => 10,
            Rule::BindingNotAList { .. } => 11,
            Rule::BindingNotEmpty { .. } => 12,
            Rule::BindingRequired { .. } => 13,
            Rule::BadCapabilityId { .. } => 14,
            Rule::EmptyChanges => 15,
            Rule::BadAffects { .. } => 16,
            Rule::BadAffectsSection { .. } => 17,
            Rule::AffectsNotInChanges { .. } => 18,
            Rule::BadContractEffect { .. } => 19,
            Rule::ItemUnknownKey { .. } => 20,
            Rule::ItemClaimEmpty => 21,
            Rule::BadOutcome { .. } => 22,
            Rule::EmptyTag => 23,
            Rule::DuplicateTag { .. } => 24,
            Rule::BadTimestamp { .. } => 25,
            Rule::SectionMissing => 26,
            Rule::SectionUnexpected => 27,
            Rule::SectionDuplicated => 28,
            Rule::SectionsOutOfOrder { .. } => 29,
            Rule::SectionEmpty => 30,
            Rule::BodyEmpty => 31,
            Rule::ProposedOutsideCapability { .. } => 32,
            Rule::ProposedOutsideSection { .. } => 33,
            Rule::Placeholder { .. } => 34,
            Rule::SplitLink { .. } => 35,
        }
    }

    /// How many variants `Rule` has.
    const VARIANTS: usize = 36;

    #[test]
    fn all_lists_every_rule_once_in_variant_order() {
        assert_eq!(Rule::ALL.len(), VARIANTS);
        for (index, tag) in Rule::ALL.into_iter().enumerate() {
            let rule = sample(tag);
            assert_eq!(rule.kind(), tag, "{tag}");
            assert_eq!(position(&rule), index, "{tag} is out of order in Rule::ALL");
        }
    }

    #[test]
    fn names_are_unique_lowercase_kebab_case() {
        let mut names: Vec<&str> = Rule::ALL.iter().map(|tag| tag.name()).collect();
        for name in &names {
            assert!(
                !name.is_empty()
                    && name.split('-').all(
                        |part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase())
                    ),
                "{name}"
            );
        }
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), VARIANTS);
    }
}
