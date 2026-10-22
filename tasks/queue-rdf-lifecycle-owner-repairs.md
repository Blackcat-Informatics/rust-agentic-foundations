<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Actual completion-owner repairs

Production completion session 8165 returned exit 1 after real bound test and
example execution. `raw/queue-rdf-lifecycle-current.log` retains two refusals.
The actual default CLI check before completion succeeded. The record remains
open; no assessment or completion success is inferred from that check.

## Census selectable policy proof

An example-local `#[test]` is not selectable by the current bound-test runner's
library/binary/integration target commands. The actual test now lives only in
`crates/raf-census/tests/selection.rs`. It imports the real example as a private
module and calls its existing Body/Policy/groups and fixture registry helpers,
now `pub(crate)` within those executable targets. Command-only unused functions
have one explicit module-scoped dead-code allowance; no public library API or
duplicate policy implementation is introduced. The example no longer contains
a `#[test]`. The capability binds the selectable integration owner.

The unchanged assertions compare all fixture default-selected forms to actual
production Tower prints, pin 261 fixture bodies / 3900 tokens and 22 duplicate
groups / 54 pairs. Initial importing without relocating the test caused an
extra uncovered example test; the exact CLI count assertion caught 143 versus
142. That failure is retained at `queue-cli-coverage-adoption-current.log`.
Final owner execution passed in session 76197, exit 0, at
`queue-selection-owner-final-test.log`. Earlier nested-owner passes are historical.

## Actual CLI repository adoption

The old test expected raf-okf/raf-source outside scope and gaps. The real report
instead returned exit 0 with all eight packages in scope; original refusal is
reproduced with exit 101 at `queue-cli-coverage-lifecycle-repro.log`.
The replacement requires both code and test directions to report the exact
eight package names, all in scope, identical total/covered counts and no gaps,
exclusions or errors. Hyphen/underscore selectors require exactly the requested
normalized package, rather than allowing an ignored selector to pass because
the entire repository is covered. No allowance, scope escape or count filter
was introduced. Final whole-tree replay session 46362 returned exit 0: one test
passed, no failures. The exact receipt is
`raw/queue-cli-coverage-adoption-final.log`.

Final actual docs sync session 94629 and all-target OKF/Census/CLI Clippy session
83794 returned exit 0. Source-index bytes remain exactly unchanged across sync;
raf-source was not edited. Independent review is
`queue-lifecycle-owner-native-probe-review.md`. Fresh contexts and explicit
publications passed, followed by current currency and a clean production
default check. Production RDF completion 17112 returned exit 0 after the real
bound phase and deleted its record. `raw/queue-rdf-lifecycle-final.{ttl,log}`
retain the exact result. Broader native, strict/adoption and post-integration
full gates remain separate acceptance.
