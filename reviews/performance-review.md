# Performance and resource-bound review

PR 13 / issue 2, HEAD 6c744af, scope `crates/raf-okf crates/raf-source fuzz`.
No tracked file was edited. Scratch programs live in `/tmp/perf/{okf,src,src_patched,wsx,disk}`
(path dependencies on the worktree; `wsx` is a patched copy of raf-source used only to
validate one fix). Release builds unless stated. The machine was shared and loaded, so
absolute times vary by about 2x between runs; scaling exponents are what matter.

## Verdict

* The stated numeric caps are enforced exactly where they are stated (table below).
* One contract breach in raf-okf: a 1 MiB document can allocate tens of GB. REQUIRED.
* raf-source has three unbounded-resource paths that the "Resource bounds" text either
  denies or does not mention (item records, glob scope tables, settle loop), and one
  inaccurate cost statement (module-path budget). REQUIRED for the item-record bound and
  the inaccurate statement; the rest advisory.
* The 64 MiB parse thread is cheap (3% of a real load). The resolver has no blow-up on
  lookups, only on a pathological path string.

## 1. Are the stated bounds enforced (public API)

| Bound | Probe | Result |
|---|---|---|
| okf text 1 MiB | `Document::parse`, 1048576 B / 1048577 B | ok in 6.1 ms / `TooLarge` in 30 us, line number found |
| okf YAML depth 32 | 31/32/33/500 nested maps; same with a list at the bottom | 32 levels ok, 33 refused, 500 refused in 19 us, 3 MB RSS; zig-zag 100k lines refused by size |
| anchors/aliases/merge/tag/flow map | `&x`, `*x`, `<<:`, `!!str`, `{}`; also inside lists and flow lists | all refused, positioned, sub-17 us; a quoted `"&x"` is accepted (correct) |
| okf block nesting 64 | `> ` and `- ` runs of 63/64/65, 10k, 200k, 500k | 64 ok, 65 refused; 500k markers refused in 27 ms, 6 MB |
| okf link part 4096 | dest, `<dest>`, title at 4096/4097 | 4096 ok, 4097 positioned error |
| source file cap 8 MiB | `MemoryTree`, 8388608 B / 8388609 B file; manifest +1 | ok (22 ms) / typed error in 55 us; manifest typed error |
| budgets (public `parse_source`) | delimiters 128/129, angles 64/65, prefix 256/257, bars 128/129, else-if 256/257 | each at limit ok, limit+1 `Budget` error with line |
| toml recursion | array nesting 100/1000/20000/100000, inline tables to 100000 | typed `Manifest` error, 0.1-7.6 ms, no crash |
| module-path budget 262144 | `diamond` chain of files declaring the next twice | depth 17 refused after 0.26 s; depth 16 passes (see F4) |

## 2. Measurements

### raf-okf (`/tmp/perf/okf`, `okfperf <case>`)

* Linear and cheap at 1 MiB: many keys 22 ms/24 MB; 520k prose lines 130 ms/127 MB;
  1M blank lines 94 ms/124 MB; 250k headings 107 ms/89 MB; 140k reference definitions
  46 ms/55 MB; link-scanner adversaries (long dest then failure, unterminated angle or
  title, `](` x2000 repeated) 6-10 ms each; code-span adversaries 45-92 ms. No quadratic
  in the link scanner, the YAML reader or the line splitter. Memory amplification is about
  150x the text, bounded.
* `Profile::validate` on 1 MiB documents: 40 ms (placeholders, unexpected sections).
* **Quadratic memory** (F1): `okfperf slug <h> <prose|blank>`, a `## ` heading of h bytes
  followed by short lines to 1048576 B:

  | h | time | peak RSS |
  |---|---|---|
  | 1000 | 0.50 s | 1.1 GB |
  | 10000 | 2.6 s | 10.0 GB |
  | 30000 | 7.2 s | 29.3 GB |
  | 60000 | allocation failure, process aborted (ulimit -v 30 GB) | - |

  Peak RSS = lines x slug length, to the byte.
* validate: `affects` x `changes` (F2): n=5000 51 ms, n=20000 479 ms (n=40000 does not fit
  in 1 MiB). Sections x generated blocks (F3): g=10000 gives 302 ms.

### raf-source (`/tmp/perf/src`, `srcperf <case>`)

Workspace::load on this repository (`srcperf self <worktree>`, 90 files, 1.09 MB of source,
4 packages, 40 targets, 2178 items):

| Build | load | peak RSS |
|---|---|---|
| release, quiet box | 85-113 ms | 17 MB |
| release, loaded box | 188-230 ms | 17 MB |
| debug | 0.82-0.97 s | 19-21 MB |

Parse thread cost (`srcperf threads <worktree>`, same 90 files):

* empty 64 MiB-stack thread spawn+join 37.5 us; empty 2 MiB 25.7 us. The stack is mmap'd
  lazily, so the size costs about 12 us, not 64 MiB of memory (peak RSS stayed at 24 MB).
* `parse_source` (pre-scan plus thread) on the 90 files 99 ms against 72.5 ms for bare
  `syn::parse_file` on the current thread. Of the 27 ms difference, 90 x 37 us = 3.4 ms is
  thread creation (about 3% of the load); the rest is the pre-scan, about 300 MB/s.
* tiny file: 54 us per `parse_source` against 1.4 us for bare `syn` (absolute cost is
  small). 20000 files on `DiskTree`: 1.22 s (61 us per file, linear); 5000 packages on
  `DiskTree`: 0.70 s. Verdict: the per-file thread is not a performance problem.
* Pre-scan itself is linear: 8 MiB inputs of idents, lifetimes, chars, strings, raw
  strings, block comments, `<`, `>`, `&`, `-`, `|a|` all complete in 2-30 ms before the
  parser starts. Whole-load cost on a valid 8 MiB file is `syn`: `a<b; ` repeated took
  5.1 s and 2.6 GB (about 300x the file size); 8 MiB of tokens costs 0.4-1.0 GB.
* Glob chain, `mod m_i { pub use super::m_{i+1}::*; }` (F5): L=250 0.04 s, 500 0.16 s,
  1000 0.68 s, 2000 3.05 s, 4000 10.7 s (source 154 KB). Named re-export chain is linear:
  L=2000 19 ms, 4000 35 ms (even declared in reverse).
* Unresolved-import settle (F6): independent `use serde::xN;` in one module: N=1000 7 ms,
  10000 1.57 s, 50000 15.0 s. Realistic spread (8 imports per module, std/serde/crate):
  800 uses 10 ms, 4000 71 ms, 8000 336 ms, 16000 1.20 s, 32000 4.29 s (exponent 2.0).
  Import 2-cycles: N=100 31 ms, 400 0.99 s, 800 6.3 s, 1600 70.7 s (source 120 KB).
  Re-export chain that ends outside the workspace: L=500 0.38 s, 1000 2.45 s, 2000 17.4 s.
  With a 10-line index change (see F6) in a /tmp copy: 50000 external imports 15.0 -> 0.29 s;
  32000 realistic 4.29 -> 0.27 s; 2-cycles N=1600 70.7 -> 9.4 s; external chain L=2000
  17.4 -> 3.2 s.
* Glob scope tables (F7): one module of N fns globbed by G modules: N=1000,G=100 327 MB;
  N=1000,G=400 1.29 GB; N=4000,G=400 5.1 GB / 11.8 s; N=4000,G=1000 12.8 GB / 32 s
  (source 107 KB). Realistic private prelude (`use crate::prelude::*;` in every module):
  50 names x 500 modules 151 MB; 150 x 1500 1.31 GB / 1.9 s; 150 x 3000 2.6 GB / 3.5 s;
  300 x 3000 5.2 GB / 7.0 s. About 5.8 KB per (module, glob name) pair.
* Module-path budget (F4): `diamond` depth 16 (passes the budget): 2.14 s, 1.58 GB, 524285
  items, from 0 bytes of extra source (one file per level, 3 lines each). One 400-fn leaf
  declared 4000 times (source 114 KB + 6.7 KB, 4000 paths of a 262144 budget): 1.6 M items,
  3.1 GB, 3.4 s.
* Discovery on `DiskTree` (F8): a single manifest at the bottom of a chain of directories:
  depth 100 17 ms, 200 103 ms, 400 586 ms, 800 5.3 s; depth 1990 did not finish in two
  minutes (cubic).
* Resolver (F9): normal resolves 2-4 us on 50000 items; a test-crate path 31 us at 50000
  items (a scan of all items per call). A path of 1000 segments takes 2.6 ms, 10000 244 ms,
  50000 6.9 s (quadratic; 150 KB of text).
* Members glob (`MemoryTree`): `crates/*` over 5000 dirs 1.24 s but `DiskTree` over 5000
  real member crates 0.70 s total, so the `MemoryTree` super-linearity is a test-double
  property (also seen for `files N=40000`: 7.2 s vs 1.2 s for 20000 on disk). A hostile
  pattern `*a*a*a...b` (3000 stars) against 200 long names: 10 ms, so the matcher is not
  exponential. 80000 `[[bin]]` tables sharing one root: refused in 122 ms.
* Stack (F10): the module walk recurses on the caller's stack: inline nesting 127 completes
  on 256 KiB in release; in debug it needs more than 512 KiB and passes on 1 MiB (so
  about 4-8 KB per level).
* Uncounted recursion shapes (F11), release, 64 MiB thread, `srcperf shape`: process abort
  by stack overflow at about 103k `x()()()...` (206 KB), 108k `1+1+1...` (216 KB), 120k
  `x?????...` (120 KB), 120k `x.a.a.a...` (240 KB), also `x[0][0]...`, `x as u8 as u8...`,
  `.await`, `.m()` and `a*a*a...` chains, `return ` x ~20000 (140 KB). All are far under the
  8 MiB cap and under every budget. In a debug build the budget-composite shapes abort
  (below).

## 3. Findings

### F1. REQUIRED, high: quadratic memory in the body model; a 1 MiB document allocates tens of GB

`crates/raf-okf/src/body/mod.rs:131-139,173` and `crates/raf-okf/src/body/marker.rs:191`
(and `prose.rs:12`, `marker.rs:24`). `section_of: Vec<Option<String>>` clones the section
slug into every line of the section, `prose.push(ProseLine { section: section_of[index].clone() ..})`
clones it again for each prose line, and each PROPOSED block clones it again. Cost is
lines x slug length, and both are only bounded by the 1 MiB text. Measured: 1 KB heading
plus short lines = 1.1 GB; 30 KB heading = 29 GB; 60 KB = allocation failure, abort.
`Document::parse`, `Body::parse`, `Index::parse`, `Document::new` and `replace_spans`
all reach it. The contract (`okf-profile.md` Resource bounds, line 96-98) says "never a
panic, abort or unbounded allocation"; the parse doc comment says it "never allocates
beyond the size of the parsed result". No test or fuzz target exercises it.

Fix: keep `section_of` as `Vec<Option<usize>>` (index into `sections`) and store
`Option<Arc<str>>` (or the section index) in `ProseLine` and `ProposedBlock`; the public
accessors still return `Option<&str>`. Then memory is O(lines + sections x slug). Add a
test with a 20 KB heading and 500k lines under a counting allocator (the dev-dependency
`cap` is already present) asserting a bound proportional to the text.

### F2. Advisory, low: O(affects x changes) in validate

`crates/raf-okf/src/profile/mod.rs:565` (`changes.iter().any(..)` per affects entry).
Measured 479 ms at n=20000 each, about 1 s at the 1 MiB maximum (about 28k x 28k).
Fix: build a `HashSet<&str>` of `changes` once.

### F3. Advisory, low: sections x generated blocks in validate

`crates/raf-okf/src/profile/body_rules.rs:47` (`visible_content` loops over every
generated block for every wanted-named section). 300 ms at 43k sections x 10k blocks.
Fix: binary-search the blocks by `outer.start` (they are sorted), or have the section
span carry the first block index.

### F4. REQUIRED (documentation) and advisory (bound): the module-path budget does not bound cost and the stated cost is wrong

`crates/raf-source/src/modtree.rs:221`, `budget.rs`, `docs/capabilities/source-index.md:187-198`.
The text says the cap admits "a quarter of a million records, a fraction of a second and a
few tens of megabytes". Measured: depth-16 diamond (passes the cap) 2.14 s and 1.58 GB with
524k items; a 120 KB input (400-fn leaf declared 4000 times, 4000 paths) 1.6 M items, 3.1 GB,
3.4 s. The unit counted is the arrival, but the allocation is per item per arrival, so
the cost is paths x items-in-file and nothing bounds the second factor: a 7 MB lib.rs of
`#[path] mod` lines times a 100 KB leaf is of the order 10^9 item records (terabytes).
This is an unbounded allocation reachable from source text, in a crate whose Resource
bounds paragraph presents the walk as bounded.
Fix (required): add a budget on total item records per load (for example 4 M, `Budget::Items`),
checked where items are recorded, with a typed `Budget` error; and correct the sentence
to the measured figures. Advisory: also count files-times-items, or store the items of a
re-walked file once and reference them per arrival.

### F5. Advisory, medium: glob re-export chains are quadratic or worse; the doc claims linearity only for named chains

`crates/raf-source/src/settle.rs:510-533` (`rebuild_globs` repeats `glob_pass`, which
rescans every glob and every visible name, until no change; one pass per hop of a glob
chain) and `settle.rs:331` (`visible` clones every name of the target on each call).
10.7 s for a 154 KB file of 4000 chained globs; the cap at 8 MiB allows chains of
hundreds of thousands. The sentence "The work is linear in the length of a chain of
re-exports, and nothing caps it" (`settle.rs:429`, `source-index.md`) is true for named
chains (measured linear) and false for glob chains.
Fix: propagate glob bindings with the same work-queue as named imports (wake only the
globs watching a module whose scope changed, and pass only the new names), or cap glob
chains and refuse with a typed `Budget` error.

### F6. Advisory, medium (high for a large workspace): quadratic scan in the settle loop

`crates/raf-source/src/resolve.rs:274-286` (`unresolved_uses` scans the whole `uses` vector)
called from `blockers` (`resolve.rs:291`) for every stuck import in every round
(`settle.rs:459-475`). With U imports in the entire workspace the cost is O(U^2) per round
and the number of rounds is the longest chain or cycle. Realistic spread of 32000 imports:
4.3 s (exponent 2.0). This repository has about 350 `use` lines (not a problem); a
workspace of 100k imports is projected at 40 s from this step alone.
Fix (validated in a /tmp copy): build `uses_by_name: BTreeMap<(module, name), Vec<usize>>`
once at the start of `settle` and have `unresolved_uses` read it; results above (15.0 s to
0.29 s, 4.29 s to 0.27 s). The remaining quadratic part is one round per leaf of a chain
or cycle (`settle.rs:459-480`): settle all leaves and, for a cycle, a whole strongly
connected component per round.

### F7. Advisory, medium: glob scope tables cost about 5.8 KB per (module, imported name)

`crates/raf-source/src/settle.rs:331-368` (`visible` returns owned `(String, usize, Bound)`
per name, `glob_pass` builds a full `fresh` scope table per pass, and each binding carries
an `Origin` with a `Link` with two `String`s). A prelude of 150 names glob-imported by 1500
modules takes 1.3 GB and 1.9 s. This is memory proportional to modules x names, the
usual shape of a real monorepo (`use crate::prelude::*;` in every module), and a 107 KB
source file can reach 12.8 GB.
Fix: do not materialise glob imports per importing module; resolve a name through the
list of glob targets lazily (with a memo), or intern names (`Arc<str>`) and use compact
origins. At minimum, add a typed budget on glob entries per load.

### F8. Advisory, low: discovery cost is cubic in directory depth

`crates/raf-source/src/discovery.rs:133-156` (`visit` recurses once per directory and
calls `Tree::entry` and `Tree::list`; `DiskTree` calls `fs::canonicalize` on the path in
each, `tree/disk.rs:96-98,272,287`), so the cost per level is quadratic in depth and the
total cubic. Also the recursion runs on the caller's stack. 5.3 s at depth 800; more than
2 minutes at 1990 (the longest path that fits PATH_MAX). Deep trees do not occur in real
workspaces, but `Workspace::load` walks every non-package directory including vendored
trees. Fix: refuse a directory depth above a stated bound (for example 128) with a typed
error, and/or resolve components with `openat` from a held directory descriptor instead
of canonicalising full paths. Also `discovery.rs:104` scans all found packages for every
package (O(P^2), harmless below 10^5 packages); use a `BTreeMap<name, manifest>`.

### F9. Advisory, low: resolver worst cases are only on adversarial path strings

`crates/raf-source/src/resolve.rs:376` (`segs[..taken].join("::")` for each prefix) is
quadratic in the number of segments: 50000 segments take 6.9 s, and a binding line in an
OKF document may be 1 MiB (about 350k segments, minutes). `resolve.rs:467-481`
(`shadowed_namespace` filters all items for every prefix of the crate path) is O(items)
for each resolve of a test, bench, example or bin item: 31 us at 50k items, so a docs
contract with thousands of test bindings pays items x bindings.
Lookups, `walk`, and cycle detection did not blow up in any probe. Fix: return
`ResolveError::Invalid` for a path over, say, 64 segments (no real item path is deeper
than module nesting 128), and index `Mod` items by path in `Index` once.

### F10. Advisory, low: caller-stack requirement of the module walk is undocumented

`crates/raf-source/src/modtree.rs` recurses on the caller's stack to depth 128 (admitted in
the docs). Measured: fits 256 KiB in release, but needs about 1 MiB in debug (512 KiB
aborts). A caller on a small thread (a 512 KiB worker, a debug test thread configured
smaller than the default 2 MiB) aborts instead of getting an error. Fix: state the figure,
or run the walk on the same 64 MiB thread helper, or make it iterative.

### F11. Advisory (known and documented), but wider than documented: ordinary expression chains abort the process

`crates/raf-source/src/prescan.rs`, `budget.rs`. The docs list `return`, `&mut`, `*const`,
`fn() ->` and `a =` as uncounted. The same abort happens at about 100-120 k units of the
everyday left-nested shapes `x?`, `x.f`, `x()`, `x[0]`, `x as T`, `a + b`, `a * b`, `.await`
and `.m()` (120-240 KB, under the 8 MiB cap and every budget). That is a valid generated
file (a 110 k-term sum) and is not "unlikely". Separately, in a DEBUG build every
budget-composite shape aborts with the budgets all at their limits: 127 nested parentheses
each holding a run of 256 `!` (32 KB), `-`, `*`, `&` likewise, 127 levels each holding 63
nested `<` (24 KB), 127 levels each holding 128 `||` (49 KB). They pass in release. The
repository tests run in debug and the harness (`tests/overflow_harness.rs`) accepts the
abort as an allowed outcome, so the documented worst case is a process abort in the
default `cargo test` profile.
Fix: add one additive budget that tracks the real recursion depth: the number of open
delimiters plus the lengths of the unfinished prefix runs, `<` counts and closure heads on
the whole open path (cap about 2048), which bounds the composite shapes; and a budget on
the number of tokens at one delimiter level between `;` `,` `{` `}` (cap about 16384) for
the postfix and binary-operator chains. Both are one counter in the existing pass.

## 4. Required versus advisory

Required before merge:

1. F1 (raf-okf quadratic memory; the contract promises the opposite).
2. F4 (item-record explosion reachable under the module-path budget; and correct the
   "fraction of a second and a few tens of megabytes" statement, which is off by about
   50x in memory and 10x in time).

Advisory (may be moved to a tracked follow-up with a named item): F2, F3, F5, F6 (the
10-line index fix is cheap enough to take now), F7, F8, F9, F10, F11.

Not a finding: the 64 MiB thread per file (3% of load, lazily mapped), the pre-scan
(linear, about 300 MB/s), the glob matcher (no exponential case), TOML recursion,
`Document` link scanning and YAML reading (linear), and all numeric caps (exact at limit
and limit+1).
