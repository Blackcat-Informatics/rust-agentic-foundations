// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A supersession chain with one current sibling and a removable source definition.

use std::path::Path;

use super::{Repo, capability};

pub(crate) fn decision(repo: &Repo, slug: &str, replacements: &str, body: &str) {
    repo.put(
        &format!("docs/architecture/{slug}.md"),
        &format!(
            "---\ntype: Architecture\nid: arch:{slug}\ntitle: Design\n\
             description: A design decision.\n{replacements}---\n\n# Design\n\n{body}\n\n\
             ## Conditions to revisit\n\n- A governing requirement changes.\n"
        ),
    );
}

pub(crate) const MIDDLE: &str = "docs/architecture/middle.md";
pub(crate) const CURRENT: &str = "docs/architecture/current.md";
pub(crate) const RETIRED_BLOCK: &str =
    "## Decision\n\nEarlier rationale.\n\n### Intermediate\n\nApply the earlier compromise.\n\n";
pub(crate) const INITIAL_BLOCK: &str =
    "## Decision\n\n### Initial\n\nApply the initial assumptions.\n\nApply the earlier compromise.";
pub(crate) const STANDALONE: &str = "## Decision\n\nA separate current decision.";
pub(crate) const OLD_DEFINITION: &str = "pub fn obsolete() -> bool { false }\n";
pub(crate) const OLD_FILE: &str = "src/implementation.rs";

pub(crate) fn retirement_fixture(tag: &str) -> Repo {
    let repo = Repo::bare_in(Path::new(env!("CARGO_TARGET_TMPDIR")), tag);
    repo.put(
        "docs/capabilities/view.md",
        &capability(
            "view",
            "delivered",
            [
                "[demo::real]",
                "[demo::test_crate::it::case]",
                "[demo::real]",
            ],
            "[]",
            &[("Explanation", "[Current](../architecture/current.md).")],
        ),
    );
    decision(
        &repo,
        "current",
        "supersedes:\n  latest: ['arch:middle#intermediate']\n",
        "[Middle](middle.md).\n\n## Decision\n\n### Latest\n\nRead the latest requirements.",
    );
    decision(
        &repo,
        "middle",
        "supersedes:\n  intermediate: ['arch:earliest#initial']\n",
        &format!(
            "[Current](current.md) and [earlier](earliest.md).\n\n{RETIRED_BLOCK}## Decision\n\n### Bounds\n\nKeep exact bounds.\n\n{STANDALONE}"
        ),
    );
    decision(&repo, "earliest", "", INITIAL_BLOCK);
    repo.put(
        "src/lib.rs",
        "mod implementation;\n/// A retained example.\n/// ```\n/// demo::real();\n/// ```\npub fn real() {}\n",
    );
    repo.put(OLD_FILE, &format!("{OLD_DEFINITION}fn keep() {{}}\n"));
    repo.checkout();
    repo
}

pub(crate) fn retire_middle(repo: &Repo) {
    repo.edit(CURRENT, "arch:middle#intermediate", "arch:earliest#initial");
    repo.edit(
        MIDDLE,
        "supersedes:\n  intermediate: ['arch:earliest#initial']\n",
        "",
    );
    repo.edit(MIDDLE, RETIRED_BLOCK, "");
}

pub(crate) const RETIRE: &[&str] = &[
    "docs",
    "retire",
    "--base",
    "refs/heads/main",
    "--section",
    "arch:middle#intermediate",
];

pub(crate) fn select_definition() -> Vec<&'static str> {
    RETIRE
        .iter()
        .copied()
        .chain(["--definition", "demo::implementation::obsolete"])
        .collect()
}
