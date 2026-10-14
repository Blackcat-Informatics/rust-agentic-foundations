// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The link rules over every link form `raf-okf` reads: inline links and
//! images, reference definitions, `href` and `src` attributes, and fragment
//! links, judged against a set of tracked paths and the anchors of the target.

mod support;

use std::collections::BTreeSet;

use raf_okf::Location;
use raf_source::{
    Entry, EntryKind, FileId, MemoryTree, Opened, RelPath, Tree, TreeError, Workspace,
};
use raf_workplan::{Finding, LinkContext, Workplan};
use support::fixtures::{OVERVIEW, mutate, valid};
use support::{assert_clean, check, demo_workspace, line_of, path_of, plain_check, with_file};

const README: &str = "README.md";

/// The findings of the README of the valid repository after one replacement.
fn readme(from: &str, to: &str) -> Vec<Finding> {
    let repo = mutate(README, from, to);
    plain_check(&repo, README, &[])
}

/// Require exactly one finding, of `kind`, at the line of `needle` in `text`.
fn assert_only(findings: &[Finding], kind: &str, file: &str, text: &str, needle: &str) {
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, kind);
    assert_eq!(findings[0].file, file);
    assert_eq!(
        findings[0].location,
        Location::Line(line_of(text, needle)),
        "{findings:#?}"
    );
}

fn assert_kind_with(findings: &[Finding], kind: &str, message: &str) {
    assert!(
        findings
            .iter()
            .any(|f| f.kind == kind && f.message.contains(message)),
        "expected `{kind}` with `{message}`, got: {findings:#?}"
    );
}

#[test]
fn rejects_broken_relative_link() {
    let findings = readme(
        "docs/capabilities/alpha.md#purpose",
        "docs/capabilities/nope.md",
    );
    assert_kind_with(
        &findings,
        "link-broken",
        "broken link `docs/capabilities/nope.md`",
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].location, Location::Line(3));
}

#[test]
fn rejects_broken_anchor() {
    let findings = readme("alpha.md#purpose", "alpha.md#no-such-heading");
    assert_kind_with(&findings, "anchor-broken", "broken anchor");
    assert_eq!(findings.len(), 1);
}

#[test]
fn rejects_link_to_a_tracked_symlink() {
    let mut repo = valid();
    repo.insert("docs/outside-link.svg".to_string(), String::new());
    let template = ".github/PULL_REQUEST_TEMPLATE.md";
    repo.insert(
        template.to_string(),
        "# Template\n\n![x](../docs/outside-link.svg)\n".to_string(),
    );
    let findings = plain_check(&repo, template, &["docs/outside-link.svg"]);
    assert_kind_with(
        &findings,
        "link-symlink",
        "link `../docs/outside-link.svg` targets a tracked symlink",
    );
    // The same link to the same path, when the path is not a symlink.
    assert_clean(&plain_check(&repo, template, &[]));
}

#[test]
fn rejects_link_leaving_the_repository() {
    let findings = readme("docs/capabilities/alpha.md#purpose", "../../outside.md");
    assert_kind_with(&findings, "link-outside", "leaves the repository");
    assert_eq!(findings.len(), 1);
}

#[test]
fn anchors_follow_github_rules() {
    let target = "# Target\n\n## Why `x`: it's (fine)\n\n## Same\n\n## Same\n\n## Ünï Größe\n";
    let link = |anchor: &str| {
        let mut repo = valid();
        repo.insert("TARGET.md".to_string(), target.to_string());
        repo.insert("LINKS.md".to_string(), format!("[a](TARGET.md#{anchor})\n"));
        plain_check(&repo, "LINKS.md", &[])
    };
    for anchor in ["why-x-its-fine", "same", "same-1", "target", "ünï-größe"] {
        assert_clean(&link(anchor));
    }
    assert_kind_with(
        &link("same-2"),
        "anchor-broken",
        "broken anchor `TARGET.md#same-2`",
    );
    assert_kind_with(&link("Why-x-its-fine"), "anchor-broken", "broken anchor");
}

#[test]
fn rejects_broken_reference_definition() {
    for (text, kind, message) in [
        (
            "# Readme\n\n[ref]: ./missing.md",
            "link-broken",
            "broken link `./missing.md`",
        ),
        (
            "# Readme\n\n  [ref]: <./missing.md> \"Title\"",
            "link-broken",
            "broken link `./missing.md`",
        ),
        (
            "# Readme\n\n[ref]: docs/capabilities/alpha.md#no-such",
            "anchor-broken",
            "broken anchor",
        ),
    ] {
        assert_kind_with(&readme("# Readme", text), kind, message);
    }
}

#[test]
fn rejects_broken_href_and_src_attributes() {
    for (text, kind, message) in [
        (
            "# Readme\n\n<a href=\"docs/missing.md\">gone</a>",
            "link-broken",
            "broken link `docs/missing.md`",
        ),
        (
            "# Readme\n\n<img src='docs/missing.svg' width=\"2\">",
            "link-broken",
            "broken link `docs/missing.svg`",
        ),
        (
            "# Readme\n\n<a HREF=\"../../outside.md\">out</a>",
            "link-outside",
            "leaves the repository",
        ),
    ] {
        assert_kind_with(&readme("# Readme", text), kind, message);
    }
}

#[test]
fn rejects_broken_link_outside_the_document_directories() {
    for path in [
        ".github/PULL_REQUEST_TEMPLATE.md",
        ".github/ISSUE_TEMPLATE/bug.md",
        "crates/tool/README.md",
    ] {
        let text = "See [gone](docs/missing.md).\n";
        assert_only(&with_file(path, text), "link-broken", path, text, "gone");
    }
}

#[test]
fn accepts_resolving_reference_and_attribute_links() {
    let text = "# Template\n\n[docs]: ../docs/architecture/overview.md\n\
         <img src=\"../docs/architecture/overview.md\" alt=\"x\">\n\
         <a href=\"https://example.invalid/a\">site</a> <a href=\"#template\">top</a>\n\
         A footnote[^1] and `[code]: ./missing.md` stay quiet.\n\n[^1]: A note.\n";
    assert_clean(&with_file(".github/PULL_REQUEST_TEMPLATE.md", text));
}

#[test]
fn autolinks_are_external_and_not_judged() {
    let text =
        "# Notes\n\n<https://example.invalid/a> and <mailto:a@example.invalid> and <nope.md>.\n";
    assert_clean(&with_file("NOTES.md", text));
}

#[test]
fn a_link_to_a_directory_resolves_and_its_anchor_is_not_read() {
    assert_clean(&readme(
        "# Readme",
        "# Readme\n\n[docs](docs/) and [d](docs/capabilities#x) and [root](.)",
    ));
}

#[test]
fn rejects_a_link_whose_target_cannot_be_read_for_its_anchor() {
    let mut repo = valid();
    repo.insert(
        "BAD.md".to_string(),
        "# Bad\n\n> [!PROPOSED add-alpha]\n".to_string(),
    );
    repo.insert("LINK.md".to_string(), "[a](BAD.md#bad)\n".to_string());
    let findings = plain_check(&repo, "LINK.md", &[]);
    assert_kind_with(
        &findings,
        "anchor-broken",
        "link `BAD.md#bad` targets `BAD.md`, which cannot be read as Markdown",
    );
    assert_eq!(findings[0].location, Location::Line(1));
}

#[test]
fn an_anchor_in_a_non_markdown_file_is_not_judged() {
    let mut repo = valid();
    repo.insert("data.json".to_string(), "{}".to_string());
    repo.insert(
        "LINK.md".to_string(),
        "[a](data.json#section)\n".to_string(),
    );
    assert_clean(&plain_check(&repo, "LINK.md", &[]));
}

#[test]
fn the_oracle_is_the_tracked_set_and_not_the_tree() {
    let tree = MemoryTree::from([
        ("README.md", "# Readme\n"),
        ("extra.md", "# Extra\n"),
        ("docs/architecture/overview.md", "x"),
    ]);
    let text = "See [extra](extra.md) and [readme](README.md).\n";
    let tracked =
        |paths: &[&str]| -> BTreeSet<RelPath> { paths.iter().map(|p| path_of(p)).collect() };
    // Every file the tree holds is tracked.
    assert_clean(&raf_workplan::plain::check(
        "NOTES.md",
        text,
        &LinkContext::of_tree(&tree),
    ));
    // `extra.md` exists in the tree and is not tracked.
    let context = LinkContext::new(&tree, tracked(&["README.md", "NOTES.md"]));
    let findings = raf_workplan::plain::check("NOTES.md", text, &context);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "link-broken");
    assert!(findings[0].message.contains("extra.md"));
}

#[test]
fn a_symlink_the_caller_names_is_tracked_and_still_a_symlink() {
    let tree = MemoryTree::from([("README.md", "# Readme\n")]);
    let context = LinkContext::new(&tree, tracked_readme()).with_symlinks([path_of("link.md")]);
    let findings = raf_workplan::plain::check("NOTES.md", "[a](link.md)\n", &context);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "link-symlink");
}

fn tracked_readme() -> BTreeSet<RelPath> {
    BTreeSet::from([path_of("README.md")])
}

#[test]
fn a_document_is_judged_against_the_supplied_tracked_set() {
    let mut repo = valid();
    repo.insert("extra.md".to_string(), "# Extra\n".to_string());
    let text = repo.get_mut(OVERVIEW).expect("overview");
    *text = text.replace("Body text.", "Body text. See [extra](../../extra.md).");
    // Walked from the tree, the link resolves.
    assert_clean(&check(&repo));
    // With a tracked set that leaves the file out, it does not.
    let tree: MemoryTree = repo.iter().collect();
    let tracked: BTreeSet<RelPath> = repo
        .keys()
        .filter(|path| *path != "extra.md")
        .map(|path| path_of(path))
        .collect();
    let workspace = demo_workspace();
    let findings =
        Workplan::load_with(&tree, &workspace, &LinkContext::new(&tree, tracked)).check();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "link-broken");
    assert_eq!(findings[0].file, OVERVIEW);
}

/// A tree that cannot list one directory.
struct Flaky(MemoryTree);
impl Tree for Flaky {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        self.0.read(path, max_bytes)
    }
    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        self.0.read_unseen(path, max_bytes, seen)
    }
    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        if path.as_str() == "locked" {
            Err(TreeError::NotRegular {
                path: path.to_string(),
            })
        } else {
            let mut entries = self.0.list(path)?;
            if path.is_root() {
                entries.push(Entry::new("locked", EntryKind::Dir));
            }
            Ok(entries)
        }
    }
    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        self.0.entry(path)
    }
}

#[test]
fn a_directory_the_walk_cannot_list_is_reported_and_not_skipped() {
    let mut repo = valid();
    let overview = repo.get_mut(OVERVIEW).expect("overview");
    *overview = overview.replace("Body text.", "Body text. See [readme](../../README.md).");
    let tree = Flaky(repo.iter().collect());
    let workspace: Workspace = demo_workspace();
    let findings = Workplan::load(&tree, &workspace).check();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "unreadable-directory");
    assert_eq!(findings[0].file, "locked");
}

#[test]
fn the_tree_is_walked_only_when_a_relative_link_needs_it() {
    // The repository has no relative link in a document, so the directory the
    // walk could not list is never reached.
    let tree = Flaky(valid().iter().collect());
    let workspace = demo_workspace();
    assert_clean(&Workplan::load(&tree, &workspace).check());
}

#[test]
fn links_inside_documents_are_judged_wherever_the_document_is_valid_or_not() {
    // A capability that breaks a rule of the profile still has its links
    // judged, so one fix does not uncover the next.
    let repo = mutate(OVERVIEW, "Body text.", "Body text. See [gone](nope.md).");
    let mut repo = repo;
    let text = repo.get_mut(OVERVIEW).expect("overview");
    *text = text.replace("title: Overview", "title: \"\"");
    let findings = check(&repo);
    assert!(
        findings.iter().any(|f| f.kind == "okf-empty-text"),
        "{findings:#?}"
    );
    assert!(
        findings.iter().any(|f| f.kind == "link-broken"),
        "{findings:#?}"
    );
}

#[test]
fn a_fragment_link_is_judged_against_its_own_document() {
    let repo = mutate(
        OVERVIEW,
        "Body text.",
        "Body text. [up](#overview) [no](#absent).",
    );
    let findings = check(&repo);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "anchor-broken");
    assert_eq!(
        findings[0].location,
        Location::Line(line_of(&repo[OVERVIEW], "#absent"))
    );
}

#[test]
fn rejects_an_empty_link_target() {
    for (text, line) in [
        ("# Notes\n\n[a]()\n", 3),
        ("# Notes\n\n![img]()\n", 3),
        ("# Notes\n\n[a]( )\n", 3),
    ] {
        let findings = with_file("NOTES.md", text);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].kind, "link-broken");
        assert_eq!(findings[0].location, Location::Line(line));
        assert!(findings[0].message.contains("empty target"));
    }
    assert_clean(&with_file(
        "NOTES.md",
        "# Notes\n\n[a](#notes) and `[b]()`\n",
    ));
}

#[test]
fn rejects_absolute_and_empty_link_targets_beside_relative_ones() {
    let findings = with_file("NOTES.md", "# Notes\n\n[a](/docs/x.md)\n");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "link-outside");
    assert!(
        findings[0]
            .message
            .contains("link `/docs/x.md` is an absolute path")
    );
    assert_clean(&with_file(
        "NOTES.md",
        "# Notes\n\n[a](docs/architecture/overview.md)\n",
    ));
}

/// Require a finding of `kind` at `line` whose message holds `message`.
fn assert_at(findings: &[Finding], kind: &str, line: usize, message: &str) {
    assert!(
        findings.iter().any(|f| f.kind == kind
            && f.location == Location::Line(line)
            && f.message.contains(message)),
        "expected `{kind}` at line {line} with `{message}`, got: {findings:#?}"
    );
}

#[test]
fn rejects_broken_unquoted_attribute_values() {
    for (text, kind, message) in [
        (
            "# Readme\n\n<a href=docs/missing.md>x</a>",
            "link-broken",
            "broken link `docs/missing.md`",
        ),
        (
            "# Readme\n\n<img src=docs/missing.svg />",
            "link-broken",
            "broken link `docs/missing.svg`",
        ),
        (
            "# Readme\n\n<a href=../../outside.md>x</a>",
            "link-outside",
            "leaves the repository",
        ),
    ] {
        assert_kind_with(&readme("# Readme", text), kind, message);
    }
    assert_clean(&with_file(
        ".github/note.md",
        "# Note\n\n<a href=../docs/architecture/overview.md>ok</a>\n",
    ));
}

#[test]
fn reads_attributes_with_spaces_around_the_equals_sign() {
    for (text, message) in [
        (
            "# Readme\n\n<a href = \"docs/missing.md\">x</a>",
            "broken link `docs/missing.md`",
        ),
        (
            "# Readme\n\n<img src\t=  docs/missing.svg>",
            "broken link `docs/missing.svg`",
        ),
    ] {
        assert_kind_with(&readme("# Readme", text), "link-broken", message);
    }
}

#[test]
fn does_not_read_attributes_that_only_end_in_href_or_src() {
    let text = "# Readme\n\n<a data-href=\"docs/missing.md\" x-src='docs/none.svg' data_src=docs/n.svg>x</a>\n\
                Use the href attribute and src = nothing here.\n";
    assert_clean(&readme("# Readme\n", text));
}

#[test]
fn rejects_broken_attributes_in_tags_split_across_lines() {
    for (text, line, message) in [
        (
            "<img\n  src=\"docs/missing.svg\"\n  alt=\"x\">\n",
            4,
            "broken link `docs/missing.svg`",
        ),
        (
            "<img\n  alt=\"x\"\n  src=docs/missing.svg />\n",
            5,
            "broken link `docs/missing.svg`",
        ),
        (
            "<img\nalt=\"x\"\nsrc = 'docs/missing.svg'\nwidth=\"2\">\n",
            5,
            "broken link `docs/missing.svg`",
        ),
        (
            "<a\n  href=\"docs/missing.md\">gone</a>\n",
            4,
            "broken link `docs/missing.md`",
        ),
        (
            "<a title=\"x>y\" href=\"./nope.md\">gone</a>\n",
            3,
            "broken link `./nope.md`",
        ),
        (
            "<a title=\"two\nlines>\" href=\"./nope.md\">gone</a>\n",
            4,
            "broken link `./nope.md`",
        ),
    ] {
        let findings = readme("# Readme\n", &format!("# Readme\n\n{text}"));
        assert_at(&findings, "link-broken", line, message);
    }
}

#[test]
fn tag_scan_spares_prose_autolinks_and_closed_tags() {
    let text = "# Readme\n\nWrite src = nothing or href = nothing; if a < b then x > y.\n\
                <https://example.invalid/?src=a&href=b>\n\
                <p align=\"center\">\n  <img src=\"docs/architecture/overview.md\" alt=\"ok\">\n</p>\n\
                After the tag, src = nothing is prose again.\n";
    assert_clean(&readme("# Readme\n", text));
}

#[test]
fn tag_state_does_not_survive_a_blank_line_or_a_fence() {
    let broken = "<img src=\"./nope.png\">\n";
    for prose in [
        "a<b it's fine\n\n",
        "a<b \"quoted\n\n",
        "a<b it's fine\n\n```text\ncode\n```\n",
    ] {
        let findings = readme("# Readme\n", &format!("# Readme\n\n{prose}{broken}"));
        assert_kind_with(&findings, "link-broken", "broken link `./nope.png`");
    }
}

#[test]
fn reads_an_attribute_value_that_starts_on_the_next_line() {
    for (text, line, message) in [
        (
            "<img src=\n  \"docs/missing.svg\" alt=\"x\">\n",
            4,
            "broken link `docs/missing.svg`",
        ),
        (
            "<a href =\n  docs/missing.md>gone</a>\n",
            4,
            "broken link `docs/missing.md`",
        ),
    ] {
        let findings = readme("# Readme\n", &format!("# Readme\n\n{text}"));
        assert_at(&findings, "link-broken", line, message);
    }
    let fine = "<img src=\n  \"docs/architecture/overview.md\">\n";
    assert_clean(&readme("# Readme\n", &format!("# Readme\n\n{fine}")));
}

/// A tree that lists a symlink at its root.
struct Linked(MemoryTree);

impl Tree for Linked {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        self.0.read(path, max_bytes)
    }
    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        self.0.read_unseen(path, max_bytes, seen)
    }
    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        let mut entries = self.0.list(path)?;
        if path.is_root() {
            entries.push(Entry::new("link.md", EntryKind::Symlink));
            entries.push(Entry::new("linked", EntryKind::Symlink));
        }
        Ok(entries)
    }
    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        self.0.entry(path)
    }
}

#[test]
fn the_symlinks_a_tree_lists_are_tracked_symlinks() {
    let tree = Linked(MemoryTree::from([("other.md", "# Other\n")]));
    let context = LinkContext::of_tree(&tree);
    let findings = raf_workplan::plain::check(
        "NOTES.md",
        "[a](link.md) [b](other.md) [c](linked) [d](link.md#top)\n",
        &context,
    );
    let kinds: Vec<(&str, &str)> = findings
        .iter()
        .map(|f| (f.kind, f.message.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            ("link-symlink", "link `link.md` targets a tracked symlink"),
            ("link-symlink", "link `linked` targets a tracked symlink"),
            (
                "link-symlink",
                "link `link.md#top` targets a tracked symlink"
            ),
        ],
        "{findings:#?}"
    );
}

#[test]
fn the_directories_of_a_named_symlink_are_tracked_directories() {
    let tree = MemoryTree::from([("README.md", "# Readme\n")]);
    let context =
        LinkContext::new(&tree, tracked_readme()).with_symlinks([path_of("deep/dir/link.md")]);
    assert_clean(&raf_workplan::plain::check(
        "NOTES.md",
        "[a](deep/dir) [b](deep/)\n",
        &context,
    ));
    let findings = raf_workplan::plain::check("NOTES.md", "[a](deep/di)\n", &context);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "link-broken");
}
