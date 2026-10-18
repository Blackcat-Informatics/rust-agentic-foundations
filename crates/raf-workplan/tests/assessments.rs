// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Prose assessment over repositories on a disk: the record and its inputs,
//! what makes it stale and what does not, the context an assessor reads, the
//! replacement of a record, the obligations of `reconcile`, the refusals with
//! their neighbours, and hostile sizes.

mod support;

use std::fmt::Write as _;
use std::path::Path;

use raf_okf::{Actor, AssessmentVerdict, InputKind};
use raf_workplan::assessment::{
    AssessError, AssessmentRequest, AssessmentState, Currency, InputChange,
};
use raf_workplan::record::RecordOptions;
use raf_workplan::{Finding, SyncOptions, Workplan};
use support::changes::{DONE, capability, record};
use support::disk::{Scratch, changed, on_disk, put, read, snapshot};
use support::evidence::{Evidence, claim};

const CAP: &str = "docs/capabilities/omega.md";
const RECORD: &str = "work/changes/shape.md";
const CONTRACT: &str = "cap:omega#contract";
const ASSESSMENT: &str = "work/assessments/omega-contract.md";

const LIB: &str = "\
/// Runs.
pub fn run() {}

/// Does something else.
pub fn other() {}
";

/// A repository with the crate `demo`, the capability `omega` bound to
/// `code` with `contract` added to its Contract, and the change `shape` that
/// affects that Contract; synced.
struct Fixture {
    scratch: Scratch,
}

impl Fixture {
    fn new(code: &str, contract: &str) -> Self {
        Self::with_lib(LIB, code, contract)
    }

    fn with_lib(lib: &str, code: &str, contract: &str) -> Self {
        Self::with_files(&[("src/lib.rs", lib)], code, contract)
    }

    /// The fixture with the source files `files` (`src/lib.rs` among them).
    fn with_files(files: &[(&str, &str)], code: &str, contract: &str) -> Self {
        let scratch = Scratch::new("assessments");
        let root = scratch.path();
        put(
            root,
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        for (path, text) in files {
            put(root, path, text);
        }
        put(root, "tests/it.rs", "#[test]\nfn case() {}\n");
        put(
            root,
            CAP,
            &capability(
                "omega",
                "implementing",
                [code, "[]", "[]"],
                &[("Contract", contract)],
            ),
        );
        put(
            root,
            RECORD,
            &record("shape", "[cap:omega]", &[CONTRACT], "changes", DONE),
        );
        put(root, support::COVERAGE, &support::open_coverage());
        let fixture = Self { scratch };
        fixture.sync();
        fixture
    }

    fn root(&self) -> &Path {
        self.scratch.path()
    }

    fn sync(&self) {
        let report = on_disk(self.root(), |plan| {
            plan.sync(&SyncOptions::new(
                "2026 Example Org <dev@example.org>",
                "CC-BY-4.0",
            ))
        })
        .unwrap_or_else(|error| panic!("the fixture syncs: {error}"));
        Workplan::write(&report, self.root()).expect("the sync is written");
    }

    fn edit(&self, path: &str, from: &str, to: &str) {
        let text = read(self.root(), path);
        assert!(text.contains(from), "{path} has no `{from}`");
        put(self.root(), path, &text.replacen(from, to, 1));
    }

    /// Record `verdict` by `by` on `subject`.
    fn assess(
        &self,
        subject: &str,
        verdict: AssessmentVerdict,
        by: Actor,
    ) -> Result<raf_workplan::assessment::Assessed, AssessError> {
        let request = AssessmentRequest::new(subject, verdict, by);
        on_disk(self.root(), |plan| {
            plan.record_assessment(self.root(), &request)
        })
    }

    fn accept(&self, subject: &str) {
        self.assess(subject, AssessmentVerdict::Accepted, Actor::Agent)
            .unwrap_or_else(|error| panic!("{subject} is assessed: {error}"));
    }

    /// The currency of the one assessment of `subject`.
    fn currency(&self, subject: &str) -> Currency {
        let states = on_disk(self.root(), listed);
        let found: Vec<_> = states.iter().filter(|s| s.subject == subject).collect();
        assert_eq!(found.len(), 1, "{states:?}");
        found[0].currency.clone()
    }

    /// The assessment lines of `reconcile shape`.
    fn owed(&self) -> Vec<String> {
        on_disk(self.root(), |plan| {
            let list = plan.reconcile("shape").expect("reconciles");
            list.obligations()
                .iter()
                .filter(|o| o.kind.name().starts_with("assessment-"))
                .map(ToString::to_string)
                .collect()
        })
    }
}

/// The assessments of a plan and their currency.
fn listed(plan: &Workplan<'_>) -> Vec<AssessmentState> {
    plan.list_assessments()
}

/// The findings of a plan.
fn findings(plan: &Workplan<'_>) -> Vec<Finding> {
    plan.check()
}

/// The inputs that moved, as kind, name and how.
fn stale(currency: &Currency) -> Vec<(InputKind, String, InputChange)> {
    match currency {
        Currency::Current => Vec::new(),
        Currency::Stale(moved) => moved
            .iter()
            .map(|input| (input.kind, input.name.clone(), input.change.clone()))
            .collect(),
        other => panic!("an unknown currency {other:?}"),
    }
}

fn changed_input(kind: InputKind, name: &str) -> (InputKind, String, InputChange) {
    (kind, name.to_string(), InputChange::Changed)
}

/// The digest of `path` in the crate of `root`.
fn digest_of(root: &Path, path: &str) -> String {
    on_disk(root, |plan| {
        support::evidence::digest_of(plan.source(), path)
    })
}

// --- the record and its currency ---------------------------------------------

#[test]
fn an_unchanged_section_is_current_and_a_second_record_is_byte_identical_and_replaces_the_first() {
    let fixture = Fixture::new("[demo::run]", "");
    let before = snapshot(fixture.root());
    let first = fixture
        .assess(CONTRACT, AssessmentVerdict::Accepted, Actor::Agent)
        .expect("recorded");
    assert!(!first.replaced);
    assert_eq!(first.path.as_str(), ASSESSMENT);
    assert_eq!(
        first.to_string(),
        format!("recorded: {ASSESSMENT} (accepted by agent)")
    );
    // The record is the one file written, and it loads clean.
    assert_eq!(changed(&before, &snapshot(fixture.root())), [ASSESSMENT]);
    assert_eq!(on_disk(fixture.root(), findings), []);
    let text = read(fixture.root(), ASSESSMENT);
    assert!(text.contains("\nid: assessment:omega-contract\n"), "{text}");
    assert!(text.contains("\nsubject: cap:omega#contract\n"), "{text}");
    assert!(text.contains("\nby: agent\n"), "{text}");
    assert!(text.contains("    name: cap:omega#contract\n"), "{text}");
    assert!(text.contains("    name: demo::run\n"), "{text}");
    assert_eq!(fixture.currency(CONTRACT), Currency::Current);
    assert!(fixture.owed().is_empty(), "{:?}", fixture.owed());

    // The same inputs give the same digests, so a second record is the same
    // bytes; it replaces the first and nothing of the first is kept.
    let again = fixture
        .assess(CONTRACT, AssessmentVerdict::Accepted, Actor::Agent)
        .expect("recorded again");
    assert!(again.replaced);
    assert_eq!(read(fixture.root(), ASSESSMENT), text);
    let rejected = fixture
        .assess(CONTRACT, AssessmentVerdict::Rejected, Actor::Human)
        .expect("replaced");
    assert!(rejected.replaced);
    assert_eq!(
        rejected.to_string(),
        format!("replaced: {ASSESSMENT} (rejected by human)")
    );
    let replaced = read(fixture.root(), ASSESSMENT);
    assert_eq!(
        replaced,
        text.replace("verdict: accepted", "verdict: rejected")
            .replace("by: agent", "by: human")
    );
    assert_eq!(changed(&before, &snapshot(fixture.root())), [ASSESSMENT]);
}

#[test]
fn an_edit_of_the_section_makes_it_stale_naming_the_section_and_an_edit_of_another_does_not() {
    let fixture = Fixture::new("[demo::run]", "");
    fixture.accept(CONTRACT);
    fixture.edit(
        CAP,
        "Text for Usage of omega.",
        "Text for Usage of omega, revised.",
    );
    assert_eq!(fixture.currency(CONTRACT), Currency::Current);
    fixture.edit(
        CAP,
        "Text for Contract of omega.",
        "Text for Contract of omega, revised.",
    );
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [changed_input(InputKind::Section, CONTRACT)]
    );
    assert_eq!(
        fixture.owed(),
        [
            "cap:omega  contract  assessment-stale  the section assessed again with assess (`assessment:omega-contract`: the section `cap:omega#contract` has changed since)"
        ]
    );
    // Assessing it again makes it current.
    fixture.accept(CONTRACT);
    assert_eq!(fixture.owed(), Vec::<String>::new());
}

#[test]
fn a_symbol_the_section_names_is_an_input_and_a_bound_symbol_it_does_not_name_is_not() {
    let fixture = Fixture::new(
        "[demo::run, demo::other]",
        "\nThe contract is kept by `demo::run()`.\n",
    );
    fixture.accept(CONTRACT);
    let text = read(fixture.root(), ASSESSMENT);
    assert!(
        text.contains("name: demo::run\n") && !text.contains("demo::other"),
        "{text}"
    );
    // An edit of the bound symbol the section does not name leaves it current.
    fixture.edit(
        "src/lib.rs",
        "pub fn other() {}",
        "pub fn other() { let _ = 1; }",
    );
    assert_eq!(fixture.currency(CONTRACT), Currency::Current);
    // An edit of the named one moves it, and the obligation names the symbol.
    fixture.edit(
        "src/lib.rs",
        "pub fn run() {}",
        "pub fn run() { let _ = 2; }",
    );
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [changed_input(InputKind::Symbol, "demo::run")]
    );
    assert_eq!(
        fixture.owed(),
        [
            "cap:omega  contract  assessment-stale  the section assessed again with assess (`assessment:omega-contract`: the symbol `demo::run` has changed since)"
        ]
    );
}

#[test]
fn an_edit_inside_a_bound_module_moves_the_assessment_and_a_sibling_edit_does_not() {
    let fixture = Fixture::with_files(
        &[
            ("src/lib.rs", "pub mod tools;\npub mod other;\n"),
            ("src/tools.rs", "pub mod deep;\npub fn t() {}\n"),
            ("src/tools/deep.rs", "pub fn d() {}\n"),
            ("src/other.rs", "pub fn o() {}\n"),
        ],
        "[demo::tools]",
        "",
    );
    fixture.accept(CONTRACT);
    // A sibling module edited: the assessment stays current.
    fixture.edit("src/other.rs", "pub fn o() {}", "pub fn o() { let _ = 1; }");
    assert_eq!(fixture.currency(CONTRACT), Currency::Current);
    // A function of a child module of the bound one edited: it moves, naming
    // the module.
    fixture.edit(
        "src/tools/deep.rs",
        "pub fn d() {}",
        "pub fn d() { let _ = 2; }",
    );
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [changed_input(InputKind::Symbol, "demo::tools")]
    );
    // Assessed again, a function in the module's own file moves it too.
    fixture.accept(CONTRACT);
    fixture.edit("src/tools.rs", "pub fn t() {}", "pub fn t() { let _ = 3; }");
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [changed_input(InputKind::Symbol, "demo::tools")]
    );
}

/// The crate of the naming fixtures: functions the sections name and one
/// they do not, one named like a common word, a type with two methods, and two
/// functions of one name in two modules.
const NAMING_LIB: &str = "\
pub fn run() {}
pub fn other() {}
pub fn spare() {}
pub fn complete() {}
pub struct Window;
impl Window {
    pub fn record(&self) {}
    pub fn len(&self) -> usize {
        0
    }
}
pub mod left {
    pub mod side {
        pub fn go() {}
    }
}
pub mod right {
    pub mod side {
        pub fn go() {}
    }
}
";

/// The symbol inputs of the record of the Contract.
fn symbol_inputs(fixture: &Fixture) -> Vec<String> {
    let text = read(fixture.root(), ASSESSMENT);
    text.split("- kind: symbol\n    name: ")
        .skip(1)
        .map(|rest| rest.lines().next().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn a_bound_symbol_named_by_its_last_segment_is_an_input_beside_one_named_in_full() {
    // A section that names one bound symbol by its whole path, which is a
    // strong name, and another by its last segment, a weak one: both are
    // inputs, and the symbol it does not name is not.
    let fixture = Fixture::with_lib(
        NAMING_LIB,
        "[demo::run, demo::other, demo::spare]",
        "\n`demo::run` starts the work and `other` finishes it.\n",
    );
    fixture.accept(CONTRACT);
    assert_eq!(symbol_inputs(&fixture), ["demo::other", "demo::run"]);
    // An edit of the symbol the section does not name leaves it current.
    fixture.edit(
        "src/lib.rs",
        "pub fn spare() {}",
        "pub fn spare() { let _ = 0; }",
    );
    assert_eq!(fixture.currency(CONTRACT), Currency::Current);
    // An edit of the one it names by its last segment makes it stale.
    fixture.edit(
        "src/lib.rs",
        "pub fn other() {}",
        "pub fn other() { panic!(\"finished\") }",
    );
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [changed_input(InputKind::Symbol, "demo::other")]
    );
}

#[test]
fn a_common_word_that_is_the_last_segment_of_a_binding_does_not_narrow_the_inputs() {
    // `complete` and `demo` in a command line name nothing strongly, so every
    // bound symbol stays an input, and an edit of `run` makes it stale.
    let fixture = Fixture::with_lib(
        NAMING_LIB,
        "[demo::run, demo::complete]",
        "\nType `cargo demo complete`; the runner then passes `--exact` to every test.\n",
    );
    fixture.accept(CONTRACT);
    assert_eq!(symbol_inputs(&fixture), ["demo::complete", "demo::run"]);
    fixture.edit(
        "src/lib.rs",
        "pub fn run() {}",
        "pub fn run() { let _ = 3; }",
    );
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [changed_input(InputKind::Symbol, "demo::run")]
    );
}

#[test]
fn a_method_named_by_its_type_and_name_is_an_input_and_its_sibling_is_not() {
    // `Window::record` is a strong name of two segments: the inputs narrow to
    // what the section names.
    let fixture = Fixture::with_lib(
        NAMING_LIB,
        "[demo::Window::record, demo::Window::len]",
        "\n`Window::record()` stores a value.\n",
    );
    fixture.accept(CONTRACT);
    assert_eq!(symbol_inputs(&fixture), ["demo::Window::record"]);
    fixture.edit("src/lib.rs", "        0\n", "        1\n");
    assert_eq!(fixture.currency(CONTRACT), Currency::Current);
    fixture.edit(
        "src/lib.rs",
        "pub fn record(&self) {}",
        "pub fn record(&self) { let _ = self; }",
    );
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [changed_input(InputKind::Symbol, "demo::Window::record")]
    );
}

#[test]
fn a_suffix_two_bindings_end_in_names_both() {
    let code = "[demo::left::side::go, demo::right::side::go, demo::spare]";
    // `side::go` is a strong name that both bindings end in: both are inputs,
    // and the binding the section does not name is not.
    let fixture = Fixture::with_lib(NAMING_LIB, code, "\nEach side has its own `side::go`.\n");
    fixture.accept(CONTRACT);
    assert_eq!(
        symbol_inputs(&fixture),
        ["demo::left::side::go", "demo::right::side::go"]
    );
    fixture.edit(
        "src/lib.rs",
        "pub mod right {\n    pub mod side {\n        pub fn go() {}",
        "pub mod right {\n    pub mod side {\n        pub fn go() { let _ = 1; }",
    );
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [changed_input(InputKind::Symbol, "demo::right::side::go")]
    );
    // `go` alone is a weak name: it narrows nothing, and every binding stays
    // an input.
    let weak = Fixture::with_lib(NAMING_LIB, code, "\nEach side has its own `go`.\n");
    weak.accept(CONTRACT);
    assert_eq!(
        symbol_inputs(&weak),
        [
            "demo::left::side::go",
            "demo::right::side::go",
            "demo::spare"
        ]
    );
}

#[test]
fn crlf_line_endings_in_the_document_and_the_source_leave_an_assessment_current() {
    let fixture = Fixture::new(
        "[demo::run]",
        &format!("\nIt is bounded.\n{}", claim("bound")),
    );
    let run = digest_of(fixture.root(), "demo::run");
    let evidence =
        Evidence::passing("bound", "[cap:omega]").inputs(vec![("demo::run".to_string(), run)]);
    put(fixture.root(), &evidence.path(), &evidence.text());
    fixture.accept(CONTRACT);
    let crlf = |path: &str| {
        let text = read(fixture.root(), path).replace("\r\n", "\n");
        put(fixture.root(), path, &text.replace('\n', "\r\n"));
    };
    for path in [CAP, "src/lib.rs", &evidence.path()] {
        crlf(path);
    }
    assert!(read(fixture.root(), CAP).contains("\r\n"));
    assert_eq!(fixture.currency(CONTRACT), Currency::Current);
    // The neighbour: an edit under CRLF still moves it.
    fixture.edit(
        CAP,
        "Text for Contract of omega.\r\n",
        "Text for Contract of omega, revised.\r\n",
    );
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [changed_input(InputKind::Section, CONTRACT)]
    );
}

#[test]
fn with_no_bound_symbol_named_every_bound_symbol_is_an_input() {
    let fixture = Fixture::new("[demo::run, demo::other]", "");
    fixture.accept(CONTRACT);
    let text = read(fixture.root(), ASSESSMENT);
    assert!(text.contains("name: demo::run\n") && text.contains("name: demo::other\n"));
    fixture.edit(
        "src/lib.rs",
        "/// Does something else.",
        "/// Does another thing.",
    );
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [changed_input(InputKind::Symbol, "demo::other")]
    );
    // A new binding is an input it was not made against, and a binding gone
    // is one it no longer has.
    fixture.accept(CONTRACT);
    fixture.edit(CAP, "code: [demo::run, demo::other]", "code: [demo::run]");
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [(
            InputKind::Symbol,
            "demo::other".to_string(),
            InputChange::NoLongerInput
        )]
    );
    fixture.accept(CONTRACT);
    fixture.edit(CAP, "code: [demo::run]", "code: [demo::run, demo::other]");
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [(
            InputKind::Symbol,
            "demo::other".to_string(),
            InputChange::NewInput
        )]
    );
}

#[test]
fn an_evidence_record_a_claim_of_the_section_names_is_an_input() {
    let fixture = Fixture::new(
        "[demo::run]",
        &format!("\nIt is bounded.\n{}", claim("bound")),
    );
    let run = digest_of(fixture.root(), "demo::run");
    let evidence = Evidence::passing("bound", "[cap:omega]")
        .inputs(vec![("demo::run".to_string(), run)])
        .values(&[("peak", "41")])
        .limits(&[("peak", "64")]);
    put(fixture.root(), &evidence.path(), &evidence.text());
    fixture.accept(CONTRACT);
    let text = read(fixture.root(), ASSESSMENT);
    assert!(
        text.contains("kind: record\n    name: evidence:bound\n"),
        "{text}"
    );
    // A record recorded again with another value moves the assessment.
    let again = evidence.values(&[("peak", "42")]);
    put(fixture.root(), &again.path(), &again.text());
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [changed_input(InputKind::Record, "evidence:bound")]
    );
    assert_eq!(
        fixture.owed(),
        [
            "cap:omega  contract  assessment-stale  the section assessed again with assess (`assessment:omega-contract`: the evidence record `evidence:bound` has changed since)"
        ]
    );
    // A record that no longer loads is an input that does not resolve.
    std::fs::remove_file(fixture.root().join(again.path())).expect("removed");
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [(
            InputKind::Record,
            "evidence:bound".to_string(),
            InputChange::Unresolved("it is not a loaded evidence record".to_string())
        )]
    );
}

#[test]
fn a_regenerated_block_leaves_the_assessment_of_its_section_current() {
    let fixture = Fixture::new("[demo::run]", "");
    let verification = "cap:omega#verification";
    fixture.accept(verification);
    // A test binding added: the matrix of the section is rewritten by a sync.
    fixture.edit(CAP, "tests: []", "tests: [demo::test_crate::it::case]");
    let before = read(fixture.root(), CAP);
    fixture.sync();
    assert_ne!(
        read(fixture.root(), CAP),
        before,
        "the sync rewrote the matrix"
    );
    assert_eq!(fixture.currency(verification), Currency::Current);
}

#[test]
fn the_context_is_the_same_for_a_human_and_an_agent_and_holds_what_the_assessor_reads() {
    let fixture = Fixture::new(
        "[demo::run, demo::other]",
        &format!("\nThe contract is kept by `demo::run`.\n{}", claim("bound")),
    );
    let run = digest_of(fixture.root(), "demo::run");
    let evidence = Evidence::passing("bound", "[cap:omega]")
        .inputs(vec![("demo::run".to_string(), run)])
        .values(&[("peak", "41")])
        .limits(&[("peak", "64")]);
    put(fixture.root(), &evidence.path(), &evidence.text());
    let context = || {
        on_disk(fixture.root(), |plan| plan.assess_context(CONTRACT))
            .expect("a context")
            .to_string()
    };
    let first = context();
    for line in [
        "assess cap:omega#contract\n",
        &format!("record assessment:omega-contract ({ASSESSMENT})\n"),
        &format!("section {CAP}\n"),
        "    Contract\n",
        "    The contract is kept by `demo::run`.\n",
        "symbol demo::run (named in the section)\n",
        "  in src/lib.rs\n",
        "    /// Runs.\n    pub fn run() {}\n",
        "evidence evidence:bound pass (work/evidence/bound.md)\n",
        "  value peak=41\n",
        "  limit peak=64\n",
        "inputs\n  section cap:omega#contract ",
        "  symbol demo::run ",
        "  record evidence:bound ",
    ] {
        assert!(first.contains(line), "`{line}` in:\n{first}");
    }
    assert!(!first.contains("demo::other"), "{first}");
    let human = fixture
        .assess(CONTRACT, AssessmentVerdict::Accepted, Actor::Human)
        .expect("recorded");
    let after_human = context();
    let human_text = read(fixture.root(), ASSESSMENT);
    let agent = fixture
        .assess(CONTRACT, AssessmentVerdict::Accepted, Actor::Agent)
        .expect("recorded");
    let after_agent = context();
    assert_eq!(after_human, first);
    assert_eq!(after_agent, first);
    // The two records say who assessed and nothing else differs.
    assert_eq!(human.record.inputs, agent.record.inputs);
    assert_eq!(
        read(fixture.root(), ASSESSMENT),
        human_text.replace("\nby: human\n", "\nby: agent\n")
    );
}

#[test]
fn a_rejected_assessment_is_current_and_owes_an_acceptance_whoever_gave_it() {
    let fixture = Fixture::new("[demo::run]", "");
    for by in Actor::ALL {
        fixture
            .assess(CONTRACT, AssessmentVerdict::Rejected, by)
            .expect("recorded");
        assert_eq!(fixture.currency(CONTRACT), Currency::Current);
        assert_eq!(
            fixture.owed(),
            [format!(
                "cap:omega  contract  assessment-rejected  the section revised and assessed again with assess, with the verdict accepted (`assessment:omega-contract` is current and has the verdict rejected, by {})",
                by.name()
            )]
        );
    }
    fixture.accept(CONTRACT);
    assert_eq!(fixture.owed(), Vec::<String>::new());
    let listed = on_disk(fixture.root(), listed);
    assert_eq!(
        listed.iter().map(ToString::to_string).collect::<Vec<_>>(),
        ["cap:omega#contract  assessment:omega-contract  accepted  agent  current"]
    );
}

#[test]
fn a_record_named_otherwise_than_its_subject_is_a_finding_and_not_the_assessment_of_it() {
    let fixture = Fixture::new("[demo::run]", "");
    fixture.accept(CONTRACT);
    let text = read(fixture.root(), ASSESSMENT)
        .replace("id: assessment:omega-contract", "id: assessment:review");
    std::fs::remove_file(fixture.root().join(ASSESSMENT)).expect("removed");
    put(fixture.root(), "work/assessments/review.md", &text);
    let kinds: Vec<&str> = on_disk(fixture.root(), findings)
        .iter()
        .map(|f| f.kind)
        .collect();
    assert_eq!(kinds, ["assessment-id-mismatch"]);
    assert_eq!(
        fixture.owed(),
        [
            "cap:omega  contract  assessment-missing  an assessment of the section recorded with assess (`cap:omega#contract` has no assessment `assessment:omega-contract`)"
        ]
    );
}

// --- refusals and their neighbours ---------------------------------------------

#[test]
fn each_refusal_writes_nothing_and_its_neighbour_records() {
    let fixture = Fixture::new("[demo::run]", "");
    let before = snapshot(fixture.root());
    let refused = |subject: &str| {
        fixture
            .assess(subject, AssessmentVerdict::Accepted, Actor::Agent)
            .expect_err(subject)
    };
    assert!(matches!(
        refused("omega#contract"),
        AssessError::BadSubject { .. }
    ));
    assert!(matches!(
        refused("cap:omega#intro"),
        AssessError::NoSuchSection { .. }
    ));
    assert!(matches!(
        refused("cap:ghost#contract"),
        AssessError::UnknownCapability { .. }
    ));
    assert_eq!(snapshot(fixture.root()), before);
    // The neighbour.
    fixture.accept(CONTRACT);

    // A bound symbol that does not resolve, and a claim on a record that does
    // not load, are each refused; the neighbours above and below record.
    let unresolved = Fixture::new("[demo::missing]", "");
    let before = snapshot(unresolved.root());
    let error = unresolved
        .assess(CONTRACT, AssessmentVerdict::Accepted, Actor::Agent)
        .expect_err("refused");
    assert!(
        matches!(&error, AssessError::SymbolUnresolved { path, .. } if path == "demo::missing"),
        "{error:?}"
    );
    assert!(error.to_string().contains("nothing was written"), "{error}");
    assert_eq!(snapshot(unresolved.root()), before);

    let unsupported = Fixture::new(
        "[demo::run]",
        &format!("\nIt is bounded.\n{}", claim("ghost")),
    );
    let error = unsupported
        .assess(CONTRACT, AssessmentVerdict::Accepted, Actor::Agent)
        .expect_err("refused");
    assert!(
        matches!(&error, AssessError::EvidenceNotLoaded { id } if id == "evidence:ghost"),
        "{error:?}"
    );
    // Its context is still given, with the record named as not loaded.
    let context = on_disk(unsupported.root(), |plan| plan.assess_context(CONTRACT))
        .expect("a context")
        .to_string();
    assert!(
        context.contains("evidence evidence:ghost is not a loaded evidence record"),
        "{context}"
    );
    assert!(
        context.contains("record evidence:ghost does not resolve"),
        "{context}"
    );

    // SPDX lines given together are written; one alone is refused.
    let request = AssessmentRequest::new(CONTRACT, AssessmentVerdict::Accepted, Actor::Agent)
        .options(RecordOptions::new().copyright("2026 Example Org <dev@example.org>"));
    let error = on_disk(fixture.root(), |plan| {
        plan.record_assessment(fixture.root(), &request)
    })
    .expect_err("one SPDX field");
    assert!(matches!(error, AssessError::Record(_)), "{error:?}");
    let request = request.options(
        RecordOptions::new()
            .copyright("2026 Example Org <dev@example.org>")
            .licence("CC-BY-4.0"),
    );
    on_disk(fixture.root(), |plan| {
        plan.record_assessment(fixture.root(), &request)
    })
    .expect("both SPDX fields");
    assert!(read(fixture.root(), ASSESSMENT).starts_with("---\n# SPDX-"));
}

#[test]
fn a_capability_that_does_not_load_is_refused_and_a_section_of_generated_blocks_only_does_not_load()
{
    // The profile refuses a section with nothing but generated blocks, so a
    // capability that loads has authored text in every section, and the one
    // that does not is named with its findings.
    let fixture = Fixture::new("[demo::run]", "");
    fixture.edit(CAP, "Text for Usage of omega.\n", "<!-- a note -->\n");
    let error = on_disk(fixture.root(), |plan| {
        plan.assess_context("cap:omega#usage")
    })
    .expect_err("not loaded");
    assert!(
        matches!(&error, AssessError::CapabilityNotLoaded { .. }),
        "{error:?}"
    );
    assert!(error.to_string().contains("okf-section-empty"), "{error}");
    fixture.edit(CAP, "<!-- a note -->\n", "Text for Usage of omega.\n");
    on_disk(fixture.root(), |plan| {
        plan.assess_context("cap:omega#usage")
    })
    .expect("loads again");
}

// --- hostile sizes ---------------------------------------------------------------

#[test]
fn a_huge_section_and_ten_thousand_named_symbols_are_recorded_and_judged() {
    // A Contract near the document bound.
    let line = "The window keeps the values it was given, in order, and no more.\n";
    let huge = format!("\n{}", line.repeat(12_000));
    let fixture = Fixture::new("[demo::run]", &huge);
    fixture.accept(CONTRACT);
    assert_eq!(fixture.currency(CONTRACT), Currency::Current);
    assert_eq!(fixture.owed(), Vec::<String>::new());

    // Ten thousand bound symbols, each named in the section: the context and
    // the obligations are found in bounded time, and a record of ten thousand
    // inputs is over the document bound, so it is refused and nothing is
    // written.
    let named = |count: usize| {
        let lib = (0..count).fold(String::new(), |mut text, n| {
            let _ = writeln!(text, "pub fn f{n}() {{}}");
            text
        });
        let code = format!(
            "[{}]",
            (0..count)
                .map(|n| format!("demo::f{n}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let spans = (0..count).fold(String::new(), |mut text, n| {
            let _ = write!(text, "`demo::f{n}` ");
            text
        });
        Fixture::with_lib(&lib, &code, &format!("\n{spans}\n"))
    };
    let fixture = named(10_000);
    let context = on_disk(fixture.root(), |plan| plan.assess_context(CONTRACT)).expect("a context");
    assert_eq!(context.symbols.len(), 10_000);
    assert!(context.symbols.iter().all(|symbol| symbol.named));
    assert_eq!(fixture.owed().len(), 1);
    let before = snapshot(fixture.root());
    let error = fixture
        .assess(CONTRACT, AssessmentVerdict::Accepted, Actor::Agent)
        .expect_err("over the bound");
    assert!(matches!(error, AssessError::Record(_)), "{error:?}");
    assert_eq!(snapshot(fixture.root()), before);

    // Its neighbour, eight thousand, fits: it is recorded, current, and one
    // edit names the one symbol that moved.
    let count = 8_000;
    let fixture = named(count);
    let assessed = fixture
        .assess(CONTRACT, AssessmentVerdict::Accepted, Actor::Agent)
        .expect("recorded");
    assert_eq!(assessed.record.inputs.len(), count + 1);
    assert_eq!(fixture.currency(CONTRACT), Currency::Current);
    fixture.edit(
        "src/lib.rs",
        "pub fn f7999() {}",
        "pub fn f7999() { let _ = 1; }",
    );
    assert_eq!(
        stale(&fixture.currency(CONTRACT)),
        [changed_input(InputKind::Symbol, "demo::f7999")]
    );
}

/// A directory link inside the root at `work` or `work/assessments`, with the
/// record of the section behind it, and a file where `work/assessments`
/// belongs, are each refused by `record_assessment` with the tree
/// byte-identical; the neighbour with real directories records, replacing the
/// file of the section.
#[cfg(unix)]
#[test]
fn an_assessment_is_never_written_through_a_linked_directory_and_a_real_one_records() {
    let request = AssessmentRequest::new(CONTRACT, AssessmentVerdict::Accepted, Actor::Agent);
    for link in ["work", "work/assessments"] {
        let fixture = Fixture::new("[demo::run]", "");
        let root = fixture.root();
        put(root, ASSESSMENT, "mine\n");
        let error = on_disk(root, |plan| {
            // The plan is read before the link is made: the default package
            // discovery refuses a linked directory, and a library caller may
            // name other source roots.
            std::fs::rename(root.join(link), root.join("moved")).expect("moves");
            let up = "../".repeat(link.matches('/').count());
            std::os::unix::fs::symlink(format!("{up}moved"), root.join(link)).expect("links");
            let before = snapshot(root);
            let error = plan
                .record_assessment(root, &request)
                .expect_err("refused through the link");
            assert_eq!(snapshot(root), before, "{link}: nothing is written");
            error
        });
        assert!(
            matches!(
                &error,
                AssessError::Record(raf_workplan::record::RecordError::Write { path, .. })
                    if path == link
            ),
            "{link}: {error:?}"
        );
        assert_eq!(
            read(root, &format!("moved{}", &ASSESSMENT[link.len()..])),
            "mine\n"
        );
        // The neighbour: the directory is real again.
        std::fs::remove_file(root.join(link)).expect("unlinks");
        std::fs::rename(root.join("moved"), root.join(link)).expect("moves back");
        let assessed = fixture
            .assess(CONTRACT, AssessmentVerdict::Accepted, Actor::Agent)
            .expect("recorded");
        assert!(assessed.replaced, "{link}");
        assert_ne!(read(root, ASSESSMENT), "mine\n");
    }
    // A file where the directory belongs.
    let fixture = Fixture::new("[demo::run]", "");
    let root = fixture.root();
    put(root, "work/assessments", "a file\n");
    let before = snapshot(root);
    let error = fixture
        .assess(CONTRACT, AssessmentVerdict::Accepted, Actor::Agent)
        .expect_err("refused");
    assert!(
        matches!(
            &error,
            AssessError::Record(raf_workplan::record::RecordError::Write { path, .. })
                if path == "work/assessments"
        ),
        "{error:?}"
    );
    assert_eq!(snapshot(root), before);
}
