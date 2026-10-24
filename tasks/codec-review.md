<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Registered-check codec and host review

Bounded static review of the initial registered.rs codec/tests, runnable example,
CLI dispatch and registered host. No builds, test execution or product edits.

## Concrete finding

Actionable failure diagnostics are lost for allowed process exits with invalid
responses. cargo-census/src/registered.rs retains stderr for exits other than
zero/one, but drops it for UTF-8, RDF/profile/version failures and exit/outcome
disagreement. A repository target whose Result-returning main fails before
emitting its response exits one and normally puts its actual cause only on
stderr. The host then diagnoses only an absent/malformed RDF response. Include
the captured bounded escaped stderr in these failure contexts while preserving
the typed codec/UTF-8 source and strict failure. Do not retry or interpret stderr
as a replacement response. Add an exit-one/empty-response/distinct-stderr control
beside a valid error-protocol neighbor.

## Verified source properties

No concrete schema/cardinality/order acceptance defect found. The native Graph
owns assertions and the canonical raf-rdf reader supplies canonical-byte/profile
validation. Root type, exact protocol IRI, scalar field cardinalities, statuses,
count/outcome agreement and error exclusivity are checked. Finding membership
has exactly count distinct links, canonical IDs encode unique in-range ordinals,
and the total subject count excludes extras. Together these enforce the complete
contiguous finding sequence without a parallel assertion map. Equal message
strings remain distinct ordinal findings; their original order is reconstructed
independently of native triple ordering. Unknown fields/classes, wrong string
types, dangling references and orphan subjects refuse.

Actual host APIs exist: SystemRunner::new, Programs::new, Limits::build,
with_max_output and Invocation::example_run. Check dispatch resolves gate.root(),
passes it as the runner cwd and uses the existing frozen package/example argv
without a shell, request arguments, invented TrustedArgv or process-runner copy.
Both streams use the native existing host output limit. Exit zero/one is checked
against the decoded outcome; target evaluation errors are distinct from ordinary
findings and no partial output is written before response validation. There is
no fallback gate or debt path in registered check.

The example proves only construction/round-trip of a clean owned fixture
response; it does not yet demonstrate requested-root acquisition, registered
Rule/Extension preservation or locked native provider invocation. Its gate call
uses check_tree over MemoryTree intentionally. The initial codec tests cover
order/multiplicity, evaluation errors, version, closed fields, outcome/count,
membership/ordinals/orphans and canonical/oversized input refusal. They do not
replace the owed real findings/native integration targets, actual missing/build
failure controls, cross-directory CLI execution or settled qualification.

The builder checks cumulative raw message bytes and the complete canonical output
size; the host bounds captured bytes and the decoder checks size before parsing.
The builder's raw-message counter does not bound RDF envelope/node overhead before
graph construction, so do not describe that counter as a total allocation grant.
No new memory-admission guarantee is inferred from the 16-MiB wire limit.

Disposition: initial source shape is coherent; retain the diagnostic finding and
complete actual integration/refusal fixtures and compilation before qualification.
