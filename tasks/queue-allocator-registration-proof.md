<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Compiler allocator registration prerequisite

The native allocation probe migration removed direct test-body reads of its
global allocator instances. The actual default check in sequence 26838 stopped
with seven genuine unowned test lines: two compiler-registered statics and the
heading allocator's hard ceiling. Its exact finding is retained in
`raw/queue-owner-repairs-default.ttl`. RDF completion did not start in that sequence.

Direct `code` bindings are not a valid remedy: production
`bindings::wrong_role` requires shipping source, while `tests` requires a test
function. Tentative entries were removed. No fake reference, scope exclusion,
debt baseline or ceiling change was introduced.

## Actual repair

`coverage::lines::module_units` already parses each file through the source
library's bounded worker. That same parse now visits native syn nodes for direct
bare `global_allocator` attributes and exact static byte spans. Collection maps
these spans to the existing indexed Static definitions and retains their real
target/test roles. No raf-source product/API change or second parse is needed.

Before the production `Workspace::reachable` call, registered statics are added
only for targets present in actual bound or extra roots. A test-only definition
also requires a participating test context. Their real initializer references
reach dependent constants and types. An unbound target contributes no root.
Malformed, repeated, non-static or unindexed registrations abort collection and
produce the existing error-only report, before totals. Macro and cfg_attr
expansion remain outside the direct-registration observation.

The five new actual Workplan caller tests prove direct/removed/cfg_attr neighbors,
real initializer reach, bound/unbound targets, code versus test-only context,
all three malformed shapes, unindexed nested static refusal and valid module
neighbors. They preserve test-line classification and assert empty partial
results on failure. Macro-expansion opacity is source-inspected and composes
with the existing no-expansion law; these are not five executed allocator-macro
expansion proofs.

## Owner execution

- Production all-target workplan caller check: exit 0, session 38234;
  `queue-allocator-registration-callers.log`.
- All 28 line integration tests: passed in sequence 24714;
  `queue-allocator-registration-lines-current.log`. Its following Clippy found
  five empty-assert style issues; the source assertions now compare exact empty
  slices. The earlier two test accessor compile errors remain historical.
- Final warnings-denied workplan/CLI all-target Clippy: exit 0, session 25530;
  `queue-allocator-registration-clippy-current.log`.
- Actual docs sync, exact source-index byte comparison and production default
  CLI check: exit 0, sequence 86333. The canonical
  `queue-allocator-registration-default-current.ttl` has no findings. Earlier
  new binding indentation caused a strict document load refusal; that refusal
  and the refused sync are retained separately, not presented as clean checks.
- Independent bounded source/assertion review:
  `queue-allocator-registration-review.md`, no concrete defect found.

The native adoption prerequisite is resolved on this focused tree. Fresh
affected assessment contexts and production RDF completion remain active work.
The broader source-only claims, remaining assessments, sole-policy adoption and
mandatory combined full gates have separate acceptance; no completion is inferred.
