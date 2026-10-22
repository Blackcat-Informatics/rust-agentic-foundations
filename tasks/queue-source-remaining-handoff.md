<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Remaining source claim ownership before integration

This is the issue 16 branch's measured ownership map. No source Rust or
source-index capability edits accompany this handoff. The last completed
candidate is `raw/queue-adoption-reconcile-complete-candidate.log` (exit 0),
with exact findings in `raw/purrdf-adoption-reconcile-complete/findings.txt`:
28 source-index uncovered claims, 9 coverage uncovered claims, zero code/test
gaps and zero orphan, excepted or debt lines. Later focused coverage additions
do not establish a new candidate count. All 59 stale or unaccepted assessments
remain open.

## Nine source Contract groups still uncovered

- Disk lookup failure boundary.
- Path containment.
- Resource bounds.
- Work and memory bounds.
- Mod-tree following.
- cfg(test) tagging.
- Use-alias index.
- Resolution from every target.
- API surface.

`tasks/queue-source-index-claim-map.md` contains assertion-inspected owners and
explicit candidate-only families. Its appended Tree/discovery supplement and
`tasks/queue-source-discovery-review.md` distinguish already repaired
membership/target/order proofs from these remaining compound groups. These need
meaningful per-guarantee splits, not a marker naming a whole suite. Distinguish
actual Linux execution, strategy overrides and native Windows runtime.
Historical allocation measurements do not prove universal bounds or arbitrary
future parser behavior.

## Nineteen remaining Constraints

The candidate's exact statement anchors are:

1. Syntactic parsing, without macro expansion or type evaluation.
2. Opened paths stay inside the canonical workspace root.
3. Per-file size/token/nesting and accumulated load refusals.
4. Settling queue and undecided-import graph memory are currently unbudgeted.
5. Settling lookups are currently outside the work budget.
6. Source-defined resolution, not dependency/macro-generated items.
7. Token reachability approximation and same-name local overapproximation.
8. Self member handling and self-type limitation.
9. Edition 2015 body-use shadow behavior.
10. Impl blocks inside const/block initializers are not indexed there.
11. Narrow handling of syn verbatim forms.
12. Existing cyclic glob/import divergences.
13. Compiler order-sensitive glob ambiguities.
14. Superlinear settling cost.
15. Input-derived error text escapes controls.
16. Reexported syn version ownership.
17. Path-sorted deterministic output.
18. Public surface visibility and its explicit source limitations.
19. Actual Windows-host proof requirement.

The old unbudgeted queue/work and cyclic-divergence statements are known
integration overlap: the parent owns replacement source behavior, actual native
memory/admission tests and original performance acceptance. Do not copy sibling
test names here before their source exists. Native spill/admission and
selected-route Storage contracts require integration preserving both contracts,
then contexts/assessments against the resulting source. The source lane's current
ambiguous-hub count 25,276,275,336 remains failing acceptance evidence.

Lexical reachability/format-capture work belongs to this branch, separate from
the sibling resolver algorithm. Its proof must retain standard-macro identity,
opaque-input, shadow, decode/work and retained-memory boundaries. This artifact
does not authorize a source edit while the sibling source is frozen.

## Exact lexical format donor inventory

The untracked implementation is `crates/raf-source/src/reach/format.rs`,
`crates/raf-source/src/reach/format/arguments.rs` and
`crates/raf-source/src/reach/format/identity.rs`; its integration fixture is
`crates/raf-source/tests/format_reach.rs`. Preserve all four files. The tracked
`reach.rs` declares the private module, stores lexical candidates and counted
decode work in its file/edition cache, and creates the context resolver during
the actual reachability walk. `work.rs` supplies `ReachFormatTokens` and
`ReachFormatBytes`. The implementation consumes existing `proc_macro2` and
`syn` parsing, `prescan`, `facts::AttrFact`, private index/use/namespace APIs,
`resolve::Heads`, `SourceFile`, `Budget` and `LoadError`; it adds no separate
external dependency. Current source manifests also contain pinned native core
and lex production dependencies and native XSD for development callers. Those
native dependencies serve other source donors and must not be inferred to be
format-only requirements. All source files remain frozen pending the primary's
deliberate integration and qualification.

The primary's native traversal audit identified one further supplied seam:
`reach/format/identity.rs::Resolver::new` uses `Vec` plus `pending.pop()` for
LIFO traversal; pinned `purrdf_lex::walk::WorkList` supplies that operation.
Its original ItemBytes admission/error propagation must be integrated with the
native stack rather than silently lost. The continuation frames in `format.rs`
use `last_mut()` and `pop()` together; native WorkList has no peek/last_mut API,
so they are not a direct replacement. A coherent explicit current-frame design
would need separate justification. FIFO VecDeque and indexed child/readers/scope
storage have different semantics and are not LIFO duplication. These are exact
remaining integration boundaries, not a claim that the format donor already
uses native traversal. No source edits accompany this inventory.

Other explicitly owned donor reconciliation remains with the primary source
integration: its native testkit temporary-directory/text/shape owners must retain
the exact shared dependency pin, and its shared native unsigned helper for
oracle/program tags, answer dumping and equivalence should replace this branch's
`registry_scan.rs::unsigned<T>` only after comparing accepted sign, zero padding,
digit grammar and checked-width behavior. Preserve the current qualified
registry source until that actual helper is available; do not invent a second
public generic scalar API. This is a named primary integration responsibility,
not a native-reuse exemption or a source edit in the frozen checkout.
