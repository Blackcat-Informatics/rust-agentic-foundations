// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The ratchet against a report of lines that could not be counted: no row
//! can hold that finding, and the rows of `lines` are not judged resolved
//! while the lines are not known.

use raf_source::{MemoryTree, Namespace, Node, Workspace};

use super::{Judged, Recorded, lines_judged};
use crate::coverage::lines;
use crate::{Loaded, Workplan};

/// A workspace of one function, and a larger one whose item is no item of it.
fn workspaces() -> (Workspace, Workspace) {
    let load = |name: &str, library: &str| {
        Workspace::load(
            &MemoryTree::from([
                (
                    "Cargo.toml",
                    format!(
                        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"
                    )
                    .as_str(),
                ),
                ("src/lib.rs", library),
            ]),
            &["."],
        )
        .expect("a workspace")
    };
    (
        load("demo", "pub fn run() {}\n"),
        load("wide", "pub fn a() {}\npub fn b() {}\npub fn c() {}\n"),
    )
}

#[test]
fn a_report_that_could_not_count_is_one_finding_no_row_holds_and_resolves_no_row() {
    let (small, large) = workspaces();
    let foreign = large
        .resolve_in("wide::c", Namespace::Value)
        .expect("the item")
        .sites[0];
    let docs = MemoryTree::from([
        (
            "work/coverage/default.md",
            "---\ntype: Coverage\nid: coverage:default\nstrictness: strict\nscope: []\nexclusions: []\n---\n",
        ),
        (
            "work/debt/default.md",
            "---\ntype: Debt\nid: debt:default\nfindings: []\nlines:\n  - item: demo::run\n    orphans: 1\n---\n",
        ),
    ]);
    let plan = Workplan::load(&docs, &small);
    let report = lines::compute(&plan, &[Node::Item(foreign)]);
    assert_eq!(report.errors().len(), 1);
    let loaded: &Loaded<_> = plan.coverage().expect("the coverage record");
    let overs = lines::ceiling_findings(loaded, &report);
    let recorded = Recorded::default();
    let mut out = Judged::default();
    let mut resolved = Vec::new();
    lines_judged(&plan, (report, overs), &recorded, &mut out, &mut resolved);
    assert_eq!(out.unheld.len(), 1, "{:#?}", out.unheld);
    assert_eq!(out.unheld[0].kind, "coverage-lines-over-ceiling");
    assert!(out.unheld[0].message.contains("cannot be counted"));
    assert_eq!(out.reported, out.unheld);
    assert!(out.growth.is_empty() && out.lines.is_empty() && resolved.is_empty());
}
