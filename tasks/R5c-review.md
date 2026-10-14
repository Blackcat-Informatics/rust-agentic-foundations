# R5c review: shared glob scope tables under a glob budget

Reviewer: independent single pass, 2026-10-13 22:41 to 23:30 MDT.
Status: first pass BLOCKED on REQUIRED 1; the recheck below (2026-10-13 23:48 to 23:58 MDT) finds it fixed: PASS.
Tree: uncommitted R5c diff on d145545 (new `glob_scope.rs`; edits to budget, index,
lib, meter, options, resolve, settle, registry_scan, load_budgets, use_resolution,
source-index.md, SECURITY.md).

## Scope

All experiments ran on copies, never on the worktree. `/tmp/r5crev/base` is
`git archive d145545`. `/tmp/r5crev/new` is the same archive with
`git diff | git apply -p0` and the new `glob_scope.rs` copied in; `diff -r` against
the worktree `crates/` is identical. For the gate tests, `/tmp/r5crev/gatecopy` is
that copy in a fresh scratch git repository, because `docs_contract` runs
`git ls-files`. Probe sources are in `/tmp/r5crev/probe/src/main.rs` (R5c) and
`/tmp/r5crev/probe-base/src/main.rs` (d145545). Logs are in `/tmp/r5crev/`. Every
probe ran under `ulimit -v`.

I checked these against the contract (`### R5c`, the Per-item gates, the Registry
harness and R3's rule that what a load holds is never more than it is charged):
1. Whether the charges are upper bounds, including at what point a charge is made
   relative to the allocation it covers.
2. Whether the two new `use_resolution` tests agree with rustc.
3. Whether the resolve dumps changed against d145545.
4. Whether the Contract and SECURITY figures are consistent with the code.
5. Whether the unbudgeted part is described honestly.
6. The gates.

## Evidence

**Gates (all on the snapshot copy, `CARGO_BUILD_JOBS=6`).** All pass:

| Gate | Result |
| --- | --- |
| `cargo test -p raf-source --locked` (debug) | rc=0 (every binary ok) |
| `cargo test -p raf-source --release --locked` | rc=0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | rc=0 |
| `cargo test --locked --test docs_contract --test repo_gates` | rc=0 (139 and 57 passed) |

**Resolve dumps (`/tmp/r5c/probe-src/dump.rs`, rebuilt by me against each tree).**
Every item path and import path, resolved in every package, plus trait keys. The
d145545 and R5c outputs are byte-identical on all three inputs:

| Input | Lines | Result |
| --- | --- | --- |
| This repository (the patched tree) | 9474 | identical |
| katamari | 47513 | identical |
| purrdf | 256374 | identical |

**Registry (`raw/registry-scan-R5c.txt`).**
* 1204 OK and 29 ERR. The ERR kinds are Syntax 2, Target 4, Tree 2, UnresolvedMod 7,
  UnresolvedTrait 5 and UnsupportedItem 9.
* No budget refusals.
* Against R5b, the only change in outcome is `web-sys@0.3.106` (ERR memory to OK).
* No crate's item count changed.

**rustc (1.101.0-nightly 2026-09-30) on the two new fixtures.** I compiled each
fixture as crate `demo`, edition 2021, in every order the tests use. I gave each `Y`
function a distinct return type and assigned the probed path to `()`, so rustc's
type error names the item it resolved.
* `a_cycle_left_to_a_glob_reads_all_that_the_glob_brings_in_once_its_round_is_decided`:
  in all 28 orders, rustc reports E0659 for `m5::Y`, `m21::Y` and `m25::Y` (glob
  `m58::*` against glob `m31::*`, which leads to `m33`), and resolves `m38::Y` to
  `m33::Y`. RAF's new answer is right, and the old single item was wrong.
* `imports_that_read_a_name_through_a_glob_agree_with_the_glob_on_an_ambiguous_re_export`:
  rustc itself depends on the order here.
  * In the 18 even-indexed orders it compiles cleanly, and `m1::Y`, `m49::Y`,
    `m43::Y` and `m46::Y` are all `m28::Y`.
  * In the 18 odd-indexed orders it reports E0659 for all four.
  * `m42::Y` is `m0::Y` in every order, as the test asserts.
  * RAF's order-independent `Ambiguous` is the conservative answer (see ADVISORY 2).

**Probes reproduced from the evidence.** S-3, the chain, the prelude, the fan and
`stuck50`/`stuck100` match `raw/R5c-falsification.txt` and `/tmp/r5c/perf-final.txt`
within host noise (load average 17 to 25).

## Findings

### REQUIRED

**1. `make_view` allocates its gather before charging it, once for each glob that
reads the same table, so real memory passes the glob budget without bound before
the refusal.**

In `glob_scope.rs:541-579`, `make_view` copies the lists of every read of the
target into `content`:

```rust
for read in self.reads_of(target) { ...
    content.entry(name.as_str()).or_default()[at].extend_from_slice(ids);
```

It charges `gathered` only afterwards (`self.charge_work(gathered, site, meter)?`).
Several globs of one module that reach the same target with the same class all read
one view. That is valid Rust: rustc accepts repeated `pub use crate::a::*;` with
only an `unused_imports` warning. Each read is charged only `READ_BYTES`, but each
contributes a full copy of the view's lists to `content`. So the gather holds
(globs × names) item ids before any charge is made.

Reproducer:
* `src/a.rs` holds N lines of `pub struct S{i};`.
* `src/t.rs` holds R lines of `pub use crate::a::*;`.
* `src/u.rs` is `pub use crate::t::*;`.
* `lib.rs` is `pub mod a; pub mod t; pub mod u;`.

`/tmp/r5crev/probe`, case `dup`:

| N × R | d145545 | R5c |
| --- | --- | --- |
| 20000 × 500 | OK, 48.5 s, 76 MiB | OK, 11.0 s, glob_bytes 347 MB, hwm 204 MiB |
| 20000 × 1500 | | ERR GlobBytes at `src/u.rs:1`, 12.5 s, hwm 664 MiB |
| 40000 × 1500 | OK, 271 s, 152 MiB | ERR GlobBytes, 40.4 s, hwm 1270 MiB |
| 60000 × 3000 (`ulimit -v` 3 GiB) | | `memory allocation of 32768 bytes failed`, abort (rc=134), no typed error |

Item bytes are about 67 MB at N=40000, so almost all of the 1270 MiB is the uncharged
gather.

Consequences:
* R3's rule (held ≤ charged at every moment) is broken.
* The Contract's sentences "a refusal comes at the charge that would make what is
  held pass the limit" and "the scope tables are within figure (1)" are false.
* The 4.6 GiB "cannot exhaust a machine" bound does not hold. R is limited only by
  the token budget (about 285k globs per file) and N by the item budget, so the
  product is far past any host.
* It is also a new false refusal: the 40000 × 1500 input that d145545 loads at
  152 MiB is now refused.

Required fix and checks:
* Gather each distinct view once. All reads of one view that the class allows carry
  identical lists, so this gives the same result.
* Charge the bound before allocating. For example, sum the sizes of the distinct
  views first. Then nothing is allocated past a charge.
* Add a fixture: R duplicate globs of one module of N names, then a glob of that
  module.
  * `glob_bytes` must not grow with R × N, only with R + N.
  * A low `max_glob_bytes` must refuse the load before the gather.
* Re-run the `dup` probes above. Each must stay at or below its charge (counting
  allocator), and `dup 40000 1500` must load as it does on d145545.

### ADVISORY

**1. Repeated reads of one table also cost (globs × names) time.**

When the importer's view already exists (`dupu`: `u` declared before `t`), each
repeated glob is active, and `read_glob` passes every name of the view through
`arrive` and `add_to_view`, which find nothing new. `dupu 40000 1500` loads OK in
162 s at 80 MiB.

This is not a memory problem. But the Contract's claim that "a chain of globs costs
what each link brings in, once" holds only for chains: a module that reads one table
through R globs pays R times. Deduplicating reads by (module, view, allowed
visibility) for passing on would fix this together with REQUIRED 1.

**2. Record rustc's order dependence for the second new test.**

The test asserts `Ambiguous` in all 36 orders. rustc gives `m28::Y` with no error in
18 of them and E0659 in the other 18 (the even-indexed and odd-indexed
`rotations()` orders respectively). RAF's order-independent `Ambiguous` is the
defensible choice: it agrees with rustc's error whenever rustc reports one. The test
comment and the Contract bullet should say that rustc's answer depends on the
declaration order here, so that a later reader does not "fix" RAF to match one
order. The first new test matches rustc in every order.

**3. The unbudgeted undecided-import graph is disclosed, but the example
understates how it grows.**

The figure "100 modules → 20 to 32 s at 87 MiB" reproduces with the implementer's
generator (`/tmp/r5c/stuck100.rs`). Larger and nearby inputs grow much faster:
* The same generator at 200 modules (`/tmp/r5c/stuck200.rs`) loads in 613 s at a
  peak of 555 MiB resident, with only 9.7 MB charged to the glob tables. That is
  already past the 512 MiB glob budget in unbudgeted memory.
* A nearby shape, where each `m{i}` has `pub use crate::*; pub fn g{i}() {}
  pub use crate::Nothing{i};` and the root globs each module, takes 35 s at
  n=50 (d145545: 41 s). At n=100 it is killed at 900 s.

The R5c Contract item does not cover this, and it is not a regression, so it does
not block. The Contract and SECURITY text should still add the 200-module point
(or a stated growth rate), so that "they grow with ... and the time grows faster
still" is backed by a figure. That figure would show the unbudgeted memory can pass
the glob budget.

**4. Minor transient allocations are not charged.**

* `glob_visibilities` builds `vis_of: Vec<Vec<Vis>>` with one clone per item and per
  `use` before it deduplicates.
* `globbed()` builds a fresh `Bound` of `Origin`s on every lookup.

Both are bounded by a constant factor of what the item and fact budgets already
charge, and both are dropped at once, so they do not need a fix. A line in
`glob_scope.rs`'s module doc naming them as transient would keep the "every charge
is an upper bound" claim precise.

### Figures (focus 3)

The arithmetic is consistent with the code:
* `BUDGETS` gives 512 MiB of item bytes, 256 MiB of total bytes (source text),
  256 MiB of fact bytes and 512 MiB of glob bytes, which is 1.5 GiB for figure (1).
* Figure (2) is 2,000,000 tokens × 1664 bytes, about 3.1 GiB.
* So "about 4.6 GiB" is right, and the SECURITY totals (256, 512, 256 and 512 MiB,
  1.5 GiB) match `budget.rs`.

But the statement that the glob tables are inside figure (1) is falsified by
REQUIRED 1. Once that is fixed, the text is accurate.

### Mutation rule

I did not re-run the mutations. `raw/R5c-falsification.txt` §3 records:
* the at-limit fixture failing under the "refuse" mutation;
* the sharing test failing under the "share" mutation;
* the charge mutation caught by `modules_that_glob_one_another_...`.

It also records that both new `use_resolution` tests fail on the tree before the
change (44e18b7). The `load_budgets` tests cannot compile there, because they need
the new API.

First-pass verdict: BLOCKED (superseded by the recheck below).

## Recheck

The tree is the worktree's uncommitted diff at 23:48. I copied it into
`/tmp/r5crev/fix` (`git archive d145545`, then the diff, then `glob_scope.rs`), and
`diff -r` against the worktree `crates/` and `SECURITY.md` is identical.

The fix makes four changes to `make_view` and `read_glob`:
* `make_view` charges the list of the target's reads first.
* It gathers each distinct view once, from `sources` sorted and with repeats
  removed.
* It charges the whole gather through `gather_bytes` before allocating.
* `read_glob` removes repeated reads by `read_by` (glob, view) and by `passing`
  (module, view, visibility).

### Probes with a counting allocator

The probe is `/tmp/r5crev/probe-fix/src/main.rs`, a global allocator that tracks the
heap peak during `load_with`. It ran in release under `ulimit -v` 3 GiB. "Sum
charged" is total, item, fact and glob bytes.

**My REQUIRED 1 shapes.** No abort, and the heap peak stays at or below the sum
charged in every case:

| Probe | Load | glob_bytes | Heap peak | Sum charged |
| --- | --- | --- | --- | --- |
| `dup 40000 1500` | 0.48 s | 65.4 MB | 73.7 MB | 143.6 MB |
| `dupu 40000 1500` | 0.46 s | 40.4 MB | 70.6 MB | 118.6 MB |
| `dup 60000 3000` | 0.76 s | 98.6 MB | 105.2 MB | 216.5 MB |
| `dup 20000 500` | 0.23 s | 32.6 MB | 36.6 MB | 71.4 MB |

* `dup 60000 3000` used to abort; it now loads.
* `dup 40000 1500` with `max_glob_bytes` set to 20 MB gives a typed GlobBytes
  refusal at `src/t.rs:1` in 0.17 s, with a heap peak of 49 MB, almost all of it the
  index (items alone are charged 67 MB on the full load).
* Before the fix these were 1270 MiB with a refusal, an abort, and 162 s.
* The figures in the new Contract sentence (94 MiB charged, a 100 MiB heap peak and
  206 MiB in all for 60000 × 3000) match mine.

**Variants aimed at the new deduplication.** All of these give heap peak ≤ sum
charged, and every resolution is right:
* `vis 20000 2000`: `t` sits 40 modules deep. Its 2000 globs of `a` cycle through 44
  visibilities: `pub`, `pub(crate)`, private, `pub(super)` and `pub(in <each of the
  40 ancestors>)`. Visibility-path tables are retried throughout.
  * `u` (a child of the same parent) globs `t`, and the root module `w` globs `u`.
  * Loads in 1.14 s. glob_bytes is 38.9 MB, the heap peak 42.2 MB, the sum charged
    79.9 MB.
  * `…::u::S19999` and `p::w::S19999` both resolve to `a::S19999`.
* `mutual 20000 1000`: `t1` globs `a` once and `t2` 1000 times, `t2` globs `t1`
  1000 times, and `u` globs `t2`.
  * Loads in 0.41 s, with a heap peak of 53.2 MB against 73.0 MB charged.
  * `u::S19999` and `t1::S0` resolve to `a`.
* `explicit 2000 4000`: in `t`, 4000 globs of `a` are interleaved with 2000 explicit
  `pub use crate::b::S{i};` imports of the same names. `u` is declared first.
  * Loads in 2.47 s, with a heap peak of 14.1 MB against 22.0 MB charged.
  * `u::S0` and `u::S1999` resolve to `b`, as the explicit import shadows the glob.
* `mixvis` (repeated globs of differing visibility, in two declaration orders):
  * Private globs then a `pub` glob, or a `pub` glob then private ones: `u::S0` is
    `a::S0` in both orders. So the read that passes on is not lost when the private
    duplicate comes first; the deduplication key includes the visibility.
  * Private and `pub(crate)` globs: `u::S0` is `a::S0`.
  * Private globs only: `u::S0` is NotFound and `t::S0` is `a::S0`. A sibling
    cannot see private globs.

### Regression and gates

**Dumps.** The resolve dumps of this repository, katamari and purrdf with the fixed
tree are byte-identical to d145545.

**New fixture.** `load_budgets::repeated_globs_of_one_module_are_charged_once_for_what_it_holds`
passes on the fix. Run on the pre-fix R5c copy, it fails with "64391 bytes for each
more glob of two thousand names".

**Gates.** All of these were run on a snapshot copy, and all pass:
* `cargo test -p raf-source --locked` (debug): rc=0
* `cargo clippy --workspace --all-targets --locked -- -D warnings`: rc=0
* `docs_contract` (139) and `repo_gates` (57)

**Advisories.**
* 1 is resolved by the passing deduplication (`dupu` went from 162 s to 0.46 s).
* 2 is applied, in the test comment and the Contract bullet.
* 3 is applied: the 200-module point is now in the Contract and in SECURITY.
* 4 is applied, in the module doc.

### Remaining advisory (not blocking)

* Two new prose sentences in `source-index.md` are not wrapped like the
  surrounding text. They are the Work and memory bounds sentence that ends "…is
  charged before it is allocated. While the tables settle, a load also", and the
  test-list sentence that ends "…one byte below what it is charged. `usage()`
  reports…". This is cosmetic; the docs gates pass.

VERDICT: PASS
