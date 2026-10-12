<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAF licensing backtranslation record

An independent model read the entire [Chinese licensing explanation](./LICENSING.zh-Hans.md)
and translated it into English. It did not read the English original, the linked licenses,
repository history or any other project material. It read the current source in full.
This record is a model backtranslation and wording comparison. It is not human,
native-speaker or legal review. The explanatory document remains an unreviewed draft; the
complete licenses govern recipients' rights and obligations.

- Source read: `2026-10-12T05:13:17.759242+00:00`.
- Translation completed: `2026-10-12T05:13:31.914738+00:00`.
- Source: `docs/LICENSING.zh-Hans.md` (1,692 UTF-8 bytes).
- Source SHA-256: `01eb7226919c931465673a39a57941c70bddc9455e4e12154662b351e3bd7aa0`.
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
- that distributed license text must match the pinned digest.

The translator flagged eight possible ambiguities in the source alone. None of them changes
the meaning compared with the English explanation or the license:

- "固定摘要" means the pinned digest.
- "冲突不一致" covers conflict or inconsistency.
- Termination applies to the patent license only, as in MulanPSL-2.0 §2.
- Affiliates, the commercial-license terms and the scope of "explanatory documentation"
  are governed by the license and by `LICENSING.md`.

The translator's source-only findings do not independently verify licenses, package
contents, trademarks or contribution authority. A change to the Chinese source requires
updating this record and its source fingerprint; `make licenses` compares the fingerprint
above with the current file.

## Complete independent backtranslation

The source's SPDX metadata is recorded in the source file itself, so the translation omits it.

### RAF Licensing Notes

This file is an explanatory draft and has not undergone legal review. For the record of the independent-model back-translation, see
[Back-translation Verification Notes](./LICENSING-BACKTRANSLATION.md). This file cannot substitute for the full text of the licences.

RAF first-party code allows users to choose any one of MIT, Apache-2.0, or MulanPSL-2.0.
The SPDX expression is `MIT OR Apache-2.0 OR MulanPSL-2.0`.
These are three open-source licensing options. Commercial licences are provided separately by the copyright holder and are not a fourth open-source option.
Documentation bearing the `CC-BY-4.0` identifier retains that licence.

Article 6 of the [full Chinese and English text of MulanPSL-2.0](../LICENSE-MULAN) provides that the Chinese and English versions have equal legal effect;
in case of any conflict or inconsistency between the two, the Chinese version prevails. Article 2 provides that each Contributor grants You a patent licence under its terms.
If You or Your affiliated entities directly or indirectly initiate patent infringement litigation (including a counterclaim or cross-claim) or other patent enforcement action concerning the Software or any Contribution therein,
then the patent licence to the Software granted to You under this Licence terminates as of the date on which such action is initiated.
The scope and conditions of the patent licence are governed by the full text of Article 2. Article 3 grants no trademark licence, except for use necessary to fulfil the notice obligations of Article 4.
Article 4 requires providing recipients with a copy of the Licence, and retaining the copyright,
patent, trademark and disclaimer notices. The text in a distribution package must be byte-for-byte identical to the bilingual text whose digest is pinned in the repository.

For the sources and scope of application of materials, see the [English notes](../LICENSING.md), the provenance records, and the SPDX identifiers in the files;
the actual grants and obligations are governed by the full text of each licence. Explanatory documentation cannot alter the language clause of MulanPSL-2.0.
