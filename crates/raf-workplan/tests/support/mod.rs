// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared fixtures: a valid repository over an in-memory tree, the helpers that
//! mutate it and judge the findings, and the named cases the coverage test
//! reads.

#![allow(
    dead_code,
    reason = "each test binary uses a part of the shared support"
)]

pub(crate) mod assessed;
pub(crate) mod cases;
pub(crate) mod changes;
pub(crate) mod checkout;
pub(crate) mod disk;
pub(crate) mod evidence;
pub(crate) mod project;
pub(crate) mod story;
pub(crate) mod streaming;
pub(crate) mod streaming_code;
pub(crate) mod strict;
pub(crate) mod synced;
pub(crate) mod window;
pub(crate) mod window_code;

use std::collections::BTreeMap;

use raf_okf::Location;
use raf_source::{MemoryTree, RelPath, Workspace};
use raf_workplan::{Finding, LinkContext, Workplan};

/// Path to text, as the repository holds them.
pub(crate) type Repo = BTreeMap<String, String>;

/// The library of the demo workspace: a documented and an undocumented
/// function, a macro-generated function, a module declared in its own file, an
/// inline module, a re-export, two globs that bring in one name each, and a
/// test-only module with a re-export of its function.
pub(crate) const DEMO_LIB: &str = "\
macro_rules! make {
    ($name:ident) => {
        pub fn $name() {}
    };
}
make!(generated);
/// A documented function.
pub fn real() {}
pub fn bare() { generated(); }
mod body;
pub mod inner {
    /// Goes.
    pub fn go() {}
}
pub use inner::go as shown;
pub mod left {
    pub fn dup() {}
}
pub mod right {
    pub fn dup() {}
}
#[allow(unused_imports, reason = \"The imports exercise ambiguous binding resolution.\")] use left::*;
#[allow(unused_imports, reason = \"The imports exercise ambiguous binding resolution.\")] use right::*;
#[cfg(test)]
mod unit {
    /// A documented helper of the tests.
    pub fn helper() {}
    #[test] fn helper_contract() { helper(); }
}
#[cfg(test)]
pub use unit::helper as unit_helper;
";

/// A small source workspace the documents bind to: the library
/// [`DEMO_LIB`], a module in its own file, an integration test with a test
/// function and a plain one, an example with a documented helper, and an
/// example the manifest marks `doctest = true`.
pub(crate) fn demo_workspace() -> Workspace {
    let tree: MemoryTree = demo_files().into_iter().collect();
    match Workspace::load(&tree, &["."]) {
        Ok(workspace) => workspace,
        Err(error) => panic!("the demo workspace loads: {error}"),
    }
}

/// The files of the demo workspace, by path.
pub(crate) fn demo_files() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n\
             [[example]]\nname = \"flagged\"\npath = \"examples/flagged.rs\"\ndoctest = true\n",
        ),
        ("src/lib.rs", DEMO_LIB),
        // `blocks.rs` sorts before `mod.rs`, which declares it.
        ("src/body/mod.rs", "mod blocks;\npub fn in_body() {}\n"),
        ("src/body/blocks.rs", "pub(crate) struct FenceBlock;\n"),
        (
            "tests/it.rs",
            "#[test]\nfn case() { plain(); }\npub fn plain() {}\n",
        ),
        (
            "examples/show.rs",
            "fn main() {}\n/// A documented helper of the example.\npub fn helper() {}\n/// A documented module of the example.\npub mod inner {}\n",
        ),
        (
            "examples/flagged.rs",
            "fn main() {}\n/// A documented helper.\npub fn flagged_helper() {}\n",
        ),
    ]
}

/// The source workspace of `files`.
pub(crate) fn workspace_of(files: &[(&'static str, String)]) -> Workspace {
    let tree: MemoryTree = files
        .iter()
        .map(|(path, text)| (*path, text.as_str()))
        .collect();
    match Workspace::load(&tree, &["."]) {
        Ok(workspace) => workspace,
        Err(error) => panic!("the workspace loads: {error}"),
    }
}

/// Load `repo` through a memory tree against `workspace` and return every
/// finding but those that only say the repository is not synced.
pub(crate) fn check_in(repo: &Repo, workspace: &Workspace) -> Vec<Finding> {
    let tree: MemoryTree = repo.iter().collect();
    without_sync_findings(Workplan::load(&tree, workspace).check())
}

/// Load `repo` through a memory tree and return every finding.
pub(crate) fn check(repo: &Repo) -> Vec<Finding> {
    check_with_symlinks(repo, &[])
}

/// Load `repo` through a memory tree whose paths `symlinks` are tracked
/// symlinks, and return every finding.
pub(crate) fn check_with_symlinks(repo: &Repo, symlinks: &[&str]) -> Vec<Finding> {
    let tree: MemoryTree = repo.iter().collect();
    let workspace = demo_workspace();
    let context =
        LinkContext::of_tree(&tree).with_symlinks(symlinks.iter().map(|path| path_of(path)));
    without_sync_findings(Workplan::load_with(&tree, &workspace, &context).check())
}

/// The kinds that only say the generated content has not been synced.
const SYNC_KINDS: [&str; 2] = ["index-missing", "generated-stale"];

/// `findings` without the two kinds that say the repository has not been
/// synced. The shared fixtures carry no capability index and are changed one
/// document at a time, so these findings would repeat in every case; the
/// generated-block rules are tested on synced repositories in `generated.rs`
/// and `sync.rs`.
pub(crate) fn without_sync_findings(findings: Vec<Finding>) -> Vec<Finding> {
    findings
        .into_iter()
        .filter(|finding| !SYNC_KINDS.contains(&finding.kind))
        .collect()
}

/// The path `text` as a tree path.
pub(crate) fn path_of(text: &str) -> RelPath {
    match RelPath::new(text) {
        Ok(path) => path,
        Err(error) => panic!("`{text}` is not a path: {error}"),
    }
}

/// Check the Markdown file `path` of `repo` as a file outside the document
/// directories, against the tracked paths of the repository and the named
/// symlinks.
pub(crate) fn plain_check(repo: &Repo, path: &str, symlinks: &[&str]) -> Vec<Finding> {
    let tree: MemoryTree = repo.iter().collect();
    let context =
        LinkContext::of_tree(&tree).with_symlinks(symlinks.iter().map(|path| path_of(path)));
    let text = repo
        .get(path)
        .unwrap_or_else(|| panic!("the repository has no {path}"));
    raf_workplan::plain::check(path, text, &context)
}

/// The valid repository with `text` added at `path`, and the findings of
/// checking that file as plain Markdown.
pub(crate) fn with_file(path: &str, text: &str) -> Vec<Finding> {
    let mut repo = fixtures::valid();
    repo.insert(path.to_string(), text.to_string());
    plain_check(&repo, path, &[])
}

/// Where the coverage record lives.
pub(crate) const COVERAGE: &str = "work/coverage/default.md";

/// An empty explicit scope with no exclusions; binding-reached packages remain judged.
/// Every fixture repository holds it, so none relies on a repository without
/// a record, which `check` reports as `coverage-record-missing`.
pub(crate) fn open_coverage() -> String {
    "---\ntype: Coverage\nid: coverage:default\nscope: []\nexclusions: []\n---\n".to_string()
}

/// The document owning auxiliary items of the binding-index fixture.
pub(crate) const AUXILIARY: &str = "docs/capabilities/auxiliary.md";

/// The auxiliary source is fully explained, independently of the capability
/// whose binding a test edits. No exclusion, ceiling or alternate policy is used.
pub(crate) fn auxiliary_capability() -> String {
    strict::capability()
        .replace("id: cap:demo", "id: cap:auxiliary")
        .replace("title: \"Demo\"", "title: \"Auxiliary binding fixtures\"")
        .replace(
            "  code: [demo::run]",
            "  code: [demo, demo::make, demo::real, demo::bare, demo::body, demo::inner, demo::left, demo::right]",
        )
        .replace(
            "  tests: [demo::test_crate::it::case]",
            "  tests: [demo::unit::helper_contract, demo::test_crate::it::case]",
        )
        .replace(
            strict::EXAMPLES,
            "  examples: [demo::example_crate::show, demo::example_crate::flagged]\n",
        )
        .replace(
            strict::MARKER,
            "<!-- raf:tested-by demo::unit::helper_contract -->",
        )
        .replace(
            strict::USAGE_SPAN,
            "Run `demo::example_crate::show` to build the example target.",
        )
}

/// Add the actual auxiliary source owner to a demo-index repository.
pub(crate) fn with_auxiliary(mut repo: Repo) -> Repo {
    repo.insert(AUXILIARY.to_string(), auxiliary_capability());
    repo
}

/// `repo` with the coverage record of [`open_coverage`] at its home, unless
/// it holds one.
pub(crate) fn with_open_coverage(mut repo: Repo) -> Repo {
    repo.entry(COVERAGE.to_string())
        .or_insert_with(open_coverage);
    repo
}

pub(crate) mod fixtures {
    use super::{Repo, check};
    use raf_workplan::Finding;

    pub(crate) const SECTIONS: [&str; 7] = [
        "Purpose",
        "Contract",
        "Usage",
        "Examples",
        "Constraints",
        "Explanation",
        "Verification",
    ];

    /// The complete authored paragraph used by the capability fixture.
    pub(crate) fn paragraph(name: &str) -> String {
        format!(
            "Text for {name}. This fixture describes the declared source surface and its bindings so each observed result can be checked against the same small workspace.\n"
        )
    }

    /// One complete section of the capability fixture, excluding its separator.
    pub(crate) fn capability_section(name: &str, state: &str, extra: &str) -> String {
        let lead = if name == "Contract" {
            "**Result.** "
        } else {
            ""
        };
        let marker = if state == "delivered" {
            "<!-- raf:tested-by demo::test_crate::it::case -->\n"
        } else {
            ""
        };
        let policy = match name {
            "Contract" => marker.to_string(),
            "Constraints" => format!(
                "{marker}\n- The small fixture describes its own source declarations and examples, without claiming behavior for another package or unlisted external input.\n{marker}"
            ),
            "Usage" => "Run `demo::example_crate::show` as the bound example.\n".to_string(),
            "Verification" => super::strict::MATRIX.to_string(),
            _ => String::new(),
        };
        format!("## {name}\n\n{extra}{lead}{}{policy}", paragraph(name))
    }

    pub(crate) fn capability(slug: &str, state: &str, depends: &str, body: &str) -> String {
        let sections = SECTIONS
            .iter()
            .map(|name| {
                let extra = if *name == "Purpose" { body } else { "" };
                capability_section(name, state, extra)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let (code, tests) = if state == "delivered" {
            ("[demo::real]", "[demo::test_crate::it::case]")
        } else {
            ("[]", "[]")
        };
        let examples = if state == "delivered" {
            "[demo::example_crate::show]"
        } else {
            "[]"
        };
        format!(
            "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\n\
             type: Capability\nid: cap:{slug}\ntitle: \"Title of {slug}\"\n\
             description: A description of {slug}.\nstate: {state}\n\
             bindings:\n  code: {code}\n  tests: {tests}\n  examples: {examples}\n\
             depends: {depends}\n---\n\n# {slug}\n\n{sections}"
        )
    }

    const PROPOSED: &str = "> [!PROPOSED change:add-alpha]\n> The candidate contract.\n\n";

    pub(crate) const ALPHA: &str = "docs/capabilities/alpha.md";
    pub(crate) const BETA: &str = "docs/capabilities/beta.md";
    pub(crate) const GAMMA: &str = "docs/capabilities/gamma.md";
    pub(crate) const OVERVIEW: &str = "docs/architecture/overview.md";
    pub(crate) const CHANGE: &str = "work/changes/add-alpha.md";
    /// The Explanation text of gamma in the valid repository: it links the
    /// architecture page.
    pub(crate) const OVERVIEW_LINK: &str =
        "Text for Explanation. See [the overview](../architecture/overview.md).";

    pub(crate) fn valid() -> Repo {
        let mut repo = Repo::new();
        let mut put = |path: &str, text: String| {
            repo.insert(path.to_string(), text);
        };
        put(ALPHA, capability("alpha", "designed", "[cap:beta]", ""));
        put(BETA, capability("beta", "considered", "[]", PROPOSED));
        // Gamma's Explanation links the architecture page, so no page is
        // unlinked.
        put(
            GAMMA,
            capability("gamma", "delivered", "[cap:beta, cap:alpha]", "").replacen(
                "Text for Explanation.",
                OVERVIEW_LINK,
                1,
            ),
        );
        put(
            OVERVIEW,
            "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
             description: How it fits.\n---\n\n# Overview\n\nBody text.\n"
                .to_string(),
        );
        put(
            CHANGE,
            "---\ntype: Change\nid: change:add-alpha\ntitle: Add alpha\n\
             description: Adds alpha and beta.\n\
             changes: [cap:alpha, cap:beta]\naffects:\n  - cap:alpha#contract\n  - cap:beta#purpose\n\
             contract_effect: changes\nitems:\n  - claim: \"alpha reads input\"\n    outcome: implemented\n  \
             - claim: \"beta is sketched\"\n---\n\n## Summary\n\nAdds both.\n"
                .to_string(),
        );
        put(
            "README.md",
            "# Readme\n\nSee [alpha](docs/capabilities/alpha.md#purpose).\n".to_string(),
        );
        put(super::COVERAGE, super::open_coverage());
        super::with_auxiliary(repo)
    }

    /// A change record with the given `changes` list, `affects` lines and
    /// `items` block (the text after `items:`).
    pub(crate) fn change_record(slug: &str, changes: &str, affects: &str, items: &str) -> String {
        let items = if items.is_empty() {
            "items: []\n".to_string()
        } else {
            format!("items:\n{items}")
        };
        format!(
            "---\ntype: Change\nid: change:{slug}\ntitle: Change {slug}\n\
             description: Changes {slug}.\nchanges: {changes}\naffects:\n{affects}\
             contract_effect: changes\n{items}---\n\n## Summary\n\nChanges it.\n"
        )
    }

    /// The valid repository with the first `from` in `path` replaced by `to`.
    pub(crate) fn mutate(path: &str, from: &str, to: &str) -> Repo {
        let mut repo = valid();
        let text = repo
            .get_mut(path)
            .unwrap_or_else(|| panic!("fixture has no {path}"));
        assert!(text.contains(from), "fixture {path} has no `{from}`");
        *text = text.replacen(from, to, 1);
        repo
    }

    /// The valid repository with `gamma` bound as given: the three lists as
    /// the text after `code:`, `tests:` and `examples:`.
    pub(crate) fn bound(code: &str, tests: &str, examples: &str) -> Repo {
        let mut repo = mutate(GAMMA, "  code: [demo::real]", &format!("  code: {code}"));
        let text = repo.get_mut(GAMMA).unwrap_or_else(|| panic!("no gamma"));
        *text = text
            .replacen(
                "  tests: [demo::test_crate::it::case]",
                &format!("  tests: {tests}"),
                1,
            )
            .replacen(
                "  examples: [demo::example_crate::show]",
                &format!("  examples: {examples}"),
                1,
            );
        let example = examples
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .next()
            .unwrap_or("")
            .trim();
        let marker = if example.is_empty() {
            String::new()
        } else {
            format!("<!-- raf:tested-by {example} -->")
        };
        *text = text
            .replace(
                "Run `demo::example_crate::show` as the bound example.",
                &format!("Run `{example}` as the bound example."),
            )
            .replace("<!-- raf:tested-by demo::test_crate::it::case -->", &marker);
        repo
    }

    /// The findings of the valid repository with `gamma` bound as given.
    pub(crate) fn bound_findings(code: &str, tests: &str, examples: &str) -> Vec<Finding> {
        check(&bound(code, tests, examples))
    }

    pub(crate) fn mutated(path: &str, from: &str, to: &str) -> Vec<Finding> {
        check(&mutate(path, from, to))
    }
}

/// Require a finding of `kind` in `file` at `at`.
pub(crate) fn assert_hit(findings: &[Finding], kind: &str, file: &str, at: &Location) {
    assert!(
        findings
            .iter()
            .any(|f| f.kind == kind && f.file == file && &f.location == at),
        "expected `{kind}` in {file} at {at:?}, got: {findings:#?}"
    );
}

/// Require a finding of `kind` in `file`, wherever it is.
pub(crate) fn assert_kind(findings: &[Finding], kind: &str, file: &str) {
    assert!(
        findings.iter().any(|f| f.kind == kind && f.file == file),
        "expected `{kind}` in {file}, got: {findings:#?}"
    );
}

/// Apply each `(from, to, kind, location)` mutation to `path` and require the
/// finding.
pub(crate) fn assert_cases(path: &str, cases: &[(&str, &str, &str, Location)]) {
    for (from, to, kind, at) in cases {
        let findings = fixtures::mutated(path, from, to);
        assert!(
            findings
                .iter()
                .any(|f| f.kind == *kind && f.file == path && &f.location == at),
            "replacing `{from}` with `{to}` should report `{kind}` at {at:?}, got: {findings:#?}"
        );
    }
}

pub(crate) fn assert_clean(findings: &[Finding]) {
    assert!(findings.is_empty(), "{findings:#?}");
}

pub(crate) fn key(name: &str) -> Location {
    Location::Key(name.to_string())
}

pub(crate) fn section(name: &str) -> Location {
    Location::Section(name.to_string())
}

/// The 1-based line of the first line of `text` that contains `needle`.
pub(crate) fn line_of(text: &str, needle: &str) -> usize {
    match text.lines().position(|line| line.contains(needle)) {
        Some(index) => index + 1,
        None => panic!("no line contains `{needle}`"),
    }
}
