<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Registered final delta review

Independent static review of current codec/public API, CLI-local errors and host,
real Cargo fixture, shared registrations and census-cli capability. No compilers,
tests, product edits or service actions performed.

## Required corrections

1. Capability test bindings and tested-by markers name test modules rather than
   executable test functions: raf_census::registered::tests and
   cargo_census::bin_crate::cargo_census::registered::tests. Workplan's actual
   bindings.rs:162 rejects a Test binding whose resolved item is not TestFn.
   Replace with these individual existing functions, and map claims to the
   relevant ones rather than relying on a module:

   - raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors
   - raf_census::registered::tests::protocol_version_fields_and_outcomes_are_closed_with_valid_neighbors
   - raf_census::registered::tests::membership_ordinals_and_orphans_cannot_drop_findings
   - raf_census::registered::tests::canonical_and_size_refusals_do_not_accept_partial_carriers
   - cargo_census::bin_crate::cargo_census::registered::tests::malformed_target_responses_preserve_the_actual_stderr_cause
   - cargo_census::bin_crate::cargo_census::registered::tests::process_failure_protocol_disagreement_and_gate_error_remain_distinct

2. Real fixture read-only assertions cover clean and planted evaluations only.
   After changing the extension owner, and after selecting missing/broken
   examples, the test checks status/diagnostic but not snapshots or debt absence.
   Snapshot each branch after its intentional setup edit and immediately before
   invocation, compare after it and assert no debt exists. The current code
   contains no unexpected writer, but these refusal paths do not yet establish
   the requested all-outcome read-only evidence. Existing snapshot enumerates
   exactly six fixture files, not a recursive proof of every possible new file.

## Source verdict and limits

No concrete codec acceptance defect found. The native Graph remains private;
constructors establish the invariants used by finding/error iterators. The
decoder now returns a typed missing-root refusal. Status, findingCount, error
exclusivity, exact version IRI, closed fields/classes, membership/ordinal IDs and
subject accounting ensure complete findings with preserved duplicates and order.
GateReport::is_clean is exactly findings.is_empty, so from_check's three outcomes
are consistent with the actual report. The canonical/size limits are wire-byte
constraints, accurately distinguished in docs from total graph allocation.

The earlier stderr loss is fixed for UTF-8, RDF/version and exit mismatch
failures. CLI-local Error preserves Census/Input source chains, keeps transport
errors local and retains registration-required exit two for the existing debt
path. Registered framed evaluation errors consistently return one, while
ordinary findings print the whole ordered sequence only after full validation.
No structural fallback, implicit debt route or second process runner exists.
Actual frozen invocation and requested-root cwd use the existing bounded host.

The real fixture shares exact policy bytes between direct Gate oracle and Cargo
example. Nonempty extension ownership and the required registered Rule have
negative controls; a documented root home and exact hit owner/line make the
planted finding meaningful. The fixture uses actual unchanged Cargo dependencies
and an offline-generated lock, and an unrelated caller cwd establishes root
selection. The check target is trusted executable code: host protocol validation
does not prove it evaluated policy honestly or sandbox its filesystem effects.
The capability correctly states that boundary and leaves source acquisition,
compiler authentication and registered writes outside check-v1.

Observed raw/registered-real-target-second.txt has one actual integration test
passing with zero failures. Owner-reported codec/transport/Clippy passes are not
reviewer reruns. Native CLI stage design uses exact committed provider source,
typed local/native associations and frozen BLAKE3 outputs, but that witness is
explicitly still compiling/debugging and is NOT acceptance evidence here.
Mandatory complete gate, contract reconciliation, fresh assessments, hooks and
hosted checks remain owner qualification. This review does not relabel pending
native or lifecycle execution as passed.

## Correction reread disposition

Both required corrections above are resolved in current source. census-cli
frontmatter now binds the four codec and two host test functions individually,
and its Closed native response and Failure/exit tested-by markers name those
same executable functions. The real invocation helper now snapshots all six
known fixture inputs before each actual CLI command, compares them after it and
asserts absence of census-debt.ttl. Every clean/planted/extension/missing/build
invocation uses that helper, so the assertions cover refusal controls after
their intentional setup edits. This remains an exact known-file snapshot,
not a sandbox or recursive new-file detector. Static disposition: PASS for
these fixes; reviewer execution was not performed.

Related documentation clarification: census-rule-api Constraints lines 169-170
say cargo census registers no rule/extension and the Gate owns the whole census
for such policy. The literal host-registration statement remains true, but its
unqualified CLI/Gate distinction needs the explicit registered check path: the
host now invokes a repository example that owns those registrations and its Gate.
Line 168 restricts write_debt, so it remains valid and needs no check exception.
Lines 172-173 describe the Gate itself not running a compiler, also still true;
the CLI builds its target outside that Gate. No other conflicting statement was
found in this bounded reread. No product edits or compilers were performed.

## Shared runnable registration/lifecycle reread

Bounded static verdict: PASS; no new substantive defect found. The runnable
registered example and actual CLI integration now import the same registration
and ledger helper sources under raf-census/examples/support. The integration
still copies the exact registration bytes to the actual Cargo target, so moving
the implementation does not weaken direct-Gate/target parity. The prior
six-input snapshot and per-invocation refusal checks remain intact. Relative
source paths resolve to the actual shared files; no replacement registration or
schema fallback was introduced.

The runnable example now executes a registered clean Gate, verifies the clean
canonical response, plants local_copy and asserts the exact registered-rule
finding fragment, then refuses the invalid owner before emitting the original
clean response. Production evaluate.rs:354 renders rule ID/message rather than
the hit symbol, so `rule no-local-copy: use home` is the correct discriminant.
The real CLI test additionally demands that fragment before comparing the
complete ordered finding output to its direct Gate oracle. This prevents an
unrelated finding from satisfying planted-rule coverage.

The shared ledger builder uses native Term::integer for the u32 form constants.
The exact pinned native source 5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43,
crates/rdf-core/src/ir/term.rs:624 supplies that lossless i128 constructor and
canonical XSD integer spelling; this is valid supplied-native reuse. Core
ledger/extension identity and positive/negative owner semantics are unchanged.

census-rule-api's runnable example binding and Examples text now match those
behaviors; its Constraints correctly distinguish the CLI host from the target
Gate registrations. The change's capability/affects entries include rule-api
Examples and Constraints. Independent initializer enumeration confirms 153
assessment records and 156 total records: seven CLI plus two rule-api additions
to the preceding 144/147 inventory. These counts preserve exact assertions.

The owner accepted-build log records compilation and actual execution of the
new example, and accepted stdout is a complete clean check-v1 response.
registered-real-target-final.txt records the real integration test passing;
registered-shared-example-clippy.txt ends successfully. These are owner receipts,
not reviewer executions. This reread does not assert fresh prose/digest
acceptance or complete qualification from those focused results alone.
Refreshed native lifecycle, mandatory make check, normal hooks and hosted checks
remain owner acceptance obligations. Earlier failed example assertions and prototype logs
remain historical evidence; none is relabeled successful. No product edits.

## Test-budget consolidation review

New standing user policy applies to pending work: adding a test requires
removing another. Independent diff count against main finds seven added and
seven removed Rust #[test] attributes, net zero. Added entries are four native
codec tests, two host transport tests and one actual Cargo integration test.
The seven retired entries are in reports.rs; no other test removals were found.

Static preservation verdict: PASS. The existing golden test is renamed
canonical_reports_preserve_golden_output_documentation_and_roundtrip and invokes
the four unchanged retired bodies covering expected fingerprint spelling,
documented output, both report read/write round trips and canonical Turtle,
then retains its original golden equality assertion. The existing shape test
is renamed closed_report_shapes_refuse_unknown_missing_mistyped_or_misowned_assertions
and invokes the three unchanged retired bodies covering unknown terms, node
names/form identity and orphan units/reifier completeness, then retains all
original missing/repeated/mistyped assertions. All seven retired bodies are
called; no assertion body or negative fixture was removed or weakened.

census-normal-form binds the two current executable functions and maps Closed
canonical reports to their appropriate claims. Obsolete old test references
are absent from capability documents. Current uncommitted diff is limited to
reports.rs and census-normal-form authored/generated bindings; the unrelated
capability edits mentioned by the owner are not present. Coverage is preserved
through coherent existing execution entrypoints with fewer test attributes.

Observed owner raw/registered-consolidated-report-tests.txt ends with 19 passed,
zero failed/ignored. This is focused owner execution, not a reviewer run. The
previous complete gate/hooks/commit precede this consolidation and do not
qualify the new delta. Fresh affected contract/assessment/sync checks, required
full gate and normal commit hooks remain owner obligations; no bypass is
authorized. The next backend delivery's extension of the existing debt example
adds no test/example entrypoint under the same policy. No reviewer compilers or
product edits were performed.
