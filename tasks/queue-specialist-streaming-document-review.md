<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Streaming document metadata patch

Prepared from the current product after the preceding window/story/order patch
was applied. Only ignored stage copies were edited. Patch:
`tasks/queue-specialist-streaming-document.patch`; prepared files:
`scratch/streaming-document-repair/support/streaming.rs` and `scenarios.rs`.
`git apply --check` returns exit 0. No builds, formatting invocation, runtime
tests or product edits were performed.

## Authored document changes

All five Contract versions have explicit bold clause leads and attached owners
for each actual behavior: whole input, chunk invariance/finalization, valid pair
shape, blank or skipped lines, malformed-line diagnostics, persistent failure,
incremental output and the configured/default byte limit as applicable. Owners
match the current version's bindings and actual assertion bodies. In particular,
the now-extended whole-text tests own CRLF stripping, duplicate ordering and empty
values; the extended limit test owns UTF-8 byte counting. This patch relies on
those preceding generated-test assertion edits remaining present.

Usage names actual example bindings for every version. The short version 0 and
version 3 Examples paragraphs now describe the literal example input/output and
its control flow. Other additions explain actual invocation and results, rather
than adding generic length padding. No generator-wide minimum-word filler or
all-tests marker is used.

Constraints state concrete observable boundaries with their actual owners:
line endings, duplicate entries, final unterminated input, source error lines,
sticky errors and byte/default limits. Representation and retained-memory
reasoning remains visibly in Explanation, including the whole-text memory model,
retained entries before incremental output, version 3's unbounded line, and the
bounded partial-line rationale of version 4. These are not claims of measured
allocation behavior.

Pinned historical sentences remain present in their corresponding versions;
clause leads precede them. The exact final contract phrases, decision rationale,
proposal text, WORDING definitions and every historical/direct-account assertion
are unchanged. The version 3 example description does not invent an unterminated
last entry: its actual literal chunks end with a newline.

## Local unfinished-state marker handling

`Step::build` retains its previous account beside the proposal and current source
bindings. The metadata patch removes a marker line only when at least one owner
on that line is absent from the new test/example bindings. It does this solely
while authoring that implementing build document. It neither changes the
production marker reader nor suppresses any finding, and it never assigns an old
skip claim to a new error-policy test. The fully settled and historical delivered
documents retain all their explicit current markers.

The local transformation keeps every non-marker line and every marker whose
owners remain bound. Joint obsolete markers are removed whole rather than
silently claiming the surviving owner proves the entire old statement. No
clean-commit assertion, refusal/history assertion, gate, ceiling, debt or source
coverage rule changes.

## Remaining verification and scope limits

This is a reviewable, unexecuted patch, not scenario acceptance. The owner must
format/compile it, run both complete scenarios and their meaningful negative
controls, and inspect the actual strict findings for authored word counts and
marker attachment. Exact negative refusal sets may need the already identified
stronger-policy findings; none is changed by this patch.

The existing build phase also keeps obsolete example files until settle removes
them. If the sole-default clean gate exposes those files as orphan source during
the intermediate build commit, preserve the full historical/account assertions
and diagnose that exact transition; do not relax Project::commit or coverage.
That potential source-file transition is not changed here because this task was
limited to streaming prose and necessary marker removal.
