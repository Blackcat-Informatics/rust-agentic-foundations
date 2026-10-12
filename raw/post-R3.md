### Stage 2 fix R3 (`6120029`): trust model, containment, subprocess and parser contracts

**User decisions applied (verbatim)**
- "Workspace = build input (Recommended)".
- "Fail-closed, honest (Recommended)", for parser stack overflow.

**Gaps closed (security review M1, M2, L3, L4)**
- **Trust model.** SECURITY.md states that the checked workspace is trusted build input. RAF executes its cargo configuration exactly as `cargo build` and `cargo test` do.
- **M1, path containment.**
  - Every path is canonicalised and must lie inside the workspace root, or inside the reference package directory from `cargo metadata`.
  - Symlinks that escape and non-regular files are refused, and the file type is checked on the opened handle.
  - Fixtures cover symlinked files and directories, `link/..`, loops, FIFOs and manifest target paths.
- **M2, subprocesses.**
  - Every spawn lists its exact argv: no shell, `--frozen`, `--no-deps` for root discovery, and `$CARGO` or else `cargo` from PATH.
  - Document-derived values use `--`, single-argument embedding, or validation plus a pattern.
  - The complete gate requires proof that each bound test ran, since `--exact` alone passes on zero tests. Doc tests are listed and matched strictly, and `no_run`/`compile_fail` bindings are refused.
  - Git: minimum 2.30, full-ref default-branch discovery, and output caps.
- **L4, parser bounds.**
  - A configurable 8 MiB per-file cap.
  - Nesting budgets checked before `syn`, a 64 MiB parse stack and a fuzz target.
  - YAML aliases are refused, and ledger and metadata inputs are bounded.
  - A residual overflow aborts the run, so the gate fails and never passes.
- **L3.** The PGP fingerprint `52E5 4B13 1C29 7D1E AB14 3BAA 0C54 057C 8F9A 0090` and the key expiry are published. The user-ID mismatch is noted.

**Review:** `G-R3-review` by the security reviewer. It took 4 passes; the first three BLOCKED, each finding confirmed by experiment:
- syn overflow shapes;
- libtest's zero-test pass;
- rustdoc's whitespace splitting;
- the `--end-of-options` Git version;
- a fixture that contradicted the budgets.

Recheck 3 gave PASS.

**Checks:** R3 plan checks; docs_contract 46/46; `make issue-refs` rc 0; reuse 77/77; hook passed.

**Owner action, outside the repo:** the PGP key expires on 2026-12-24 and needs renewal.
