// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `section-untouched` through `Lifecycle::reconcile`: the affected sections
//! compared with the file at the merge base in real git repositories (edited on
//! the branch, unchanged, only a generated block changed, a file new on the
//! branch, the default branch itself, a linked work tree, a local branch named
//! like the remote one, a default branch named like an option, no default
//! branch, an oversize and an unparsable file at the merge base), the version
//! gate over stubs that record what they were started with, and the digest
//! itself.

mod support;

use std::fs;
use std::path::Path;

use raf_okf::{Body, CapabilitySection, MAX_TEXT_BYTES};
use raf_workplan::Gate;
use raf_workplan::exec::{Call, FakeRunner, Output, Program};
use raf_workplan::git::GitError;
use raf_workplan::lifecycle::{Lifecycle, LifecycleError, Side, section_digest};
use raf_workplan::obligation::ObligationKind;
use support::Repo;
use support::changes::{DONE, OMEGA, SHAPE, capability, record, replaced, repo};
use support::checkout::{Checkout, serial};
use support::disk::{self, snapshot};
use support::synced::assessed;

/// The evidence column of `section-untouched`.
const EVIDENCE: &str =
    "a diff of that section against the merge base, or an edit that records why none is needed";

/// The bindings of `omega`: code and a test.
const BOUND: [&str; 3] = ["[demo::real]", "[demo::test_crate::it::case]", "[]"];

/// `omega`, delivered and bound to code and a test (no example, so no `cargo`
/// is needed), and the change `shape`, which affects its `contract` and `usage`
/// sections. Synced.
fn base() -> Repo {
    with_affects(&["cap:omega#contract", "cap:omega#usage"], "changes")
}

/// The repository, its affected sections assessed as they stand, so the list
/// holds the kinds these fixtures are about and nothing else.
fn with_affects(affects: &[&str], effect: &str) -> Repo {
    assessed(
        &repo(&[
            (OMEGA, capability("omega", "delivered", BOUND, &[])),
            (SHAPE, record("shape", "[cap:omega]", affects, effect, DONE)),
        ]),
        "shape",
    )
}

fn lifecycle(checkout: &Checkout) -> Lifecycle {
    checkout
        .lifecycle()
        .unwrap_or_else(|error| panic!("the checkout opens: {error}"))
}

/// The `section-untouched` lines of the reconcile, as `capability section`.
fn untouched(checkout: &Checkout) -> Vec<String> {
    let list = lifecycle(checkout)
        .reconcile("shape")
        .unwrap_or_else(|error| panic!("reconcile should succeed: {error}"));
    for obligation in list.of_kind(ObligationKind::SectionUntouched) {
        assert_eq!(obligation.evidence.to_string(), EVIDENCE);
    }
    list.of_kind(ObligationKind::SectionUntouched)
        .map(|o| format!("{} {}", o.subject, o.section))
        .collect()
}

fn reconcile_error(checkout: &Checkout) -> LifecycleError {
    lifecycle(checkout)
        .reconcile("shape")
        .expect_err("reconcile should fail")
}

const CONTRACT: &str = "cap:omega contract";
const USAGE: &str = "cap:omega usage";

// --- the matrix of the merge base ------------------------------------------

#[test]
fn an_unchanged_affected_section_is_owed_with_the_line_the_contract_shows() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-unchanged", &base());
    let list = lifecycle(&checkout)
        .reconcile("shape")
        .expect("reconcile succeeds");
    assert_eq!(
        list.to_string(),
        format!(
            "cap:omega  contract  section-untouched  {EVIDENCE}\n\
             cap:omega  usage  section-untouched  {EVIDENCE}\n"
        )
    );
    let first = &list.obligations()[0];
    assert_eq!(first.location.path, OMEGA);
    assert_eq!(first.location.line, None);
}

#[test]
fn a_section_edited_on_the_branch_is_not_reported_and_its_neighbour_is() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-edited", &base());
    checkout.put(
        OMEGA,
        &replaced(
            &base(),
            OMEGA,
            "Text for Contract of omega.",
            "Text for Contract of omega, now saying more.",
        )[OMEGA],
    );
    checkout.commit("edit the contract");
    assert_eq!(untouched(&checkout), [USAGE]);
}

#[test]
fn an_edit_that_is_not_committed_counts_because_the_working_tree_is_compared() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-working", &base());
    assert_eq!(untouched(&checkout), [CONTRACT, USAGE]);
    checkout.put(
        OMEGA,
        &replaced(
            &base(),
            OMEGA,
            "Text for Usage of omega.",
            "Run it like this.",
        )[OMEGA],
    );
    assert_eq!(untouched(&checkout), [CONTRACT]);
}

#[test]
fn only_a_generated_block_changed_is_still_reported() {
    let _lock = serial();
    let mut repo = base();
    // Give the contract a generated block of its own, committed on the default
    // branch, so that the section holds one at the merge base.
    let with_block = repo[OMEGA].replacen(
        "Text for Contract of omega.\n",
        "Text for Contract of omega.\n\n<!-- raf:generated:verification-matrix -->\nrow one\n<!-- raf:generated:end -->\n",
        1,
    );
    repo.insert(OMEGA.to_string(), with_block.clone());
    let checkout = Checkout::new("untouched-generated", &repo);

    // Only the text between the markers moves.
    checkout.put(
        OMEGA,
        &with_block.replacen("row one", "row one\nrow two, added by a sync", 1),
    );
    assert_eq!(
        untouched(&checkout),
        [CONTRACT, USAGE],
        "a regenerated block is not authored text"
    );

    // The neighbour: authored text beside the same block is a touch.
    checkout.put(
        OMEGA,
        &with_block.replacen(
            "Text for Contract of omega.",
            "Text for Contract, edited.",
            1,
        ),
    );
    assert_eq!(untouched(&checkout), [USAGE]);

    // The markers are authored: removing the block is an edit.
    checkout.put(
        OMEGA,
        &with_block.replacen(
            "\n<!-- raf:generated:verification-matrix -->\nrow one\n<!-- raf:generated:end -->\n",
            "",
            1,
        ),
    );
    assert_eq!(untouched(&checkout), [USAGE]);
}

#[test]
fn text_in_a_fence_and_in_a_proposed_block_is_authored() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-fence", &base());
    let text = &base()[OMEGA];
    checkout.put(
        OMEGA,
        &text.replacen(
            "Text for Contract of omega.\n",
            "Text for Contract of omega.\n\n```text\nan example\n```\n",
            1,
        ),
    );
    assert_eq!(untouched(&checkout), [USAGE], "a fence is authored");
    checkout.put(
        OMEGA,
        &text.replacen(
            "Text for Usage of omega.\n",
            "Text for Usage of omega.\n\n> [!PROPOSED change:shape]\n> The candidate text.\n",
            1,
        ),
    );
    assert_eq!(
        untouched(&checkout),
        [CONTRACT],
        "a PROPOSED block is authored"
    );
}

#[test]
fn a_file_absent_at_the_merge_base_counts_as_touched() {
    let _lock = serial();
    // The default branch holds `omega` only; the branch adds `sigma` and the
    // change that affects both.
    let main = repo(&[(OMEGA, capability("omega", "delivered", BOUND, &[]))]);
    let checkout = Checkout::new("untouched-absent", &main);
    let sigma = "docs/capabilities/sigma.md";
    let branch = repo(&[
        (OMEGA, capability("omega", "delivered", BOUND, &[])),
        (sigma, capability("sigma", "delivered", BOUND, &[])),
        (
            SHAPE,
            record(
                "shape",
                "[cap:omega, cap:sigma]",
                &["cap:omega#contract", "cap:sigma#contract"],
                "changes",
                DONE,
            ),
        ),
    ]);
    checkout.put_all(&branch);
    // `omega#contract` is the text it had; `sigma#contract` has no merge-base
    // file to equal.
    assert_eq!(untouched(&checkout), [CONTRACT]);
    // The neighbour: had `sigma` been on the default branch with the same text,
    // it would be owed too.
    checkout.commit("add sigma");
    checkout.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    assert_eq!(untouched(&checkout), [CONTRACT, "cap:sigma contract"]);
}

#[test]
fn a_section_missing_from_the_merge_base_file_counts_as_touched() {
    let _lock = serial();
    let full = base();
    let cut = without_usage(&full);
    let checkout = Checkout::new("untouched-section-missing", &cut);
    checkout.put(OMEGA, &full[OMEGA]);
    assert_eq!(untouched(&checkout), [CONTRACT]);
}

#[test]
fn the_default_branch_moving_on_after_the_fork_does_not_touch_the_section() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-moved-on", &base());
    // The default branch edits the contract after the branch forked.
    checkout.git(&["checkout", "-q", "main"]);
    checkout.put(
        OMEGA,
        &replaced(
            &base(),
            OMEGA,
            "Text for Contract of omega.",
            "Edited upstream.",
        )[OMEGA],
    );
    checkout.commit("upstream edit");
    checkout.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    checkout.git(&["checkout", "-q", "feature"]);
    // The merge base is the fork, where the section was as the branch has it.
    assert_eq!(untouched(&checkout), [CONTRACT, USAGE]);
}

#[test]
fn on_the_default_branch_itself_every_affected_section_is_owed_and_it_is_not_an_error() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-default", &base());
    checkout.git(&["checkout", "-q", "main"]);
    assert_eq!(untouched(&checkout), [CONTRACT, USAGE]);
    // Uncommitted work on the default branch is touched, as anywhere.
    checkout.put(
        OMEGA,
        &replaced(&base(), OMEGA, "Text for Contract of omega.", "Edited.")[OMEGA],
    );
    assert_eq!(untouched(&checkout), [USAGE]);
}

#[test]
fn a_linked_work_tree_compares_its_own_head() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-linked", &base());
    let linked = checkout.scratch().join("linked");
    checkout.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "linked-branch",
        linked.to_str().expect("a UTF-8 path"),
        "main",
    ]);
    assert!(linked.join(".git").is_file(), "a linked work tree");
    // A linked work tree is its own top.
    let lifecycle = Lifecycle::open(
        raf_workplan::exec::Programs::new(&linked)
            .discover_git()
            .with_cargo(checkout.scratch().join("no-such-cargo")),
    )
    .unwrap_or_else(|error| panic!("the linked work tree opens: {error}"));
    let lines = |lifecycle: &Lifecycle| {
        lifecycle
            .reconcile("shape")
            .expect("reconcile succeeds")
            .of_kind(ObligationKind::SectionUntouched)
            .map(|o| format!("{} {}", o.subject, o.section))
            .collect::<Vec<_>>()
    };
    assert!(lifecycle.prefix().is_top());
    assert_eq!(lines(&lifecycle), [CONTRACT, USAGE]);
    // An edit made in the linked work tree is seen there and not in the other.
    disk::put(
        &linked,
        OMEGA,
        &replaced(&base(), OMEGA, "Text for Usage of omega.", "Linked edit.")[OMEGA],
    );
    assert_eq!(lines(&lifecycle), [CONTRACT]);
    assert_eq!(untouched(&checkout), [CONTRACT, USAGE]);
}

#[test]
fn a_workspace_below_the_top_of_the_repository_is_compared_at_its_prefix() {
    let _lock = serial();
    // The neighbour at the top, and one and two levels down.
    for (below, prefix) in [("", ""), ("sub", "sub/"), ("services/api", "services/api/")] {
        let checkout = Checkout::below("untouched-below", &base(), below);
        assert_eq!(lifecycle(&checkout).prefix().as_str(), prefix);
        assert!(checkout.root().ends_with(below));
        // Unchanged since the merge base: owed. (A file read at the wrong path
        // would be absent there and so touched, and the list would be empty.)
        assert_eq!(untouched(&checkout), [CONTRACT, USAGE], "{below}");
        // Edited on the branch: not reported; its neighbour is.
        checkout.put(
            OMEGA,
            &replaced(&base(), OMEGA, "Text for Contract of omega.", "Edited.")[OMEGA],
        );
        checkout.commit("edit the contract");
        assert_eq!(untouched(&checkout), [USAGE], "{below}");
    }
}

#[test]
fn a_prefix_with_a_space_is_an_error_and_a_neighbour_without_one_is_not() {
    let _lock = serial();
    let spaced = Checkout::below("untouched-spaced", &base(), "my dir");
    let error = spaced
        .lifecycle()
        .expect_err("a prefix the pattern cannot carry");
    assert!(
        matches!(&error, LifecycleError::Git(GitError::Answer { answer, .. }) if answer == "my dir/"),
        "{error:?}"
    );
    let plain = Checkout::below("untouched-plain", &base(), "my-dir");
    assert_eq!(untouched(&plain), [CONTRACT, USAGE]);
}

#[test]
fn a_root_outside_any_work_tree_is_an_error_at_the_open() {
    let _lock = serial();
    let scratch = disk::Scratch::new("untouched-no-tree");
    disk::write_repo(scratch.path(), &base());
    let programs = raf_workplan::exec::Programs::new(scratch.path()).discover_git();
    let error = Lifecycle::open(programs).expect_err("not a work tree");
    assert!(
        matches!(&error, LifecycleError::Git(GitError::Failed { .. })),
        "{error:?}"
    );
}

#[test]
fn a_local_branch_named_origin_main_does_not_stand_in_for_the_remote_one() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-origin-main", &base());
    checkout.git(&["checkout", "-q", "-b", "origin/main"]);
    checkout.put(
        OMEGA,
        &replaced(&base(), OMEGA, "Text for Contract of omega.", "Local edit.")[OMEGA],
    );
    checkout.commit("edit on the local branch named origin/main");
    // The short name now means the local branch, whose tip is `HEAD`.
    assert_eq!(
        checkout.git(&["rev-parse", "origin/main"]),
        checkout.git(&["rev-parse", "HEAD"])
    );
    // Read as the local branch the merge base would be `HEAD` and the contract
    // would be owed; read as the remote-tracking branch it is touched.
    assert_eq!(untouched(&checkout), [USAGE]);
}

#[test]
fn a_default_branch_named_like_an_option_is_resolved_and_never_reaches_git_as_one() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-option", &base());
    checkout.git(&[
        "update-ref",
        "refs/remotes/origin/--output=x",
        "refs/remotes/origin/main",
    ]);
    checkout.git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/--output=x",
    ]);
    assert_eq!(untouched(&checkout), [CONTRACT, USAGE]);
    assert!(!checkout.root().join("x").exists());
    assert!(
        !std::env::current_dir()
            .expect("a working directory")
            .join("x")
            .exists()
    );
}

// --- the refusals, each with its passing neighbour ----------------------------

#[test]
fn no_default_branch_is_an_error_that_names_the_fix_and_the_fix_clears_it() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-no-head", &base());
    checkout.git(&[
        "remote",
        "add",
        "origin",
        "https://example.invalid/none.git",
    ]);
    checkout.git(&["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"]);
    let error = reconcile_error(&checkout);
    assert!(
        matches!(
            &error,
            LifecycleError::Git(GitError::NoDefaultBranch { .. })
        ),
        "{error:?}"
    );
    let text = error.to_string();
    assert!(text.contains("git remote set-head origin --auto"), "{text}");
    // The neighbour: restore the reference and the same command answers.
    checkout.git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/main",
    ]);
    assert_eq!(untouched(&checkout), [CONTRACT, USAGE]);
}

#[test]
fn a_repository_with_no_remote_at_all_is_an_error_and_not_a_pass() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-no-remote", &base());
    checkout.git(&["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"]);
    checkout.git(&["update-ref", "-d", "refs/remotes/origin/main"]);
    let error = reconcile_error(&checkout);
    assert!(
        matches!(
            &error,
            LifecycleError::Git(GitError::NoDefaultBranch { .. })
        ),
        "{error:?}"
    );
}

#[test]
fn a_default_branch_with_no_common_ancestor_is_an_error() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-unrelated", &base());
    checkout.git(&["checkout", "-q", "--orphan", "island"]);
    checkout.commit("an unrelated root");
    let error = reconcile_error(&checkout);
    assert!(
        matches!(&error, LifecycleError::Git(GitError::NoMergeBase { .. })),
        "{error:?}"
    );
}

/// A capability document for `omega` that is `size` bytes long.
fn omega_of_size(size: usize) -> String {
    let text = &base()[OMEGA];
    let filler = size - text.len();
    // One long line is as good as many, and keeps the document valid.
    replaced(
        &base(),
        OMEGA,
        "Text for Purpose of omega.\n",
        &format!(
            "{}\n",
            "a".repeat(filler + "Text for Purpose of omega.".len())
        ),
    )[OMEGA]
        .clone()
}

#[test]
fn a_file_over_one_mebibyte_at_the_merge_base_is_an_error_and_exactly_one_is_read() {
    let _lock = serial();
    let over = omega_of_size(MAX_TEXT_BYTES + 1);
    assert_eq!(over.len(), MAX_TEXT_BYTES + 1);
    let mut huge = base();
    huge.insert(OMEGA.to_string(), over);
    let checkout = Checkout::new("untouched-oversize", &huge);
    checkout.put_all(&base());
    let error = reconcile_error(&checkout);
    assert!(
        matches!(&error, LifecycleError::Git(GitError::BlobTooLarge { path, limit })
            if path.as_str() == OMEGA && *limit == MAX_TEXT_BYTES),
        "{error:?}"
    );

    // The neighbour: exactly the bound is read, parsed and compared; its
    // contract differs from the small file's, so it is touched.
    let exact = omega_of_size(MAX_TEXT_BYTES);
    assert_eq!(exact.len(), MAX_TEXT_BYTES);
    let mut at_bound = base();
    at_bound.insert(OMEGA.to_string(), exact);
    let checkout = Checkout::new("untouched-at-bound", &at_bound);
    checkout.put_all(&base());
    assert_eq!(untouched(&checkout), [CONTRACT, USAGE]);
}

#[test]
fn a_file_at_the_merge_base_that_does_not_parse_is_an_error_and_not_a_pass() {
    let _lock = serial();
    // A generated block that is never closed does not parse as a body.
    let broken = replaced(
        &base(),
        OMEGA,
        "Text for Contract of omega.\n",
        "Text for Contract of omega.\n\n<!-- raf:generated:verification-matrix -->\nnever closed\n",
    );
    assert!(
        Body::parse(&broken[OMEGA]).is_err(),
        "the fixture is broken"
    );
    let checkout = Checkout::new("untouched-unparsable", &broken);
    checkout.put_all(&base());
    let error = reconcile_error(&checkout);
    assert!(
        matches!(&error, LifecycleError::Document { path, side: Side::MergeBase, .. } if path == OMEGA),
        "{error:?}"
    );
    let text = error.to_string();
    assert!(
        text.contains(OMEGA) && text.contains("at the merge base"),
        "{text}"
    );

    // A file that is not UTF-8 is the same error.
    let mut bytes = base()[OMEGA].clone().into_bytes();
    bytes.extend_from_slice(&[0xff, 0xfe, b'\n']);
    let checkout = Checkout::new("untouched-not-utf8", &base());
    fs::write(checkout.root().join(OMEGA), &bytes).expect("a file");
    checkout.git(&["add", "-A"]);
    checkout.git(&["commit", "-q", "-m", "not UTF-8"]);
    checkout.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    checkout.put_all(&base());
    let error = reconcile_error(&checkout);
    assert!(
        matches!(&error, LifecycleError::Document { side: Side::MergeBase, reason, .. } if reason.contains("UTF-8")),
        "{error:?}"
    );
}

#[test]
fn an_affected_section_the_working_tree_file_lacks_is_an_error() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-working-lacks", &base());
    // `omega` loads only with all seven sections, so the load fails first: the
    // error is the capability that is not loaded and not a silent pass.
    checkout.put(OMEGA, &without_usage(&base())[OMEGA]);
    let error = reconcile_error(&checkout);
    assert!(matches!(&error, LifecycleError::Reconcile(_)), "{error:?}");
}

// --- contract_effect: none ----------------------------------------------------

#[test]
fn contract_effect_none_keeps_usage_and_examples_and_requires_a_default_branch() {
    let _lock = serial();
    // The complete source audit requires the merge base even when the Contract
    // is exempt from the section-untouched rule.
    let only_three = with_affects(
        &[
            "cap:omega#contract",
            "cap:omega#usage",
            "cap:omega#examples",
        ],
        "none",
    );
    let checkout = Checkout::new("untouched-none", &only_three);
    checkout.git(&["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"]);
    assert!(matches!(
        reconcile_error(&checkout),
        LifecycleError::Git(GitError::NoDefaultBranch { .. })
    ));
    checkout.git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/main",
    ]);
    assert_eq!(
        untouched(&checkout),
        ["cap:omega usage", "cap:omega examples"]
    );

    // The neighbour: the same change with `contract_effect: changes` needs the
    // merge base and fails without it.
    let changes = with_affects(
        &[
            "cap:omega#contract",
            "cap:omega#usage",
            "cap:omega#examples",
        ],
        "changes",
    );
    let checkout = Checkout::new("untouched-changes", &changes);
    checkout.git(&["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"]);
    assert!(matches!(
        reconcile_error(&checkout),
        LifecycleError::Git(GitError::NoDefaultBranch { .. })
    ));
}

fn without_usage(repo: &Repo) -> Repo {
    let mut cut = repo.clone();
    let text = cut.get_mut(OMEGA).expect("omega");
    let start = text.find("## Usage\n").expect("Usage");
    let end = start + text[start..].find("## Examples\n").expect("Examples");
    text.replace_range(start..end, "");
    cut
}

#[test]
fn contract_effect_none_still_reports_the_other_sections() {
    let _lock = serial();
    let repo = with_affects(
        &[
            "cap:omega#contract",
            "cap:omega#constraints",
            "cap:omega#verification",
        ],
        "none",
    );
    let checkout = Checkout::new("untouched-none-others", &repo);
    assert_eq!(
        untouched(&checkout),
        ["cap:omega constraints", "cap:omega verification"]
    );
}

// --- the version gate -----------------------------------------------------------

fn said(text: &str) -> Output {
    Output::new(Some(0), text.as_bytes().to_vec(), Vec::new())
}

#[test]
fn git_2_29_stops_the_open_before_any_other_process_and_2_30_goes_on() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-version", &base());
    let gate = Gate::workspace(checkout.root());

    let old = FakeRunner::new().on(
        Program::Git,
        &["--version"],
        Ok(said("git version 2.29.2\n")),
    );
    let error = Lifecycle::open_with(&gate, &old).expect_err("2.29 is refused");
    assert!(
        matches!(&error, LifecycleError::Git(GitError::VersionTooOld { .. })),
        "{error:?}"
    );
    assert!(error.to_string().contains("2.30"), "{error}");
    let calls: Vec<String> = old.calls().iter().map(Call::command).collect();
    assert_eq!(calls, ["git --version"], "nothing else was run");

    // The neighbour: 2.30 opens and lists the fixture's tracked paths through
    // the supplied runner.
    let tracked = checkout.git(&["ls-files", "-z"]);
    let new = FakeRunner::new()
        .on(
            Program::Git,
            &["--version"],
            Ok(said("git version 2.30.0\n")),
        )
        .on(
            Program::Git,
            &["rev-parse", "--show-prefix"],
            Ok(said("\n")),
        )
        .on(
            Program::Git,
            &["rev-parse", "--is-inside-work-tree"],
            Ok(said("true\n")),
        )
        .on(Program::Git, &["ls-files", "-z"], Ok(said(&tracked)));
    let lifecycle = Lifecycle::open_with(&gate, &new).expect("2.30 opens");
    let asked: Vec<String> = new.calls().iter().map(Call::command).collect();
    assert_eq!(
        asked,
        [
            "git --version",
            "git rev-parse --show-prefix",
            "git rev-parse --is-inside-work-tree",
            "git ls-files -z",
        ]
    );
    assert!(lifecycle.prefix().is_top());
    assert_eq!(lifecycle.opened().workplan().changes().len(), 1);

    // An unreadable version is an error too, and also stops the open.
    let odd = FakeRunner::new().on(Program::Git, &["--version"], Ok(said("not git\n")));
    assert!(matches!(
        Lifecycle::open_with(&gate, &odd),
        Err(LifecycleError::Git(GitError::VersionUnreadable { .. }))
    ));
    assert_eq!(odd.calls().len(), 1);
}

#[test]
fn an_unstartable_git_is_an_error_before_the_repository_is_looked_at() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-no-git", &base());
    let programs = raf_workplan::exec::Programs::new(checkout.root())
        .with_git(checkout.scratch().join("no-such-git"));
    let error = Lifecycle::open(programs).expect_err("no git");
    assert!(
        matches!(&error, LifecycleError::Git(GitError::Exec(_))),
        "{error:?}"
    );
}

// --- stubs that record what they were started with -------------------------------

#[cfg(unix)]
mod stubs {
    use super::*;
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    use std::path::PathBuf;

    use raf_workplan::exec::Programs;

    const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
    const MERGE_BASE: &str = "fedcba9876543210fedcba9876543210fedcba98";

    /// A `git` that records each call (argv elements separated by the unit
    /// separator, one call to a line, and the directory and locale it ran
    /// with) and answers the questions of a reconcile; `cat-file` prints the
    /// file `blob` beside it.
    fn recording_git(version: &str, default_ref: &str, prefix: &str) -> String {
        format!(
            r#"dir=$(dirname "$0")
printf '%s\037' "$@" >> "$dir/argv"
printf '\n' >> "$dir/argv"
pwd -P >> "$dir/cwd"
printf '%s\n' "${{LC_ALL-<unset>}}" >> "$dir/lc_all"
case "$1" in
--version) echo '{version}' ;;
rev-parse)
  case "$2" in
  --show-prefix) echo '{prefix}' ;;
  --is-inside-work-tree) echo true ;;
  *) echo {COMMIT} ;;
  esac ;;
ls-files) printf 'docs/capabilities/omega.md\0' ;;
symbolic-ref) echo '{default_ref}' ;;
merge-base) echo {MERGE_BASE} ;;
cat-file) cat "$dir/blob" ;;
*) echo "unexpected $*" >&2; exit 64 ;;
esac"#
        )
    }

    struct Stub {
        checkout: Checkout,
        git: PathBuf,
    }

    impl Stub {
        fn new(tag: &str, version: &str, default_ref: &str, blob: &str) -> Self {
            Self::with_prefix(tag, version, default_ref, "", blob)
        }

        /// A stub that says the root is `prefix` below the top of the work
        /// tree (`""`, or a path ending in `/`).
        fn with_prefix(
            tag: &str,
            version: &str,
            default_ref: &str,
            prefix: &str,
            blob: &str,
        ) -> Self {
            let checkout = Checkout::new(tag, &base());
            let bin = checkout.scratch().join("bin");
            fs::create_dir(&bin).expect("a bin directory");
            fs::write(bin.join("blob"), blob).expect("a blob");
            let git = bin.join("git");
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o755)
                .open(&git)
                .expect("a stub file");
            write!(
                file,
                "#!/bin/sh\n{}\n",
                recording_git(version, default_ref, prefix)
            )
            .expect("a stub body");
            drop(file);
            Self { checkout, git }
        }

        fn programs(&self) -> Programs {
            Programs::new(self.checkout.root()).with_git(&self.git)
        }

        fn lines(&self, name: &str) -> Vec<String> {
            let text = fs::read_to_string(self.git.with_file_name(name)).unwrap_or_default();
            text.lines().map(str::to_string).collect()
        }

        fn calls(&self) -> Vec<Vec<String>> {
            self.lines("argv")
                .iter()
                .map(|line| {
                    let mut parts: Vec<&str> = line.split('\u{1f}').collect();
                    assert_eq!(parts.pop(), Some(""), "the record ends in a separator");
                    parts.into_iter().map(str::to_string).collect()
                })
                .collect()
        }
    }

    /// The commands of a reconcile with a root `prefix` below the top, and
    /// the path the one read names.
    fn assert_argv(tag: &str, prefix: &str) {
        for hostile_prose in [false, true] {
            assert_argv_with_prose(tag, prefix, hostile_prose);
        }
    }

    fn assert_argv_with_prose(tag: &str, prefix: &str, hostile_prose: bool) {
        let _lock = serial();
        let stub = Stub::with_prefix(
            tag,
            "git version 2.45.1.windows.1",
            "refs/remotes/origin/--output=x",
            prefix,
            &base()[OMEGA],
        );
        if hostile_prose {
            let ordinary = &base()[OMEGA];
            let changed = ordinary.replace(
                "Text for Purpose of omega.",
                "Text for Purpose of omega. PROSE_PAYLOAD_$(touch prose-executed); `printf injected` | cat.",
            );
            assert_ne!(&changed, ordinary);
            stub.checkout.put(OMEGA, &changed);
        }
        let lifecycle =
            Lifecycle::open(stub.programs()).unwrap_or_else(|error| panic!("opens: {error}"));
        let list = lifecycle.reconcile("shape").expect("reconcile succeeds");
        // The blob is the file as it is, so both sections are unchanged.
        assert_eq!(list.of_kind(ObligationKind::SectionUntouched).count(), 2);

        let expected: Vec<Vec<String>> = [
            vec!["--version"],
            vec!["rev-parse", "--show-prefix"],
            vec!["rev-parse", "--is-inside-work-tree"],
            vec!["ls-files", "-z"],
            vec!["symbolic-ref", "refs/remotes/origin/HEAD"],
            vec![
                "rev-parse",
                "--verify",
                "--end-of-options",
                "refs/remotes/origin/--output=x^{commit}",
            ],
            vec!["merge-base", "HEAD", COMMIT],
            // One read for the file, though two of its sections are affected.
            vec![
                "cat-file",
                "blob",
                &format!("{MERGE_BASE}:{prefix}docs/capabilities/omega.md"),
            ],
        ]
        .into_iter()
        .map(|argv| argv.into_iter().map(str::to_string).collect())
        .collect();
        assert_eq!(stub.calls(), expected);
        assert!(!stub.checkout.root().join("prose-executed").exists());

        // The only words that begin with a dash are the fixed options.
        let fixed = [
            "--version",
            "--show-prefix",
            "--is-inside-work-tree",
            "-z",
            "--verify",
            "--end-of-options",
        ];
        for call in stub.calls() {
            for arg in call.iter().filter(|arg| arg.starts_with('-')) {
                assert!(fixed.contains(&arg.as_str()), "{arg} in {call:?}");
            }
        }

        // Every call ran in the root, in the C locale.
        let root = fs::canonicalize(stub.checkout.root()).expect("a canonical root");
        let cwd = root.to_str().expect("a UTF-8 path");
        assert_eq!(stub.lines("cwd").len(), expected.len());
        assert!(stub.lines("cwd").iter().all(|line| line == cwd));
        assert!(stub.lines("lc_all").iter().all(|line| line == "C"));
    }

    #[test]
    fn reconcile_asks_git_exactly_the_commands_of_the_contract_in_order() {
        assert_argv("untouched-stub-argv", "");
    }

    #[test]
    fn a_root_below_the_top_reads_each_file_at_the_prefix() {
        assert_argv("untouched-stub-prefix", "services/api/");
        assert_argv("untouched-stub-prefix-one", "sub/");
    }

    #[test]
    fn a_prefix_the_source_path_pattern_cannot_carry_is_an_error_and_not_a_guess() {
        let _lock = serial();
        for prefix in [
            "my docs/",
            "sub",
            "../up/",
            "-opts/",
            "caf\u{e9}/",
            "a//b/",
            "./a/",
        ] {
            let stub = Stub::with_prefix(
                "untouched-stub-bad-prefix",
                "git version 2.45.1",
                "refs/remotes/origin/main",
                prefix,
                &base()[OMEGA],
            );
            let error = Lifecycle::open(stub.programs()).expect_err(prefix);
            assert!(
                matches!(&error, LifecycleError::Git(GitError::Answer { .. })),
                "{prefix}: {error:?}"
            );
            // Nothing past the question was asked.
            assert_eq!(
                stub.calls(),
                [vec!["--version"], vec!["rev-parse", "--show-prefix"]],
                "{prefix}"
            );
        }
    }

    #[test]
    fn the_blob_the_stub_prints_is_what_is_compared() {
        let _lock = serial();
        let edited = replaced(
            &base(),
            OMEGA,
            "Text for Contract of omega.",
            "Another contract.",
        );
        let stub = Stub::new(
            "untouched-stub-blob",
            "git version 2.45.1",
            "refs/remotes/origin/main",
            &edited[OMEGA],
        );
        let list = Lifecycle::open(stub.programs())
            .expect("opens")
            .reconcile("shape")
            .expect("reconcile succeeds");
        let kinds: Vec<String> = list
            .of_kind(ObligationKind::SectionUntouched)
            .map(|o| o.section.to_string())
            .collect();
        assert_eq!(kinds, ["usage"]);
    }

    #[test]
    fn a_stub_git_2_29_runs_one_process_and_exits_with_the_error() {
        let _lock = serial();
        for (version, accepted) in [
            ("git version 2.29.9", false),
            ("git version 2.29.3 (Apple Git-130)", false),
            ("git version 2.30.0", true),
        ] {
            let stub = Stub::new(
                "untouched-stub-version",
                version,
                "refs/remotes/origin/main",
                &base()[OMEGA],
            );
            let result = Lifecycle::open(stub.programs());
            assert_eq!(result.is_ok(), accepted, "{version}");
            if !accepted {
                assert_eq!(stub.calls(), [["--version"]], "{version}: one process");
            }
        }
    }
}

// --- the digest -------------------------------------------------------------------

fn body(text: &str) -> Body {
    Body::parse(text).unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn the_digest_leaves_out_generated_blocks_on_both_sides_and_nothing_else() {
    let section = CapabilitySection::Contract;
    let digest = |text: &str| section_digest(&body(text), section);
    let with = |inside: &str| {
        format!(
            "# T\n\n## Contract\n\nIt reads.\n\n<!-- raf:generated:matrix -->\n{inside}<!-- raf:generated:end -->\n\nAfter.\n\n## Usage\n\nUse.\n"
        )
    };
    // The block's content does not matter, whether it is empty, other or long.
    assert_eq!(digest(&with("")), digest(&with("one row\n")));
    assert_eq!(digest(&with("x\n")), digest(&with("y\ny\ny\n")));
    // The text around it does, before and after.
    assert_ne!(
        digest(&with("")),
        digest(&with("").replace("It reads.", "It writes."))
    );
    assert_ne!(
        digest(&with("")),
        digest(&with("").replace("After.", "Later."))
    );
    // A marker is authored text: a block removed, or one of another kind.
    assert_ne!(
        digest(&with("")),
        digest("# T\n\n## Contract\n\nIt reads.\n\nAfter.\n\n## Usage\n\nUse.\n")
    );
    assert_ne!(
        digest(&with("")),
        digest(&with("").replace("matrix", "relationships"))
    );
    // The text of another section is not this section's.
    assert_eq!(
        digest(&with("")),
        digest(&with("").replace("Use.", "Use it differently."))
    );
    // A heading is part of the section.
    assert_ne!(
        digest("# T\n\n## Contract\n\nText.\n"),
        digest("# T\n\n## Contract (draft)\n\nText.\n")
    );
    // Fences and PROPOSED blocks stay.
    assert_ne!(
        digest("# T\n\n## Contract\n\nText.\n"),
        digest("# T\n\n## Contract\n\nText.\n\n```text\nfenced\n```\n")
    );
    assert_ne!(
        digest("# T\n\n## Contract\n\nText.\n"),
        digest("# T\n\n## Contract\n\nText.\n\n> [!PROPOSED change:x]\n> New.\n")
    );
    // No such section is `None`.
    assert_eq!(
        section_digest(
            &body("# T\n\n## Contract\n\nText.\n"),
            CapabilitySection::Usage
        ),
        None
    );
    // A block that sits in the neighbouring section does not leak.
    let other = |inside: &str| {
        format!(
            "# T\n\n## Contract\n\nText.\n\n## Usage\n\n<!-- raf:generated:matrix -->\n{inside}<!-- raf:generated:end -->\n"
        )
    };
    assert_eq!(digest(&other("a\n")), digest(&other("b\n")));
    assert_eq!(
        section_digest(&body(&other("a\n")), CapabilitySection::Usage),
        section_digest(&body(&other("b\n")), CapabilitySection::Usage)
    );
}

#[test]
fn line_endings_do_not_touch_a_section() {
    let lf = "# T\n\n## Contract\n\nLine one.\nLine two.\n\n## Usage\n\nUse.\n";
    let crlf = lf.replace('\n', "\r\n");
    for section in [CapabilitySection::Contract, CapabilitySection::Usage] {
        assert_eq!(
            section_digest(&body(lf), section),
            section_digest(&body(&crlf), section)
        );
    }
    assert_ne!(
        section_digest(&body(lf), CapabilitySection::Contract),
        section_digest(
            &body(&lf.replace("Line two.", "Line three.")),
            CapabilitySection::Contract
        )
    );
}

#[test]
fn a_checkout_with_converted_line_endings_is_untouched_in_every_section() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-crlf", &base());
    let converted = base()[OMEGA].replace('\n', "\r\n");
    fs::write(checkout.root().join(OMEGA), converted).expect("a file");
    assert_eq!(untouched(&checkout), [CONTRACT, USAGE]);
}

// --- nothing is written, and equal input prints equal output -------------------------

#[test]
fn reconcile_writes_nothing_and_prints_the_same_list_twice() {
    let _lock = serial();
    let checkout = Checkout::new("untouched-readonly", &base());
    checkout.put(
        OMEGA,
        &replaced(&base(), OMEGA, "Text for Usage of omega.", "Edited usage.")[OMEGA],
    );
    let work_tree = |root: &Path| {
        let mut files = snapshot(root);
        files.retain(|path, _| !path.starts_with(".git/"));
        files
    };
    let before = work_tree(checkout.root());
    let lifecycle = lifecycle(&checkout);
    let first = lifecycle.reconcile("shape").expect("first run");
    let second = lifecycle.reconcile("shape").expect("second run");
    assert_eq!(first.to_string(), second.to_string());
    assert!(!first.is_empty());
    assert_eq!(work_tree(checkout.root()), before);
}
