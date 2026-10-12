### Stage 2 fix R7 (`dba307f`): document contract hardened

**Gaps closed (Q-H1, Q-M1–M4, Q-L2/3/5/6/7, plus the G6/G8/G12 rules and carried items)**
- **Inventory table (Q-H1).** It pins the 21 capabilities with their states, the 2 architecture pages and the change records. Deleting or renaming a capability now fails. Before, deleting a capability together with its change record passed.
- **Resolved text.** Placeholder tokens are refused in any case, spacing or dotted form. A section left empty by comments is refused too.
- **Frontmatter parsing.** These now fail: unterminated quotes, unquoted `: `, a leading indicator character and a bare block indicator. Trailing comments are accepted.
- **PROPOSED markers.** They are detected in any case or spacing, and a near-miss is reported as malformed. Designed docs may not carry them. A section that holds one must appear in its change's `affects` (G6).
- **Bindings.** Designed and considered capabilities bind nothing (G8). Delivered capabilities bind at least 1 code path and at least 1 test (G12). A `moved:change:` target must exist.
- **Links.** Every tracked `*.md` is now checked, including `.github` templates and the README logo. Reference definitions are checked, and so are `href`/`src` attributes, with tag state tracked across lines and reset at blank lines and fences.
- **Placement.** Typed docs must live in their own directory. A generated index may hold only an SPDX header and generated blocks.
- **File reads.** Only regular files are read. FIFOs and symlinks are refused, and a swap after inspection is detected.
- **Negative fixtures.** Every rule now has one: docs_contract went from **46 to 112 tests**.

**Checks run**
- docs_contract 112/112 and repo_gates 40/40.
- `make clippy fmt msrv`.
- reuse 79/79.
- The hook passed.
- In scratch copies, quality-review mutation probes 7, 9, 18, 19, 22, 24, 25, 26, 29, 31, 32 and 34 now each fail with the named rule.

**Review:** `G-R7-review` (quality engineer), 5 passes. The reviewer's evasion probes found:
- a forgeable "generated" index;
- link tag state checked per line;
- prose tag state that persisted across blank lines.

Each was fixed with fixtures. The reviewer confirmed that no original test was weakened, and the final pass is PASS.
