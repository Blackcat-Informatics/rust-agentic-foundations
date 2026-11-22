// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Dated source/document history with a removed binding and stable commit data.

use super::{Repo, capability, change, git_with};

pub(crate) const ALPHA: &str = "docs/capabilities/alpha.md";

/// Commit everything with an author and a committer date of `day` at noon
/// UTC (`--since` reads the author date).
pub(crate) fn commit_on(repo: &Repo, message: &str, day: &str) {
    repo.git(&["add", "-A"]);
    let date = format!("{day}T12:00:00+00:00");
    git_with(
        repo.root(),
        &["commit", "-q", "--date", &date, "-m", message],
        &[("GIT_COMMITTER_DATE", &date)],
    );
}

/// A repository of four dated commits:
///
/// * `base` (2030-01-01) adds `alpha`, bound to `demo::real` and
///   `demo::gone`, and the change `grow`;
/// * `edit alpha` (2030-01-02) edits the document;
/// * `change real` (2030-01-03) edits the body of `demo::real`;
/// * `remove gone` (2030-01-04) removes `demo::gone`;
/// * `unrelated` (2030-01-05) edits the test file, which no source names.
pub(crate) fn dated(tag: &str) -> Repo {
    let repo = Repo::bare(tag);
    repo.put(
        "src/lib.rs",
        "/// A documented function.\npub fn real() {}\n\n/// Soon gone.\npub fn gone() {}\n",
    );
    repo.put(
        ALPHA,
        &capability(
            "alpha",
            "implementing",
            [
                "[demo::real, demo::gone]",
                "[demo::test_crate::it::case]",
                "[]",
            ],
            "[]",
            &[],
        ),
    );
    repo.put(
        "work/changes/grow.md",
        &change("grow", "[cap:alpha]", &["cap:alpha#contract"]),
    );
    repo.sync();
    repo.git(&["init", "-q", "-b", "main"]);
    commit_on(&repo, "base", "2030-01-01");
    repo.edit(ALPHA, "Text for Purpose of alpha.", "Alpha has a purpose.");
    commit_on(&repo, "edit alpha", "2030-01-02");
    repo.edit(
        "src/lib.rs",
        "pub fn real() {}",
        "pub fn real() {\n    let _ = 1;\n}",
    );
    commit_on(&repo, "change real", "2030-01-03");
    repo.edit("src/lib.rs", "\n/// Soon gone.\npub fn gone() {}\n", "");
    commit_on(&repo, "remove gone", "2030-01-04");
    repo.edit(
        "tests/it.rs",
        "fn case() {}",
        "fn case() {\n    let _ = 2;\n}",
    );
    commit_on(&repo, "unrelated", "2030-01-05");
    repo
}
