<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native scalar and calendar review handoff

This is a bounded source checkpoint, not delivered-state adoption, lifecycle,
full-gate, commit or merge acceptance. Production source is stable for primary
review; subsequent claim ownership work remains distinct.

## Scalar review actions

`Number::exact()` borrows the native Decimal retained at construction. Actual
line-ceiling products use that reference, and integer counts use native
Integer::from(u128). No validated Number text is reparsed in those products.
Comparison rustdoc now states native value order. Census scalar reading uses
native integer and Boolean readers, then preserves its nonnegative, canonical
lexical and u128 width laws. Canonical u128 text is limited to 39 bytes before
the additional arbitrary-precision XSD reading; this is after RDF parsing and
is not a preparse budget on the native RDF parser. Native full-width integer
canonical output removes the previous signed-constructor branch. Root and fuzz
locks include the explicit census XSD dependency.

Actual terminal results:

- All 26 census report tests passed in
  `raw/queue-purrdf-native-scalar-readers-final.log`, including both formats,
  ordinary valid reports, noncanonical integer/Boolean refusals, the 40-digit
  refusal, u128::MAX and its overflow neighbor, and unchanged goldens.
- All 8 line-ceiling owner tests and 3 Number doctests passed in
  `raw/queue-purrdf-retained-number-ceilings.log` and
  `raw/queue-purrdf-retained-number-doc.log`.
- Affected four-package all-target Clippy passed with exit 0 in
  `raw/queue-purrdf-retained-scalar-clippy-2.log`.
- The current scalar census-report-read sanitizer replay completed all 5,000
  cases with terminal exit 0 in `raw/queue-purrdf-final-scalar-fuzz.log`.
  Its exact command is `raw/queue-purrdf-final-scalar-fuzz-command.txt` and its
  generated corpus is isolated under `raw/purrdf-final-scalar-corpus/`.
  Calendar changes began only after that terminal result. This is not a
  sanitizer claim for the later timestamp/date paths.

## Calendar source files

- `crates/raf-okf/src/profile/timestamp.rs`: native RFC parsing replaces the
  private digit/separator/range parser and copied Gregorian month calculation.
  RAF retains uppercase T/Z, required zone, no leap seconds, and its typed
  Malformed versus named OutOfRange diagnostics. Native day-of-month is mapped
  to the existing RAF day field. Native leap-second errors map to second.
- `crates/raf-workplan/src/exec/pattern.rs`: native full-date parsing with RAF's
  1970–2099 admission window replaces digit/calendar decoding.
- `crates/raf-workplan/src/exec.rs`: obsolete month-helper reexport removed.
- `crates/raf-workplan/src/git/log.rs`: native RFC parsing supplies AuthorDate
  validation and UTC normalization; native full-date/calendar APIs supply the
  start-of-day conversion. The copied days-from-civil kernel is removed.
  Git's closed uppercase, no-fraction, seconds-00-through-59 subset is explicit.
  Arbitrary leap-second epoch compatibility is not claimed. This removes the
  old accidental acceptance of second 60 at arbitrary minutes.
- `crates/raf-okf/tests/schema.rs`: actual no-leap timestamp refusals at UTC and
  a numeric offset have adjacent 59-second valid neighbors. Existing field
  diagnostics and lexical refusals remain.

Related authored contract changes are in
`docs/capabilities/okf-profile.md`,
`docs/capabilities/workplan-entity-history.md` and `CHANGELOG.md`.
These files also contain separately described adoption claim edits; their full
dirty diffs are not represented as calendar-only deltas.

## Actual calendar execution

Every successful result below has a collected terminal exit 0:

| Scope | Result | Raw log |
| --- | --- | --- |
| Actual OKF/workplan/CLI all-target callers | compile pass | `queue-purrdf-calendar-callers.log` |
| OKF timestamp/schema matrix | 6 tests pass | `queue-purrdf-calendar-schema.log` |
| Actual Profile validator | 37 tests pass | `queue-purrdf-calendar-profile-caller.log` |
| Git log/date units | 9 tests pass | `queue-purrdf-calendar-git.log` |
| Public execution pattern/newtype owners | 7 tests pass | `queue-purrdf-calendar-patterns.log` |
| Real/scripted history integration | 24 tests pass | `queue-purrdf-calendar-history-callers.log` |
| Affected three-package all-target Clippy | pass | `queue-purrdf-calendar-clippy.log` |
| Production documentation sync | pass | `queue-purrdf-calendar-docs-sync.log` |

The added nested-workspace/diff.relative test and both-format CLI history field
proofs are adoption tests, not calendar parser implementations. Their actual
passing focused logs are `queue-adoption-history-nested-paths.log` and
`queue-adoption-history-rdf-fields.log`.

Current assessment currency and delivered-state claim coverage remain open.
Earlier implementing-state check success does not discharge those obligations.
No baseline, exclusion, policy opt-out, native-clock assertion or fallback
parser is introduced. No full suite, commit or push has been started.
