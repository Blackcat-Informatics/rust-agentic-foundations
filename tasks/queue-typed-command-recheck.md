<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Narrow typed-command finding recheck

Read-only source/test/doc recheck of the four findings in
queue-typed-command-review.md. No product edits, builds or tests were performed
by this reviewer. The frozen runtime surface can be released after this report.
This closes the four source findings subject to the separate qualification
limit below; it does not approve default adoption, full gates or census debt.

1. **Diagnostic flush propagation: repaired.** Report::print now flushes
   diagnostics after writing nonquiet notes and propagates its result. It does
   not touch that sink for quiet or no-note output. The new
   diagnostic_flush_failure_propagates_and_quiet_never_touches_that_sink test
   uses a writer that accepts and records all bytes, fails only flush with
   BrokenPipe, and checks the exact note bytes and one flush. It checks all
   three output formats, normal successful-writer neighbors, and quiet's zero
   bytes/zero flushes. The existing write-refusal neighbor remains. The
   inspected queue-render-review-sinks.log reports all three renderer tests
   passing. No new source concern found on this path.
2. **Help/version output failure: repaired.** main propagates Clap error.print,
   then flushes the selected stdout/stderr sink and returns Failed if either
   step fails. It no longer returns the clean help/version exit after a failed
   output. The Linux-only real process test sends stdout to /dev/full, checks
   exit 2 and the actionable command-line-response diagnostic for both help and
   version in Direct and Cargo shapes, then checks a normal nonempty exit-0
   neighbor. queue-help-output-full-device.log reports that test passed. The
   earlier read-only-file stdout attempt did not surface an I/O failure; it is
   not positive failure-path evidence and is not claimed here. This is Linux
   execution evidence, not an assertion that the device exists on other hosts.
3. **Recorded binding outcome: repaired.** recorded_fields now writes the
   actual BoundOutcome::name as outcome for Failed::Binding before preserving
   the actual reason/message. The matching profile table documents this
   field's conditional meaning. The new actual CLI test records a failing
   test, an ignored-test unresolved outcome and a passing test through
   --from-binding. It requires failed versus unresolved on the two failure
   rows, no failed row for pass, and no invented VerificationRun for any
   recorded-evidence result. queue-recorded-binding-fidelity.log reports one
   actual passing selected test. The field source is the domain enum, not
   human-message parsing.
4. **Assessment raw unresolved reason: repaired.** assessment_fields now emits
   unresolved directly from InputChange::Unresolved(reason), separately from
   change and the display message; other variants do not receive it. The
   profile table states this condition. The new CLI test records an actual
   assessment, renames its bound function and checks the raw reason separately
   in Turtle and N-Triples; changing the binding then checks no-longer-input and
   new-input rows have no unresolved field. The pre-existing section-edit
   neighbor now asserts changed also has no unresolved field. The raw reason
   assertion is contains-name plus not-equal-to-message, rather than exact
   equality to a separately computed library reason; source inspection confirms
   the projection performs the direct exact copy. queue-assessment-input-fidelity.log
   reports the selected actual test passed.

## Current qualification gap: new exact registry fixture

At this read, raw/queue-review-fidelity-registry-pins.log ends in a failure of
command_extension_registry_matches_the_documented_terms, at the newly added
recorded-failed-binding golden. Its native Turtle fixture embeds Cargo's
variable elapsed/profile output and a previous temporary target path containing
a process id. The current production registry observe still compares exact
whole native bytes to this fixture. A new invocation need not reproduce those
bytes or the resulting content address. This is a test-fixture determinism
problem; it is not evidence that any of the four runtime repairs is incorrect.

The capture log's producer run completed successfully, and the independent
parser log reports the new assessment and two recorded-binding graphs parsed
in both MIME formats with equal respective graph sizes (27, 28, 28 triples).
Those are useful format checks, but neither proves the current registry's exact
native pin test passes. Do not conflate successful fixture capture with the
later test result. Preserve the failed log and fix the fixture design without
changing production failure text or dropping meaningful outcome/reason checks.
For example, deterministic executor responses can exercise the typed projection
with exact message pins, while the separate real failing-binding CLI test keeps
actual execution semantics covered. The chosen approach must retain the
existing registry/emission acceptance and the distinction between real process
evidence and controlled projection fixtures.

No new full gate result, normal commit/push hook result, CI or campaign
completion is established by this recheck. The owner must collect the actual
terminal result of the corrected final registry qualification and subsequent
required gates.
