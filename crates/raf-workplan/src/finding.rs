// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The findings of the workplan checks: one vocabulary for every rule.

use std::borrow::Cow;
use std::fmt;

use raf_okf::{DocType, ErrorTag, Location, ParseError, Rule, RuleTag, Violation};

/// Declare the finding kinds as `const` members of [`kind`] and list them, in
/// declaration order, as the one slice [`Finding::KINDS`] names. A finding is
/// built only from these constants, so no ad-hoc kind string exists.
macro_rules! kinds {
    ($($name:ident = $text:literal;)*) => {
        /// The `const` members every finding kind is taken from.
        pub(crate) mod kind {
            $(pub(crate) const $name: &str = $text;)*
        }

        /// Every kind, in declaration order.
        const ALL_KINDS: &[&str] = &[$(kind::$name),*];
    };
}

kinds! {
    WRONG_DIRECTORY = "wrong-directory";
    NESTED_DIRECTORY = "nested-directory";
    STRAY_MARKDOWN = "stray-markdown";
    UNREADABLE_FILE = "unreadable-file";
    UNREADABLE_DIRECTORY = "unreadable-directory";
    DEPENDS_MISSING = "depends-missing";
    CHANGES_MISSING = "changes-missing";
    MOVED_MISSING = "moved-missing";
    SINGLETON_OTHER_FILE = "singleton-other-file";
    CLAIM_EVIDENCE_MISSING = "claim-evidence-missing";
    CLAIM_EVIDENCE_NOT_FOR = "claim-evidence-not-for";
    CLAIM_TEST_UNBOUND = "claim-test-unbound";
    EVIDENCE_FOR_MISSING = "evidence-for-missing";
    ASSESSMENT_SUBJECT_MISSING = "assessment-subject-missing";
    ASSESSMENT_ID_MISMATCH = "assessment-id-mismatch";
    SCOPE_CRATE_MISSING = "scope-crate-missing";
    COVERAGE_RECORD_MISSING = "coverage-record-missing";
    COVERAGE_ITEM_UNBOUND = "coverage-item-unbound";
    COVERAGE_TEST_UNBOUND = "coverage-test-unbound";
    COVERAGE_REEXPORT_UNBOUND = "coverage-reexport-unbound";
    COVERAGE_EXCLUSION_UNMATCHED = "coverage-exclusion-unmatched";
    COVERAGE_SURFACE_UNREADABLE = "coverage-surface-unreadable";
    COVERAGE_EXCEPTION_UNMATCHED = "coverage-exception-unmatched";
    COVERAGE_LINES_OVER_CEILING = "coverage-lines-over-ceiling";
    ARCHITECTURE_PAGE_UNLINKED = "architecture-page-unlinked";
    DECISION_CONDITIONS_MISSING = "decision-conditions-missing";
    EXPLANATION_CONDITIONS_MISSING = "explanation-conditions-missing";
    DEPENDS_CYCLE = "depends-cycle";
    PROPOSED_MISSING_CHANGE = "proposed-missing-change";
    PROPOSED_CHANGE_NOT_LISTING = "proposed-change-not-listing";
    PROPOSED_SECTION_NOT_AFFECTED = "proposed-section-not-affected";
    STATE_CONSIDERED_NO_PROPOSAL = "state-considered-no-proposal";
    STATE_DESIGNED_PROPOSAL = "state-designed-proposal";
    STATE_DELIVERED_OPEN_RECORD = "state-delivered-open-record";
    STATE_DELIVERED_PROPOSAL = "state-delivered-proposal";
    NO_OPEN_RECORD = "no-open-record";
    LINK_BROKEN = "link-broken";
    ANCHOR_BROKEN = "anchor-broken";
    LINK_OUTSIDE = "link-outside";
    LINK_SYMLINK = "link-symlink";
    ISSUE_REFERENCE = "issue-reference";
    BINDING_UNRESOLVED = "binding-unresolved";
    BINDING_AMBIGUOUS = "binding-ambiguous";
    BINDING_WRONG_ROLE = "binding-wrong-role";
    BINDING_MACRO_GENERATED = "binding-macro-generated";
    INDEX_MISSING = "index-missing";
    GENERATED_STALE = "generated-stale";
    GENERATED_UNKNOWN_KIND = "generated-unknown-kind";
    GENERATED_MISPLACED = "generated-misplaced";
    INDEX_HAND_WRITTEN = "index-hand-written";
    MIXED_LINE_ENDINGS = "mixed-line-endings";
    STRICT_EXAMPLE_MISSING = "strict-example-missing";
    STRICT_MATRIX_MISSING = "strict-matrix-missing";
    STRICT_CLAIM_UNCOVERED = "strict-claim-uncovered";
    STRICT_CLAIM_UNIT_COLLAPSED = "strict-claim-unit-collapsed";
    STRICT_SECTION_THIN = "strict-section-thin";
    STRICT_USAGE_UNBOUND = "strict-usage-unbound";
    STRICT_LIMITATIONS_MISSING = "strict-limitations-missing";
    OKF_MISSING_KEY = "okf-missing-key";
    OKF_UNKNOWN_KEY = "okf-unknown-key";
    OKF_RESOURCE_KEY = "okf-resource-key";
    OKF_UNKNOWN_TYPE = "okf-unknown-type";
    OKF_WRONG_DIRECTORY = "okf-wrong-directory";
    OKF_WRONG_SHAPE = "okf-wrong-shape";
    OKF_INVALID_ID = "okf-invalid-id";
    OKF_ID_FILE_NAME = "okf-id-file-name";
    OKF_EMPTY_TEXT = "okf-empty-text";
    OKF_BAD_STATE = "okf-bad-state";
    OKF_BINDINGS_KEYS = "okf-bindings-keys";
    OKF_BINDING_NOT_A_LIST = "okf-binding-not-a-list";
    OKF_BINDING_NOT_EMPTY = "okf-binding-not-empty";
    OKF_BINDING_REQUIRED = "okf-binding-required";
    OKF_BAD_CAPABILITY_ID = "okf-bad-capability-id";
    OKF_EMPTY_CHANGES = "okf-empty-changes";
    OKF_BAD_AFFECTS = "okf-bad-affects";
    OKF_BAD_AFFECTS_SECTION = "okf-bad-affects-section";
    OKF_AFFECTS_NOT_IN_CHANGES = "okf-affects-not-in-changes";
    OKF_BAD_CONTRACT_EFFECT = "okf-bad-contract-effect";
    OKF_ITEM_UNKNOWN_KEY = "okf-item-unknown-key";
    OKF_ITEM_CLAIM_EMPTY = "okf-item-claim-empty";
    OKF_BAD_OUTCOME = "okf-bad-outcome";
    OKF_EMPTY_TAG = "okf-empty-tag";
    OKF_DUPLICATE_TAG = "okf-duplicate-tag";
    OKF_BAD_TIMESTAMP = "okf-bad-timestamp";
    OKF_SECTION_MISSING = "okf-section-missing";
    OKF_SECTION_UNEXPECTED = "okf-section-unexpected";
    OKF_SECTION_DUPLICATED = "okf-section-duplicated";
    OKF_SECTIONS_OUT_OF_ORDER = "okf-sections-out-of-order";
    OKF_SECTION_EMPTY = "okf-section-empty";
    OKF_BODY_EMPTY = "okf-body-empty";
    OKF_PROPOSED_OUTSIDE_CAPABILITY = "okf-proposed-outside-capability";
    OKF_PROPOSED_OUTSIDE_SECTION = "okf-proposed-outside-section";
    OKF_PLACEHOLDER = "okf-placeholder";
    OKF_SPLIT_LINK = "okf-split-link";
    OKF_CLAIM_OUTSIDE_CAPABILITY = "okf-claim-outside-capability";
    OKF_BODY_NOT_EMPTY = "okf-body-not-empty";
    OKF_SINGLETON_ID = "okf-singleton-id";
    OKF_BAD_SUBJECT = "okf-bad-subject";
    OKF_COMMAND_SUBJECT = "okf-command-subject";
    OKF_ROW_UNKNOWN_KEY = "okf-row-unknown-key";
    OKF_ROW_MISSING_KEY = "okf-row-missing-key";
    OKF_BAD_INPUT_KIND = "okf-bad-input-kind";
    OKF_BAD_INPUT_NAME = "okf-bad-input-name";
    OKF_BAD_DIGEST = "okf-bad-digest";
    OKF_DUPLICATE_ENTRY = "okf-duplicate-entry";
    OKF_EMPTY_LIST = "okf-empty-list";
    OKF_SUBJECT_INPUT_MISSING = "okf-subject-input-missing";
    OKF_BAD_VERDICT = "okf-bad-verdict";
    OKF_BAD_ACTOR = "okf-bad-actor";
    OKF_BAD_REVISION = "okf-bad-revision";
    OKF_BAD_MEASURE = "okf-bad-measure";
    OKF_LIMIT_WITHOUT_VALUE = "okf-limit-without-value";
    OKF_VERDICT_OVER_LIMIT = "okf-verdict-over-limit";
    OKF_BAD_CRATE_NAME = "okf-bad-crate-name";
    OKF_BAD_EXCLUSION_ITEM = "okf-bad-exclusion-item";
    OKF_CRATE_ROOT_EXCLUSION = "okf-crate-root-exclusion";
    OKF_EMPTY_REASON = "okf-empty-reason";
    OKF_BAD_STRICTNESS = "okf-bad-strictness";
    OKF_TESTED_BY_OUTSIDE_CAPABILITY = "okf-tested-by-outside-capability";
    OKF_ITEM_REASON_EMPTY = "okf-item-reason-empty";
    OKF_BAD_CEILING = "okf-bad-ceiling";
    OKF_BAD_EXCEPTION = "okf-bad-exception";
    OKF_PARSE_UNCLOSED_FENCE = "okf-parse-unclosed-fence";
    OKF_PARSE_TOO_LARGE = "okf-parse-too-large";
    OKF_PARSE_MIXED_LINE_ENDINGS = "okf-parse-mixed-line-endings";
    OKF_PARSE_CONTROL_CHARACTER = "okf-parse-control-character";
    OKF_PARSE_TRAILING_WHITESPACE = "okf-parse-trailing-whitespace";
    OKF_PARSE_INDENTATION = "okf-parse-indentation";
    OKF_PARSE_SPACING = "okf-parse-spacing";
    OKF_PARSE_NOT_A_MAPPING = "okf-parse-not-a-mapping";
    OKF_PARSE_EXPECTED_KEY = "okf-parse-expected-key";
    OKF_PARSE_EXPECTED_ITEM = "okf-parse-expected-item";
    OKF_PARSE_INVALID_KEY = "okf-parse-invalid-key";
    OKF_PARSE_DUPLICATE_KEY = "okf-parse-duplicate-key";
    OKF_PARSE_MERGE_KEY = "okf-parse-merge-key";
    OKF_PARSE_ANCHOR = "okf-parse-anchor";
    OKF_PARSE_ALIAS = "okf-parse-alias";
    OKF_PARSE_TAG = "okf-parse-tag";
    OKF_PARSE_BLOCK_SCALAR = "okf-parse-block-scalar";
    OKF_PARSE_INDICATOR_CHARACTER = "okf-parse-indicator-character";
    OKF_PARSE_FLOW_MAPPING = "okf-parse-flow-mapping";
    OKF_PARSE_NESTED_FLOW = "okf-parse-nested-flow";
    OKF_PARSE_NESTED_SEQUENCE = "okf-parse-nested-sequence";
    OKF_PARSE_UNTERMINATED_FLOW = "okf-parse-unterminated-flow";
    OKF_PARSE_EMPTY_FLOW_ENTRY = "okf-parse-empty-flow-entry";
    OKF_PARSE_TEXT_AFTER_FLOW = "okf-parse-text-after-flow";
    OKF_PARSE_UNTERMINATED_QUOTE = "okf-parse-unterminated-quote";
    OKF_PARSE_TEXT_AFTER_QUOTE = "okf-parse-text-after-quote";
    OKF_PARSE_UNKNOWN_ESCAPE = "okf-parse-unknown-escape";
    OKF_PARSE_PLAIN_COLON_SPACE = "okf-parse-plain-colon-space";
    OKF_PARSE_EMPTY_ITEM = "okf-parse-empty-item";
    OKF_PARSE_MIXED_LIST = "okf-parse-mixed-list";
    OKF_PARSE_DEPTH_EXCEEDED = "okf-parse-depth-exceeded";
    OKF_PARSE_BLOCK_NESTING = "okf-parse-block-nesting";
    OKF_PARSE_MALFORMED_PROPOSED = "okf-parse-malformed-proposed";
    OKF_PARSE_MALFORMED_GENERATED = "okf-parse-malformed-generated";
    OKF_PARSE_NESTED_GENERATED = "okf-parse-nested-generated";
    OKF_PARSE_ORPHAN_GENERATED_END = "okf-parse-orphan-generated-end";
    OKF_PARSE_UNCLOSED_GENERATED = "okf-parse-unclosed-generated";
    OKF_PARSE_MALFORMED_CLAIM = "okf-parse-malformed-claim";
    OKF_PARSE_UNATTACHED_CLAIM = "okf-parse-unattached-claim";
    OKF_PARSE_MALFORMED_TESTED_BY = "okf-parse-malformed-tested-by";
    OKF_PARSE_UNATTACHED_TESTED_BY = "okf-parse-unattached-tested-by";
    OKF_PARSE_UNKNOWN_MARKER = "okf-parse-unknown-marker";
    OKF_PARSE_LINK_PART_TOO_LONG = "okf-parse-link-part-too-long";
    OKF_PARSE_INDEX_HEADER = "okf-parse-index-header";
    OKF_PARSE_INDEX_MARKER = "okf-parse-index-marker";
    OKF_PARSE_INDEX_EMPTY = "okf-parse-index-empty";
}

/// One result of a check: what is wrong, where, and how to put it right.
///
/// The `kind` is a stable kebab-case name from [`Finding::KINDS`]. The file is
/// the repository-relative path with `/` separators, and `location` points
/// inside it with the location type of `raf-okf`: a frontmatter key, a section,
/// a line, or the document as a whole.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Finding {
    /// The stable name of the rule, one of [`Finding::KINDS`].
    pub kind: &'static str,
    /// The path of the file the finding is about.
    pub file: String,
    /// Where in the file.
    pub location: Location,
    /// What is wrong.
    pub message: String,
    /// How to put it right.
    pub fix: String,
}

impl Finding {
    /// Every kind a finding can have.
    ///
    /// * `okf-<rule>`: one kind for each rule of the `raf-okf` profile, named
    ///   by `raf_okf::Rule::ALL`.
    /// * `okf-parse-<error>`: one kind for each way a document fails to read,
    ///   named by `raf_okf::ErrorKind::ALL`. A text that does not begin with a
    ///   frontmatter fence is `stray-markdown` instead, an index with text
    ///   outside its blocks is `index-hand-written`, and any other index that
    ///   does not read is `okf-parse-index-*`.
    /// * `wrong-directory`, `nested-directory` and `stray-markdown`: the
    ///   rules of the flat document directories.
    /// * `unreadable-file` and `unreadable-directory`: a file or a directory
    ///   the tree refused to give, which is reported and never skipped.
    /// * `depends-missing`, `changes-missing` and `moved-missing`: a `depends`
    ///   or `changes` entry names a capability with no document, or a `moved`
    ///   outcome names a change with no record. A name whose document exists
    ///   but is broken is not missing; the findings of that document say what
    ///   is wrong with it.
    /// * `depends-cycle`: capabilities that depend on each other, once for
    ///   each group, a capability that depends on itself included.
    /// * `singleton-other-file`: a file other than `default.md` in
    ///   `work/coverage`, the directory of the one coverage record.
    /// * `claim-evidence-missing` and `claim-evidence-not-for`: a claim marker
    ///   of a capability names an evidence record that does not exist, or one
    ///   whose `for` does not list the capability; the finding names the
    ///   capability document and the line of the marker.
    ///   `claim-test-unbound`: an entry of a tested-by marker of a capability
    ///   that is no path of the capability's `bindings.tests` or
    ///   `bindings.examples`, at the line of the marker, under either
    ///   strictness.
    /// * `evidence-for-missing`, `assessment-subject-missing` and
    ///   `scope-crate-missing`: an evidence record's `for` names a capability
    ///   with no document, an assessment's subject is a section of a
    ///   capability with no document, or the coverage record's `scope` names
    ///   a crate that is no package of the source workspace. As for the
    ///   documents, a name whose record or document exists but is broken is
    ///   not missing.
    /// * `coverage-record-missing`: a repository with capability documents
    ///   has no coverage record `work/coverage/default.md` (`scope: []`
    ///   enforces nothing, visibly). `coverage-item-unbound` and
    ///   `coverage-test-unbound`: a public item or a test of a crate of the
    ///   record's `scope` that no capability binds and no exclusion matches,
    ///   at the file and line of the item; `coverage-reexport-unbound`: a
    ///   public re-export of such a crate that the source index cannot follow
    ///   to an item (a type a macro generates, an enum variant) and that no
    ///   module binding covers, at its declaration. `coverage-surface-unreadable`: the
    ///   public surface of a crate of `scope` cannot be listed.
    ///   `coverage-exclusion-unmatched`: an exclusion that matches no public
    ///   item and no test of the workspace. See [`crate::coverage`].
    /// * `assessment-id-mismatch`: an assessment whose id is not
    ///   `assessment:<capability slug>-<section>` of its subject, the one id
    ///   an assessment of that section has, so a section has at most one
    ///   assessment.
    /// * `architecture-page-unlinked`: an architecture page that no other
    ///   capability, change or architecture document links to.
    ///   `decision-conditions-missing`: an architecture page with a
    ///   `## Decision` heading and no `## Conditions to revisit` heading, or a
    ///   `## Conditions to revisit` heading with no list item.
    ///   `explanation-conditions-missing`: a `### Decision` in the Explanation
    ///   of a capability that no `### Conditions to revisit` follows before
    ///   the next decision, or such a subheading with no list item. See
    ///   [`crate::Workplan::check`].
    ///
    /// * `proposed-missing-change`, `proposed-change-not-listing` and
    ///   `proposed-section-not-affected`: a PROPOSED block names a change
    ///   with no record, names a change that does not list its capability,
    ///   or sits in a section the change does not list in `affects`.
    /// * `state-considered-no-proposal`, `state-designed-proposal`,
    ///   `state-delivered-open-record` and `state-delivered-proposal`: a
    ///   capability whose PROPOSED blocks or open change records do not fit
    ///   its state, one kind for each rule. `no-open-record`: a capability
    ///   that is not delivered and that no open change record lists (not
    ///   reported while a change record exists that failed to load, since that
    ///   record may list it and its own findings say what is wrong).
    /// * `link-broken`, `anchor-broken`, `link-outside` and `link-symlink`: a
    ///   link whose target is not a tracked path, whose anchor no heading
    ///   has, that leaves the repository, or that leads to a tracked symlink.
    /// * `issue-reference`: an issue or pull request number, or wording that
    ///   refers to the development process, in a document.
    /// * `binding-unresolved`, `binding-ambiguous`, `binding-wrong-role` and
    ///   `binding-macro-generated`: an entry of `bindings` that the source
    ///   index does not resolve (or resolves to more than one symbol), whose
    ///   definitions do not fit its list (`code` is a shipping item, `tests`
    ///   a `#[test]` function, `examples` an example target or a documented
    ///   item of a library whose documentation tests run), or that does not
    ///   resolve because a macro invocation in the parent module generates it.
    /// * `index-missing`: a repository with capability documents has no
    ///   `docs/capabilities/index.md`; sync creates it.
    /// * `generated-stale`: a generated block differs from what sync writes;
    ///   the message names the file and the kind.
    /// * `generated-unknown-kind` and `generated-misplaced`: a block of a kind
    ///   `raf-workplan` does not own, or of an owned kind in a document that
    ///   does not take it (`verification-matrix` and `relationships` belong in
    ///   a section of a capability document, `catalogue` in
    ///   `docs/capabilities/index.md`).
    /// * `index-hand-written`: text in `docs/capabilities/index.md` outside a
    ///   generated block, other than its SPDX comment.
    /// * `mixed-line-endings`: the body of a document mixes CRLF and LF line
    ///   endings, so sync refuses to rewrite it.
    /// * `strict-example-missing`, `strict-matrix-missing`,
    ///   `strict-claim-uncovered`, `strict-claim-unit-collapsed`,
    ///   `strict-section-thin`, `strict-usage-unbound` and
    ///   `strict-limitations-missing`: the rules of strict mode for a delivered
    ///   capability, judged only when [`crate::Workplan::strictness`] is
    ///   [`crate::Strictness::Strict`]; see [`crate::strict`].
    pub const KINDS: &'static [&'static str] = ALL_KINDS;

    /// The heading of the section the finding is in, when its location is a
    /// section.
    #[must_use]
    pub fn section(&self) -> Option<&str> {
        match &self.location {
            Location::Section(name) => Some(name),
            _ => None,
        }
    }

    /// The 1-based line the finding is at, when its location is a line.
    #[must_use]
    pub fn line(&self) -> Option<usize> {
        match &self.location {
            Location::Line(line) => Some(*line),
            _ => None,
        }
    }

    /// The frontmatter key the finding is about, when its location is a key.
    #[must_use]
    pub fn key(&self) -> Option<&str> {
        match &self.location {
            Location::Key(key) => Some(key),
            _ => None,
        }
    }

    pub(crate) fn new(
        kind: &'static str,
        file: &str,
        location: Location,
        message: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            file: file.to_string(),
            location,
            message: message.into(),
            fix: fix.into(),
        }
    }

    /// The finding for a violation of the `raf-okf` profile.
    pub(crate) fn from_violation(violation: &Violation) -> Self {
        let (kind, fix) = rule_entry(violation.rule().kind());
        Self::new(
            kind,
            violation.file(),
            violation.at().clone(),
            violation.rule().to_string(),
            fix,
        )
    }

    /// The finding for a rule of the `raf-okf` profile that a document outside
    /// the typed documents breaks.
    pub(crate) fn from_rule(rule: &Rule, file: &str, at: Location) -> Self {
        let (kind, fix) = rule_entry(rule.kind());
        Self::new(kind, file, at, rule.to_string(), fix)
    }

    /// The finding for a document or an index that does not read.
    pub(crate) fn from_parse_error(file: &str, error: &ParseError) -> Self {
        let (kind, fix) = error_entry(error.kind().tag());
        Self::new(
            kind,
            file,
            Location::Line(error.line()),
            error.kind().to_string(),
            fix,
        )
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.file)?;
        match &self.location {
            Location::Key(key) => write!(f, "`{key}`: ")?,
            Location::Section(name) => write!(f, "section `{name}`: ")?,
            Location::Line(line) => write!(f, "line {line}: ")?,
            _ => {}
        }
        write!(f, "{} [{}]; {}", self.message, self.kind, self.fix)
    }
}

/// The kind and the fix of a profile rule.
#[allow(clippy::too_many_lines, reason = "one arm per rule of the profile")]
fn rule_entry(tag: RuleTag) -> (&'static str, Cow<'static, str>) {
    let (kind, fix) = match tag {
        RuleTag::MissingKey => (
            kind::OKF_MISSING_KEY,
            "add the required key to the frontmatter",
        ),
        RuleTag::UnknownKey => (
            kind::OKF_UNKNOWN_KEY,
            "remove the key or rename it to one the type allows",
        ),
        RuleTag::ResourceKey => (
            kind::OKF_RESOURCE_KEY,
            "remove the `resource` key; the profile closes the key set",
        ),
        RuleTag::UnknownType => {
            return (
                kind::OKF_UNKNOWN_TYPE,
                Cow::Owned(format!("set `type` to {}", DocType::listed("or"))),
            );
        }
        RuleTag::WrongDirectory => (
            kind::OKF_WRONG_DIRECTORY,
            "move the file directly into the directory of its type",
        ),
        RuleTag::WrongShape => (
            kind::OKF_WRONG_SHAPE,
            "write the value in the shape the key requires",
        ),
        RuleTag::InvalidId => (
            kind::OKF_INVALID_ID,
            "write the id as the type prefix followed by a lowercase kebab-case slug",
        ),
        RuleTag::IdFileName => (
            kind::OKF_ID_FILE_NAME,
            "rename the file or the id so the slug and the file name agree",
        ),
        RuleTag::EmptyText => (kind::OKF_EMPTY_TEXT, "give the key a non-empty string"),
        RuleTag::BadState => (
            kind::OKF_BAD_STATE,
            "set `state` to considered, designed, implementing or delivered",
        ),
        RuleTag::BindingsKeys => (
            kind::OKF_BINDINGS_KEYS,
            "give `bindings` exactly the keys code, tests and examples",
        ),
        RuleTag::BindingNotAList => (
            kind::OKF_BINDING_NOT_A_LIST,
            "write the binding as a list of strings",
        ),
        RuleTag::BindingNotEmpty => (
            kind::OKF_BINDING_NOT_EMPTY,
            "empty the binding, or move the capability to a state that binds",
        ),
        RuleTag::BindingRequired => (
            kind::OKF_BINDING_REQUIRED,
            "bind at least one entry, or lower the state from delivered",
        ),
        RuleTag::BadCapabilityId => (
            kind::OKF_BAD_CAPABILITY_ID,
            "write the entry as `cap:<kebab-slug>`",
        ),
        RuleTag::EmptyChanges => (
            kind::OKF_EMPTY_CHANGES,
            "list at least one capability id in `changes`",
        ),
        RuleTag::BadAffects => (
            kind::OKF_BAD_AFFECTS,
            "write the entry as `cap:<slug>#<section>`",
        ),
        RuleTag::BadAffectsSection => (
            kind::OKF_BAD_AFFECTS_SECTION,
            "name one of the seven capability sections",
        ),
        RuleTag::AffectsNotInChanges => (
            kind::OKF_AFFECTS_NOT_IN_CHANGES,
            "add the capability to `changes`, or remove the entry",
        ),
        RuleTag::BadContractEffect => (
            kind::OKF_BAD_CONTRACT_EFFECT,
            "set `contract_effect` to none or changes",
        ),
        RuleTag::ItemUnknownKey => (
            kind::OKF_ITEM_UNKNOWN_KEY,
            "keep only `claim`, `outcome` and `reason` in an item",
        ),
        RuleTag::ItemClaimEmpty => (
            kind::OKF_ITEM_CLAIM_EMPTY,
            "give the item a non-empty `claim`",
        ),
        RuleTag::BadOutcome => (
            kind::OKF_BAD_OUTCOME,
            "set `outcome` to implemented, dropped or moved:change:<slug>",
        ),
        RuleTag::EmptyTag => (
            kind::OKF_EMPTY_TAG,
            "remove the empty tag or write its text",
        ),
        RuleTag::DuplicateTag => (kind::OKF_DUPLICATE_TAG, "remove the repeated tag"),
        RuleTag::BadTimestamp => (kind::OKF_BAD_TIMESTAMP, "write the timestamp as RFC 3339"),
        RuleTag::SectionMissing => (
            kind::OKF_SECTION_MISSING,
            "add the missing second-level section",
        ),
        RuleTag::SectionUnexpected => (
            kind::OKF_SECTION_UNEXPECTED,
            "remove the section or rename it to one the type has",
        ),
        RuleTag::SectionDuplicated => (
            kind::OKF_SECTION_DUPLICATED,
            "merge the repeated sections into one",
        ),
        RuleTag::SectionsOutOfOrder => (
            kind::OKF_SECTIONS_OUT_OF_ORDER,
            "reorder the sections into the order of the type",
        ),
        RuleTag::SectionEmpty => (kind::OKF_SECTION_EMPTY, "write the contents of the section"),
        RuleTag::BodyEmpty => (kind::OKF_BODY_EMPTY, "write the body of the page"),
        RuleTag::ProposedOutsideCapability => (
            kind::OKF_PROPOSED_OUTSIDE_CAPABILITY,
            "move the PROPOSED block into a capability document",
        ),
        RuleTag::ProposedOutsideSection => (
            kind::OKF_PROPOSED_OUTSIDE_SECTION,
            "move the PROPOSED block under a second-level heading",
        ),
        RuleTag::Placeholder => (
            kind::OKF_PLACEHOLDER,
            "replace the placeholder with the resolved text",
        ),
        RuleTag::SplitLink => (kind::OKF_SPLIT_LINK, "write the link on one line"),
        RuleTag::ClaimOutsideCapability => (
            kind::OKF_CLAIM_OUTSIDE_CAPABILITY,
            "move the statement and its claim marker into a capability document",
        ),
        RuleTag::BodyNotEmpty => (
            kind::OKF_BODY_NOT_EMPTY,
            "remove the body; a record states its result in the frontmatter",
        ),
        RuleTag::SingletonId => (
            kind::OKF_SINGLETON_ID,
            "name the record `coverage:default` at work/coverage/default.md",
        ),
        RuleTag::BadSubject => (
            kind::OKF_BAD_SUBJECT,
            "write the subject in the form of the record type",
        ),
        RuleTag::CommandSubject => (
            kind::OKF_COMMAND_SUBJECT,
            "state the `command` exactly when the subject is `command`",
        ),
        RuleTag::RowUnknownKey => (kind::OKF_ROW_UNKNOWN_KEY, "keep only the keys of the row"),
        RuleTag::RowMissingKey => (kind::OKF_ROW_MISSING_KEY, "add the missing key to the row"),
        RuleTag::BadInputKind => (
            kind::OKF_BAD_INPUT_KIND,
            "use an input kind the record type takes",
        ),
        RuleTag::BadInputName => (
            kind::OKF_BAD_INPUT_NAME,
            "write the input name in the form of its kind",
        ),
        RuleTag::BadDigest => (
            kind::OKF_BAD_DIGEST,
            "write the digest as 64 lowercase hexadecimal digits",
        ),
        RuleTag::DuplicateEntry => (kind::OKF_DUPLICATE_ENTRY, "remove the repeated entry"),
        RuleTag::EmptyList => (kind::OKF_EMPTY_LIST, "list at least one entry"),
        RuleTag::SubjectInputMissing => (
            kind::OKF_SUBJECT_INPUT_MISSING,
            "add the `section` input row of the subject",
        ),
        RuleTag::BadVerdict => (
            kind::OKF_BAD_VERDICT,
            "set `verdict` to one of the verdicts of the record type",
        ),
        RuleTag::BadActor => (kind::OKF_BAD_ACTOR, "set `by` to human or agent"),
        RuleTag::BadRevision => (
            kind::OKF_BAD_REVISION,
            "write the revision as a full commit id",
        ),
        RuleTag::BadMeasure => (kind::OKF_BAD_MEASURE, "write the measure as a number"),
        RuleTag::LimitWithoutValue => (
            kind::OKF_LIMIT_WITHOUT_VALUE,
            "remove the limit, or record the value it bounds",
        ),
        RuleTag::VerdictOverLimit => (
            kind::OKF_VERDICT_OVER_LIMIT,
            "record the verdict as fail, or correct the value",
        ),
        RuleTag::BadCrateName => (
            kind::OKF_BAD_CRATE_NAME,
            "write the crate by its package name",
        ),
        RuleTag::BadExclusionItem => (
            kind::OKF_BAD_EXCLUSION_ITEM,
            "write the item as a path or a `<module>::*` prefix",
        ),
        RuleTag::CrateRootExclusion => (
            kind::OKF_CRATE_ROOT_EXCLUSION,
            "remove the crate from `scope` instead of excluding it",
        ),
        RuleTag::EmptyReason => (
            kind::OKF_EMPTY_REASON,
            "give the exclusion or the exception its reason",
        ),
        RuleTag::BadStrictness => (
            kind::OKF_BAD_STRICTNESS,
            "set `strictness` to default or strict, or remove the key, which reads as default",
        ),
        RuleTag::TestedByOutsideCapability => (
            kind::OKF_TESTED_BY_OUTSIDE_CAPABILITY,
            "move the statement and its tested-by marker into a capability document",
        ),
        RuleTag::ItemReasonEmpty => (
            kind::OKF_ITEM_REASON_EMPTY,
            "give the item a non-empty `reason`, or remove the key",
        ),
        RuleTag::BadCeiling => (
            kind::OKF_BAD_CEILING,
            "write the row as `crate`, exactly one of `lines` (a whole number) or `percent` (from 0 to 100), and an optional `of` of `shipping`, `test` or `all`",
        ),
        RuleTag::BadException => (
            kind::OKF_BAD_EXCEPTION,
            "write the row with exactly one of `item` (an item path or a `<module>::*` prefix below a crate root) or `file` (a path relative to the repository root, a directory ending in `/`), and its `reason`",
        ),
    };
    (kind, Cow::Borrowed(fix))
}

/// The kind and the fix of a reader error.
#[allow(
    clippy::too_many_lines,
    reason = "one arm per error kind of the reader"
)]
fn error_entry(tag: ErrorTag) -> (&'static str, &'static str) {
    match tag {
        ErrorTag::MissingFence => (
            kind::STRAY_MARKDOWN,
            "give the file a frontmatter block, or remove the file; only typed documents belong in a document directory",
        ),
        ErrorTag::UnclosedFence => (
            kind::OKF_PARSE_UNCLOSED_FENCE,
            "close the frontmatter with a `---` line",
        ),
        ErrorTag::TooLarge => (
            kind::OKF_PARSE_TOO_LARGE,
            "split the document below the size bound",
        ),
        ErrorTag::MixedLineEndings => (
            kind::OKF_PARSE_MIXED_LINE_ENDINGS,
            "use the line ending of the opening fence on every line",
        ),
        ErrorTag::ControlCharacter => (
            kind::OKF_PARSE_CONTROL_CHARACTER,
            "remove the control character from the frontmatter",
        ),
        ErrorTag::TrailingWhitespace => (
            kind::OKF_PARSE_TRAILING_WHITESPACE,
            "remove the trailing spaces",
        ),
        ErrorTag::Indentation => (
            kind::OKF_PARSE_INDENTATION,
            "indent by multiples of two spaces, only under a key with no value",
        ),
        ErrorTag::Spacing => (
            kind::OKF_PARSE_SPACING,
            "put exactly one space between the key and its value",
        ),
        ErrorTag::NotAMapping => (
            kind::OKF_PARSE_NOT_A_MAPPING,
            "write the frontmatter as a block mapping of `key: value` lines",
        ),
        ErrorTag::ExpectedKey => (
            kind::OKF_PARSE_EXPECTED_KEY,
            "write the line as `key: value` or `key:`",
        ),
        ErrorTag::ExpectedItem => (
            kind::OKF_PARSE_EXPECTED_ITEM,
            "start the list line with `- `",
        ),
        ErrorTag::InvalidKey => (
            kind::OKF_PARSE_INVALID_KEY,
            "use only letters, digits, `_` and `-` in a key",
        ),
        ErrorTag::DuplicateKey => (kind::OKF_PARSE_DUPLICATE_KEY, "remove the repeated key"),
        ErrorTag::MergeKey => (kind::OKF_PARSE_MERGE_KEY, "remove the `<<` merge key"),
        ErrorTag::Anchor => (
            kind::OKF_PARSE_ANCHOR,
            "remove the anchor and write the value out",
        ),
        ErrorTag::Alias => (
            kind::OKF_PARSE_ALIAS,
            "remove the alias and write the value out",
        ),
        ErrorTag::Tag => (kind::OKF_PARSE_TAG, "remove the tag"),
        ErrorTag::BlockScalar => (
            kind::OKF_PARSE_BLOCK_SCALAR,
            "write the value as a quoted scalar",
        ),
        ErrorTag::IndicatorCharacter => (kind::OKF_PARSE_INDICATOR_CHARACTER, "quote the value"),
        ErrorTag::FlowMapping => (
            kind::OKF_PARSE_FLOW_MAPPING,
            "write the value as a block mapping",
        ),
        ErrorTag::NestedFlow => (
            kind::OKF_PARSE_NESTED_FLOW,
            "write the nested list as a block list",
        ),
        ErrorTag::NestedSequence => (
            kind::OKF_PARSE_NESTED_SEQUENCE,
            "remove the list nested in a list item",
        ),
        ErrorTag::UnterminatedFlow => (
            kind::OKF_PARSE_UNTERMINATED_FLOW,
            "close the flow list with `]`",
        ),
        ErrorTag::EmptyFlowEntry => (
            kind::OKF_PARSE_EMPTY_FLOW_ENTRY,
            "remove the empty entry from the flow list",
        ),
        ErrorTag::TextAfterFlow => (
            kind::OKF_PARSE_TEXT_AFTER_FLOW,
            "remove the text after the closing `]`",
        ),
        ErrorTag::UnterminatedQuote => {
            (kind::OKF_PARSE_UNTERMINATED_QUOTE, "close the quoted value")
        }
        ErrorTag::TextAfterQuote => (
            kind::OKF_PARSE_TEXT_AFTER_QUOTE,
            "remove the text after the closing quote",
        ),
        ErrorTag::UnknownEscape => (
            kind::OKF_PARSE_UNKNOWN_ESCAPE,
            "use only the escapes the profile accepts",
        ),
        ErrorTag::PlainColonSpace => (kind::OKF_PARSE_PLAIN_COLON_SPACE, "quote the value"),
        ErrorTag::EmptyItem => (
            kind::OKF_PARSE_EMPTY_ITEM,
            "give the list item a value or remove it",
        ),
        ErrorTag::MixedList => (
            kind::OKF_PARSE_MIXED_LIST,
            "keep one kind of item in a list",
        ),
        ErrorTag::DepthExceeded => (kind::OKF_PARSE_DEPTH_EXCEEDED, "flatten the frontmatter"),
        ErrorTag::BlockNesting => (
            kind::OKF_PARSE_BLOCK_NESTING,
            "flatten the nested quotes and lists",
        ),
        ErrorTag::MalformedProposed => (
            kind::OKF_PARSE_MALFORMED_PROPOSED,
            "write exactly `> [!PROPOSED change:<slug>]`",
        ),
        ErrorTag::MalformedGenerated => (
            kind::OKF_PARSE_MALFORMED_GENERATED,
            "write `<!-- raf:generated:<kind> -->` with a kebab-case kind, or the end marker",
        ),
        ErrorTag::NestedGenerated => (
            kind::OKF_PARSE_NESTED_GENERATED,
            "close the open generated block before opening another",
        ),
        ErrorTag::OrphanGeneratedEnd => (
            kind::OKF_PARSE_ORPHAN_GENERATED_END,
            "remove the end marker or add its start",
        ),
        ErrorTag::UnclosedGenerated => (
            kind::OKF_PARSE_UNCLOSED_GENERATED,
            "close the generated block with its end marker",
        ),
        ErrorTag::MalformedClaim => (
            kind::OKF_PARSE_MALFORMED_CLAIM,
            "write exactly `<!-- raf:claim evidence=<slug>[,<slug>...] -->` on its own line",
        ),
        ErrorTag::UnattachedClaim => (
            kind::OKF_PARSE_UNATTACHED_CLAIM,
            "put the claim marker directly after the paragraph or list item it supports",
        ),
        ErrorTag::MalformedTestedBy => (
            kind::OKF_PARSE_MALFORMED_TESTED_BY,
            "write exactly `<!-- raf:tested-by <path>[,<path>...] -->` on its own line, the paths distinct and separated by `,` with no space",
        ),
        ErrorTag::UnattachedTestedBy => (
            kind::OKF_PARSE_UNATTACHED_TESTED_BY,
            "put the tested-by marker directly after the paragraph, list item or heading it maps, or directly after its claim marker",
        ),
        ErrorTag::UnknownMarker => (
            kind::OKF_PARSE_UNKNOWN_MARKER,
            "remove the comment, or write it as a generated, claim or tested-by marker",
        ),
        ErrorTag::LinkPartTooLong => (
            kind::OKF_PARSE_LINK_PART_TOO_LONG,
            "shorten the link destination or title",
        ),
        ErrorTag::IndexHeader => (
            kind::OKF_PARSE_INDEX_HEADER,
            "keep only SPDX tags with a value in the header comment, before the first block",
        ),
        ErrorTag::IndexText => (
            kind::INDEX_HAND_WRITTEN,
            "remove the text outside the generated blocks; the index is generated",
        ),
        ErrorTag::IndexMarker => (
            kind::OKF_PARSE_INDEX_MARKER,
            "start each marker on its own line with no indentation",
        ),
        ErrorTag::IndexEmpty => (
            kind::OKF_PARSE_INDEX_EMPTY,
            "regenerate the index so it holds a generated block",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(location: Location) -> Finding {
        Finding::new(kind::LINK_BROKEN, "docs/a.md", location, "message", "fix")
    }

    #[test]
    fn each_location_answers_to_exactly_one_accessor() {
        let section = at(Location::Section("Usage".to_string()));
        assert_eq!(
            (section.section(), section.line(), section.key()),
            (Some("Usage"), None, None)
        );
        let line = at(Location::Line(7));
        assert_eq!(
            (line.section(), line.line(), line.key()),
            (None, Some(7), None)
        );
        let key = at(Location::Key("state".to_string()));
        assert_eq!(
            (key.section(), key.line(), key.key()),
            (None, None, Some("state"))
        );
        let document = at(Location::Document);
        assert_eq!(
            (document.section(), document.line(), document.key()),
            (None, None, None)
        );
    }
}
