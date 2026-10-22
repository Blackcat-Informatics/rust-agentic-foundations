<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAF licensing backtranslation record

An independent model read the entire [Chinese licensing explanation](./LICENSING.zh-Hans.md)
and translated it into English. It did not read the English original, the linked licenses,
repository history or any other project material. It read the current source in full.
This record is a model backtranslation and wording comparison. It is not human,
native-speaker or legal review. The explanatory document remains an unreviewed draft; the
complete licenses govern recipients' rights and obligations.

- Source read: `2026-10-12T08:01:44.581905+00:00`.
- Translation completed: `2026-10-12T08:02:13.613036+00:00`.
- Source: `docs/LICENSING.zh-Hans.md` (1,712 UTF-8 bytes).
- Source SHA-256: `e536de4822bfc2adb03a9450ac37c2e7ff19a075f2b18758862f956e8d173d4b`.
- Unchanged [bilingual license](../LICENSE-MULAN) SHA-256:
  `eb7a1d713eb919b146787629e22e4c975cb701f529a65d4d7e0fcd417558bf1c`.

## Comparison

A separate repository check compared the returned text with [`LICENSING.md`](../LICENSING.md)
and with the unchanged bilingual MulanPSL-2.0 text. The backtranslation preserves:

- the choice of any one of MIT, Apache-2.0 or MulanPSL-2.0;
- that commercial licensing is not a fourth open-source option;
- that documentation keeps its CC-BY-4.0 grant;
- Chinese-language precedence (§6);
- the contributor-to-recipient patent grant (§2), its affiliate and litigation triggers, and termination from the date the action is initiated;
- the trademark exception (§3);
- the duty to give recipients a copy and retain notices (§4);
- that third-party material keeps its own licence, and choosing MulanPSL-2.0 does not
  relicense it.

The translator reported no sentence fragments. It noted that "冲突不一致" reads as a
run-on, and that the §4 list is split across two lines. It flagged eighteen possible
ambiguities in the source alone, in these terms:

1. "使用者" (user) is undefined.
2. "冲突不一致" runs two near-synonyms together.
3. Whether "以中文版本为准" applies only in a conflict.
4. Which document the article numbers belong to.
5. Whether "其条款" means the license's or the contributor's terms, and "您" versus "使用者".
6. The scope of "其中的贡献" and "本软件".
7. What "专利维权行动" covers, and what "其他" attaches to.
8. The referent shift of "本许可证", and which patent licenses terminate.
9. Whether the §3 exception qualifies the grant or the non-grant.
10. Whether "声明" in §4 attaches to all four items.
11. "继承材料" (inherited materials) is undefined.
12. Who chooses, and who would relicense, in "重新许可".
13. "英文说明" is identified only by its link.
14. Whether "文件中的 SPDX 标识" means source files or this file.
15. Whether "每项许可证" includes third-party licenses.
16. What "语言条款" means, and why it names only MulanPSL-2.0.
17. Whether "这些" refers to the three licenses or the SPDX expression.
18. Who did not review the draft ("未经法律审核").

The repository check, not the translator, assessed these against the English explanation
and the license, and found none that changes the meaning:

- Item 2 is the wording of the bilingual license text itself (§6), quoted unchanged; it
  covers conflict or inconsistency.
- Items 3 to 10 concern MulanPSL-2.0 provisions; the source names the license and defers
  to its full text, which governs.
- Items 1, 11 to 18 are general wording points; `LICENSING.md` and the licenses define
  the terms.

The translator's source-only findings do not independently verify licenses, package
contents, trademarks or contribution authority. A change to the Chinese source requires
updating this record and its source fingerprint; `make licenses` compares the fingerprint
above with the current file.

## Complete independent backtranslation

The source's SPDX metadata is recorded in the source file itself, so the translation omits it.

### RAF Licensing Notes

This file is an explanatory draft and has not been legally reviewed. For the independent model back-translation record, see
[Back-translation check notes](./LICENSING-BACKTRANSLATION.md). This file cannot replace the full text of the licenses.

RAF first-party code may be used by the user under any one of MIT, Apache-2.0, or MulanPSL-2.0, at the user's choice.
The SPDX expression is `MIT OR Apache-2.0 OR MulanPSL-2.0`.
These are three open-source license options. A commercial license is provided separately by the copyright holder and is not a fourth open-source option.
Documents bearing the `CC-BY-4.0` identifier retain that license.

[The full Chinese-English text of MulanPSL-2.0](../LICENSE-MULAN) provides in Article 6 that the Chinese and English versions have equal legal effect;
if the two conflict or are inconsistent, the Chinese version prevails. Article 2 provides that each contributor grants a patent license to you under its terms.
If you or your affiliated entities, directly or indirectly, initiate patent infringement litigation (including counterclaims or cross-claims)
or other patent enforcement actions regarding the software or the contributions in it, then the patent license that this license grants you for the software terminates as of the date that action is initiated.
The scope and conditions of the patent license are governed by the full text of Article 2. Article 3 grants no trademark license, except for the use necessary to fulfill the notice obligations of Article 4.
Article 4 requires providing recipients with a copy of the license, and retaining the copyright,
patent, trademark, and disclaimer notices.

Third-party code and inherited materials retain their original licenses. Choosing MulanPSL-2.0 does not
relicense third-party materials.

For the sources of materials and the scope of application, see the [English notes](../LICENSING.md) and the SPDX identifiers in the files;
the actual grant and obligations are governed by the full text of each license. An explanatory document cannot change the language terms of MulanPSL-2.0.
