// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The two acceptance scenarios of the completion gate, built as real
//! repositories in temporary directories with real cargo crates, through the
//! library.
//!
//! * Scenario A, "several redesigns, one readable present": one capability
//!   passes through four redesigns by successive changes, each opened with
//!   `change new`, proposed in PROPOSED blocks, built, written up, reconciled to
//!   an empty list and completed. At the end the document states the contract
//!   that applies, directly, and every earlier state is in git.
//! * Scenario B, "planning produces the documentation": a `considered`
//!   capability with open questions is implemented with a renamed API and one
//!   guarantee dropped. `change complete` refuses at each step until the
//!   documentation is the delivered account, and then leaves it so. A document
//!   that merely exists does not satisfy it.
//!
//! Each assertion that checks a sentence of the Verification section of the
//! completion gate is preceded by a comment that quotes the sentence. The
//! projects are generated from the templates in `support/`: a crate with its
//! own `[workspace]`, a template `Cargo.lock`, no dependency and a pinned
//! edition, owned target and build directories per project and the explicit `$CARGO`.
//! Nothing is a shell script.

mod support;

use std::collections::BTreeSet;
use std::path::Path;

use raf_workplan::exec::Programs;
use raf_workplan::lifecycle::{Completion, NewChangeRequest, StateAdvice};
use raf_workplan::{BindingKind, Gate, Opened};
use support::assessed::assess_on_disk;
use support::disk::{Scratch, on_disk};
use support::project::{COPYRIGHT, LICENCE, Project, cargo};
use support::story::{
    Bindings, Capability, HEADINGS, author_record, capability, generated_block, index_of, links_of,
    proposal, section_text,
};
use support::streaming::{self, REDESIGNS, Redesign, WORDING};
use support::window;

// --- shared steps ----------------------------------------------------------------

/// The path of the record of a change.
fn record_path(slug: &str) -> String {
    format!("work/changes/{slug}.md")
}

/// The lines of an obligation list, as `kind section`, in the order the list
/// prints them.
fn owed(project: &Project, slug: &str) -> Vec<String> {
    let reconciled = project
        .lifecycle()
        .reconcile(slug)
        .unwrap_or_else(|error| panic!("reconcile answers: {error}"));
    reconciled
        .obligations()
        .iter()
        .map(|obligation| format!("{} {}", obligation.kind.name(), obligation.section))
        .collect()
}

/// The lines of `owed` that are not about an assessment, and the ones that
/// are, in order.
fn split_assessments(owed: Vec<String>) -> (Vec<String>, Vec<String>) {
    owed.into_iter()
        .partition(|line| !line.starts_with("assessment-"))
}

/// The failures of a refusal, as `condition:name` and, for an obligation, the
/// section after it, sorted.
fn shape(completion: &Completion) -> Vec<String> {
    assert!(!completion.is_completed(), "{completion}");
    let mut lines: Vec<String> = completion
        .failures()
        .iter()
        .map(|failure| {
            let line = format!("{}:{}", failure.condition().name(), failure.name());
            match failure.obligation() {
                Some(obligation) => format!("{line} {}", obligation.section),
                None => line,
            }
        })
        .collect();
    lines.sort();
    lines
}

/// Run `change complete` and require a refusal that changed nothing: the tree
/// is byte for byte what it was, and the record is still there.
fn refused(project: &Project, slug: &str) -> Completion {
    let before = project.snapshot();
    let completion = project
        .lifecycle()
        .complete(slug)
        .unwrap_or_else(|error| panic!("complete answers: {error}"));
    assert!(!completion.is_completed(), "{completion}");
    assert_eq!(project.snapshot(), before, "a refusal writes nothing");
    assert!(project.exists(&record_path(slug)), "the record is kept");
    completion
}

/// Require that `completion` is a refusal with exactly these failures.
fn assert_refusal<S: AsRef<str>>(label: &str, completion: &Completion, expected: &[S]) {
    let mut expected: Vec<String> = expected.iter().map(|s| s.as_ref().to_string()).collect();
    expected.sort();
    assert_eq!(shape(completion), expected, "{label}:\n{completion}");
}

/// Run `change complete` and require that it removes the record and rewrites
/// the capability document, and nothing else.
fn completed(project: &Project, slug: &str, document: &str) {
    let completion = project
        .lifecycle()
        .complete(slug)
        .unwrap_or_else(|error| panic!("complete answers: {error}"));
    assert_eq!(
        completion,
        Completion::Completed {
            evidence: completion.evidence().to_vec(),
            removed: record_path(slug),
            rewritten: vec![document.to_string()],
        },
        "{completion}"
    );
    assert!(!project.exists(&record_path(slug)), "the record is deleted");
}

/// The text with its whitespace collapsed, so that a sentence the document
/// wraps over two lines is still found.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether `path` names a Markdown file.
fn is_markdown(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

// --- scenario A ------------------------------------------------------------------

/// The capability document at `version`, in `state`, with these PROPOSED
/// blocks.
fn streaming_document(version: usize, state: &str, blocks: Vec<(usize, String)>) -> String {
    capability(&Capability {
        slug: streaming::SLUG,
        title: streaming::TITLE,
        description: streaming::DESCRIPTION,
        state,
        bindings: streaming::bindings(version),
        sections: streaming::sections(version),
        blocks,
    })
}

/// A state of the capability that was delivered, and is in git.
struct Landed {
    version: usize,
    revision: String,
    document: String,
    library: String,
    tests: String,
}

impl Landed {
    fn of(project: &Project, version: usize) -> Self {
        Self {
            version,
            revision: project.git(&["rev-parse", "HEAD"]),
            document: project.text(streaming::DOCUMENT),
            library: project.text("src/lib.rs"),
            tests: project.text("tests/parser.rs"),
        }
    }
}

/// The commits of one redesign: the proposal (and its amendments), the build,
/// the account and the completion.
struct Commits {
    slug: &'static str,
    revisions: Vec<String>,
}

/// The first state: the capability delivered, bound, with its crate, its
/// architecture page and the README that links to it.
fn genesis() -> (Project, Landed) {
    let mut files: Vec<(String, String)> = vec![
        ("README.md".to_string(), streaming::readme()),
        (
            streaming::ARCHITECTURE.to_string(),
            streaming::FRAMING.to_string(),
        ),
        (
            streaming::DOCUMENT.to_string(),
            streaming_document(0, "delivered", Vec::new()),
        ),
    ];
    for (path, text) in streaming::files(0) {
        files.push((path.to_string(), text.to_string()));
    }
    let project = Project::create("scenario-a", streaming::PACKAGE, &files);
    // The sync creates the capability index and fills the generated blocks.
    project.sync();
    project.commit("docs: the streaming parser, delivered");
    project.publish();
    let landed = Landed::of(&project, 0);
    (project, landed)
}

/// What `reconcile` lists while the proposals of `redesign` stand, beside the
/// assessments: an unresolved proposal for each block, the sections that are
/// affected and carry no block untouched, and each item without outcome.
fn expected_while_proposed(redesign: &Redesign) -> Vec<String> {
    let proposed = redesign.drafts.last().expect("a draft");
    let mut lines = Vec::new();
    for heading in HEADINGS {
        let slug = heading.to_lowercase();
        if proposed.iter().any(|(section, _)| *section == slug) {
            lines.push(format!("unresolved-proposal {slug}"));
        } else if redesign.affects.contains(&slug.as_str()) {
            lines.push(format!("section-untouched {slug}"));
        }
    }
    for number in 1..=redesign.items.len() {
        lines.push(format!("item-without-outcome item {number}"));
    }
    lines
}

/// The assessment lines while the proposals of `redesign` stand, before
/// anything is built: with no earlier assessment, each affected section has
/// none; after the earlier redesigns were assessed, a section the proposal
/// edited has moved, and an affected section it left as it was is current.
fn assessments_while_proposed(redesign: &Redesign, first: bool) -> Vec<String> {
    let proposed = redesign.drafts.last().expect("a draft");
    HEADINGS
        .iter()
        .map(|heading| heading.to_lowercase())
        .filter(|slug| redesign.affects.contains(&slug.as_str()))
        .filter_map(|slug| {
            if first {
                Some(format!("assessment-missing {slug}"))
            } else {
                proposed
                    .iter()
                    .any(|(section, _)| *section == slug)
                    .then(|| format!("assessment-stale {slug}"))
            }
        })
        .collect()
}

/// The items of a redesign with their outcomes, or without.
fn items(
    redesign: &Redesign,
    settled: bool,
) -> Vec<(&'static str, Option<&'static str>, Option<&'static str>)> {
    redesign
        .items
        .iter()
        .map(|(claim, outcome)| {
            let reason = if settled && *outcome == "dropped" {
                Some(match *claim {
                    "An error names the line and the column" =>
                        "Malformed input is diagnosed by its line; the implemented error type does not retain a column",
                    "The limit counts characters and not bytes" =>
                        "The parser bounds its retained UTF-8 bytes, so the implemented limit counts bytes rather than characters",
                    other => panic!("the dropped claim needs its specific reason: {other}"),
                })
            } else {
                None
            };
            (*claim, settled.then_some(*outcome), reason)
        })
        .collect()
}

/// The blocks of a draft, from the template `change new` printed.
fn blocks_of(template: &str, draft: &[(&str, &str)]) -> Vec<(usize, String)> {
    draft
        .iter()
        .map(|(section, text)| (index_of(section), proposal(template, text)))
        .collect()
}

/// One redesign in progress: the change, the template `change new` printed
/// for it, and the version the capability is at before it.
struct Step<'a> {
    project: &'a Project,
    redesign: &'a Redesign,
    before: usize,
    template: String,
    path: String,
}

impl<'a> Step<'a> {
    /// Fork a branch and open the change with `change new`. The capability is
    /// delivered, so the guidance says what the author does to its state.
    fn open(project: &'a Project, redesign: &'a Redesign, before: usize) -> Self {
        let root = project.root();
        let path = record_path(redesign.slug);
        project.fork(redesign.slug);
        let mut request = NewChangeRequest::new(redesign.slug)
            .capability(streaming::ID)
            .copyright(COPYRIGHT)
            .licence(LICENCE);
        for section in redesign.affects {
            request = request.affects(format!("{}#{section}", streaming::ID));
        }
        let created = on_disk(root, |plan| plan.new_change(root, &request))
            .unwrap_or_else(|error| panic!("`change new` should succeed: {error}"));
        assert_eq!(created.path.as_str(), path);
        assert_eq!(created.guidance.len(), 1);
        assert_eq!(created.guidance[0].advice, StateAdvice::SetImplementing);
        assert!(
            created.sync_needed,
            "the relationship table of the capability lists the change"
        );
        Self {
            project,
            redesign,
            before,
            template: created.template,
            path,
        }
    }

    /// The record with its claims, with or without their outcomes.
    fn write_record(&self, settled: bool) {
        let redesign = self.redesign;
        self.project.put(
            &self.path,
            &author_record(
                &self.project.text(&self.path),
                redesign.title,
                redesign.description,
                redesign.summary,
                &items(redesign, settled),
            ),
        );
    }

    /// The author writes the record and the drafts of the proposal, and moves the
    /// capability to `implementing` in the same commit, as the guidance says.
    /// The PROPOSED text states the amendment, as an amended decision record
    /// does; the current account beside it is untouched. Returns the commits.
    fn propose(&self) -> Vec<String> {
        let slug = self.redesign.slug;
        self.write_record(false);
        let mut revisions = Vec::new();
        for (number, draft) in self.redesign.drafts.iter().enumerate() {
            self.project.put(
                streaming::DOCUMENT,
                &streaming_document(
                    self.before,
                    "implementing",
                    blocks_of(&self.template, draft),
                ),
            );
            self.project.git(&["add", "-A"]);
            self.project.sync();
            let message = if number == 0 {
                format!("docs: propose {slug}")
            } else {
                format!("docs: amend the proposal of {slug}")
            };
            revisions.push(self.project.commit(&message));
        }
        let (other, assessments) = split_assessments(owed(self.project, slug));
        assert_eq!(other, expected_while_proposed(self.redesign));
        assert_eq!(
            assessments,
            assessments_while_proposed(self.redesign, self.before == 0)
        );
        revisions
    }

    /// Build the code, tests, examples and current executable usage together.
    /// The preceding contract and rationale still stand beside the proposal.
    fn build(&self) -> String {
        let slug = self.redesign.slug;
        let after = streaming::files(self.redesign.version);
        for &(path, text) in &after {
            self.project.put(path, text);
        }
        // Removed executable targets leave when their bindings do, before
        // the next clean commit; historical revisions retain their contents.
        for (path, _) in streaming::files(self.before) {
            if !after.iter().any(|(kept, _)| *kept == path) {
                self.project.remove(path);
            }
        }
        let proposed = self.redesign.drafts.last().expect("a draft");
        let bindings = streaming::bindings(self.redesign.version);
        let current = streaming::sections(self.redesign.version);
        let mut previous = streaming::sections(self.before);
        for heading in ["usage", "examples"] {
            let at = index_of(heading);
            previous[at] = current[at];
        }
        // This is still the unfinished preceding account beside the proposal.
        // A removed test cannot substantiate that account against new code.
        // Drop only those obsolete markers in this implementing build state;
        // settled and historical delivered versions retain their exact owners.
        let sections = previous.map(|section| {
            section
                .lines()
                .filter(|line| {
                    let Some(paths) = line
                        .trim()
                        .strip_prefix("<!-- raf:tested-by ")
                        .and_then(|rest| rest.strip_suffix(" -->"))
                    else {
                        return true;
                    };
                    paths.split(',').all(|path| {
                        let path = path.trim();
                        bindings.tests.contains(&path) || bindings.examples.contains(&path)
                    })
                })
                .collect::<Vec<_>>()
                .join("\n")
        });
        self.project.put(
            streaming::DOCUMENT,
            &capability(&Capability {
                slug: streaming::SLUG,
                title: streaming::TITLE,
                description: streaming::DESCRIPTION,
                state: "implementing",
                bindings,
                sections: sections.each_ref().map(String::as_str),
                blocks: blocks_of(&self.template, proposed),
            }),
        );
        self.project.sync();
        let revision = self.project.commit(&format!("feat: build {slug}"));
        let (other, assessments) = split_assessments(owed(self.project, slug));
        let expected: Vec<_> = expected_while_proposed(self.redesign)
            .into_iter()
            .filter(|line| {
                !matches!(
                    line.as_str(),
                    "section-untouched usage" | "section-untouched examples"
                )
            })
            .collect();
        assert_eq!(other, expected);
        // The build moved the bound code, so every affected section owes an
        // assessment of the account against it.
        let owing: BTreeSet<String> = assessments
            .iter()
            .filter_map(|line| line.split_once(' ').map(|(_, section)| section.to_string()))
            .collect();
        let affected: BTreeSet<String> = self
            .redesign
            .affects
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(owing, affected, "{assessments:?}");
        revision
    }

    /// Write the account: the blocks give way to the contract text, every item
    /// gets its outcome, and reconcile has nothing left. The bound tests and
    /// examples pass.
    fn settle(&self) -> String {
        let slug = self.redesign.slug;
        let version = self.redesign.version;
        self.write_record(true);
        self.project.put(
            streaming::DOCUMENT,
            &streaming_document(version, "implementing", Vec::new()),
        );
        self.project.sync();
        // The author has the account assessed against the code it describes.
        let assessed = assess_on_disk(self.project.root(), slug);
        assert_eq!(assessed.len(), self.redesign.affects.len(), "{assessed:?}");
        let revision = self.project.commit(&format!("docs: settle {slug}"));
        let reconciled = owed(self.project, slug);
        assert!(reconciled.is_empty(), "{reconciled:?}");
        let verified = self
            .project
            .lifecycle()
            .verify(Some(slug))
            .unwrap_or_else(|error| panic!("verify answers: {error}"));
        let bound = streaming::bindings(version);
        assert_eq!(verified.len(), bound.tests.len() + bound.examples.len());
        assert!(verified.passed(), "{verified}");
        revision
    }

    /// The author sets the state the rules require, syncs, and `complete`
    /// removes the record.
    fn deliver(&self) -> String {
        let project = self.project;
        let slug = self.redesign.slug;
        project.edit(
            streaming::DOCUMENT,
            "state: implementing",
            "state: delivered",
        );
        // The catalogue lists the capability under its state, so the new state
        // stales it, and the gate says so until the author syncs.
        let unsynced: Vec<&str> = project.findings().iter().map(|f| f.kind).collect();
        assert_eq!(unsynced, ["state-delivered-open-record", "generated-stale"]);
        assert_refusal(
            &format!("{slug} before the sync"),
            &refused(project, slug),
            &[
                "obligations:generated-stale document",
                "generated-blocks:generated-stale",
            ],
        );
        project.sync();
        // A delivered capability beside its open record is not a tree that can be
        // committed, which is what `complete` resolves.
        let beside: Vec<&str> = project.findings().iter().map(|f| f.kind).collect();
        assert_eq!(beside, ["state-delivered-open-record"]);
        assert!(owed(project, slug).is_empty(), "nothing is owed");
        completed(project, slug, streaming::DOCUMENT);
        project.commit(&format!("docs: complete {slug}"))
    }
}

/// One redesign, through every step of the lifecycle, each step committed.
fn redesign(project: &Project, redesign: &Redesign, before: usize) -> (Commits, Landed) {
    let step = Step::open(project, redesign, before);
    let mut revisions = step.propose();
    revisions.push(step.build());
    revisions.push(step.settle());
    revisions.push(step.deliver());
    project.land(redesign.slug);
    (
        Commits {
            slug: redesign.slug,
            revisions,
        },
        Landed::of(project, redesign.version),
    )
}

/// Words that tie a statement to an earlier one: a document that needs them
/// asks its reader to reconstruct a history.
const CHAIN: [&str; 13] = [
    "supersed",
    "amended",
    "amends",
    "previous",
    "formerly",
    "originally",
    "no longer",
    "used to",
    "replaced by",
    "earlier",
    "reversed",
    " was ",
    "renamed from",
];

fn chain_words(text: &str) -> Vec<&'static str> {
    // Padded, so that a word that stands alone (` was `) is not found inside
    // another.
    let lower = format!(" {} ", flat(text).to_lowercase());
    CHAIN
        .iter()
        .copied()
        .filter(|word| lower.contains(word))
        .collect()
}

/// Scenario A: "the document states the active contract directly".
fn assert_contract_stated_directly(project: &Project, document: &str, commits: &[Commits]) {
    let contract = flat(&section_text(document, "Contract"));
    for sentence in [
        "`Parser::feed` takes the next chunk and returns the entries whose lines it completed",
        "`Parser::close` ends the input",
        "A line is at most 65536 bytes long without its line ending",
        "`Parser::with_max_line` sets another limit",
        "`Reason::Malformed` or `Reason::TooLong`",
        "The first error ends the parse",
        "A blank line is not an entry and is not an error.",
    ] {
        assert!(
            contract.contains(sentence),
            "the Contract lacks `{sentence}`"
        );
    }
    // It states it without asking the reader to apply anything: no reference to
    // a change, no word that ties a statement to a previous one. The same scan
    // finds the amendments while a change is open.
    assert_eq!(chain_words(document), Vec::<&str>::new(), "{document}");
    assert_eq!(chain_words("It was a chunk."), [" was "]);
    assert_eq!(chain_words("The washed rows."), Vec::<&str>::new());
    assert_eq!(
        chain_words("`feed`, renamed from `push`."),
        ["renamed from"]
    );
    for commit in commits {
        assert!(
            !document.contains(commit.slug),
            "the document names the change `{}`",
            commit.slug
        );
        let proposed = project.show(&commit.revisions[0], streaming::DOCUMENT);
        assert!(
            chain_words(&proposed).contains(&"amended"),
            "the scan finds the amendment of `{}` while it is open",
            commit.slug
        );
    }
    assert!(
        !document.contains("git "),
        "the document sends no reader to git"
    );
}

/// The path a link of a document in `docs/capabilities` names, from the root.
fn link_target(link: &str) -> Option<String> {
    let target = link.split('#').next().expect("a target");
    if target.is_empty() {
        return None;
    }
    let joined = format!("docs/capabilities/{target}");
    let mut parts: Vec<&str> = Vec::new();
    for part in joined.split('/') {
        match part {
            ".." => {
                parts.pop();
            }
            "." => {}
            _ => parts.push(part),
        }
    }
    Some(parts.join("/"))
}

/// Scenario A: "every link resolves".
fn assert_links_resolve(project: &Project, document: &str) {
    assert_eq!(project.findings(), []);
    Gate::workspace(project.root())
        .programs(project.programs())
        .assert_clean();
    let tracked = tracked_files(project);
    let links = links_of(document);
    assert!(
        links.len() >= 9,
        "the document links to the architecture page, the examples and its own sections: {links:?}"
    );
    for link in &links {
        if let Some(target) = link_target(link) {
            assert!(tracked.contains(&target), "`{link}` resolves to {target}");
        }
    }
    let opened = Gate::workspace(project.root())
        .programs(project.programs())
        .open()
        .expect("opens");
    assert_eq!(
        raf_workplan::plain::check("README.md", &project.text("README.md"), &opened.links()),
        []
    );
}

/// The files git tracks.
fn tracked_files(project: &Project) -> BTreeSet<String> {
    project
        .git(&["ls-files"])
        .lines()
        .map(str::to_string)
        .collect()
}

/// Scenario A: "no PROPOSED block or change file is on the reading path".
fn assert_reading_path_is_clear(project: &Project) {
    let opened = Gate::workspace(project.root())
        .programs(project.programs())
        .open()
        .expect("opens");
    let plan = opened.workplan();
    assert_eq!(plan.graph().proposals(), []);
    assert!(plan.changes().is_empty(), "no change is loaded");
    assert!(plan.open_changes().is_empty(), "no change is open");
    let tracked = tracked_files(project);
    for path in tracked.iter().filter(|path| is_markdown(path)) {
        let text = project.text(path);
        assert!(
            !text.contains("[!PROPOSED"),
            "{path} holds a PROPOSED block"
        );
        assert!(!text.contains("work/changes/"), "{path} links a record");
    }
    assert!(
        tracked
            .iter()
            .all(|path| !path.starts_with("work/changes/")),
        "no change file is tracked"
    );
    let roadmap = plan.status();
    let entry = roadmap.get(streaming::SLUG).expect("the capability");
    assert_eq!(entry.state(), raf_okf::State::Delivered);
    assert!(entry.gaps().is_empty() && entry.open_changes().is_empty());
    let catalogue = flat(&project.text("docs/capabilities/index.md"));
    assert!(catalogue.contains("## Delivered"), "{catalogue}");
    assert!(catalogue.contains("[streaming-parser](./streaming-parser.md)"));
}

/// Scenario A: "the Explanation section still holds the rationale".
fn assert_rationale_kept(document: &str) {
    let explanation = flat(&section_text(document, "Explanation"));
    for rationale in [
        "Decision 1, input model",
        "sockets and pipes deliver text in pieces",
        "Decision 2, error policy",
        "Skipping hid a truncated feed",
        "Decision 3, output model",
        "a caller that reads a stream with no end cannot wait for the end",
        "Decision 4, line limit",
        "a stream that never sends a line ending would otherwise fill the memory",
    ] {
        assert!(
            explanation.contains(rationale),
            "the Explanation lacks `{rationale}`"
        );
    }
}

/// Scenario A: "every earlier state is recoverable with `git show`".
fn assert_earlier_states_recoverable(project: &Project, earlier: &[Landed]) {
    for state in earlier {
        let version = state.version;
        let shown = project.show(&state.revision, streaming::DOCUMENT);
        assert_eq!(shown, state.document, "version {version}");
        assert_eq!(
            project.show(&state.revision, "src/lib.rs"),
            state.library,
            "version {version}"
        );
        assert_eq!(
            project.show(&state.revision, "tests/parser.rs"),
            state.tests,
            "version {version}"
        );
        // It is a state of its own: delivered, with no record open.
        assert!(shown.contains("state: delivered"));
        assert!(
            project
                .tracked_at(&state.revision)
                .iter()
                .all(|path| !path.starts_with("work/changes/")),
            "version {version} has no change file"
        );
        // It is a valid repository: checked out, it passes the gate.
        let scratch = Scratch::new("scenario-a-old");
        let tree = scratch.path().join("tree");
        project.checkout(&state.revision, &tree);
        assert_eq!(
            Gate::workspace(&tree)
                .programs(Programs::new(&tree).discover_git())
                .check(),
            [],
            "version {version} is not clean"
        );
        std::fs::remove_dir_all(&tree).expect("the checkout is removed");
    }
}

/// The wording that applied at a version is in that state and, once it stops
/// applying, not in the later ones: the reader of the present is not asked to
/// subtract it.
fn assert_superseded_wording_gone(landed: &[Landed], document: &str) {
    let last = landed.len() - 1;
    for wording in &WORDING {
        let needle = flat(wording.text);
        let last_holder = wording.until.map_or(last, |until| until - 1);
        for state in landed {
            let holds = flat(&state.document).contains(&needle);
            let expected = state.version >= wording.from && state.version <= last_holder;
            assert_eq!(
                holds, expected,
                "`{}` at version {} (from {}, until {:?})",
                wording.text, state.version, wording.from, wording.until
            );
        }
        let now = flat(document).contains(&needle);
        assert_eq!(now, wording.until.is_none(), "`{}` now", wording.text);
    }
}

/// The changes themselves are recoverable: each record is in the history of its
/// path, deleted, and its commits show the proposal and the outcomes.
fn assert_changes_recoverable(project: &Project, commits: &[Commits]) {
    for commit in commits {
        let path = record_path(commit.slug);
        let record = project.show(&commit.revisions[0], &path);
        assert!(record.contains("## Summary") && record.contains("Amended 2026-"));
        let settled = project.show(&commit.revisions[commit.revisions.len() - 2], &path);
        assert!(settled.contains("outcome: implemented"), "{settled}");
        let deleted = project.git(&[
            "log",
            "--diff-filter=D",
            "--name-only",
            "--format=",
            "--",
            &path,
        ]);
        assert_eq!(deleted.trim(), path);
    }
    // The amended decision of the last redesign: the first draft and the
    // amendment are both in the history.
    let last = commits.last().expect("a redesign");
    let first_draft = project.show(&last.revisions[0], streaming::DOCUMENT);
    let amended = project.show(&last.revisions[1], streaming::DOCUMENT);
    assert!(first_draft.contains("4096 bytes") && !first_draft.contains("65536"));
    assert!(amended.contains("not 4096") && amended.contains("65536"));
}

#[test]
fn several_redesigns_leave_one_readable_present() {
    let (project, first) = genesis();
    let mut landed = vec![first];
    let mut commits = Vec::new();
    for step in &REDESIGNS {
        let before = landed.last().expect("a state").version;
        let (made, state) = redesign(&project, step, before);
        commits.push(made);
        landed.push(state);
    }
    assert_eq!(landed.len(), 5);
    let document = project.text(streaming::DOCUMENT);
    assert_eq!(landed[4].document, document);

    // Scenario A: "At the end the fixture asserts: the document states the
    // active contract directly; every link resolves; no PROPOSED block or change
    // file is on the reading path; the Explanation section still holds the
    // rationale; and every earlier state is recoverable with `git show`."
    assert_contract_stated_directly(&project, &document, &commits);
    assert_links_resolve(&project, &document);
    assert_reading_path_is_clear(&project);
    assert_rationale_kept(&document);
    assert_earlier_states_recoverable(&project, &landed[..4]);

    // Scenario A: "The test is whether a reader needs to reconstruct the changes
    // to understand what applies now." What applied at each earlier version is
    // in that version and nowhere in this one.
    assert_superseded_wording_gone(&landed, &document);
    assert_changes_recoverable(&project, &commits);

    // Scenario A: "Decisions are amended and reversed, and interfaces and
    // constraints evolve." The code, the tests and the prose are five different
    // texts, and so are the bindings.
    let distinct =
        |select: fn(&Landed) -> &String| landed.iter().map(select).collect::<BTreeSet<_>>().len();
    assert_eq!(distinct(|state| &state.document), 5);
    assert_eq!(distinct(|state| &state.library), 5);
    assert_eq!(distinct(|state| &state.tests), 5);
    let bindings: BTreeSet<String> = (0..5)
        .map(|version| format!("{:?}", streaming::bindings(version).code))
        .collect();
    assert_eq!(bindings.len(), 5);
}

// --- scenario B ------------------------------------------------------------------

/// The capability document of the second scenario.
fn window_document(
    state: &str,
    bindings: Bindings,
    sections: [&str; 7],
    blocks: Vec<(usize, String)>,
) -> String {
    capability(&Capability {
        slug: window::SLUG,
        title: window::TITLE,
        description: window::DESCRIPTION,
        state,
        bindings,
        sections,
        blocks,
    })
}

/// The candidate's PROPOSED blocks, for the sections named.
fn candidate_blocks(template: &str, only: &[&str]) -> Vec<(usize, String)> {
    window::CANDIDATE
        .iter()
        .filter(|(section, _)| only.contains(section))
        .map(|(section, text)| (index_of(section), proposal(template, text)))
        .collect()
}

/// The sections that carry a block while the capability is a candidate.
const CANDIDATE_SECTIONS: [&str; 4] = ["contract", "usage", "examples", "constraints"];

/// The record of the change, with these outcomes for its three claims.
fn window_record(project: &Project, outcomes: [Option<&str>; 3]) {
    let items: Vec<(&str, Option<&str>, Option<&str>)> = window::CLAIMS
        .iter()
        .copied()
        .zip(outcomes)
        .map(|(claim, outcome)| {
            let reason = (claim == window::CLAIMS[1] && outcome == Some("dropped"))
                .then_some("The values use a VecDeque that grows while the window fills, so recording may allocate");
            (claim, outcome, reason)
        })
        .collect();
    project.put(
        window::RECORD,
        &author_record(
            &project.text(window::RECORD),
            window::RECORD_TITLE,
            window::RECORD_DESCRIPTION,
            window::RECORD_SUMMARY,
            &items,
        ),
    );
}

/// The repository at the start of the scenario: the capability `considered`,
/// with its open questions and its candidate contract in PROPOSED blocks, the
/// change that lists it, and a crate with nothing in it. The work is on the
/// branch `window`, forked from the default branch.
fn candidate() -> (Project, String) {
    let files = vec![
        ("src/lib.rs".to_string(), window::EMPTY_LIB.to_string()),
        (
            window::DOCUMENT.to_string(),
            window_document(
                "considered",
                window::UNBOUND,
                window::CONSIDERED,
                Vec::new(),
            ),
        ),
    ];
    let project = Project::create("scenario-b", window::PACKAGE, &files);
    let mut request = NewChangeRequest::new(window::CHANGE)
        .capability(window::ID)
        .copyright(COPYRIGHT)
        .licence(LICENCE);
    for section in window::AFFECTS {
        request = request.affects(format!("{}#{section}", window::ID));
    }
    let root = project.root();
    let created = on_disk(root, |plan| plan.new_change(root, &request))
        .unwrap_or_else(|error| panic!("`change new` should succeed: {error}"));
    // A considered capability needs no change of state to hold a block.
    assert_eq!(created.guidance[0].advice, StateAdvice::Keep);
    window_record(&project, [None, None, None]);
    project.put(
        window::DOCUMENT,
        &window_document(
            "considered",
            window::UNBOUND,
            window::CONSIDERED,
            candidate_blocks(&created.template, &CANDIDATE_SECTIONS),
        ),
    );
    project.git(&["add", "-A"]);
    project.sync();
    project.commit("docs: the candidate rolling window");
    project.publish();
    project.fork("window");
    (project, created.template)
}

/// Build the window, its tests and example as an uncommitted transient state.
/// The considered, unbound account remains available for the refusal checks.
fn implement(project: &Project) {
    for (path, text) in window::built() {
        project.put(path, text);
        // Link checking uses Git's real tracked-path inventory. Register each
        // new executable without committing the still-invalid considered state.
        project.git(&["add", "--", path]);
    }
}

/// What the author writes at a stage: the state, the bindings, the sections,
/// the candidate blocks that stay and the outcomes of the claims.
struct Edit<'a> {
    state: &'a str,
    bindings: Bindings,
    sections: [&'a str; 7],
    kept_blocks: &'a [&'a str],
    outcomes: [Option<&'a str>; 3],
}

/// The author's edit, then a sync.
fn author(project: &Project, template: &str, edit: &Edit<'_>) {
    window_record(project, edit.outcomes);
    project.put(
        window::DOCUMENT,
        &window_document(
            edit.state,
            edit.bindings,
            edit.sections,
            candidate_blocks(template, edit.kept_blocks),
        ),
    );
    project.sync();
}

/// The prose of a document: no fenced code, no generated block, no quoted
/// PROPOSED block, no code span.
fn prose(document: &str) -> String {
    let mut out = String::new();
    let mut fenced = false;
    let mut generated = false;
    for line in document.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        } else if line == "<!-- raf:generated:end -->" {
            generated = false;
        } else if line.starts_with("<!-- raf:generated:") {
            generated = true;
        } else if !fenced && !generated && !line.starts_with('>') {
            let mut in_code = false;
            for c in line.chars() {
                if c == '`' {
                    in_code = !in_code;
                    out.push(' ');
                } else if !in_code {
                    out.push(c);
                }
            }
            out.push('\n');
        }
    }
    out
}

/// The phrases of the tense list that the prose of a document uses, found
/// independently of the library: whole words, case ignored.
fn tense_phrases(document: &str) -> Vec<&'static str> {
    let text = prose(document).to_lowercase();
    let words: Vec<&str> = text
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|word| !word.is_empty())
        .collect();
    raf_workplan::obligation::tense::PHRASES
        .iter()
        .copied()
        .filter(|phrase| {
            let parts: Vec<&str> = phrase.split(' ').collect();
            words.windows(parts.len()).any(|window| window == parts)
        })
        .collect()
}

/// The code of the first `rust` fence of a section.
fn fenced_rust(section: &str) -> Option<String> {
    let start = section.find("```rust\n")? + "```rust\n".len();
    let end = section[start..].find("```")?;
    Some(section[start..start + end].to_string())
}

/// Build `body` as the body of `main` in an example of the project, run it, and
/// say whether it exited with status 0. The example is removed again.
fn runs_as_example(project: &Project, body: &str) -> bool {
    let path = "examples/usage_check.rs";
    project.put(path, &format!("fn main() {{\n{body}\n}}\n"));
    let output = std::process::Command::new(cargo())
        .args(["run", "--frozen", "--quiet", "--example", "usage_check"])
        .current_dir(project.root())
        .output()
        .expect("cargo runs");
    project.remove(path);
    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    output.status.success()
}

/// The account of the window in the working tree, read for what Scenario B
/// asks of it.
struct Account<'a> {
    project: &'a Project,
    document: String,
    library: String,
    opened: Opened,
}

impl<'a> Account<'a> {
    fn of(project: &'a Project) -> Self {
        Self {
            project,
            document: project.text(window::DOCUMENT),
            library: project.text("src/lib.rs"),
            opened: Gate::workspace(project.root())
                .programs(project.programs())
                .open()
                .expect("opens"),
        }
    }

    /// What is wrong with the account, if anything. An empty list is the end
    /// state Scenario B asks for; each entry names one of the things the
    /// scenario lists, or the proposal text it forbids. Every check reads the
    /// repository and not the author's intent: the source, the graph, the
    /// generated table and a run of the usage example.
    fn faults(&self) -> Vec<&'static str> {
        let mut faults = Vec::new();
        self.standing(&mut faults);
        self.explanation(&mut faults);
        self.references(&mut faults);
        self.limitations(&mut faults);
        self.usage(&mut faults);
        self.evidence(&mut faults);
        faults
    }

    /// The state, and no proposal text presented as established behaviour.
    fn standing(&self, faults: &mut Vec<&'static str>) {
        let plan = self.opened.workplan();
        let state = plan
            .status()
            .get(window::SLUG)
            .map(raf_workplan::RoadmapEntry::state);
        if state != Some(raf_okf::State::Delivered) {
            faults.push("state");
        }
        if self.document.contains("[!PROPOSED") || !plan.graph().proposals().is_empty() {
            faults.push("proposal-left");
        }
        if !tense_phrases(&self.document).is_empty() {
            faults.push("tense");
        }
        if self.document.contains("never allocates") || self.document.contains("Window::add") {
            faults.push("candidate-text");
        }
    }

    /// An accurate explanation: it gives the reasons, and what it says about
    /// the implementation is what the implementation is.
    fn explanation(&self, faults: &mut Vec<&'static str>) {
        let explanation = flat(&section_text(&self.document, "Explanation"));
        let reasons = [
            "bounded by a count because",
            "`VecDeque` that grows up to the capacity",
            "computed on demand",
        ];
        if !reasons.iter().all(|reason| explanation.contains(reason))
            || !self.library.contains("VecDeque::new()")
            || self.library.contains("with_capacity")
        {
            faults.push("explanation");
        }
    }

    /// Current API references: every `Window::name` the body of the document
    /// writes is a function of the library, there are several, and the renamed
    /// function is the one named.
    fn references(&self, faults: &mut Vec<&'static str>) {
        let body = self
            .document
            .find("\n---\n")
            .map_or(self.document.as_str(), |end| &self.document[end + 5..]);
        let mut references = BTreeSet::new();
        for part in body.split("Window::").skip(1) {
            let name: String = part
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            references.insert(name);
        }
        if references.len() < 4
            || !references
                .iter()
                .all(|name| self.library.contains(&format!("fn {name}(")))
            || !references.contains("record")
        {
            faults.push("api-reference");
        }
    }

    /// Explicit observable limitations and visible allocation reasoning.
    fn limitations(&self, faults: &mut Vec<&'static str>) {
        let constraints = section_text(&self.document, "Constraints");
        let bullets = constraints
            .lines()
            .filter(|line| line.starts_with("- "))
            .count();
        let explanation = section_text(&self.document, "Explanation");
        if bullets < 3
            || !flat(&constraints).contains("A zero-capacity window stores no value")
            || !flat(&constraints)
                .contains("A `NaN` value makes the mean `NaN` until it leaves the window")
            || !flat(&explanation).contains("Recording a value can allocate while the window fills")
        {
            faults.push("limitations");
        }
    }

    /// A real usage example: code that uses the API, and that builds and runs.
    fn usage(&self, faults: &mut Vec<&'static str>) {
        let usage = section_text(&self.document, "Usage");
        match fenced_rust(&usage) {
            Some(code)
                if code.contains("Window::new(")
                    && code.contains(".record(")
                    && code.contains(".mean()") =>
            {
                if !runs_as_example(self.project, &code) {
                    faults.push("usage-runs");
                }
            }
            _ => faults.push("usage-example"),
        }
    }

    /// A bound example that runs, and a traceability table that lists every
    /// bound test and example, and every test of the test file.
    fn evidence(&self, faults: &mut Vec<&'static str>) {
        let plan = self.opened.workplan();
        let bindings: Vec<_> = plan.graph().bindings_of(window::ID).collect();
        if !bindings.iter().any(|b| b.kind() == BindingKind::Example) {
            faults.push("example-binding");
        }
        match self.project.lifecycle().verify(None) {
            Ok(report) if report.passed() && !report.is_empty() => {}
            failed => {
                eprintln!("scenario bound verification failed: {failed:#?}");
                faults.push("bound-run");
            }
        }
        let matrix = generated_block(&self.document, "verification-matrix");
        let mut listed: Vec<String> = bindings
            .iter()
            .filter(|b| b.kind() != BindingKind::Code)
            .map(|b| b.path().to_string())
            .collect();
        let test_file = self.project.text("tests/window.rs");
        listed.extend(test_file.split("#[test]\nfn ").skip(1).map(|rest| {
            let name: String = rest.chars().take_while(|c| *c != '(').collect();
            format!("window_stats::test_crate::window::{name}")
        }));
        listed.sort();
        listed.dedup();
        let rows = matrix.iter().filter(|line| line.starts_with("| `")).count();
        let all_listed = listed.iter().all(|path| {
            matrix
                .iter()
                .any(|row| row.starts_with(&format!("| `{path}` |")))
        });
        if listed.is_empty() || !all_listed || rows != listed.len() {
            faults.push("matrix");
        }
    }
}

/// The failures of a section of the window with no assessment, for each of
/// the seven sections the change affects: an `assessment-missing` obligation
/// (condition 1) and the same fault under condition 6.
fn assessments_missing() -> Vec<String> {
    window::AFFECTS
        .iter()
        .flat_map(|section| {
            [
                format!("obligations:assessment-missing {section}"),
                format!("assessments:assessment-missing {section}"),
            ]
        })
        .collect()
}

/// `lines`, and the failures of the seven sections with no assessment.
fn and_assessments_missing(lines: &[&str]) -> Vec<String> {
    let mut all: Vec<String> = lines.iter().map(ToString::to_string).collect();
    all.extend(assessments_missing());
    all
}

/// Exact complete-policy failures shared by the two three-word accounts.
/// Their absent or unresolved example binding is asserted by each caller.
fn thin_account_failures() -> Vec<String> {
    let mut expected: Vec<String> = [
        "after-removal:coverage-lines-over-ceiling",
        "after-removal:strict-claim-uncovered",
        "after-removal:strict-claim-uncovered",
        "after-removal:strict-limitations-missing",
        "after-removal:strict-usage-unbound",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    expected.extend(
        HEADINGS
            .iter()
            .map(|_| "after-removal:strict-section-thin".to_owned()),
    );
    expected
}

/// Building the code alone leaves the entire documentation account unfinished.
fn assert_implementation_needs_its_account(project: &Project) {
    // Nothing of the documentation has moved.
    let completion = refused(project, window::CHANGE);
    assert_refusal(
        "implemented only",
        &completion,
        &and_assessments_missing(&[
            "after-removal:no-open-record",
            "after-removal:coverage-lines-over-ceiling",
            "after-removal:coverage-lines-over-ceiling",
            "obligations:item-without-outcome item 1",
            "obligations:item-without-outcome item 2",
            "obligations:item-without-outcome item 3",
            "obligations:section-untouched constraints",
            "obligations:section-untouched contract",
            "obligations:section-untouched examples",
            "obligations:section-untouched explanation",
            "obligations:section-untouched purpose",
            "obligations:section-untouched usage",
            "obligations:section-untouched verification",
            "obligations:unresolved-proposal constraints",
            "obligations:unresolved-proposal contract",
            "obligations:unresolved-proposal examples",
            "obligations:unresolved-proposal usage",
            "outcomes:item-without-outcome item 1",
            "outcomes:item-without-outcome item 2",
            "outcomes:item-without-outcome item 3",
            "proposals:unresolved-proposal constraints",
            "proposals:unresolved-proposal contract",
            "proposals:unresolved-proposal examples",
            "proposals:unresolved-proposal usage",
        ]),
    );
}

/// Scenario B: "`change complete` must refuse until Usage, Examples and the
/// bindings match the new API and the dropped claim has the outcome `dropped`."
/// No section is assessed on the way, so each refusal also lists the seven
/// sections without an assessment. Returns the commit of the account once it
/// is settled.
fn refuse_until_reconciled(project: &Project, template: &str) -> String {
    assert_implementation_needs_its_account(project);

    // The author settles the contract, the constraints, the explanation and the
    // verification text from what was built, sets the state the rules require
    // and copies the bindings from the candidate. Usage and Examples still hold
    // the candidate's blocks, and the dropped claim has no outcome.
    let mut edit = Edit {
        state: "delivered",
        bindings: window::CANDIDATE_BINDINGS,
        sections: window::partly_settled(),
        kept_blocks: &["usage", "examples"],
        outcomes: [Some("implemented"), None, Some("implemented")],
    };
    author(project, template, &edit);
    assert_refusal(
        "contract settled, usage and examples not",
        &refused(project, window::CHANGE),
        &and_assessments_missing(&[
            "after-removal:binding-unresolved",
            "after-removal:binding-unresolved",
            "after-removal:state-delivered-proposal",
            "after-removal:strict-section-thin",
            "after-removal:strict-section-thin",
            "after-removal:strict-usage-unbound",
            "obligations:binding-missing verification",
            "obligations:binding-missing verification",
            "obligations:item-without-outcome item 2",
            "obligations:section-untouched examples",
            "obligations:section-untouched usage",
            "obligations:tense-mismatch examples",
            "obligations:tense-mismatch usage",
            "obligations:unresolved-proposal examples",
            "obligations:unresolved-proposal usage",
            "outcomes:item-without-outcome item 2",
            "proposals:unresolved-proposal examples",
            "proposals:unresolved-proposal usage",
        ]),
    );

    // Usage and Examples are written for the new API, but the bindings still
    // name the candidate's `add`.
    edit.sections = window::settled();
    edit.kept_blocks = &[];
    author(project, template, &edit);
    assert_refusal(
        "bindings of the old API",
        &refused(project, window::CHANGE),
        &and_assessments_missing(&[
            "after-removal:binding-unresolved",
            "after-removal:binding-unresolved",
            "obligations:binding-missing verification",
            "obligations:binding-missing verification",
            "obligations:item-without-outcome item 2",
            "outcomes:item-without-outcome item 2",
        ]),
    );

    // The bindings match the new API; the dropped claim has no outcome yet.
    edit.bindings = window::CURRENT_BINDINGS;
    author(project, template, &edit);
    assert_refusal(
        "one claim without outcome",
        &refused(project, window::CHANGE),
        &and_assessments_missing(&[
            "obligations:item-without-outcome item 2",
            "outcomes:item-without-outcome item 2",
        ]),
    );

    // The claim is dropped. The documentation is the account of the built
    // window, but the state is the author's to set and `complete` does not set
    // it: left at `implementing`, the capability would have no open record.
    edit.state = "implementing";
    edit.outcomes = [Some("implemented"), Some("dropped"), Some("implemented")];
    author(project, template, &edit);
    assert_refusal(
        "state implementing",
        &refused(project, window::CHANGE),
        &and_assessments_missing(&["after-removal:no-open-record"]),
    );
    let settled = project.commit("docs: settle the rolling window");
    // The account is settled, and what it still owes is its assessments.
    let expected: Vec<String> = window::AFFECTS
        .iter()
        .map(|section| format!("assessment-missing {section}"))
        .collect();
    assert_eq!(owed(project, window::CHANGE), expected);
    settled
}

/// Scenario B: "It then succeeds and leaves the capability `delivered` with an
/// accurate capability explanation, a real usage example, current API
/// references, explicit limitations and a test traceability table, and no
/// proposal text presented as established behaviour."
fn assert_delivered_account(project: &Project, settled: &str, finished: &str) {
    assert_eq!(Account::of(project).faults(), Vec::<&str>::new());
    assert_eq!(project.findings(), []);
    let document = project.text(window::DOCUMENT);
    assert!(document.contains("state: delivered"));
    // The explanation, the example, the references, the limitations, the table.
    let explanation = flat(&section_text(&document, "Explanation"));
    assert!(explanation.contains("callers choose it at run time"));
    let usage = section_text(&document, "Usage");
    let snippet = fenced_rust(&usage).expect("a usage example");
    assert!(snippet.contains("window.record(1.0);") && snippet.contains("Window::new(3)"));
    assert!(document.contains("`Window::record(value)` stores a value"));
    let matrix = generated_block(&document, "verification-matrix");
    assert_eq!(matrix[0], "| Item | Kind | Section |");
    for test in window::CURRENT_BINDINGS.tests {
        assert!(
            matrix
                .iter()
                .any(|row| row.starts_with(&format!("| `{test}` | test |"))),
            "{test} in {matrix:?}"
        );
    }
    for row in [
        "| `window_stats::example_crate::rolling` | example target |",
        "| `window_stats::Window::new` | doctest |",
    ] {
        assert!(matrix.iter().any(|line| line.starts_with(row)), "{row}");
    }
    // The dropped claim is stated as a limitation and not as a guarantee. The
    // candidate's wording and the outcome are in git.
    let explanation = flat(&section_text(&document, "Explanation"));
    assert!(explanation.contains("Recording a value can allocate while the window fills"));
    assert!(!flat(&document).contains("never allocates"));
    assert!(
        project
            .show("main", window::DOCUMENT)
            .contains("`add` never allocates")
    );
    assert!(
        project
            .show(settled, window::RECORD)
            .contains("outcome: dropped")
    );
    assert!(
        project
            .show(&format!("{finished}~1"), window::RECORD)
            .contains("outcome: dropped")
    );
    assert!(
        project
            .tracked_at(finished)
            .iter()
            .all(|path| !path.starts_with("work/changes/"))
    );
}

#[test]
fn planning_produces_the_documentation() {
    let (project, template) = candidate();
    let before = project.text(window::DOCUMENT);
    // Scenario B: "The fixture starts with a capability in `considered` with
    // open questions."
    assert!(before.contains("state: considered"));
    assert_eq!(
        before
            .matches("[!PROPOSED change:first-window-api]")
            .count(),
        4
    );
    assert!(before.contains("Two questions are open."));
    assert_eq!(project.findings(), []);

    // Scenario B: "The implementation renames an API and drops one guarantee."
    // The candidate wrote `add` and promised that it never allocates; the window
    // written is `record` over a `VecDeque` that grows.
    implement(&project);
    let library = project.text("src/lib.rs");
    assert!(library.contains("pub fn record(") && !library.contains("pub fn add("));
    assert!(library.contains("VecDeque::new()") && !library.contains("with_capacity"));
    // The detector of the end state finds the start wanting, in every respect
    // that the start is: the state, the blocks, the candidate's wording.
    let faults = Account::of(&project).faults();
    for fault in [
        "state",
        "proposal-left",
        "candidate-text",
        "usage-example",
        "example-binding",
    ] {
        assert!(faults.contains(&fault), "{fault} in {faults:?}");
    }

    let settled = refuse_until_reconciled(&project, &template);

    // The author sets `delivered`. Now `complete` succeeds, and the only changes
    // to the tree are the capability, the catalogue that lists its state and the
    // record.
    project.edit(window::DOCUMENT, "state: implementing", "state: delivered");
    project.sync();
    // Implemented and settled, but no section is assessed: condition 6
    // refuses, and only it.
    assert_refusal(
        "settled and not assessed",
        &refused(&project, window::CHANGE),
        &assessments_missing(),
    );
    // The assessor reads each section and accepts it; the records are
    // committed on their own.
    let assessed = assess_on_disk(project.root(), window::CHANGE);
    assert_eq!(assessed.len(), window::AFFECTS.len(), "{assessed:?}");
    project.git(&["add", "--", "work/assessments"]);
    project.git(&["commit", "-q", "-m", "docs: assess the rolling window"]);
    assert!(owed(&project, window::CHANGE).is_empty(), "nothing is owed");
    completed(&project, window::CHANGE, window::DOCUMENT);
    assert_eq!(
        project.git(&["status", "--porcelain"]),
        format!(
            " M docs/capabilities/index.md\n M {}\n D {}",
            window::DOCUMENT,
            window::RECORD
        )
    );
    let finished = project.commit("docs: complete the first window API");
    assert_delivered_account(&project, &settled, &finished);
}

#[test]
fn a_document_that_merely_exists_does_not_satisfy_the_scenario() {
    let (project, template) = candidate();
    implement(&project);
    let all_settled = [Some("implemented"), Some("dropped"), Some("implemented")];

    // Scenario B: "A document that merely exists does not satisfy the
    // scenario." A document whose sections are all there and all empty does not
    // load, so `complete` cannot judge it, errors, and writes nothing.
    window_record(&project, all_settled);
    project.put(
        window::DOCUMENT,
        &window_document("delivered", window::CURRENT_BINDINGS, [""; 7], Vec::new()),
    );
    let before = project.snapshot();
    let error = project
        .lifecycle()
        .complete(window::CHANGE)
        .expect_err("a document with empty sections cannot be judged");
    let message = error.to_string();
    assert!(message.contains("is not loaded"), "{message}");
    assert_eq!(
        message.matches("[okf-section-empty]").count(),
        7,
        "{message}"
    );
    assert_eq!(project.snapshot(), before);
    assert!(project.exists(window::RECORD));

    // A document with a sentence in each section, and an example that is not
    // there: the example binding names the candidate's API, and the gate refuses
    // it as an unresolved binding.
    project.put(
        window::DOCUMENT,
        &window_document(
            "delivered",
            Bindings {
                examples: &["window_stats::Window::add"],
                ..window::CURRENT_BINDINGS
            },
            window::THIN,
            Vec::new(),
        ),
    );
    project.sync();
    assess_on_disk(project.root(), window::CHANGE);
    let mut expected = thin_account_failures();
    expected.extend([
        "after-removal:binding-unresolved".to_owned(),
        "obligations:binding-missing verification".to_owned(),
    ]);
    assert_refusal(
        "an example that is not there",
        &refused(&project, window::CHANGE),
        &expected,
    );

    // An example that is there and fails is refused under conditions 1 and 5,
    // and the record stays. The neighbour is the account the first scenario
    // builds, which completes.
    project.put("examples/broken.rs", window::FAILING_EXAMPLE);
    project.put(
        window::DOCUMENT,
        &window_document(
            "delivered",
            Bindings {
                examples: &[
                    "window_stats::example_crate::rolling",
                    "window_stats::Window::new",
                    "window_stats::example_crate::broken",
                ],
                ..window::CURRENT_BINDINGS
            },
            window::settled(),
            Vec::new(),
        ),
    );
    project.sync();
    assess_on_disk(project.root(), window::CHANGE);
    assert_refusal(
        "an example that fails",
        &refused(&project, window::CHANGE),
        &[
            "bound-runs:bound-example",
            "obligations:example-failing examples",
        ],
    );
    // Repair the actual failing executable and restore the complete account.
    // The same lifecycle now succeeds; neither refusal weakened its gate.
    project.remove("examples/broken.rs");
    author(
        &project,
        &template,
        &Edit {
            state: "delivered",
            bindings: window::CURRENT_BINDINGS,
            sections: window::settled(),
            kept_blocks: &[],
            outcomes: all_settled,
        },
    );
    assess_on_disk(project.root(), window::CHANGE);
    assert_eq!(Account::of(&project).faults(), Vec::<&str>::new());
    completed(&project, window::CHANGE, window::DOCUMENT);
}

#[test]
fn a_thin_document_is_not_the_account_the_scenario_builds() {
    let (project, template) = candidate();
    implement(&project);
    let all_settled = [Some("implemented"), Some("dropped"), Some("implemented")];
    window_record(&project, all_settled);

    // A document with a sentence in each section and no example binding is not an
    // explanation, a usage example, a set of API references or limitations, and
    // the detector of the end state says so for each.
    project.put(
        window::DOCUMENT,
        &window_document(
            "delivered",
            Bindings {
                examples: &[],
                ..window::CURRENT_BINDINGS
            },
            window::THIN,
            Vec::new(),
        ),
    );
    project.sync();
    let faults = Account::of(&project).faults();
    for fault in [
        "explanation",
        "usage-example",
        "api-reference",
        "limitations",
        "example-binding",
    ] {
        assert!(faults.contains(&fault), "{fault} in {faults:?}");
    }
    assert!(!faults.contains(&"state") && !faults.contains(&"proposal-left"));

    // The complete policy refuses this account even if its assessments accept
    // it. The refusal keeps the record and every byte of the built workspace.
    assess_on_disk(project.root(), window::CHANGE);
    let mut expected = thin_account_failures();
    expected.push("after-removal:strict-example-missing".to_owned());
    assert_refusal(
        "thin account without an example",
        &refused(&project, window::CHANGE),
        &expected,
    );
    assert!(project.exists(window::RECORD));

    // The neighbour: the account the scenario builds has none of these faults,
    // and `complete` accepts it.
    author(
        &project,
        &template,
        &Edit {
            state: "delivered",
            bindings: window::CURRENT_BINDINGS,
            sections: window::settled(),
            kept_blocks: &[],
            outcomes: all_settled,
        },
    );
    assess_on_disk(project.root(), window::CHANGE);
    assert_eq!(Account::of(&project).faults(), Vec::<&str>::new());
    completed(&project, window::CHANGE, window::DOCUMENT);
}

/// Replace the only `from` in `text`.
fn replaced(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "`{from}` once in the text");
    text.replacen(from, to, 1)
}

/// The finished account of the window, completed and committed, without the
/// refusals on the way.
fn finished_window() -> Project {
    let (project, template) = candidate();
    implement(&project);
    let edit = Edit {
        state: "implementing",
        bindings: window::CURRENT_BINDINGS,
        sections: window::settled(),
        kept_blocks: &[],
        outcomes: [Some("implemented"), Some("dropped"), Some("implemented")],
    };
    author(&project, &template, &edit);
    assess_on_disk(project.root(), window::CHANGE);
    project.commit("docs: settle the rolling window");
    project.edit(window::DOCUMENT, "state: implementing", "state: delivered");
    project.sync();
    completed(&project, window::CHANGE, window::DOCUMENT);
    project.commit("docs: complete the first window API");
    project
}

/// One way to break the finished account, and the faults of the detector that
/// it must produce and no others.
struct Break {
    faults: &'static [&'static str],
    path: &'static str,
    change: fn(&str) -> String,
}

/// The ways to break the prose of the finished account, one fault each.
fn text_breaks() -> Vec<Break> {
    vec![
        Break {
            faults: &["state"],
            path: window::DOCUMENT,
            change: |text| replaced(text, "state: delivered", "state: implementing"),
        },
        Break {
            faults: &["proposal-left"],
            path: window::DOCUMENT,
            change: |text| format!("{text}\n> [!PROPOSED change:first-window-api]\n> A block.\n"),
        },
        Break {
            faults: &["tense"],
            path: window::DOCUMENT,
            change: |text| {
                replaced(
                    text,
                    "- Capacity counts stored values: recording into a full window discards its oldest",
                    "- Capacity will count stored values: recording into a full window discards its oldest",
                )
            },
        },
        Break {
            faults: &["candidate-text"],
            path: window::DOCUMENT,
            change: |text| {
                replaced(
                    text,
                    "A `NaN` value is stored like any other.",
                    "A `NaN` value is stored; it never allocates.",
                )
            },
        },
        Break {
            faults: &["explanation"],
            path: window::DOCUMENT,
            change: |text| replaced(text, "computed on demand", "computed lazily"),
        },
        Break {
            faults: &["api-reference"],
            path: window::DOCUMENT,
            change: |text| replaced(text, "`Window::len()` and", "`Window::count()` and"),
        },
        Break {
            faults: &["limitations"],
            path: window::DOCUMENT,
            change: |text| {
                replaced(
                    text,
                    "Recording a value can allocate while the window fills",
                    "Recording a value is cheap",
                )
            },
        },
        Break {
            faults: &["usage-example"],
            path: window::DOCUMENT,
            change: |text| replaced(text, "```rust\nlet mut window", "```text\nlet mut window"),
        },
        Break {
            faults: &["usage-runs"],
            path: window::DOCUMENT,
            change: |text| {
                replaced(
                    text,
                    "let mut window = window_stats::Window::new(3);",
                    "let mut window = window_stats::Window::new(\"3\");",
                )
            },
        },
    ]
}

/// The ways to break the finished account that touch its bindings, its table
/// and its tests.
fn binding_breaks() -> Vec<Break> {
    vec![
        Break {
            faults: &["example-binding", "matrix"],
            path: window::DOCUMENT,
            change: |text| {
                replaced(
                    text,
                    "  examples:\n    - window_stats::example_crate::rolling\n    - window_stats::Window::new\n",
                    "  examples: []\n",
                )
            },
        },
        Break {
            faults: &["bound-run"],
            path: "tests/window.rs",
            change: |text| {
                replaced(
                    text,
                    "assert_eq!(window.len(), 3);",
                    "assert_eq!(window.len(), 4);",
                )
            },
        },
        Break {
            faults: &["matrix"],
            path: window::DOCUMENT,
            change: |text| {
                text.lines()
                    .filter(|line| {
                        !line.contains("a_window_of_capacity_zero_keeps_nothing` | test |")
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
                    + "\n"
            },
        },
    ]
}

#[test]
fn each_fault_of_the_account_has_a_document_that_breaks_only_it() {
    let project = finished_window();
    assert_eq!(Account::of(&project).faults(), Vec::<&str>::new());
    let mut breaks = text_breaks();
    breaks.extend(binding_breaks());
    for broken in &breaks {
        project.put(broken.path, &(broken.change)(&project.text(broken.path)));
        assert_eq!(
            Account::of(&project).faults(),
            broken.faults,
            "breaking {}",
            broken.faults.join(", ")
        );
        project.git(&["checkout", "-q", "--", broken.path]);
    }
    assert_eq!(Account::of(&project).faults(), Vec::<&str>::new());
}
