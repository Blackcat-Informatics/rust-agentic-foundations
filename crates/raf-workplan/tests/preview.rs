// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Preview: a listed capability as it reads once a change is reconciled, the
//! blocks of other changes left alone, the errors, and that nothing is written.

mod support;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use raf_source::{DiskTree, MemoryTree, Workspace};
use raf_workplan::{LinkContext, Preview, PreviewError, Workplan};
use support::{Repo, demo_files, demo_workspace};

const ALPHA: &str = "docs/capabilities/alpha.md";
const BETA: &str = "docs/capabilities/beta.md";
const GAMMA: &str = "docs/capabilities/gamma.md";
const GROW: &str = "work/changes/grow.md";
const OTHER: &str = "work/changes/other.md";

const SECTIONS: [&str; 7] = [
    "Purpose",
    "Contract",
    "Usage",
    "Examples",
    "Constraints",
    "Explanation",
    "Verification",
];

/// A considered capability whose sections hold `Text for <Section>.`, with
/// `extra` added after the text of the named section. `eol` is the line ending
/// of the whole file.
fn document(slug: &str, extra: &[(&str, &str)], eol: &str) -> String {
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
    let text = format!(
        "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\n\
         type: Capability\nid: cap:{slug}\ntitle: \"Title of {slug}\"\n\
         description: A description of {slug}.\nstate: considered\n\
         bindings:\n  code: []\n  tests: []\n  examples: []\n\
         depends: []\n---\n\n# {slug}\n\n{sections}"
    );
    if eol == "\n" {
        text
    } else {
        text.replace('\n', eol)
    }
}

fn change(slug: &str, changes: &str) -> String {
    format!(
        "---\ntype: Change\nid: change:{slug}\ntitle: Change {slug}\n\
         description: Changes {slug}.\nchanges: {changes}\naffects:\n  - cap:alpha#contract\n\
         contract_effect: changes\nitems: []\n---\n\n## Summary\n\nChanges it.\n"
    )
}

/// The Contract section of alpha: two blocks of `grow` around a block of
/// `other`, with a fenced code block, a bare quote line and a tight line.
const ALPHA_CONTRACT: &str = "\
\n> [!PROPOSED change:grow]\n> First claim.\n>\n> Second paragraph.\n\
\nMiddle text.\n\
\n> [!PROPOSED change:other]\n> Text of the other change.\n\
\n> [!PROPOSED change:grow]\n> Third line.\n>tight line\n>\n>    indented\n";

const ALPHA_CONTRACT_GROW: &str = "\
\nFirst claim.\n\nSecond paragraph.\n\
\nMiddle text.\n\
\n> [!PROPOSED change:other]\n> Text of the other change.\n\
\nThird line.\ntight line\n\n   indented\n";

fn base() -> Repo {
    let mut repo = Repo::new();
    let mut put = |path: &str, text: String| {
        repo.insert(path.to_string(), text);
    };
    put(
        ALPHA,
        document("alpha", &[("Contract", ALPHA_CONTRACT)], "\n"),
    );
    put(
        BETA,
        document(
            "beta",
            &[("Purpose", "\n> [!PROPOSED change:grow]\n> Beta grows.\n")],
            "\n",
        ),
    );
    put(GAMMA, document("gamma", &[], "\n"));
    put(GROW, change("grow", "[cap:alpha, cap:beta, cap:gamma]"));
    put(OTHER, change("other", "[cap:alpha]"));
    repo
}

fn preview(repo: &Repo, slug: &str) -> Result<Vec<Preview>, PreviewError> {
    let tree: MemoryTree = repo.iter().collect();
    let source = demo_workspace();
    Workplan::load(&tree, &source).preview(slug)
}

fn texts(previews: &[Preview]) -> BTreeMap<&str, &str> {
    previews
        .iter()
        .map(|p| (p.path.as_str(), p.text.as_str()))
        .collect()
}

fn error(repo: &Repo, slug: &str) -> PreviewError {
    match preview(repo, slug) {
        Ok(previews) => panic!("expected an error, got {previews:#?}"),
        Err(error) => error,
    }
}

#[test]
fn a_change_with_two_blocks_in_a_capability_reads_reconciled_and_other_blocks_stay() {
    let repo = base();
    let previews = preview(&repo, "grow").expect("previews");
    let ids: Vec<&str> = previews.iter().map(|p| p.capability.as_str()).collect();
    assert_eq!(ids, ["cap:alpha", "cap:beta", "cap:gamma"]);
    let paths: Vec<&str> = previews.iter().map(|p| p.path.as_str()).collect();
    assert_eq!(paths, [ALPHA, BETA, GAMMA]);
    assert_eq!(
        previews[0].text,
        document("alpha", &[("Contract", ALPHA_CONTRACT_GROW)], "\n")
    );
    assert_eq!(
        previews[1].text,
        document("beta", &[("Purpose", "\nBeta grows.\n")], "\n")
    );
    // No block of the change: the document, byte for byte.
    assert_eq!(previews[2].text, repo[GAMMA]);
    for preview in &previews {
        assert!(!preview.text.contains("[!PROPOSED change:grow]"));
    }
}

#[test]
fn another_change_reconciles_only_its_own_blocks() {
    let repo = base();
    let previews = preview(&repo, "other").expect("previews");
    assert_eq!(previews.len(), 1);
    let contract = ALPHA_CONTRACT.replace(
        "> [!PROPOSED change:other]\n> Text of the other change.\n",
        "Text of the other change.\n",
    );
    assert!(contract.contains("> [!PROPOSED change:grow]\n> First claim."));
    assert_eq!(
        previews[0].text,
        document("alpha", &[("Contract", &contract)], "\n")
    );
}

#[test]
fn the_slug_is_taken_with_or_without_the_prefix() {
    let repo = base();
    assert_eq!(
        preview(&repo, "change:grow").expect("previews"),
        preview(&repo, "grow").expect("previews")
    );
}

#[test]
fn preview_is_deterministic() {
    let repo = base();
    assert_eq!(
        preview(&repo, "grow").expect("previews"),
        preview(&repo, "grow").expect("previews")
    );
}

#[test]
fn a_crlf_document_keeps_its_line_endings() {
    let mut repo = base();
    repo.insert(
        ALPHA.to_string(),
        document("alpha", &[("Contract", ALPHA_CONTRACT)], "\r\n"),
    );
    let previews = preview(&repo, "grow").expect("previews");
    let expected = document("alpha", &[("Contract", ALPHA_CONTRACT_GROW)], "\r\n");
    assert_eq!(previews[0].text, expected);
    assert!(previews[0].text.contains("First claim.\r\n\r\nSecond"));
    assert!(!previews[0].text.replace("\r\n", "").contains('\n'));
}

#[test]
fn a_block_at_the_end_of_a_file_without_a_final_break_is_reconciled() {
    let mut repo = base();
    let text = format!("{}\n> [!PROPOSED change:grow]\n> Last words.", repo[GAMMA]);
    repo.insert(GAMMA.to_string(), text.clone());
    let previews = preview(&repo, "grow").expect("previews");
    assert_eq!(
        previews[2].text,
        text.replace("> [!PROPOSED change:grow]\n> ", "")
    );
}

#[test]
fn a_block_with_no_content_lines_disappears_with_its_marker() {
    let mut repo = base();
    let text = format!("{}\n> [!PROPOSED change:grow]\n\nAfter.\n", repo[GAMMA]);
    repo.insert(GAMMA.to_string(), text.clone());
    let previews = preview(&repo, "grow").expect("previews");
    assert_eq!(
        previews[2].text,
        text.replace("> [!PROPOSED change:grow]\n", "")
    );
}

#[test]
fn a_block_in_a_list_item_keeps_its_indentation() {
    let mut repo = base();
    let text = format!(
        "{}\n- item\n\n  > [!PROPOSED change:grow]\n  > nested text\n  >\n  > more\n",
        repo[GAMMA]
    );
    repo.insert(GAMMA.to_string(), text);
    let previews = preview(&repo, "grow").expect("previews");
    assert_eq!(
        previews[2].text,
        format!(
            "{}\n- item\n\n  nested text\n\n  more\n",
            document("gamma", &[], "\n")
        )
    );
}

#[test]
fn a_tab_after_the_quote_marker_is_kept_as_written() {
    let mut repo = base();
    let text = format!(
        "{}\n> [!PROPOSED change:grow]\n>\ttabbed\n> \tspace then tab\n",
        repo[GAMMA]
    );
    repo.insert(GAMMA.to_string(), text);
    let previews = preview(&repo, "grow").expect("previews");
    assert_eq!(
        previews[2].text,
        format!(
            "{}\n\ttabbed\n\tspace then tab\n",
            document("gamma", &[], "\n")
        )
    );
}

#[test]
fn a_continuation_that_is_not_indented_as_the_marker_is_refused_with_its_line() {
    let mut repo = base();
    let text = format!(
        "{}\n- item\n\n  > [!PROPOSED change:grow]\n> lazy line\n",
        repo[GAMMA]
    );
    let line = text.lines().count();
    repo.insert(GAMMA.to_string(), text);
    match error(&repo, "grow") {
        PreviewError::Prefix { file, line: at, .. } => {
            assert_eq!(file, GAMMA);
            assert_eq!(at, line);
        }
        other => panic!("{other:?}"),
    }
    let message = error(&repo, "grow").to_string();
    assert!(message.contains(&format!("{GAMMA}:{line}")), "{message}");
}

#[test]
fn a_nested_quote_loses_one_level_only() {
    let mut repo = base();
    let text = format!("{}\n> [!PROPOSED change:grow]\n> > inner\n", repo[GAMMA]);
    repo.insert(GAMMA.to_string(), text);
    let previews = preview(&repo, "grow").expect("previews");
    assert_eq!(
        previews[2].text,
        format!("{}\n> inner\n", document("gamma", &[], "\n"))
    );
}

#[test]
fn an_unknown_slug_names_it_and_lists_the_open_changes() {
    let repo = base();
    match error(&repo, "change:nope") {
        PreviewError::UnknownChange { slug, known } => {
            assert_eq!(slug, "nope");
            assert_eq!(known, ["grow", "other"]);
        }
        other => panic!("{other:?}"),
    }
    let message = error(&repo, "nope").to_string();
    assert!(message.contains("`nope`") && message.contains("grow, other"));
    assert!(matches!(
        error(&Repo::new(), "grow"),
        PreviewError::UnknownChange { known, .. } if known.is_empty()
    ));
    assert!(matches!(
        error(&repo, ""),
        PreviewError::UnknownChange { .. }
    ));
}

#[test]
fn a_change_record_that_did_not_load_says_why() {
    let mut repo = base();
    let text = repo[GROW].replacen("contract_effect: changes", "contract_effect: bogus", 1);
    repo.insert(GROW.to_string(), text);
    match error(&repo, "grow") {
        PreviewError::ChangeNotLoaded { slug, findings } => {
            assert_eq!(slug, "grow");
            assert!(
                findings.iter().any(|f| f.kind.starts_with("okf-")),
                "{findings:?}"
            );
            assert!(findings.iter().all(|f| f.file == GROW));
        }
        other => panic!("{other:?}"),
    }
    let message = error(&repo, "grow").to_string();
    assert!(message.contains("did not load") && message.contains(GROW));
}

#[test]
fn a_listed_capability_that_did_not_load_is_named_never_skipped() {
    let mut repo = base();
    let text = repo[BETA].replacen("state: considered", "state: finished", 1);
    repo.insert(BETA.to_string(), text);
    match error(&repo, "grow") {
        PreviewError::CapabilityNotLoaded {
            slug,
            capability,
            findings,
        } => {
            assert_eq!(slug, "grow");
            assert_eq!(capability, "cap:beta");
            assert!(
                findings.iter().any(|f| f.kind.starts_with("okf-")),
                "{findings:?}"
            );
            assert!(findings.iter().all(|f| f.file == BETA));
        }
        other => panic!("{other:?}"),
    }
    let message = error(&repo, "grow").to_string();
    assert!(message.contains("cap:beta") && message.contains(BETA));
}

#[test]
fn a_listed_capability_without_a_document_is_an_error() {
    let mut repo = base();
    repo.remove(GAMMA);
    match error(&repo, "grow") {
        PreviewError::CapabilityMissing { capability, .. } => {
            assert_eq!(capability, "cap:gamma");
        }
        other => panic!("{other:?}"),
    }
}

// --- the tree is not written ---------------------------------------------------

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "raf-workplan-preview-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Under the temporary directory; only litter.
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Every file under `dir` with its bytes and modification time, by path.
fn snapshot(dir: &Path, out: &mut BTreeMap<PathBuf, (Vec<u8>, std::time::SystemTime)>) {
    for entry in fs::read_dir(dir).expect("lists") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            snapshot(&path, out);
        } else {
            let modified = fs::metadata(&path)
                .and_then(|m| m.modified())
                .expect("mtime");
            out.insert(path.clone(), (fs::read(&path).expect("reads"), modified));
        }
    }
}

#[test]
fn a_disk_fixture_is_byte_identical_afterwards() {
    let scratch = Scratch::new();
    let root = &scratch.0;
    for (path, text) in demo_files() {
        write(root, path, text);
    }
    for (path, text) in &base() {
        write(root, path, text);
    }
    let mut before = BTreeMap::new();
    snapshot(root, &mut before);
    let tree = DiskTree::new(root).expect("opens");
    let source = Workspace::load(&tree, &["."]).expect("workspace");
    let plan = Workplan::load_with(&tree, &source, &LinkContext::of_tree(&tree));
    let first = plan.preview("grow").expect("previews");
    assert_eq!(first, plan.preview("grow").expect("previews"));
    assert_ne!(first[0].text, base()[ALPHA]);
    let mut after = BTreeMap::new();
    snapshot(root, &mut after);
    assert_eq!(before, after);
}

fn write(root: &Path, path: &str, text: &str) {
    let target = root.join(path);
    fs::create_dir_all(target.parent().expect("parent")).expect("creates");
    fs::write(&target, text).expect("writes");
}

#[test]
fn the_text_outside_blocks_is_unchanged_byte_for_byte() {
    // Strip the replaced blocks from both sides: what remains is equal.
    let repo = base();
    let all = preview(&repo, "grow").expect("previews");
    let previews = texts(&all);
    let alpha = previews[ALPHA];
    for kept in [
        "---\n# SPDX-FileCopyrightText: 2026 Example\n",
        "\nMiddle text.\n",
        "> [!PROPOSED change:other]\n> Text of the other change.\n",
        "## Verification\n\nText for Verification of alpha.\n",
    ] {
        assert!(alpha.contains(kept), "{kept}");
    }
}

#[test]
fn a_fenced_code_block_in_a_block_of_the_change_is_refused_with_its_line() {
    // The document model ends a block at a fence, so the quote that goes on
    // after it leaves the end of the block ambiguous: fail, never guess.
    let mut repo = base();
    let head = document("gamma", &[], "\n");
    let text =
        format!("{head}\n> [!PROPOSED change:grow]\n> Before.\n> ```rust\n> let x = 1;\n> ```\n");
    let fence_line = head.lines().count() + 4;
    repo.insert(GAMMA.to_string(), text);
    match error(&repo, "grow") {
        PreviewError::Prefix { file, line, .. } => {
            assert_eq!(file, GAMMA);
            assert_eq!(line, fence_line);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_fenced_code_block_in_a_block_of_another_change_is_no_obstacle() {
    let mut repo = base();
    let head = document("gamma", &[], "\n");
    let text = format!(
        "{head}\n> [!PROPOSED change:other]\n> ```rust\n> let x = 1;\n> ```\n\n> [!PROPOSED change:grow]\n> Mine.\n"
    );
    repo.insert(GAMMA.to_string(), text);
    let previews = preview(&repo, "grow").expect("previews");
    assert_eq!(
        previews[2].text,
        format!("{head}\n> [!PROPOSED change:other]\n> ```rust\n> let x = 1;\n> ```\n\nMine.\n")
    );
}

#[test]
fn adjacent_blocks_of_two_changes_are_told_apart() {
    let mut repo = base();
    let head = document("gamma", &[], "\n");
    let text = format!(
        "{head}\n> [!PROPOSED change:grow]\n> Mine.\n> [!PROPOSED change:other]\n> Theirs.\n"
    );
    repo.insert(GAMMA.to_string(), text);
    let previews = preview(&repo, "grow").expect("previews");
    assert_eq!(
        previews[2].text,
        format!("{head}\nMine.\n> [!PROPOSED change:other]\n> Theirs.\n")
    );
}
