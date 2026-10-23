<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# PR acceptance inventory

PR 58 is open at f601a80. The candidate is frozen, with no owned local build or
test process. Normal push and signed commit hooks passed. The initial supported
creation command exited 1 after publishing because its response could not be
parsed; the raw remote inventory confirmed one open PR, and subsequent supported
stagectl retrieval passed. This is a successful publication with a separately
preserved helper failure, not a successful initial helper invocation.

Local mandatory coverage and native full-bound lifecycle are complete as stated
in `queue-delivered-base-qualification.md`. The first make check exit 2 and its
native comment-only licence repair remain visible there. Original issues 48 and
53 remain open; no code-occurrence/provider execution or full original Source
contract is claimed.

Hosted run 39978828042 is in progress. Eleven jobs pass: Clippy, formatting,
documentation, deny, cross-check, MSRV, REUSE, release tests, fuzz, Linux and
macOS. Windows remains running and unverified; its current acceptance is
explicitly waived below. Its Source, bound executor and CLI test steps passed;
its full native binding counts are not claimed.
No rerun was requested.
GitHub's required-check query reports no required checks on this branch (exit 1);
this supplies no waiver by itself. The explicit user exception below applies
only to current Windows acceptance; eleven jobs have actual successful evidence.
Raw current checks and native job state: `raw/queue-pr58-checks.txt`,
`raw/queue-pr58-required-checks.{json,stderr,status}` and
`raw/queue-pr58-ci.json`.

Linux's complete native log (`raw/queue-pr58-linux.log`) accounts for all 2,875
bindings: 2,873 passed and two compiler-proved not applicable (the existing
Windows-only disk cases), with no failure/error disposition. The installed GitHub
CLI initially refused terminal escapes in the log; its documented
`--allow-escape-sequences` option preserved the exact raw log on retry. This was
log retrieval only, not a CI rerun.

macOS's complete native log (`raw/queue-pr58-macos.log`) accounts for all 2,875
bindings: 2,861 passed and 14 compiler-proved not applicable, with no failure/error
disposition. Log retrieval passed, exit 0.

Initial complete feedback capture has zero reviews, zero inline comments, zero
PR issue comments and zero GraphQL review threads; the outer cursor is exhausted
and no nested comment page remains. Parent issues 48 and 53 each have one
captured comment. REST captures use pagination and preserve page arrays:
`raw/queue-pr58-{reviews,inline-comments,issue-comments,parent48-comments,parent53-comments}.json`.
Threads and review decision are in `raw/queue-pr58-review-threads.json`.
All feedback was refreshed successfully after the explicit waiver below, with
pagination exhausted. Do not await terminal Windows CI under this direction.
No merge or cleanup is authorized in this monitoring lane.

Explicit user direction: “Please disable CI windows tests on your next PR. For
now, ignore those CI results.” Current PR 58 may proceed without Windows
acceptance. Windows is unverified waived scope, not green evidence. No source or
CI configuration was changed and no GitHub job was cancelled/restarted. Only
the owned local watch was stopped; its exit 143 records that local termination,
not a hosted result. The latest canonical check capture remains pending/exit 1
and is preserved without relabeling. Evidence writers freeze after this capture
for the parent owner's sequential merge preflight.
