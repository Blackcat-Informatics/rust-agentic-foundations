# Gap Analysis

**Issue:** #14 (raf-source: model rustc's visibility of re-exported private glob bindings and of ambiguity through inaccessible globs)
**PR:** #26
**Branch:** paudley/14-raf-source-model-rustc-s-visibility-of (HEAD fc21ada)
**Base:** origin/main (16ffd2f), scope `origin/main...HEAD`, 21 commits
**Files changed:** 64 (+7000 / -996)
**Toolchain:** rustc 1.101.0-nightly (21b707e3f 2026-09-30)

Method: I built a scratch probe against the shipped API (`Workspace::resolve_from_in` and
`resolve_in`) of HEAD, of origin/main (via `git archive`) and of each resolver commit. A
scratch driver shuffles the declarations at every level, writes `use <path> as __pN;` probes
plus a `TAG` assertion per probe, and compiles with `rustc --edition 2024 --crate-type lib
--emit=metadata` (one case rechecked under 2021, the oracle's edition). Verdicts count only
errors on the probe line. All scratch files are under /tmp/gap14 and /tmp/gapan-*; nothing
tracked was modified. I ran no timing test, no `cargo test --workspace` and no `make check`.

## Requirements Coverage

| # | Requirement | Status | Location | Gap Description |
|---|-------------|--------|----------|-----------------|
| R1 | Model both rules in the settling of `use` paths, independent of declaration order | PARTIAL | settle.rs, glob_scope.rs, resolve.rs | Rule 1: complete. Rule 2: complete except for a bare first segment (H1). Order independence: broken by the rule-2 commit 79fb598 on the companion shape (C1), a regression against base. |
| R2 | An every-order fixture for each rule | COMPLETE (one gap) | tests/use_resolution.rs, tests/exported_items.rs | 4 rule-1 and 13 rule-2/neighbour every-order fixtures. None covers a bare first segment (`s::A`), and none covers the companion shape of C1 (it would fail in debug). |
| R3 | Re-run the rustc-oracle fuzz with private globs, no new disagreements | MET for the generated distribution | tests/rustc_oracle.rs | Reproduced (see below). The comparison with base is not stated in the PR. The generator never emits a bare first segment, so the oracle cannot see H1. |
| R4 | Remove the two sentences from the Contract's Constraints | COMPLETE | docs/capabilities/source-index.md | The base bullet "Two visibility rules of rustc are not modelled ..." is gone. The surface bullet "Of the two visibility rules above that the index does not model ..." is rewritten to the delivered behaviour. |
| R5 | rustc keeps a re-export no more visible than the binding it re-exports | COMPLETE | glob_scope.rs | 7 shapes x 12 random orders agree with rustc (below). |
| R6 | rustc refuses the path (E0659, sometimes E0573) | PARTIAL | resolve.rs `start` | Refused correctly in 9 shapes x 12 orders. Not refused when the ambiguous name is the first segment of a relative path (H1). |
| R7 | The two rules are no longer "not modelled" | PARTIAL | - | As R1. |
| O1 | Owner: update README.md to the current status | COMPLETE | README.md | Matches `workplan status` (delivered 12, designed 9, 9 open census changes, implementing 0) and the `--help` command list. `raf_workplan::Gate::workspace(..).assert_clean()` exists (gate.rs:190, :301). |
| O2 | Owner: "Convert timing tests first" | COMPLETE | crates/raf-source, tests/repo_gates.rs | The clock grep over src and tests is empty. The `time_in_raf_source` gate passes 5/5 and its word list now includes core::time, tokio::time and the *_timeout waits. The whole raf-source suite (debug, run once) passes: 0 failed, 2 ignored corpus writers. |
| O3 | Owner: no timing runs on this machine | COMPLIED (this review) | - | The PR discloses three earlier whole-crate runs that executed the old elapsed-time tests before conversion. They cannot be undone, and they were disclosed. |

### Verification run in this review
- `RAF_ORACLE_PROGRAMS=5522 cargo test -p raf-source --locked --release --test rustc_oracle -- --nocapture`: exit 0, 5 passed, 1 ignored.
  rustc 1.101.0-nightly (21b707e3f 2026-09-30); seeds 0xa11ce01..08; programs 5522 (5522 judged); probes 24670 (11728 accepted, 12942 refused).
  | class | count |
  |---|---|
  | rule1 | 0 |
  | rule2 | 0 |
  | documented | 5 |
  | order | 9 |
  | cycle | 1 |
  | unclassified | 0 |
  This matches the PR's claim for 5,522. I did not re-run 11,044.
- Extra: `RAF_ORACLE_PROGRAMS=1500` in a debug build (the size issue #25 names): 1500 judged, rule1 0, rule2 0, documented 0, order 2, cycle 0, unclassified 0, no debug panic.
- `cargo test -p raf-source --locked` (debug, once): every binary passes, plus doctests 10 + 6.
- `cargo test --locked --test docs_contract --test self_workplan`: 28/28 and 1/1. `cargo test --locked --test repo_gates`: 62/62 (the `time_in_raf_source` subset is 5/5). `cargo test -p raf-workplan --locked --test digest` (clock-free): 11/11.
- `cargo run -q -p cargo-workplan -- workplan check`: exit 0. `workplan assess --list`: every assessment is `accepted` and `current`, including the 5 source-index sections.
- CI for PR #26: 12/12 checks pass (clippy, cross-check, deny, doc, fmt, fuzz, msrv, reuse, test, test-macos, test-release, test-windows).

### Independent probes of the two rules (HEAD, `resolve_from_in`, rustc 2024, 12 random orders each; root blocks and every module body shuffled)
Rule 1, 7 shapes, 0 diverging probes. The shapes:
- the issue/doc shape (`pub use super::A` of a private-glob binding). rustc refuses the re-export itself with E0365; reads inside `m` resolve, reads outside are refused.
- a glob re-export of a private-glob binding (rustc accepts the crate);
- a cap at a deeper ancestor;
- a chain of two capped re-exports;
- a `pub` glob neighbour;
- routes combining by the widest;
- the value name space under a `pub(super)` cap.

Rule 2, 9 shapes. The shapes:
- the issue shape (a `pub` glob and a private glob of two `A`s): index Ambiguous; rustc E0659, or E0432 in some orders, both refusals;
- two private globs read from outside;
- one item reached through both globs;
- different names;
- the module's own item shadows its globs;
- a private named import shadows its globs;
- a module ambiguous in the middle of a path;
- the value name space with a `pub(crate)` glob;
- a second glob layer.

Every probe agrees in every order except the bare first segment: H1.

## Acceptance Criteria Status

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Rule 1 modelled, every order | MET | Probes above. Fixtures `a_pub_use_of_a_name_read_through_a_private_glob_is_seen_no_wider_than_the_glob_in_every_order` and 3 siblings. |
| Rule 2 modelled, every order | NOT MET | C1: order dependence and a debug panic introduced by 79fb598. H1: bare first segment answers the other item. |
| Every-order fixture per rule | MET | use_resolution.rs and exported_items.rs (list in Requirements). |
| Oracle re-run, no new disagreements | MET (generated distribution) | 5,522 release reproduced; 1,500 debug clean. |
| Constraints sentences removed | MET | grep shows no "not modelled" or "does not model" for these rules. No PROPOSED block remains. |
| README current | MET | See O1. |

## Integration Gaps

### Wiring Issues
- [ ] `Index::start` (crates/raf-source/src/resolve.rs, the `_` arm around lines 617-648) merges every item a bare first segment binds. It lacks the `!last && bound.len() > 1` ambiguity check that the segment loop applies (line 721). Every `use` path whose first segment is a name the module binds therefore skips the rule-2 and middle-segment ambiguity rules. This covers paths through `resolve_from`/`resolve_from_in` and through the settling of `use` declarations (H1).
- [ ] `Index::module_named` (resolve.rs) uses `find_map` over `crate_roots[first]`. With two packages whose libraries are both named `shared`, `resolve_from("shared::m", "self::Y")` silently reads p-one's `shared::m` and answers `NotFound` for p-two's `Y`. `resolve("shared::m::X")` answers `Ambiguous` for the same crate path (M1).

### Consumers
- raf-workplan uses `Workspace::load`, `resolve`, `resolve_in_package` and `exported_items` (gate.rs, bindings.rs, digest.rs, graph.rs, coverage.rs:894, :1040). It does not use `resolve_from`. The rule changes reach it through the settled tables. The repository's own bindings still resolve (`docs_contract::repository_bindings_resolve_in_the_source_index` passes; `self_workplan` passes).
- The debug panic of C1 is in `Workspace::load`. A consumer runs RAF as a dev-dependency in debug `cargo test`, so a consumer workspace with that shape would crash its workplan gate with a panic instead of getting a typed error or finding.

### WIRING_GAPS markers
None found in the added lines: no "In full implementation", "TODO: wire" or "Placeholder", and no open-ended "later" or "follow-up".

## ETHOS/SOLID Assessment

| Principle | Status | Notes |
|-----------|--------|-------|
| SRP | PASS | The rules live in glob_scope.rs/settle.rs; resolve_from is a thin entry over the `Heads::Use` walk; test-only tallies are in work.rs. |
| OCP | PASS | Additive public API. |
| LSP | PASS | n/a |
| ISP | PASS | `resolve_from` and `resolve_from_in` mirror `resolve` and `resolve_in`. |
| DIP | PASS | Tests count work through the `Tree` seam (support/counting.rs). |

| Section | Status | Notes |
|---------|--------|-------|
| Fail hard (AGENTS.md section 5) | VIOLATION | M1: a silent first-root choice for an ambiguous module path. C1: a debug-only panic where release gives an order-dependent answer. |
| No deferrals (section 7) | VIOLATION | C1 is a regression this change introduced into its own required property (R1, order independence). It is deferred to #25 rather than fixed. |
| Contract states the delivered contract only (section 6) | VIOLATION | The Contract (source-index.md line 1318) says the answer "does not depend on the order of the declarations" and that "a path through a module that two globs bring in as different modules is ambiguous". Both are false (C1, H1). Constraints line 1668 contradicts the first. |
| Negative fixture / refusal neighbour (section 5) | PASS | Each rule has refusals and passing neighbours in every order. |

### Optionality to Remove
- None. `ResolveError` is `#[non_exhaustive]`, so `UnknownModule` is additive. `resolve_from` takes no optional arguments.

## Reviewer Feedback (Unaddressed)

| Source | Feedback | Status |
|--------|----------|--------|
| T4c re-review (c) | Residual companion order dependence: record its owner or fix it | PENDING. Recorded as #25, but stated as pre-existing, which it is not (C1). |
| T4c re-review (blocking) | Fixture comment misstated rustc | ADDRESSED: the comment now says rustc agrees (E0659). |
| T4b-timing F1/F3 | `> 0` guard on the ops equality; gate words | ADDRESSED (19f3701). |
| T3 R3 | State the fixed Ambiguous answer where rustc is order-dependent | ADDRESSED (Contract and Constraints). |
| T1 F2 | `write_corpus` is `#[ignore]`d | OPEN (LOW). Kept, with the reason in its ignore message. |
| PR/issue comments and inline reviews | None beyond the plan and the least-confident note | - |

## Quality Gaps

### Test Coverage
- [ ] No every-order fixture for a bare first segment through an ambiguous glob-imported module (H1).
- [ ] No fixture for the companion shape of #25 in a debug build (C1). It would fail today.
- [ ] The oracle generator (`tests/support/program.rs`, `reach` and `random_probe`) writes only paths that start with `crate`, `self` or `super`. `resolve_from` documents "a name the module binds" as a first segment, and the oracle never exercises that form.
- [ ] No fixture for `resolve_from` on a module path that two crates share (M1).

### Documentation
- [ ] source-index.md:1318: the order-independence and middle-segment sentences are false (C1, H1).
- [ ] CHANGELOG `[Unreleased]`: "Both rules hold in every declaration order" is false (C1).
- [ ] source-index.md Verification: "the rustc of the pinned toolchain" is wrong, because rust-toolchain.toml floats (`channel = "nightly"`) (L1).
- [ ] source-index.md:1318: the rule-1 example crate is refused by rustc at the re-export (E0365). The paragraph does not say so; the exports paragraph does (L2).

## Findings, ranked

### C1 [CRITICAL] The rule-2 commit introduced declaration-order dependence and a debug-build panic in `Workspace::load`
Program (the #25 shape; type name space; probe `demo::mm::probe`):
```rust
pub mod m1 { pub mod s1 { pub struct B {} } }
pub mod m2 { pub struct B {} }
pub mod m0 { pub use crate::m2::*; use crate::m1::s1::*; }
pub mod w  { pub use crate::m2::B; }
pub mod mm { pub use crate::m0::*; pub use crate::w::*; use self::B as probe; }
```
All 120 orders of the five top-level blocks:
| Build | Answers |
|---|---|
| origin/main 16ffd2f, debug and release | `m2::B` in 120/120, no panic |
| HEAD, debug | `m2::B` in 100; panic in 20 at settle.rs:2047, "the imports of `probe` bound {ItemId(2), ItemId(4)} and reach {ItemId(4)} in the settled tables" |
| HEAD, release | `m2::B` in 100, `Ambiguous(m1::s1::B, m2::B)` in 20 |

rustc: E0659 in every order.

Variant `w { pub(crate) use crate::m2::B; }` gives the same split. Three other variants (`w` = `pub use m2::*; pub use mm::*`, or `pub use mm::*`, or `pub use m0::B`) give one answer at both HEAD and base: Ambiguous at HEAD (agreeing with rustc), `m2::B` at base.

Per-commit check (forced rebuilds):
| Commit | Result |
|---|---|
| e0a75e9 (rule 1) | 120/120 consistent |
| 79fb598 (rule 2) | 20/120 panic |
| 173a2cb | 20/120 panic |
| e5bc866 | 20/120 panic |
| f1603cd | 20/120 panic |
| HEAD | 20/120 panic |

So 79fb598 introduced it.

This breaks R1 ("keeping resolution independent of declaration order") on a rule-2 shape (`m1::s1::B` reaches `mm` through `m0`'s private glob). The Contract (line 1318) says the opposite, and so does CHANGELOG ("Both rules hold in every declaration order"). The handoff note on the issue says "(the same at the base commit; tracked)", which is false: base is consistent and never panics on this family. The PR body names only the glob-cycle companion as a regression the second rule introduced. Moving a regression in the issue's own required property to #25 is not a permitted deferral.

### H1 [HIGH] Rule 2 and the middle-segment ambiguity are not applied to a bare first segment
Program, which rustc accepts as written (editions 2021 and 2024):
```rust
pub mod p { pub mod s { pub struct A {} } }
pub mod q { pub mod s { pub struct B {} } }
pub mod m { pub use crate::p::*; use crate::q::*; pub mod k { use super::*; } }
pub mod r { use crate::m::*; }
```
`resolve_from` answers the other item in 8/8 orders, while rustc refuses a `use` of the path:
| Reader | Path | Index | rustc |
|---|---|---|---|
| `demo::r` | `s::A` | `demo::p::s::A` | E0659, or E0432 in some orders (`q::s` comes through `m`'s private glob, which `r` cannot access: the issue's rule-2 defect class) |
| `demo::m` | `s::A` | `demo::p::s::A` | E0659 |
| `demo::m` | `s::B` | `demo::q::s::B` | E0659 |
| `demo::m::k` | `s::A` | `demo::p::s::A` | E0659 |

Two `pub` globs behave the same. `self::s::A` and `super::s::A` are correctly Ambiguous.

In the settling, `pub use s::A as X;` inside `r` binds `X` to `p::s::A` at HEAD and at base, while rustc refuses that import in 16/16 orders (8 with E0659, 8 with E0432).

Cause: the bare-first-segment arm of `start` (resolve.rs ~617-648) has no ambiguity check.

The Contract sentence at line 1318, "A path through a module that two globs bring in as different modules is ambiguous, also for a name only one of the two modules has", is false for this form. The fixture `a_path_through_a_module_two_globs_bring_in_as_different_modules_is_ambiguous_in_every_order` covers only `crate::m::s::X` and `super::s::X`. The oracle cannot see the gap, because no generated path starts with a bare name.

### M1 [MEDIUM] `resolve_from` silently picks the first crate when two crates share a module path
`module_named` uses `find_map`. In a workspace where `p-one` and `p-two` both have a library `shared` with a module `m`:
| Call | Answer |
|---|---|
| `resolve_from("shared::m", "self::X")` | `shared::m::X` of p-one |
| `resolve_from("shared::m", "self::Y")` | `NotFound`, though p-two defines `Y` |
| `resolve("shared::m::X")` | `Ambiguous` |

This is a silent fallback, against AGENTS.md section 5. It is undocumented, and there is no package-qualified variant.

### M2 [MEDIUM] Honesty of the PR body, handoff note and CHANGELOG
- "the same at the base commit" (least-confident note on #14) is false (C1).
- CHANGELOG: "Both rules hold in every declaration order" is false (C1).
- The PR does not state the R3 comparison with base. The T3 review found 4 probes that newly diverge against base at 5,522. All are shapes where rustc's own answer depends on order, so they sit in the `order` and `documented` classes. That supports "no new disagreements" in substance, but the PR should say it.
- The Verification section presents 5,522 and 11,044 as "the last full runs". The PR itself reports five further glob-cycle divergences past 11,044. Pre-existing at base: #24's comments say all five diverge at every commit before this change, and I confirmed it for program 1578 (base resolves `demo::B` to `m0::C` where rustc finds no `B`). They are tracked in #24, but the document should not imply the test is clean beyond its stated sizes without saying so.

### L1 [LOW] "Pinned toolchain" wording
The Verification text says "the rustc of the pinned toolchain", but the toolchain floats (rust-toolchain.toml, AGENTS.md section 2).

### L2 [LOW] The rule-1 example is a crate rustc refuses
`mod m { use crate::x::*; pub mod c { pub use super::A; } }` is E0365 at the re-export. The reads the doc states do match rustc, but the paragraph should say the crate is refused.

### L3 [LOW] examples/registry_scan.rs is untracked
It keeps a `--timeout` and a `sleep` poll. It is not a test and this change did not touch it, but no open issue tracks it, although the PR's disclosures read as if it were handed off. Add it to #21 or open its own issue.

### L4 [LOW] Two ignored corpus writers
The two `write_corpus` tests show as ignored on every run. There is precedent, and the reason is stated.

## Deferrals judged (the user's question 7)
| Item | Tracked | Required by this issue? | Judgement |
|---|---|---|---|
| Glob-cycle gap (cycle class; five shapes past 11,044) | #24, with scope and acceptance; stated in Constraints line 1666 | No | Pre-existing at base (reviews; 1578 confirmed). It does not count as a new disagreement against the base. Both stated directions reproduce: (a) corpus `import_cycle_through_a_named_import_and_two_globs_of_a_module`: the type name space is `Cyclic` through `resolve_from` where rustc accepts `use self::B`; (b) corpus `import_cycle_rustc_cannot_determine_through_a_named_import_and_two_globs`: the index resolves the value `crate::m2::A` where rustc says "cannot determine resolution for the import". Acceptable deferral. |
| Companion order dependence | #25; stated in Constraints line 1668 | YES | Introduced by 79fb598 (C1). R1 requires order independence. Not an acceptable deferral. |
| Timing tests of raf-okf / raf-workplan | #21 | No | Outside raf-source. The owner's decision covered raf-source before completion. Acceptable. |
| examples/registry_scan.rs | Not tracked | No | Flag it with a tracking issue (L3). |

## Priority Ranking

1. **[CRITICAL]** C1: the rule-2 commit makes `Workspace::load` order-dependent and panic in debug on a rule-2 shape. This is a regression against base, deferred, and misreported as pre-existing.
2. **[HIGH]** H1: a bare first segment skips rule 2 and the middle-segment ambiguity, in `resolve_from` and in the settling. The Contract claims otherwise, and the oracle cannot see it.
3. **[MEDIUM]** M1: `resolve_from` picks the first crate silently when a module path is shared.
4. **[MEDIUM]** M2: the "same at base" claim, the CHANGELOG order-independence claim, the comparison with base, and the run-size framing.
5. **[LOW]** L1 "pinned"; L2 the rule-1 example is E0365; L3 registry_scan untracked; L4 ignored corpus writers.

## Recommended Actions (required fixes for PASS: 1 to 4)

1. **C1.** Fix the companion mechanism so the glob views and the settled tables agree in every order for the #25 shape and its `pub(crate)` variant, keeping views monotone. Work in glob_scope.rs `arrive` / `add_to_view` / `delivered`, and in settle.rs `check_settled`'s counterpart. Add an every-order fixture that passes in debug and release, with neighbours, and an `agree` corpus entry. Re-run the oracle at 5,522 (release) and 1,500 (debug). Then close #25 through this change or fold it in. If no fix is made, rule 2 cannot be claimed order-independent and R1 is unmet.
2. **H1.** In `Index::start`, refuse a bare first segment that binds more than one item when the path continues (`segs.len() > 1 && bound.len() > 1` gives `Fail::Ambiguous`). This matches the loop at resolve.rs:721. Add every-order fixtures for `s::A` read from `m`, from `m::k` (`use super::*`) and from `r` (glob of `m`, with `q::s` behind a private glob), and for `pub use s::A as X;` in the settling. Let the oracle generator emit bare first segments (for example, `reach` writing the bare name of a glob-imported or child module), then re-run the oracle.
3. **M2.** Correct CHANGELOG ("Both rules hold in every declaration order") and the Contract sentence at line 1318 if any order dependence remains. Correct the issue/PR handoff note ("the same at the base commit"). State the R3 comparison with base and the run-size caveat.
4. **M1.** Make `resolve_from` refuse a module path that several crates share (`Ambiguous` naming the module roots, as `resolve` does), or add a package-qualified variant. Document it, and add a fixture with a passing neighbour.
5. LOW items: replace "pinned" with the floating-nightly wording; note E0365 for the rule-1 example; open a tracking issue for examples/registry_scan.rs (or add it to #21).

VERDICT: BLOCKED
