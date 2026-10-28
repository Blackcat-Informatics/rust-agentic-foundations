<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Canonical RDF ordering

Scope: the complete `raf-rdf` crate at
`50422681f2e8de3c6ce258f169e0d3660b5c78c1`, including its seven production files,
manifest, profile and embedded tests, report example, and the Census and Workplan
construction, parsing, migration and rendering boundaries. The review kind is
design and the verdict is findings.

| Finding | Severity | Location at the reviewed revision | Status |
| --- | --- | --- | --- |
| `native-ordering-provider` | medium | [Canonical writer](../../crates/raf-rdf/src/write.rs), line 27; [profile normalization](../../crates/raf-rdf/tests/profile.rs), line 363 | open |

The canonical writer and its profile normalization use standard unstable sorting.
The pinned native `purrdf_lex::walk::try_sort_unstable_by` supplies the matching
allocation-free ordering operation. Required provider reuse applies to both
locations. The comparison keys and canonical output bytes must remain equal when
the native operation supplies that ordering.

This finding concerns provider reuse and design policy. Source inspection did not
identify incorrect RDF output or a demonstrated security exploit. The inspected
scope produced no further required correctness or security finding. The review
used source and caller inspection; it did not run builds, tests or an independent
parser replay. Native dependencies and external consumers received boundary
inspection rather than complete reviews.
