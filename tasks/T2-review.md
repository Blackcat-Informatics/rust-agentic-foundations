VERDICT: PASS

Commands (all green): cargo test --test self_workplan --test docs_contract --test repo_gates
(1 + 28 + 57 passed); cargo clippy --tests -q clean; cargo fmt --check clean.

Checks:
- tests/self_workplan.rs: SPDX header, one #[allow(dead_code, reason)] on mod support, Gate::open
  panics on Err, non-empty capabilities guard, check() empty, every list_assessments entry
  current and Accepted. No Instant/sleep, no issue numbers, no milestone/process wording.
- docs_contract.rs: old test removed, Gate import dropped, module doc updated.
- pre-commit hook: line 39 names --test self_workplan; header and "three test binaries" updated.
  pre-merge-commit execs pre-commit, so it is covered.
- AGENTS.md, CONTRIBUTING.md, Makefile header updated; no remaining "two test binaries" text;
  remaining docs_contract mentions are accurate.

Failure format: Gate::assert_clean prints "workplan gate: N finding(s) in <dir>:" then
"  kind file:place message (fix)". The new test prints the same header, then each finding by
Finding's Display ("file: `key`|section|line N: message [kind]; fix"). Same information
(file, location, message, kind, fix), different line layout; the findings are the same set
(try_check = open().workplan().check()). Assessment failures list AssessmentState Display lines
(subject, id, verdict, actor, "stale: ..." inputs), which name the offender. Read from code
only; nothing was mutated.

Non-blocking nits: AGENTS.md line ~142 and CONTRIBUTING.md line ~60 / Makefile line 10 wrap
unevenly after the edit (cosmetic). Report layout differs trivially from assert_clean.
