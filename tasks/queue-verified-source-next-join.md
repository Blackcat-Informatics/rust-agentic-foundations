<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Next verified source boundary and join

Source-only design checkpoint, not execution or completed acceptance. This reuses
`queue-source-authority-design.md` and the implemented declaration/edge notes;
it does not replace the original issue requirements. The parent subsequently
authorized implementation of the immutable Git Tree prerequisite only.

There is no honest provenance proof through current `Tree` alone. `read` returns
UTF-8 text, `read_unseen`/FileId provide physical identity, and `list`/`entry`
provide topology; all may be implemented by callers. SourceTrees and authority
constructors deliberately provide separation, not authentication. A claimed Git
revision or authority cannot authorize a provider.

The minimal additional boundary is one bounded raw Git-object supplier and an
opaque checked immutable Tree. The supplier returns object kind and exact binary
body for an explicitly requested object ID, receiving the maximum before
acquisition; it supplies evidence rather than a trusted verification flag. The
Tree constructor checks the actual object-framed hash against the catalog's
full locked commit, decodes the commit's tree reference, then checks every used
tree/blob against its authenticated parent ID. Hash `kind SP decimal_length NUL
body`, never body alone. Directory decoding must exhaust fallible entries before
publishing membership/absence, reject invalid/duplicate names and unsupported
traversal, and preserve exact UTF-8/blob bytes. File identity needs snapshot plus
path, not blob ID, because equal-content modules have different contexts.

Existing native APIs verified at pin
`5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43`: `purrdf_hash::sha1::Sha1`
(`new`, `update`, `finalize`, `digest`), `hex::decode_to_slice`, native
`Memory`/`Admission`, `OrderedTable` and `WorkList`. The pin does not export
SHA-256 (its hash lib exports SHA-1 and SHA-3); catalog observation of 64-digit
Git revisions is not executable Git-object verification. A SHA-1-only first
binding must refuse other formats explicitly and leave their acceptance open.
No compatible native Git object decoder was found in the prior audit.

Cached maintained `gix-object 0.64.1` has `CommitRefIter::from_bytes` and
`TreeRefIter::from_bytes` with an explicit hash kind. Its actual
`tree/ref_iter.rs` uses `filter_map(Result::ok)` inside `next_entry`, and
`lookup_entry` delegates to it; do not use those helpers. Iterate every raw
Result with `?`. Commit convenience iterators also discard errors; use the raw
fallible tokens. Dependency resolution, features and licensing still require
actual qualification; no successful dependency build is inferred from this read.

Keep host acquisition outside portable semantic verification. A host adapter may
read actual immutable objects from the existing pinned Git store, without a new
checkout, server or automatic fetch; the portable verifier checks the bytes
itself. Generic supplier prior allocations are not retrospectively admitted by
RAF. A real host supplier must bound acquisition and its buffers before reading.
An ordinary-checkout adapter additionally checks actual HEAD against the expected
commit and checks relevant returned files/topology against authenticated objects;
caller labels, origin URLs, `.cargo_vcs_info.json` and `git status` alone do not
prove this. Edited manifest/source bytes and discovery-visible additions/removals
must refuse in that adapter. An immutable object Tree instead exposes committed
contents and never reads a dirty checkout; do not sell that as a dirty-checkout
verification result. No hostile-filesystem or repository-authorship claim follows.

Exact remaining join changes, after Tree admission:

1. `manifest.rs::PackageDecl` currently discards package version, including
   `version.workspace`; retain and validate it using the same parser/cache as
   edition inheritance. Require actual explicit/inherited version evidence for
   grounded identities; do not synthesize an identity from a convenient default.
2. A locked load entry point validates each explicit source/manifest association
   against its checked snapshot, name and effective version. It mounts each
   admitted repository once via SourceTrees and discovers selected package roots
   using the existing `Workspace::load_from` path. Keep the repository root for
   workspace inheritance and sibling paths; for PurRDF select `crates/lex` and
   `crates/hash`, rather than treating a crate directory as the repository root.
3. Compute the opaque `(LockedPackageId, authority-qualified manifest) ->
   PackageId` association inside that same load, from exact discovered manifests.
   Reject absent/duplicate/wrong-source manifests. Public caller-supplied PackageId
   or name equality cannot fabricate the mapping. A source-less local lock row
   still needs its explicit first-party manifest association; it is no universal
   path identity.
4. `workspace.rs::dependency_info` currently resolves only path dependencies.
   For qualified external edges, use the selected owner's retained declaration,
   `LockedCatalog::select_dependency`, then the verified row mapping. Supply its
   real PackageId into existing `index.rs::DependencyInfo`; preserve kind/rename
   and `Builder::preludes` visibility routing. No parallel resolver is needed.
   Relevant unresolved optional/features/target configuration must remain
   incomplete evidence, rather than turning unioned syntax into active proof.
5. A grounded query must report the defining row/source identity by following
   Resolution.sites -> Item.target -> Target.package_id -> verified row mapping;
   keep ordinary syntax queries distinct where source/configuration is unresolved.

Concrete pinned route to execute: actual Source `workspace.rs:11` imports
`purrdf_lex::allocation::Memory`. With the real root lock row and the verified pin,
`resolve_from_package_id_in(source_package, "raf_source::workspace", "Memory",
Namespace::Type)` must select the admitted lex PackageId and actual definition
`crates/lex/src/allocation.rs:367`. A bounded alias/reexport neighbor can import
`purrdf_lex::allocation::Shared as NativeShared`; the pin actually reexports
Shared at `allocation.rs:18`, with definition `allocation/shared.rs:80`. Assert
actual item file authority, manifest/PackageId and locked source/revision; a
displayed path assertion alone is inadequate. No such runtime probe was run here.

Acceptance evidence still required: actual pinned objects and source route;
wrong commit, corrupt object kind/body/link and incomplete/malformed tree refusal;
same-name forged package on another authority refusal; missing/wrong manifest
name/version and alias/reexport valid neighbors; checkout dirty/revision refusal
if that host adapter is implemented; file/work/storage bounds and equal-content
distinct-path identity. Tree admission alone is not the final provider join.
The original cfg, external/standard-library, macro/unsupported-marker, trait and
expanded differential requirements remain open.
