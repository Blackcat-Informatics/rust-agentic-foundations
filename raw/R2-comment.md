**R2 fixed: modules are placed under their declared path** (commit 0617985, pushed)

- **Gap:** CA-F2 (HIGH). Whenever a module's file sorted before the file that declares it, its items were recorded under the parent's path. In this repository the binding gate accepted the non-existent `raf_okf::body::FenceBlock`, and valid crates failed with a false `SuperAboveRoot`.
- **Root cause:** `Index::build` read files in file-name order, and `Builder::ensure` let the child take a slot before it created the missing parent.
- **Fix:**
  - The walk records each file arrival in walk order, before following that file's own `mod` declarations, and the index processes arrivals in that order.
  - A module is created only by its `mod` declaration.
  - A reach whose module was not declared first is a fail-hard internal error.
- **Checks:** seven `module_tree.rs` placement fixtures:
  - a `mod.rs` child and a flat child that sort first;
  - the `#[path]` children `a`, `ga` and `z`;
  - a shared `tests/common` module reached from a later target;
  - `super::super`;
  - a file module inside an inline module;
  - paths that must not be found.

  Also: `this_repository.rs` (`raf_okf::body::blocks::FenceBlock` resolves, the misplaced path is `NotFound`), and a `docs_contract` negative fixture. **Every one fails on the old code**, which the implementer and the reviewer each checked separately.
- **Registry scan:** the new example `crates/raf-source/examples/registry_scan.rs` loads each crate in its own child process. Over 1,177 crates:
  - crates with misplaced items: 165 → **0** (2,757 items → 0);
  - `SuperAboveRoot`: 3 → **0**;
  - crates that load: 1,014 → 1,021.

  Every changed outcome is root-caused in the stage validation index:
  - jni, rustls and wasm-bindgen now stop at the macro-generated-impl class, which R4 owns.
  - web-sys's quadratic glob memory is unchanged by R2 (measured), and R5c owns it.
- **Gates:**
  - raf-source tests pass in debug and release (257+).
  - Workspace clippy with `-D warnings`: clean.
  - docs_contract 138, repo_gates 55.
  - `make reuse`: 211/211.
- **Review:** `tasks/R2-review.md`, VERDICT: PASS after recheck. One advisory is folded into R3 (a tighter macro oracle and a per-child memory cap in the harness). One pre-existing defect is folded into R4 (a lib module named `bin_crate`).
