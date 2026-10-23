<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Census foundation source findings

## Correction disposition

Findings 1–3 below are resolved by source inspection of the owner's current
corrections. No builds/tests were run for this disposition. The parent reports
nine Gate and ten debt cases passing; that runtime receipt is not independently
reexecuted here and is not full issue completion.

- Finding 1: `gate/owners.rs` indexes actual units once by `(file,start_byte)`.
  Resolved Source sites map through that index and deduplicate physical reaches;
  the returned owners retain all distinct cfg sites. Homes/entries and variants
  classify physical UnitKeys. Variant admission filters the declared file and
  individually requires the declared detector to match. Rule hits retain the
  containing-span check and normalize to the canonical owner's symbol.
- Finding 2: `DebtGroup::with_counts` retains positive per-symbol multiplicities,
  checks total overflow and requires at least two total members. Ordinary `new`
  still rejects repeated symbols and constructs count-one entries. Comparison
  uses count containment without losing exact-row absence checks. A count-one
  group has the original node name and assertions; no source line or offset
  enters ordinary debt identity. Lowering 2 to 1 is a strict counted subset.
- Finding 3: rules use the shared physical-owner index and build the file-context
  index once. Per-hit ownership now visits only resolved candidate sites rather
  than all Census units. This removes the reported O(U*H) lookup scan, without a
  measured claim about the rule or Source resolver's own costs.

Count codec/sanction boundary: debt emits MemberMultiplicity nodes only above
one. Closed shapes require their class, string symbol and count of at least two;
the debt reader additionally checks group-owned canonical node names, membership,
one count per symbol, host overflow and total cardinality. Orphan multiplicity
nodes are refused. The count delimiter `@` cannot alias an ordinary symbol byte
because the existing percent encoder encodes that byte. The counted node and
ordinary member-symbol identity therefore remain distinct.

Distinct ledger counts are declared, not inferred as sanctions from current
source. Each resolved physical cardinality must equal its declaration; aliases
of one physical owner cannot supply extra members. Only a fully valid row becomes
a sanctioned counted group. The cfg test retains the 2-member positive and a
new third member refusal, preventing unchanged sanctions from silently growing.
The native reader also refuses absent-symbol counts, counts below two, repeated
count ownership and unknown fields. No new concrete codec or sanction defect was
found in this delta.

Debt intentionally keys counted-symbol duplication relations. Separate native
fingerprint groups with the same projected relation coalesce; this is not an
injective archive of physical occurrences. The parent owns explicit capability
documentation of that choice. No new body fingerprint/normal-form behavior is
inferred from the debt representation. The original findings follow for
provenance; they no longer describe the corrected source.

Read-only review of Gate evaluation/rule ownership/debt publication, native ledger
decoding, Rule admission and shared JobReport collection. Reused the resolved
debt review; did not reopen its comparison defect. No edits, builds, tests or
subagents. Parent-reported 19 focused cases and standalone disk writer success
do not cover the identities below. Patterns, built-in detector implementations,
reference scan, import and full delivery are explicitly unfinished, not findings
against this bounded foundation.

## 1. Resolved path text is not Census physical ownership

`gate/evaluate.rs::resolve` returns `Workspace::resolve(...).path`, then home,
entry and variant classification compares that string with `Unit.symbol`.
`gate/rules.rs:63–89` additionally requires `unit.symbol == resolved.path` even
after obtaining the owner's actual source spans.

Those paths have different canonicalization contracts. `units/mod.rs:176–193`
collapses all reaches of one physical source item and chooses a shipping-first,
then lexically smallest path. Source resolution preserves the particular
defining module reach. For a library declaring the same `shared.rs` as both
modules `a` and `b`, one physical `f` becomes Census unit `demo::a::f`; a home
`demo::b::f`, or a reexport of it, resolves successfully but is classified Copy.
A variant on that valid reach is reported absent, a distinct row retains the
wrong symbol, and a legitimate rule hit owned by that reach is hard-refused.
The existing ordinary reexport fixture targets just one defining reach and does
not exercise this difference.

Map actual resolved sites through `(file, span.start_byte)` Census ownership,
then use its canonical unit identity. Retain all genuine cfg alternatives and
the declared variant file restriction; do not select an arbitrary first site
or widen a file-specific sanction. Rules should normalize hit.symbol to the
actual canonical owner, rather than the independent Source path spelling.

## 2. Valid repeated-symbol groups cannot become debt

`gate/evaluate.rs` constructs every DebtGroup from `group.member_symbols()`.
`DebtGroup::new` refuses repeated symbols. Census deliberately retains distinct
cfg-alternative units of one symbol (`census.rs:116–125`), while
`group::build_groups` groups physical units by fingerprint and preserves their
file/start identities.

Two cfg alternatives of `demo::f` with equal admitted bodies therefore form a
valid two-member Census group whose symbol iterator repeats `demo::f`.
Gate evaluation returns an error when projecting it, before either checking or
adopting debt. This is a concrete mismatch between the existing normal-form
producer and the new debt consumer, rather than malformed ledger input.

Define a compatible exact debt member identity for this producer case and carry
it consistently through comparison/codec/sanctions. If a symbol-set projection
is intended instead, specify and validate its semantics explicitly; merely
deduplicating the two symbols gives an invalid one-member group, and omitting
the group silently loses real Census output. Do not change body fingerprints
or discard cfg alternatives incidentally to repair the projection.

## 3. Rule ownership adds a whole-workspace scan per hit

`gate/rules.rs:70–78` filters every Census unit for each emitted RuleHit.
For U units and H legitimate hits this performs O(U*H) unit inspections, even
when every hit belongs to one distinct function in one file. A rule reporting
one hit per unit makes this quadratic independently of parser or rule cost.
`census.file` is also a linear lookup once per source file.

Build one borrowed physical-owner index for this evaluation and use the actual
resolved source keys from finding 1 to query it. Reuse the source file context
index or ordered lookup rather than repeating linear searches. This can repair
ownership and remove the new quadratic scan together without a competing graph
algorithm or changes to native parsing/admission. No measured runtime or new
performance threshold is claimed by this source-derived cost finding.
