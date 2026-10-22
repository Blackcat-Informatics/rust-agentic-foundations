<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Bounded completion Constraints proof review

Read-only review of the new Constraints proof and wording in
`docs/capabilities/workplan-complete-gate.md`, the new
`tests/verify.rs::selected_cargo_output_is_trusted_but_an_empty_list_is_refused`,
and their actual production paths. No builds or product edits performed.
This does not replace the full completion review or qualification.

## Trusted executable proof

The new Unix test uses the actual `SystemRunner` with a caller-selected executable.
The ordinary Cargo neighbor runs the bound authored panic and asserts a failed
report and `BoundOutcome::Failed`. The selected executable then supplies the
exact indexed list name and a successful one-passed result without executing
that source; the report passes. Its empty-list neighbor is unresolved, and
literal receipt files require list then run for the positive and only list
for the refusal. This supports the revised selected-executable trust limitation
without claiming a particular ordinary libtest spoofing mechanism.

Production `bound/run.rs::test` checks the successful list response for exactly
one matching test, then reads one result line and requires successful exit,
one passed, zero failed and zero ignored. Thus output is evidence supplied by
the selected executable, not an attestation of compilation or execution.
The numerical parser is undergoing an owner-led native scalar repair; this
review does not treat that concurrent source as qualified.

The same Constraints item also claims protection against multiple entries and
wrong names. The new test covers empty selection and trusted output, not those
two cases. Existing inspected tests
`bound::tests::two_listed_entries_are_unresolved` and
`bound::tests::one_entry_under_another_name_is_unresolved` assert the specific
`TestListedWrong` refusal; the former also asserts only one invocation. Their
paths should accompany the item marker, rather than relying on a blanket
binding elsewhere. Reported directly to the owner.

## Two judgments and publication

The existing exact tests are
`complete::a_document_edited_while_the_tests_ran_refuses_and_nothing_is_written`
and `complete::a_record_linked_while_the_tests_ran_is_refused_at_the_second_judgement`.
The first introduces a tense obligation and asserts `Failure::Changed`, the
specific `TenseMismatch`, a retained record and the edited document unchanged.
The second swaps the record for a link during the runner callback, asserts
`LifecycleError::RecordRefused { ran: true, .. }`, and verifies no generated
rewrite and preservation of link plus backing file.

A wording defect was reported: saying the second judgment detects any document
edit is too broad. `lifecycle/complete.rs::complete` compares binding keys and
new static failures after reopening; it does not compare every document's text.
A still-valid prose edit need not cause refusal. The claim should name newly
failing conditions and changed binding sets, and separately the rejected link.
The existing added-binding test can own the changed-set guarantee if included.

The moved Explanation correctly retains the unprotected interval. After the
second evaluation, `complete::write` calls `Workplan::write` and then path-based
`remove_file`; sync judges targets, stages replacements and renames them without
comparing their text to the second evaluation. This supports the explicit
absence of an atomic compare-and-publish operation. Mutation during Cargo
proves the earlier second judgment, not this later interval. No execution proof
of the later race is claimed or supplied.

## Verification limits

The new focused test was still running when the owner requested review. No
execution pass is claimed here. Marker updates and the overly broad edit wording
above require a narrow final source recheck. No whole-contract, adoption,
completion, hook or merge acceptance is implied.

## Final narrow recheck

The final on-disk Constraints text now states re-evaluation, newly failing
conditions and changed binding sets, rather than universal edit detection.
Its marker includes the actual added-binding test, whose assertions require
`Failure::Changed` despite otherwise satisfied conditions. The trust item now
also names both specific multiple/wrong-entry refusal tests. Both reported
mapping and wording findings are resolved in the inspected bytes. The
publication gap remains explicit in Explanation without a tested atomicity
claim. Markers immediately follow their respective Constraints items.

Inspected `raw/queue-trusted-runner-proof.log` shows the new real executable
test passing, with one test and zero failures; the owner reports terminal exit
zero. This reviewer did not run it. The new native count path retains the
closed lexical subset and delegates exact integer value parsing before a
checked host narrowing; its new unit asserts optional-plus/leading-zero
neighbors, exact host maximum, overflow, negative and fractional refusals.
No additional concrete defect was found in the bounded final source review.
Full document/adoption checks are still the owner's separate running work.

## Policy reconsideration: publication limitation needs proof ownership

The parent requested a substantive policy recheck. The earlier acceptance of
moving the publication gap solely to Explanation was too permissive and is
superseded here. `strict/claims.rs::units` deliberately collects only Contract
and Constraints; `workplan-strict.md` describes Constraints as a limitation
list. `work/changes/workplan-strict.md` promises every Contract clause and
Constraints item a bound marker, and `repository-adoption.md` promises complete
authored sections and claim-specific proof, with no lighter policy path.

No literal rule was found requiring every limitation in Constraints, so this
is a semantic assessment rather than a parser rejection. Nevertheless, the
post-judgment gap remains a present operational limitation of completion.
Relocating it solely because there is no executable proof removes this exact
claim from the policy's proof population. It differs from relocating historical
counts or rationale, which are not universal current behavior guarantees.
The smaller limitation still present in Constraints, saying there is no atomic
snapshot, is not actually proven by its mutation-during-Cargo test owners.

Recommendation: retain a concise Constraints item stating the unprotected
publication boundary and bind meaningful deterministic production-path proof.
A report computed before an otherwise valid document edit can be applied via
the real public `Workplan::write`; assertions on the resulting text can prove
stale-report overwrite without a timing race. That proves the writer boundary,
not by itself the entire completion sequence. If the path-based record-removal
limitation is also asserted, a module-local test can invoke the actual private
completion write phase after replacing the judged path, retaining existing
judgment-refusal neighbors. It must exercise production phase code rather than
reimplement the algorithm. The source wiring establishes where that phase
follows the second judgment. Do not attach an arbitrary earlier test to the
later gap, or invent a tested atomicity claim. Detailed rationale can remain in
Explanation once the actual limitation is owned and proved.

This recommendation was sent directly to the owner and parent. Publication-gap
proof ownership remains open; the prior final narrow source disposition is
therefore not acceptance of that portion of the complete-policy adoption.

## Restored publication constraint and concrete writer proof

The owner restored the publication limitation to Constraints and added
`sync::write_publishes_the_computed_snapshot_without_comparing_later_authored_edits`.
Its actual assertions compute a real on-disk SyncReport, save its alpha text,
append newer authored prose, confirm the newer document still loads and yields
a report retaining that prose, then publish the OLD report through the actual
public writer. The final file must equal the exact saved snapshot and lack the
newer prose. An unedited neighbor also succeeds, and both finish with empty sync.
This supplies meaningful deterministic proof of stale-snapshot replacement.

The restored marker owns this concrete writer limitation; source inspection
of `complete` establishes that its second evaluation supplies the report to
that same writer. The remaining record-removal sentence is supported by the
source's path-based `remove_file`, not execution of a post-judgment link swap.
The new test does not execute the completion phase itself or that later removal.
The owner reports its focused log terminal zero; this reviewer inspected source
and assertions only. The earlier section-move concern is repaired for the writer
boundary, with this exact execution limitation retained rather than representing
the entire publication interval as tested.
