<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Preferred functionality provider policy

Read-only design, no source edits or builds. Existing source ownership is
unchanged: strict owner retains this checkout; primary retains resolver work.
Read captured `provider-policy-issue-5.md`, `provider-policy-issue-9.md` and the
portfolio before inspecting contracts/source; no repeated forge intake.
Repository memory recall corroborated the explicit user requirement but was
not treated as delivered code. Main Graphify is unavailable; no rebuild made.
The canonical main checkout currently lacks a Census source directory; the
strict checkout contains the integrated foundation inspected here.

## Required result and actual starting point

The user requires “always use library or project X for functionality,” with
`purrdf.*` preferred throughout RAF production, tests, examples, benches and
tools. A dependency, import or ignored/dummy call does not discharge this
obligation. Valid RAF-specific validation of an actual native result remains
an adapter. Missing/stale/ambiguous provider evidence and unsupported delegation
must fail actionably, and provider violations must never become baseline debt.

Preserve the original fourteen detector ids, their shipping/all scopes and
fixtures, six pattern forms, custom Rule registration/fixture checks, reference
scanning, ledger/staleness/debt and both original self-census flagship scenarios.
The new all-target provider policy does not silently change the existing
shipping-only built-in detector semantics. Ledger representation remains in
the ledger delivery; shared checks in detector/rule/reference delivery; RAF
policy and production self-gate adoption in the self-census delivery.

Current `raf_census::Census` implements source units, normal forms, grouping,
in-memory `Job` matching and RDF reports. `FileCtx` already records shipping
and all reached TargetKinds. `Unit` retains source identity, location, body
fingerprint and constants; it does not retain a resolved result-delegation
certificate. There is no implemented provider checker, ledger parser, Rule
registration engine or Gate in this source snapshot. Those remain designed.
Do not sell an import-pattern check as filling that missing functionality.

The designed `Rule::check` sees one parsed file plus FileCtx and emits RuleHit.
That is sufficient for structural competing-implementation witnesses, not for
proving dependency package identity, cross-file aliases/reexports or returned
native result ownership. Add a resolved workspace-wide provider phase to the
intended shared check pipeline; do not overload the existing per-file Rule
with hidden global state.

## Smallest coherent shape

Reuse a ledger job as the functionality unit and its native `home` as the
canonical implementation. Keep `entry_points` as additional paths to that
implementation; **do not reinterpret it as a list of local wrappers**.
Add one closed preferred-provider subtable and a shared provider identity table.
Illustrative new schema, not implemented or accepted syntax:

```toml
[[provider]]
id = "purrdf"
packages = ["purrdf-hash", "purrdf-lex", "purrdf-core", "purrdf-xsd"]

[[job]]
id = "canonical-digest-hex"
summary = "Canonical lowercase digest spelling and decoding."
home = "purrdf_hash::hex::decode_32_canonical"
entry_points = []
enforced = true
# Existing forbidden keys still present here, unchanged.

[job.preferred]
provider = "purrdf"
consumers = ["raf_workplan::digest::Digest::from_str"]
proof = "returned-result"
checks = ["hex-pair-radix", "hex-table"]
```

The example identifies one operation, not all hex functionality; encoding,
formatting and decoding need their actual respective homes. Resolve every
consumer and registered check id. No unknown keys, empty placeholder consumer,
unmatched selector or unresolved method may silently pass. The provider policy
is mandatory even if ordinary legacy `job.enforced` is false: either forbid that
combination at validation or make preferred-provider findings independently
non-debt/non-waivable. Do not let variant/distinct/debt mechanisms sanction a
competing preferred implementation.

Resolve packages to exact identities from manifest dependency aliases and
Cargo.lock source/version/checksum or git revision, then resolve public API
paths and reexports to those identities. Match a provider project to explicit
resolved package members, not a written `purrdf_` token. A fake local module,
same-name different-source crate or unrelated renamed dependency cannot qualify.
Verify API/body evidence against the same pinned source snapshot. Current
`~/Active/purrdf` main supplies candidate APIs; the actual Cargo pin determines
what this RAF run can prove. A pin/API change invalidates that evidence.

The captured requirement explicitly replaces the old reference-scan contract's
`cargo metadata` JSON recipe with manifest/lock resolution through raf-source.
Update that designed contract when implementing; do not revive the stale recipe.

## Real delegation, without a generic semantic oracle

Two complementary checks are required and neither substitutes for the other:

1. **Competing implementation witnesses.** Scan every first-party target using
   the existing reference normal form and registered structural checks. Compare
   local bodies/shims to the exact reference implementation; run structural
   detectors for known reformulations. Report actual local symbol/file/span,
   normal-form or structural witness, functionality id, canonical provider
   package/API and required replacement. Native source is read-only reference,
   never first-party debt. Function names or crate prefixes alone are not proof
   of equivalent behavior. Do not depend on min_tokens to catch tiny helpers.
2. **Declared consumer ownership.** Validate a small explicit source grammar
   for the selected proof kind. Start with resolved direct-call return and
   a returned local binding initialized by that call, plus explicit Result/Option
   error conversion/validation forms whose success value is the native value.
   Every successful return must satisfy the contract; error exits may be local
   validation refusals. A call followed by return of a local algorithm, a provider
   result assigned to `_`, a dead branch call, or `.map(|_| local_kernel(...))`
   cannot certify delegation. The provider result must be consumed by the
   returned result or the declared mutating output operation, with relevant
   provider inputs derived from the consumer's declared inputs. Presence or
   undifferentiated “taint” is insufficient.

Keep the source grammar typed and bounded, over the existing syn AST/token
source. Unsupported forms are actionable proof errors, not guessed success.
Methods require actual receiver/callee identity, not a method-name whitelist.
Stateful hash update/finalize, allocation ownership and RDF writers need their
own small registered structural proof: receiver originates from the native
constructor/type, calls operate on that same receiver/output, and returned
digest/text/storage comes from it. Do not claim the direct-return checker proves
those forms. Extend registered proof kinds only with their own positive/negative
fixtures; this is not permission to invent a generic Rust interpreter, second
parser, type checker or kernel.

The accepted adapter forms should be useful immediately: closed RAF numeric
grammar before native parse_decimal; validation of a native parsed value;
native canonical hex decode mapped to a RAF error; native RDF parse followed by
closed report-shape validation. Local copying of the parsing/decimal/hex/RDF
operation remains a competing implementation even if a dummy native result
also reaches some irrelevant output field.

No finite structural rule set establishes arbitrary program equivalence or
discovers every possible new functionality. Keep the functionality inventory
explicit and auditable, run copy detectors across the entire first-party
surface, and fail unsupported declared obligations. “Prefer PurRDF everywhere”
is not complete adoption merely because the initial hex mapping passes.

## Native implementation and production callers

Use the existing native kernels rather than duplicating the gate's own work:

- Reference prints: current `Normalizer` and `purrdf_hash::blake3::Hasher` in
  `normalize/fingerprint.rs`; hex identity formatting/decoding via
  `purrdf_hash::hex::{Lower,decode_32_canonical}`.
- Bounded provider/delegation worklists and vector growth:
  `purrdf_lex::allocation::{Admission,Memory}` (`reserve`, `push`, `try_map`,
  scoped accounting) rather than a second admission engine. Preserve original
  typed refusal and charge before allocation.
- Cycle/group traversals where needed: existing
  `purrdf_core::graph::scc_component_index_with_memory`, already consumed by
  raf-source. Do not copy an SCC kernel for provider reexport chains.
- Typed report construction and native RDF 1.2 terms/writing through the
  existing report/raf-rdf path, preserving graph identity/profile law. No new
  generic graph, RDF writer, hash implementation or JSON report model.

Analyze while the source Workspace and scoped parsing worker are available.
`Census::load` presently drops its local Workspace after collecting plain Unit
data; do not reparse/reload the entire workspace just to recover missing
provider evidence. Extend the shared collection pipeline to return plain,
resolved proof witnesses with Units, retaining proc-macro span ownership and
pre-allocation budgets. A caller-supplied Workspace path can share the same
production collection core if the next source delivery introduces that seam.

The intended `Gate::assert_clean` and `cargo census check` must invoke the same
shared built-in provider engine after validated package/ledger resolution.
Registered repository proofs/checks still require the Gate path, as the Rule
contract states. `write_debt` must refuse provider findings before any rewrite.
RAF's actual `tests/census.rs` then supplies the complete preferred-operation
inventory and repository structural proofs. A disconnected helper or a new
optional CLI mode is not adoption.

## Required adjacent fixtures and ordering

For every proof/check: genuine delegation passes; copied implementation fails;
dependency-only/import-only/ignored call/dead branch/dummy-result return fail;
RAF validation consuming the real native result passes. Include alias and public
reexport positives, shadowed local name and same-name foreign package negatives,
stale lock/API/body evidence, missing/ambiguous source, unsupported delegation,
and a budget refusal with valid admitted neighbor. Exercise shipping, test,
example, bench, build/tool targets and registered multi-file paths, not only a
single library function. Prove provider findings survive ordinary debt inputs
and debt-write refuses unchanged output. Plant competing implementations and
dummy delegation through the actual self-census gate.

Dependencies: settle current strict/native caller work and resolver correctness
first; dependency/cfg/source-symbol grounding from the queued source delivery
is a real prerequisite, not supplied by glob resolution alone. The ledger,
shared detectors/reference/registration and self-gate deliveries can be designed
as one coherent sequence with their original parent obligations preserved.
Native reuse now is not a substitute for the automatic gate; automatic gating
cannot qualify before exact dependency identities and actual callers exist.
No delivery, performance, wasm, whole-Census or full-gate acceptance is claimed.
