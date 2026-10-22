<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Counted coverage owner review

Bounded read-only review of the new listing receipt, binary visibility fixture,
test/example-only automatic enforcement and their relevant prose. No builds or
product edits. No concrete defect found in this slice. The broader missing
subtype/ownership proofs in queue-two-way-claim-owner-map.md remain separate.

## Real surface listing receipt

The test-only `tests::listing(id)` hook is immediately before the actual
`Workspace::exported_items(id)` operation in `Judge::surface`. It records actual
TargetIds and does not supply answers, skip execution or change production
admission. `listed` resolves the recorded ids through the actual source
workspace and compares the full sorted target-root multiset against independently
written expected package/lib/bin paths. Thus duplicate, absent, extra and wrong
targets fail, rather than only a total call count. The receipt is thread-local,
bounded by expected calls, rejects nesting and clears through Drop after failure.

`surface_listings_obey_the_load_check_and_report_target_sets` loads five actual
two-target packages and a real capability/coverage record. It requires:

- Load: scoped, excluded-only and exclusion-plus-binding packages, each lib/bin
  once; no bound-only or untouched target.
- Each of two checks: bound-only and overlap packages each lib/bin once; no
  duplicate scoped listing and no excluded-only or untouched listing.
- Each of two reports: all five packages, each lib/bin once, equal complete
  report objects between calls.

The ordinary and ambiguous-surface variants both run every phase. The refused
outside-scope overlap surface must yield exactly one actual check finding at
the capability's bindings key and exactly one report error; the ordinary
neighbor must yield none. Both actual graph bindings must resolve. This is a
meaningful producer-level proof of the existing per-load/per-check/per-report
laws and the intended overlap, not a cache assumption or finding-count proxy.
It does not assert enumeration order; the existing deterministic report owner
remains separately bound for ordering. No performance contract was dropped.

## Binary and role boundaries

`binary_public_items_follow_visibility_rather_than_their_names` adds a real
binary with public `private_looking`, private `public_looking`, crate-only and
doc-hidden functions. The report requires exactly one binary gap, with the
exact public path, definition path and line. Binding that function removes
all binary report gaps and increments covered items by exactly one. This proves
report visibility and repair. It does not independently run `check`; the
previous actual binary completion/effect fixture remains the separate check
enforcement evidence. The corrected capability and module prose now say explicit
public binary items count, preserving the actual behavior.

`test_and_example_bindings_each_enforce_the_package_they_reach` starts with an
unreached package and no findings, then changes only its test binding and
requires two exact public-item gaps attributed to that test binding. Its
example-only variant leaves code/test ownership absent in that package and
requires two public-item gaps plus the actual test gap, all attributed to the
example binding. The fully bound neighbor clears findings. These are actual
resolved graph roles passed through production strict coverage, not an enum
iteration or a dummy import. The fixture's existing roomy line ceilings keep
this assertion about item/test enforcement; it must not be cited as proof of
default-zero line coverage. It does not execute the example, which is unrelated
to automatic binding reach.

## Markers and limits

The two listing-frequency Constraints markers follow their actual list items,
and retain distinct once-per-load and once-per-check semantics. The visibility
marker owns the direct report visibility fixture. Existing deterministic report
markers own output order separately. The original map's missing operation-count
proof and test-only/example-only reach proof are now closed by these inspected
assertions. The new phase refusal also covers the automatic unreadable-surface
capability-key location previously missing from the map.

Import-cycle, empty-glob, CarryBudget and the crate-only/external-carried collision
producer proofs remain unestablished by this slice; the new ambiguous fixture
must not be represented as all those refusals. Other existing Contract compound
ownership gaps remain as mapped. No limitation was moved outside judged text.

Execution was not repeated here. Owner reports terminal exit 0 for the phase
receipt and binary visibility focused logs; role test and all-target Clippy
session 50690 were active at request. Full gates, candidate counts and adoption
completion are not inferred. The read freeze for this bounded slice is released.

## Role fixture correction and final narrow recheck

The first role fixture did not establish the claimed valid example role:
`other::helper` lacked documentation, so execution 50690 correctly produced an
additional `binding-wrong-role` finding and stopped before Clippy. My preceding
description of its resolved example role was premature. That failed checkpoint
is not a pass receipt, and its assertions were not relaxed to hide the refusal.

Current `test_and_example_bindings_each_enforce_the_package_they_reach` creates
its own real source workspace and adds an actual block documentation comment
to `helper` on the same physical line. All unreached, test-only, example-only
and fully covered variants now use this same source. The exact expected finding
sets and attribution checks remain unchanged; the actual example role is valid
while the example binding still covers no public item. This is the smallest
fixture correction and does not add an ownership binding or weaken a role gate.

The listing guard declaration has only moved before statements; its thread-local
receipt, bound, cleanup and real-operation hook are unchanged. The six direction
clauses now name their structural question/check explicitly rather than asking
one table marker to own unrelated behavior. No semantic truth judgment is added
to binding resolution by that reorganization.

Owner reports role proof 67400 terminal exit 0 in
`queue-two-way-binding-roles-current.log`, and warnings-denied all-target Clippy
46307 terminal exit 0 in `queue-two-way-owners-clippy-current.log`. These
supersede the earlier pending/failed role checkpoint; the listing and binary
focused passes remain valid. No checks were executed by this reviewer.
Docs synchronization/candidate 81010 was still active at this recheck; no full
gate, measured candidate count or adoption completion is claimed. The missing
refused-surface subtype producer proofs remain explicit.

## Final two producer recheck

Inspected the two new real-source producers without execution. No concrete
implementation or assertion defect found in this delta.

`carried_name_search_budget_refusal_is_not_an_empty_covered_surface` constructs
the actual 16-link/eight-carried-name graph with two blocking modules. It admits
the same source at default and load-usage-plus-52-KiB item limits, requires load
success, then requires exactly CarryBudget for `probe::C0`, exact target/root
and carried declaration file/line. Empty code/items must still be incomplete.
The identical default-budget source must produce no errors and be complete
because its remaining exports are external, not local items. This is a real
carried-search refusal/valid neighbor, not a forged SurfaceError. The 52-KiB
headroom is explicitly a sufficient fixture choice, not a minimum or universal
resource bound; its exactness is not claimed.

`a_crate_visible_item_sharing_an_external_carried_name_is_not_a_public_item`
uses the actual external carried Cell/local Cell collision. Both variants must
have no surface error; crate-visible local Cell yields no gap and a complete
report, while public Cell yields exactly one local public-item gap at the exact
path/file/line and is incomplete. This owns the coverage consequence that the
crate-visible local item is not counted. Because coverage deliberately omits
external reexports, this test alone does not distinguish an underlying listed
external Reexport from an omitted external Reexport; do not cite it as an exact
source-index Reexport-identity assertion. That source limitation is delegated
to the source index, while this fixture proves its visible coverage consequence.

The complete retained Constraints item now has its marker immediately after
the item and frontmatter bindings for these actual owners. Existing ambiguity,
ordinary-budget and cycle/empty-public-glob producers remain separately named;
the new tests do not stand in for them. This closes the two new producer gaps
at the coverage report boundary. These two tests themselves do not call check:
scope finding propagation remains composed with the inspected generic
SurfaceError-to-finding path and the actual ambiguity/ordinary-budget check
owners, rather than being newly executed for every subtype here.

Owner reports both selected producers in session 52508 terminal exit 0, with
`queue-two-way-carry-refusal.log` and `queue-two-way-carried-collision.log`.
Qualification 45150 was active at request; no broader target/Clippy/gate pass or
adoption completion is inferred. No reviewer builds or product edits.

## Collision topology and exact projection recheck

The strengthened current fixture replaces the reader-local explicit shadow
with two public globs: one of a module defining local Cell and one of a module
reexporting external Cell. For the crate-visible local definition it now calls
the actual source `exported_items` and requires no local items and exactly one
Reexport, path `probe::Cell`, written `std::cell::Cell`, item None, before checking
the coverage report's omission/completeness. This closes the earlier missing
underlying Reexport-identity assertion for that topology. Its public-local
neighbor requires the exact exported path/defining path `probe::local::Cell` and
line 1 as a public-item gap.

The prior direct-reader shadow fixture is a different topology: its explicit
local definition can suppress the external route itself. Its failed attempted
strengthening must not be treated as a source-index contradiction or an accepted
Reexport receipt. Current assertions and retained historical failure distinguish
those shapes correctly. No product defect found in this narrow recheck.

Owner reports session 58977 terminal exit 0 for the strengthened producer and
warnings-denied all-target Clippy, final log
`queue-two-way-carried-collision-final.log`; historical failed direct-reader
assertion remains in `queue-two-way-carried-collision-current.log`. No execution
by this reviewer and no broader gate or adoption completion inferred. RDF
allocation disclosure changes were not part of this review.
