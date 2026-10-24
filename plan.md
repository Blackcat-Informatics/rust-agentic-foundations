<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Registered check delivery

Started 2026-10-24T05:33:50Z; four-hour deadline09:33:50Z. Base is published
parent2b8af84. Land63 before this delivery. No64 builds while61 full85083 runs.

## Contract and real callers

Wire an explicit package/example registered `check` into cargo-census. Invoke the
existing frozen `Invocation::example_run` with `Programs::new(workspace_root)`
and bounded `SystemRunner`; no target arguments or invocation extension. The
target emits one canonical N-Triples1.2 response with check-v1 protocol IRI.
Registration remains repository-owned and can call the actual locked Gate.

Use raf-rdf native storage, indexed views, term types, canonical parser/writer
and snapshot identity. Reuse existing Census shape scalar/cardinality helpers.
The local adapter owns only registered-check classes, exact fields, ordered
finding membership/multiplicity, outcome and exit consistency. Retain the native
Graph; do not build another assertion map. Classes must not conflict with the
existing Workplan Finding projection. Preserve all finding strings/order and
evaluation errors. Unknown shape/version, partial response, process failure,
oversize output, UTF-8 failure and status disagreement are hard errors.

CLI migration/debt retain their real existing path. Registered check never
falls back to built-in-only evaluation. Check-v1 cannot write debt or be silently
repurposed into a write operation. Parent12 retains explicit write authorization,
exact ratchet and registered debt. Parents6/9/40/53 retain the full command set,
machine vocabulary, automatic observed compilation closure, self-census,
read-only sibling trials and complete native adoption.

## Requirement to acceptance

| Surface | Production proof |
| --- | --- |
| Native response codec | Real Gate result and error projection; duplicate messages/order retained; strict unknown/version/shape/cardinality/count/ordinal/canonical/size controls with valid neighbors |
| Host transport | Existing Runner/Invocation only; scripted output/exit/refusal cases; actual missing-target and compile-failure runs; no fallback |
| Registered Rule/Extension | Real fixture example calls registered Gate; actual shipped CLI clean and planted findings equal Gate findings and leave inputs/debt untouched |
| Root authority | Execute CLI from another caller cwd with explicit workspace; target reads requested workspace |
| Configured native caller | Wire real locked Gate through target; native delegation positive/competitor negative, plus actual pinned Hasher witness through CLI |
| Lifecycle | Current CLI capability subset and proposals for retained scope; binding inventory, native completion, relevant assessments including indirect stale closure |
| Qualification | Focused API/caller compile early, independent focused review, required make check, normal signed hooks, hosted checks/feedback, exact ghprsq archive and owned cleanup |

## Implementation order

1. Native response codec and direct library/target caller. Compile these before
   extending fixtures once the61 full gate releases the compiler lane.
2. Typed CLI selection and bounded transport using existing host APIs. Add real
   target fixture and protocol/exit/registration controls together.
3. Actual locked/pinned native proof, capability lifecycle and settled gate batch;
   publish and qualify while respecting the already active parent merge lane.

## Reuse and limits

Current PurRDF main8652dee has no public bounded host process API: inspected
private helper-census metadata/Git and wasm-link calls are unbounded; SARIF
Invocation and property Runner are unrelated. Existing RAF exec is the actual
bounded host boundary. Main Graphify snapshot12,317 nodes was queried before
source inspection, but broad results truncate and predate this branch; current
source governs. Explicit repository bank resolved and relevant native RDF/profile
lessons recalled; no global bank. No cross-repository implementation authority.

One implementation lane64;61 is qualification-only. Two build jobs, no stacked
cold/full suites, preserve source/glob donors and validation baseline. Windows
hosted runtime remains waived/unverified under the existing explicit exception;
other local and hosted gates remain required.
