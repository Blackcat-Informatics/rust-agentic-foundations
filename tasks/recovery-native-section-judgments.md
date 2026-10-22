<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Recovery native section judgments

Independent read-only prose/source judgments for the production contexts captured under `raw/queue-context-*.txt`. No assessment records were published, product files edited or builds performed by this reviewer. Acceptance below means agreement with the stated contract and inspected assertion owners; it is not current whole-tree runtime, lifecycle or full-gate acceptance.

## RDF report profile

| Section | Judgment | Basis |
| --- | --- | --- |
| Purpose | accepted | Actual native dataset, mutable membership, parser, indexed queries, terms, traversal and output implement the named ownership. Census and Workplan retain domain validation. |
| Contract | accepted | The structured storage, term, ownership, identity and format clauses match the landed authoritative native implementation. Builder finish and physical overlap refusal remain real; no old Nodes/Properties graph was restored. Named Workplan and CLI projection owners preserve typed fields, absent locations, order, source provenance, outcome and unresolved-reason distinctions. Workplan reportForm 1 remains explicitly separate from Census form 2. |
| Usage | accepted | Actual Builder insertion/finish, immutable Graph render/read/snapshot/dataset and borrowed subject/value methods match. Shared finding append now uses Builder; production callers compile and execute through the shared projection. |
| Examples | accepted | Current example retains native Builder and implements default Turtle, `nt`, typed/control/Unicode values and named provenance. Baseline/Index expose all four named read/write methods. Its per-format read/render assertion is actual code, not a claim that a zero-test example target executed. |
| Constraints | accepted | Actual boundary tests cover closed syntax/depth, domain nonvalidation, 64 KiB literal and post-parse exact rejection. Native physical order is explicitly distinguished from output order. No input/allocation/stack ceiling or authentication is claimed. |
| Explanation | accepted | Parser-before-profile, resident allocation and normalized body/hash ownership are accurately limited. The owned Graph wrapper retains native storage, and operation-local selectors do not reinstate a generic graph. This is rationale, not an unexecuted universal allocation bound. |
| Verification, original captured context | rejected | `queue-context-rdf-report-profile-verification.txt` says the parser expansions are independent. Actual pins are another revision of the PurRDF parser family, as already established in the landed review. This original context must not receive an accepted record. |
| Verification, corrected on-disk section | accepted | Re-read the actual correction at lines 354–361: it now says a separately pinned revision of the PurRDF parser family. The rest of the profile/projection/normal-form assertion summary agrees with the inspected tests. Capture this corrected text before publishing an accepted assessment; the original context above remains rejected. |

## Census normal form

| Section | Judgment | Basis |
| --- | --- | --- |
| Purpose | accepted | Supplied Tree, deterministic selected form, complete fingerprint and explicit normal-form gate agree with the core. Collection threads are not claimed absent; filesystem/process discovery remains outside the supplied-source core. |
| Contract | accepted | Existing normalization/threshold/shim/unit/group/job guarantees retain their actual owner assertions. Native hashing retains length-prefixed NF1 bytes. The restored native report storage paragraph is accurate; reportForm 2 and declared-normal-form preservation are distinct and backed by refusal/valid neighbors. Worker/error wording preserves hard failure rather than partial output. |
| Examples | accepted | Full fingerprint versus human abbreviation, built-in baseline/index output and shared forms/selection examples remain supported by the inspected source and immutable page/report/golden owners. No candidate selection policy is advertised as a production fingerprint change. |
| Verification | accepted | Native hash/scalar/report claims agree with retained framing, full-suffix and domain controls. Caller span/worker TLS/panic observations remain explicitly lifetime observations, not RSS/allocation measurements. Parser pins are correctly identified as another revision of the same family. NF1 goldens and normal-form migration law are not weakened. |

## OKF profile

| Section | Judgment | Basis |
| --- | --- | --- |
| Contract | accepted | Current closed seven-type schema, removed policy selector, debt/coverage keys, positioned validation, markers, body edits and deterministic writer match the retained implementation and specific owner assertions. Native ECHAR/float/Decimal/hex/RFC3339 delegation keeps the closed authored grammar and timestamp diagnostic precedence adapters. No broad native YAML parser substitution is claimed. |
| Verification | accepted | Native scalar neighbors retain grammar, values, exact Decimal comparison and timestamp precedence. Actual allocation fixtures use synchronous current-thread windows, nonzero observation checks, unchanged allowances and a separate hard 1 GiB heading-process cap. Compiler registration is correctly described as an attribute root rather than a fake explicit call. Existing marker, closed-schema, body/resource and writer assertions remain meaningful. |

## Evidence and remaining qualification

Reused the substantive native storage review and landed PR 54/55 primitive/report receipts only where source and contract inputs remain unchanged. The recovered tree has fresh actual execution in `recovery-native-profile-runtime.log` (14 profile tests), `recovery-workplan-projection-runtime.log` (four actual report projections), and `recovery-cli-projection-runtime.log` (five RDF callers and the actual registry producer). These support the changed native API adapters and newly retained public boundary controls. All-target check logs are compilation, not execution. Historical failed logs are not passing evidence.

Execution update: inspected `raw/recovery-census-okf-runtime.log` after the primary collected terminal exit 0 from session 78705. All 32 suite summaries pass: 422 tests, zero failures or ignored tests, including doctests. This closes the fresh recovered-tree Census/OKF execution gap above. The preceding failed run exposed stale exhaustive Rule positions/count after removal of the opt-in rule; it is not counted as passing evidence. The primary audited the exhaustive mappings and repaired positions 59–62 and count 63 before this complete rerun. No further code review or broader gate acceptance is inferred from that correction.

Corrected RDF Verification remains accepted for the re-read on-disk text, pending its exact refreshed context capture/publication. Real lifecycle obligations, required full gates and native-host/CI coverage remain the primary's acceptance work. No unrun host or full-gate result is claimed. Document-model and SHA256 prose repairs belong to the primary's separate judgments and are not adjudicated by this artifact.

## Missing-subject judgments

Read the seven exact production captures in `raw/recovery-missing-contexts` after they became available. These are new individual judgments, not inferred from currency of the earlier 75 records.

| Subject | Judgment | Basis or concrete mismatch |
| --- | --- | --- |
| census-debt-ratchet#explanation | accepted as design rationale | The capability is designed with empty implementation bindings. The shared ratchet rationale explains why both growth and stale shrinkage refuse and why creation/version adoption remains review-sensitive. The native content-addressed BASE supports the intended visible content identity. This does not claim a delivered Census debt reader/writer, command, or executed ratchet suite. |
| okf-profile#purpose | accepted | Shared parse/profile/body/render owners are the actual document boundary of Workplan and its tooling. This describes common interpretation, not a new filesystem or subprocess API. |
| okf-profile#usage | accepted | Current Document parse/render, Profile typed Architecture validation and body span replacement match the shown string-only calls. The named EvidenceRecord::to_document example constructs a document before serialization and executes among the 13 passing OKF doctests in the fresh runtime receipt. The generated-block indexing illustration requires a document that actually has a generated block; it is not an invocation on the preceding block-free Architecture string. |
| okf-profile#examples | accepted | The shown designed Capability uses the closed current keys and, with the stated fences/sections/path, satisfies the parser/profile. Actual valid-type, required-key and resource-key tests assert the accepted and refused neighbors with the named key/rule. |
| okf-profile#constraints | accepted | Text-only boundary, exact byte/depth/link caps and alias refusal, ATX/fenced-code interpretation, no extra keys/warning level, preserved comments/body bytes and prose-tense nonvalidation match current implementation and specific owner assertions. The 422-test recovered-tree receipt includes these actual families; it is not a universal native-host/full-gate claim. |
| okf-profile#explanation | rejected | The captured claim that every key is required except tags/timestamp contradicts current `profile/schema.rs`: command/revision and ceilings/exceptions are explicitly optional, among others. Retain the closed-schema rationale but say the per-type tables distinguish required and optional keys, with tags/timestamp optional base keys. The native YAML metadata/ECHAR explanation otherwise agrees with the inspected boundary; it does not excuse the universal-key error. |
| source-index#purpose | rejected | The captured present-tense assertion that Census resolves ledger homes with this crate implies a delivered consumer that does not yet exist: Census ledger remains designed, while current Job receives a home String and compares it to unit symbols. The actual shared source use is Census workspace/unit indexing and Workplan code/test binding resolution. State that current use, or explicitly distinguish intended ledger-home resolution, rather than claim the designed consumer is delivered. The shared walker/index rationale is sound but does not itself prove unlimited consumer agreement. |

These two rejected exact contexts require corrected prose and exact recapture before accepted publication. No implementation change, extra review panel or invented Census ratchet execution is required by these findings.

### Corrected missing-subject recheck

Re-read the actual corrected sections after the primary's repair:

- `okf-profile#explanation`: **accepted**. It now distinguishes required and optional keys in the per-type tables, identifies tags/timestamp as optional common keys and retains other explicitly optional fields. This closes the schema contradiction without weakening the missing-required-key or unknown-key rules. The remaining native YAML metadata/ECHAR and deterministic body/writer rationale is unchanged and agrees with the inspected source.
- `source-index#purpose`: **accepted**. It now names actual Census source-unit loading and Workplan binding resolution, scoped to the shared interpretation of a given workspace and target context. The unsupported present-tense ledger-home assertion and unlimited consumer-agreement implication are removed.

The original captures in `raw/recovery-missing-contexts` remain rejected historical inputs. Accepted publication must use the exact corrected captures planned under `raw/recovery-reviewed-contexts`. These source/prose-only fixes do not imply execution of a native-host or full gate. The primary's new 113-assessment/116-record inventory is a separate completeness check, not substituted evidence for any judgment above.
