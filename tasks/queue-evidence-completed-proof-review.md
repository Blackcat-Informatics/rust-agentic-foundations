<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Evidence boundary proof slice review

Bounded read-only review. `git check-ignore` confirms this exact stage artifact
is ignored. No builds, product edits, timing runs or Stage panel. Reviewed the
actual current helpers/callers, deterministic fixtures and Constraints markers;
no concrete defect found in this slice. This does not complete evidence policy
adoption, assessments, lifecycle or repository acceptance.

## Publication limitation

`files::create_new` still performs actual `target` preflight, then calls private
`publish_new` with its real result. The extracted body retains existing-target
refusal, real directory creation, exclusive open, complete write/sync and failure
cleanup. `record::replace` still validates rendered record and regular-file
preflight, then calls private `publish_replacement`; its root canonicalization,
actual staging/rename and failure cleanup remain on that production path.
No bypass, alternate writer or weakened public preflight was introduced.

Both bound tests obtain real preflight state, swap an in-root parent directory
to an internal symlink, then invoke exactly the remaining production phase.
They check actual alternate bytes and preservation of the original location.
Public writer calls then refuse the already-present link, while restored real
directories succeed. Exclusive creation also refuses an existing file without
overwriting its bytes. This is genuine deterministic proof of the documented
post-check path limitation, not an invented source failure or mocked race.

Replacement uses a Coverage record to exercise the shared publication kernel;
it does not pretend to have raced a complete evidence subprocess. That is an
appropriate exact owner because evidence publication uses those same writers.
The limitation stays in Constraints with those precise marker paths.

## Directory blockers

`evidence::tests::a_regular_file_on_the_way_to_the_record_refuses_before_any_command_runs`
uses the actual loaded Workplan/public record_evidence caller for work and
work/evidence, Create and Replace. It checks exact RecordError::Write path,
zero FakeRunner calls and unchanged snapshots. Real-directory restoration
then admits one actual runner request and replaces/creates real record bytes.
This owns preflight ordering and publication, not OS execution supervision.
Existing internal-link/Gate document-preservation tests remain bound beside it;
the shared target_fits call precedes both command and binding branches in source.

## Supervisor lifetime and group escape

The CLI supervisor test uses support::command, whose binary is the actual
CARGO_BIN_EXE_cargo-workplan, rather than a mocked command adapter. Its trusted
command is a re-execution of the test executable, held by owned FIFOs. PID
receipts are bounded; pidfds retain exact process identity. Assertions prove
the CLI and command each lead separate groups. SIGINT and SIGHUP target only
the owned supervisor group; actual supervisor wait status reports that signal.
The command is still alive, then its explicit release writes a continuation
marker and ends. Exact repository snapshots and record absence show no actor
publishes after supervisor death. The uninterrupted neighbor writes exactly
one record, with actual pass verdict and peak value.

Supervisor and command guards stop owned processes on unwind; pidfd termination
observations do not rely on PID reuse. Control files live in native TempDir
owners and FIFO handles are scoped with the tests. No host duration is measured
or asserted. The test injects the same ordinary OS group signals that matter to
the limitation; it does not claim an interactive terminal generated those signals.

The escaped-group test runs actual SystemRunner. Each re-executed helper calls
real setsid or setpgid, asserts its new process group and acknowledges before the
controlled clock can reach the limit. The original leader exits; its inherited
pipe holder keeps the run live until the deterministic timeout. The escaped
pidfd remains live in its distinct group, and no marker exists while held.
Explicit release then produces the marker and termination; original leader
reaping is separately asserted. Seen's cleanup sends SIGKILL by retained pidfds
on scope exit, including assertion failures. Existing actual same-group cleanup
tests remain the adjacent contrasting behavior.

These are Linux-executed fixtures. They do not claim to reap an orphaned escaped
descendant, execute non-Unix killing, or prove unbounded real elapsed lifetime.
The non-Unix process-only limitation remains in Constraints without a false
tested-by owner. Unix source uses process_group(0) and group kill; the new markers
do not grant platform execution evidence beyond the actual Linux tests.

## Ownership and execution limits

Each added Constraints marker immediately follows its real guarantee/limitation
item. Supervisor lifetime and descendant escape have separate owners. The path
swap limitation was retained rather than moved away from coverage. Linux-only
rustix features are development dependencies; there is no production platform
dependency widening. The native TempDir provider owns fixture directories; no
new generic temporary-directory or process/FIFO kernel was copied.

Read existing logs showing the supervisor and two-method escape focused tests
passed. The owner also reports publication, regular-file preflight, actual
caller and warnings-denied Clippy terminal passes. This reviewer ran none of
those commands and does not infer terminal success for the active family/docs/
locked-metadata batch. Remaining unmarked evidence constraints and broader
acceptance stay explicitly unfinished.

Subsequent owner receipt: batch 41127 returned terminal exit 0 for system runner
23, evidence units 21, CLI evidence 15, production docs sync and locked offline
root/fuzz metadata. Named logs are queue-evidence-process-family.log,
queue-evidence-preflight-family.log, queue-evidence-cli-family.log and
queue-evidence-process-docs-sync.log. The owner also reports clean diff --check.
This updates execution state without broadening the proof or acceptance scope.

## Direct-child shared-path delta

Read-only source recheck of the subsequent portable helper/test. No concrete
defect found. Production terminate calls terminate_process after its optional
Unix group signal. The helper attempts both kill and wait, preserves signal
error precedence and now propagates wait failure when signalling succeeded.
The actual descendant fixture acknowledges readiness through inherited pipes,
then calls that production helper to kill/reap the leader. Only afterward does
the parent release the descendant and observe actual continuation bytes and
output closure. Caught assertion unwind drops the release pipe for EOF, closes
captured outputs and kills/reaps the owned leader before resuming the failure.
No clock or wait duration decides ordering.

This owns the shared direct-child behavior on the executed Linux platform;
it is not native Windows SystemRunner execution. Owner reports selected proof,
all 24 system runner units and warnings-denied all-target Clippy passed. This
reviewer ran none of those checks. The source/marker must retain that precise
execution boundary and broader acceptance remains separate.
