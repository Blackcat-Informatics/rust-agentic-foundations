<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native OKF section assessment review

Read the complete two fresh CLI section contexts and current packet source,
tests, docs/config/dependency changes. Verdicts below are independent judgments
of each exact context. No assessment was published, no product was modified,
and no builds were run. Mandatory full gates and lifecycle remain outstanding.

## Contract: accepted

Context: raw/native-okf-contract-context.txt.
Section digest: `01e83b69dd71a64af8cc9a7fdee3be0a8c8b41537b146b877dabc3a543638375`.

The complete Contract remains consistent with the current selected packet.
Its six document types, closed key/value/schema rules, document/body/writer/
index behavior and record serialization remain unchanged; corresponding
previously inspected evidence is reused rather than inferred from this packet's
87-test subset. No strict-policy Debt kind or schema removal entered this
checkout. No new mismatch was found in those unchanged clauses.

Changed statements are supported by actual paths and assertions: ESCAPES is
the shared six-member reader/writer table backed by native ECHAR for its five
overlapping values, with RAF NUL extension and native-only escape refusals.
Numeric quoting retains the previous accepted aliases under native numeric
interpretation. Number keeps authored text equality while retained native
Decimal supplies compare/exact/value; the signed-zero adapter and exact
padding/long-value/precision neighbors support the stated distinction.
Canonical hex uses native readers under the same 40/64 and lowercase rules.

The timestamp clause now accurately states the actual phased profile adapter:
fixed date/clock/nonempty fraction grammar before native calendar/clock range
validation, then complete offset grammar/ranges, including leap-second policy
precedence. The exact new 17-refusal/five-valid fixture exercises the reported
two-defect boundaries and neighboring valid cases. No duplicate date math or
second native parse is introduced.

This accepts the section's meaning for the current implementation; it is not
a claim that every mandatory gate or every pre-existing test was rerun here.

## Verification: rejected

Context: raw/native-okf-verification-context.txt.
Section digest: `b29190a869b7a268ca4f554934f22fbe3e3921f0dfbae49703f10701e1e4bea4`.

One concrete clause mismatch prevents acceptance of this exact complete
section. The unchanged alias bullet claims a bound on what the process
allocates. Current alloc_bound uses CurrentThreadWindow and asserts only the
synchronous parse thread's peak working bytes, excluding sibling test/harness
activity; its former Cap<System> wrapper had no meaningful finite cap, and
the new target has no process cap. The section's new opening allocation
bullet correctly distinguishes current-thread measurement and the separate
heading fixture's hard process ceiling, so the later process-bound wording
also conflicts with that accurate statement.

Required repair: retain the alias refusal and state its actual measured
calling-thread bound, eight times input plus 16 KiB, with the synchronous
scope explicit. Keep the heading target's distinct 1 GiB whole-process Cap.
This is a precision correction, not permission to remove the quantitative
bound. Obtain a fresh Verification context after changing its bytes; this
rejected digest must not be published as accepted.

Other Verification additions match inspected actual assertions: closed escape
refusal positions, constructor styles/roundtrip, exact Number comparisons and
negative zero, phased timestamp cases, unchanged deterministic MMIX stream,
and native calling-thread allocation windows with the original allowances.
Unchanged parse/body/profile/write/cost/marker evidence remains reusable;
this defect does not require discarding unrelated valid evidence or repeating
their entire review.

## Packet docs, dependencies and configuration

The packet's change record names only Contract/Verification and its selected
native primitives. Capability bindings add the actual Number::exact and
timestamp owners. AGENTS correctly removes the obsolete local unsafe counting
allocator description. No local unsafe implementation or allow was added.

Production closure is pinned git hash/lex/xsd; dev adds alloc-probe and keeps
cap. Existing cap's stats feature is removed because native windows replace
that measurement, while `Cap<CountingAllocator>` still enforces the original
`1 << 30` heading-process ceiling. 64 KiB heading allowance, four-times-heading
coefficient, and alias eight-times-input plus 16 KiB remain unchanged.

Root/fuzz allow only the named PurRDF git source and require rev; unknown git
and registry remain denied. The lex license exception is scoped to version
3.0.1 and adds BSD-3-Clause beside Unicode-3.0. Native manifest declares those
terms and retained package NOTICE/WHATWG notices identify the corresponding
data. This is concrete dependency licensing support, not an executed deny or
REUSE gate. Root/fuzz configurations retain their existing crate-scoped fuzz
exception and mirrored common changes.

Observed dependency concern requiring actual gate disposition: both locks
currently hold registry and git package identities for purrdf-hash and
purrdf-lex 3.0.1, since Census still uses registry. No duplicate-package waiver
was added. This review did not execute cargo-deny and does not assert its
multiple-versions verdict; reconcile these identities or collect the required
gate result rather than infer a pass from matching version numbers.

Inspected native-okf-callers-current.log ending in successful all-target
dev-profile completion, and native-okf-focused.log terminal target results:
1 allocation-refusal, 1 deterministic body corpus, 1 heading bound, 26 records,
29 refusals, 22 render and 7 schema tests, all zero failures (87 total).
These are actual packet receipts; doctest/full gate/lifecycle acceptance is
not implied by them.

## Repaired Verification context: accepted

Context: raw/native-okf-verification-context-current.txt.
Section digest: `f33b088a6293ef2f9d5775aecc1346a6d6c1ed23ef778019fa451e88160e110b`.

Read the fresh complete section. Its alias item now states the actual
synchronous calling-thread peak bound, eight times input plus 16 KiB. This
closes the only concrete mismatch in the earlier context. The separate opening
item still accurately identifies the heading process's 1 GiB Cap. All other
section clauses/symbol inputs are unchanged and their applicable evidence is
reused. Accept this repaired digest; the previous rejected digest remains
rejected historical evidence.

The current allocator assertions additionally require nonzero measured growth
for both actual refusal paths and both heading parses. These are meaningful
probe-registration controls: an unregistered native allocator would no longer
satisfy the upper bound by reporting zero. The numerical upper allowances and
hard Cap are unchanged. Current stronger-test execution remains qualification
work for the primary; earlier 87 results predate these nonzero additions.

## Shared native identity reconciliation: source review

Necessary expanded packet scope: Census's existing native lexical dependency
must use the same workspace pin, and its external BLAKE3/hash formatting must
follow that same native hash home. Manifest now selects purrdf-hash.workspace
and purrdf-lex.workspace and removes external blake3. This is coherent with
the provider identity concern, without importing strict policy or changing
normal-form semantics.

Inspected normalize/fingerprint.rs: the input stream remains each token's
UTF-8 byte length as u32 little endian followed by those exact token bytes,
in the same token order. ENCODING remains
tokens-len-u32le-utf8/blake3/hex64. Native streaming Hasher returns the same
32-byte algorithm identity; Display uses full lowercase native hex, while
abbreviated uses only the first eight bytes plus the existing ellipsis and
never replaces complete identity. FromStr retains width-first refusal, then
native exact lowercase decoding, and unchanged kind/table/rule grammar.
No normal-form version change is justified when existing full golden bytes
remain identical; they must actually execute rather than update to new output.

Actual assertion owners inspected: forms::the_digest_is_blake3_of_length_prefixed_tokens
builds hand-framed bytes and independently pins the complete b3sum identity;
a_fingerprint_displays_and_parses_back checks full exact roundtrips;
human_abbreviations_do_not_replace_complete_identity distinguishes same-prefix
but different full values and refuses abbreviated spellings. The golden owner
compares every computed fixture identity to unchanged full literal pins and
refuses moved/orphan/unpinned/duplicate rows. These preserve actual semantic
identity, beyond simply comparing two new native calls to one another.

Concrete incomplete-caller finding sent to the primary: removal of external
blake3 leaves actual references in report/write.rs (fingerprint IRI and body
digest), tests/forms.rs, tests/reports.rs and examples/selection.rs (corpus and
refusal IDs). Native Hash exposes as_bytes, not external Hash::from_bytes,
to_hex or Display; use the native hex writer/Lower over those bytes without
altering report body normalization, namespace, normal-form frame or goldens.
This source checkpoint is not yet a complete independently compilable Census
cutover until those actual callers are repaired. No compile was run here.
The added source scope does not revoke either OKF section judgment: their
current authored meaning is independent, while packet delivery still requires
the expanded actual caller/golden/dependency gates.

## Repaired Census callers: bounded recheck

Inspected the actual repair in report/write.rs, forms.rs, reports.rs and
examples/selection.rs. The external-call compilation gap is closed in source.
Report fingerprint IRIs retain the same namespace plus all 32 bytes in native
lowercase hex. Report digest still hashes exactly body.as_bytes(), with no
change to which normalized body is supplied or how the BASE line is omitted
by its assertion. Selection hashes the same corpus label/input label/refusal
label bytes, and every surrounding IRI/literal/output delimiter is unchanged.
Native Lower is consumed during formatting or to_string before its temporary
hash owner dies; no borrowed digest escapes.

Both forms expectations now use native digest/hex while retaining the complete
independent b3sum literal. No golden literal, framing constant, normalizer pass,
normal-form version or RDF report-model/schema change accompanies these edits.
The actual updated locks unify hash/lex to one pinned git source and remove
the external BLAKE3 dependency closure. This closes the earlier provider-
identity source concern; actual locked build/deny and semantic goldens remain
required execution evidence. No concrete source defect found in this narrowed
repair. The primary's workspace caller build was active when reviewed; this
reviewer did not execute it or infer a pass.

## Census Contract: accepted

Context: raw/native-census-contract-context.txt.
Section digest: `1513b93e5498dc24c1ab8ee7fbfef1206c0385dfe51112b3646b183e8931b2ba`.

Read the complete section and its changed source callers. The native hash/hex
and shared lexical identity statements are accurate. The same full 32-byte
BLAKE3 digest, little-endian u32 UTF-8 token framing, canonical lowercase
representation and complete fingerprint identity remain authoritative; human
abbreviation remains non-identity. Report BASE hashing retains exactly the
same normalized body bytes, and no normal-form/frame/report-model/schema
change entered the packet. Hash has no external runtime dependency or C build.

Unchanged normalization/threshold/binder/shim, unit collection, grouping/job,
error/lifetime and closed report-shape assertions retain their prior evidence;
this native packet does not change those rules. No newly contradictory clause
was found. Existing mandatory versioning policy remains intact: the unchanged
golden identities, not a new normal-form number or regenerated pins, establish
this implementation substitution's compatibility.

## Census Verification: rejected

Context: raw/native-census-verification-context.txt.
Section digest: `783b251f5edc9a76e6aa4325c9c6372e030e41a0ab18fd2f803e6d58fbd3e618`.

One specific new claim overstates actual execution: the opening paragraph
includes selection identities among the outputs exercised by the listed
caller fixtures. The selected example test actually calls Body/Policy::key
and groups, compares production prints and pins 261/3900/22/54. It never
calls summarize or main, where corpus/input/refusal hash identities are
emitted. Native source substitution appears output-preserving, but this is
source-inspected identity preservation rather than an executed selection
identity caller assertion. Compilation of the example is not that execution.

Repair either the exact evidence claim or the assertion: exercise actual
summarize output with an independent fixed corpus-id pin and meaningful
selection output assertions; actual input/refusal ID callers need their own
execution if claiming those too. Alternatively distinguish the executed
selection-count law from source-inspected identity preservation without
claiming those output paths executed. Do not weaken the count assertions or
replace the normal-form goldens with expectations computed by the new kernel.
Obtain a fresh context after any prose change. This exact digest is not
accepted for publication.

The remaining complete Verification content is consistent with its unchanged
actual evidence: deterministic/lifetime/error producers, fixed registry and
normal-form pins, forms/groups/shim boundaries, complete identity/refusal
fixtures and report shape/canonical neighbors. Those are reusable and need
not be discarded because of the one new selection-execution mismatch.

Inspected raw/native-census-callers-goldens.log: 22 units, 27 forms, 4 immutable
goldens, 23 groups, 7 page, 25 reports, 30 units integration and 1 selection
example test all passed, zero failures (139 actual tests). Baseline/goldens
example targets have zero tests and count as compilation only. Owner reports
the full workspace caller build and strengthened nonzero allocator reruns
passed; this reviewer did not run them. Full gates/lifecycle remain separate.

## Census Verification rejudgment with actual output evidence: accepted

Same unchanged context and section digest:
`783b251f5edc9a76e6aa4325c9c6372e030e41a0ab18fd2f803e6d58fbd3e618`.

The earlier rejection identified missing execution evidence, not incorrect
native selection output. The primary has now supplied actual example output
covering every changed selection identity caller. Read
raw/native-selection-identities.ttl and stderr, the independent
raw/selection-id-reference.txt, and the positive-only neighbor in
raw/native-selection-valid.ttl/stderr.

The labelled fixture Corpus id matches the independent b3sum of the exact
no-newline labelled-fixtures bytes. The actual accepted disk corpus emits
both Input and Corpus records with the independently pinned accepted-label
digest, one body and six tokens. The missing disk corpus emits the Refusal
record with the independently pinned refused-label digest and an actual
missing-path reason. The mixed execution reports terminal exit 1 and expected
aggregate refusal stderr; the accepted-only execution reports terminal exit
0, has no Refusal record and empty stderr. Exit receipts are retained in the
reference note and reported by the primary; this reviewer did not rerun them.

This executes summarize and main's accepted/refused identity emitters in
addition to the existing real selection-count test. The independent references
are from /usr/bin/b3sum, not recomputed expectations from the new native kernel.
The specific evidence gap is therefore closed and the same complete section
is now accepted. Earlier rejection remains an accurate record of the evidence
then available; it is superseded by this evidence-backed verdict, without
changing the prose or weakening its meaning. Other unchanged evidence is
reused. Mandatory full gates and lifecycle still remain outstanding.
