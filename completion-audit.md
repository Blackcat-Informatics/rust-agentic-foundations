# Completion audit (Stage 2): raf-census core, normal form 1

VERDICT: FINDINGS-OPEN

## Scope, sources and limits

* Branch `paudley/3-raf-census-core-normal-form-v1` at `be93118`, compared with `origin/main`. I
  audited a scratch clone at `$S/scratch/audit` checked out at `be93118`. The worktree was not
  edited, and `git status` is clean.
* Inputs: the issue (`$S/issue.md`), the ledger R1-R18 (`$S/issue-analysis.md`), the plan
  (`$S/plan.md`, Rev 2 authoritative), the validation index, the T1-T6 reports, reviews and
  rechecks, AGENTS.md, CLAUDE.md, `docs/architecture/normal-form.md` (on the branch and on
  `origin/main`) and `docs/capabilities/census-normal-form.md`. The prior art is purrdf
  `crates/helper-census/src/normalize.rs`, read only to compare behaviour.
* No user scope decisions exist for this issue. Both `.deficiencies` files hold only the notice
  and the marker, with no entries.
* Entry points exercised: `raf_census::Census::load` over `raf_source::MemoryTree` workspaces, plus
  `Normalizer::{tower, print, explain}`, `Fingerprint::from_str`, `check_normal_form`, `match_jobs`,
  `baseline` and `index` through the public API. I also ran the bound `baseline` example (built-in
  fixture, relative `crates` and absolute `crates`) and the `goldens` example. The probes are in
  `$S/scratch/audit/crates/raf-census/tests/audit_probe.rs` (scratch only).
* Not run: the full `make check`, which the validation index lists as pending after Stage 2, and
  hosted CI. No timing runs and no mutation passes.

## Commands run (scratch clone, `CARGO_TARGET_DIR` local)

| Command | Exit | Observation |
|---|---|---|
| `cargo test -p raf-census --locked` | 0 | 99 passed (8 suites) |
| `cargo run --locked -q -p raf-census --example baseline` twice + `cmp` | 0 / 0 | byte-identical. Body group `body:6bb022e241a79a28` (alpha, beta), shim group `shim:203616cb54a3e53d` (one, two), `hex-lower` in `demo::digits`. Job `sum`: alpha home, beta copy, BIG copy, `copies: 2` |
| `... --example baseline -- crates` twice, and `-- $PWD/crates` | 0 / 0 / 0 | all three byte-identical (2,016,982 bytes): 6423 units, 59 body groups, 30 shim groups, 1 hex table, 7 shared constants |
| `cargo run --locked -q -p raf-census --example goldens` | 0 | its output equals the pinned `MANIFEST` (9 lines) and `TABLE` (199 lines, 106 fixtures) in `tests/golden.rs` exactly |
| `cargo run --locked -q -p cargo-workplan -- --root . check` | 0 | clean (run with my probe file moved out) |
| `cargo test --locked --test docs_contract --test self_workplan --test repo_gates` | 0 | 28 / 68 / 1 passed |
| `cargo test --locked -p raf-census --test audit_probe -- --nocapture` (P01-P15) | 0 | observations below |

## Criteria

| R | Requirement | Demonstration | Observation | Status |
|---|---|---|---|---|
| R1 | proc_macro2, bodies re-lexed, macro arguments normalised | P08 `explain`: locals renamed inside `println!`, `assert_eq!`, `vec!`. Separately `foo(a)` vs `bar(x)` inside `assert_eq!`. Suite test `renamed_locals_inside_macro_arguments_land_in_one_body_group` via `Census::load` | Renamed-locals pair: same `body:a2be76e833282143`. Different callee: different fingerprints | demonstrated |
| R2 | structural form: keywords/punct/delims kept; **names of items the body uses kept**; locals, parameters, lifetimes → `$0..` **by first appearance**; literals → kind | P01, P02, P03/P15, P12, P13 via `Census::load` and `explain` | Lifetimes and labels are renamed (P12). However: item names used as values are renamed and collapse (F1). Parameters are numbered in signature order, not by first appearance (F2). A local in a block after a capitalised name is not renamed (F3) | **failing** (F1, F2, F3) |
| R3 | value form erasures | P10 (borrow, `&mut`, `ref`, `.clone()`, `.as_str()`, `.to_owned()`, `x[..]`, `let` annotation) via `explain`; binary `&` kept; goldens `value_*` incl. turbofish, `m[&k]` | Each erasure gives the same value form and body fingerprint as the base. Binary `&` differs | demonstrated |
| R4 | literal tier < 30 | P11: an 18-token `match` differing in strings vs a 33-token one | Below 30, literals are kept and the pair differs. At 33 they are abstracted and the pair is equal | demonstrated |
| R5 | shim 6-29, only own params renamed, `shim_key` excludes trait-impl and `Self::` | P05 via `Census::load` | `s1`, `s2` and inherent `S::s6` share one shim group. `s3` (fixed argument 8) differs. `Self::helper` forwarders and the trait-impl `go`/`run` have a shim but no `shim_key` and no group. The `let` forwarder has no shim | demonstrated |
| R6 | `<kind>:` + hex of BLAKE3[..8]; kinds body, shim, table, rule | P14: the fingerprint recomputed by hand with `blake3::hash` over the documented encoding; parse round-trip and refusals | By hand `body:6bb022e241a79a28` equals the census value. Kind `blob`, 15 digits, uppercase hex, `table:hex-mixed` and `rule:` are refused | demonstrated |
| R7 | units fn/method/const/static with symbol, package, file, line, shipping, print, integer constants by value, tables, strings | P06, P07 via `Census::load` (lib, bin, test target, example, `#[cfg(test)]` module) | Name line is used past doc comments and attributes. Shipping: lib and bin true; test target, example and `cfg(test)` module false. `0x10000`, `65_536` and `0o200000u64` are one value. Strings carry lines. Constants and statics have no print, which matches the page and the purrdf prior art | demonstrated |
| R8 | BTreeMap grouping, ≥2, count desc / tokens desc / fingerprint asc | P09 via `Census::load`; `group.rs` keyed `BTreeMap<(GroupKind, Fingerprint), _>` | The 3-member group sorts before the 2-member one. Groups of one are dropped | demonstrated |
| R9 | hex tables (string, byte string, arrays, both cases); shared constants > 0xffff in ≥ 2 packages | P06 (`[char;16]` upper, byte string lower, `[u8;16]` byte literals lower), the built-in str lower; P07 | All reported with the right case. `0x10000` is shared across 3 packages under 3 spellings. `0xffff` is not | demonstrated |
| R10 | baseline JSON and index JSON, byte-deterministic | baseline example (above); `reports` suite (round-trip, `deny_unknown_fields`, required keys) | The keys match the issue: baseline `min_tokens`, `units`, `body_groups`, `shim_groups`, `hex_tables`, `shared_constants`; index `units` (symbols), `groups`, `jobs` | demonstrated |
| R11 | `normal-form.md` normative v1 | read the page against the code; `tests/page.rs` passes | Constants, pass ids, adapters and report members agree with the code. The page's renaming rules are not what the code does: "renames every other local name" vs F1, and "Two bodies that differ only in the names of their parameters and locals have the same structural form" vs F3. Its Decision text was changed from "by order of first appearance" (F2) | **failing** (F1-F3 page parts) |
| R12 | goldens pin every fixture; comment requires a `normal_form` bump | `tests/golden.rs` header; `goldens` example output equals the pins; the bijection test exists and passes | Every fixture is pinned. The header states the bump rule | demonstrated |
| R13 | fixtures: renamed locals, ref/adapter noise and literal spellings collapse; different called items do not | the registry fixtures `structural_renamed_*`, `value_*`, `literal_hex_and_decimal` / `literal_octal_binary` / `literal_separators`, `structural_different_function/method/field/type` | The fixture claims hold as written. No fixture covers an item used as a value, and such a fixture would fail (F1) | demonstrated for the listed fixtures; F1 adds the missing negative fixture |
| R14 | running twice gives identical bytes | example ×2 in separate processes; relative vs absolute; P09 fresh loads | identical | demonstrated |
| R15 | `cap:census-normal-form` reconciled | `workplan check` exit 0; capability `state: delivered`; change record completed; reports test pins the Examples to the example output | consistent. It must be re-reconciled if F1-F3 move any Example value | demonstrated (at this head) |
| R16 | fresh versioned normal form 1 | P14 `check_normal_form(0/1/2)`; `NORMAL_FORM == 1` | 0 and 2 are refused, naming both versions and `cargo census migrate`. 1 passes | demonstrated |
| R17 | capability delivered | as R15 | | demonstrated |
| R18 | depends on raf-source | `crates/raf-census/Cargo.toml` path dependency; every load goes through `raf_source::Workspace::load_with` | | demonstrated |

## Open findings

### F1: item names used as values are renamed as locals, so different algorithms share a body fingerprint (R2, R11, R13)

* **Where:** `crates/raf-census/src/normalize/passes.rs:73-83` (`names_an_item` keeps a name only
  when it is a path segment, is followed by `(` or `!`, is capitalised or is a primitive), and
  `passes.rs:378-403` (`Namer::ident` numbers every other identifier). The page
  `docs/architecture/normal-form.md:91-93` says `rename_locals` renames "every other local name:
  bindings, closure parameters, lifetimes, labels and the types a body declares", which a function
  item is not.
* **Demonstration (P01, `Census::load`):** `first` uses `xs.iter().copied().map(double).filter(is_small).collect::<Vec<u8>>()`
  and `second` uses `...map(triple).filter(is_big)...`. Both get `body:44af1080ba7e6a41` and form
  one body group. Their value forms are both `$0 . iter ( ) . map ( $1 ) . filter ( $2 ) . collect ( )`.
  P02: two bodies that read different lowercase statics (`limit_a`, `limit_b`) also form one group
  (`body:678e330afa99401c`).
* **Consequence:** the issue requires "names of items the body uses kept". Passing a function item
  by name (`map(f)`, `filter(pred)`, `and_then(parse)`, `unwrap_or_else(default_x)`) is a common
  idiom. Bodies that apply different functions are grouped as duplicates, and a ledger job that
  forbids such a fingerprint would match unrelated code. This is the false-positive grouping the
  page's "Conditions to revisit" warns against, and the normative text does not state it.
* **Required fix:**
  * Rename only the names a body binds: parameters, and the names introduced by `let`, closure
    parameters, `for`, `match` arm, `if let` and `while let` patterns, plus labels, lifetimes and
    local type declarations.
  * Keep every other identifier verbatim as an item name.
  * Add fixture pairs, each with a valid neighbour: `map(double)` vs `map(triple)` must differ, and
    the same bodies with renamed closure parameters must still collapse. Pin them.
  * Regenerate the golden table in the same change, and state the rule on the page.

### F2: parameters are numbered in signature order, not by first appearance as the issue and the original decision require (R2, R11)

* **Where:** `passes.rs:223-236` (`rename_params` numbers `$0..` in signature order) and
  `passes.rs:268-283` (`rename_locals` continues after the highest parameter number). The tower
  carries these numbers into the structural and value forms (`normalize/mod.rs:160-163`).
* **Page change:** `docs/architecture/normal-form.md:87-95` (the branch) replaced the decision on
  `origin/main` ("Local names are renamed by order of first appearance"; "Local bindings and the
  function's own parameters are renamed by order of first appearance"). The issue says "locals,
  parameters and lifetimes become `$0..` by first appearance".
* **Who decided:** the plan's final reviewer (`reviews/plan-review.md:46-51`, Rev 2 "Tower (A3)").
  No user decision covers it.
* **Demonstration (P13, `Census::load`):** `count_a(xs, k)` and `count_b(k, xs)` have byte-identical
  bodies but get `body:c368d176a00bed5c` and `body:f888bed35a9f7532`, so they are not grouped.
  `count_c` (renamed `ys`, `j`) groups with `count_a`. `explain(count_a, count_b)` reports
  `equal_from: Verbatim`, `differs_through: Value`.
* **Consequence:** copies that reorder their parameters, a frequent divergence between copied
  helpers, are missed.
* **Required fix:** one of the following.
  * Number parameters and locals together by first appearance in the structural and value forms.
    The shim level may keep signature order, since the issue only says the shim form renames the
    function's own parameters. Restore the page's decision text, add a fixture pair with swapped
    parameter order that collapses at the value level, and regenerate the pins.
  * Or obtain and quote the user's explicit decision that signature order replaces the issue's
    rule, accepting the plan reviewer's `b / a` vs `a / b` trade-off.

  An agent's judgment cannot close this finding.

### F3: a local in a block that follows a capitalised name is kept verbatim (R2, R11)

* **Where:** `passes.rs:98-107` (`opens_fields` treats any `{..}` after a capitalised identifier as
  a struct literal or pattern), together with `passes.rs:88-96` (`names_a_field` then keeps a name
  at the start of that block). Both `params_level` (`:244-249`) and `Namer::level` (`:361-367`) use
  it.
* **Demonstration (P03, P15, `Census::load`):** `clamp_a` (`if next == MAX { next } else { fallback }`)
  and `clamp_b` (the same with `later`, `other`, `input`, `delta`) get different fingerprints. The
  value form keeps `{ total }` / `{ sum }` verbatim. The neighbour pair `clamp_d` and `clamp_e`,
  which compare with `limit()` instead of `MAX`, does group (`body:75385651b28ffd69`).
* **Consequence:** this contradicts the page's normative claim (`normal-form.md:110-111`) and R2's
  "locals ... become `$0..`". Duplicates are missed whenever an `if`/`while`/`match` head ends in a
  capitalised constant and the block holds a single name.
* **Required fix:** do not open field position for the block of an `if`, `while`, `match` or
  `for .. in` head, where Rust admits no struct literal. Add a fixture pair with a non-grouping
  neighbour and pin it.

### Version rule for the fixes

F1 to F3 move pinned values that exist only on this unmerged branch. `origin/main` carries no
normal-form implementation and no pinned fingerprint. Whether regenerating them before the first
merge needs a `NORMAL_FORM` bump is the repository owner's call under the CLAUDE.md rule. Either
way, the pins and the capability Examples must be regenerated in the same change.

## Scope findings

* No scope cut and nothing laundered. Job-match classification moved into this change and is
  built here. `change:census-ledger` already held "A normal_form mismatch fails before any job is
  read" on `origin/main`.
* No DARK, REFUSED, PRODUCER-WITHOUT-CONSUMER, DUPLICATE, WEAKENED-TEST or VACUOUS-OUTPUT shape
  found.
  * The library's real caller in this change is the bound `baseline` example; census-cli is a
    separate tracked consumer.
  * The JSON goldens and the Examples carry real groups, tables, constants and jobs.
  * The golden pins equal the `goldens` example output.
* Silent narrowing: F1 and F2 narrow R2 as written, and F2 also rewrote the governing decision text
  without a user decision.

## Remaining work for PASS

1. Fix F1, F2 (or quote a user decision) and F3, with fixtures, regenerated pins, and page and
   capability text that state the actual rules.
2. Re-run `cargo test -p raf-census --locked`, the `goldens` and `baseline` examples (twice, with
   `cmp`), `cargo workplan --root . check`, and the root `docs_contract`, `self_workplan` and
   `repo_gates` binaries.
3. Run the full `make check` gate (still pending per `validation.md`); hosted CI is also pending.
