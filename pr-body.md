Closes #27

## What changes

The registry scan example reads no clock and does not sleep. The per-crate bound is now a counted work budget in the index, refused like its other budgets.

* **raf-source**: `Budget::Work` ("glob table work steps"), `LoadOptions::max_work` (default 2^31), `Usage::work`, charged at the glob-table sites that carry the superlinear cost (view gather, `pass_on`, `read_glob`, `export_ids`, and the supply search step). A load that passes it is refused with a `Budget` error at the charge that passes it. Counted on generated shapes: a chain of n globs costs 2n, a fan-in of private globs 4n^2 + 6n, the prelude pattern n(15n + 17)/2, the hub 2n^3 + 4n^2 + 4n.
* **registry_scan example**: no `Instant`, `Duration`, `sleep` or poll; the child is waited on; `--max-work N` (at least 1) replaces `--timeout`; `work=` is printed per crate and in the largest-value block; a refusal is an `ERR Budget` line.
* **Gate**: the clock gate now covers the raf-source `examples/` with no carve-out; the clippy allow in the example is gone.
* The source-index capability, its Verification text and assessments, the changelog, AGENTS and CONTRIBUTING are updated.

## Validation

`make fmt clippy test test-release doc deny reuse msrv fuzz licenses issue-refs` exit 0 locally. `cross-check` was not run locally (targets not installed); the example uses no Unix-only API, and hosted CI covers it. Every counted claim has a mutant in a scratch clone that fails it (charge dropped from a site, `>=` for `>`, a shared count, a child that ignores the limit, a planted clock reading in the example). The shipped example was run over a scratch registry: the 40-glob hub loads with `work=134560` and is refused at `--max-work 1000`; `--max-work 0` is refused. Over a local registry of 1271 crates the most work of one load was 34343004 steps (`web-sys`), a sixty-second of the default.

## Residual risk

* The work budget does not count the lookups made while settling (a module with n globs and n named imports of a re-exported name makes 4n+2 steps but about 25n^2 glob reads). Such a load can run long; this is stated in the capability and the example, and a comment on issue 28 names it for the settling-cost change.
* The default admits a hub of about 1,023 globs (about 1e12 instructions, from a per-step count); documented.
* A child stuck in work the counter does not see is not stopped.
* Removing `--timeout` and the `timeout` outcome changes the example's interface (changelog).

## Least confident

That 2^31 is the right default: it is chosen to admit every real crate with a large margin, not derived from a cost model; the hub it admits is slow.
