// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A synced repository with one capability, `omega`, and one change record,
//! `shape`, that lists it: the base the obligation fixtures break, one
//! document at a time.

use super::Repo;
use super::synced::{self, document};

pub(crate) const OMEGA: &str = "docs/capabilities/omega.md";
pub(crate) const SHAPE: &str = "work/changes/shape.md";

/// The generated blocks of a section the fixtures keep in `Verification`.
pub(crate) const BLOCKS: &str = "\n<!-- raf:generated:verification-matrix -->\n<!-- raf:generated:end -->\n\n<!-- raf:generated:relationships -->\n<!-- raf:generated:end -->\n";

/// A bound `omega`: code, a test and an example, all of which resolve.
pub(crate) const BOUND: [&str; 3] = [
    "[demo::real]",
    "[demo::test_crate::it::case]",
    "[demo::example_crate::show]",
];

/// A capability document with the generated blocks of `Verification`, and
/// `extra` added to the end of the named sections.
pub(crate) fn capability(
    slug: &str,
    state: &str,
    bindings: [&str; 3],
    extra: &[(&str, &str)],
) -> String {
    let mut extra = extra.to_vec();
    extra.push(("Verification", BLOCKS));
    document(slug, "Shaped.", state, bindings, "[]", &extra)
}

/// A change record that lists `changes` and affects `affects`.
pub(crate) fn record(
    slug: &str,
    changes: &str,
    affects: &[&str],
    effect: &str,
    items: &str,
) -> String {
    let affects = affects
        .iter()
        .map(|entry| format!("  - {entry}\n"))
        .collect::<Vec<_>>()
        .concat();
    let items = if items.is_empty() {
        "items: []\n".to_string()
    } else {
        format!("items:\n{items}")
    };
    format!(
        "---\ntype: Change\nid: change:{slug}\ntitle: Change {slug}\n\
         description: Changes {slug}.\nchanges: {changes}\naffects:\n{affects}\
         contract_effect: {effect}\n{items}---\n\n## Summary\n\nChanges it.\n"
    )
}

pub(crate) const DONE: &str = "  - claim: \"omega is shaped\"\n    outcome: implemented\n";

/// The repository of the fixtures, synced: `omega` implementing and fully
/// bound, and the change `shape` that lists it and resolves its one item.
pub(crate) fn base() -> Repo {
    repo(&[
        (OMEGA, capability("omega", "implementing", BOUND, &[])),
        (
            SHAPE,
            record(
                "shape",
                "[cap:omega]",
                &["cap:omega#contract"],
                "changes",
                DONE,
            ),
        ),
    ])
}

/// The documents synced into a repository.
pub(crate) fn repo(files: &[(&str, String)]) -> Repo {
    let mut repo = Repo::new();
    for (path, text) in files {
        repo.insert((*path).to_string(), text.clone());
    }
    synced::synced(&super::with_open_coverage(repo))
}

/// `repo` with the first `from` of `path` replaced by `to`.
pub(crate) fn replaced(repo: &Repo, path: &str, from: &str, to: &str) -> Repo {
    let mut next = repo.clone();
    let text = next
        .get_mut(path)
        .unwrap_or_else(|| panic!("the fixture has no {path}"));
    assert!(text.contains(from), "{path} has no `{from}`");
    *text = text.replacen(from, to, 1);
    next
}

/// `repo` without `path`.
pub(crate) fn without(repo: &Repo, path: &str) -> Repo {
    let mut next = repo.clone();
    assert!(next.remove(path).is_some(), "the fixture has no {path}");
    next
}
