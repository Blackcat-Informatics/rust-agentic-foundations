# Referenced artifacts — issue #27

Everything the issue names, checked against disk. Facts only: whether a
file is a viable port source, or may be relicensed, is decided in the plan.
Paths resolve against the main checkout, then against a sibling checkout
only when that repo is named (first path component, or in the prose just
before the reference). "not here; exists in sibling(s)" is NOT resolved.

## Paths

| Reference | Exists | Resolved in | Kind | Lines | Licence header |
|---|---|---|---|---|---|
| `crates/raf-source/examples/registry_scan.rs` | yes | repo | file | 671 | SPDX: MIT OR Apache-2.0 OR MulanPSL-2.0 |
| `examples/` | NO | not here; exists in sibling(s): biocode-2026-09-30, cf-config, llama-cpp-rs-logit-loom, purrassist, purrdf (+2 more) | — | — | — |

### History: `crates/raf-source/examples/registry_scan.rs`

- `bde54f0a6 docs(raf-source): the registry scan's clock allow says what it is: a real child polled against the wall clock`
- `01a15d808 test: the clock gate covers every crate and the root tests, and clippy refuses Instant, SystemTime and sleep`
- `bf08e459e feat(raf-source)!: seal the public types consumers bind to`
- `be71d40f9 fix(raf-source): bound glob scope tables with a byte budget`
- `b187300cd fix(raf-source): bound parser recursion with one additive budget`
- `cd7d4fb16 fix(raf-source): key files by identity and bound total work`
- `061798548 fix(raf-source): place a module under its declared path`

## Identifiers (files containing each at HEAD)

- `sleep`: 10
- `Usage`: 93

## Workspace packages named

- `raf-source`

## URLs

None.
