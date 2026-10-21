VERDICT: BLOCKED

# T3 review: units by source identity and Census::load (commit 05d7be1)

## Scope
Commit 05d7be1 (crates/raf-census: src/census.rs, src/units/{mod,scan,sig}.rs, tests/units.rs, lib.rs,
normalize/mod.rs re-exports) against plan Rev 2 > Rev 1 > original: Units by source identity, Token sources,
Fail hard, Entry point, Shim candidate / shim_key, span-locations, Group and file context, max_file_bytes,
Shim candidate's body print, Entry-point fixture; rows R7, R18. The worktree was not edited; probes ran in a
clone at `.stage/raf-census-core-normal-form-v1/scratch/t3review` (probe test file only, not committed). No
timing runs, no mutation passes.

## Commands
* `cargo test -p raf-census --locked`: exit 0 (17 unit, 19 forms, 4 golden, 17 units, 1 doctest).
* `make clippy`: exit 0 (`-D warnings`). It prints two cargo `unused dependency` warnings (`serde`,
  `serde_json`) for crates/raf-census/Cargo.toml; these are T1 dependencies not yet used (reports are T5), not
  a T3 defect, but T5 must use them or drop them.
* Probe clone, three probes, exit 0: (1) 60 consts, 60 statics and 60 fns in one file, every constant and
  string line asserted (proc-macro2 byte ranges are file-relative across many parses: correct); (2) `Census::load`
  of this repository's `crates` via DiskTree twice, 6323 units, symbols/files/lines/shim keys/body fingerprints
  identical across runs (deterministic), 540 keyed, 184 `via_self`, 260 trait-impl, no string line before its
  unit line; (3) edge probes listed below.

## What was verified (no defect)
* One unit per source item through `items_by_source` (file, offset); symbol = shipping reach first, then
  smallest path, a total order, so deterministic; `shipping` is any reach; fixtures for lib+bin `mod shared;` and
  the support module shared by two test binaries; units come in file/offset order as documented.
* Kinds: Fn / inherent, impl, default methods / Const, AssocConst / Static; others yield no unit (tested).
* Parameters come from the parsed signature (pattern bindings, types not visited); constants and statics read the
  initialiser only (type naming `0x10000` is not a constant); `0xff`/`255`/`1_000` are one value; raw multi-line
  string gets its starting line, the next string the right absolute line.
* Fail hard: an unparsable signature or initialiser is `Error::Source` (sig.rs tests with valid neighbours); a span
  that is not text of the file is `Error::Source` (`text`, mod.rs:141); a literal that does not decode is an error;
  an unlexable body is an error from the tower (forms.rs). An unlexable body cannot be produced through
  `Census::load` because raf-source lexes first and reports `Error::Load`; coverage at the layer that can fail is
  adequate.
* `max_file_bytes` reaches `LoadOptions`; above-cap file is `Error::Load` naming it, at-cap loads. `min_tokens`
  19/20 at default and 10, and `Error::Options` for 0, are tested. `Unit.print.body` is kept for a shim candidate
  (Rev 2 body-print item) and `shim_key` is a separate field.
* shim_key exclusions with neighbours: trait-impl `make` has a shim and no key while `S::build`, `S::plain`, the
  trait default `dflt` share one key; `Self::inner(x,1)` and `Self::inner::<u8>(x,1)` have no key; `helper(x,
  Self::ONE)` keeps its key.

## Adjudications
(a) Turbofish `f::<T>(x)`: DEFECT, fix now (finding 1). `shim::has_call` requires an identifier or `!` directly
before a paren group; in `helper::<u8>(x, 1)` the paren follows `>`, so the 11-token forwarder has `shim:
Some(false)` (probe) although the page says a shim is a body with "at least one call". `x.collect::<Vec<_>>()` is
pinned in golden.rs:102-103 with no shim for the same reason. Normal form 1 is unreleased, so correcting the
predicate now costs a golden regeneration; after release it is a `normal_form` bump invalidating every ledger. The
plan's "ported exactly" is not a fingerprint-compatibility promise with purrdf (no such requirement in the plan),
and it ports a bug. The T3 code is correct relative to T2; the fix lands in T2's predicate with T3 fixtures.

(b) `Self::` rule: ACCEPT the callee-path rule as implemented (page: "forwarders that call through `Self::`"; the
only reading under which `helper(x, Self::ONE)`, a path with no call, is not excluded). Probe shows it applies at
any depth: `helper(x, Self::f(1))`, `Self::f(helper(x, 1))` and the variant constructor `Self::V(x, 1)` are all
`via_self` with no key, whereas `Vec::<u8>::new(x, 1)` keeps one. That behaviour is reasonable (a `Self::` call
anywhere ties the body to its type) but is untested and the page must say it (finding 2).

(c) `trait_impl`, `via_self`, `params` on `Unit`: ACCEPT. `shim_key` is still computed once in `collect` (`build`,
mod.rs:233-237) as `Option<Fingerprint>`; the extra fields are plain facts on a `#[non_exhaustive]` struct, are
needed to assert the exclusions by fixture, and cannot disagree with `shim_key` because it is derived from them
in one place. No change required.

## Findings
1. crates/raf-census/src/normalize/shim.rs:32 (`has_call`), tests/support/fixtures.rs:288 and tests/golden.rs:102-103,
   tests/units.rs:552-555. A call whose callee ends in a turbofish (`f::<T>(x)`, `x.m::<T>()`) is not recognised, so
   such a forwarder has no shim form and can never be shim-grouped, and a `Self::inner::<T>(..)` forwarder is
   untestable for its shim. Consequence: forwarders silently ungrouped, and fixing it after release is a
   `normal_form` bump. Required fix: make `has_call` treat `::` followed by a balanced `<...>` after a name as part
   of the name (a name is still followed by the paren group), add shim fixtures (`helper::<u8>(x, 1)` is a
   candidate, has a shim, and shim-groups with its renamed twin; `Self::inner::<u8>(x, 1)` has a shim and no key),
   regenerate the affected goldens in golden.rs (the `x.collect::<Vec<_>>()` pair now has a shim), tighten the
   comment at tests/units.rs:552-553 ("whatever its shim form is") into an assertion that the shim exists, and
   update the page's shim paragraph only if its wording needs it.
2. crates/raf-census/tests/units.rs:457-558 (`shim_keys_are_absent...`). The any-depth behaviour of the `Self::`
   rule is untested. Required fix: add fixtures pinning `helper(x, Self::f(1))` (via_self, no key),
   `Self::f(helper(x, 1))` (via_self, no key) and `Self::Variant(x, 1)` (via_self, no key), and have T6 state the
   rule in the page as "a call, at any depth, whose callee path starts with `Self::`" (a bare `Self::CONST` is not a
   call and keeps the key).
3. crates/raf-census/src/units/mod.rs:198-199. `whole.get(range).unwrap_or_default()` and `.map_or(0, ..)` turn an
   out-of-range initialiser range into an empty slice, so a constant would silently lose its constants and strings;
   likewise mod.rs:173-174 `saturating_sub(1).max(..)` hides a body span without braces. The plan says a bad slice
   is `Error::Source`, no fallback. These cannot fire today (probe 1 passes) but are exactly the silent skips the
   design forbids. Required fix: use the existing `text()` helper (or `ok_or_else` to `Error::Source`) for the
   initialiser range and the newline count, and check `end_byte >= start_byte + 2` for the body, failing with
   `Error::Source` naming file and line.
4. crates/raf-census/src/units/scan.rs:42-55. Only `$str` and `$bytes` literals are collected, and a byte string
   that is not UTF-8 yields `None` and is dropped: `b"\xff\xfe"` and `c"hi"` appear in neither `strings` nor the
   table detection (probe). Plan R7 says "string literals". Required fix: either collect C strings (`$cstr`,
   decoded via `syn::Lit::CStr`) and document that a non-UTF-8 byte string carries no text and is not a string
   literal, or record it with a lossy decoding; in both cases a fixture beside a valid neighbour pins the choice
   so the omission is a stated rule rather than a silent drop.
5. (Observation, no required change) `const _` items yield no unit (raf-source classes them `UnnamedConst`, outside
   the plan's four kinds) and nested `const`/`fn` items inside a body are folded into the enclosing unit's
   constants. Both match the plan; T6 may state them. `Census::unit` and `file` are linear scans; T4 should index
   if it calls them per unit.

## Verdict
BLOCKED on findings 1 to 4 (all small; 1 touches T2's predicate and goldens). Everything else in T3 meets its
requirements; no silent skip of a body, item or span was found beyond items 3 and 4.
