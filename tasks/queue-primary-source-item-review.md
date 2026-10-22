<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Source-item clause and root-group review

Read-only primary review of the indexed item, byte/line, parameter, initializer,
namespace metadata and source-identity clauses against current production methods
and named symbol_index/index_disk assertions. No primary builds or source edits.
Resource measurements and the remaining resolver clauses are not this review.

The review found an actual API defect: group construction excludes whole-file
crate roots, but same_source originally used only file/start-byte. A byte-zero
first item could therefore be returned as the root's group. The owner reproduced
the failing public-call assertion before repair. An old grouping test accidentally
selected a test-target root by name and passed through this defect; it now selects
the intended function by kind as well as name.

The repair reuses one Index::is_root predicate for group construction and lookup.
Its existing module_of/root_items maps identify the actual root item id; a normal
module or macro at the root does not match that root id. It adds no storage and
removes the former per-item scan across every target root. same_source returns
empty for a root before its location can collide with written source. Written
source grouping remains keyed by file/offset. Rustdoc and capability clauses now
state the whole-file exception.

The new public regression checks byte-zero, prefixed and empty source fixtures,
with actual first-item singleton grouping beside root exclusion. Existing
shared-path grouping asserts exact member identities, ordering, coverage and
ordinary singleton behavior. No concrete defect was found in the repaired path.
Actual caller qualification is still owner-run and must be collected.

Initializer shared-path ownership was also missing its specific existing test;
the marker now names bindings_and_an_initializer_are_shared_by_every_path_that_reaches_the_file.
Inspected tests meaningfully assert parameter order, names occurring only in types,
required/nonconstant initializer absence, namespace distinctions and tree/index
equality. This bounded review is not blanket acceptance of every listed Item field
or of the whole source capability. Source and documentation changes can still
invalidate individual assessment digests.

## Real workplan caller requalification

The initial coverage caller run exposed obsolete lighter-policy expectations,
not a reason to weaken the root repair or production policy. Reviewed the actual
item_findings_after_line_refusals helper: it first requires exact typed per-package
shipping/test orphan counts, zero exceptions/debt and no ceilings. It then requires
the exact number of default-zero findings, their coverage-file/key location,
package totals, class-specific orphan counts and default-zero message. Only those
explicitly verified findings are separated before the unchanged item/test/reexport
assertions; every unexpected other finding remains visible. Stress expectations
derive their line totals from fixture dimensions. The repository fixture names
all eight actual packages and requires zero gaps and line debt.

No blanket discard or permissive ceiling was introduced. Owner reports all 24
coverage fixtures, 34 untouched tests, source symbol 35/target 6 and current
all-target source/workplan Clippy passing, plus docs sync. These are focused owner
results, not a full gate or reviewer-run qualification. No further bounded defect
was found in these caller repairs.
