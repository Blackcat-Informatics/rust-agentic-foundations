<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Committed Git host source identity

Implemented the requirement-based replacement described in
`queue-committed-git-host-design.md`. Installed Git is the trusted committed-object
decoder. The raw-object/Merkle subsystem was an implementation choice, not an
original requirement. Its source, tests, public API diff and both resolved locks
are preserved under `tasks/raw-object-prototype`; all earlier raw-object proof
sources/logs remain intact. There is one production source-identity path.

The public constructor is now
`CommittedGitTree::open(&LockedCatalog, LockedPackageId, repository: impl AsRef<Path>,
&LoadOptions) -> Result<CommittedGitTree, LoadError>`. Its fields are private.
`GitPackageRoots::new` consumes this sealed host Tree instead of a generic Tree
or caller verification label. The original `LockedManifest`, `LockedEdge`,
`Workspace::load_locked_sources`, actual manifest/version/root-workspace checks,
requested outgoing selection and actual PackageId association are retained.
The old raw-object types/constructor and decoder error path are removed from
product, not kept behind an alternative feature.

The adapter resolves the exact full locked SHA-1 ID as that commit, collects
one complete recursive tree listing with explicit directory records, and reads
blobs by full object ID. Git owns actual object grammar. RAF strictly validates
its bounded NUL-delimited output, mode/type/size/full-ID fields, exact paths,
parents, uniqueness and UTF-8. Gitlinks refuse; symlinks are listable but never
traversed. File identities remain commit plus canonical path. Blob size is
checked before acquisition, exact returned size is checked afterward, and native
PurRDF SHA-1 verifies blob framing/bytes before text or Seen is returned. Missing
objects cannot become absent paths or Seen. Inside-root parent spelling works;
non-directory/symlink traversal and above-root paths refuse.

Fixed direct Commands clear inherited Git environment/config overrides, disable
system/global config, replacements, lazy fetching and optional locks, deny
transport protocols, and use no shell, checkout reads, textconv or filters.
Git must support the required flags. A bounded merged output/diagnostic pipe
avoids multi-pipe deadlock and unbounded output capture. Native storage admission
precedes buffer growth; total bytes and processing work are charged to the same
original meter used by catalog, source, discovery and index. On refusal only the
owned child is stopped/reaped. Typed GitIo/GitExit/GitCleanup retain operating
system causes, child status and primary/cleanup failures. A failed command or
malformed/truncated output is a hard error, never source absence.

This is a host contract. Installed Git and its local object store are trusted;
its own internal CPU, allocations and object-store access are not RAF-native
admitted work and are not DiskTree's handle containment. Unsupported process
hosts explicitly refuse construction; that branch has not been cross-compiled
here. A dirty checkout does not affect committed contents and is not certified
clean. No repository-authorship, hostile object-store race, registry-archive,
active cfg/feature or preferred-provider authority claim is made.

## Actual evidence

All receipts below have raw `.log` and `.status` files; statuses are actual,
not inferred from compressed output.

- `queue-committed-git-callers-fixed`: actual workspace library callers compile,
  session 57380, exit 0. First compile found/fixed the native hex API's str input.
  Initial all-target follow-up failed solely on a removed unused fixture import;
  affected all-target Source compilation is covered by the passing Clippy below.
- `queue-committed-git-focused-indexed`: session 18792, exit 0; all six host Git
  cases and all seven actual package-binding cases pass. The latter retain exact
  combined and one-short Work/ItemBytes/TotalBytes grants, duplicate-name distinct
  sources, actual alias/reexport target IDs, forged manifest/name/version/source,
  nested same-name/version locator and missing/optional/unrequested edge neighbors.
- `queue-committed-git-protocol`: session 90323, exit 0; both narrow unit cases
  pass. They assert strict malformed/truncated/unsupported/invalid-path/UTF-8
  protocol refusal, valid complete/empty neighbors and exact/one-short acquisition
  bounds before buffer growth. Other library tests were not run.
- `queue-committed-git-substituted-blob`: session 88810, exit 0; the strengthened
  one-case test proves a different valid compressed blob substituted into the
  owned object's location refuses before Seen, then proves missing-object refusal.
  The initial overwrite hit Git loose-file read-only permissions; removing only
  that owned file before substitution fixed the fixture. The earlier full target
  failure and this exact corrected result are both preserved.
- `queue-committed-purrdf-route`: session 47148, exit 0. Actual reading-package
  queries resolve Memory to `purrdf_lex::allocation::Memory` at
  `crates/lex/src/allocation.rs:367` and Shared through its reexport to
  `purrdf_lex::allocation::shared::Shared` at `allocation/shared.rs:80`. Every
  actual site targets the admitted purrdf-lex 3.0.1 PackageId, authority 1 and
  exact locked revision `5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43`.
- `queue-committed-git-final-clippy`: session 55690, exit 0, Source all-target
  warnings-denied Clippy. The final owned-file removal statement in the fixture
  has its own affected-target lint receipt `queue-committed-git-fixture-clippy`,
  session 9729, exit 0.
- `queue-committed-git-fuzz-lock`: offline workspace-only lock pruning, exit 0.
  Both root and fuzz locks remove all 49 decoder-added package rows and retain
  unrelated versions. Relative to HEAD both now add only the earlier three
  native Source dependency edges, semver edge and semver 1.0.28 package row.
- `queue-committed-git-deny`: unchanged offline make deny, exit 0:
  `advisories ok, bans ok, licenses ok, sources ok`. No exception/ban changed.
  Earlier make licenses passed all 68 repo_gates tests; it was not replayed merely
  for this status. Full make check/MSRV/platform/runtime acceptance was not run.

The worktree-edit fixture populates only its test-owned index/files with
read-tree/checkout-index, changes source bytes, adds an untracked file and creates
a replacement ref; committed reads retain the intended original bytes. An initial
attempt to update an unsigned fixture branch was refused by the normal signature
hook. No hook or signature policy was bypassed. The missing-Git test uses an
isolated child test environment, without mutating the parent's PATH.

## Consumer inputs and limits

Exact executable route source is retained in `tasks/queue-committed-purrdf-route.rs`;
the temporary example is removed from product. Inputs are the actual root lock,
first-party DiskTree, existing cached native repository, local raf-source manifest
requesting dependencies.purrdf-lex, verified committed crates/lex manifest
requesting dependencies.purrdf-hash and crates/hash manifest. No clone, fetch,
cache/worktree edit or host service operation was performed for this route.

The consumer must inspect EVERY resolved site through Item.target → Target's
PackageId → Workspace::locked_binding and compare exact configured eligible row/
source identity. A displayed crate path or authority number alone is insufficient.
The actual route used unchanged defaults and charged 10,737,049 total bytes,
41,014,644 item bytes and 202,813,959 work units. This is a bounded execution
receipt, not a performance qualification; its temporary example was present
during first-party indexing. No ceilings were raised.

Changed production files for this replacement: tree/git.rs and its new
git/command.rs, tree/mod.rs, tree/disk.rs's host-boundary comment, error.rs, lib.rs,
workspace/locked.rs's concrete snapshot type, Source Cargo.toml, root/fuzz locks
and the source-index capability. Focused fixtures are git_tree.rs,
locked_sources.rs and support/git_objects.rs. All unrelated dirty work and stash
remain preserved. No commit/push/PR/full suite. Original declared configuration,
external/std, macro/unsupported markers, differential coverage and actual
preferred-provider enforcement remain unfinished; this is one grounded prerequisite.
