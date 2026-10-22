<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Enforce preferred providers through Census

This is a proposed implementation contract, based on the current source and
capabilities. No product edits, builds or posting were performed. The explicit
user requirement is that Census express “always use library or project X for
functionality”, with RAF preferring PurRDF for applicable code in every target.

## Current evidence

The delivered Census foundation computes units, normal forms, groups and
in-memory job matches. `JobForbidden` currently matches body/shim/table
fingerprints and integer constants. Its documentation explicitly says rule
fingerprints match no unit. `Census::load` builds a source Workspace, collects
plain unit data, and drops the Workspace before returning. It does not retain
a dependency-aware provider graph.

The ledger, fourteen built-ins, declarative patterns, `Rule`/`Gate` and reference
scan are still **designed**. The current written capability still describes
Cargo metadata, but the newer owner's issue comments supersede that design:
reference resolution must read Cargo.lock and manifests through raf-source,
without invoking Cargo metadata or parsing its JSON. Local pattern aliases
must use raf-source as well; dependency-crate resolution is owned by issue 48.
Containment-checked external sources, external homes and reference-copy findings
remain requirements. That dependency index is not implemented by the current
Census foundation. The designed Rule is file-local; its FileCtx is package/path/target
context. It cannot by itself establish an external call's package identity,
cross-file dataflow or the absence of an equivalent algorithm.

## Immediate repository policy

Suggested authoritative AGENTS text:

> PurRDF is RAF's preferred provider for all reusable functionality it supplies,
> across shipping code, tests, examples, benches and tools. Before adding or
> retaining a local implementation or another dependency, inspect the applicable
> public APIs and current implementation in `/home/paudley/Active/purrdf` on
> `main` and use the `purrdf_*` provider. A dependency declaration or token call
> does not demonstrate reuse: route the actual production caller through the
> provider and remove the duplicate implementation. If the provider lacks a
> required capability, name the exact API or contract gap; do not silently
> substitute a local implementation or another library.

This is a current engineering requirement. The automated gate described below
must not be represented as implemented by adding these words. Source inspection
of current upstream main and the gate's exact locked dependency revision serve
different purposes; neither should be silently substituted for the other.

## Smallest coherent integration

Add a **Gate-level preferred-provider check** alongside the designed reference
scan. Reuse the ledger's jobs and external `home`/`entry_points`; do not create
a competing fingerprint engine or a scanner that guesses semantics from crate
names. Add a closed provider declaration and a provider requirement on a job:

- A library provider selects one exact Cargo package identity from the source-only
  dependency graph derived from Cargo.lock/manifests through raf-source. This
  requires the dependency-crate work owned by issue 48; do not add a Cargo
  metadata subprocess or a parallel JSON dependency resolver. A project provider
  selects a configured set of such packages under one
  logical id, such as PurRDF. Package selection must include name/version/source
  disambiguation when Cargo contains more than one candidate. Dependency renaming
  is resolved through Cargo's actual dependency edges. A `purrdf-*` prefix may
  select candidates for the configured project; the prefix is not evidence of
  implementation reuse or trusted publisher identity.
- The job names the functionality, allowed provider symbols and the first-party
  **consumer entry points** whose actual implementation must use them. Bind
  consumers to the relevant capability code bindings rather than to a directory
  naming convention. The gate refuses missing/stale/ambiguous consumers, homes,
  provider packages and rule registrations.
- Provider resolution is workspace-level evidence owned by Gate. Preserve the
  existing file-local Rule contract for structural syntax checks. Pass their
  hits into the same job classification/report path; do not hide mutable global
  provider state inside a file-local rule or run a second resolver per file.
- Register all check kinds through the same Gate and expose that Gate in the
  production self-census test. The CLI must either run the declarative policy
  completely or explicitly require the registered Gate path; it must not report
  clean after omitting required rules.

A suitable capability is `cap:census-provider-policy`, depending on ledger,
reference scan and rule API. A dedicated tracked change gives this added user
requirement explicit ownership. It can be implemented with the detector/rule
delivery, followed by RAF adoption; the current RDF renderer delivery must not
acquire an unrelated partial Census implementation while both mutation lanes
are occupied.

## Evidence and what each check establishes

Keep findings typed by the evidence actually obtained:

| Evidence | Enforceable claim |
|---|---|
| Resolved external call/type path | This call/type names an allowed or forbidden provider symbol at a pinned package identity. |
| Reference normal-form equality | This local unit duplicates this particular provider unit under this normal form. |
| Built-in or registered structural match | This unit contains the exact forbidden structure the rule specifies and fixtures demonstrate. |
| Checked consumer adapter | This declared entry point's input/result path delegates the specified operation to the allowed provider. |

An allowed dependency, `use purrdf_*`, or one reachable provider call does not
prove the last claim. A dummy call beside a local parser is still a local parser.
For the concrete RDF case, the adapter check must follow the input bytes into
`purrdf_rdf::parse_dataset_with`/`parse_dataset`, then follow that parsed dataset
into the validated RAF projection and returned result. A fixed, bounded AST
adapter recognizer can prove supported straight-line forms; it must refuse an
unsupported/ambiguous flow with an actionable location, rather than mark it
delegating. It must not accept an unreachable branch, ignored return, shadowed
callee or call on unrelated input. Compiler execution is not silently added to
the existing source-only Gate contract.

Reuse normal-form matching to catch renamed copies of known provider bodies.
Use registered structural checks for exact recurring local implementations
(for example, the planted cursor/UCHAR grammar fragment), with messages stating
the structure recognized. An arbitrary independently written algorithm may
escape both. Neither a function-name match nor a broad “looks like a parser”
score is proof that two programs implement the same functionality.

The global preference is repository policy plus explicit functional obligations.
Automation can enforce the declared provider mappings and recognized forbidden
structures. It cannot infer every function's purpose or prove the absence of
all semantically equivalent programs. Keep uncovered functionality visible in
the provider audit/coverage mapping; do not print “all code uses PurRDF” merely
because every current pattern returned zero hits. Adding a new capability or
changing its consumer bindings must update its provider decision and fixtures.
RAF-specific schema validation and digest-preimage policy remain stated local
contracts; they should call shared primitive providers without fake delegation
to an API that does not supply that domain policy.

## Failure paths and neighboring fixtures

Each registered check needs a failing fixture and a close valid neighbor. The
minimum production acceptance is:

- A renamed dependency and re-exported provider call resolve to the allowed
  package; a same-spelled local module/function fails. Multiple locked package
  versions require disambiguation. Missing source/API/dependency fails, naming
  the exact provider and required action; no network fetch or silent skip.
- A consumer whose actual bytes and result flow through the allowed parser
  passes. A dummy parser call plus the old local parser fails; ignored return,
  changed input and an unreachable call fail. Local closed-profile validation
  of the provider result passes and is not called a duplicate grammar.
- A copied provider body with renamed locals fails by reference fingerprint;
  an unrelated same-shaped domain transformation is not called an algorithm
  copy without the actual normal-form evidence.
- A non-clone local UCHAR decoder matching the specified structural detector
  fails; using `purrdf_lex::terminals::decode_uchar` passes; a neighboring byte
  loop doing a different job remains clear. The detector states its exact
  syntactic scope, including what it does not establish.
- Production, test, example and bench targets all exercise the preference;
  merely changing target kind cannot hide a forbidden implementation. A file
  reached by several targets is not double-counted and is not omitted.
- An unregistered rule, unsupported consumer flow, unresolved alias/method or
  stale provider symbol fails with a typed incomplete-policy-evidence error.
  A neighboring fully resolved supported form passes. Budget/containment
  refusals propagate as failures with exact-limit/one-below neighbors.

An actionable diagnostic should identify job, consumer file/line/symbol,
evidence kind and witness, allowed package/API, and repair, for example:
“rdf-text-parse: raf_rdf::read::parse contains the forbidden local UCHAR cursor
structure at read.rs:63; delegate text parsing to
purrdf-rdf::<locked identity>::parse_dataset_with and retain RAF profile
validation.” A dependency failure should instead say which source/API is
unavailable. Do not attach an algorithm-copy claim to either message unless
the reference fingerprint actually matched.

Provider violations and incomplete policy evidence are hard gate failures,
not debt to baseline away. Existing sanctioned-variant mechanisms do not grant
an exception to the user's blanket PurRDF preference.

## Queue ownership and scope preservation

The captured issue bodies put built-ins, patterns, Rule/Gate and reference scan
in **issue 5**. Extend that delivery with the shared provider-policy capability
and explicit new acceptance above; retain all fourteen built-ins and every
original pattern, fixture and reference-scan requirement, incorporating the
owner's source-only resolution direction above. Ledger schema and strict parsing
depend on **issue 4**; provider identity and cross-crate aliases require the
actual **issue 48** source dependency index. Source-independent structural rules
can proceed, but complete provider resolution cannot be claimed before that
prerequisite exists.

**Issue 9** contains the two flagship workplan scenarios and self-workplan plus
self-census CI gates. It owns RAF's actual preferred-provider adoption and
self-census wiring, alongside those original scenarios. Do not replace the
scenarios with provider tests or mark the parent delivered while they remain
unfinished. The CLI/registered-rule path must preserve the existing delivery
dependencies as well. Record this mapping in the portfolio and affected issue
scope when primary resumes authorized issue updates; this report posts nothing.
