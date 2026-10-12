# G-R4 review: remediation group R4 (licensing)

## Scope
Unstaged diff of LICENSING.md, docs/LICENSING.zh-Hans.md and docs/LICENSING-BACKTRANSLATION.md against the R4 spec, G14a-e and A3. Read-only. Nothing edited, staged or committed.

## Evidence
1. Deletion-only: `python3 -I` difflib (autojunk off) of purrdf's file (PurRDF/purrdf substituted) against the current zh-Hans file gives one insert (the ® in the SPDX line) and four deletes. The deletes are the byte-identity sentence, "、Unicode 数据", the Unicode-3.0 sentence plus the YAML/XML/package paragraph, and "、来源记录". No other insertion. PASS.
2. G14a: `grep 来源记录` is empty. G14b: no "发行包" remains. A3: `grep -c 第三方` = 2, and the third-party paragraph reads as complete sentences ("第三方代码和继承材料保留其原有许可。选择 MulanPSL-2.0 不会将第三方材料重新许可。"). The shortened §4 sentence and the final "见...英文说明...和文件中的 SPDX 标识" sentence are grammatical. The record's translation renders them as full sentences. PASS.
3. G14d: FAIL, see finding R1.
4. `sha256sum` of the zh file = e536de48...3d4b and `wc -c` = 1712, both equal to the record. `cmp` of raw/backtranslation-r4/source.md against the file: identical. LICENSE-MULAN digest eb7a1d71... matches the record. PASS.
5. G14e: LICENSING.md now says "any code contribution" (l.67), "Code contributions to RAF" and "Documentation contributions (all `*.md` files) are licensed under CC-BY-4.0" (Contributions). This agrees with CONTRIBUTING.md 115-116 and with the REUSE.toml rule (every *.md is CC-BY-4.0). `grep -n CC-BY LICENSING.md` shows the sentence. No CLA text exists in the tree beyond CONTRIBUTING.md and LICENSING.md, and both agree. PASS.
6. Comparison bullets: choice of licence, no fourth option, CC-BY-4.0, §6 precedence, §2 grant and termination, §3 and §4 all match LICENSING.md and LICENSE-MULAN. The "冲突不一致" quote is verbatim at LICENSE-MULAN:44. The third-party bullet is backed by LICENSING.md 72-75 for third-party work, but see advisory A1.
7. Gates: `make licenses` rc 0, `make issue-refs` rc 0, `uvx reuse@6.2.0 lint` rc 0 (77/77 compliant).

## Findings
- R1 (required): the ambiguity count does not match its list. The record says "eighteen". The list holds 17 distinct items:
  - 冲突不一致: 1
  - article numbers, 本许可证, 其条款, 其中的贡献, termination scope: 5
  - the §3 exception, 声明: 2
  - 使用者, 您, 继承材料, 英文说明, SPDX 标识, 每项许可证, 语言条款, 这些, who did not review: 9
  If 使用者/您 is treated as one item the total is 16. Either way, it is not 18. Fix the number or the list so they agree.
- R2 (required): the record does not show what the translator actually reported. No raw translator output is preserved in raw/backtranslation-r4/ (only source.md and read-at). The sentences "The translator reported no sentence fragments" and "flagged eighteen ambiguities" therefore cannot be checked against the translator output. The bullet text is the orchestrator's own regrouping, with added explanations ("quoted unchanged", "settled by the license text", "None of them changes the meaning"), but the record presents it as the translator's findings. Save the translator's raw reply in raw/, and make the Comparison say which statements are the translator's and which are the checker's. Otherwise the record overclaims. The count in R1 depends on the raw reply.
- A1 (advisory): the bullet "third-party code and inherited material keep their own licences" goes beyond LICENSING.md 72-75, which speaks of "Dependencies" and "other third-party work". "Inherited material" appears nowhere in LICENSING.md. It matches the surviving Chinese wording, so consider "third-party material" in the bullet.
- A2 (advisory): the Contributions relicensing sentence still says "their contributions" without qualification, while the licence sentence is now code-only. This agrees with CONTRIBUTING.md and is not a contradiction, but the interaction between the CC-BY-4.0 docs and the relicensing grant is implicit.

VERDICT: BLOCKED

## Recheck

- R1 closed. `grep -c '^- "'` on raw/backtranslation-r4/translator-reply.txt gives 18. The record lists 18 numbered items. I mapped each to the archived bullet and they correspond one to one (使用者, 冲突不一致, 以中文版本为准, article numbers, 其条款/您, 其中的贡献, 专利维权行动, 本许可证, the §3 exception, 声明, 继承材料, 重新许可, 英文说明, SPDX 标识, 每项许可证, 语言条款, 这些, 未经法律审核).
- R2 closed with a caveat. The FRAGMENTS line is archived, and the record reports it accurately (no fragments, "冲突不一致" run-on, §4 list split across lines). The record now separates "the translator reported" from "the repository check assessed". The assessment grouping (item 2, items 3-10, items 1 and 11-18) is consistent with the items.
- A1 closed. The bullet now reads "third-party material keeps its own licence", which matches LICENSING.md 72-75.
- A2 left unchanged, accepted.
- New advisory A3. The coordinator said the archived reply includes the TRANSLATION section. It does not: the file holds only the AMBIGUITIES list and the FRAGMENTS line, and its header says the translation is reproduced verbatim in the record. The record body therefore cannot be diffed against a separate archived copy. Archive the TRANSLATION section too if you want the "verbatim" claim to be checkable. This does not block.
- Unchanged and rechecked: the SHA-256 and byte count of the zh file equal the record (e536de48...3d4b, 1712), and the raw snapshot is identical. `make licenses`, `make issue-refs` and `uvx reuse@6.2.0 lint` all return rc 0.

VERDICT: PASS
