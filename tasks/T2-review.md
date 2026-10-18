# T2 review (rule 1 in the settling)

VERDICT: PASS

Two non-blocking findings (N1, N2) and two notes. Nothing in the worktree was modified; scratch clones under /tmp only.

## (a) Fixtures encode rustc and fail on the old code
* Each `use_resolution.rs` fixture's expected answers were re-derived by compiling the same crates with rustc (edition 2021, `--emit=metadata`) and probing each read in its module and name space: single re-export, the neighbours (`pub` glob, `pub(crate)` glob), the two-glob chain, both `n` variants with `pub(crate)` and private globs, and the `s0/s1/m2` private-item case. Every accept/reject agrees with the asserted "found"/"not found"; the item identity is determined by the program in each case.
* Scratch clone of HEAD plus only the new test files and the two new corpus files: the four new `use_resolution` tests FAIL on the old code; the `rustc_oracle` default tests FAIL there too (rule1 not in KNOWN, 1 rule1 + 3 rule2). On the new code all pass.
* N1: the `exported_items.rs` fixture (`an_item_reached_only_by_a_pub_use_of_a_private_glob_binding_is_not_exported_in_every_order`) PASSES on the old code, so it does not fail-first. exports.rs judges each link by its declared `Vis::Public`, and the private glob link already excludes the struct, so rule 1 does not reach the surface through this shape. The fixture is a correct regression and neighbour test, and the one-line exports.rs change is only the `view_ids` signature adaptation. Either accept it as such or find a shape where settled visibility, not declared, decides the surface (none found by inspection). Non-blocking.

## (b) Every-order determinism
`reads_in_every_order` permutes the top-level blocks (root modules) and, as the product, the declarations inside every listed module; the load counts asserted (36, 864, 144, 12) match the factorial products (3!x3!, 4!x4!... etc.), so the permutations are actually exercised. `surface_in_every_order` does the same for exported_items (36 loads). Sound.

## (c) Oracle classifier
* AcceptedButRefused: previously `when_visible` => Rule1; now `when_visible && ambiguous_in_a_module` => Rule2, else Unclassified. The comment's reasoning (rule 1 only hides, so it cannot make the index refuse a name) is correct and it is the right direction: a refusal with no ambiguity is no longer silently called rule 1.
* Permissive branch: `_ if ambiguous_in_a_module() => Rule2` before the Rule1 default.
* Risk: `ambiguous_in_a_module` asks about EVERY declared/bound name of the program in EVERY module with a `use`, not the probed name. A genuine over-cap bug (index too strict or too permissive on rule 1) in a program that also contains any unrelated ambiguity is therefore labelled rule2 and hidden in KNOWN. It is transient: Task 3 removes Rule2 from KNOWN and Task 4 requires zero divergences, so it cannot survive to completion, but the Task 2 claim "rule1 101 -> 0" is weaker than it reads.
* N2 (stricter classification is feasible): I built the narrowing in a scratch copy (names = the probed name only, instead of all names) and ran 1500 generated programs in release: identical result to the broad predicate (rule1 0, rule2 35, unclassified 1). Recommend adopting it (delete the `names` collection except the probe name); it costs nothing on this sample and removes the unrelated-ambiguity loophole. A tighter form (the ambiguity must be in a module on the probe's own resolution path) is possible but was not measured.
* Note: debug builds of the oracle at 800 programs trip a settled-tables consistency `assert!` (settle.rs, "the imports of `probeN` bound {..} and reach {}") on 3 programs; the same assert fires on HEAD (4 programs), so it is pre-existing and probably the import-cycle shapes the implementer reported as unclassified. Release runs complete. Task 4 must run in release or fix that.

## (d) Corpus lifecycle
The old `private_glob_binding_reexported.rs` still diverges (index finds A ambiguous in m3, rustc accepts), so under the plan's rule ("delete when the task fixing its rule lands") it correctly stays; relabelling it rule2 and renaming it by shape (`ambiguity_hidden_by_a_private_glob_in_a_chain`) is consistent and corrects a Task 1 misclassification. The two new entries are `agree` replay inputs (named re-export of a private glob binding; private item re-exported by a child glob), as the plan's "regression inputs that now agree" describes. The plan's phrase "the oracle corpus entry for rule 1 now agrees" is met by those two, not by the renamed one; worth one sentence in the Task 5 Verification text.

## (e) Gates
`cargo fmt --check` clean; `cargo clippy -p raf-source --all-targets --locked -- -D warnings` clean; `cargo test -p raf-source --locked` (debug) all green, every test binary.

## (f) Repo rules
No Instant/sleep/timeouts, no issue/PR numbers or process words in added lines (grepped the diff). Test sources keep their SPDX headers (unchanged files). Corpus files carry no SPDX header, as the existing ones do not (they are replay inputs). No attribution text.

## (g) Doc comments
glob_scope module doc, `Level`, `Held`, `class_for`, `allowed`, `wider/meet`, `vis_at`, `import_vis` and `glob_origin` docs match the code. One garbled sentence in a test comment (`use_resolution.rs`, first fixture: "The module `m` is a struct and a function of one name apart"); rewrite it. Non-blocking.
