### Stage 2 fix R4 (`3e20c61`): licensing consistency and fresh backtranslation

**Gaps closed (G14a–e, A3)**
- **zh-Hans.** Deleted the purrdf-only "来源记录" phrase (G14a) and the distribution-package byte-identity sentence (G14b). Restored the third-party-licence statement as 第三方代码和继承材料保留其原有许可 (A3).
  - **Deletion-only rule.** It now applies at token level: a character-level difflib comparison against the source, with names substituted, shows a single insertion, the ® in the SPDX line.
- **Backtranslation (G14c).** A separate agent produced it; that agent could read only a snapshot of the new Chinese file. The record has new timestamps, 1,712 bytes and SHA-256 `e536de48…3d4b`, and lists all 18 ambiguities the translator flagged (G14d). It keeps the translator's statements separate from the repository check's assessment. The translator reported no fragments.
- **LICENSING.md (G14e).** Documentation contributions are CC-BY-4.0, and the code-licence sentences are qualified as code.

**Checks:** `make licenses` (the fingerprint matches), `make issue-refs`, reuse 77/77, and the hook passed.

**Review:** `G-R4-review`. The first pass BLOCKED: the count was off and the translator reply was not archived. Both are fixed; the recheck gave PASS.
