<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Fixture Git verification repair

The shared workplan checkout helper, standalone workplan Git fixture helper and
command-line fixture helper contained explicit hook-path and signing-disable
overrides. These inherited overrides conflicted with the repository instructions
and have all been removed. No permission exception or substitute bypass is used.
Ordinary isolated fixture author/configuration handling remains; local hooks and
local signing settings remain effective. Freshly initialized fixtures ordinarily
have no installed repository hook. This does not disable an existing one.

The new public caller regression
`raf_workplan::test_crate::history::a_fixture_commit_obeys_an_installed_hook_and_the_successful_neighbor`
installs a real refusing pre-commit hook, proves that the normal shared helper
cannot create HEAD and preserves the staged candidate, then installs a successful
hook and proves that hook executed and exactly one normal commit exists. It passed
with exit 0 in `raw/queue-fixture-hook-preservation.log`. Its guarantee belongs to
the repository-contract capability, where it is explicitly bound.

Under the repaired ordinary helper invocation, the actual workplan completion,
Git, history and verification integration targets passed all 66, 22, 27 and 18
tests with terminal exit 0 in `raw/queue-fixture-normal-commit-callers.log`.
Command-line completion/history caller requalification passed all 12 and 6 tests
with terminal exit 0 in `raw/queue-cli-normal-commit-current.log`. The initial
command-line rerun exposed three state-promotion fixtures that omitted proof
markers; their authored proof ownership now exists before promotion. The first
CLI log remains historical in `raw/queue-cli-fixture-normal-commit-callers.log`.
Earlier helper-backed fixture
commit results are preserved as historical snapshots and are not current normal
hook qualification. The final full gate and hook still remain required for this
delivery; no product commit has been made.

A focused source search of both affected crates and the root tests/examples found
no remaining explicit hook-path, signing-disable, no-verify or environment bypass
forms. This is an inspection result for this checkout, not a claim about sibling
repositories. Required full-suite qualification still applies to the other shared
helper consumers before delivery.
