<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Final integration review: default policy and adoption

Verdict: pending actual qualification. This read-only consolidation found no new
execution defect in the reviewed boundaries. It does not certify completion
session 11246, the prospective candidate, the full gate, hooks or CI. No builds,
tests, lifecycle commands or product edits were performed for this review.

The scope is the current frozen combined checkout for issues 16 and 37. Reviewed
the current validation index, requirements capture and authoritative later issue
comments, canonical portfolio queue, prior native RDF, Source, caller, claim,
completion and crate-root reviews, and current production paths. Read
`/home/paudley/stage/agents/docs/RUST_GUIDELINES.md` fully. Executable verification
examples and CLI protocol stdout are intentional callers, not incidental logging.
The guidelines do not justify unrelated restyling or additional dependencies.

## Observed integration

- `model.rs:398` unconditionally includes the strict checks. The CLI has no
  policy selector; searches found old strictness spellings only in rejection
  fixtures. `strict.rs:115` fixes the minimum section length at 20 words. The
  original opt-in/JSON requirements are superseded by the captured default-only
  policy and RDF direction; they are not reasons to restore obsolete paths.
- `cargo-workplan/src/render.rs` admits Text, Turtle and N-Triples. Its machine
  branch builds the native graph from typed command results; serialization,
  output writes and flushes propagate errors. `main.rs` maps inability to answer
  or publish output to exit 2. No JSON output path was found in the scoped
  production search. Prior native projection/storage and actual caller reviews
  remain applicable; the source constructor does not claim a universal storage
  bound, and resident allocation is not claimed.
- `work/coverage/default.md` covers all eight first-party packages with no
  exclusions; `work/debt/default.md` has empty findings and lines. The validation
  index records the current delivered-state diagnostic as zero code gaps, test
  gaps, uncovered claims, orphan lines, excepted lines and debt lines. Its 26
  expected findings are the still-open record/state/catalogue transition, not
  accepted debt. Earlier diagnostic logs with gaps are historical failures.
- `raw/recovery-selectable-assessment-publication.log` contains exactly 113
  current accepted capability/architecture assessment rows. The repository
  self-workplan gate requires both current and accepted, and refuses findings.
  This publication receipt is not an exemption from checking currency after
  lifecycle writes or subsequent changes. The shared gate-root seam selects
  the judged tree; final path-set and byte equality remain required.
- Prior claim/completion review distinguishes static test ownership from actual
  execution. The current validation index records all five complete-policy
  scenarios and all-target warnings-denied Clippy passing in session 41047.
  Negative fixtures now refuse and repair rather than completing invalid
  documents; actual old example files and stale ownership markers were removed
  at their transitions. No generator search, execution of a static claim marker,
  or broad current-tree qualification is inferred from those focused receipts.

## Platform and failure truth

`bound/run.rs` requires every resolved source guard to be proved false using the
actual compiler platform before returning `NotApplicable`; unknown guards or a
live alternative continue through normal execution. Compiler, source, malformed
output and process errors remain failures. Rustdoc has its own listing path and
does not inherit ordinary rustc exclusion proof.

`bound/evidence.rs::BoundOutcome::is_passed` accepts only `Passed`.
`evidence.rs:485` refuses an excluded binding as `BindingNotApplicable` before
publishing a pass record. `lifecycle/complete.rs:407` allows a proved exclusion to
satisfy host applicability while retaining its separate receipt;
`cargo-workplan/src/change.rs:109` emits that receipt and the RDF binding fields
retain its outcome, reason, proof and runs. This is not evidence of execution on
another platform. Current Linux-focused receipts do not establish the remaining
actual macOS/Windows acceptance.

Completion evaluates static obligations before execution, executes the bound
tests/examples, reloads and reevaluates the repository, and compares binding run
keys before publication (`lifecycle/complete.rs:191`). Typed execution/rewrite/
removal errors are not converted to successful refusals. Generated writes precede
record removal. The documented trusted-workspace race boundary remains; the
implementation is not an atomic snapshot or sandbox for child programs.

## Concrete remaining gaps

1. API documentation still has an overbroad side-effect claim:
   `cargo-workplan/src/main.rs:161–179,248` says reads write nothing or leave the
   tree byte-identical, and `lifecycle/complete.rs:165–170` says no other file or
   state changes. Trusted Cargo programs can write and Git can fetch objects.
   The capability Constraints already state the correct publication boundary,
   and the command helper at `change.rs:95` also states it. Scope these API
   comments to tool-owned publication when the owner can safely resume edits;
   this is the previously recorded prose gap, not a new behavior failure.
   Prepared `recovery-publication-scope.patch` without modifying product files;
   `git apply --check` returned exit 0 against the frozen source. Its application
   and any required final qualification remain the parent's responsibility.
2. Validation and queue snapshots contain older in-progress statements below
   newer receipts. Preserve their historical context and put actual terminal
   outcomes first. In particular, rustdoc-focused regression success does not
   close three-platform execution acceptance.
3. Final acceptance still requires actual successful strict and adoption
   lifecycles, current accepted assessments after their writes, exact final
   candidate path/byte equality, default-root `make check`, normal hooks and CI.
   The initial actual strict completion exited 1 and is not replaced by focused
   repair successes. Do not remove live records manually or treat a prospective
   detached-tree diagnostic as lifecycle success.

No finding authorizes weakened ceilings, debt baselines, policy exceptions,
test exclusions, hook bypasses or closure of the separate portability/platform
work. Production changes remain owned by the parent after the frozen execution.
