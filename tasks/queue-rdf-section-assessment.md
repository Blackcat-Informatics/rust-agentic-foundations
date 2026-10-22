<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Shared RDF section judgments

Actual current owned-input writes: session 23614 exited 0 and wrote all seven
explicitly named records through the production API with preserved SPDX.
`raw/queue-assessment-rdf-owned-inputs-recorded.log` records each path, and
`raw/queue-assessment-history-final-contexts.log` replays all seven as accepted
and current. No other capability was automatically assessed by that command.

Actual RDF change completion readiness attempt 46794 exited 1, retaining its
record: `raw/queue-rdf-lifecycle-readiness.log` reports 99 obligations and 13
sections without current accepted assessments. They belong to census, CLI,
history and coverage; RDF assessments are not the refusing sections. All seven
RDF change items now state their actual implemented outcomes, but the change
has not completed and its bound-run phase has not been reached. This measured
static refusal, not the outcome fields, determines current readiness.

The subsequent owned-input capture is `raw/assessment-rdf-owned-inputs/`
(61525 exit 0 after current Clippy and production sync). It explicitly adds the
actual graph/read/write/term implementation modules as declared code inputs:
the Graph data-struct digest alone does not cover method or helper bodies.
Their complete current sources were reread and match the judgments below.
No module input is unresolved. These new inputs intentionally stale the
applicable initial assessments; they require a new actual record action rather
than being ignored as cosmetic bindings. The current record receipt must name
this owned-input directory, not the earlier smaller snapshots.

The seven individually named sections in `raw/assessment-rdf-settled/` were
captured through the production `Workplan::assess_context` API, terminal session
84201 exit 0. This follows the initial contexts in `assessment-current/` and
`assessment-rdf-final/`. No source-index assessment is included. The intended
verdict for each section below is accepted; actual record writes and later
currency checks have separate receipts and must not be inferred from this file.

- Purpose: the owned graph and domain-validation boundary matches actual
  `Graph`, native TermValue/parser/writer/hash calls and the census/workplan
  adapters. Ground validation deliberately does not validate datatype values or
  domain shapes, as its actual negative-value-space test demonstrates.
- Contract: current `graph.rs`, `term.rs`, `read.rs` and `write.rs` were reread.
  Empty rows/duplicate insertion, root and reference validation, absolute roles,
  language metadata and nesting refusal match the closed profile. Normalized
  VERSION/body bytes alone determine BLAKE3 identity; the N-Triples arm drops
  that body before expanded output. Native parsed terms move through the flat
  default graph API after named-graph refusal; both formats compare exact
  selected canonical bytes. Registry/typed projection ownership is supported by
  the retained native and command reviews, exact equality/pins and actual
  report/command fixtures. The last ambiguous command-values sentence now
  explicitly distinguishes check/coverage adapters from all command results.
- Usage: actual public Graph methods, native-term insertion and borrowed/native
  storage transfer match the example and adapters. `from_nodes` sorts/adopts
  original storage; no second complete graph is rebuilt just before output.
- Examples: the actual example selects only absent/nt arguments and emits all
  stated primitive/custom/control/provenance values. Census public report APIs
  retain both supported formats with their separate shape validation.
- Constraints: the three existing closed-profile/domain restrictions match
  actual refusal tests. Native allocation/admission limits are now also retained
  in judged Constraints, not solely Explanation. The new public test proves a
  real 64 KiB literal is accepted in both formats and a native-accepted comment
  is subsequently refused as noncanonical. Source inspection establishes the
  public API has no budget argument and calls default native parsing/Resident
  output. This is not a measured allocation bound, universal stack proof or
  permission for consumers to omit their own admission.
- Explanation: the native allocation/cost rationale and one-preimage reason
  match the implementation. Ordered maps and sorting prevent an unsupported
  linear-reader claim. This section no longer exclusively owns the operational
  absence of profile allocation/stack admission.
- Verification: the listed executable profile/report/command fixtures exercise
  the claimed cases. The latest profile run has all 13 passing and RDF all-target
  warnings-denied Clippy passed (83345). Independent parser pins remain distinct
  from native canonical golden files; no independent parser execution was
  invented by the new profile test.

Relevant source reviews include `queue-purrdf-native-review.md`,
`queue-purrdf-native-recheck.md`, `queue-primary-native-cutover-review.md`,
`queue-typed-command-review.md` and its actual repair/recheck artifacts. These
are source review evidence, not a substitute for the mandatory final full gate.
The current focused receipts are `queue-rdf-admission-profile.log` and
`queue-rdf-admission-clippy.log`. Whole grouped delivery, source integration,
other assessments and lifecycle/full gates remain open.
