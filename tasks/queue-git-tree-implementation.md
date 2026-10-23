<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Immutable checked Git Tree prerequisite

Current amendment: the subsequent shared-meter source-to-actual-PackageId join
and actual pinned native definition queries are implemented and evidenced in
`queue-locked-source-binding.md`. The separate-meter and missing-join observations
below describe this earlier prerequisite checkpoint, not current implementation.
Full original acceptance and dependency qualification remain unfinished.

Implemented the bounded prerequisite authorized after the source-only note
`queue-verified-source-next-join.md`. This is not the package/provider join.

Public APIs reexported by raf_source: `GitObjectId::from_sha1([u8;20])`,
`GitObjectKind::{Commit,Tree,Blob,Tag}`, borrowed `GitObject { kind, body }`,
`GitObjects::read_object(id, max_bytes) -> Result<GitObject<'_>,TreeError>`, and
`VerifiedGitTree::new(&LockedCatalog, LockedPackageId, &dyn GitObjects,
&LoadOptions) -> Result<VerifiedGitTree,LoadError>`. The snapshot implements the
existing Tree seam and exposes its exact checked source and usage. Raw object
IDs/kinds are requests/evidence, not verification claims. Only the checking
constructor creates the opaque snapshot, and only its backend creates the private
FileId Git identity variant. Public key/device/authority constructors cannot forge
that variant or Tree admission.

The constructor checks the complete locked SHA-1 commit and its root tree.
Every accessed directory/blob is checked against its authenticated parent ID;
native PurRDF Sha1 hashes Git's actual kind/length/NUL/body framing. Fallible
maintained Git iterators are exhausted, including trailing decode errors; no
error-filtering lookup helper is used. Complete checked directories govern
absence. Names/modes are validated, duplicates refuse, submodules refuse, symlink
entries can be listed but never traversed, and native admitted parent traversal
stays within real directories. File identity includes commit plus canonical path,
not blob ID; equal contents do not merge distinct module contexts. Hash checking
precedes any Seen callback, and source text preserves exact UTF-8 bytes.

Object bodies are borrowed caller-owned inputs, with the maximum passed before
acquisition and checked before processing. Existing TotalBytes/Work limits apply
before hashing/decoding. Native Memory/ItemAdmission and WorkList own buffer and
walk growth. gix-object's continued extra headers allocate exact source-span
buffers internally; their disjoint spans total no more than the commit body, so
that local bound is admitted before decoding. Its allocator has no native callback;
do not describe its individual allocation failures as native fallible storage.
Each snapshot currently owns its own meter. The integrated locked load must
coordinate snapshot/catalog/walker accounts before claiming one combined global
allowance; separate per-operation grants are not that proof.

Direct decoder dependencies: gix-object =0.64.1, gix-hash ^0.26.2 with sha1.
Production object verification uses native PurRDF hashing; the decoder's hash-ID
crate nevertheless requires a hash feature and pulls sha1-checked. Relative to
HEAD, root Cargo.lock adds 50 package rows, one of which is the already retained
semver prerequisite (49 additional rows for this step). This material closure is
not hidden. Full licensing, deny, MSRV and platform qualification remain required.
Both root and fuzz locks include the closure. Generating the fuzz lock advanced
seven unrelated versions; explicit Cargo updates restored every prior version,
and raw diff now contains no removed version lines. No prior dirty work or stash
was discarded.

Actual receipts in raw:

- Initial offline caller compile refused missing decoder cache material:
  queue-git-tree-callers-initial, exit 101. Normal dependency resolution and real
  `cargo check --workspace --all-targets` passed, session 19540, exit 0:
  queue-git-tree-callers-resolve.{log,status}. This compiled real callers early.
- Final `cargo test -p raf-source --test git_tree --locked --offline` passed all
  six tests, none ignored/filtered, session 80296, exit 0:
  queue-git-tree-final-focused.{log,status}. Covers existing loader integration,
  known Git blob vector, exact bytes, canonical/distinct path identity, wrong
  commit/kind/body, corrupt blob before Seen, missing evidence, complete malformed
  tree/commit refusal, unsafe names/gitlinks/symlinks, exact/one-short Work,
  ItemBytes and TotalBytes, object acquisition limit, and local/SHA-256 refusal
  with a real SHA-1 neighbor. Continued header fixtures exercise owned decoding.
- Final Source all-target Clippy with warnings denied passed sequentially in
  session 80296, exit 0: queue-git-tree-final-clippy.{log,status}. Earlier style
  findings were corrected without waivers. Formatting/diff checks passed.
- Fuzz lock resolution/preservation receipts:
  queue-git-tree-fuzz-lock and queue-git-tree-fuzz-lock-preserve, both exit 0.
  This was metadata resolution, not a fuzz runtime or full gate.

Exact changed files for this step: new src/tree/git.rs and tests/git_tree.rs;
src/tree/mod.rs, src/error.rs, src/meter.rs, src/lib.rs;
crates/raf-source/Cargo.toml, root/fuzz Cargo.lock, and the source-index capability.
No commit, push, PR, full suite, benchmark, service operation, new checkout or
external worktree mutation was performed.

The API can verify the actual pinned PurRDF commit
5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43 from a caller-supplied bounded raw-object
store: its 40-digit Git format is supported and the Tree remains rooted at the
whole committed repository. Acquiring those bodies read-only from the existing
Git cache requires no checkout/service lifecycle. This implementation does not
yet provide that host acquisition adapter, and no actual pinned PurRDF definition
query was run. It does not inspect or certify a dirty ordinary checkout.

Required next production join: retain actual package version/workspace inheritance
through the single manifest path; associate exact checked manifest locations with
their lock name/version/source, compute actual discovered PackageIds inside the
same load, select declarations using their exact owning row, then populate the
existing DependencyInfo/prelude resolver with verified destination PackageIds.
Keep active cfg/features and missing source evidence explicit; no name-only
fallback. The concrete pinned Memory and Shared alias/reexport routes and forged
name/wrong source/refusal neighbors remain mandatory. SHA-256 verification,
ordinary-checkout correspondence and every original issue 48 cfg/macro/external/
differential requirement remain open; Tree admission alone grounds no provider.
