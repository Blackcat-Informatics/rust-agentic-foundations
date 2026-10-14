# X2 review (independent, single pass)

Subject: the uncommitted X2 fix on HEAD bf08e45 (new `verbatim.rs`; edits to `facts.rs`, `lib.rs`,
`workspace.rs`, three test files and `docs/capabilities/source-index.md`).
All experiments ran in copies (`/tmp/x2rev/copy`, `/tmp/x2rev/rc`); the issue worktree was only read.

## Scope

1. rustc fidelity of the group split, and the judgement on indexing bodiless items.
2. Placement and charging of the new facts (held <= charged).
3. Spans.
4. Refused forms still refuse, typed.
5. Contract text.
Gates: raf-source tests (debug), workspace clippy `-D warnings`, `docs_contract` + `repo_gates`.

## Evidence

Gates, run in a copy of the tree (`CARGO_BUILD_JOBS=6`):

- `cargo test -p raf-source` (debug): 476 tests, 0 failed, doctests pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0. `cargo fmt --all --check`: exit 0.
- `cargo test --test docs_contract --test repo_gates` (copy as its own git repo with `verbatim.rs` tracked):
  141 and 57 passed. The new Contract bindings resolve.

(1) rustc fidelity (rustc 1.101.0-nightly):

- `pub use { ::std::fmt::Debug as D, crate::a::f };` compiles in 2018 and 2021, and the grouped form
  with nested bare groups and trailing commas compiles. `pub use { ::inner::go as colon, super::inner };`
  compiles in 2015. The use_resolution fixtures compare the grouped form against the separate uses.
- `use a::{::b};` and `use { crate::a::{::b} };` are E0433 "crate root in paths can only be used in start
  position"; the index gives a `Syntax` error naming file and line.
- `use ::{ ::a }` is E0433 in rustc; the index refuses it as `Syntax`. `use { ::{ std::fmt::Debug } }`,
  `use { ::std }`, `use { ::std::fmt::* }`, `use { {}, ::x }` compile in rustc and load in the index.
- `pub fn f(x:&str)->&str;`, `pub type bool;` and `impl S { pub fn m(&self); }` are rustc errors
  ("free function without a body", "free type alias without body", "associated function in `impl` without
  body"), raised after parsing. Under `#[cfg(any())]`, in item and impl position, all compile.
  So these are parse-valid and error only if they survive cfg stripping and macro expansion.

(2) Charging: `use_fact` charges `size_of::<UseFact>() + vis_bytes` per member exactly as the plain path,
and `bodiless` goes through `self.item`, the same call as plain functions and aliases. The new
`load_budgets` test asserts equal `fact_bytes` against the separate-uses and empty-body forms and refusal at
the right line. Memory probe, release, VmHWM: 150k-member verbatim group 157 MB vs 150k separate uses
173 MB vs plain group 179 MB; 100k bodiless fns 225 MB vs 100k plain fns 234 MB. Held does not exceed the plain forms.
Stack: bare-group nesting is capped by the 128-delimiter pre-scan; path nesting inside a verbatim group at
n=100 loads on a 2 MiB debug thread like the plain form, n=110 is refused by the same recursion budget;
bodiless fns with 500 reference marks load, 1500 are refused (prefix-run budget) identically for bodiless,
impl and plain forms.

(3) Spans: the verbatim `whole` span runs from the first attribute to the `;`. `symbol_index` asserts start
and end bytes and line ranges for item-level, impl and type forms; `facts.rs` unit test asserts the group
span and per-member line/test role/visibility. Correct.

(4) Refusals, probed through `Workspace::load`: `static X: u8;`, `pub macro m() {}`, `const C: u8;`,
impl `type T: B;`, `pub trait Tr = Send;` are `UnsupportedItem` at the item's line; `a::{::b}`,
`use ::{ ::a }`, `use { ::a } extra;` are `Syntax`. `unsupported()` produces the same text as the old arms.

## Judgement: indexing a bodiless fn/type versus refusing

Index it. A bodiless fn or type parses in rustc and is rejected only by a post-expansion validation, so it is
legal in exactly two situations, both ones the index does not evaluate: cfg-stripped (the index evaluates no
`cfg` and indexes `#[cfg(any())] fn f() {}` today) and attribute-macro input (the index expands no macro and
indexes `#[attr] fn f() {}` as written today). A bodiless item that survives both is a compile error, so no
compiling crate has one that the macro does not consume. The nine registry crates are real compiling crates
that need it. Refusing would fail the whole load of a crate rustc accepts, contrary to the Contract's
"parsing is syntactic" stance, with no fidelity gain. The binding is "as written": a macro may drop or rename
the item (same unknown as for any attributed item with a body, and for macro-generated items the Contract
already says are not indexed), so the fix creates no new class of divergence. It does not violate fail-hard:
nothing is substituted or guessed, and the genuinely unreadable forms still fail typed. The Contract should
say "as written" explicitly (ADVISORY A1).

## Findings

### REQUIRED

**R1. Quadratic time on a verbatim `use` group with a large attribute.** `use_fact` calls
`summarize(attrs, ..)` once per member, so a group's attributes are formatted and parsed once per member.
The plain forms pay once per declaration, so total work was linear; here it is members x attribute size.
Reproduced in release on a MemoryTree (`#[cfg(any(a0,a1,...,aN,))] pub use { ::std::m0, ..., ::std::mN, };`):

| members = attr items | file bytes | verbatim | plain group (same attr, no `::`) |
|---|---|---|---|
| 2000 | 35.8 KB | 2.12 s | 12 ms |
| 4000 | 73.8 KB | 8.83 s | 24 ms |
| 8000 | 149.8 KB | 35.2 s | 37 ms |

Time quadruples per doubling. Within the 8 MiB / 2M-token limits this is hours for one file, with no budget
refusal, in a crate whose Contract bounds work per load. Cause confirmed: passing the attributes to the first
member only (experiment) returns all three rows to 9/18/32 ms.
Fix: call `summarize` once per group (in the `Item::Verbatim` use-group arm, or have `use_fact` take the
already-computed `own_test`) and reuse the role for each member. Add a regression test with a large
`cfg(any(..))` attribute and thousands of members that asserts completion at a lowered budget or a bounded
count of summarize calls; keep the existing per-member charging.

### ADVISORY

**A1.** The Contract (Items section and Constraints) says a bodiless function "is the input of an attribute
macro" but does not say the item is indexed as written, that the macro's output (kept, renamed, dropped, or
a different kind such as a struct for `#[ty] type bool;`) is not seen, or that the same shape is valid under
a false `cfg`. One sentence under Constraints, next to "the crate does not expand macros", closes it.

**A2.** The bodiless type is `SymbolKind::TypeAlias` with `alias: None`; a consumer that treats `TypeAlias` as
"has a definition" will be told nothing wrong (`alias` is `None` and the symbol is not followed), but the doc
could state that `alias` is `None` for it.

**A3.** The group split attaches the same span to every member's `UseFact`; no consumer keys on use spans
(checked `dedup` and span-keyed maps in `src/`), so this is correct today. The Contract already states it
("its facts span the whole declaration"); keep it if a consumer ever indexes uses by span.

## Verdict

R1 is a real, reproducible super-linear cost introduced by the fix, with a one-line cause and a local fix.
Everything else (fidelity, charging, spans, refusals, gates, Contract binding) checks out.

VERDICT: BLOCKED (superseded by the Recheck below)

## Recheck

Subject: the committed fix, HEAD 5bcd741 (`git archive` copy, so the issue worktree was only read).
`use_role(attrs, inherited)` runs once per verbatim group; `use_fact` takes the role; charging is unchanged.

- R1 timing, release, same probe as before (cfg(any(..)) attribute with N names, N verbatim members):

  | members = attr names | file bytes | verbatim before | verbatim now | plain group |
  |---|---|---|---|---|
  | 2000 | 35.8 KB | 2.12 s | 5.4 ms | 5.1 ms |
  | 4000 | 73.8 KB | 8.83 s | 10.7 ms | 11.0 ms |
  | 8000 | 149.8 KB | 35.2 s | 20.9 ms | 21.3 ms |
  | 64000 | 1.32 MB | not run | 178 ms | 181 ms |

  Linear, and equal to the plain form.
- Role per member (added a probe test to the copy, read `UseFact.test` for each member):
  `#[cfg(test)]`, `cfg(all(test, unix))` and `cfg_attr(test, cfg(test))` give `CfgTest` for every member;
  `cfg(any(test, unix))`, `cfg(not(test))`, `cfg_attr(test, allow(..))` and no cfg give `None` for every member;
  a group in a `#[cfg(test)] mod` is `CfgTest` for every member whether or not it has its own cfg; in one
  module `#[cfg(test)]` on a group and a plain neighbour give `[CfgTest, CfgTest, None]`. All match the plain
  `#[cfg(test)] use ::std::a;` (`CfgTest`). Attribute facts are not kept on `UseFact`, so nothing else per member
  could regress; visibility, `leading_colon`, span and tree per member are asserted by the new regression test
  (6,000 members) and by the earlier use-group unit test.
- Gates on the committed tree: `cargo test -p raf-source` (debug) 478 passed, 0 failed; workspace clippy
  `-D warnings` exit 0; `cargo fmt --check` clean; `docs_contract` 141 and `repo_gates` 57 pass (copy initialised
  as a git repo, since both read the index).
- A1 and A2: the Contract now says the bodiless items are indexed as written (the macro may keep, rename or
  drop them; also valid under a false `cfg`), and that the bodiless type is an alias whose target is `None` so
  no path is followed. Both resolved. A3 stands as noted, no action.

R1 is resolved. No new findings.

VERDICT: PASS
