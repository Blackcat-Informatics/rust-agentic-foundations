### Stage 2 fix R11 (`45bc98e`, `edf93d5`): exit-audit findings

The first exit-gate completion audit returned FINDINGS-OPEN. Its findings are fixed:

- **F1 (MEDIUM), a gate regression (`45bc98e`).**
  - **Cause:** the Rust issue-reference gate spared a `#` that follows a word character. So `org/repo#12` and `Fixes#12` passed, though the Stage 1 grep caught them.
  - **Fix:** the exemption is removed. An all-digit token after `:` is no longer read as a colour, so `Fixes:#123` fails too. Colours stay legal in attribute values and whenever they contain a hex letter.
  - **Parity:** a parity table against the Stage 1 grep now differs only on the deliberate `fill="#000"` and `&#38;` cases.
  - **Hook:** it refuses a signed scratch commit that adds `Blackcat-Informatics/rust-agentic-foundations#12`.
- **F2–F5 (LOW), spec contradictions (`edf93d5`).**
  - **F2:** the OKF profile now describes links resolved against the tracked file list, and docs_contract refuses a link to a tracked symlink.
  - **F3:** `sole-name` is stated as the exception to the detectors' home-package rule.
  - **F4:** `migrate` also rewrites a current-format ledger written at an older `normal_form`, so the documented bump path exists. The normal-form page names `migrate` as the one reader of an older-version ledger.
  - **F5:** the complete gate states the proof for each binding kind: test, doc test, example.

**Checks:** docs_contract 113, repo_gates 42; `make clippy fmt msrv`; reuse 81/81; each set passes alone; the hook passed on both commits.

**Review:** `G-R11-review` gave PASS. One advisory, the `:` colour context (A1), was adopted.
