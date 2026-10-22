<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Partial clone history proof review

Bounded read-only source review; no builds or product edits. No concrete defect
found in the new partial-clone proof. This is not assessment or adoption acceptance.

`a_partial_clone_history_can_fetch_git_objects_without_writing_working_files_or_index`
uses the actual ordinary multi-commit `story` repository. The selected historical
blob is obtained from the real `second` commit. Local upload-pack filtering is
explicitly enabled, and filtered/full clones both use `--no-local --no-checkout`:
the partial fixture cannot accidentally reuse the source repository's hardlinked
object store. `read-tree HEAD` establishes an index without working-file checkout.
Known working bytes are then populated directly, preventing fixture checkout from
fetching historical blobs ahead of the caller. The actual `rev-list --missing=print`
receipt requires the selected blob absent only in the filtered clone.

The public `Workplan::history` is called on a real disk-loaded workplan using
`SystemRunner`. It must return four entries and one removed binding. The new
test checks these typed collection sizes, rather than all exact commit/date/source
identities; those detailed history guarantees remain with their existing owners.

Complete before/after snapshots encompass Git metadata and working files.
The shared changed helper includes additions, byte modifications and removals;
it does not filter away inconvenient paths. On the filtered side, at least one
changed path must be under `.git/objects/`, every changed path must be under
`.git/`, and index bytes must be identical. A subsequent missing-object receipt
requires the exact historical blob now present. On the full neighbor the complete
snapshot has no changed path. The after snapshot precedes that final receipt,
so the claimed object-store mutation is attributed to the actual history caller,
not to the verifying receipt.

Both repositories belong to the existing native Scratch/TempDir fixture owner;
no detached server, new global process owner or hook bypass is added. The
Constraints item retains the real partial-clone side effect and immediately
binds this owner alongside the benign full-snapshot history owner. The explicit
AuthorDate::parse code binding includes its parser body rather than relying on
a public type declaration's digest; no date-parser execution was reviewed anew.

The proof is an actual local promisor-remote fetch, not execution against an
arbitrary network server or every Git partial-clone configuration. It establishes
the retained “may fetch” limitation without claiming every clone fetches or
that arbitrary Git hooks/configuration are sandboxed.

Owner reports the corrected selected test 19136 and warnings-denied all-target
Clippy 61525 terminal exit 0. The first compile-only ambiguous Vec assertion
failure remains historical evidence, not an executed pass. Earlier family 27
and CLI 6 preceded the new test and are not represented as rerunning it.
No reviewer execution, full gate, new candidate count or completion claim.
