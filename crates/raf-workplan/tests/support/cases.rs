// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The named cases: one repository per finding kind, each a valid repository
//! with one document broken in one way.

use std::fmt::Write as _;

use raf_okf::{ErrorTag, Location, MAX_TEXT_BYTES, MAX_YAML_DEPTH, Rule, RuleTag};
use raf_workplan::Finding;

use super::fixtures::{ALPHA, BETA, CHANGE, GAMMA, OVERVIEW, bound, mutate, valid};
use super::synced::{self, check_all};
use super::{Repo, check_with_symlinks, key, line_of, section};

/// A repository in which one file breaks one rule.
pub(crate) struct Case {
    /// The name of the fixture.
    pub(crate) name: String,
    /// The whole repository.
    pub(crate) repo: Repo,
    /// The kind the case must report.
    pub(crate) kind: String,
    /// The file that breaks the rule.
    pub(crate) file: String,
    /// Where in the file the finding points.
    pub(crate) at: Location,
    /// The other kinds the same break also causes, all in `file`.
    pub(crate) also: Vec<&'static str>,
    /// The paths of the repository that are tracked symlinks.
    pub(crate) symlinks: Vec<&'static str>,
    /// Whether the case keeps the findings that only say the repository is not
    /// synced (see `without_sync_findings`).
    pub(crate) unfiltered: bool,
}

impl Case {
    fn new(name: &str, repo: Repo, kind: &str, file: &str, at: Location) -> Self {
        Self {
            name: name.to_string(),
            repo,
            kind: kind.to_string(),
            file: file.to_string(),
            at,
            also: Vec::new(),
            symlinks: Vec::new(),
            unfiltered: false,
        }
    }

    /// A case in a synced repository, judged with every finding kept.
    fn unfiltered(mut self) -> Self {
        self.unfiltered = true;
        self
    }

    /// The paths of the repository that are tracked symlinks.
    fn symlinks(mut self, paths: &[&'static str]) -> Self {
        self.symlinks = paths.to_vec();
        self
    }

    fn also(mut self, kinds: &[&'static str]) -> Self {
        self.also = kinds.to_vec();
        self
    }

    /// Run the case and return every finding.
    pub(crate) fn run(&self) -> Vec<Finding> {
        if self.unfiltered {
            check_all(&self.repo)
        } else {
            check_with_symlinks(&self.repo, &self.symlinks)
        }
    }

    /// Check the findings: the wanted kind is at the wanted place in the
    /// file, and nothing else is reported, in this file or in its neighbours,
    /// beyond the kinds the case names.
    pub(crate) fn assert_exact(&self) {
        let found = self.run();
        assert!(
            found
                .iter()
                .any(|f| f.kind == self.kind && f.file == self.file && f.location == self.at),
            "{}: expected `{}` in {} at {:?}, got: {found:#?}",
            self.name,
            self.kind,
            self.file,
            self.at
        );
        for finding in &found {
            assert!(
                finding.file == self.file
                    && (finding.kind == self.kind || self.also.contains(&finding.kind)),
                "{}: unexpected finding {finding:#?} among {found:#?}",
                self.name
            );
        }
    }
}

fn architecture(front: &str, body: &str) -> String {
    format!(
        "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
         description: How it fits.\n{front}---\n\n{body}"
    )
}

fn in_overview(text: String) -> Repo {
    let mut repo = valid();
    repo.insert(OVERVIEW.to_string(), text);
    repo
}

/// The case for a rule of the profile.
pub(crate) fn rule_case(tag: RuleTag) -> Case {
    rule_case_unplaced(tag)
}

#[allow(clippy::too_many_lines, reason = "one arm per rule of the profile")]
fn rule_case_unplaced(tag: RuleTag) -> Case {
    let name = format!("okf-{tag}");
    let kind = name.clone();
    let on = |path: &str, from: &str, to: &str, at: Location| {
        Case::new(&name, mutate(path, from, to), &kind, path, at)
    };
    let alpha = |from: &str, to: &str, at: Location| on(ALPHA, from, to, at);
    let overview = |from: &str, to: &str, at: Location| on(OVERVIEW, from, to, at);
    let change = |from: &str, to: &str, at: Location| on(CHANGE, from, to, at);
    match tag {
        RuleTag::MissingKey => alpha(
            "description: A description of alpha.\n",
            "",
            key("description"),
        ),
        RuleTag::UnknownKey => alpha(
            "state: designed",
            "state: designed\ncolour: red",
            key("colour"),
        ),
        RuleTag::ResourceKey => alpha(
            "state: designed",
            "state: designed\nresource: https://example.invalid/alpha",
            key("resource"),
        ),
        RuleTag::UnknownType => overview("type: Architecture", "type: Notes", key("type")),
        RuleTag::WrongDirectory => {
            // A well-formed architecture page in the capability directory: the
            // profile and the directory rule both see it.
            let mut repo = valid();
            let text = repo.remove(OVERVIEW).expect("the fixture has an overview");
            let moved = "docs/capabilities/overview.md";
            repo.insert(moved.to_string(), text);
            Case::new(&name, repo, &kind, moved, key("type")).also(&["wrong-directory"])
        }
        RuleTag::WrongShape => overview("type: Architecture", "type: [Architecture]", key("type")),
        RuleTag::InvalidId => overview("id: arch:overview", "id: overview", key("id")),
        RuleTag::IdFileName => overview("id: arch:overview", "id: arch:other", key("id")),
        RuleTag::EmptyText => overview("title: Overview", "title: \"\"", key("title")),
        RuleTag::BadState => alpha("state: designed", "state: draft", key("state")),
        RuleTag::BindingsKeys => alpha("  examples: []\n", "", key("bindings")),
        RuleTag::BindingNotAList => alpha("  code: []", "  code: src", key("bindings")),
        RuleTag::BindingNotEmpty => alpha("  code: []", "  code: [demo::real]", key("bindings")),
        RuleTag::BindingRequired => {
            on(GAMMA, "  code: [demo::real]", "  code: []", key("bindings"))
        }
        RuleTag::BadCapabilityId => alpha("depends: [cap:beta]", "depends: [beta]", key("depends")),
        RuleTag::EmptyChanges => change(
            "changes: [cap:alpha, cap:beta]",
            "changes: []",
            key("changes"),
        )
        .also(&["okf-affects-not-in-changes"]),
        RuleTag::BadAffects => change("cap:alpha#contract", "cap:alpha", key("affects")),
        RuleTag::BadAffectsSection => {
            change("cap:alpha#contract", "cap:alpha#nonsense", key("affects"))
        }
        RuleTag::AffectsNotInChanges => {
            change("cap:alpha#contract", "cap:gamma#contract", key("affects"))
        }
        RuleTag::BadContractEffect => change(
            "contract_effect: changes",
            "contract_effect: sometimes",
            key("contract_effect"),
        ),
        RuleTag::ItemUnknownKey => change(
            "  - claim: \"beta is sketched\"\n",
            "  - claim: \"beta is sketched\"\n    note: x\n",
            key("items"),
        ),
        RuleTag::ItemClaimEmpty => change(
            "  - claim: \"beta is sketched\"",
            "  - claim: \"\"",
            key("items"),
        ),
        RuleTag::BadOutcome => change("outcome: implemented", "outcome: done", key("items")),
        RuleTag::EmptyTag => alpha(
            "state: designed",
            "state: designed\ntags: [\"\"]",
            key("tags"),
        ),
        RuleTag::DuplicateTag => alpha(
            "state: designed",
            "state: designed\ntags: [a, a]",
            key("tags"),
        ),
        RuleTag::BadTimestamp => alpha(
            "state: designed",
            "state: designed\ntimestamp: yesterday",
            key("timestamp"),
        ),
        RuleTag::SectionMissing => alpha("## Usage\n\nText for Usage.\n\n", "", section("Usage")),
        RuleTag::SectionUnexpected => alpha(
            "Text for Verification.\n",
            "Text for Verification.\n\n## Swap\n\nText.\n",
            section("Swap"),
        ),
        RuleTag::SectionDuplicated => alpha(
            "Text for Verification.\n",
            "Text for Verification.\n\n## Verification\n\nAgain.\n",
            Location::Document,
        ),
        RuleTag::SectionsOutOfOrder => {
            let swapped = "## Contract\n\nText for Contract.\n\n## Usage\n\nText for Usage.\n";
            alpha(
                swapped,
                "## Usage\n\nText for Usage.\n\n## Contract\n\nText for Contract.\n",
                Location::Document,
            )
        }
        RuleTag::SectionEmpty => alpha(
            "## Usage\n\nText for Usage.\n",
            "## Usage\n",
            section("Usage"),
        ),
        RuleTag::BodyEmpty => overview("\n# Overview\n\nBody text.\n", "", Location::Document),
        RuleTag::ProposedOutsideCapability => {
            let text = mutate(
                OVERVIEW,
                "Body text.",
                "> [!PROPOSED change:add-alpha]\n> Not allowed here.",
            );
            let line = line_of(&text[OVERVIEW], "[!PROPOSED");
            Case::new(&name, text, &kind, OVERVIEW, Location::Line(line))
        }
        RuleTag::ProposedOutsideSection => {
            let mut repo = valid();
            let beta = repo.get_mut(BETA).expect("the fixture has a beta");
            let marker = "> [!PROPOSED change:add-alpha]\n> The candidate contract.\n\n";
            *beta = beta.replacen(marker, "", 1).replacen(
                "# beta\n\n",
                &format!("# beta\n\n{marker}"),
                1,
            );
            let line = line_of(&repo[BETA], "[!PROPOSED");
            Case::new(&name, repo, &kind, BETA, Location::Line(line))
        }
        RuleTag::Placeholder => {
            let token = concat!("to", "do");
            let repo = mutate(
                OVERVIEW,
                "Body text.",
                &format!("Body text. Still {token} here."),
            );
            let line = line_of(&repo[OVERVIEW], token);
            Case::new(&name, repo, &kind, OVERVIEW, Location::Line(line))
        }
        RuleTag::SplitLink => {
            let repo = mutate(
                OVERVIEW,
                "Body text.",
                "Body text. See [the page](\nother.md).",
            );
            let line = line_of(&repo[OVERVIEW], "[the page](");
            Case::new(&name, repo, &kind, OVERVIEW, Location::Line(line))
        }
    }
}

/// The case for a way a document fails to read.
#[allow(
    clippy::too_many_lines,
    reason = "one arm per error kind of the reader"
)]
pub(crate) fn parse_case(tag: ErrorTag) -> Case {
    let name = format!("okf-parse-{tag}");
    let kind = name.clone();
    // Front matter lines go in from line 6; the body starts on line 8 of a
    // document with no extra lines.
    let front = |extra: &str, line: usize| {
        Case::new(
            &name,
            in_overview(architecture(extra, "# Overview\n\nBody text.\n")),
            &kind,
            OVERVIEW,
            Location::Line(line),
        )
    };
    let body = |text: &str, line: usize| {
        Case::new(
            &name,
            in_overview(architecture("", text)),
            &kind,
            OVERVIEW,
            Location::Line(line),
        )
    };
    let whole = |text: &str, line: usize| {
        Case::new(
            &name,
            in_overview(text.to_string()),
            &kind,
            OVERVIEW,
            Location::Line(line),
        )
    };
    let index = |text: &str, line: usize| {
        let mut repo = valid();
        repo.insert("docs/capabilities/index.md".to_string(), text.to_string());
        Case::new(
            &name,
            repo,
            &kind,
            "docs/capabilities/index.md",
            Location::Line(line),
        )
    };
    match tag {
        ErrorTag::MissingFence => {
            // A page with no frontmatter is `stray-markdown`; the case is named
            // for the reader's error it comes from.
            let mut case = whole("# Overview\n\nNo frontmatter.\n", 1);
            case.kind = "stray-markdown".to_string();
            case
        }
        ErrorTag::UnclosedFence => whole("---\ntype: Architecture\n", 1),
        ErrorTag::TooLarge => {
            let head = architecture("", "# Overview\n\n");
            let pad = MAX_TEXT_BYTES + 1 - head.len();
            whole(
                &format!("{head}{}", "a".repeat(pad)),
                head.lines().count() + 1,
            )
        }
        ErrorTag::MixedLineEndings => whole(
            "---\r\ntype: Architecture\nid: arch:overview\ntitle: Overview\ndescription: How it fits.\n---\n\n# Overview\n",
            6,
        ),
        ErrorTag::ControlCharacter => front("note: a\u{1}b\n", 6),
        ErrorTag::TrailingWhitespace => front("note: a \n", 6),
        ErrorTag::Indentation => front("note:\n   deep: x\n", 7),
        ErrorTag::Spacing => front("note:  x\n", 6),
        ErrorTag::NotAMapping => whole("---\n- a\n---\n\n# Overview\n", 2),
        ErrorTag::ExpectedKey => front("just text\n", 6),
        ErrorTag::ExpectedItem => front("tags:\n  - a\n  b\n", 8),
        ErrorTag::InvalidKey => front("bad key: x\n", 6),
        ErrorTag::DuplicateKey => front("title: Again\n", 6),
        ErrorTag::MergeKey => front("<<: x\n", 6),
        ErrorTag::Anchor => front("note: &a x\n", 6),
        ErrorTag::Alias => front("note: *a\n", 6),
        ErrorTag::Tag => front("note: !t x\n", 6),
        ErrorTag::BlockScalar => front("note: |\n", 6),
        ErrorTag::IndicatorCharacter => front("note: @x\n", 6),
        ErrorTag::FlowMapping => front("note: {a: b}\n", 6),
        ErrorTag::NestedFlow => front("tags: [[a]]\n", 6),
        ErrorTag::NestedSequence => front("tags:\n  - - a\n", 7),
        ErrorTag::UnterminatedFlow => front("tags: [a, b\n", 6),
        ErrorTag::EmptyFlowEntry => front("tags: [a, , b]\n", 6),
        ErrorTag::TextAfterFlow => front("tags: [a] x\n", 6),
        ErrorTag::UnterminatedQuote => front("note: \"abc\n", 6),
        ErrorTag::TextAfterQuote => front("note: \"abc\" x\n", 6),
        ErrorTag::UnknownEscape => front("note: \"a\\qb\"\n", 6),
        ErrorTag::PlainColonSpace => front("note: a: b\n", 6),
        ErrorTag::EmptyItem => front("tags:\n  -\n", 7),
        ErrorTag::MixedList => front("tags:\n  - a\n  - k: v\n", 8),
        ErrorTag::DepthExceeded => {
            let mut nested = String::new();
            for level in 0..=MAX_YAML_DEPTH {
                nested.push_str(&"  ".repeat(level));
                writeln!(nested, "k{level}:").expect("a string takes text");
            }
            nested.push_str(&"  ".repeat(MAX_YAML_DEPTH + 1));
            nested.push_str("leaf: x\n");
            front(&nested, 6 + MAX_YAML_DEPTH)
        }
        ErrorTag::BlockNesting => body(&format!("{}x\n", "> ".repeat(65)), 8),
        ErrorTag::MalformedProposed => body("> [!proposed change:x]\n> text\n", 8),
        ErrorTag::MalformedGenerated => body("<!-- raf:generated:Bad -->\n", 8),
        ErrorTag::NestedGenerated => body(
            "<!-- raf:generated:a -->\n<!-- raf:generated:b -->\n<!-- raf:generated:end -->\n",
            9,
        ),
        ErrorTag::OrphanGeneratedEnd => body("<!-- raf:generated:end -->\n", 8),
        ErrorTag::UnclosedGenerated => body("<!-- raf:generated:a -->\ntext\n", 8),
        ErrorTag::LinkPartTooLong => body(&format!("[a]({})\n", "x".repeat(4097)), 8),
        ErrorTag::IndexHeader => index(
            "<!--\nnot a tag\n-->\n<!-- raf:generated:a -->\n<!-- raf:generated:end -->\n",
            2,
        ),
        ErrorTag::IndexText => {
            // Text in the index is `index-hand-written`; the case is named for
            // the reader's error it comes from.
            let mut case = index("hand-written text\n", 1);
            case.kind = "index-hand-written".to_string();
            case
        }
        ErrorTag::IndexMarker => index("<!-- raf:generated:Bad -->\n", 1),
        ErrorTag::IndexEmpty => index("\n", 1),
    }
}

/// The cases for the rules of the flat directories and for unreadable input.
pub(crate) fn directory_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    // A typed document in the directory of another type.
    let moved = |name: &str, from: &str, to: &str, also: &[&'static str]| {
        let mut repo = valid();
        let text = repo.remove(from).expect("the fixture holds the document");
        repo.insert(to.to_string(), text);
        Case::new(name, repo, "wrong-directory", to, key("type")).also(also)
    };
    cases.push(moved(
        "wrong-directory-architecture-in-capabilities",
        OVERVIEW,
        "docs/capabilities/overview.md",
        &["okf-wrong-directory"],
    ));
    cases.push(moved(
        "wrong-directory-capability-in-architecture",
        ALPHA,
        "docs/architecture/alpha.md",
        &["okf-wrong-directory"],
    ));
    cases.push(moved(
        "wrong-directory-change-in-capabilities",
        CHANGE,
        "docs/capabilities/add-alpha.md",
        &["okf-wrong-directory"],
    ));
    // A Markdown file below a document directory.
    for path in [
        "docs/capabilities/sub/x.md",
        "docs/architecture/sub/deeper/y.md",
        "work/changes/sub/z.md",
    ] {
        let mut repo = valid();
        repo.insert(path.to_string(), "# Nested\n".to_string());
        cases.push(Case::new(
            &format!("nested-directory-{path}"),
            repo,
            "nested-directory",
            path,
            Location::Document,
        ));
    }
    // Prose without frontmatter where a document belongs.
    for path in [
        "docs/capabilities/plain.md",
        "docs/architecture/plain.md",
        "work/changes/plain.md",
    ] {
        let mut repo = valid();
        repo.insert(
            path.to_string(),
            "# Plain\n\nNo frontmatter here.\n".to_string(),
        );
        cases.push(Case::new(
            &format!("stray-markdown-{path}"),
            repo,
            "stray-markdown",
            path,
            Location::Line(1),
        ));
    }
    // A file over the read bound is reported, not skipped.
    let mut repo = valid();
    let huge = "docs/architecture/huge.md".to_string();
    repo.insert(huge.clone(), "a".repeat(MAX_TEXT_BYTES + 2));
    cases.push(Case::new(
        "unreadable-file-over-the-bound",
        repo,
        "unreadable-file",
        &huge,
        Location::Document,
    ));
    // A document directory that is a file.
    let mut repo = Repo::new();
    repo.insert(
        "docs/capabilities".to_string(),
        "not a directory".to_string(),
    );
    cases.push(Case::new(
        "unreadable-directory-is-a-file",
        repo,
        "unreadable-directory",
        "docs/capabilities",
        Location::Document,
    ));
    cases
}

/// The cases for the rules between documents, and for the outcome the profile
/// refuses.
pub(crate) fn reference_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    let broken = |name: &str, path: &str, from: &str, to: &str, kind: &str, at: Location| {
        Case::new(name, mutate(path, from, to), kind, path, at)
    };
    cases.push(broken(
        "depends-missing",
        ALPHA,
        "depends: [cap:beta]",
        "depends: [cap:beta, cap:ghost]",
        "depends-missing",
        key("depends"),
    ));
    cases.push(broken(
        "changes-missing",
        CHANGE,
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:alpha, cap:beta, cap:ghost]",
        "changes-missing",
        key("changes"),
    ));
    cases.push(broken(
        "moved-missing",
        CHANGE,
        "outcome: implemented",
        "outcome: moved:change:nowhere",
        "moved-missing",
        key("items"),
    ));
    // Reported once, in the file of the first member.
    cases.push(broken(
        "depends-cycle-self",
        ALPHA,
        "depends: [cap:beta]",
        "depends: [cap:alpha, cap:beta]",
        "depends-cycle",
        key("depends"),
    ));
    cases.push(Case::new(
        "depends-cycle-two",
        mutate(BETA, "depends: []", "depends: [cap:alpha]"),
        "depends-cycle",
        ALPHA,
        key("depends"),
    ));
    let mut three = mutate(BETA, "depends: []", "depends: [cap:gamma]");
    three.insert(
        GAMMA.to_string(),
        three[GAMMA].replace("depends: [cap:beta, cap:alpha]", "depends: [cap:alpha]"),
    );
    cases.push(Case::new(
        "depends-cycle-three",
        three,
        "depends-cycle",
        ALPHA,
        key("depends"),
    ));
    // The profile closes the outcome set; a deferred claim is refused.
    cases.push(broken(
        "okf-bad-outcome-deferred",
        CHANGE,
        "outcome: implemented",
        "outcome: deferred",
        "okf-bad-outcome",
        key("items"),
    ));
    cases
}

/// Replace the first `from` of `path` in `repo`.
fn edit(repo: &mut Repo, path: &str, from: &str, to: &str) {
    let text = repo
        .get_mut(path)
        .unwrap_or_else(|| panic!("fixture has no {path}"));
    assert!(text.contains(from), "fixture {path} has no `{from}`");
    *text = text.replacen(from, to, 1);
}

/// The proposed block of the beta fixture.
pub(crate) const BETA_BLOCK: &str = "> [!PROPOSED change:add-alpha]\n> The candidate contract.\n\n";

/// The cases for the PROPOSED block rules and, through `state_rule_cases`, the
/// state rules between documents.
pub(crate) fn state_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    let marker_line = |repo: &Repo, path: &str| Location::Line(line_of(&repo[path], "[!PROPOSED"));
    // A block names a change record that does not exist.
    let repo = mutate(BETA, "change:add-alpha", "change:ghost");
    let at = marker_line(&repo, BETA);
    cases.push(Case::new(
        "proposed-missing-change",
        repo,
        "proposed-missing-change",
        BETA,
        at,
    ));
    // The change exists, and does not list the capability. Beta is then
    // listed by no record at all.
    let mut repo = valid();
    edit(
        &mut repo,
        CHANGE,
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:alpha]",
    );
    edit(&mut repo, CHANGE, "  - cap:beta#purpose\n", "");
    let at = marker_line(&repo, BETA);
    cases.push(
        Case::new(
            "proposed-change-not-listing",
            repo,
            "proposed-change-not-listing",
            BETA,
            at,
        )
        .also(&["no-open-record"]),
    );
    // The change lists the capability and does not affect the section.
    let repo = mutate(CHANGE, "  - cap:beta#purpose\n", "  - cap:beta#usage\n");
    let at = marker_line(&repo, BETA);
    cases.push(Case::new(
        "proposed-section-not-affected",
        repo,
        "proposed-section-not-affected",
        BETA,
        at,
    ));
    cases.extend(state_rule_cases());
    cases
}

/// The cases for the rules a state fixes.
fn state_rule_cases() -> Vec<Case> {
    // A considered capability with no block.
    let mut cases = vec![Case::new(
        "state-considered-no-proposal",
        mutate(BETA, BETA_BLOCK, ""),
        "state-considered-no-proposal",
        BETA,
        key("state"),
    )];
    // A marker inside a code fence is not a block.
    cases.push(Case::new(
        "state-considered-fenced-marker",
        mutate(
            BETA,
            BETA_BLOCK,
            "```markdown\n> [!PROPOSED change:add-alpha]\n> The candidate contract.\n```\n\n",
        ),
        "state-considered-no-proposal",
        BETA,
        key("state"),
    ));
    cases.push(Case::new(
        "state-designed-proposal",
        mutate(
            ALPHA,
            "Text for Contract.\n",
            "> [!PROPOSED change:add-alpha]\n> Not yet.\n\nText for Contract.\n",
        ),
        "state-designed-proposal",
        ALPHA,
        key("state"),
    ));
    // The change does not list gamma, so the block also names a change that
    // does not list its capability.
    cases.push(
        Case::new(
            "state-delivered-proposal",
            mutate(
                GAMMA,
                "Text for Purpose.\n",
                "Text for Purpose.\n\n> [!PROPOSED change:add-alpha]\n> Left over.\n",
            ),
            "state-delivered-proposal",
            GAMMA,
            key("state"),
        )
        .also(&["proposed-change-not-listing"]),
    );
    cases.push(Case::new(
        "state-delivered-open-record",
        mutate(
            CHANGE,
            "changes: [cap:alpha, cap:beta]",
            "changes: [cap:alpha, cap:beta, cap:gamma]",
        ),
        "state-delivered-open-record",
        GAMMA,
        key("state"),
    ));
    let mut repo = valid();
    edit(
        &mut repo,
        CHANGE,
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:beta]",
    );
    edit(&mut repo, CHANGE, "  - cap:alpha#contract\n", "");
    cases.push(Case::new(
        "no-open-record",
        repo,
        "no-open-record",
        ALPHA,
        key("state"),
    ));
    cases
}

/// The cases for the link rules and the issue-reference rule, each in a
/// document that is loaded.
pub(crate) fn link_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    let in_overview = |name: &str, text: &str, kind: &str, needle: &str| {
        let repo = mutate(OVERVIEW, "Body text.", &format!("Body text. {text}"));
        let at = Location::Line(line_of(&repo[OVERVIEW], needle));
        Case::new(name, repo, kind, OVERVIEW, at)
    };
    cases.push(in_overview(
        "link-broken",
        "See [gone](nope.md).",
        "link-broken",
        "nope.md",
    ));
    cases.push(in_overview(
        "anchor-broken",
        "See [gone](../capabilities/alpha.md#no-such-heading).",
        "anchor-broken",
        "no-such-heading",
    ));
    cases.push(in_overview(
        "link-outside",
        "See [out](../../../outside.md).",
        "link-outside",
        "outside.md",
    ));
    let mut linked = in_overview(
        "link-symlink",
        "See [out](../outside-link.svg).",
        "link-symlink",
        "outside-link.svg",
    );
    linked
        .repo
        .insert("docs/outside-link.svg".to_string(), String::new());
    cases.push(linked.symlinks(&["docs/outside-link.svg"]));
    let number = concat!("Per ", "#", "12");
    cases.push(in_overview(
        "issue-reference",
        &format!("{number} said so."),
        "issue-reference",
        number,
    ));
    let phrase = concat!("As agreed in ", "this", " branch");
    cases.push(in_overview(
        "issue-reference-process-wording",
        &format!("{phrase}."),
        "issue-reference",
        phrase,
    ));
    cases
}

const T: &str = "[demo::test_crate::it::case]";
const BINDING_ROWS: [(&str, &str, &str, &str, &str); 15] = [
    ("renamed-symbol", "unresolved", "[demo::nope]", T, "[]"),
    (
        "missing-test",
        "unresolved",
        "[demo::real]",
        "[demo::test_crate::it::missing]",
        "[]",
    ),
    (
        "missing-example-target",
        "unresolved",
        "[demo::real]",
        T,
        "[demo::example_crate::absent]",
    ),
    (
        "item-under-the-declaring-module",
        "unresolved",
        "[demo::body::FenceBlock]",
        T,
        "[]",
    ),
    (
        "macro-generated-symbol",
        "macro-generated",
        "[demo::generated]",
        T,
        "[]",
    ),
    ("ambiguous-glob-names", "ambiguous", "[demo::dup]", T, "[]"),
    (
        "code-in-test-module",
        "wrong-role",
        "[demo::unit::helper]",
        T,
        "[]",
    ),
    (
        "code-in-test-target",
        "wrong-role",
        "[demo::test_crate::it::case]",
        T,
        "[]",
    ),
    (
        "code-through-a-re-export-of-test-code",
        "wrong-role",
        "[demo::unit_helper]",
        T,
        "[]",
    ),
    (
        "test-is-not-a-test-function",
        "wrong-role",
        "[demo::real]",
        "[demo::test_crate::it::plain]",
        "[]",
    ),
    (
        "test-is-shipping-code",
        "wrong-role",
        "[demo::real]",
        "[demo::real]",
        "[]",
    ),
    (
        "example-in-a-target-without-doctests",
        "wrong-role",
        "[demo::real]",
        T,
        "[demo::example_crate::show::helper]",
    ),
    (
        "example-is-undocumented",
        "wrong-role",
        "[demo::real]",
        T,
        "[demo::bare]",
    ),
    (
        "example-is-not-an-example-target",
        "wrong-role",
        "[demo::real]",
        T,
        "[demo::test_crate::it]",
    ),
    (
        "example-doctest-key-on-a-non-library",
        "wrong-role",
        "[demo::real]",
        T,
        "[demo::example_crate::flagged::flagged_helper]",
    ),
];

/// The cases for the binding rules: a delivered capability whose lists hold
/// one entry the source index resolves and the rules refuse. Each row is the
/// name, the kind, and the `code`, `tests` and `examples` lists.
pub(crate) fn binding_cases() -> Vec<Case> {
    BINDING_ROWS
        .iter()
        .map(|(name, kind, code, tests, examples)| {
            Case::new(
                &format!("binding-{name}"),
                bound(code, tests, examples),
                &format!("binding-{kind}"),
                GAMMA,
                key("bindings"),
            )
        })
        .collect()
}

/// Every named case, one or more per kind.
pub(crate) fn all_cases() -> Vec<Case> {
    let mut cases: Vec<Case> = Rule::ALL.into_iter().map(rule_case).collect();
    cases.extend(raf_okf::ErrorKind::ALL.into_iter().map(parse_case));
    cases.extend(directory_cases());
    cases.extend(reference_cases());
    cases.extend(state_cases());
    cases.extend(link_cases());
    cases.extend(binding_cases());
    cases.extend(generated_cases());
    cases
}

/// The cases for the generated blocks: a synced repository with one break
/// each, judged with every finding kept.
#[allow(
    clippy::too_many_lines,
    reason = "one block per generated-block kind, each a repository with one break"
)]
pub(crate) fn generated_cases() -> Vec<Case> {
    let base = synced::base();
    let broken = |path: &str, from: &str, to: &str| {
        let mut repo = base.clone();
        let text = repo.get_mut(path).expect("the fixture holds the file");
        assert!(text.contains(from), "{path} has no `{from}`");
        *text = text.replacen(from, to, 1);
        repo
    };
    let at = |repo: &Repo, path: &str, needle: &str| Location::Line(line_of(&repo[path], needle));
    let mut cases = Vec::new();

    let mut repo = base.clone();
    repo.remove(synced::INDEX);
    cases.push(
        Case::new(
            "index-missing-with-capabilities",
            repo,
            "index-missing",
            synced::INDEX,
            Location::Document,
        )
        .unfiltered(),
    );

    let repo = broken(
        synced::GAMMA,
        "| Depended on by | [alpha](./alpha.md) |",
        "| Depended on by | [alphx](./alpha.md) |",
    );
    let line = at(&repo, synced::GAMMA, "raf:generated:relationships");
    cases.push(
        Case::new(
            "generated-stale-edited-relationship-row",
            repo,
            "generated-stale",
            synced::GAMMA,
            line,
        )
        .unfiltered(),
    );

    let repo = broken(
        synced::BETA,
        "Text for Purpose of beta.\n",
        "Text for Purpose of beta.\n\n<!-- raf:generated:nope -->\n<!-- raf:generated:end -->\n",
    );
    let line = at(&repo, synced::BETA, "raf:generated:nope");
    cases.push(
        Case::new(
            "generated-unknown-kind-in-a-section",
            repo,
            "generated-unknown-kind",
            synced::BETA,
            line,
        )
        .unfiltered(),
    );

    let repo = broken(
        synced::BETA,
        "Text for Purpose of beta.\n",
        "Text for Purpose of beta.\n\n<!-- raf:generated:catalogue -->\n<!-- raf:generated:end -->\n",
    );
    let line = at(&repo, synced::BETA, "raf:generated:catalogue");
    cases.push(
        Case::new(
            "generated-misplaced-catalogue-in-a-capability",
            repo,
            "generated-misplaced",
            synced::BETA,
            line,
        )
        .unfiltered(),
    );

    let repo = broken(
        synced::ALPHA,
        "Text for Purpose of alpha.\n",
        "Text for Purpose of alpha.\nCRLF line here\r\n",
    );
    let line = at(&repo, synced::ALPHA, "CRLF line here");
    cases.push(
        Case::new(
            "mixed-line-endings-in-a-body-with-blocks",
            repo,
            "mixed-line-endings",
            synced::ALPHA,
            line,
        )
        .unfiltered(),
    );

    let mut repo = base;
    repo.get_mut(synced::INDEX)
        .expect("the fixture holds the index")
        .push_str("\nHand written.\n");
    let line = at(&repo, synced::INDEX, "Hand written.");
    cases.push(
        Case::new(
            "index-hand-written-after-the-catalogue",
            repo,
            "index-hand-written",
            synced::INDEX,
            line,
        )
        .unfiltered(),
    );
    cases
}
