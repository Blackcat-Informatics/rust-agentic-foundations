<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Bounded discovery and target proof review

Read-only review of the source-index Contract slice from Explicit package roots
through Target read origins, the two new fixture bodies and the strengthened
default/explicit target fixtures. Production callers inspected in
`workspace.rs::load_with`, target derivation in `target.rs`, and the target-root
origin handoff to the module walker. No builds or product mutations performed;
unchanged API-root/coverage repairs were not rereviewed.

## New actual proof

The competing-failure fixture genuinely exercises the phase order: malformed
discovery manifest before a missing explicit target; missing explicit target
before malformed module parsing; malformed child before invalid index use-path.
Repairing that child surfaces the exact later `SuperAboveRoot` error, and
repairing the use permits a load. Production uses fallible discovery first,
derives/checks targets before constructing and finishing the walker, then
builds the index. The marker owns these actual competing errors rather than
an unrelated successful load.

The new unreadable-root fixture pins `TooLarge`, exact file and exact
`ManifestKey` origin for each bin/test/bench/example explicit path and table-name
default. Every case has an under-cap valid neighbor. It separately checks
named/default build origins. Existing library tests pin Discovery, lib and
lib.path. Production stores each derived origin beside its target and passes
that pair to `Walker::root`; inferred targets use the common Discovery branch.
The fixture does not assert a native filesystem permission failure, nor is one
needed for its documented unreadable-root origin behavior.

Default discovery now actually asserts grouped bench/example roots alongside
the existing file/grouped bin/test forms. Explicit tables for all target kinds
remain found with all four automatic flags false. The independent automatic-flag
fixture has real suppressed inferred files plus a flags-absent positive neighbor.
Thus the explicit fixture is not being used alone to claim suppression.

## Concrete remaining claim-proof gap

Package identity/order says packages are sorted by manifest path. Its current
marker owners prove duplicate-name rejection, overlap counts, sorted target
names and equality of package lists across loads. They do not assert the actual
package manifest sequence. In the sorting fixture package names equal their
path stems, so sorting packages by name instead could satisfy those assertions.
The source does sort discovery output, but the guarantee-specific execution
proof is incomplete. Reported directly to the owner: strengthen the existing
bound fixture with opposing package-name and manifest-path order and assert
the exact manifest sequence. No extra blanket marker is warranted.

## Limits

No additional concrete defect found in the new phase/read-origin proofs or
strengthened target controls. Prior depth/Unix containment families were not
re-executed or broadly re-reviewed. The owner reports 29 discovery and nine
target-root tests passing and final all-target Clippy/docs sync terminal zero;
this reviewer makes no independent execution or whole-adoption claim. Final
disposition remains subject to the specific package-order proof correction.

## Ordering assertion recheck

The existing ordering fixture is now strengthened on disk. Package alpha is
declared under b and zulu under a; the second load reverses insertion order.
Assertions require targets zulu then alpha, exact package manifest/name rows
(a/Cargo.toml, zulu) then (b/Cargo.toml, alpha), and equal results across loads.
This directly distinguishes manifest-path ordering from package-name ordering
and resolves the identified proof gap without changing marker ownership.
The owner is running its focused test and Clippy; those results are not yet
claimed by this source-only recheck. No further concrete defect found within
this bounded slice.
