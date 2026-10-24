<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Configured provider backend review

Independent source review against this stage's bounded issue contract. Read the
transferred Census backend, Gate integration, typed source ownership, existing
provider controls and new configured_provider_targets fixture. Source prerequisite
APIs were checked in the Source evidence checkout because they are not integrated
here yet. No reviewer builds, probes, product changes or forge actions.

Verdict: bounded production and corrected target-fixture source PASS. All three
concrete fixture findings are closed by the inspected corrections. Publication and
delivery completion are not established by this review.

## Production paths

- Gate::check_locked reads the native ledger, loads its complete first-party
  roots with all exact local/source associations, and uses the same Workspace
  identities for Census and provider proof. External source remains available
  for lookup without supplying first-party units or registered-rule subjects.
  Typed RelPath/authority ownership is retained through unit/subject/home lookup.
- routes::allowed consumes actual borrowed Source library receipts. Renames,
  custom library names and lexical extern roots preserve public defining sites
  and original ambiguity. A selected API must name one configured supported
  Fn/InherentMethod with a body; inherent applicability goes through Source's
  explicit restricted proof. Missing/ambiguous paths retain CodeResolveError
  through ProviderPath rather than being converted to empty matches.
- flow::check tracks admitted input bindings and actual native returned values.
  Discarded/replaced native results and local-only returns refuse. Mutable state
  must originate from a declared native call and be an exact mutable binding;
  required mutation-event lineage must be present in the returned native value.
  Updating dead state or returning a pre-update snapshot therefore cannot pass
  merely because a native function appeared in the body. Native event snapshots
  clone only allocation-free key/payload types under native ownership APIs.
- The finite grammar refuses opaque control flow, Try/MethodCall propagation,
  arbitrary validation helpers, receiver inference, async and competing local
  computation with the owning file/line and job context. Generic syntax is still
  visited for executable/opaque computation even when name lookup uses identifier
  paths. Pure generic argument controls do not imply general substitution proof.
- consumer_sites expands each declaration to every same_source ItemId. Work is
  performed per actual target/configuration and physical reach. Configured syntax
  uses the shared bounded parser/header filter; the observer requires every
  expected consumer and API to be seen. Own nominal/alias headers and enclosing
  containers are audited, with a final header-only ancestor pass triggered by
  newly discovered aliases. Own-header observation alone cannot close ancestry.
- api_home requires the already validated declared API spelling to agree on one
  actual definition across consumers. home_sites retains only fully proved
  physical identities, including their complete same_source reach set. Ordinary
  arbitrary missing owners receive no public-API fallback, and module homes do
  not confer a blanket descendant exemption.
- Provider rows remove package-wide detector exemptions consistently from unit
  matching, contextual hits and subject reporting. Selected detector findings
  remain hard findings even for unenforced job rows; rule hits are not accepted
  merely through debt. Existing unlisted-function/struct and decorative-native
  cases cover the relevant bypass. This is enforcement of selected functionality
  detectors, not a claim to solve arbitrary program equivalence or every parent
  detector/default/reference requirement.

No new concrete production defect was found in these bounded paths. This is
independent source judgment, not independent runtime execution.

## New target fixture findings

1. Initial policy() used consumer/home selected::run for every target. Source's
   canonical paths are demo::bin_crate::selected::run and corresponding test_crate,
   bench_crate and example_crate paths. The old spelling would make all positives
   fail before provider proof and render is_err-only negatives vacuous. Root
   corrected the policy using Selected.consumer per actual TargetKind. The
   final source reread closes this finding.
2. Initial shared=true cases used ordinary Lib cfg alongside Bin-test/Test cfg
   on the same physical shared.rs. Source intentionally refuses incompatible
   physical syntax contexts before provider execution. Those cases cannot serve
   as clean dependency neighbors. Root restricted shared Dev-isolation proof
   to syntax-compatible ordinary Bench/Example contexts; separate-file test-mode
   positives remain. Their negative explicitly requires the provider's no
   admitted dependency route reason. The final source reread closes this finding.
3. The ordinary Bin Dev-only negative has no eligible context at all: both its
   Bin and its ordinary Lib companion exclude Dev edges. Source::eligible_targets
   therefore refuses the requested edge before provider proof. Its assertion
   must expect the original typed Source failure, with inactive-or-ineligible
   in every selected target context, rather than a provider no-route error.
   The latter remains the correct cause for compatible shared Bench/Example:
   their Dev edge is eligible in that selected target, then the ordinary Lib
   physical consumer reach fails provider dependency admission.
   Root corrected the ordinary Bin assertion to Error::Load plus that actual
   inactive-or-ineligible reason; final source reread closes this finding.

The corrected fixture must demonstrate that ordinary own-library reexports name
the actual native definition in each selected target, custom library and renamed
dependency routes survive, and every compatible physical consumer reach is
proved. A Dev-only dependency must be usable in its selected development target
but unavailable in the ordinary Lib reach of the same body. Incompatible syntax
is a separate typed Source refusal, not evidence of this provider behavior.

## Reused judgment and current execution limits

Earlier donor source reviews diagnosed and closed retained root/native account
resets, typed nested-error loss, nominal applicability overreach, alias ancestry
ordering, unproved-home exemptions and cross-consumer API identity disagreement.
Those lessons and the inspected transferred code are useful qualifying review
context. Source 60 additionally repairs target lookup/work admission and public
alias query scratch; actual current consumers must compile after integration.

The donor acceptance index records 26 library plus 73 Gate cases at a historical
integration checkpoint, and actual pinned public Hasher new/update/finalize with
frozen empty-input output. Those are attributed historical owner results, not
new runtime for this checkout. The Source dependency, new target fixtures and
pinned nonempty-input witness remain unexecuted at this review. Root must obtain
their actual terminal results and required capability lifecycle/gates/hooks.

This bounded delivery supplies the locked Gate backend. Ordinary disk
check/assert_clean still use structural check_tree, and actual CLI acquisition,
registration, RAF self-gate and planted-competitor acceptance remain with the
original parents. Checked/wrapped-output validation and domain framing are real
unfinished adoption contracts; the backend's explicit unsupported grammar neither
accepts them silently nor authorizes removing their legitimate validation.

## Native source-context cache addendum

Bounded independent source verdict: PASS for the source_files BTreeMap-to-native
OrderedTable correction in census.rs and its typed error handling in error.rs.
No new concrete ownership, semantic or failure-propagation defect was found.
No reviewer builds or product edits were performed.

## Exact inventory-count correction addendum

Bounded source verdict: PASS. Independent enumeration of the INVENTORY.records
initializer gives 144 assessment records and one each of coverage, evidence and
debt: 147 total. The four newly listed census-detectors assessment IDs have
corresponding files with the matching contract, examples, usage and verification
subjects and accepted verdicts. The correction from 140/143 to 144/147 therefore
preserves the exact inventory assertion rather than relaxing or excluding a gate.
The inspected tests/docs_contract.rs diff adds those four entries and changes
only the two corresponding count expectations; other shape assertions remain.

This is a static count and binding review, not a rerun or independent acceptance
of assessment digests. The initial hook failure remains recorded in
raw/provider-commit-hooks.txt. The retry and mandatory full qualification are
owner-run and pending at review time; the full run compiled with the old count
cannot establish qualification of this correction. No reviewer builds or product
edits were performed.

## Mutable-reference forwarding security addendum

The architecture finding is concrete in the earlier flow.rs: value() erased a
mutable-reference expression into a copied Provider event set, while call()
recognized mutation targets only from direct mutable-reference argument syntax.
Thus a local reference plus a locally bound update result concealed a required
event and admitted an earlier snapshot. This invalidates the earlier blanket
lineage PASS for that version; locked identity and home scope do not repair it.

Smallest coherent finite-grammar fix: refuse mutable references in general
value propagation, and admit them only as a direct argument whose referent is
the existing mutable provider binding. Also refuse the value of any call that
has such a mutation target: a declared API can return a borrowed mutable state,
so binding or nesting that result can recreate the lost-referent alias even
after the explicit local &mut initializer is refused. Mutating calls remain
standalone statements; a nonmutating finalizer's owned result may still be
forwarded through immutable locals. This is a precise grammar boundary, not
exact Rust referent/type inference.

The final source reread already contains both owner-applied guards in
flow.rs:308-315 and direct-argument referent evaluation at 453-474. A general
value cannot create a mutable-reference alias, including parenthesized or
shared-wrapped mutable references, and nested/bound/returned mutation results
refuse. Existing standalone update(&mut state, value) still collects its target
and event; finish(&state) still receives the updated event set. The new fixture
adds the stale-local-reference regression, wrapped-reference neighbors,
borrow-return update-result forwarding refusals and the valid direct update
followed by immutable final-result forwarding. The borrowed-result stale
snapshot case is the meaningful control; source-only fixtures with mismatched
Rust result signatures do not by themselves establish compiler-valid behavior.

No further plain shared-alias bypass was established: shared references alone
do not provide a safe mutable referent, and a direct &mut alias-binding call
still creates a required event. Do not impose a broader alias ban on that
speculation. Native APIs with interior mutation or borrow-return behavior
without explicit mutable arguments require their own declared supported
contract; this review does not claim they are inferred. Exact alias identity
tracking would be needed to support mutable-reference forwarding, but is not
necessary for the documented narrowed direct-state grammar. The capability
wording must state these refusal boundaries rather than implying general
borrowed/mutating-result forwarding is supported.

Bounded static verdict: the two guards close the identified bypass and its
mutating-call-return alias path. Owner reproducer, focused/runtime tests,
refreshed contract evidence, hooks and mandatory qualification remain required;
no reviewer builds, runtime claims or product edits were performed.

## Mutable input and opaque call-effect finding

Additional concrete source defect: flow::check ignores argument.ty and initializes
every explicit consumer parameter as Value::Input (flow.rs:141-163). With
state: &mut NativeState, finish(&state) produces a Provider snapshot and
update(state, value) sees only a path Input, so it creates no mutation event.
Binding that update's result bypasses the standalone-statement guard. Returning
the snapshot then meets an empty required set. A consumer type alias to &mut
State has the same defect. This valid-Rust shape is not closed by refusing
syntactic &mut expressions and mutating-call returns; those guards remain useful
but the overall hidden-reference-effects boundary is incomplete.

The smallest immediate conservative refusal is a closed consumer input-type
grammar: support the actually required scalar/shared-byte inputs, refuse mutable
references, and refuse unresolved aliases, generic/associated/opaque paths rather
than assuming they are immutable values. Primitive-looking names also need
shadow/alias checks; text spelling alone is not proof of builtin identity.
This input guard alone is not a general solution: an admitted native function
can return a mutable reference without receiving syntactic &mut, and a declared
callee parameter can conceal &mut behind an alias. Do not certify such paths
under a claim that only explicit mutation syntax matters.

A bounded sound extension is an actual admitted-API signature/effect summary:
every possibly mutable argument position must be proved supported, with explicit
&mut receiver/typed positions accepted only from a direct tracked mutable native
binding. Unknown alias/generic/effect positions and possibly borrowed returns
refuse unless their finite exact contract is resolved. Preserve ordinary owned
constructor results and shared finalizers; preserve direct update(&mut state,
value) as the positive neighbor. This requires no general Rust inference or
referent tracking if forwarding stays refused.

Existing production seam: Providers::observe already receives each admitted
API's selected syn::Signature and checks its ItemId at providers/mod.rs:554-576;
it currently checks only attributes. Capture a bounded native-backed summary
there, before consumer proof, with missing summaries an actionable refusal.
Avoid visit-order assumptions or an empty default. Source Item stores spans,
target and body, not a public retained signature; CodeResolver::with_configured_syntax
already supplies the exact selected syn::File when separate inspection is needed.
Alias/type paths must use that actual configured identity and retain original
resource limits; a raw mutable-token scan is insufficient.

Static verdict: substantive additional finding, owner reproduction/fix pending.
The earlier narrow-guard PASS does not establish the stronger hidden-reference
effects contract. No reviewer builds or product edits were performed.

## Native signature-effect delta review

Observed improvement: API-only traversal precedes consumer flow proof on the
same MatchLimits, missing effect summaries refuse, and each resolved native
definition must match call arity. The mutation mask is the union across actual
definitions. Explicit mutable receiver/parameter positions require direct
tracked &mut arguments, independent of consumer input spelling; borrowed native
results refuse in value contexts. Borrowed flags are combined before iterating
arguments, so zero-argument borrow-return APIs are covered. Type aliases and
opaque/generic value paths refuse instead of being assumed owned. Actual impl
Self identity comes from workspace.impls, avoiding the earlier bare Self lookup.

Remaining concrete soundness finding: nominal identity alone does not prove an
owned value contains no mutable reference. For example the native definitions
`pub struct Holder<'a>(&'a mut State); fn finish(h:&Holder)->u32{h.0.0}
fn update(h:Holder,value:u32){h.0.0=value;}` admit the valid consumer
`fn run(h:Holder,value:u32)->u32{let result=finish(&h);
let unused=update(h,value);result}`. Elided lifetime Holder is an ordinary
argument-free TypePath resolving to Struct, so owned() accepts it, but update's
mutation mask is false. The earlier snapshot again passes an empty required set.
This needs conservative refusal of reference-carrying/unknown nominal argument
effects or a finite proven nominal-field contract; merely rejecting written
generic arguments does not reject lifetime-bearing nominal declarations.
The constructor return/actual Self and direct mutable receiver positive paths
can be retained while tightening argument admission.

Resource finding: signature parameter cells and merged call masks use fallible
native WorkList allocation and share work charges, but Effects::push/signature
does not admit those retained cells against MatchLimits.states. A large-arity
native call with literal arguments can add many effect cells under a small
state grant without an equivalent consumer-binding count. Admit effect cells
under the applicable state ceiling; table/API-definition bounds alone do not
bound parameter-mask storage. New owned-type path/string scratch also remains
bounded indirectly by parsed source/work rather than a newly declared byte grant;
no claim of a new external allocation budget is made here.

Native Hasher compatibility is structurally preserved: current public new
returns actual impl Self; update has &mut self plus shared &[u8] and returns
&mut Self (standalone statement); finalize has &self and owned Hash. No general
wrapper/alias inference is required for those signatures. The owner log
raw/provider-signature-runtime-complete.txt was observed terminal with 75 passed,
zero failed; this is owner execution evidence, not reviewer rerun, and does not
exercise or resolve the Holder finding. Verdict: exact reference/zero-arity
bypasses improved, overall hidden-reference-effects acceptance remains blocked
by the concrete nominal-carrier case. No reviewer builds or product edits.

## Final required-receipt implementation verdict

Bounded independent verdict: PASS for the delivered all-call receipt design.
The native-signature prototype and its nominal-carrier/effect-mask findings
above are historical experiment findings, not descriptions of the final source.
flow/effects.rs and its API-only prepass are absent. The experiment is retained
only in raw/provider-signature-prototype.patch and
tasks/provider-signature-prototype.rs; neither is delivered product source.

Final flow::call unconditionally admits a fresh required event for every
successfully resolved native call, including constructors, reads, zero-argument
borrow producers and calls receiving flat Input/nominal values. Its returned
Provider set contains the event plus argument provenance. Only direct tracked
&mut arguments propagate that set into state. Thus the input/alias/borrow-return
and Holder updates each leave a missing receipt in an earlier snapshot and
cannot qualify it, without guessing native parameter purity or field ownership.
The mutable-reference forwarding and tracked-mutating-call result refusals
remain. Shared work limits are unchanged, and the existing state admission now
charges two cells for the new required/result receipts plus newly propagated
target provenance before insertion. There is no separate effect-mask allocation
or fresh signature-classification allowance in the delivered path.

The helper invoked by the existing bound mutation test exercises mutable inputs,
consumer aliases, zero-argument borrowed results, native parameter/return aliases
and the nominal Holder negative. The neighboring Holder read is accepted.
Existing/new direct update and immutable final-result forwarding remain valid;
extra unused constructor/read calls are refused by required receipt reachability.
The precise contract remains argument/state taint through trusted native calls:
passing a receipt-bearing result into another native call counts as contribution;
it does not infer whether that API semantically ignores an argument. Hidden
updates without a tracked state edge may conservatively refuse fresh results.

The census-ledger Contract, Usage, Examples and Constraints faithfully state
every native call's receipt must reach returned argument/state lineage, unused
calls refuse, direct mutable-state edges are explicit, and hidden reference
identity is not inferred. This supersedes signature-shape ownership claims.

Observed owner evidence: raw/provider-receipt-runtime-accepted.txt ends with
26 library plus 75 Gate tests passing, none failed/ignored; the accepted Clippy
log ends successfully. raw/provider-receipt-native-proof-final.txt records
canonical documented ledger parsing, frozen native BLAKE3 bytes at input lengths
zero and one, and a clean locked public Hasher home at pinned revision
5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43. These are owner-run receipts, not
reviewer executions. Mandatory full gate, refreshed prose/lifecycle evidence and
hooks remain owner qualification; no pending run is called a pass here.
No reviewer builds or product edits were performed.

The table starts empty, and one Memory<Resident> remains in scope for all of its
insertions; no live arena is passed to a fresh zeroed account. The pinned native
insert admits/reserves slots before publishing a new association and preserves
existing associations on refusal. RelPath clones share Arc text and retain their
SourceAuthorityId, so keys acquire no unadmitted owned text copy and equal printed
paths from different source authorities remain distinct. The table owns only
RelPath/usize associations, not a clone of allocating file-context payloads.

Only exact-key lookup consumes this cache. Existing Workspace file iteration and
files vector order are unchanged; each successful insertion records files.len()
immediately before pushing that same FileCtx. Native ordering cannot change
public file iteration or the selected physical-source context. The loader's
unique full RelPath keys preserve the old map's replacement semantics/invariant.

Native layout/allocation failures become SourceIndexStorage with the original
StorageError retained by Error::source, and its Display identifies source-context
ownership-index reservation. The error-rendering test asserts both the diagnostic
and the downcast original AllocationFailed cause. Failure exits before a Census
is returned, so no partial source-context index can produce a clean Gate verdict.
Resident supplies fallible native allocation, not a new byte grant or a claim
that all Census allocations spend Source's loader allowance.

Root reports the preceding candidate-only validation checkout, against Source
36a3b70, passed 75 Gate cases, 26 library cases, all-target Clippy and the pinned
native witness. That evidence predates this narrow table correction. Root is
rerunning affected runtime/lint/native witness now; this source addendum does
not claim those current terminal results, integrated-main acceptance, mandatory
repository gates, native lifecycle or hooks passed. The earlier broader source
judgment remains applicable because routing/proof semantics are unchanged.

## Runnable source example and documentation addendum

Bounded source/document verdict: PASS. Reviewed examples/providers.rs and the
ledger/detector binding, Usage, Examples and Verification changes after Source
integration. No new concrete defect was found, and the earlier backend judgment
remains applicable. No reviewer builds, product edits or forge actions.

The runnable example owns a MemoryTree source fixture, constructs its ledger
through the native RDF Builder/writer, and supplies exact lock rows, renamed
dependency, custom provider library name and declared library configurations to
the real Gate::check_locked. It requires a clean report for direct approved
encoder forwarding, then requires an actual hex-table finding for an unlisted
helper in the adapter package. The approved API is an exact public function home;
the rest of the consumer package receives no blanket exemption. All fixture
construction and Gate failures propagate through Result; neither case rewrites
tracked files, writes debt or treats a missing executor as a clean verdict.

The encoder and competing table are deliberately source text inspected by the
policy. This example demonstrates configured source-policy evaluation, not
execution/output qualification of that synthetic encoder or a pinned PurRDF
build. The separately required actual public-Hasher empty/nonempty witness
remains the native execution evidence; the example does not replace it.

The detector Usage now contains an entire canonical RDF 1.2 native snapshot,
including its real BASE identity and current normal/report versions, and uses
the actual forbiddenFingerprint vocabulary. It directs ordinary jobs to the
structural Gate and preferred-provider jobs to check_locked, without claiming
the unfinished CLI/self-gate already supplies grounded compilation context.
Both capability example bindings identify the actual new executable target.

The two registry/fixture metaguards remain explicitly proposed under the existing
census-detectors change. This accurately avoids claiming absent enforcement as
delivered; their original parent requirement remains live and no test/code gate
was removed or weakened. The detector capability remains implementing, and this
bounded delivery does not close its broader detector/default/reference scope.

The inspected raw/provider-example.txt records compilation followed by the
success line for both asserted example cases. The inspected
raw/provider-candidate-native-ledger-usage-proof-canonical.txt records that the
documented detector ledger parsed, followed by both frozen native input outputs
and a clean pinned public-Hasher Gate. These are owner execution logs, not
reviewer reruns. The earlier failed usage-proof snapshot remains a failure in
its separate log. Final example/lint terminal status, refreshed assessments,
native lifecycle, mandatory gates and hooks remain root-owned qualification.

## Source-ownership binding addendum

Bounded document/source verdict: PASS for the two added census-ledger test
bindings and their Grounded provider execution tested-by mapping. The exact
functions exist under raf_census::gate::owners::tests.

equal_display_paths_keep_distinct_authorities_and_consumer_scope builds two
source authorities with the same displayed src/lib.rs path, resolves both actual
owners, then selects only first-party packages and asserts the provider unit,
source context and owner resolution are absent from that consumer Census.
It still resolves the provider definition through the shared Workspace. This
supports typed authority ownership and external-lookup/consumer-scope separation.

every_pattern_kind_uses_the_selected_consumer_file_set declares all six matcher
kinds over equal displayed source paths, evaluates the selected consumer set as
clean and includes the external source as the copied-code positive control.
It supports the claim that pattern occurrences follow the selected consumer
scope; other already bound integration fixtures supply the individual matcher
execution contracts. These source-ownership tests alone are not claimed as
configured execution or native-result proof.

No gate exclusion was added. The inspected root test diff only adds the current
change/capability mapping and assessment records to its explicit inventory;
hooks, Makefile, CI configuration and Census manifest have no corresponding
exclusion change. The tests and their assertions remain intact. Current
assessment/reconcile/native completion outcomes remain root-owned, and this
binding review does not relabel the earlier unbound-test refusal as a pass.
No reviewer builds or product edits were performed.
