<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Narrow sync proof recheck

Independent source review only. Read the current `src/sync.rs`, new
`src/sync/tests.rs`, the changed integration assertions in `tests/sync.rs`, and
the three supplied focused logs. No builds, tests or product edits were run by
this reviewer. No blocking finding in the requested repair delta.

## Publication phase and its production owner

`crates/raf-workplan/src/sync.rs:288-320` retains the actual write sequence:
preflight every target and duplicate, canonicalize the root, call the real
`stage` for every text, discard already staged temporaries on staging error,
then call private `publish(&staged)`. The extracted publication loop is unchanged
in behavior: real `fs::rename`, return the actual first rename error, and discard
the failed and not-yet-published suffix. Successful earlier targets are not
rolled back. This is a production helper, not a test-only replacement path.

`src/sync/tests.rs::a_late_rename_failure_preserves_published_files_and_discards_the_tail`
uses actual files and actual production `stage` and `publish`. All three new
texts exist in temporary files before the second target is replaced by a
nonempty directory. The filesystem therefore produces the refusal after the
first successful rename; no synthetic error, fake source reader or substituted
rename function supplies it. Assertions establish:

- the first target contains its new text;
- the refused second target's sentinel remains unchanged;
- the third target retains its old text;
- every staged temporary pathname is absent;
- a separate all-regular neighbor publishes all three new texts and leaves no
  staged temporary pathname.

This addresses the missing late-publication proof identified in the claim map.
The test enters the private phase boundary deliberately; it does not pretend to
exercise a deterministic concurrent mutation through public `Workplan::write`.
The inspected production caller supplies precisely that staged list after
successful staging. Existing public write tests continue to cover preflight and
staging failure.

`discard` still attempts removal of only this call's temporary files and retains
the first error. Its ignored removal result is preexisting behavior, not cleanup
deleted by extraction. The new test proves successful cleanup in its fixture;
it is not proof that arbitrary filesystem cleanup failures cannot leave litter.

## Permission refusal fixture

`tests/sync.rs::write_changes_no_target_when_a_later_file_cannot_be_written`
no longer returns successfully if its permission fixture is ineffective. It
closes/removes the successfully created probe, restores directory permissions,
then panics with an actionable unprivileged-user requirement. In the intended
failure path it calls the public `Workplan::write`, restores permissions, asserts
the error, unchanged contents of all three targets and exactly one original file
in each directory. The repair prevents a successful test result from claiming a
failure path it did not exercise. It is Unix-scoped; no native cross-platform
execution is claimed.

## Unloaded change neighbor

`tests/sync.rs::a_document_that_does_not_load_stops_the_sync_since_the_tables_would_miss_it`
now mutates the actual valid `work/changes/grow.md` fixture from `type: Change`
to the invalid `type: Chnage`. The underlying valid literal is present in
`tests/support/synced.rs:199-203`. The existing loop calls the real load/sync
helpers and requires exactly `SyncError::Unloaded(["change:grow"])`, with a
nonempty actual check verdict. Existing malformed capability cases remain.
This closes the change-document-specific assertion gap without filtering away
the malformed document or manufacturing a sync error.

## Observed execution evidence and limits

The supplied `raw/queue-sync-publish-refusal.log`,
`raw/queue-sync-write-refusal-current.log` and
`raw/queue-sync-unloaded-neighbors.log` each show the intended exact test running
and one passed, zero failed. The owner reports terminal exit 0 for all three;
the files inspected show Cargo's successful result but do not embed separate
shell exit markers. This reviewer did not rerun them.

The renamed generated-only equivalence test accurately describes its existing
`generated_clean` predicate. This narrow review does not claim the entire sync
contract, current documentation adoption, final parent review or required full
gates complete. The requested source freeze can be released.
