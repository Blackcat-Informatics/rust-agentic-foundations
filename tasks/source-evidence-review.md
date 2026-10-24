<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Bounded Source evidence review

Independent source inspection of the Source-only delivery, including its new
receipt scratch correction and target-context work accounting. No reviewer
builds, runtime probes, product Rust edits or forge actions. Documentation edits
were a separately assigned task. Root owns actual runtime, lifecycle, mandatory
qualification and hooks. This review does not claim Census execution, full
external/std resolution or parent completion.

## Current disposition

Native retained-list ownership and original typed provenance findings are closed
on source inspection. The listed target-context Work findings are closed on
source inspection. Candidate-root scratch now has the right native ownership
and composed-usage shape, including the final rejected-candidate scan admission.
The method-applicability over-refusal below is now corrected on source reread.
Runtime/lint after these final changes remains owner qualification.

## Scope and native load admission

load_locked_sources_under uses explicit first-party discovery roots while
retaining the complete exact local manifest associations and sealed external Git
roots. An omitted hint does not narrow directory discovery; a caller can select
an explicitly narrower root. Existing package/target/source identities remain
typed, rather than reconstructed from printed crate names.

The donor scope review identified retained roots/hints crossing a fresh native
account and views/trees resetting one account's live total. Current construction
keeps one Memory/ItemAdmission for those simultaneously live buffers, passes it
through add_roots and recovers once at the outer boundary. The target-context
addition gives rows, each active-target vector and the configurations vector
valid separate accounting ownership. configure retains one Memory throughout
its vector construction. workspace keeps infos and dependencies in separate
accounts rather than resetting one admission for a second live allocation.

ItemAdmission::recover restores its own saved refusal when present; otherwise
it returns the incoming typed LoadError unchanged. Original nested file, line,
origin and storage kind survive. The dedicated paired meter fixture inspects
both branches. The original source-scope review and owner receipt index remain
in the donor stage; this review rereads the transferred code rather than treating
those historical receipts as current full qualification.

## Borrowed library and associated-member boundary

extern_libraries_at yields borrowed actual indexed library candidates. Renames,
custom library names, prelude candidate groups and explicit extern declarations
retain their actual roots. Same-package roots do not become external receipts.
Each receipt's public-surface query keeps original defining sites and refuses
private endpoints, rather than treating matching spelling as export proof.
Prelude and lexical routes intentionally differ: a prelude receipt retains the
original competing roots; a genuine lexical extern declaration keeps its own
declaration roots. The existing paired code_paths fixtures exercise both.

Alias projection only follows a final missing Value member through one actual
TypeAlias prefix. Every alias hop resolves its retained RHS from its defining
module, records the ItemId and propagates original errors. Supported aliases
have no parameters and only literal-const nominal arguments. Cycles refuse with
Cyclic; unsupported shape, substitution or arity refuse with AliasMember.
Inherent precedence requires direct nominal const arity and restricted impl
headers, preserving canonical inventory ambiguity. Declaration rewriting and
configuration remain explicit caller audits, not a Source execution certificate.

Donor raw/provider-alias-ancestor-runtime.log contains an actual owner-run
11-case Source code_paths pass alongside 64 Census Gate cases. Only the Source
portion is relevant context here; it predates current scratch/target changes and
is not final acceptance for this delivery. Provider ancestor proofs and native
output receipts are outside the Source-only delivery.

## Candidate-root scratch correction

candidate_roots now has one native Memory/ItemAdmission for its vector throughout
construction. Original group roots and inherited roots are merged, sorted using
the same item sort key and deduplicated by identity. Sort comparison strings and
dedup steps are metered. Native failures recover their original Budget cause
through CodeResolveError::Query; an ambiguous result owns that admitted vector.
This closes the former unadmitted BTreeSet scratch allocation.

resolve starts from the original CodeResolver usage, passes the spent candidate
usage to resolve_code_after, and then passes returned usage into exports_after.
Direct resolution returns the supplied usage unchanged; alias fallback resumes
that usage instead of opening a fresh query grant. The final export-membership
scan resumes the composed surface usage. The shared extern_root_candidates
iterator preserves the old extern_roots prelude/explicit semantics; existing
ordinary resolver traversal is not being replaced with a new query engine.

The final narrow Work finding was: Group::roots filters inactive prelude rows, and
extern_root_candidates filters root definitions that are not ExternCrate before
candidate_roots sees a yield. Charging only yielded roots omits those rejected
input entries. The inspected correction calls charge_candidate_scan before
iteration. Group::scan_len admits every raw group row; inherited Prelude queries
admit the active prelude's raw root count and the root Type-definition count
before the shared iterator filters them. Yielded module roots, sorting and dedup
continue to charge their own work. This closes the filtered-input finding without
rebuilding the general resolver. The lazy extern_libraries_at enumeration itself borrows existing
prelude/crate_of tables and allocates no retained scratch; it is distinct from
this owned candidate-root query finding.

The new external_library_query_scratch_uses_the_original_native_grants fixture
pairs default-grant public-query success with typed Query/Work and Query/ItemBytes
failure when the original allowance leaves no additional query grant. This is
not an exact minimum successful query budget. Documentation now states that
precise evidence boundary; the target-list/load fixtures retain their separate
exact-bound and one-unit-short controls.

## Target contexts and work repairs

Explicit target kind/name configurations cover Lib/Bin/Test/Bench/Example and
BuildScript. Duplicate/missing selection and missing required features refuse.
Own-library companions are ordinary contexts with matching triples and unified
features, while custom predicates remain caller declarations. Dev edges are
available to actual development or declared test contexts and cannot leak into
ordinary selected siblings. Destinations select one ordinary library; host and
target configuration responsibility is preserved. Physical cached facts are
shared only for the same package and selected syntax; incompatible views refuse.

All earlier listed target-context Work repairs are present: duplicate names and
triples charge their byte comparisons; feature/predicate comparisons admit each
entry incrementally; select_targets retains using one fallible metered lookup;
preludes admits active-target scans; root test selection admits reached scans.
with_configured_syntax resumes original usage before file/reach/configuration
lookup and passes the spent usage into the existing bounded parser/filter.
The actual-target fixture selects cfg(test) declarations, pairs an unreached
file refusal and preserves typed original Work exhaustion. Structural-only
syntax inspection also has a refusal neighbor in code_paths.

## Method-local applicability finding

Originally facts::unconditional_self rejected an entire impl if any sibling method
has generic parameters or a where clause. For example, `struct S; impl S {
fn generic<T>() {} pub fn run() {} }` causes the nongeneric `S::run` to fail
require_unconditional_inherent_method, and prevents its inherent precedence
beside a same-name trait member. Its own header and self type need no inference.
The new block metadata introduced this conservative over-refusal; it is not a
request for general generic substitution. Current correction removes the
sibling-wide veto from unconditional_self. The final with_bindings representation
records a negative method_requires_inference flag for that selected impl method's
own generic parameters or where clause. unconditional_method refuses that flag
before the existing block self-type/nominal-arity proof. Plain zero-parameter
methods keep no detail allocation; zero-parameter generic methods retain their
required refusal metadata under the fact-layout charge. This closes the source
finding without general type inference or an extra common-case Arc. The new
inherent_method_applicability_is_local_to_the_selected_signature fixture pairs
the nongeneric method and generic sibling in both orders, and checks generic or
where-bounded selected methods beside same-name trait members. The paired
runtime result remains owner qualification.

## Acceptance ownership

Root's runtime session was live at the final review request. Its subsequently
read log records 134 library cases passing and one exact glob work-count failure
(chain of 8, actual 17 versus expected 16). That run is not qualification; the
owner must diagnose the changed computation. Current final owner results,
fixed finding dispositions, capability reconciliation, mandatory make check,
normal commit hooks and publication readiness must be recorded in the current
stage validation index before completion is claimed.

## Final correction reread

Bounded source verdict: PASS. No open concrete source finding remains in this
reviewed Source evidence boundary. The former exact structural work-count
failure is also explained by current code: add_roots now uses the unchanged
target-kind path for unconfigured loads and performs/admits the reached-context
scan only for configured loads. It removes the unnecessary computation rather
than weakening the counter or test. Root must collect the actual affected test
result; this source explanation does not relabel the earlier failed run.

The final reread covers library candidate scratch and rejected-input accounting,
scratch/alias/surface usage carry, per-method applicability, configured target
selection/physical cache consistency, syntax-query usage and original native
error provenance. Runtime, all-target lint, full repository gates, capability
lifecycle and normal hooks remain root acceptance responsibilities. Broader
resolver inference and all Census provider/CLI/self-gate contracts remain open.

Root reports terminal 0 for raw/source-evidence-final-focused-runtime.txt. The
inspected log records all 40 focused cases passing: 13 code_paths, 12
configured_sources, seven configured_targets and eight locked_sources. This
includes the final candidate-root original-grant refusal, method-local
applicability, exact target-list/load grants and actual-target syntax query
controls. Settled lint and the isolated structural Work regression are still
live at this update; their results and mandatory make check remain root-owned.
No reviewer tests were run and no product source was edited by this reviewer.

## Final metadata and const-API correction

The native completion's recorded budget refusal was an actual plain/bodiless
FactBytes mismatch: the earlier positive eligibility representation created an
extra ItemDetail for one zero-argument form (88 bytes). The negative flag above
removes the unnecessary common-case allocation instead of changing a limit or
weakening the equality assertion. bodiless now receives its actual syntactic
context: false from top-level named items, true from impl_member. Consequently
an unsupported generic bodiless impl signature retains the same refusal flag
as its body-bearing counterpart; absent detail does not falsely admit it.

The existing method-local regression now includes plain and generic bodiless
methods beside the trait ambiguity control. The existing load_budgets parity
fixture includes a generic plain/bodiless pair and retains its original exact
FactBytes success and smaller-grant refusal assertions. Source inspection found
no new concrete defect in this correction.

LockedBinding::configuration again retains its original public const fn
capability. A slice singleton match returns the sole context and refuses empty
or multiple contexts without allocation. The installed standard library marks
Vec::as_slice const-stable since 1.87.0, below the repository's 1.98.1 MSRV.

The inspected raw/source-final-metadata-runtime.txt records all 64 focused cases
passing: 13 code_paths, 12 configured_sources, seven configured_targets, 24
load_budgets and eight locked_sources. Command terminal confirmation remains
root-owned. The earlier native completion refusal remains historical failure;
this review does not report that completion rerun, final gates or hooks passed.
