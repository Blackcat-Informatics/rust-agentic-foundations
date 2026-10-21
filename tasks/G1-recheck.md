VERDICT: PASS

# G1 recheck: fix commit 31858dd

Scratch clone `$S/scratch/c1review` at 31858dd, target dir `/tmp/g1t`. Worktree not edited.

| Command | Exit |
|---|---|
| `cargo test -p raf-census --locked` | 0 (22, 23, 4, 23, 7, 11, 21, 1 doctest) |
| `cargo test --locked --test docs_contract --test self_workplan --test repo_gates` | 0 (28, 68, 1) |

* F1 verified. The law is qualified to slices with one parameter list in the page (`normal-form.md:79-83`), `normalize/mod.rs:19-22`, the `forms.rs` test comment and the capability. `Divergence::equal_from` is now suffix-based (`rev().take_while(same).last()`) and its doc says so. The new test `explain_reports_reordered_parameters_as_equal_from_the_literal_tier` asserts equal_from LiteralTier and differs_through Shim for the reordered pair, and passes. No fingerprint moves (goldens unchanged except the two new pairs).
* F2 verified. `binder_at_binding_match_arm` and `binder_at_binding_if_let` are registered Equal(LiteralTier), pinned in `golden.rs`, listed in the capability's binder sentence; registry, golden and monotone tests pass.
* F3 verified. The CHANGELOG paragraph is rewrapped. The capability shim bullet is partly rewrapped; one line is still a little long (cosmetic, not blocking).
* Assessments: six `census-normal-form-*` assessments were re-accepted in the commit; `docs_contract` and `self_workplan` pass, so their digests are current.
* Advisory (unchanged): F4 whole-body binding of a shadowing name is a documented limit.
