### Stage 2 fix R11, round 2 (`80c14d7`): migrate bump input

The exit audit's pass-2 recheck discharged F1–F5 and found one new LOW contradiction, N1, introduced by the F4 fix. It is now fixed.

**N1.** At a `normal_form` bump, `migrate` mapped fingerprints through the baseline report. That report drops single-member groups, so a clean ledger could not be rebuilt.
- `migrate` now reads the **index report** (`cargo census index`) written before the upgrade. The index report lists every unit.
- Only `body:`/`shim:` fingerprints are remapped. `table:`/`rule:` ids, patterns, distinct sets, the reference list and the file-size cap carry over unchanged.
- Fixtures cover each of these.

**Also fixed.**
- docs_contract refuses any tracked symlink.
- The migrate example uses `helpers-ledger.toml`.

**Checks.** docs_contract 114, repo_gates 42; `make clippy msrv`; reuse 81/81; the hook passed.

**Review.** `G-R11-review`, recheck 2: PASS. The reviewer walked the clean-job example through the algorithm.
