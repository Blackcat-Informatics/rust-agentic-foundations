<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Focused configured-source review

Reviewed the configured-source slice independently of its implementer: public
configuration inputs and receipts, manifest feature admission and requested-edge
destination checks, selected targets, syntax filtering, physical-file cache
context, and active PackageId-qualified lookup. Inspected configuration.rs,
configuration/features.rs, configuration/syntax.rs, cfg.rs, workspace/locked.rs,
the affected workspace.rs and modtree.rs paths, configured_sources.rs and the
source-index capability contract. This is not a review of the whole donor or
the complete source/provider contracts.

The implementation uses one existing resolver and bounded parse worker. Missing
configuration, inactive requested edges and incompatible physical-file contexts
fail explicitly. Native Memory, WorkList and sorted lookup supply reusable
mechanisms; syn owns Rust-specific parsing. No new generic utility or dependency
is required for this slice.

Findings and dispositions:

- Caller-declared predicates can differ between packages sharing a compilation
  triple. The API and capability now explicitly state that the caller supplies
  complete package-specific inputs; this is not observed Cargo state or a proof
  that those inputs came from one invocation.
- Filtering covers modeled declaration headers, not body statements, fields,
  enum variants or cfg-specific Census fingerprints. The documented boundary
  now matches the inspected traversal and preserves remaining provider work.
- Raw cfg identifiers initially kept distinct textual identities and could
  bypass reserved feature-key separation. Shared syn identifier normalization
  now unifies those identities before admission and matching.
- Root-qualified predicates initially lost their leading separator. Configured
  parsing now rejects qualified names/operators before normalization.
- Empty cfg_attr application lists were wrongly refused. Punctuated parsing
  now accepts the required comma with zero applied attributes and rejects a
  missing comma or predicate. Boolean literal parsing stays distinct from raw
  flags named true/false. Applied attributes retain ordinary syn Meta grammar.

The implementer's terminal receipts record 51 affected cases before the final
grammar delta and all 12 configured-source cases after it, exit 0. The latter
exercise each grammar refusal with valid neighbors, including raw flags both
present and absent. I inspected their actual public-load/lookup assertions;
I did not rerun them or claim separate runtime evidence. Earlier actual pinned
PurRDF lookup is useful scoped evidence, not full provider acceptance.

No concrete finding remains open in this bounded slice. The implementer's final
affected Source all-target Clippy receipt (68672) reports exit 0. Complete
delivery qualification/lifecycle, base integration and full original external,
std, macro, unsupported-item and differential acceptance remain separate and
unclaimed. This review does not authorize closing the full source/provider scope.

## Current-main consumer follow-through

After synchronization, current main adds a platform-evidence consumer of the
shared predicate grammar. The owner found that valid cfg_attr(P,) still refused
there. Independent tracing also found that its permissive predicate conversion
could drop a leading separator from cfg(::windows), allowing malformed syntax
to supply a platform exclusion. Both are concrete findings in that newly
integrated consumer. The owner is batching no-op/comma handling and strict
qualification checks before refreshing affected assessments. Earlier configured
runtime evidence does not close these platform-consumer findings; their final
source recheck and focused refusal/valid-neighbor results were required.

Final narrow recheck closes both findings. validate_attribute and
evaluate_attribute retain Punctuated comma information and use the shared strict
predicate conversion. A validated empty cfg_attr list is a no-op; absent
predicate/comma or qualified names refuse before a guard can supply evidence.
The owner reports all eight platform cases passed (20715, exit 0), and actual
test_platform_guard public-entry controls passed (69821, exit 0), including both
malformed root-qualified shapes, missing comma, ordinary platform guards and
valid empty applications under Linux/Windows facts. These are owner receipts,
not a duplicate reviewer execution. Affected lint and refreshed source-bound
assessments remain owner qualification; prior pre-delta lifecycle receipts are
historical. No concrete source finding remains open in this follow-through.
