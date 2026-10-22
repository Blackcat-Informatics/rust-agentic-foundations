<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Typed commands and default-policy checkpoint

The existing issue 16 checkout remains at base commit
`62bc8395ca424b4a9e5b6591d9ff20852a95b7b7`, with its RDF, command, sole-policy
and adoption changes uncommitted. All historical donor stashes, parser outputs
and unsuccessful intermediate logs remain preserved. No full gate, commit or
push was performed for this active delivery. Product capability states remain
implementing and the three active delivery records remain open; the designed
census capability records are also preserved. This checkpoint is ready
for primary review; it does not qualify the complete delivery.

## Current implementation and evidence

- Both machine formats project one closed typed command-result graph. Evidence
  failures retain failed versus unresolved outcomes; moved inputs retain raw
  unresolved reasons. Diagnostic writes and flushes propagate errors, and real
  failed help/version output returns exit 2. Human/quiet/exit semantics retain
  their explicit neighboring assertions. The independent runtime recheck is
  `queue-typed-command-recheck.md`.
- The deterministic registry retains real Cargo execution, stdout, status and
  proof outcomes while controlling only build stderr in two native fixture
  producers. It performs no graph or message rewriting. Independent review of
  this input boundary and the actual allocator ownership is
  `queue-registry-allocator-recheck.md`.
- All 28 native result families and all 152 documented command terms pass exact
  registry assertions. The current result is
  `raw/queue-current-policy-registry.log`. The unified independent MIME-specific
  parser replay passed every format pair and oracle pin in
  `raw/queue-typed-command-oracle-final.log`; the native bytes are unchanged by
  subsequent diagnostics wording, as the current exact registry confirms.
- The latest selected all-target Clippy check passed for cargo-workplan,
  raf-workplan, raf-okf, raf-source, raf-census and raf-rdf. All 272 workplan unit
  tests and all 20 complete-policy integration tests passed. The batch returned
  exit 0; logs are `raw/queue-current-policy-{clippy,unit,integration}.log`.
- Census unit integration passes all 31 tests, including caller-local
  `Rc<Cell<usize>>` Tree access, written versus generated items and initializer
  versus attribute metadata. Its 29 form tests pass, including documentation
  versus ordinary comments and the whole-body binding limitation. See
  `raw/queue-census-adoption-units-current.log` and
  `raw/queue-census-adoption-form-neighbors.log`. No normal-form bytes changed.
- The kernel's complete focused crate check passes its internal ownership test
  and 11 profile tests. New refusal neighbors cover prefixes, abbreviations,
  blank nodes and quads; a generic accepted graph demonstrates that domain
  shapes and datatype value spaces remain the consumer's responsibility.
  Evidence is `raw/queue-rdf-adoption-current.log`.
- The registry example has a real no-argument in-memory loading, placement and
  report demonstration. Explicit directory arguments retain the original native
  scanner. Its smoke invocation and six existing tests pass; the smoke does not
  claim registry-wide compatibility or child-orchestration coverage. See
  `raw/queue-source-registry-sample.log` and
  `raw/queue-source-registry-neighbors.log`.
- The three fuzz targets call named actual exercise functions. Their unchanged
  bodies are now directly owned instead of relying on a crate-root binding that
  cannot reach ordinary contained items. The harness all-target caller check
  and Clippy pass in `raw/queue-fuzz-owned-entry-{callers,clippy}.log`. Live fuzz
  qualification remains coordinated with final source changes.
- The emitter helper now uses a direct lifetime-parameterized callable type
  alias, removing an empty trait and blanket implementation with no additional
  semantics. Real raf-source all-target callers compile in
  `raw/queue-source-emitter-callers.log`; selected current Clippy also covers it.
- Actual sync refusal proof is strengthened: a malformed Change joins the
  capability refusal neighbors; ineffective permission fixtures fail visibly;
  the unchanged rename phase has a private production helper and a deterministic
  real filesystem late-failure/success neighbor. Earlier published files remain,
  unpublished targets and sentinels remain untouched, and temporary tails are
  removed. The three focused logs returned exit 0. Independent review is
  `queue-sync-proof-recheck.md`.
- Current documentation removes obsolete optional-mode descriptions and the
  removed debt-without-strict rule. Generated-content cleanliness is accurately
  distinguished from unrelated check findings. Production docs sync returned
  exit 0 in `raw/queue-adoption-checkpoint-docs-sync.log`.

## Remaining acceptance and next coherent work

The settled checkpoint's stronger delivered-state diagnostic reports 225 findings,
zero public-item gaps, zero test gaps and 188 uncovered claims. It retains 56
orphan lines across the packages rather than excepting or baselining them. See
`raw/queue-adoption-checkpoint.log` and `raw/adoption-checkpoint/`. These are
diagnostic snapshot counts with in-memory candidate states; product states are
unchanged. Open-record consistency findings in that candidate are expected and
are not waived from actual lifecycle completion.

The source-format-capture omission is still a real owned production defect.
`raw/queue-format-capture-gap.log` and `queue-format-capture-design.md` preserve
the actual public reproducer and bounded design. Its repair must retain decoded
capture names, distinguish real standard macro identity from textual/local and
imported shadows, suppress explicit named format bindings appropriately, handle
width/precision and raw/cooked strings, resolve values in the actual unit context,
and charge retained storage and counted work. Do not replace it with unsupported
test-constant binding roles, redundant format arguments, exclusions or a baseline.

Remaining claim adoption is substantive. The named-clause maps separate actual
assertions, candidate tests and source/policy proof gaps. Constraints markers must
follow the supporting paragraph or list item; a marker after its heading does not
cover its text. Source memory/platform/audit claims and authoring policies must
remain honest; do not substitute arbitrary nearby tests or delete guarantees to
produce a zero count. Current implementing states are deliberate, not completion.

After source repair and clause reconciliation, run the actual zero-debt checks,
review the final states/inventory, refresh current assessments with SPDX preserved,
complete the coherent production change lifecycles with their actual bound runs,
then obtain primary review and the single coordinated full gate. Parent RDF scope
still includes census debt and future census CLI/protocol delivery; parent census
portability remains independently open. Neither foundation acceptance nor a debt
record closes those remaining requirements.
