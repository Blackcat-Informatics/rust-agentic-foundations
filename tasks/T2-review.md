VERDICT: BLOCKED

# T2 review: normaliser and fingerprints (commit 08071b8, parent cb08b0c)

## Scope

Read in full: `crates/raf-census/src/normalize/{mod,form,passes,literal,shim,fingerprint,words}.rs`,
`tests/{forms,golden}.rs`, `tests/support/fixtures.rs` (pairs and registry), the `lib.rs` diff,
the T2 report, plan.md (original, Rev 1, Rev 2), `docs/architecture/normal-form.md` (current
text, T6 owns its rewrite), and the port source `purrdf/.../normalize.rs` (threshold and
numbering code). Behaviour probed in a clone at `.stage/.../scratch/t2review` (a throwaway
`tests/probe.rs`; the worktree was not edited). No timing runs, no mutation passes.

## Commands

| Command | Exit |
|---|---|
| `cargo test -p raf-census --locked` (12 unit, 19 forms, 4 golden, 1 doctest) | 0 |
| `cargo run --locked -q -p raf-census --example goldens` | 0 (prints the table, 177 lines) |
| `make clippy` (`-D warnings`) | 0; two `cargo::unused_dependencies` warnings (`serde`, `serde_json`) from T1 dependencies that T5 uses; not clippy failures |
| probe: `cargo test -p raf-census --locked --test probe -- --nocapture` in the scratch clone | 0 (output cited in finding 1) |

## Adjudication of the five deviations

1. **Thresholds count structural tokens before `erase_held`: ACCEPT.** The plan's tower names the
   level "structural" (after `kind_literals`, before `erase_held`) and Rev 2 defines the tier as
   "< 30 structural tokens"; the decision is a function of the literal-tier form, so it is
   monotone. It does differ numerically from the port, where `structural.len()` is already
   erased (`&`, `.clone()`, `let` annotations do not count there). The pinned token counts and
   the 5/6/29/30 and 25/29/30/35 boundaries are consistent with it. T6 carry-forward: the page
   must say thresholds count the structural level, before the value-form erasures, and that
   this is not the port's count.
2. **Locals numbered after the highest parameter number the body uses: ACCEPT, conditional on
   finding 1.** With `params.len()`, `f(a,b){let x=a;x}` gives `let $2=$0;$2` while
   `g(a){let x=a;x}` gives `let $1=$0;$1`, yet their shim forms are identical (`let x = $0; x`),
   so equal-at-shim would not imply equal-at-literal-tier: the monotonicity the plan requires
   would break. Starting at "highest used + 1" makes the literal-tier form a pure function of the
   shim form, which is what monotonicity needs. Probe: `f(a,b){let x=a;x}` vs `g(a){let x=a;x}`
   are equal from Verbatim (a miss that is wanted: an unused trailing parameter is not
   behaviour); `f(a,b){let x=b;x}` vs `g(a){let x=a;x}` stay distinct (`$2=$1` vs `$1=$0`).
   `$` cannot start a user identifier, so params and locals cannot alias by name. The one real
   collision this scheme admits is finding 1. The reading differs from Rev 2's "one counter"
   wording only when trailing parameters are unused; T6 must word the page as "locals are
   numbered after the highest parameter number the body uses".
3. **Names inside erased tokens are not numbered (`$_`): ACCEPT.** The shim form is hashed before
   `rename_locals`, and the value form removes those tokens, so no fingerprint sees `$_`; it only
   appears in the unhashed literal-tier and structural levels shown by `explain`. The
   fixture `value_erased_names_do_not_shift_numbers` pins the intent.
4. **Integers respelled to decimal at lex time: ACCEPT.** It is the only way `0xff`/`255` and
   `1_000`/`1000` are equal below the 30-token tier (R4 and the literal-spellings bullet), and it
   is applied to the very first level, so monotonicity is trivial. Suffix is kept, floats are
   untouched, beyond-`u128` keeps digits without underscores (`literal.rs:22-69`; hex digits
   cannot collide with a suffix since no suffix letter is a hex digit). The level named
   "verbatim" is therefore "as lexed, integers in decimal"; mod.rs docs say so; the page must
   too (T6 carry-forward).
5. **Kept `_`, `gen/union/raw/safe/macro_rules`; range end is a literal; names after `.` or in
   field position not renamed: ACCEPT.** `_` renamed to `$n` (as the port did) would equate a
   wildcard with a binding; fixture `lexical_wildcard_kept` is Distinct. The weak keywords are
   mandated by Rev 1. `..1` as a literal (`after_dot` requires a non-joint `.` not preceded by a
   joint `.`) is covered by `tier_30_range_end_is_a_literal`; `p.0` kept by
   `tier_30_tuple_index_kept`. Observation, not a finding (port parity): a struct shorthand
   `S { x }` is in field position, so a parameter used as shorthand is not renamed
   (`S { x }.x + x` gives `S { x }.x + $0`); two functions differing only in that parameter's
   name are therefore not equal. A missed collapse, never a false match.

## Checks asked for

* Fixtures from the plan: all present and assert the right relation in `forms.rs`
  `every_registered_pair_compares_as_registered` (Equal(level) pins the finest equal level,
  Distinct pins `equal_from == None` and `differs_through == Value`). Present: R2 collapses and
  differs (locals, params, lifetimes, function, method, field, type, local type, tuple field);
  R3 one pair per erasure incl. all 11 adapters (loop over `VALUE_ADAPTERS`), `&mut`, deref,
  `ref`, `ref mut`, `x[..]`, `x[1..]`/`x[..2]` kept, let annotation (plain and generic),
  turbofish, const and bool turbofish kept, `m[&k]`, binary `&`, `*`, `&&`, compound assign;
  R4 25/29/30/35 plus kind, float, big-int, suffix, tuple-index, range-end neighbours; R5
  5/6/29/30 plus let, two statements, trailing `;`, `self`, no-call, macro call, own-parameters
  only, callee, fixed argument, parameter order; spellings `0xff/255`, `1_000/1000`,
  `0o17,0b1010`, suffix kept, value and suffix differ; inline captures and named arguments below
  and above the tier, unknown name, `{0}`, `{{`, `\u{..}`; lexical edges (five words, wildcard,
  macro_rules definition, nesting 128/129 in `forms.rs`). Trait-impl and `Self::` shim
  exclusions are T3/T4 as planned.
* Goldens: `every_fixture_fingerprint_is_pinned` fails on an unpinned fixture, an orphan pin, a
  moved value and a duplicated count (checked against the real table). The header states the
  bump rule. See finding 2 for the weak companion test.
* Monotonicity covers every pair: `the_tower_is_monotone_over_every_registry_pair` iterates all
  `n*(n-1)/2` fixture pairs (not only registered ones), checks token counts never grow per
  fixture, and asserts more than 40 equal pairs so it is not vacuous. Sound.
* `explain()` tested: equal, renamed local (level and first token with index), different
  literal (line 41 vs 71 from `first_line`, byte range slice), unequal lengths.
* Encoding matches the plan: u32 LE length + bytes per token (`fingerprint.rs:19-28`), BLAKE3,
  first 8 bytes via `u64::from_be_bytes`, `{:016x}` (`fingerprint.rs:126`); a hand-built byte
  string is hashed independently and compared to the digest and to the `body:` text; no hex
  table. A joined punctuation char hashes as `c~` (`form.rs:185`); the page must state it (T6).
* Fail hard: unlexable slice, over 128 nesting and over 2 GiB are `Error::Source`; each test has
  its valid neighbour (`f(1);g("terminated")`, 128 deep). No fallback paths found.
* No katamari code: seven small modules, each one concern; the largest, `passes.rs` (596
  lines), is the four passes and their helpers.
* `Fingerprint` enum, parse/display round trip, refusals (unknown kind, no kind, 15/17 digits,
  uppercase, `+` sign, unknown table, empty table, empty rule, rule with a space): complete.
* Shim candidate keeps its body print and joins only via the shim print: tested
  (`a_shim_candidate_keeps_its_body_print...`, `shim_forms_exist...`).

## Findings

1. **False collision: a parameter used only through an inline format placeholder shares a
   number with a local.** `crates/raf-census/src/normalize/passes.rs:257-285` (`rename_locals`
   starts at `next_number`, which counts only identifier leaves) versus `passes.rs:218-220`
   (`rename_params` rewrites `{a}` inside a macro string to `{$0}`, a place `next_number` never
   reads). Probe (scratch clone), `params = ["a"]`:
   `let x = 1; println!("{a} {x}")` gives literal-tier/value `let $0 = 1 ; println ! ( "{$0} {$0}" )`,
   identical to `let x = 1; println!("{x} {x}")`. The two bodies compute different things (one
   prints the parameter) and get the same `body:` fingerprint, so the census would group them
   as copies. Same defect with `format!("{a}{z}")`. It also contradicts the plan's
   "parameters first, then locals, no aliasing" requirement and the tower's purpose (a false
   group is the worse failure than a missed one).
   Required fix: make the first local number follow every parameter number actually used,
   including those only in rewritten placeholders (e.g. have `rename_params` return the highest
   number it produced alongside the form, or scan macro-string placeholders for `{$N` in
   `next_number`; whichever is chosen must keep the start a pure function of the shim form so
   monotonicity holds). Add a Distinct pair to the registry
   (`(["a"], "let x = 1; println!(\"{a} {x}\")")` vs `(["a"], "let x = 1; println!(\"{x} {x}\")")`),
   one more above the 30-token tier, a pair showing the fixed numbering is still monotone
   (`{a}` only vs `a` as an argument gives the same value form where they are the same
   algorithm, and the unused-trailing-parameter pair of finding 3), and regenerate the goldens
   with the example. The version is still the unreleased 1, so no bump is needed.

2. **The golden self-test proves nothing about the real check.**
   `crates/raf-census/tests/golden.rs:264-274`
   (`an_unpinned_fixture_and_an_orphan_pin_are_detected`) builds forged maps and asserts
   set-difference results on its own local expressions; it never runs the code that
   `every_fixture_fingerprint_is_pinned` (`golden.rs:230-262`) uses, so deleting that test's
   assertions would leave this one green. Consequence: the "goldens fail on unpinned or orphan
   entries" claim is demonstrated only by the happy path.
   Required fix: factor the comparison into one function (e.g. `pin_problems(computed, pinned)
   -> Vec<String>` in `tests/support`) used by the real test, and have the negative test call
   it with a forged unpinned fixture, a forged orphan, a moved value and a duplicated line,
   each expecting a problem naming the id, with the matching neighbour expecting none.

3. **Deviation 2 has no fixture.** `tests/forms.rs:270-282`
   (`parameters_are_numbered_in_signature_order_before_locals`) pins the numbers but no registry
   pair states the consequence the deviation exists for. Required fix: add to the registry
   `f(a,b){let x=a;x}` vs `g(a){let x=a;x}` (`Equal(Verbatim)`) and `f(a,b){let x=b;x}` vs
   `g(a){let x=a;x}` (`Distinct`), so the monotonicity test and the pins cover the rule and a
   future switch to `params.len()` fails a named pair.

## T6 carry-forward (page wording; not T2 defects)

* Thresholds count the structural level, before `erase_held` (differs from the port).
* Locals are numbered after the highest parameter number the body uses (not "one counter" over
  all parameters).
* "Verbatim" means as lexed with integers in decimal; a joined punctuation character hashes as
  `c~`; `$_` appears only in unhashed levels.
* The T2 report says "about 190 fixtures, ~95 pairs"; the pinned table has 177 fixtures. Correct
  it when the report is next touched.

## Summary

Encoding, fingerprint type, tower, monotonicity, explain, fixtures and fail-hard behaviour are
sound and all four checks run clean. Deviations 1 and 3 to 5 are accepted outright and 2 is
accepted once finding 1 is fixed. Finding 1 is a real false-collision defect in numbering, so the
task is blocked until findings 1 to 3 are fixed and the goldens regenerated.
