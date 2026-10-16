// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A repository with generated blocks: the builder of its documents, the
//! helpers that sync it in memory, and the synced base the generated-block
//! cases break.

use raf_okf::{Actor, AssessmentVerdict};
use raf_source::MemoryTree;
use raf_workplan::{Finding, SyncError, SyncOptions, SyncReport, Workplan};

use super::assessed::records_of;
use super::{Repo, demo_workspace};

/// The copyright text of an index the fixtures create.
pub(crate) const COPYRIGHT: &str = "2026 Example Org <dev@example.org>";
/// The licence identifier of an index the fixtures create.
pub(crate) const LICENCE: &str = "CC-BY-4.0";

pub(crate) const INDEX: &str = "docs/capabilities/index.md";
pub(crate) const ALPHA: &str = "docs/capabilities/alpha.md";
pub(crate) const BETA: &str = "docs/capabilities/beta.md";
pub(crate) const GAMMA: &str = "docs/capabilities/gamma.md";
pub(crate) const DELTA: &str = "docs/capabilities/delta.md";
pub(crate) const GROW: &str = "work/changes/grow.md";
pub(crate) const OVERVIEW: &str = "docs/architecture/overview.md";

pub(crate) const MATRIX: &str =
    "<!-- raf:generated:verification-matrix -->\n<!-- raf:generated:end -->\n";
pub(crate) const RELATIONSHIPS: &str =
    "<!-- raf:generated:relationships -->\n<!-- raf:generated:end -->\n";

pub(crate) fn options() -> SyncOptions {
    SyncOptions::new(COPYRIGHT, LICENCE)
}

/// The SPDX comment lines of an index the fixtures create.
pub(crate) fn index_header() -> String {
    format!(
        "<!-- {}{COPYRIGHT} -->\n<!-- {}{LICENCE} -->\n",
        concat!("SPDX-", "FileCopyrightText: "),
        concat!("SPDX-", "License-Identifier: "),
    )
}

const SECTIONS: [&str; 7] = [
    "Purpose",
    "Contract",
    "Usage",
    "Examples",
    "Constraints",
    "Explanation",
    "Verification",
];

/// A capability document. `bindings` is the text after `code:`, `tests:` and
/// `examples:`; `extra` is added after the text of each section, by section
/// name.
pub(crate) fn document(
    slug: &str,
    description: &str,
    state: &str,
    bindings: [&str; 3],
    depends: &str,
    extra: &[(&str, &str)],
) -> String {
    let [code, tests, examples] = bindings;
    let sections = SECTIONS
        .iter()
        .map(|name| {
            let added: String = extra
                .iter()
                .filter(|(section, _)| section == name)
                .map(|(_, text)| *text)
                .collect();
            format!("## {name}\n\nText for {name} of {slug}.\n{added}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\n\
         type: Capability\nid: cap:{slug}\ntitle: \"Title of {slug}\"\n\
         description: {description}\nstate: {state}\n\
         bindings:\n  code: {code}\n  tests: {tests}\n  examples: {examples}\n\
         depends: {depends}\n---\n\n# {slug}\n\n{sections}"
    )
}

/// The documents of the repository the generated blocks are judged on, with
/// every block empty and no index:
///
/// * `gamma`, delivered, binds a test and two examples (an example target and
///   a documented library item), names the test in its Contract and the target
///   in its Usage and its Examples, and holds a matrix and a relationship
///   table;
/// * `beta`, implementing, binds one code item and nothing else, depends on `gamma`, and holds a relationship table and a matrix;
/// * `alpha`, designed, depends on `beta` and `gamma` and holds a relationship
///   table;
/// * `delta`, considered, has a proposal and no block;
///
/// and the open change `grow`, which lists `alpha`, `beta` and `delta`.
pub(crate) fn unsynced() -> Repo {
    let mut repo = Repo::new();
    let mut put = |path: &str, text: String| {
        repo.insert(path.to_string(), text);
    };
    put(
        GAMMA,
        document(
            "gamma",
            "Delivered and bound.",
            "delivered",
            [
                "[demo::real]",
                "[demo::test_crate::it::case]",
                "[demo::example_crate::show, demo::real]",
            ],
            "[]",
            &[
                (
                    "Contract",
                    "\nThe test `demo::test_crate::it::case` checks it.\n",
                ),
                ("Usage", "\nRun `demo::example_crate::show` to see it.\n"),
                (
                    "Examples",
                    "\nThe example `demo::example_crate::show` runs.\n",
                ),
                ("Verification", &format!("\n{MATRIX}\n{RELATIONSHIPS}")),
            ],
        ),
    );
    put(
        BETA,
        document(
            "beta",
            "Being built | with \"care\".",
            "implementing",
            ["[demo::bare]", "[]", "[]"],
            "[cap:gamma]",
            &[("Verification", &format!("\n{RELATIONSHIPS}\n{MATRIX}"))],
        ),
    );
    put(
        ALPHA,
        document(
            "alpha",
            "Planned work.",
            "designed",
            ["[]", "[]", "[]"],
            "[cap:gamma, cap:beta]",
            &[("Verification", &format!("\n{RELATIONSHIPS}"))],
        ),
    );
    put(
        DELTA,
        document(
            "delta",
            "Only considered.",
            "considered",
            ["[]", "[]", "[]"],
            "[]",
            &[(
                "Purpose",
                "\n> [!PROPOSED change:grow]\n> The candidate contract.\n",
            )],
        ),
    );
    put(
        GROW,
        "---\ntype: Change\nid: change:grow\ntitle: Grow\n\
         description: Grows three capabilities.\n\
         changes: [cap:alpha, cap:beta, cap:delta]\naffects:\n  - cap:delta#purpose\n\
         contract_effect: changes\nitems:\n  - claim: \"beta is built\"\n---\n\n## Summary\n\nGrows.\n"
            .to_string(),
    );
    put(
        OVERVIEW,
        "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
         description: How it fits.\n---\n\n# Overview\n\nBody text.\n"
            .to_string(),
    );
    repo
}

/// `unsynced` after one sync: every block filled and the index created.
pub(crate) fn base() -> Repo {
    synced(&unsynced())
}

/// Load `repo` and run `f` on the plan.
pub(crate) fn with_plan<R>(repo: &Repo, f: impl FnOnce(&Workplan<'_>) -> R) -> R {
    let tree: MemoryTree = repo.iter().collect();
    let workspace = demo_workspace();
    f(&Workplan::load(&tree, &workspace))
}

/// The findings of `check` of a plan.
fn findings_of(plan: &Workplan<'_>) -> Vec<Finding> {
    plan.check()
}

/// Every finding of `check`, none left out.
pub(crate) fn check_all(repo: &Repo) -> Vec<Finding> {
    with_plan(repo, findings_of)
}

/// The findings of `check` that belong to the generated blocks.
pub(crate) fn generated_findings(repo: &Repo) -> Vec<Finding> {
    check_all(repo)
        .into_iter()
        .filter(|f| {
            [
                "index-missing",
                "generated-stale",
                "generated-unknown-kind",
                "generated-misplaced",
                "index-hand-written",
                "mixed-line-endings",
            ]
            .contains(&f.kind)
        })
        .collect()
}

/// The report of a sync of `repo` with the fixture options.
pub(crate) fn sync(repo: &Repo) -> Result<SyncReport, SyncError> {
    with_plan(repo, |plan| plan.sync(&options()))
}

/// `repo` with the edits of `report` applied.
pub(crate) fn apply(repo: &Repo, report: &SyncReport) -> Repo {
    let mut next = repo.clone();
    for (path, text) in &report.edits {
        next.insert(path.as_str().to_string(), text.clone());
    }
    next
}

/// `repo` after one sync.
pub(crate) fn synced(repo: &Repo) -> Repo {
    match sync(repo) {
        Ok(report) => apply(repo, &report),
        Err(error) => panic!("the fixture does not sync: {error}"),
    }
}

/// `repo` with a current accepted assessment by an agent of each authored
/// section `change` affects, replacing the assessments of those sections
/// that are there.
pub(crate) fn assessed(repo: &Repo, change: &str) -> Repo {
    assessed_as(repo, change, AssessmentVerdict::Accepted, Actor::Agent)
}

/// `repo` with a current assessment by `by` with `verdict` of each authored
/// section `change` affects.
pub(crate) fn assessed_as(
    repo: &Repo,
    change: &str,
    verdict: AssessmentVerdict,
    by: Actor,
) -> Repo {
    let records = with_plan(repo, |plan| records_of(plan, change, verdict, by));
    let mut next = repo.clone();
    for (path, text) in records {
        next.insert(path, text);
    }
    next
}
