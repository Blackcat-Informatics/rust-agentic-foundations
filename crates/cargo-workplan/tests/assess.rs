// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `assess` through the built binary: the context, the same for a human and
//! an agent, that writes nothing; a verdict that writes exactly one record and
//! exits 0, rejected or accepted; the list, current until an input moves and
//! then stale naming it; `-q`; the JSON objects; and the exit 2 of a section
//! that cannot be assessed.

mod support;

use raf_workplan::Gate;
use support::{COPYRIGHT, LICENCE, Repo, array, changed, member_names, strings};

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
    // The JSON object, and `-q`, which hides it since it is no finding.
    let json = repo
        .both(&["--format", "json", "assess", SUBJECT, "--context"])
        .expect(0);
    assert_eq!(
        member_names(&json),
        [strings(&[
            "kind",
            "subject",
            "record",
            "record_path",
            "document",
            "section",
            "symbols",
            "evidence",
            "inputs"
        ])]
    );
    let object = &array(&json)[0];
    assert_eq!(object["kind"], "assess-context");
    assert_eq!(object["record_path"], RECORD);
    assert_eq!(object["symbols"][0]["path"], "demo::real");
    assert_eq!(object["inputs"][0]["kind"], "section");
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
            "json",
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
    assert_eq!(
        member_names(&run),
        [strings(&[
            "kind", "path", "id", "subject", "verdict", "by", "replaced", "inputs"
        ])]
    );
    let object = &array(&run)[0];
    assert_eq!(object["kind"], "assessment-recorded");
    assert_eq!(object["path"], RECORD);
    assert_eq!(object["id"], "assessment:alpha-contract");
    assert_eq!(object["subject"], SUBJECT);
    assert_eq!(object["verdict"], "rejected");
    assert_eq!(object["by"], "human");
    assert_eq!(object["replaced"], true);

    // Under `-q` a recorded verdict prints nothing and is still written.
    repo.both_writing(&[
        "-q",
        "assess",
        SUBJECT,
        "--verdict",
        "accepted",
        "--by",
        "agent",
    ])
    .expect(0);
    assert!(repo.read(RECORD).contains("verdict: accepted"));
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
        .both(&["--format", "json", "assess", "--list"])
        .expect(0);
    assert_eq!(
        member_names(&json),
        [strings(&[
            "kind", "id", "path", "subject", "verdict", "by", "current", "moved"
        ])]
    );
    let object = &array(&json)[0];
    assert_eq!(object["kind"], "assessment");
    assert_eq!(object["current"], false);
    assert_eq!(
        object["moved"],
        serde_json::json!([{
            "kind": "section",
            "name": SUBJECT,
            "change": "changed",
            "message": format!("the section `{SUBJECT}` has changed since"),
        }])
    );
    assert_eq!(repo.both(&["-q", "assess", "--list"]).expect(0).stdout, "");
    // The list writes nothing.
    assert_eq!(repo.snapshot(), edited);
    assert_ne!(edited, before);
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
