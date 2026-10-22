<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Document model assertion ownership review

Bounded read-only review of the frozen document-model slice and its stated
fixtures. No builds, product edits, timing checks, mutation runs, or broad panel
were performed. Owner execution receipts are reported separately; this review
does not establish a new pass or whole-contract completion.

## Actionable wording and ownership

The Graph nodes clause at `workplan-document-model.md:503` says “the records,”
after Typed records explicitly includes strict debt. `src/graph.rs` documents
evidence, assessment and coverage nodes, imports those three record types, and
has no debt node. The actual `graph::the_graph_has_every_kind_of_node` and
`records::the_records_are_nodes_with_their_edges` assertions prove the three
represented record kinds, not debt. Name evidence, assessment and coverage
records explicitly in this clause. This repairs an overbroad statement rather
than introducing a new graph obligation merely to match ambiguous wording.

The generic record writer clauses are source-backed for Debt through
`src/debt.rs:702` onward, which passes `Record::Debt` to the same production
`record::create` / `record::replace`. Their current round-trip marker directly
tests Evidence, Assessment and Coverage only. The existing
`debt::the_record_keeps_its_spdx_lines_and_a_new_one_takes_those_it_is_given`
actually creates and replaces debt through that caller, checks authored SPDX
headers and retained/replaced options, and includes half-options refusal. It is
a specific additional owner for the generic writer/header claim; it is not a
typed full-value Debt round-trip assertion. Do not describe the existing
three-record round-trip test as exercising all four variants.

## Inspected substantive ownership

- `Repo::assert_capability` opens the actual repository with `Gate`, finds the
  typed header id, checks the exact permanent capability path and current state.
  The four-state `new` test invokes it before creation, after creation, after
  the explicit authored state edit, and after reconciliation. The actual
  `from_new_to_a_completed_change_through_the_binary` test checks it through
  creation, refusal, accepted assessment, explicit delivered edit, verification
  and completion. These assertions support command identity/state retention;
  they do not claim an historical rename detector or decide when a state is
  semantically earned.
- The completed-record test actually compares ordinary `git show
  main:work/changes/shape.md` output with the pre-completion record bytes, while
  checking deletion, exact changed paths and a clean subsequent check. It proves
  retained committed history in that fixture, not a new post-completion commit.
- Graph node/edge tests inspect concrete typed nodes, endpoints, counts,
  deduplication and section attribution. Query tests compare actual paths and
  reverse edges. Record input tests compare exact Section/Symbol/Record targets,
  omit unresolved inputs without load findings, and check absent-record neighbors.
  The proposed-claim test proves exclusion while proposed; it alone does not
  execute proposal resolution, so the wording's resolved behavior additionally
  relies on the ordinary claim fixtures and shared parser path.
- `records` creation/replacement assertions check typed reload equality,
  exclusive existing-file refusal with unchanged snapshots, missing replacement
  refusal, actual rejected-verdict replacement, no leftover temporary file,
  invalid digest/ref/slug/header refusal with unchanged snapshots, and a valid
  write neighbor. The symlink tests exercise both external and internal directory
  links and preservation of the document a replacement might otherwise reach.
  Production `create` and `replace` both call `Record::text` before filesystem
  writes; replacement does not have a separate unvalidated rendering path.
- The common paragraph and section builders now supply the actual mutation
  anchors to document and rule fixtures. Missing/empty/reordered/duplicated
  sections remain real malformed inputs. The rule loop starts from a clean
  valid fixture and `Case::assert_exact` requires the requested kind, file and
  location, rejecting neighboring-file findings and unlisted extra kinds.
  The complete named capability path/state set includes the auxiliary owner;
  broken-document exclusion still removes only the malformed document.

## Constraints and semantic limits

All seven Constraints items retain paragraph/list-item markers immediately
following the actual claim. Profile/closed-set and change-affects ownership uses
the actual Rule inventory loop; line-ending refusal uses the real sync/check
neighbor; flat-directory refusal retains its nested-document fixture.

The current-prose obligation remains in Constraints. It explicitly assigns
semantic truth to authors and assessors, and limits runtime checks to proposal
placement/state and current accepted assessments. The marked complete test
rejects both human and agent rejected assessments without writes or Cargo,
then accepts the actual accepted neighbor. The CLI lifecycle checks assessment
currency and refusal before completion. This is an honest boundary, not an
assertion that structural tests recognize arbitrary false prose. The state
editing requirement likewise stays an authoring obligation with command
nonmutation assertions and structural state/content neighbors.

The tracked-path limitation has direct ownership: an actual on-disk untracked
target is refused when omitted from caller paths, accepted when included, and
accepted by a non-checkout fallback walk. The explicit-list test proves no Git
process starts by supplying a nonexistent Git executable, with the ordinary
no-list startup failure as its negative neighbor.

No semantic coverage escape or new product defect was found in this bounded
slice beyond the graph wording above. Typed state checks are not a guarantee of
human prose truth; full policy adoption, lifecycle completion and repository
gates remain unfinished acceptance work. The owner reports model batch 69749
passed and the two identity cases passed in batch 3771; this reviewer did not
execute those commands or infer terminal status of its other active checks.
