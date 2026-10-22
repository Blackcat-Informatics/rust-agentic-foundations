<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Five scenario failures: policy repair proposal

Read all five failure blocks in `raw/recovery-platform-scenarios-runtime.log`
and the current scenario source. No product edits, builds or runtime tests.
Prepared copy: `scratch/scenario-policy-repair/scenarios.rs`. Metadata patch:
`tasks/queue-specialist-scenario-policy.patch`. `git apply --check` exits 0.
The patch still requires formatting, compilation and the full five-test run.

## Individual causes and repairs

1. **planning_produces_the_documentation** fails at the first precise refusal
   comparison. The built but considered/unbound transient account has 95 orphan
   lines: 56 shipping and 39 test. Two coverage findings are correctly added to
   the prior no-open-record and lifecycle refusals. The patch preserves this
   uncommitted negative and includes both findings in its exact expected shape.
   It also anticipates the next genuinely deficient partly settled account:
   candidate Usage/Examples are thin, and Usage names no current example. Their
   two thin-section and one unbound-usage findings join, rather than replace,
   the existing API/proposal/outcome/assessment failures. These latter findings
   are source-derived and have not yet been observed by a new run.

2. **a_document_that_merely_exists_does_not_satisfy_the_scenario** reaches the
   unresolved-example negative and correctly refuses the entire three-word
   account. The old expected shape omitted seven thin-section, two uncovered
   claim, missing-limitations, unbound-usage and orphan-example findings. The
   patch asserts that full observed shape plus its unresolved binding and
   binding-missing obligation. Empty sections still fail to load with seven
   exact errors and no writes. For the subsequent failing executable negative,
   real rolling/doctest bindings remain alongside the deliberately failing
   target, so its runtime failure is isolated without orphaning another example
   or making Usage unbound. After that refusal the bad file is removed, the
   real account restored/assessed, and actual completion is required.

3. **a_thin_document_is_not_the_account_the_scenario_builds** explicitly calls
   `completed` on a deficient delivered account. The complete policy correctly
   returns thirteen findings. The patch requires that exact refusal with the
   no-write/record-retention helper, retains every independent Account fault
   assertion, and then repairs and completes the genuine neighbor. The obsolete
   hard reset is removed: because the build is now intentionally uncommitted,
   resetting to the candidate HEAD would also erase the implementation before
   the supposedly valid neighbor. A genuine refusal already leaves all bytes
   unchanged, so no reset is needed or appropriate.

4. **several_redesigns_leave_one_readable_present** fails the clean commit of
   strict-errors because the obsolete dump example remains as five uncovered
   lines after its binding disappears. The patch moves actual obsolete-file
   removal from settle to build, alongside the binding/target change. Build
   authors the new version's real Usage and Examples at the same time, avoiding
   dead example links or executable paths. Contract/rationale remain the prior
   unfinished account beside its proposal and retain local obsolete-marker
   handling. The build's precise obligation assertion omits only untouched
   Usage/Examples, since those sections are now genuinely written; all proposal,
   outcome and assessment checks remain. Settle no longer removes the same files
   a second time. Historical files and proposal/account recovery assertions are
   untouched.

5. **each_fault_of_the_account_has_a_document_that_breaks_only_it** fails before
   testing its tense fault because the replaced old Constraints sentence no
   longer exists there. The patch mutates the actual capacity clause from
   present to future tense. Its remaining text, list count, zero-capacity/NaN
   statements and allocation rationale remain intact, so the exact expected
   Account fault stays `tense`. All other precision mutations/assertions remain.

The shared THIN helper encodes only the exact observed three-word account
failures, including multiplicity of seven sections/two claims. It does not read
or filter actual findings to manufacture agreement. Each caller separately adds
its distinct missing-example or unresolved-binding expectation.

## Acceptance preserved and remaining limits

Project::commit, zero ceilings, debt, finding production and lifecycle gates are
unchanged. No source or claim is excluded. The empty/thin/unresolved/failing
examples remain real negatives; repaired accounts must pass their original API,
usage, traceability, limitation, source and history assertions. Streaming still
has five distinct delivered states, real proposal history and actual completion.

This is a coherent unexecuted fixture proposal, not evidence that all downstream
assertions pass. The owner must examine any newly exposed exact failure rather
than weaken a gate or convert these negatives into unchecked setup. In
particular, the partly settled refusal additions and streaming build obligation
timing are inferred from the current production rules and need the runtime run.
