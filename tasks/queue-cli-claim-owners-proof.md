<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# CLI claim ownership checkpoint

Final current result: production sync and the delivered-state candidate returned
terminal exit 0 in session 79704. `queue-adoption-cli-qualified-candidate.log`
records 109 findings and 78 uncovered claims, none in CLI or strict, no code/test
gaps and no orphan lines, exceptions or debt across eight packages. All other
findings are 30 delivered/open-record state mismatches and one generated
catalogue mismatch caused by the read-only overlay. Product states did not move.
The bounded review is `queue-cli-claim-sync-review.md`; its two concrete findings
and quiet-clause refinement were repaired and reread. No whole-delivery or
assessment/lifecycle acceptance follows from this diagnostic.

The first helper invocation failed because its isolated manifest has several
binaries and required `--bin adoption-probe`; the explicit-bin replay then
reported 110/78 with one genuine missing completed-change frontmatter binding.
The existing actual fixture passed, and adding its binding removed that finding
in the final measured 109/78 candidate. Failed commands and intermediate outputs
remain retained under their original raw paths.

Implementation and historical qualification detail follows.

The primary assertion map is `queue-cli-claim-assertion-map.md`. The capability
now separates argument values, evidence argv, replacement metadata, general
exit classes, record view/recording/coverage/lookup exits, output options and
explicit debt writes. Constraints retain the actual read-only and publishing
effects, process construction, slug/record refusals and safe terminal display.
This is current claimed behavior, not a waiver of repository adoption.

The new actual fixture is
`verify::verification_uses_only_tracked_path_git_calls_and_frozen_cargo_without_a_commit`.
It initializes Git and adds files without a commit, runs the real binary through
recording wrappers that forward to actual Git/Cargo, requires a passing binding,
compares both complete Git argv vectors and both frozen Cargo argv vectors,
and compares the entire fixture snapshot. It passed one selected test with
terminal exit 0 in `raw/queue-cli-verify-tracked-git.log`.

Existing inspected assertions retain their ownership: actual record commands
compare output, parsed records and exact changed file sets; evidence views,
history, assessment context/list and coverage reports compare unchanged
snapshots; real completion tests compare the removed record and rewritten
generated blocks. Parser tests compare typed argument variants and malformed
neighbors. Process refusal tests record absence of calls before rejection,
then execute accepted neighbors. Terminal tests compare human escaping with
original RDF values; native escape tests enumerate controls and bidi values.

The no-shell and process-routing claims have distinct support. Actual
`exec::tests::every_argument_is_one_element_and_no_shell_reads_it` verifies
SystemRunner argv using NUL-separated output and absence of an injected file.
Bound test/doc/example plans assert frozen Cargo invocations and hostile-input
refusals. Actual CLI verify and history callers exercise those library paths.
The complete routing claim also requires source inspection of command handlers
and the executor; no single invocation test proves a universal source property.
The current CLI source has no separate process spawn implementation.

Qualification completed as follows:

- `queue-cli-claim-parser-owners.log`: all 24 parser owners passed.
- `queue-cli-record-verify-owners.log`: assessment 6, evidence 14, history 6 and
  verify 10 passed, terminal exit 0.
- `queue-cli-command-effect-owners.log`: check 20, complete 12, impact 7, new 15,
  preview 8, process 16, reconcile 11 and status 6 passed. The aggregate batch
  returned exit 101 at four stale sync fixtures, which remain visible in the log.
- `queue-cli-sync-current-owners.log`: all 11 repaired sync fixtures passed.
  The fixtures now empty the populated real matrix between its actual markers;
  index-only cases start with current capability blocks and remove only the index.
  Exact write sets, refusal snapshots and authored bytes outside the matrix stay
  asserted.
- `queue-cli-frozen-bound-owners.log`: all 70 bound-plan/runtime owners passed.
- `queue-cli-native-argv-owner.log`: the actual no-shell/argv test passed.
- `queue-cli-claim-owners-clippy.log` and `queue-cli-claim-owners-sync.log`:
  all-target CLI Clippy and production sync passed. Session 46548 returned exit 0.
- Review found missing explicit no-SPDX proof on assessment replacement and an
  obsolete verify source comment. Both are repaired. The actual final replacement
  asserts empty quiet sinks, exact one-file change, headerless Assessment prefix
  and no SPDX text. `queue-cli-replacement-header-neighbor.log` passed the exact
  owner and `queue-cli-review-current-clippy.log` passed Clippy, terminal exit 0
  in session 18041. Quiet caller outcomes are now a separate meaningful clause
  with all eight actual handler-family owners.

Session 20640 is refreshing production sync and the delivered-state candidate;
no candidate count is claimed before it terminates. All acceptance for
assessment currency, repository zero debt, lifecycle and full gates remains
open. The preceding measured candidate was 124 findings / 92 uncovered claims;
no new count is inferred from markers.
