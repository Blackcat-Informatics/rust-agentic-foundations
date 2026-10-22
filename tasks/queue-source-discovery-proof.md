<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Discovery and target claim ownership

The overloaded Package discovery Contract is split into explicit roots, load-phase
failure order, package descent, discovery symlinks, package identity/order,
manifest syntax/admission, target settings, target paths, target crate identity,
invalid declarations, cross-package identities and target read origins. Operative
guarantees remain; no historical measurement was moved to avoid coverage.

Existing assertion bodies were inspected and mapped in
`tasks/queue-source-index-claim-map.md`, with the selected owner bodies reread
against the current checkout. Directory depth markers own the actual exact-depth,
one-over and 20,000-directory small-stack outcomes. Symlink classification owns
actual regular/directory/FIFO/outside/dangling/loop neighbors on Unix; it does not
claim native execution on Windows. Manifest size/depth markers do not imply an
universal allocation or parse-stack measurement.

The real default target fixture now also checks grouped bench and example paths.
The explicit target fixture now disables all four automatic target selectors while
retaining explicit lib/bin/test/bench/example tables. Its separately executed
default fixture supplies the automatic-path neighbor.

Two previously missing proof families were added rather than assigning arbitrary
nearby markers:

- Competing failures demonstrate discovery before target validation, targets before
  module parsing and module parsing before index use-path validation. Repairing the
  broken module surfaces the later index failure, and repairing that use path
  permits the neighboring load.
- Actual unreadable root files exercise explicit path versus table-name origins
  for binaries, tests, benches and examples, and explicit/default build-script
  origins. Each exact typed TooLarge refusal has a valid readable neighbor. The
  existing library fixture separately owns inferred/lib-table/lib-path origins.

The earlier four-target run `raw/queue-source-discovery-owners.log` returned exit 0
for discovery 28, target roots 8, walk-stack 7 and Unix containment 17 tests.
After the two proof additions,
`raw/queue-discovery-origin-current.log` passed discovery 29 and target roots 9.
Its encompassing command returned 101 on a test-string construction Clippy
finding; that is not a passing whole command. The fixture now writes into its
String directly. The final current Clippy and production sync results are in
`raw/queue-discovery-origin-final-clippy.log` and
`raw/queue-discovery-proof-docs-sync.log`; their actual terminal results govern
current qualification. No full gate, current assessment or delivered-state
adoption completion is claimed.

The independent bounded review in `tasks/queue-source-discovery-review.md` found
one genuine omission in ordering evidence: package names had matched directory
names, and no exact manifest sequence had been asserted. The existing ordering
fixture now opposes alpha at b/Cargo.toml and zulu at a/Cargo.toml, loads reversed
insertion order for its neighbor and explicitly asserts the manifest/name rows.
`raw/queue-discovery-package-order.log` and
`raw/queue-discovery-order-clippy.log` both returned terminal exit 0. The reviewer
rechecked those exact assertions without another finding. Final production sync
also returned exit 0 in `raw/queue-discovery-proof-docs-sync.log`.
