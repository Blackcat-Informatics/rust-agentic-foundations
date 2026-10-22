<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# CLI invocation, root and typed-output claims

The CLI Contract now names separate invocation, ancestor-read, membership,
explicit-root, command-dispatch, typed-report, machine-refusal/I/O and quiet-result
clauses. Actual source root membership/size assertions are bound alongside the
real CLI callers. Domain judgments remain in the library; the binary's actual
argument parsing and rendering are stated explicitly. Removed dual-policy prose
is corrected. The typed command registry is now explicitly bound by this consumer.

The no-Cargo/configured-Rust-compiler claims use actual executable interception
and receipts. They do not claim that normal Gate loading cannot subsequently
list tracked paths with Git. The old workplan MemoryTree test created an unrelated
FakeRunner and asserted its unused call list was empty. That ineffective assertion
is removed; the useful real MemoryTree membership assertion remains under the
accurate name memory_tree_membership_discovery_needs_no_disk_adapter, with its
existing binding updated. It proves its Tree-based input path, not all process
absence.

Current execution:

- `queue-cli-root-command-owners.log`: check 20 and process 16 passed, then root
  had 5 passes and one stale generated-file expectation. The actual command
  rewrites alpha and creates the index; the fixture now names both actual rewrites
  and retains the exact index-only new-file assertion.
- `queue-cli-root-status-current.log`: all 6 root and 6 status tests passed;
  terminal exit 0.
- `queue-cli-source-root-contract-owners.log`: all 7 source root contract tests
  passed; terminal exit 0.
- `queue-memory-root-actual-guarantee.log`: all 6 workplan root adapter tests
  passed after the unused-fake repair.
- `queue-cli-root-proof-clippy.log`: CLI all-target Clippy passed. The later
  final source/CLI state is checked in queue-cli-root-final-clippy.log.

The root batch queue-adoption-proof-current-root-gates.log returned terminal 101:
30 document-contract tests passed and one failed; all 71 repository gates passed;
self-workplan refused the same three claim-test-unbound findings before reaching
assessment currency. The actual registry test was already referenced by the CLI
claim markers but had not been added to that capability's test bindings. That
binding is repaired. No current stale-assessment count is inferred from this
batch; the older 51-stale snapshot remains unresolved historical evidence.

The coherent candidate under raw/purrdf-adoption-discovery-cli/ returned terminal
exit 0: 149 findings, including 118 uncovered claims, with zero code/test gaps
and zero orphan lines, exceptions or debt. The remaining findings are exactly
30 delivered/open-record state mismatches and one generated-catalogue mismatch
from the in-memory state overlay. This is a delivered-state diagnostic; product states,
assessment acceptance and lifecycle completion are not changed by it. No full
gate or delivery completion is claimed.
