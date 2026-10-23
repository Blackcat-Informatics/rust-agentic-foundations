<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Trusted committed Git host boundary

Implemented checkpoint: `queue-committed-git-implementation.md` records the
actual current API, executed focused results, native route and unchanged deny
PASS. The proposal below is preserved as its requirement-based design record.

Source-only reassessment, no implementation or executed acceptance. The captured
issue and its ordering correction require trusted rooted Trees, exact locked
source/package identities, actual symbol resolution and actionable incomplete
evidence. They do not require independently authenticated raw Git objects,
signature verification or a Merkle witness API. The raw-object decoder was our
implementation choice and currently fails duplicate bans. Its work and receipts
must be retained, but are not a reason to enlarge the requested trust contract.

A sealed `CommittedGitTree::open(catalog, row, repository, options)` can satisfy
the host-source prerequisite using installed Git as the trusted object reader.
Only that constructor creates the type; the existing `GitPackageRoots` consumes
it directly. No generic Tree, caller authority, claimed revision string or
public FileId can fabricate its admission. Keep exact source metadata borrowed
from the selected catalog row, not supplied as an independent successful label.
An exact complete locked revision must resolve to that actual commit object,
with a real root tree, in the supplied repository. Do not accept a branch,
abbreviated ID, replacement object or mutable worktree content as the snapshot.

Use fixed direct Command arguments, never shell text. Require installed Git
support for replacement disabling, optional-lock disabling and lazy-fetch
disabling; unsupported options fail actionably instead of silently falling back.
Clear inherited Git directory/object/config/namespace overrides and explicit
network/proxy/credential hooks from the child environment. Supply a known Git
executable and the selected repository explicitly. Read immutable object IDs
with `cat-file` and enumerate exact tree objects with NUL-delimited `ls-tree`;
do not use checkout file reads, textconv, filters, revision ranges or pathspec
matching. Git decodes actual commit/tree grammar. RAF parses only the documented
command-output protocol, not another binary Git object grammar.

The Tree remains rooted at the whole committed repository. Resolve inside-root
paths against complete committed directory listings. Strictly require expected
mode/type/full-ID/size fields, exact record termination, UTF-8 names and no
duplicate or invalid names. Preserve symlink entry kind but refuse traversal;
refuse gitlinks. A missing path in a successful complete listing is absence;
missing/corrupt objects, failed Git commands, malformed/truncated protocol and
unsupported records are hard errors. Do not turn a child error into NotFound.
Exact blob bytes retain CRLF and UTF-8 semantics. Private FileId remains
snapshot plus canonical path, so distinct equal-content files are distinct and
inside-root parent spellings share identity. Seen cannot bypass initial reads.

Bound the protocol before collection: preflight each object's kind/size using
cat-file metadata, refuse oversized blobs before reading, then read at most the
admitted size plus a sentinel. Directory enumeration must also be streamed into
native admitted buffers with a hard byte/work cap and full termination check.
Do not use unbounded Command::output or wait_with_output. One owned child per
bounded request avoids the previous multi-request pipe deadlock. A shared
bounded stdout/stderr pipe is one thread-free host option; any unexpected
diagnostic on successful binary protocol must refuse rather than become source.
On output overflow or read failure, stop and reap only the owned child. Preserve
spawn/read/wait io causes and nonzero exit status with bounded diagnostics.
No swallowed cleanup failure or orphan child is a successful operation.

Existing native SHA-1 blob framing checks can remain as a small exact-byte
integrity check, without a binary decoder dependency. They are not a signature,
repository-authorship proof or a new raw-object API requirement. The essential
trust boundary is installed Git's committed-object interpretation. This does
not claim adversarial Git/object-store race protection or DiskTree's filesystem
handle containment for Git's own metadata access. RAF acquisition/storage/work
limits cover its own buffers and processed protocol; Git's internal decompression,
CPU and allocations are trusted host work, not retrospectively native-admitted.
Document that limit rather than claiming a portable subprocess resource bound.

Preserve the current production join: shared Meter for snapshot operations,
discovery and index; actual manifest name/version/workspace inheritance;
unique root-package/workspace locator; exact declaration-qualified outgoing row;
actual destination PackageId; path-directory agreement; no name fallback.
Consumer proof remains Resolution.sites → Item.target → Target.package_id →
Workspace::locked_binding. The actual pinned Memory/Shared route inputs are
unchanged. Replace the raw-object-backed concrete type at this seam, not the
resolver. Preserve the old files/diffs and fixtures in the exact stage before
removing gix-object/gix-hash and their exclusive root/fuzz lock closure. Avoid
shipping two alternative production source-authority paths.

Host-only admission must be explicit. On targets without process/repository
access, the constructor fails with a typed unsupported-host error; portable
semantic Tree/index interfaces remain separate. This is not portability
acceptance for issues 43, 44 or 46, and does not evaluate active cfg/features,
provide registry archive identity, inspect dirty checkouts or enforce preferred
providers. A dirty checkout does not influence this committed Tree; it is not
certified clean. Missing installed Git is a required dependency failure.

Focused implementation proof would retain actual pin/alias target identity,
wrong revision/type/missing repository/object refusals with valid neighbors,
strict malformed/truncated directory/blob and child-error refusal, oversized
acquisition, exact bytes/path identity, forged same-name manifest refusal and
original shared resource boundaries. Replace raw-object corruption injection
with actual corrupt-object or controlled command-protocol failure witnesses
appropriate to the trusted host boundary. Rerun unchanged deny after removing
the decoder closure; do not infer success from fewer dependency rows. No such
checks ran for this proposal, and full original issue acceptance remains open.
