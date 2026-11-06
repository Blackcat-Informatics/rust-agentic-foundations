// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `assess` through the built binary: the context, the same for a human and
//! an agent, that writes nothing; a verdict that writes exactly one record and
//! exits 0, rejected or accepted; the list, current until an input moves and
//! then stale naming it; `-q`; the JSON objects; and the exit 2 of a section
//! that cannot be assessed.

mod support;

use raf_workplan::Gate;
use support::{COPYRIGHT, LICENCE, Repo, changed, rdf::Rdf};

const SUBJECT: &str = "cap:alpha#contract";
const RECORD: &str = "work/assessments/alpha-contract.md";

/// The context of the section as the library writes it.
fn library_context(repo: &Repo) -> String {
    let opened = Gate::workspace(repo.root()).open().expect("a repository");
    opened
        .workplan()
        .assess_context(SUBJECT)
        .expect("a context")
        .to_string()
}

#[test]
fn the_context_is_the_same_whoever_asks_and_writes_nothing() {
    let repo = Repo::synced("assess-context", false);
    let before = repo.snapshot();
    let plain = repo.both(&["assess", SUBJECT, "--context"]).expect(0);
    let human = repo
        .both(&["assess", SUBJECT, "--context", "--by", "human"])
        .expect(0);
    let agent = repo
        .both(&["assess", "--by", "agent", "--context", SUBJECT])
        .expect(0);
    assert_eq!(human, agent);
    assert_eq!(plain, human);
    assert_eq!(plain.stdout, library_context(&repo));
    assert!(
        plain.stdout.starts_with(&format!(
            "assess {SUBJECT}\nrecord assessment:alpha-contract ({RECORD})\n"
        )),
        "{plain:#?}"
    );
    assert!(plain.stdout.contains("symbol demo::real"), "{plain:#?}");
    assert_eq!(plain.stderr, "");
    // The typed RDF result, and `-q`, which hides it since it is no finding.
    let json = repo
        .both(&["--format", "turtle", "assess", SUBJECT, "--context"])
        .expect(0);
    let rdf = Rdf::of(&json);
    let context = rdf.rows("AssessmentContext")[0];
    assert_eq!(context.required("recordPath"), RECORD);
    assert_eq!(context.children("symbol")[0].required("path"), "demo::real");
    assert_eq!(context.children("input")[0].required("kind"), "section");
    assert_eq!(
        repo.both(&["-q", "assess", SUBJECT, "--context"])
            .expect(0)
            .stdout,
        ""
    );
    assert_eq!(repo.snapshot(), before, "the context wrote");
}

#[test]
fn a_verdict_writes_exactly_one_record_and_a_rejection_still_exits_zero() {
    let repo = Repo::synced("assess-verdict", false);
    let before = repo.snapshot();
    let run = repo
        .both_writing(&["assess", SUBJECT, "--verdict", "accepted", "--by", "agent"])
        .expect(0);
    assert_eq!(
        run.stdout,
        format!("recorded: {RECORD} (accepted by agent)\n")
    );
    assert_eq!(changed(&before, &repo.snapshot()), [RECORD]);
    let text = repo.read(RECORD);
    assert!(
        text.contains("verdict: accepted") && text.contains("by: agent"),
        "{text}"
    );

    // A rejection replaces the record of the section, with its SPDX lines,
    // and is still exit 0: the verdict was recorded.
    let accepted = repo.snapshot();
    let run = repo
        .both_writing(&[
            "--format",
            "turtle",
            "assess",
            SUBJECT,
            "--verdict",
            "rejected",
            "--by",
            "human",
            "--copyright",
            COPYRIGHT,
            "--licence",
            LICENCE,
        ])
        .expect(0);
    assert_eq!(changed(&accepted, &repo.snapshot()), [RECORD]);
    let text = repo.read(RECORD);
    assert!(
        text.starts_with(&format!(
            "---\n# SPDX-FileCopyrightText: {COPYRIGHT}\n# SPDX-License-Identifier: {LICENCE}\n"
        )),
        "{text}"
    );
    assert!(
        text.contains("verdict: rejected") && text.contains("by: human"),
        "{text}"
    );
    let rdf = Rdf::of(&run);
    let recorded = rdf.rows("AssessmentRecorded")[0];
    assert_eq!(recorded.required("path"), RECORD);
    assert_eq!(recorded.required("id"), "assessment:alpha-contract");
    assert_eq!(recorded.required("subject"), SUBJECT);
    assert_eq!(recorded.required("verdict"), "rejected");
    assert_eq!(recorded.required("by"), "human");
    assert!(recorded.boolean("replaced"));
    assert!(!recorded.children("input").is_empty());
    // Under `-q` a recorded verdict prints nothing and is still written.
    let rejected = repo.snapshot();
    let quiet = repo
        .both_writing(&[
            "-q",
            "assess",
            SUBJECT,
            "--verdict",
            "accepted",
            "--by",
            "agent",
        ])
        .expect(0);
    assert_eq!((quiet.stdout.as_str(), quiet.stderr.as_str()), ("", ""));
    assert_eq!(changed(&rejected, &repo.snapshot()), [RECORD]);
    let text = repo.read(RECORD);
    assert!(text.starts_with("---\ntype: Assessment\n"), "{text}");
    assert!(!text.contains("SPDX-"), "{text}");
    assert!(text.contains("verdict: accepted"));
}

#[test]
fn the_list_shows_each_assessment_current_until_an_input_moves() {
    let repo = Repo::synced("assess-list", false);
    let empty = repo.both(&["assess", "--list"]).expect(0);
    assert_eq!(empty.stdout, "");
    assert!(
        empty.stderr.contains("no assessment record loaded"),
        "{empty:#?}"
    );
    repo.both_writing(&["assess", SUBJECT, "--verdict", "accepted", "--by", "agent"])
        .expect(0);
    let before = repo.snapshot();
    let current = repo.both(&["assess", "--list"]).expect(0);
    assert_eq!(
        current.stdout,
        format!("{SUBJECT}  assessment:alpha-contract  accepted  agent  current\n")
    );
    assert_eq!(repo.both(&["assess", "--list", SUBJECT]), current);
    let other = repo.both(&["assess", "--list", "cap:beta#usage"]).expect(0);
    assert_eq!(other.stdout, "");
    assert!(
        other.stderr.contains("no assessment of cap:beta#usage"),
        "{other:#?}"
    );

    // An edit of the section makes it stale, naming the input that moved,
    // and the list still exits 0: `change reconcile` lists what is owed.
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Contract of alpha.",
        "Alpha states its contract.",
    );
    let edited = repo.snapshot();
    let stale = repo.both(&["assess", "--list"]).expect(0);
    assert_eq!(
        stale.stdout,
        format!(
            "{SUBJECT}  assessment:alpha-contract  accepted  agent  stale: the section `{SUBJECT}` has changed since\n"
        )
    );
    let json = repo
        .both(&["--format", "turtle", "assess", "--list"])
        .expect(0);
    let rdf = Rdf::of(&json);
    let assessment = rdf.rows("Assessment")[0];
    assert!(!assessment.boolean("current"));
    let moved = assessment.children("moved");
    assert_eq!(moved.len(), 1);
    assert_eq!(moved[0].required("kind"), "section");
    assert_eq!(moved[0].required("name"), SUBJECT);
    assert_eq!(moved[0].required("change"), "changed");
    assert!(moved[0].text("unresolved").is_none());
    assert_eq!(
        moved[0].required("message"),
        format!("the section `{SUBJECT}` has changed since")
    );
    assert_eq!(repo.both(&["-q", "assess", "--list"]).expect(0).stdout, "");
    // The list writes nothing.
    assert_eq!(repo.snapshot(), edited);
    assert_ne!(edited, before);
}

#[test]
fn moved_inputs_preserve_raw_unresolved_reasons_only_for_unresolved_changes() {
    let repo = Repo::synced("assess-input-kinds", false);
    repo.put(
        "src/lib.rs",
        &format!("{}\npub fn newer() {{}}\n", repo.read("src/lib.rs")),
    );
    repo.run(&["assess", SUBJECT, "--verdict", "accepted", "--by", "agent"])
        .expect(0);
    repo.edit("src/lib.rs", "pub fn real()", "pub fn moved()");
    for format in ["turtle", "ntriples"] {
        let run = repo
            .run(&["--format", format, "assess", "--list"])
            .expect(0);
        let rdf = Rdf::of(&run);
        let moved = rdf.rows("Assessment")[0].children("moved");
        let symbol = moved
            .iter()
            .find(|row| row.required("name") == "demo::real")
            .unwrap();
        assert_eq!(symbol.required("change"), "unresolved");
        assert!(symbol.required("unresolved").contains("demo::real"));
        assert_ne!(symbol.required("unresolved"), symbol.required("message"));
    }
    repo.edit(
        "docs/capabilities/alpha.md",
        "code: [demo::real]",
        "code: [demo::newer]",
    );
    let run = repo
        .run(&["--format", "turtle", "assess", "--list"])
        .expect(0);
    let rdf = Rdf::of(&run);
    let moved = rdf.rows("Assessment")[0].children("moved");
    assert!(
        moved
            .iter()
            .any(|row| row.required("change") == "no-longer-input")
    );
    assert!(
        moved
            .iter()
            .any(|row| row.required("change") == "new-input")
    );
    assert!(moved.iter().all(|row| row.text("unresolved").is_none()));
}

#[test]
fn a_section_that_cannot_be_assessed_is_exit_two_and_writes_nothing() {
    let repo = Repo::synced("assess-refused", false);
    let before = repo.snapshot();
    for args in [
        &["assess", "cap:nowhere#usage", "--context"][..],
        &["assess", "--list", "cap:nowhere#usage"],
        &[
            "assess",
            "cap:nowhere#usage",
            "--verdict",
            "accepted",
            "--by",
            "agent",
        ],
        &[
            "assess",
            SUBJECT,
            "--verdict",
            "accepted",
            "--by",
            "agent",
            "--copyright",
            "two\nlines",
            "--licence",
            LICENCE,
        ],
    ] {
        let run = repo.both(args).expect(2);
        assert_eq!(run.stdout, "", "{args:?}");
        assert!(
            run.stderr.starts_with("cargo-workplan: "),
            "{args:?}: {run:#?}"
        );
        assert_eq!(repo.snapshot(), before, "{args:?}");
    }
}

/// Text a terminal acts on: a clear-screen sequence, a title (OSC ... BEL)
/// and a right-to-left override.
const HOSTILE: &str = "\u{1b}[2J\u{1b}]0;PWNED\u{7} \u{202e}tfel";

#[test]
fn the_text_of_a_section_reaches_the_terminal_escaped_and_rdf_escapes_it_as_rdf() {
    let repo = Repo::synced("assess-context-escaped", false);
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Contract of alpha.",
        &format!("Text for Contract of alpha.{HOSTILE}"),
    );
    let run = repo.both(&["assess", SUBJECT, "--context"]).expect(0);
    assert!(
        !run.stdout
            .chars()
            .any(|c| c == '\u{1b}' || c == '\u{7}' || c == '\u{202e}'),
        "{run:#?}"
    );
    assert!(
        run.stdout
            .contains("Text for Contract of alpha.\\u{1b}[2J\\u{1b}]0;PWNED\\u{7} \\u{202e}tfel"),
        "{run:#?}"
    );
    // Only the hidden characters change: lines and indentation stay.
    assert_eq!(
        run.stdout,
        raf_workplan::escape::shown_lines(&library_context(&repo))
    );
    assert_eq!(
        run.stdout.lines().count(),
        library_context(&repo).lines().count()
    );
    // JSON escapes the controls as JSON does and keeps the text.
    let json = repo
        .both(&["--format", "turtle", "assess", SUBJECT, "--context"])
        .expect(0);
    assert!(
        !json.stdout.contains('\u{1b}') && json.stdout.contains("\\u001B"),
        "{json:#?}"
    );
    let rdf = Rdf::of(&json);
    let section = rdf.rows("AssessmentContext")[0].required("section");
    assert!(section.contains(HOSTILE), "{section:?}");
    // The neighbour: plain text is printed as it is.
    let plain = Repo::synced("assess-context-plain", false);
    assert_eq!(
        plain
            .both(&["assess", SUBJECT, "--context"])
            .expect(0)
            .stdout,
        library_context(&plain)
    );
}
