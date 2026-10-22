<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Licensing

Blackcat Informatics® Inc. holds the copyright in RAF’s first-party material
(© 2026). Two distinct things are true about its licensing, and this document
keeps them distinct:

1. All first-party code is offered under **three open-source licenses, at
   your option**: MIT, Apache-2.0, or MulanPSL-2.0.
2. Separately, Blackcat Informatics® reserves the right, as copyright
   holder, to grant commercial/proprietary licenses outside those terms.

## Open-source terms

First-party code in the Rust workspace, configuration, templates, brand
assets, and build tooling is offered under your choice of any one of:

| License | Text |
|---|---|
| **MIT** | [`LICENSE-MIT`](./LICENSE-MIT) |
| **Apache License 2.0** | [`LICENSE-APACHE`](./LICENSE-APACHE) |
| **Mulan Permissive Software License, Version 2 (MulanPSL-2.0)** | [`LICENSE-MULAN`](./LICENSE-MULAN) |

<!-- REUSE-IgnoreStart -->
This is the grant in first-party source headers and the owned-code part of
package metadata:

```text
SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
```
<!-- REUSE-IgnoreEnd -->

### Documentation

Every Markdown document in this repository is licensed under
[Creative Commons Attribution 4.0](https://creativecommons.org/licenses/by/4.0/)
(CC-BY-4.0). Brand assets and code use the three-license grant above.

### About MulanPSL-2.0, precisely

MulanPSL-2.0 is an [OSI-approved](https://opensource.org/license/MulanPSL-2.0)
permissive license published bilingually in Chinese and English. An adopter
should know these terms:

- Its Chinese and English versions have equal legal effect; **the Chinese
  text governs if they conflict or differ** (§6). The committed
  [`LICENSE-MULAN`](./LICENSE-MULAN) carries both languages.
- Each contributor grants you a **patent license** within the scope and
  conditions of §2. The software patent license **terminates from the date**
  you or your affiliates directly or indirectly initiate patent-infringement
  litigation (including counterclaims or crossclaims), or another patent
  enforcement action, alleging that the software or a contribution infringes
  a patent. The full §2 controls the grant and its conditions.
- It grants **no trademark license except use necessary for §4 notices**
  (§3). Section 4 requires providing recipients with a license copy and
  retaining copyright, patent, trademark, and disclaimer statements.
- Unlike Apache-2.0, it does **not** require stating changes to modified
  files.

The committed text is byte-pinned (SHA-256
`eb7a1d713eb919b146787629e22e4c975cb701f529a65d4d7e0fcd417558bf1c`, from the
SPDX license-list corpus, matching the canonical text at
`license.coscl.org.cn/MulanPSL2`) and verified by `make licenses`.

Unless you explicitly state otherwise, any code contribution intentionally
submitted for inclusion in the work shall be licensed as above
(MIT OR Apache-2.0 OR MulanPSL-2.0), without any additional terms or
conditions.

### Third-party material

Dependencies keep their own licenses. Choosing one of the licenses above does
not relicense any dependency or other third-party work.

The Unicode data tables in `unicode-ident`, `purrdf-lex`, `purrdf-iri`,
`purrdf-core` and `purrdf-text` retain
Unicode, Inc.'s copyright and the Unicode License V3
in addition to their code licenses. The dependency licence policy scopes these
additional terms to the audited versions, including the IRI parser's generated
IDNA tables. Their package notices and Unicode permission notice remain
controlling for those tables; choosing a RAF code license does not replace them.
PurRDF's lexical package also retains its additional MIT notices and the
BSD-3-Clause notice for WHATWG HTML entity and numeric-recovery tables. Those
notices and their redistribution conditions remain
controlling. The dependency policy admits the additional Unicode and BSD terms
only for their named packages; it does not add them to the general license list.

A [Simplified Chinese explanation](docs/LICENSING.zh-Hans.md) accompanies this
document. It is an explanatory draft and has not undergone legal review.
The completed [independent model backtranslation record](docs/LICENSING-BACKTRANSLATION.md)
identifies the exact Chinese source and the comparison performed. It is not
human or legal review.
The complete controlling bilingual MulanPSL text remains the recipient’s license.

## Proprietary / commercial licensing

The open licenses above are offered **in addition to — not in place of** —
Blackcat Informatics®' right, as copyright holder, to license the software
under separate commercial or proprietary terms. Granting the open licenses
does not revoke or limit this reservation.

To obtain a proprietary license, contact **licensing@blackcatinformatics.ca**.

## Trademarks

"Blackcat Informatics®" is a registered trademark of Blackcat Informatics®
Inc. None of the open licenses grants any right to use this name, its logos,
or marks — see **Apache License 2.0 §6** and **MulanPSL-2.0 §3**. Nominative
references (e.g. "built on RAF") are permitted; uses implying endorsement
or origin are not.

## Contributions

Code contributions to RAF are accepted under **MIT OR Apache-2.0 OR
MulanPSL-2.0** and, under the project CLA, under terms that permit separate
proprietary/commercial licensing. For the commercial reservation above to
extend to contributed material, contributors agree to license their
contributions to Blackcat Informatics® Inc. under terms that permit
relicensing, including under proprietary terms. Documentation
contributions (all `*.md` files) are licensed under CC-BY-4.0. A Contributor
License Agreement may be required before substantial contributions are
merged. See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for details.

## Copyright notice

> Copyright © 2026 Blackcat Informatics® Inc. All rights reserved, except as
> expressly granted under the licenses above.
