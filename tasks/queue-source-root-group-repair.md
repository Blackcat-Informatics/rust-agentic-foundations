<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Whole-file crate roots and source groups

The first-item byte-zero collision was reproduced through the public Workspace
API: same_source(crate_root) incorrectly returned the first written item's group.
`raw/queue-source-root-group-reproduction.log` returned exit 101 on that exact
regression, before changing production behavior.

The existing index policy excludes crate roots from source groups because they
represent whole files. A shared Index::is_root predicate now implements that
policy for both group construction and same_source. It uses existing keyed
module/root tables and replaces the old per-item scan of all roots; no new
retained index storage is added. Public rustdoc and the source-key caveat state
that roots can share the first item's key but have no source group.

The new actual API fixture covers first-item byte zero, a neighboring offset-one
item and an empty root. Ordinary written items retain themselves in their groups.
The old grouping fixture had selected a test target's same-named root with only
name == a; the original defect made that root appear to have a one-item group.
It now explicitly selects the actual function for its ordinary-group assertion.

`raw/queue-source-root-group-final.log` passed all 35 symbol-index and 6
target-resolution tests with terminal exit 0. Initializer range ownership also
names the existing shared-path parameter/initializer fixture.

The selected workplan coverage consumer run exposed existing sole-policy fixture
drift: 13 passed and 11 failed in
`raw/queue-source-root-group-workplan-current.log`. The retained public/test gap
assertions are accompanied by genuine default-zero orphan-line refusals, and the
repository fixture still expected only two scoped packages rather than the
current eight-package adoption record. These failures are not a qualified caller
pass. They are being repaired by explicit typed orphan-row and refusal assertions,
without changing production policy, filtering unchecked findings or raising
ceilings. The large fixture has two actual line refusals in addition to its
expected public/test findings. Current execution and remaining findings must be
read from the subsequent coverage logs before claiming completion.

Primary bounded source review is `tasks/queue-primary-source-item-review.md`;
full adoption, assessment currency, lifecycle completion and full gates remain
unfinished.

## Current caller qualification

The root guard affects only whole-file root queries. Actual public/test surface
coverage consumes written exported items and test functions, while line accounting
uses its reachability graph; it does not use same_source for those line decisions.
The observed stale fixture expectations were repaired without changing that code.

The coverage fixtures now use explicitly supplied expected package/shipping/test
orphan counts. Their helper first checks the complete typed row list, zero line
exceptions, ceilings and debt; it then checks every actual default-zero line
refusal's cardinality, location, package, class and count before preserving the
original exact public/test/reexport assertions separately. Unchecked findings
are not filtered. Public/test exclusions still do not waive counted source lines.

`raw/queue-coverage-sole-policy-final-neighbors.log` returned exit 0 for all 23
selected fixtures. The eight-package repository fixture had already passed in
the preceding full target `raw/queue-coverage-sole-policy-current.log`, whose
overall exit was 101 because three other fixtures still needed their exact row
neighbors corrected. That successful repository result is reused, not described
as a passing whole target. It asserts all eight actual scoped packages, complete
public/test reports and zero orphan/exception/debt rows. No product debt baseline
or permissive ceiling was added.

`raw/queue-source-root-group-untouched.log` returned exit 0 for all 34 actual
section/digest/Git caller tests. The initial selected Clippy checks correctly
refused fixture assertion style; no allow was added. Current terminal Clippy and
generated documentation results must be taken from
`raw/queue-source-root-group-qualified-clippy.log` and
`raw/queue-source-root-coverage-docs-sync.log`, respectively.
