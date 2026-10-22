<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Shared RDF first-consumer checkpoint

The bounded foundation is implemented in the existing issue 16 checkout and is
ready for primary review. Product source is frozen; all listed focused checks
have terminal exit 0. No product commit or push was made. This is not full
issue 40 acceptance, command migration, sole-policy adoption or a delivery-complete
claim. `work/changes/rdf-report-profile.md` remains open; the two affected
capabilities accurately remain implementing. Current assessments, actual lifecycle
completion and the coordinated full gate remain required before delivery.

## Integration and preserved work

Ordinary synchronization with origin/main completed as merge `62bc839`, including
the landed native collection correction, through the normal merge hook. Authored
maintainer policies and the reviewed mixed-edition repair remain intact. Before
the merge, tracked Cargo changes and the untracked kernel were preserved as the
exact stash `bbf20c6eca394cc9ab5baca288e09cd2a7b62361`, plus
`raw/queue-rdf-before-main.patch` and `.tar`. The stash was restored by exact OID
and retained. Earlier T8 donor stash and unique ignored evidence were preserved.

## Concrete API and domain ownership

- `raf-rdf` owns sealed validated `Iri`, atomic literals/IRIs, ground `Triple`,
  atom-only depth-one `Statement`, an owned ordered `Graph`, the two `Format`
  variants and typed errors. Absolute and fragment IRI validation uses the actual
  `purrdf-iri` 3.0.1 parser without normalization. Shared `purrdf-lex` 3.0.1 APIs
  own escaping, Unicode escape decoding and triple-term delimiters. The duplicate
  percent validator was removed. Language-only datatypes are refused without
  their absent language metadata.
- One fixed normalized Turtle spelling, excluding BASE, supplies the full BLAKE3
  identity for both formats. N-Triples expands the same owned nodes; its reader
  normalizes only the exact declared root and valid fragments. Native canonical
  bytes are required, including CR and final LF. No general RDF parser, blank
  nodes, quads, language tags, recursive terms, schema framework or alternate
  identity path was added. The N-Triples writer releases its preimage buffer
  before allocating expanded output.
- Census baseline/index retain their complete closed domain validator and all
  shape/cardinality/name/reference/classification checks. Moving conversions
  transfer terms between the domain index and kernel rather than cloning a
  second graph. The obsolete private lexer and writer were removed, together
  with their syntax-specific errors. Shared errors retain their source chain;
  digest failures retain declared/computed identities. Both report types now
  have `to_ntriples`/`from_ntriples` beside their Turtle methods. Report layout is
  explicitly 2; NF1 passes, hash bytes, full fingerprints, start-byte identities,
  table identities and version-free normal-form reading are unchanged.
- `raf_workplan::report` provides real typed Finding and item/test CoverageReport
  library adapters. Ordered members have explicit ordinals. Locations, package
  counts, scope, exclusions, failure variants and optional carry-budget paths
  remain typed graph fields. Named `SourceLocation` nodes reify the reported item
  name and retain its actual source position. An unresolved re-export is a
  declaration observation, not a resolved definition. No source digest or test
  execution is invented. These adapters are not yet wired into cargo-workplan's
  command renderer; both old command serialization routes still require migration.

The public contract, exact identity law and class/property registry are in
`docs/capabilities/rdf-report-profile.md`; census's domain shapes remain in the
normal-form architecture. Registry tests compare documentation and code in both
directions and check emitted application IRIs. An additional owning-module unit
test exercises all four typed surface-failure variants, including `failurePath`;
it tests projection, not forced backend resource exhaustion. Crate inventories,
canonical examples, designed census consumers and licensing policy were updated.
The Unicode exception is scoped to audited `purrdf-iri@=3.0.1` in root and fuzz
policies; packaged IDNA-table provenance, notices and permission text were read.

## Evidence

| Scope | Log under `raw/` | Result |
|---|---|---|
| Actual workspace callers, all targets | `queue-rdf-callers-final.log` | Compile passed; no full suite claim |
| Kernel library/profile/example targets | `queue-rdf-kernel-final.log` | 10 tests passed; example also compiled |
| Census-only all targets | `queue-rdf-census-frozen.log` | All 137 tests passed |
| Final census native/independent pin distinction | `queue-rdf-census-lib-final.log` | 20 library tests passed |
| Final census report tree after source changes | `queue-rdf-reports-final.log` | 25 report tests, including both repository snapshot checks |
| Typed adapters and actual re-export neighbor | `queue-rdf-workplan-final.log` | 5 integration tests passed |
| All typed surface-failure projection variants | `queue-rdf-surface-projection.log` | 1 owning-module unit test passed |
| Three affected libraries, all-target clippy | `queue-rdf-clippy-final.log` | Warnings denied |
| Fuzz compilation/clippy | `queue-rdf-fuzz-check.log`, `queue-rdf-fuzz-clippy-final.log` | Passed |
| Reader fuzz with current semantic seeds | `queue-rdf-reader-fuzz-final.log` | 5,000 runs passed |
| Root/fuzz dependency policies | `queue-rdf-deny.log`, `queue-rdf-fuzz-deny.log` | Both passed |
| Documentation generation | `queue-rdf-docs-sync-final.log` | Passed |
| SPDX/licensing | `queue-rdf-reuse-final.log` | Passed; all files covered, no unused licence |

The census-wide result precedes the final adapter/test-only additions; changed
report and repository snapshot checks were rerun against the final source tree.
The unchanged normalization, grouping, unit and native-collection results remain
applicable. An earlier run compared repository loads during source edits and
failed; `queue-rdf-census-all.log` is historical, not a current success. Its exact
failure evidence is retained. New diagnostics keep the same strict byte equality
while avoiding dumping entire repository reports on failure.

Canonical negative corpus seeds carry valid recomputed snapshot identities so
they reach domain validation. The fuzz self-check asserts an unknown-property
refusal and inconsistent-copy refusal beside valid Turtle/N-Triples neighbors.
Fresh generated corpus files are confined to ignored scratch. Example argument
validation refuses unsupported and extra arguments beside successful `nt` output.

## Independent graph replay

The isolated helper restored from the archived census delivery lives at
`scratch/rdfprobe/oracle`; its source and locked manifest are retained without
external build products. It uses `purrdf-rdf` and `purrdf-core` exactly 3.0.1,
parses `.ttl` as `text/turtle` and `.nt` as `application/n-triples`, with no caller
BASE, and emits sorted ground N-Triples. The retained replay command is
`raw/queue-rdf-oracle-command.txt`. `queue-rdf-oracle-final.log` records
both formats of eight graphs: kernel 7 triples, census baseline 41, index 89,
awkward names 11, workplan findings 11, code coverage 45, test coverage 28 and
surface errors 22. Each format pair independently produces identical triples.

Native `*.nt` goldens are distinct from `*.oracle.nt` independent graph pins.
The one documented fixture comparison adjustment replaces the actual U+0085
scalar with its escaped spelling because the independent serializer escapes it
while the shared canonical literal carrier writes it verbatim. Literal text
containing a backslash followed by `u0085` remains a separate tested neighbor.
Production canonical acceptance is unchanged and exact oracle bytes are retained.

## Remaining acceptance

Primary substantive review is pending. The fresh profile lifecycle, current
assessments and required full gate are not claimed complete. All workplan command
variants must still converge on typed RDF, remove JSON routes/dependencies and
propagate diagnostic I/O failures; sole default policy and actual zero-debt RAF
adoption, including the new public crate, remain in the same 40 + 16 + 37 landing unit. Census debt runtime and its
CLI/protocol consumers retain their separate acceptance owners. Native collection
does not establish the outstanding portable/WASM acceptance. No scope waiver or
baseline-as-zero adoption is implied.
