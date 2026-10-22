<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Typed command runtime review

Independent read-only source review of the frozen runtime surface: all
cargo-workplan production adapters, result.rs, result_graph.rs, render.rs,
main.rs and exit.rs; shared raf_workplan::report projections; relevant domain
structures and selected actual-caller/registry/sink test bodies. No product
edits, compilation, command execution or tests were performed. The author has
not established default-policy/adoption completion or delivered census debt.

## Actionable I/O findings

1. **Diagnostic flush failure can still report success.** `render.rs:119–126`
   flushes stdout, writes notes using `?`, then returns Ok without flushing
   diagnostics. A Write implementation that buffers successfully and fails on
   flush can lose its notes while Report::print returns Ok and run returns
   Clean. This differs from the explicit stdout flush and the comment promising
   failures of either sink propagate. Add diagnostic flush after notes are
   written, propagate the result, and retain a successful neighbor. A focused
   test writer should accept write(), fail only flush() with BrokenPipe, and
   verify propagation. Also check quiet output does not write/flush omitted
   notes if that is the intended quiet sink policy. The existing
   diagnostic_write_failure_is_propagated_and_quiet_does_not_write_notes test
   only fails write(); it does not cover this case. The final error diagnostic
   in main may itself fail, but the resulting exit must remain 2.
2. **Help/version output failures can exit 0.** `main.rs:280–287` discards
   `error.print()` and then returns Clap's help/version exit code 0. A failed
   stdout write therefore contradicts the documented output-failure exit 2
   behavior. Handle the print error before returning the requested exit code.
   This is an existing touched-path gap, not a claim that the typed graph
   migration introduced it. Usage errors already return 2, so a second failed
   diagnostic there cannot manufacture success. `fail()` deliberately ignores
   its own diagnostic write after selecting Failed; that is not the same bug.

## Domain-fidelity findings for disposition

These are concrete reductions of actual typed domain meaning. The current
profile table describes the reduced representation, so they are not claims
that the implementation disagrees with that table. They should be resolved
against the intended machine contract before claiming full semantic typed
projection.

3. **Recorded binding failures lose their outcome variant.**
   `result_graph.rs:239–243` projects
   evidence::Failed::Binding(BoundOutcome) only as reason `binding` and an
   optional display-derived message. The real BoundOutcome distinguishes
   Failed from Unresolved, while evidence.rs:503 constructs the recorded failure
   from that exact outcome. Both paths become the same machine discriminator;
   consumers must inspect human reason text to tell execution failure from
   planning/refusal. Verification Binding rows already carry outcome().name().
   Add the binding outcome to this failure's typed fields and keep its actual
   reason separately. Add real record-from-binding neighbors for execution
   failure and unresolved planning; neither should invent a VerificationRun.
   Existing evidence-recorded RDF test primarily covers an over-limit command,
   and the inspected verify test covers the separate Binding result class.
4. **Stale assessment unresolved reasons exist only in a display message.**
   `result_graph.rs:770–785` maps InputChange::Unresolved(String) to change
   `unresolved`, then writes MovedInput::to_string(). It does not expose the
   underlying reason as its own property. In contrast, CurrentInput and
   ContextSymbol preserve unresolved strings directly. The assessment view
   promises unresolved-input reasons and should expose this value without
   requiring consumers to remove the display prefix/parse prose. Retain change
   and message for compatibility within this unsettled profile, add the raw
   reason field only for Unresolved, and test changed/no-longer/new neighbors
   have it absent. A stale assessment after a formerly resolved symbol or
   evidence record disappears is an actual production path, not hypothetical
   fabricated data.

## Scoped source checks

- The production dispatch calls actual library APIs and creates a closed
  CommandResult for every command family. There is no remaining generic JSON
  object/array result or independent machine serialization passed by adapters.
  Human text is separately retained for established terminal rendering; the
  graph projection reads the domain values, not that text (apart from explicit
  message fields and the reductions noted above).
- Report::print filters visibility before projection, preserving original
  Record class for exit selection. Findings remain under quiet, outcomes are
  removed, and machine mode still builds a Report envelope when none remain.
  Coverage is classified as a whole, so quiet does not remove non-gap rows of
  a finding report. Assessment record/list/context and history remain outcomes
  even when their domain verdict/content is rejected/stale; evidence-recorded
  fail and failed verification bindings remain findings.
- Graph construction and canonical rendering finish before stdout.write_all.
  A domain/projection/rendering refusal cannot leak an earlier graph fragment
  from this function. An actual output I/O failure can leave a physical partial
  write; no stream writer can roll back a pipe. Keep that distinction from the
  documented no-partial-domain-result behavior.
- The shared coverage projection is the sole implementation used for the
  actual Coverage result. It preserves source locations as named reifiers of
  the observed item relation. An unresolved reexport reports the declaration's
  file/line; it does not claim a resolved Definition or successful test run.
  Surface failures retain scope, target/root, failure name and carry-budget path
  only when present. Tests direction intentionally omits code-surface errors.
- Finding projection validates registered kinds, preserves Document versus
  Key/Section/positive Line variants and refuses line zero. It does not invent
  a location for Document. Public finding_into documents discard-on-error;
  command graph assembly does discard its local graph on an error.
- Result and child nodes have deterministic owned IDs and explicit ordinal
  values. Ordered strings, inputs, measures, sources, ranges, arguments, runs,
  failures and nested reference rows use enumerate order; a BTreeSet triple
  serializer cannot erase that order because ordinals carry it. Raw decimal
  measurements remain exact strings, counts are integer literals, exit codes
  use signed xsd:integer, and absent proof counts remain absent. No zero counts
  or successful invocations are synthesized when the executor did not supply
  them.
- Binding result uses actual evidence.runs(), argv, Ended and Proof. Unresolved
  bindings have no invented run. Program kind is the domain program kind, not
  an asserted resolved executable or artifact identity. The projection does
  not introduce an unearned execution provenance assertion.
- Assessment context retains its actual authored text, definition file/text,
  named bit, unresolved state, evidence values/limits and current input
  digests. Actual Definition class here is appropriate: these are resolved
  definition texts, unlike coverage's declaration/source observation.
- History projection retains full commit IDs and author-date spelling,
  subject, actual source/path order, removed identifier/directories and the
  found/no-commit/no-crate variant. lastResolved is absent when absent; no
  parent is invented.
- New change, impact, obligations, completion/refusal, status/open changes,
  previews, rewritten paths, claim coverage, line coverage and debt-written
  fields follow the current profile table. Repeated completion failure text
  suppression does not erase its typed Refusal row; repeated is explicit.
  Line-counting error domain reports contain no partial rows, and the graph
  does not replace them with empty success. DebtWritten states what the write
  returned; it does not claim first-party census debt has been eliminated.
- Error branches for unknown nonexhaustive nested variants generally return
  explicit projection errors; they do not emit a guessed partial value. Actual
  domain types are fixed within this workspace build. No new compatibility
  layer is proposed for hypothetical future variants.

## Test/source evidence and execution limits

Inspected render tests check diagnostic write failure, quiet suppression,
unchanged exit classification, graph equivalence across both carriers and the
empty clean envelope. They do not cover diagnostic flush or failed help output.

The actual command registry test runs command fixtures, compares exact native
Turtle/N-Triples bytes and sorted independent graph pins, recursively scans
application IRIs including triple-term interiors, requires no duplicate
COMMAND_TERMS, disjoint base/extension sets, exact documented equality and
every extension term actually emitted by its corpus. This repairs the earlier
one-direction vocabulary concern. Its LinesError term is collected from a
pinned fixture; a separate owner-module test creates an actual reach-budget
refusal and compares that projection to the same fixture with no partial rows.

Selected actual-caller tests inspect findings' four location variants in both
formats, status nesting/absent binding gaps, verification proof/real invocation
rows/unresolved and quiet results, evidence decimal/field order and history
full IDs/sources/paths/removal. The history test helper sorts by explicit
ordinals; this is proper sequence recovery, not accidental lexical subject
order. Golden/oracle pins are byte/graph assertions; this review did not run
the external parser that originally generated those pins.

No tests were executed by this reviewer. Full make check, mandatory normal
hooks, current lifecycle/assessment completion, CI, final integration review,
default adoption and first-party census-debt acceptance remain outstanding
unless independently recorded elsewhere by the owner. The source freeze may
be released to address these findings; this review grants no commit or merge
authority and supplies no acceptance waiver.
