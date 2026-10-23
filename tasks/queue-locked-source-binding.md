<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Checked source to actual package binding

Current source adapter: the following join is retained, but the raw-object
prototype has been replaced by `CommittedGitTree::open` using trusted installed
Git. The current actual route, API, dependency PASS and host/portable limits are
in `queue-committed-git-implementation.md`. Earlier raw-object receipts below
remain unchanged historical evidence.

The production join is implemented through the existing loader and resolver.
This supersedes the missing-join observations in the earlier Git Tree and
source-only design checkpoints. It does not complete the original configuration,
macro, external-resolution or differential contract.

Public interface:

- `LockedManifest::new(row, RelPath, &[LockedEdge])` supplies candidate locators.
- `LockedEdge::new(exact_declaration_key, Option<RegistryIndex>)` explicitly
  requests an outgoing dependency; other edges remain unresolved syntax.
- `GitPackageRoots::new(&VerifiedGitTree, &[LockedManifest])` retains the whole
  authenticated repository root. Authority labels cannot substitute for it.
- `Workspace::load_locked_sources(&LockedCatalog, &dyn Tree, local, sources,
  &LoadOptions)` produces actual discovered package IDs and ordinary index data.
- `Workspace::locked_binding(PackageId) -> Option<&LockedBinding>` returns the
  computed `row()`, `name()`, `version()` and `source()` receipt. Ordinary loads
  have no such receipt.

The single manifest parser now retains explicit package version or workspace
inheritance. Missing versions remain accepted by ordinary loading, but cannot
silently synthesize grounded identity. Bound metadata must equal its exact lock
row. A Git manifest must additionally be the unique matching name/version in
the checked root package or root workspace membership, using existing membership
and inherited-version logic. A standalone nested fixture cannot claim the row
by copying labels. Other repository layouts explicitly refuse under this
binding profile; broader source acceptance remains unfinished.

Only the actual owning row's selected outgoing edge can install a destination
PackageId in existing DependencyInfo/preludes. The destination must have checked
source and manifest evidence. Duplicate associations/rows and requested missing,
ambiguous, unconfigured or unadmitted destinations refuse. A path edge of a Git
workspace retains its owner's exact Git source; a local path edge remains local.
In both cases the path must name the actual selected manifest directory. No
same-name external fallback is installed. Requested optional or conditional
edges currently refuse without configuration admission; this is not active
feature/cfg or preferred-provider acceptance.

Catalog retained bytes, snapshot rechecking, source verification, selection and
normal walking use one original Meter grant. Snapshot constructors can still be
used separately, but the joined load borrows its own shared meter and rechecks
the commit/root before continuing. Native Memory and fallible sorting/searching
cover retained association buffers; comparisons and source reads remain charged.
The exact/one-short test checks both snapshots together with catalog and walker,
not independent grants.

Actual receipts (raw files retain uncompressed output and real status):

- `queue-locked-binding-callers-initial`: workspace all-target caller compile,
  session 45000, exit 0.
- `queue-locked-binding-final-focused` and `-final-clippy`: six earlier binding
  tests and all-target Source Clippy, session 43896, both exit 0.
- `queue-locked-locator-focused`: final seven binding tests, session 15773,
  exit 0. This adds same-name/version nested-fixture refusal with the real root
  member as a valid neighbor. Other cases cover two repositories with identical
  package names/versions, an alias/reexport's actual target ID, wrong manifest
  metadata/source, missing/optional requests, unrequested names, missing version
  versus ordinary load, and the shared resource grant.
- `queue-purrdf-pinned-complete`: first actual pin query, session 10763, exit 0.
- `queue-purrdf-locator-complete`: actual pin query after locator strengthening,
  session 32879, exit 0; no default resource limit changed.
- `queue-locked-locator-clippy` exited 101 only for two lints in the temporary
  diagnostic acquisition example. That example was retained as
  `tasks/queue-purrdf-pinned-complete.rs` and removed from product source.
  Final product all-target Source Clippy passed, session 31008, exit 0:
  `queue-locked-binding-complete-clippy`.

The actual host probe reads immutable objects from the existing Cargo Git cache,
with replacement objects and lazy fetching disabled. It does not clone, checkout,
fetch, mutate that cache, or operate services. Object sizes are checked before
bounded acquisition. The portable verifier independently authenticates used bytes.
The final capture has 2,311 objects and 3,823,566 body bytes. Its native pin is
`5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43` from the actual lock row.

From the real raf-source reading package/module, `Memory` resolves through the
requested locked edge to `purrdf_lex::allocation::Memory`, actual source
`crates/lex/src/allocation.rs:367`. The `Shared` reexport resolves to
`purrdf_lex::allocation::shared::Shared`, `allocation/shared.rs:80`. Both target
the actual loaded purrdf-lex 3.0.1 PackageId and its exact Git binding, rather
than a name-only fixture. Flipping a supplied allocation.rs byte refuses its
authenticated object hash before binding. This certifies committed bytes, not
dirty-checkout correspondence or repository authorship. Final joined usage:
10,467,266 total bytes, 30,331,666 item bytes, 3,875,259 fact bytes, 388,629 glob
bytes and 10,662,552 work units; all use unchanged defaults. The diagnostic
example was still part of the first-party source during that run.

Earlier acquisition failures are preserved: initial incomplete metadata capture
refused a missing workspace Cargo.toml; expanded pipe-based transport blocked and
only its confirmed owned child was stopped. The final probe uses stage-local
stdin files to avoid pipe backpressure. Those failures are not passing evidence.

## Decoder closure

Both gix-object =0.64.1 and gix-hash ^0.26.2 already use
`default-features = false` and only `sha1`. The former forwards that feature to
gix-hash/sha1; the latter enables only its optional sha1-checked backend and
explicitly refuses compilation without sha1 or sha256. Signature, serde, SHA-256
and parallel support are not requested. The maintained decoder unconditionally
depends on actor/date/hash/hashtable/utils/validation and progress components;
its published feature set has no commit/tree-only decoding profile. Removing
sha1 does not preserve the supported public decoding API. Production object
hashing itself still uses native PurRDF Sha1.

The 49 additional decoder package rows remain material and visible; no supported
feature reduction was identified by this source inspection. No parser replacement,
upstream modification or speculative version churn was performed. Licensing,
deny, MSRV and platform qualification remain required and were not run here.

Changes for this join: new workspace/locked.rs, tests/locked_sources.rs and shared
tests/support/git_objects.rs; workspace.rs, manifest.rs, locked/{mod,select,source}.rs,
tree/git.rs, index.rs, lib.rs, git_tree.rs and the source-index capability. Earlier
dirty source, lock files and stash remain preserved. No commit, push, PR, full
suite, benchmark or original-delivery completion is claimed.
