### Stage 2 fix R2 (`6f8891e`): census docs

**Gaps closed**
- **G3 (HIGH), user decision "(1) but add an issue for (2)".**
  - `Gate::write_debt` is the write path for every ledger. It is called from a binary or xtask, never from a test.
  - The CLI handles ledgers that need no registered code. For a non-empty `[job.ext]` or an unresolvable `rule:` id it exits 2, naming the Gate path.
  - An unknown `rule:` id is a hard validation error in both the CLI and the Gate.
  - Migration sends dialect ledgers to the repository's census test and xtask.
  - The CLI mode that runs repository rules is tracked separately as an enhancement (issue 12).
- **G5, census `depends` edges.** ledger → source-index; cli → rule-api; rule-api → patterns, debt-ratchet and reference-scan. census-normal-form gets no edge, because it has no source-index usage.
- **G13a–g.**
  - a: `--frozen` replaces `--locked`.
  - b: `migrate --ledger` is separate from the common default.
  - c: shipping-only detectors are named, checked against the purrdf helper-census source: 9 shipping, 5 all.
  - d: an unreferenced pattern fails staleness.
  - e: `[job.ext]` is the only optional key.
  - f: an empty positive fixture set is refused.
  - g: the `normal_form` bump path is documented, with diff review of the record as the control.
- **G15a.** `raf-census = "0.1"`.

**Checks run**
- Every R2 grep and `yq` check in the plan passes.
- docs_contract 46/46, so the graph stays acyclic.
- `make issue-refs` rc 0.
- reuse 77/77.
- The pre-commit hook passed.

**Review:** `G-R2-review`. The first pass BLOCKED because the shipping-only detector list was inverted against the purrdf source. It was corrected with a source citation per detector, and the recheck gave PASS.
