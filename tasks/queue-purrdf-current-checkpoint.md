<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Current native reuse checkpoint

This is a focused implementation checkpoint in the existing checkout, not a
delivery, lifecycle, adoption, full-gate, commit or merge claim. Historical donor
evidence and stashes remain preserved. No PurRDF source was changed.

## Production cutover

- Workspace dependencies select PurRDF 3.0.1 at immutable revision
  `5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43` from the authorized HTTPS repository.
  Registry 3.0.1 is a distinct source and is not substituted for the pin.
- Native `TermValue`/`TermBox`, native RDF parsing, flat native values, native
  ordering, traversal and term output replace the private RDF algebra/parser and
  generic writer. Named graphs are refused before flattening; exact root and
  fragment ownership, roles, depth-one accepted terms, dangling references,
  duplicate/empty storage and exact canonical bytes remain RAF profile laws.
- Census validates the same native Nodes storage and transfers it in place.
  Its obsolete ground conversion is deleted. Domain cardinalities, full
  fingerprints, start-byte identities, named reifiers, budgets and report form 2
  remain. Normal-form 1 token framing and all fingerprint golden bytes remain.
- Both formats derive identity from one normalized Turtle body. The reader now
  reuses the renderer's returned digest and canonical text instead of making a
  second normalized preimage. N-Triples drops normalized text before expansion.
- Native BLAKE3 and canonical hex cover census fingerprints and reports plus
  tools; native W3C vocabulary constants replace copied standards names.
- Native SCC APIs replace both handwritten Tarjan implementations. Workplan
  keeps explicit sorted members/groups and singleton self-loop detection;
  source uses the native reverse-topological component-index law and a thin
  grouping adapter. This does not import sibling resolver prototypes.
- Native exact Decimal supplies OKF value storage, comparison and f64 display;
  authored negative zero remains signed, lexical grammar stays narrower than
  XSD and Number equality still compares spelling. Native exact products supply
  line-ceiling comparisons with no generated decimal products or parse-to-false
  fallback. Negative zero, long fractions and maximum counts have neighbors.
- The compiler oracle uses native JSON values/accessors directly. Its private
  JSON algebra/parser is deleted. Missing or mistyped required diagnostic level
  is an actionable failure; legitimate null code remains accepted.

## Actual API limits

PurRDF's public ContentDigest supplies one-shot SHA-256, not a streaming state.
One-shot consumers use it. Existing multi-field/module digest consumers use the
same sha2 implementation as PurRDF, preserving NUL/decimal-length framing without
materializing entire module preimages; the copied compressor is removed.

The accepted graph depth is not a preparse resource ceiling. Native parsing
precedes closed validation. Consumer input-byte admission remains, and native
Resident traversal does not assert an allocation ceiling. No new WASM or native
platform qualification is claimed.

Registry-parser pins retain exact input/output and the preserved registry lock.
That is a distinct-revision check in the same parser family, not an independent
RDF implementation. Native goldens remain separate. The one comparison rule
normalizes actual U+0085 to the registry escaped spelling and retains a literal
backslash-u0085 neighbor.

## Focused execution

Every successful row below has a terminal exit 0 from the actual tool wrapper.
Earlier failed migration/lint runs remain historical evidence; they are not
cited as current success. The first combined native final-focused command
returned 101 because of seven old records-fixture assumptions; those were fixed
with the substantive fixture/policy intact, including an attached claim, and the
current combined focused command passed.

| Scope | Actual result | Raw log |
| --- | --- | --- |
| Production report library/CLI callers | compile pass | `queue-purrdf-production-callers.log` |
| All targets of four report packages | compile pass | `queue-purrdf-all-callers-3.log` |
| All source targets and native JSON consumers | compile pass | `queue-purrdf-source-all-callers.log` |
| Census forms, NF1 goldens, reports; kernel profile | 29, 4, 25, 12 tests pass | `queue-purrdf-profile-census-2.log` |
| Current kernel, OKF profile/records, workplan adapters/records, full CLI registry | all selected targets pass | `queue-purrdf-native-current-focused.log` |
| Native SHA-256 framing/vectors | 5 tests pass | `queue-purrdf-digest-tests.log` |
| Workplan graph integration and native cycle findings | 22 tests pass | `queue-purrdf-workplan-graph.log` |
| Workplan coverage unit families | 20 tests pass | `queue-purrdf-coverage-tests.log` |
| Final native percentage and line-work neighbors | 8 tests pass | `queue-purrdf-native-percent-2.log` |
| Existing source resolution/order permutations | 104 tests pass | `queue-purrdf-source-resolution.log` |
| Native source component law and long chain | 1 test passes | `queue-purrdf-source-components.log` |
| Native JSON, metadata refusal, actual compiler callers | 17 tests pass | `queue-purrdf-native-json-oracle-2.log` |
| OKF exact numbers, record writing, allocation-bound fixture | 36, 13, 1 tests pass | `queue-purrdf-okf-number.log` |
| Number grammar/comparison doctests | 2 pass | `queue-purrdf-okf-number-doc.log` |
| Current all-target Clippy, six RAF crates, warnings denied | pass | `queue-purrdf-all-clippy-current.log` |
| Root supply-chain closure | pass | `queue-purrdf-deny-root-2.log` |
| Fuzz supply-chain closure | pass | `queue-purrdf-deny-fuzz-3.log` |
| Production documentation sync | pass | `queue-purrdf-docs-sync.log` |
| Registry parser replay of all 36 fixture families in both MIME-selected formats | all 72 inputs match their exact pins | `queue-purrdf-all-oracle.log` |

The root/fuzz policy retains global duplicate-version denial, permits only the
pinned Git source, and scopes Unicode/BSD additions to audited native packages.
The exact syn 2.0.119 exception is required by native RDF's unconditional
wasm-bindgen macro closure alongside RAF's syn 3. Fuzz jobserver is unified at
compatible 0.1.34, removing extra getrandom/r-efi versions without new exceptions.

The registry replay command is retained in `raw/queue-purrdf-all-oracle-command.txt`.
Exact output for every actual input is retained under `raw/purrdf-all-oracle/`;
the source and locked registry dependencies remain in `scratch/rdfprobe/oracle/`.
The archived earlier helper remains untouched in its historical Git ref.

The single census-report-read sanitizer run completed all 5,000 runs with terminal exit 0 in `raw/queue-purrdf-report-fuzz.log`. Subsequent scalar cleanup delegates directly to native integer/boolean constructors; all-target callers, focused reports/adapters, full-width unsigned integer neighbors and fmt passed with exit 0 in `queue-purrdf-native-scalar-callers.log`, `queue-purrdf-scalar-focused.log`, `queue-purrdf-scalar-wide-neighbors.log` and `queue-purrdf-scalar-fmt.log`. The sanitizer result predates that byte-equivalent constructor cleanup.

The current root run returned exit 101: old root fixtures/inventory need repair, and actual adoption has 18 findings. The actual no-overlay default check returned exit 1 with 16 unbound compiler-diagnostic test copies and two zero-ceiling test-line findings. Its raw canonical graph and semantic inventory are `queue-purrdf-default-check.ttl` and `queue-purrdf-default-findings.log`. These are current implementing-state findings, not the full delivered-state adoption remainder. After staging only the three authorized removed-file deletions, all 71 repository gates passed with terminal exit 0 in `queue-purrdf-repo-gates-index-repair.log`. Root fixture repair and genuine test ownership are in progress; no full-gate or assessment-currency claim is made.
Lifecycle and genuine zero-debt adoption remain open; no baseline, policy opt-out
or exclusion shortcut is introduced.

## Current root and adoption disposition

The subsequent root docs-contract target passed all 31 tests. The self-workplan target then failed only on stale accepted assessment inputs; the combined command returned exit 101, preserved in `raw/queue-purrdf-root-adoption-repair.log`. No stale record has been mislabeled current. Final scalar all-target Clippy passed with terminal exit 0 in `raw/queue-purrdf-final-scalar-clippy.log`.

The no-overlay default check subsequently passed with terminal exit 0 in `raw/queue-purrdf-default-after-ownership.ttl` (empty stderr). That snapshot follows genuine compiler-oracle test bindings and removal of an unused fixture constant. It predates the additional Tree proof tests and final claim edits. Capability states remain implementing and open lifecycle records remain; this success does not claim delivered-state adoption. The fresh stronger candidate in `raw/purrdf-adoption-current/` completed with exit 0 as a diagnostic tool and reports 217 findings, zero public-item gaps, zero test gaps and 186 uncovered claims. Expected candidate state/open-record conflicts are included. No candidate overlay was applied to product files.

The root sample now uses a real integration call and executable example, substantive prose and scoped claim markers. The repository-only duplicate matrix-marker check is removed in favor of production’s stricter section-scoped rule; missing/quoted-marker refusals and the valid neighbor all pass. The inventory pins 24 capabilities, their actual current states and 11 open records, and package discovery explicitly includes raf-rdf.

OKF claim mapping splits body coordinates, headings, sections, links, split links, fences, generated/proposed spans, prose, evidence/test markers, marker refusals and atomic edits. Resource-owner fixtures retain their finite scope: the allocator test covers its two refused-input families with the fixed allowance, and the 20,000 token-soup cases are not an exhaustive theorem. Nine focused owner targets passed with terminal exit 0 in `raw/queue-purrdf-okf-claim-owners.log`. Source Tree proof repair adds UTF-8 byte-cap neighbors and an actual Rc<Cell>-backed &dyn Tree Workspace consumer, and extends poisoned-tree identity/refusal neighbors. Both memory_tree (14 tests) and relpath (8 tests) pass with terminal exit 0 in `raw/queue-adoption-tree-owners.log`; check the exact native log count rather than infer final adoption from these fixtures.

## Primary scalar review repairs

The primary bounded source review is `tasks/queue-primary-native-cutover-review.md`. Its retained-value and supplied-reader actions are implemented. `Number::exact()` borrows the native Decimal already retained at construction; actual line ceilings use it directly, with native Integer::from(u128), rather than reparsing Number text or assembling BigInt manually. Comparison rustdoc describes the current native value law. Census now uses native exact integer canonical output and native integer/boolean readers. Nonnegative, full-u128 width and canonical lexical laws remain domain checks. A canonical u128 has at most 39 bytes, and that domain bound is checked before arbitrary-precision parsing. Native boolean 0/1 acceptance does not broaden the report profile, because canonical Term equality is also required.

The final report target passed all 26 tests with terminal exit 0 in `raw/queue-purrdf-native-scalar-readers-final.log`; this includes both-format scalar refusal/valid neighbors, the 40-digit refusal, u128::MAX, max-plus-one and unchanged goldens. Actual retained-value ceiling tests passed all 8 with exit 0 in `raw/queue-purrdf-retained-number-ceilings.log`; Number public parse/exact/compare doctests passed all 3 with exit 0 in `raw/queue-purrdf-retained-number-doc.log`. Four-package all-target Clippy passed with exit 0 in `raw/queue-purrdf-retained-scalar-clippy-2.log`; the earlier log records a corrected rustdoc finding and is historical. Root and fuzz locks include the direct census XSD dependency, with no new selected native revision.

One relevant 5,000-run sanitizer replay is currently running after these scalar edits, retained as `raw/queue-purrdf-final-scalar-fuzz.log`; its exact command is adjacent in `raw/queue-purrdf-final-scalar-fuzz-command.txt`. Production Rust source is frozen for that execution. The earlier 5,000-run success is not substituted for this current result.

Additional concrete adoption fixtures passed before this scalar recheck: the same-DiskTree changed-source reload, exact manifest member/exclude boundaries, directory alias and unsafe-parent refusals with a real parent neighbor, and dangling-manifest refusal without ancestor fallback. Logs are `raw/queue-adoption-load-boundaries-2.log` and `raw/queue-adoption-root-parent-neighbors.log`; source all-target Clippy passed in `raw/queue-adoption-source-proof-clippy.log` (the later alias/parent expansion is separately tested). The actual CLI history RDF test passes after adding both formats, exact Git author-date comparison, entry/source/path ordinals and exact last-resolved parent in `raw/queue-adoption-history-rdf-fields.log`. These proof repairs do not complete the other uncovered claims.
