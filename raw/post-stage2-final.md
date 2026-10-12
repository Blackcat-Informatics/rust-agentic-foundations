## Stage 2 final summary (head `80c14d7`)

**Review path:** full. The change touches CI/supply-chain surface, git hooks, a contract test, and 21 specs that set contracts for later crates.

### Gaps resolved
The reviews were gap analysis, the completion audit, security, quality, historical fit, and the remediation-plan review. Their findings:
- **Gap analysis:** G1–G16, of which 1 CRITICAL, 2 HIGH, 5 MEDIUM and 8 LOW.
- **Security:** M1, M2 and L1–L5.
- **Quality:** H1, M1–M5 and L1–L8.
- **Exit audit:** F1–F5 and N1.

Every finding is **FIXED** or recorded as an evidenced false positive in `remediation-plan.md`. One gap is externally blocked: **G2, hosted CI**.

| Group | Commit(s) | Closes |
|---|---|---|
| R1 workplan docs | `f711aa2` | G1 (essay content), G4, G5, G6, G8, G12 |
| R2 census docs | `6f8891e` | G3 (Gate owns the census path), G5, G13a–g, G15a |
| R3 security specs | `6120029` | trust model, path containment, subprocess argv, test-ran proof, parser bounds, PGP fingerprint |
| R4 licensing | `3e20c61` | G14 (token-level deletions only; fresh isolated backtranslation) |
| R6 Rust gates | `2faa697` | G7, G9, G16-F7, Q-M5 |
| R7 document contract | `dba307f` | Q-H1 (capability inventory), Q-M1–M4, Q-Ls, G6/G8/G12 rules (46 → 112 tests) |
| R8 CI | `cb90d10` | G11, S-L1/L2, G16 |
| R9 cliff | `8ff3859` | G10 |
| R5 project docs | `df65ee4` | G15b–d |
| R11 exit-audit fixes | `45bc98e`, `edf93d5`, `80c14d7` | F1 (gate regression), F2–F5, N1 |

### User decisions this stage (verbatim)
1. "(1) but add an issue for (2)". The Gate owns the census path, and issue 12 covers the CLI running repository rules.
2. "Workspace = build input (Recommended)".
3. "Port gates to Rust (Recommended)".
4. "Fail-closed, honest (Recommended)", for parser stack overflow.

No scope cuts were made, and `.deficiencies` has no entries.

### Reviewer dispositions
- Every group has an independent review in `tasks/G-R*-review.md` ending in PASS.
- Several reviews blocked first, each on a real defect that was then fixed:
  - R3 (3 passes): syn overflow shapes, libtest zero-test pass, rustdoc name splitting, git version.
  - R6: Latin-1 files silently skipped.
  - R7 (4 passes): forgeable index, tag state checked per line.
- The exit audit caught a gate regression introduced during this stage. It is fixed, and a parity table against the Stage 1 gate now differs only on the deliberately spared colour and character-reference cases.

### Validation, kept separate

**Local validation: PASS.**
- `make check` on `80c14d7` returned **rc 0**: fmt; clippy `-D warnings`; docs_contract 114 + repo_gates 42; doc; `deny --offline`; reuse 81/81; MSRV 1.98.1; licenses; issue-refs.
- The pre-commit and pre-merge-commit hooks ran on every commit. In scratch clones with real signed commits, they refused:
  - a planted `#12`, including `org/repo#12`;
  - an altered licence;
  - a capability missing a section;
  - a CITATION version drift;
  - a no-conflict merge carrying a token.

**Exit-gate completion audit: PASS (pass 3).** Every in-scope criterion is demonstrated on `80c14d7`, except the next item.

**Hosted CI: UNVERIFIED (external blocker).** GitHub refused every run on this branch with "recent account payments have failed or your spending limit needs to be increased", most recently run 38229506850 on `80c14d7`. No CI step has ever executed. Only the owner can clear this.

**Unrun live acceptance:** none required. This issue ships no crate code.

### Owner actions outside the repo
- Clear GitHub billing, then re-run CI on the final head, or waive the CI half of AC1 explicitly.
- After merge, upload `docs/social-preview.png` in Settings and set the repository description from `docs/BRAND.md`.
- Renew the security PGP key before 2026-12-24.

### Stage 3 readiness
- **Ready for Stage 3** apart from hosted CI.
- The exit audit and the `80c14d7` qualification can be reused if the head is unchanged and the base delta is assessed.
