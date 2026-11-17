<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Delivery review of d0e6b7bd (census: harden and document the foreign host import detector refusals)

Written 2026-11-16T17:18-06:00. Range `origin/main...HEAD` (c485447d..d0e6b7bd).

## Scope

I checked the issue text and its seven items against `crates/raf-census/src/detectors/host_import.rs`,
`crates/raf-census/src/gate/rules.rs`, `crates/raf-census/tests/census_gate.rs`,
`docs/capabilities/census-detectors.md` and the renewed assessments. I did not rely on the author's
summary. I checked upstream wasm-bindgen behaviour against its source. I fetched
`crates/macro-support/src/parser.rs` at main, 0.2.130, 0.2.128, 0.2.125, 0.2.120, 0.2.115, 0.2.110,
0.2.105, 0.2.100, 0.2.92 and 0.2.87 with `gh api` into `/tmp/wbg-review/`, outside the repository.

## Evidence

| Command (from the worktree, under `flock /tmp/raf-build.lock`) | Exit | Scope |
|---|---|---|
| `cargo test -p raf-census --locked --test census_gate foreign_host_import` (log `reviews/r-test.log`) | 0 | 4 tests ran and passed, 86 filtered out, including the new `foreign_host_import_refuses_what_it_cannot_place_beside_a_valid_neighbour` |
| `cargo clippy -p raf-census --all-targets --locked -- -D warnings` (log `reviews/r-clippy.log`) | 101 | `clippy::too_many_lines` (102/100) at `crates/raf-census/tests/census_gate.rs:3435` |
| `prose-check docs/capabilities/census-detectors.md` | 0 | whole document |

The CI clippy job (`.github/workflows/ci.yaml:141`) and `make clippy` (`Makefile:53`) both run
`cargo clippy --workspace --all-targets --locked -- -D warnings`. That command includes this target,
so the failure blocks CI. The implementer's `.stage/.../clippy.log` shows a clean run, which does
not match the committed head.

### Upstream facts, from source

- `js_namespace` on the block and the item: from 0.2.87 through 0.2.125 the item's value replaces the
  block's (`item_opts.js_namespace()...or(ctx.js_namespace)`). In 0.2.128 and 0.2.130 the
  combination is refused with "`js_namespace` cannot be set on both an `extern` block and an item
  inside it; write the full path on one of them" (0.2.130 `parser.rs:3206`). No release merges the
  two values or applies the block's value.
- `getter` with `setter`: 0.2.100 `operation_kind` lets the later check win, so the setter wins
  silently. 0.2.130 `pick_operation_kind` picks the higher rank (setter) and emits a deprecation
  warning, "`getter` conflicts with `setter` and is ignored". A repeated `getter` reads the first
  value. The repeat is unused, and `(getter, false, ..)` makes that a warning, not an error. Upstream
  accepts both forms.

### Item-by-item

1. **Block and item `js_namespace`.** Code fix: refused at `host_import.rs:392-397`. The test has a
   negative case and a block-only neighbour (`census_gate.rs`, case 8), plus block-only and
   item-only acceptance at the end of the test. The refusal matches upstream 0.2.128 and later.
   `static_method_of` does not set `namespace_written` (`host_import.rs:219,232-234`), which matches
   upstream, where `static_method_of` is not `js_namespace`. The stated reason is wrong (finding R2).
2. **Macro in an `extern` block.** Refused at `host_import.rs:425-432`. Documented at
   `census-detectors.md:227-228` and `:371-372`, and tested with a neighbour. The message is
   actionable. The refusal propagates through `gate/rules.rs:164-169` and fails the check.
3. **Undocumented refusals.** Each one is now listed (`census-detectors.md:215-228`) and tested with
   a corrected neighbour: getter with setter, repeated getter, a `method` with no typed parameter, a
   `method` receiver that is not a named type, a `constructor` without a return type, a
   `constructor` return type that is not a named type (`-> ()`), and a path-qualified key. The
   repeated-getter message is fixed (`host_import.rs:246-252`). The stricter-than-upstream point
   is O1.
4. **`link` with `cfg(..)`.** `host_import.rs:170-178` reads past the parenthesized predicate. A
   parenthesized list on any other key now gets a clean refusal ("does not take a parenthesized
   list") instead of syn's raw "expected `,`". Tests: `cfg(target_os = "linux")` is accepted with one
   copy, `kind(static)` and `catch(x)` are refused, `cfg = true` is refused through `value_text`, and
   a bare `cfg` or `cfg = ".."` now reaches the unplaceable-argument refusal because `cfg` left the
   neutral arm (`host_import.rs:231`).
5. **Gate failure scope.** A `Refusal` becomes `refused(..)` and returns with `?` from the whole
   check (`gate/rules.rs:164-169`). The tests call `Gate::workspace(".").check_tree(..).err()`. The
   wording "the Gate check fails" at `census-detectors.md:195,215,331,448` matches the code.
6. **Purpose.** Rewritten at `census-detectors.md:44-47` to cover host import declarations.
7. **Wrapping and the hard phrase.** Only partly fixed (finding R3).

## Findings

**R1. REQUIRED: the CI clippy gate fails on the new test.**
`crates/raf-census/tests/census_gate.rs:3435`
`foreign_host_import_refuses_what_it_cannot_place_beside_a_valid_neighbour` is 102 lines, and the
limit is 100. `clippy::too_many_lines` comes from the workspace `pedantic` level and is denied by
`-D warnings` (`r-clippy.log`, exit 101). Fix the test, for example by moving the case table into a
`const` or a helper. Do not add an `allow`. This is a code change, so it needs another clippy run and
a read of the changed test.

**R2. REQUIRED (wording only): the stated reason for the `js_namespace` refusal is wrong about upstream.**
`docs/capabilities/census-detectors.md:225-226` says "the binding attribute releases differ on which
of the two applies". The comment at `host_import.rs:390-391` says "which wins differs between
releases". The source shows otherwise. Releases up to 0.2.125 apply the item's value, and 0.2.128
and later refuse the combination. No release applies the block's value. Issue item 1 asks for the
upstream behaviour to be "checked against its source". Correct both texts, for example: "current
releases of the binding attribute refuse it, and earlier ones let the item's value replace the
block's". The refusal itself can stay. The refusal message already matches upstream.

**R3. REQUIRED (wording only): issue item 7 is not fully fixed, and one new cross-reference is wrong.**
- The `foreign-host-import` Verification bullet still wraps unevenly. Lines 446, 447, 448 and 452 of
  `census-detectors.md` are 109, 127, 94 and 142 columns, while the rest of the bullet wraps near 80.
  The new text went onto those long lines. The Constraints bullet at `:368-369` is 89 columns.
- The Verification bullet at `:448-449` says "Each refusal listed in the usage section". The list is
  at `:215-228`, inside `## Contract` (`:56-237`). `## Usage` starts at `:238`.
- The hard phrase in this bullet is unchanged at `:446-447`: "Two spellings of one property, by
  `js_name` and by the attribute's own value, match, and two `js_name` values on one declared name
  do not". The issue does not name the phrase, so I cannot confirm this is the one meant.
  Rewrap the bullet, fix the section name, and recheck the phrase. These are doc-only changes.

**O1. OPTIONAL: getter and setter refusals are stricter than upstream, and the document gives no reason.**
`census-detectors.md:222` refuses "both a `getter` and a `setter`, or repeats either". Upstream
accepts both forms. The setter wins, and a repeated getter reads its first value, with a warning in
current releases. These refusals already existed before this change: the old code refused any second
property argument. The issue asks for them to be documented and tested, which is done. It does not
ask for them to be relaxed. Unlike the `js_namespace` entry, this entry gives no reason. A short
reason would help, for example: "the binding attribute ignores one of them with a warning".

**O2. OPTIONAL: arguments inside `cfg_attr` are read whatever the predicate.**
`host_import.rs:147-149` reads every `cfg_attr` body. Mutually exclusive alternatives would therefore
be refused, such as `cfg_attr(a, wasm_bindgen(getter))` with `cfg_attr(not(a), wasm_bindgen(setter))`,
or `js_namespace` on block and item under exclusive predicates, even though upstream sees only one of
them. "Attributes inside `cfg_attr` are read" (`:210`) implies this but does not say it.

**O3. OPTIONAL: the `js_namespace` refusal could place the import instead.**
No release applies the block's value when both are written. Code that compiles with both therefore
targets a release where the item's value wins, so the detector could take the item's value instead
of refusing. Refusal matches current upstream and is a valid fix for item 1, so this is optional.

## Not verified

- I built no probe crate. The cases in O2 and the `cfg` forms beyond the tested ones are reasoned
  from the code, not run.
- I did not run the commit-hook binaries (`docs_contract`, `self_workplan`, `self_census`,
  `repo_gates`), so I have not checked that the renewed assessments are current. I did not run
  fmt, reuse or the issue-refs scan.
- I did not check upstream behaviour for options the detector refuses as unplaceable, such as
  `this` and `reexport`. Those refusals existed before this change and are not in the issue.

VERDICT: BLOCKED
