# R5d review: resolve-path validation and missing local globs

Reviewer: independent single pass (opus/high). Change under review: the uncommitted
tree in `/tmp/raf-par2/a-r5d` (detached at 44e18b7), identical (`cmp`) to
`raw/R5d.patch` applied with `git apply -p0` to a `git archive 44e18b7` copy in
`/tmp/r5d-rev`. All experiments ran in copies (`/tmp/r5d-base` = 44e18b7,
`/tmp/r5d-pt` = patched, `/tmp/r5d-main` = main worktree's current uncommitted R5c
state). Probe sources: `/tmp/r5d-probe/zz_probe*.rs`, outputs `/tmp/r5d-probe/*.out`.

## Scope

Contract "### R5d" of `remediation-plan.md` (P-F9 linear validation, A12 keyword
segments and the `SuperAboveRoot` arm, A13 typed error for a missing local glob with
the load succeeding, S-10 `ResolveError` Display escaping), "Mutation rule",
"Per-item gates". Files: `crates/raf-source/src/resolve.rs`, `src/workspace.rs`,
`tests/error_text.rs`, `tests/resolve_paths.rs` (new),
`docs/capabilities/source-index.md`.

## Gates (patched copy `/tmp/r5d-rev`)

- `cargo test -p raf-source --locked`: EXIT 0, all binaries ok.
- `cargo test -p raf-source --locked --release`: EXIT 0.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean.
- `cargo test --locked --test docs_contract --test repo_gates`: 139 + 57 passed.
- No process wording or issue numbers in added lines; `.deficiencies` untouched.

## Contract checks

- `demo::crate::a::x`, `demo::super::x`, `demo::self::a::x`, `demo::a::Self`,
  `r#crate`, leading keywords: `Invalid` in `resolve` and every `resolve_in`, beside
  `demo::a::x` Ok; `crates`/`selfish`/`Selves` resolve. Met.
- `Err(Fail::SuperAboveRoot) => {}` removed; the arm now returns `Invalid`. Met.
- Missing-glob fixture: load succeeds, lookups are `UnresolvedGlob`, sibling Ok. Met.
- P-F9 50k probe: release 4.4 ms + 19.8 ms (implementer); my debug measurement
  28 ms missing + 157 ms deep (185 ms total). The plan's "<100 ms" holds in release;
  the committed test bound is 1 s (see ADVISORY A3).
- S-10: every `ResolveError` field wrapped in `Esc`; unit test enumerates every
  variant through an exhaustive match; integration test covers `Invalid` and
  `UnknownPackage`. Met.
- Mutation rule: the falsification log shows the keyword, missing-glob, 50k,
  error-text and unit tests fail on 44e18b7; the NotFound test passes there and is
  covered by M1-M3. I re-ran M1 (`glob_path` never `Unknown`) on a copy:
  `a_glob_the_index_cannot_look_into_leaves_a_lookup_not_found` FAILED with
  `UnresolvedGlob { glob: "std::io" }`, file restored (`cmp` identical).

## Design question: `resolve` failing on a broken type-space glob while a value exists

Consistent. `Index::resolve` (base `resolve.rs:1010-1015`, unchanged) returns the
first name space with any answer other than `NotFound`: `Ok`, or an error such as
`Cyclic` or `Ambiguous`, which already stops the loop before a later name space's
`Ok`. `UnresolvedGlob` follows the same rule, and it is also sound: the broken glob
could bring in a type `f`, which `resolve` (type first) would have to return, so the
value answer is not known to be the answer. `resolve_in(Value)` still finds it. Not
a finding.

## Findings

### REQUIRED

**R1. Two dependency globs in one module make every missing name `Cyclic`, not
`NotFound` (confirmed on 44e18b7, on the main worktree's R5c state and on R5d).**
This is a fail-hard misreport, and it contradicts R5d's own contract that a glob
whose first segment names nothing in the workspace "stays `NotFound`".

Minimal repro (crate `demo`, `src/lib.rs`):

```rust
pub mod a { pub use std::io::*; pub use serde::*; }
```

`resolve("demo::a::gone")` and `resolve_in` in all three name spaces give
`Err(Cyclic("demo::a::gone"))`. With only `pub use std::io::*;` the answer is
`NotFound`. The same happens with `std::io::*` + `std::fmt::*`, with `serde::*` +
`tokio::*`, and with both globs at the crate root (`demo::gone`). It also spreads:
with `pub struct Here;` in `a` and `pub mod b { pub use crate::a::*; }`,
`demo::b::gone` is `Cyclic`, and so are `resolve_in(demo::a::Here, Macro)` and
`resolve_in(demo::b::Here, Macro)`. `std::io::*` + `self::missing::*` is also
`Cyclic`, where R5d intends `UnresolvedGlob`. Leading-colon spellings (`::std::io::*`
+ `::serde::*`) give `NotFound`, so the bug hits the usual spelling. rustc compiles
the repro, because `std` and `serde` come from the extern prelude.

Where `Cyclic` is chosen:
- Each glob's path reads its first segment (`std`, `serde`) in module `a`, and the
  other open glob could still bring that name in. The two globs therefore form a
  looped, glob-only blocker component, and neither path resolves.
- `settle.rs:1402` (the same line in base and patched),
  `if !imports.is_empty() || (looped && !resolved) { return Decided::Cyclic; }`,
  leaves both globs unclosed (`glob_closed` stays false).
- At lookup time, `resolve.rs` `unbound` (patched lines 838-841) calls
  `settle.rs` `waits_on_cycle`. That returns true through `open_glob(module, None)`
  at `settle.rs:667`, so the lookup gives `ResolveError::Cyclic`.

To confirm the location, I removed only the `(looped && !resolved)` term on a copy.
Every probe above then answered `NotFound` (and `demo::a::Here` / `demo::b::Here`
stayed Ok). That edit is a diagnostic, not the fix, because the term also covers real
glob cycles. The fix needs:
- a component whose unresolved glob paths can only be completed by globs whose
  heads name nothing in the workspace (dependency heads, `GlobPath::Unknown` in R5d's
  terms) is closed like a lone such glob, or the head lookup does not wait on
  dependency globs;
- fixtures for the repro, its root-level form, the spread through `crate::a::*`,
  and `std::io::*` + `self::missing::*` (which must then be `UnresolvedGlob`), each
  beside a positive resolution.

**R2. The stated error order Cyclic > Ambiguous > UnresolvedGlob > NotFound is
broken in `walk_from_root`, which also regresses a `Cyclic` answer.** `unbound`
documents and implements the order. But `walk_from_root` runs
`unresolved_glob_read(&reads)` right after the walk and returns `UnresolvedGlob`
before `unbound` gets the `missing` places to check for a cycle. Repro:

```rust
pub mod a { pub use self::missing::*; pub use crate::c::*; }
pub mod c { pub use crate::d::gone; }
pub mod d { pub use crate::c::gone; }
```

`resolve("demo::a::gone")` is `Cyclic` on 44e18b7 and `UnresolvedGlob { glob:
"self::missing", module: "demo::a" }` on R5d. Meanwhile `demo::c::gone` is still
`Cyclic`. Required: in `walk_from_root`, apply the glob check to a `Missing` walk only
after the cycle check that `unbound` makes (for example, let `unbound` decide for
missing walks and keep the early glob return for `Ok` walks only), or state and
justify a different order. Either way, add this fixture.

### ADVISORY

**A1. Per-call cost of a successful `resolve` on glob-dense input.**
`walk_from_root` now runs `unresolved_glob_read` on every walk, including `Ok`
walks. That calls `globs_read` (a transitive walk of the glob graph) for every place
read, and `glob_path` re-walks every visited glob's path, twice per glob, on every
call. Measured in release on N modules that each glob all the others:
- N=80 (6320 globs): 80 successful resolves went from 107 µs (base) to 234 ms
  (patched), about 3 ms per call;
- N=40: 84 µs to 31 ms;
- the missing lookup is unchanged (160 ms vs 173 ms), and load is unchanged.

The cost is bounded by input size, but it is O(places x globs) per call. Suggest
computing, once at load, whether any glob is `GlobPath::Misses` and skipping the
whole check when none is (almost every crate), and/or caching `glob_path` per glob.
The tables are immutable after load.

**A2. Untested consequence: a name found through a resolving glob beside a broken
glob now fails.** With `pub mod a { pub use self::missing::*; pub use crate::c::*; }
pub mod c { pub struct C; }` (and, as a second fixture, `c` holding
`pub mod inner { pub fn z() {} }`):
- `demo::a::C` and `demo::a::inner::z` were Ok on 44e18b7 and are `UnresolvedGlob`
  on R5d;
- `demo::a::gone` with `self::m::missing::*` (where `m` is a local empty module),
  `pub use a as b` then `demo::b::gone`, and a root `crate::missing::*` then
  `demo::gone` are also `UnresolvedGlob`.

This is consistent with the documented rule (only an explicit definition or import
keeps a lookup from the module's globs) and is fail-hard correct, because the broken
glob could make `C` ambiguous. No fixture pins it. Add at least the glob-found `C`
case and the `self::m::missing` case.

**A3. The 50k test bound is 1 s, while the plan's check is under 100 ms.** Debug
measures 185 ms, so a 100 ms bound in debug would be flaky. The release probe meets
the plan. Consider asserting a tighter bound under `cfg(not(debug_assertions))`, or
recording that the 100 ms figure is a release figure.

**A4. Docs.** "A path of 50,000 segments is answered in milliseconds" is true in
release only (about 185 ms in debug). Also, the precedence among
`Cyclic`/`Ambiguous`/`UnresolvedGlob` is not stated in `source-index.md`; once R2 is
settled, state it.

Initial verdict (superseded by the Recheck below): BLOCKED

## Recheck

Subject: the staged R5d state in the main issue worktree on top of R5c (be71d40) and
R11a (df71a42, HEAD). Copy: `/tmp/r5d-re` = `git archive HEAD` + `git diff --cached`
applied with `-p0`, each staged file `cmp`-identical to the index. The real tree was
not touched. The probe ran in a second copy, `/tmp/r5d-re2`
(`tests/zz_re.rs`).

### Spot gates (copy)
- `cargo test -p raf-source --locked` (debug): EXIT 0, 30 binaries ok, and
  `resolve_paths` has 8 passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: EXIT 0.
- `cargo test --locked --test docs_contract --test repo_gates`: 141 + 57 passed.

### REQUIRED-1 (two dependency globs gave Cyclic): resolved
- `pub mod a { pub use std::io::*; pub use serde::*; }`: `demo::a::gone` is
  `NotFound` in `resolve` and in every name space.
- The root form, `std::io::*` + `std::fmt::*`, and the spread through
  `pub mod b { pub use crate::a::*; }` are `NotFound` as well.
- `demo::a::Here` / `demo::b::Here` stay Ok, and the Macro space is now `NotFound`
  instead of `Cyclic`.
- `std::io::*` + `self::missing::*` is `NotFound`. That follows the documented rule:
  `std::io::*` could bring in `missing`.
- `std::io::*` + `crate::missing::*` is `UnresolvedGlob { glob: "crate::missing" }`.
- The real glob cycle `a { pub use crate::b::x::*; }` / `b { pub use crate::a::*; }`
  stays `Cyclic`.
- Mutation: with the `!at_heads` exemption disabled in `decide`,
  `globs_of_dependencies_in_one_module_leave_a_missing_name_not_found` FAILS and the
  other 7 pass. File restored.

### REQUIRED-2 (order): resolved
- `walk_from_root` no longer returns early for a missing walk. `unbound` now takes the
  walk's reads and checks Cyclic, then Ambiguous, then UnresolvedGlob, then NotFound.
  The glob check for an `Ok` walk runs after the multi-symbol `Ambiguous` check.
- My repro (`a { self::missing::*; crate::c::* }`, with `c::gone` and `d::gone`
  importing each other) gives `Cyclic` for `demo::a::gone` again, and `demo::c::gone`
  is `Cyclic`.
- The order is now written in `source-index.md`, and
  `a_cycle_is_reported_before_a_glob_that_names_nothing` pins it.

### Own probes around `held_at_head` (could a real cycle be settled as nothing?)
- **P1.** Heads that are local modules brought in only by another glob of the module,
  each head module re-exporting the other:
  `a { crate::p::*; x::*; y::* }`, with `p::x` re-exporting `q::y` and `q::y`
  re-exporting `p::x`. `demo::a::X` and `demo::a::Y` resolve; `gone` is `NotFound`.
  The rule did not settle the resolvable heads as nothing.
- **P2.** Two globs whose heads each come only from the other glob:
  `x { pub use crate::y; }`, `y { pub use crate::x; }`, `a { x::*; y::* }`.
  This gives `NotFound` for `X`, `Y` and `gone`. rustc refuses this (`x` is not in
  scope in `a`), so neither glob brings in anything. `NotFound` matches the lone
  `pub mod a { pub use x::*; }` answer: a first segment that names nothing is read as
  a dependency. It does not hide a resolvable cycle.
- **P3.** Heads brought in by a glob of a module that globs back:
  `a { x::*; y::*; crate::b::* }`, with `b { crate::a::*; mod x; mod y }`. `X`/`Y`
  resolve in `a`, `b::X` resolves, and `gone` is `NotFound`.
- **P4.** Heads that could come from a glob whose path waits on a named-import cycle:
  `a { x::*; y::*; crate::c::m::* }`, with `c::m` and `d::m` importing each other.
  `demo::a::gone` is `Cyclic`. The undecided import keeps the component Cyclic, so it
  is not settled as nothing.
- **P5.** A head brought in through a chain of globs and a re-export:
  `a { x::*; y::*; crate::r::* }`, `r { crate::s::* }`,
  `s { mod x { mod y {Deep} X } pub use self::x::y; }`. `X` and `Deep` resolve;
  `gone` is `NotFound`.
- **P6.** 2015, with crate-relative heads: `a { x::*; y::* }` and a root `mod x`.
  `demo::a::X` resolves and `gone` is `NotFound`. With neither module, `gone` is
  `NotFound`.

No probe turned a resolvable or genuinely cyclic case into "nothing".

### Remaining ADVISORY (pre-existing design, not R5d scope)
- In editions 2018 and later, a glob whose first segment is neither in scope nor in
  the extern prelude (P2, or `pub mod a { pub use x::*; }` where `x` is a root module
  and not a dependency) is read as a dependency and answers `NotFound`. rustc refuses
  it as an unresolved import. Since the index knows the use prelude
  (`in_use_prelude`), a later change could report it as `UnresolvedGlob`. R5d's
  contract defines that error as "past its first segment", so this is not a finding
  against R5d.

VERDICT: PASS
