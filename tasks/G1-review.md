VERDICT: BLOCKED

# G1 review: C1 (033574c), normaliser corrections

Two small findings block: the normative monotone law is stated without the condition it needs (F1), and one binder the page promises has no pinned pair (F2). GAP-01 to GAP-04, GAP-07 and the audit probes F1-F3 are closed on the real entry points. No behaviour change is required.

## Scope

* Commit 033574c (parent be93118), read from a scratch clone at `$S/scratch/c1review` (detached at 033574c, `CARGO_TARGET_DIR=/tmp/g1t`, outside `$S/scratch`). The worktree was not edited. The probe files I copied into the clone were deleted before the gates.
* Read in full: `normalize/binders.rs`, `normalize/passes.rs`, `normalize/mod.rs` diff, the page diff, the capability diff, the CONTRIBUTING/AGENTS/CHANGELOG diffs, the registry (`tests/support/fixtures.rs`) diff, the new `forms.rs` tests, and the assessment diffs (digest-only).
* Not run, by instruction: timing runs, mutation passes. `make check` and hosted CI are not part of this review.

## Commands and exit status

| Command (in the clone) | Exit | Result |
|---|---|---|
| `cargo test -p raf-census --locked` (run twice) | 0 | 22 unit, 22 forms, 4 golden, 23 groups, 7 page, 11 reports, 21 units, 1 doctest |
| `cargo test --locked --test docs_contract --test self_workplan --test repo_gates` | 0 | 28, 1, 68 passed |
| `make clippy` (`cargo clippy --workspace --all-targets --locked -- -D warnings`) | 0 | clean |
| `cargo run --locked -q -p raf-census --example goldens` | 0 | runs; `golden.rs` passes |
| audit probe file `audit_probe.rs`, tests p01 p02 p03 p13 p15 | 0 | see "Gap closure" |
| gap-analysis GAP-04 probe `zz_gap_probe.rs` (the 2 `Tower::structural_tokens` calls renamed to `value_tokens`) | 0 | see "Gap closure" |
| my own probe `g1_probe.rs` (23 binder, shadowing and head-block cases) | 0 | see "Binder coverage" |

The repo's cargo wrapper did not exit 1: my target directory was outside `$S/scratch`.

## Gap closure, on the real entry points

* **F1 / GAP-01 closed.** p01 through `Census::load`: `first` (`map(double).filter(is_small)`) and `second` (`map(triple).filter(is_big)`) now have different shim keys (`shim:a1e20b14c3b17620`, `shim:0d1287d75a2925e8`). Their value forms keep `double`/`triple`. `explain` gives `equal_from=None`. The closure variants `third` and `fourth` get different body fingerprints. p02: `ua` (reads `limit_a`) and `ub` (reads `limit_b`) get different fingerprints (`body:6412f4e7b8fa1dff`, `body:d46d129364dda51a`) and form no group.
* **F3 / GAP-02 closed.** p15 through `Census::load`: `clamp_a` and `clamp_b` (`if next == MAX { next }` with other local names) are one group, `body:be566e518b54f917`. `clamp_c` (different) is outside it. `clamp_d` and `clamp_e` are the other group. p03 `explain` gives `equal_from=LiteralTier`. A real struct literal keeps its field names (`a_real_struct_literal_keeps_its_field_names`, `item_in_struct_shorthand_field`).
* **F2 / GAP-03 closed.** p13 through `Census::load`: `count_a(xs, k)`, `count_b(k, xs)` and `count_c` are one 35-token group, `body:c60a766884a10224`. The forwarder pair with swapped arguments has different shim prints and different shim keys (`forms.rs`, `groups.rs`). The page Decision reads "renamed by order of first appearance" at `docs/architecture/normal-form.md:28`.
* **GAP-04 closed.** The gap-analysis probe: the clean body (21 tokens) and the body with `.clone().as_slice().to_owned()` both report `value_tokens()=21`, have identical value forms, and `explain` gives `equal_from=Some(Value)`. The `min_tokens` probe gives 15 and 15 and the same body-print state at 10, 15, 16 and 20. `held_len` counts without building the form. It equals the length of the value form for every registered fixture (`every_threshold_counts_the_value_form`). Literals are `Kind::Literal` before and after `kind_literals`, and the erasure mask reads only kinds, so the pre-literal count is exact. The decision depends only on the literal-tier form, so equal literal-tier forms make equal threshold decisions.
* **GAP-07 closed.** CONTRIBUTING.md:23-28 and AGENTS.md:237-240 carry one rule, in the same words: a version is fixed by the content of its first merge to the default branch, and after that any change that can move a fingerprint is a new version with a migration path. The page (Version-bump rule, Golden-test rule, Decision) and the capability say the same. CLAUDE.md is untouched, as decided. The added text has no issue numbers, no gap or audit references, and no history wording ("previously", "restored", "no longer").

## Binder coverage per form (registry pair, `Equal(LiteralTier)`)

`let`, `let` tuple, `let` struct, closure parameter, `for`, `match` arm, `if let`, `while let`, label, lifetime, local `fn`, local `struct`, call to a bound closure, shadowing an item, macro metavariable (and, from the truncated part of the diff, named macro argument). All are registered and pass the registry check. Distinct pairs: function passed by name, predicate passed by name, lowercase static, `S { x }` against `S { y }`.

My additional probes were all as expected:

* equal: typed closure parameters, `|&(a, b)|`, `move |a|`, `let ... else`, `for (i, a)`, `match` guards, or-patterns, a binary `|` (not a closure), an `else if` chain after `MAX`/`MIN`, a closure inside an `if` head, a struct literal inside a `while` body, labelled lifetimes, named macro arguments, `name @ pattern`, `Some(n @ 1..=5)`;
* different: lowercase statics, and local `const K`/`const J` (an uppercase local `const` is an item name and is kept, which is consistent with the stated rule).

## Adjudication

**(a) Byte-identical `count_a`/`count_b`.** The tower is not non-monotone for reordered-parameter pairs whose texts differ. `shim_parameter_order` and `numbering_parameter_order` have different texts and the law holds for them. Signature-order numbering at the shim cannot be dropped, because `f(y, x)` and `f(x, y)` must stay different shims. The real fact is this: `Verbatim` is a function of the text, and `Shim` is a function of the text and the parameter list. For slices with the same parameter list the law holds. For two slices with the same text and different parameter lists (reordered or not) it can fail: equal at verbatim, different at the shim, equal again from the literal tier. This also held before C1 (`numbering_unused_trailing_parameter` is a related shape). C1 makes it reachable by a case the issue requires to group. Keeping the pair out of the registry is sound, because the test would otherwise fail on a true statement. It is not sound to leave the law unqualified, which is what C1 did (F1). `explain` on this pair returns `equal_from=Some(Verbatim)` with `differs_through=Some(Shim)`, which a consumer reads as a contradiction. Required fix in F1.

**(b) `shim_parameter_order` is Equal(LiteralTier), shim prints asserted different.** Correct. The registry check holds, `forms.rs` `copies_with_reordered_parameters_share_a_body_print_and_forwarders_keep_their_order` asserts `in_order.body == swapped.body` and `in_order.shim != swapped.shim` (both `Some`), and `groups.rs` asserts different shim keys through `Census::load`. The golden shows the same `body:` and different `shim:` values (`golden.rs:226-227`).

**(c) Registry level changes.** All justified by the rules and not loosened:

* `shim_only_own_params_renamed` is Distinct: `first`/`second` are unbound, so they are items (matches R5 "only the function's own parameters renamed").
* `numbering_used_trailing_parameter` is Equal(LiteralTier): `b` and `a` are both `$0` by first appearance.
* `numbering_placeholder_parameter_above_tier` is Equal(Structural): the format-string literal is abstracted above the tier, and both sides number `x` as `$0`.
* `lexical_nesting_128` is Equal(Shim) with parameters `left`/`right`: the names are now unbound unless parameters, so the pair needs them.
* `shim_method_turbofish` is `x.convert::<u8>(1, 2)` with 8 value tokens: the old body had 5 value tokens and lost its shim form (see d).

**(d) Forwarders losing their shim form under value counting.** An acceptable consequence, not a regression of R5. The issue says "6-30 tokens" without naming the count, and the plan fixed it as the value form (GAP-04, with the user decision and the historical-fit F5.1). `target(x.clone())` has the value form `target ( x )`, the same as `target(x)`, and both are under the six-token floor whose purpose is that `{ f(x) }` must not group. The rule gives equal decisions to equal value forms. `forms.rs` pins the bounds (`a.f()` and `a.f().clone().as_slice()` have no shim; `shim_29` with adapters keeps it). The page states "Noise the value form erases never moves a body across a threshold".

**(e) `name @ pattern`.** The behaviour is right (all `@` probes above are equal under renaming; the binder is the identifier in the pattern, `@` is ignored). It works because `pattern_level` binds any lowercase identifier and not by a rule of its own, and no fixture, test or registry pair mentions `@`. The page (`normal-form.md`, rename_locals list) and the capability claim it. See F2.

## Findings

1. **F1 (blocking): the monotone law is stated without its condition, and `explain` reports an incoherent pair.**
   * Files: `docs/architecture/normal-form.md:79-80` ("Two bodies that are equal at one level are equal at every coarser level"), `crates/raf-census/src/normalize/mod.rs:19-20` (same sentence), `crates/raf-census/src/normalize/mod.rs:249-258` (`Divergence::equal_from`: "The finest level at which the two forms are equal"), `docs/capabilities/census-normal-form.md:665-666`, `crates/raf-census/tests/forms.rs:90-92` (test comment).
   * Consequence: the normative page and the module doc promise a law that the delivered `count_a`/`count_b` (same text, parameter lists in another order) contradicts, and `Normalizer::explain` gives `equal_from=Verbatim` and `differs_through=Shim` for it. Downstream consumers (the CLI `explain`, the rule API, ledger messages) will print the contradiction. The law is also what the registry test claims to enforce, so the exemption lives only in the implementer's report.
   * Required fix: (i) state the law as: for two slices with the same parameter list, equality at a finer level implies equality at every coarser level; the verbatim form is the text only, so two slices with the same text and different parameter lists can differ at `shim` and are equal again from the `literal-tier`; (ii) make `Divergence::equal_from` the finest level from which every coarser level is equal (suffix-based), so the pair reads `equal_from=LiteralTier`, `differs_through=Shim`, and update its doc; (iii) add one `forms.rs` test with `count_a`/`count_b` through `Normalizer::explain` asserting those two values and equal body prints, and name the same-parameter-list condition in the test comment at `forms.rs:90`. Changing `equal_from` does not move a fingerprint, so there is no goldens or normal-form impact; check that the registry check `every_registered_pair_compares_as_registered` and `explain_names_the_levels_and_the_first_differing_token_with_its_line` still hold.

2. **F2 (blocking, small): the `name @ pattern` binder has no pinned pair.**
   * Files: `crates/raf-census/tests/support/fixtures.rs` (`binders_more`), `crates/raf-census/src/normalize/binders.rs` (no `@` handling), `docs/architecture/normal-form.md` (rename_locals list) and `docs/capabilities/census-normal-form.md:263-265`.
   * Consequence: the documented rule (`name @ pattern` binds) holds only by accident of `pattern_level`. A later change that filters pattern identifiers differently would break it with no failing test, against "every rule has a negative fixture" and the plan's one-pair-per-binder-form rule.
   * Required fix: register `binder_at_binding` pairs, each `Equal(LiteralTier)`: `match x { n @ 1..=5 => g(n), _ => 0 }` against `m @ 1..=5 ... g(m)`, and `if let Some(n @ 1..=5) = x { g(n) } else { 0 }` against the `m` version; add their goldens through `examples/goldens` and list the pair in the capability's Verification sentence if it enumerates forms (it does: "one pair for each form of binder").

3. **F3 (advisory): formatting of two added prose lines.** `CHANGELOG.md:15` and `docs/capabilities/census-normal-form.md` (the shim-candidate bullet, "its shim form renames its own parameters in signature order. `MIN_SHIM_TOKENS` is 6 and") are one long line each, where the surrounding text is wrapped near 90 columns. Rewrap when fixing F1 and F2 (the capability change needs `docs sync` run twice with no change).

4. **F4 (advisory, no fix required in C1): a name bound anywhere in a body is renamed everywhere in it.** `let len = len(x); g(len)` and `let n = len(x); g(n)` therefore differ (`$0 ( $1 )` against `len ( $1 )`). This is the rule the plan chose ("a name that shadows an item", scope-free because the body is a token tree), and the page and the Constraints state it ("A name is bound for the whole body, not for one scope"). Duplicates that shadow an item by their own binding are missed; the failure is a missed group, never a false one. Record it as a known limit if the maintainer wants it tracked; it does not need a change here.

## Documents and assessments

* Page: Decision restored to "renamed by order of first appearance" (`normal-form.md:27-29`); the rename rule, numbering, field names kept, head block, threshold count and version rule are stated; `tests/page.rs` pins the renaming/numbering rules and the value-form count (both pass). The Decision and the Structural-form sentence ("differ only in the names they bind, and in the order of their parameters, have the same structural form") are true for slices with the same text up to renaming; F1 only concerns the monotone sentence.
* CONTRIBUTING.md and AGENTS.md: one rule, same words, no history narration.
* Assessments: purpose, contract, examples, constraints and verification changed digest-only; `docs_contract` and `self_workplan` pass, and the capability Examples test (`the_capability_examples_hold_the_example_output`) passes. F1 and F2 touch the capability's Contract/Verification text, so the assessments must be re-read and re-accepted again after those edits (the report's acceptance digests will change).
