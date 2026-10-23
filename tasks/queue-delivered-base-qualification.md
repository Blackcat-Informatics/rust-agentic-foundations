<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Delivered-base qualification

Base is actual delivered origin/main 26f3013. Recovery stash
75757f394328be3d6d7d9f6192c8cc4c85e5ba58 and every earlier stash remain.
Workspace all-target callers pass 53022, exit 0. Both locks retain ctutils 0.4.4,
nine-package coverage and REPORT_FORM 3 / NF 1 are unchanged, and the required
tools/MSRV/cross targets are available. Two build jobs remain the limit.

Actual completion 41369 exited 1 with three owned failures; the record was kept.
No gate or assertion was weakened:

- fs_confinement retains its disk-only source boundary. The Git host constructor
  now shares the disk backend's original root canonicalization operation, with
  unchanged GitIo mapping and no duplicate source acquisition path.
- Invalid dependency path types retain their established owning-entry manifest
  origin and name path explicitly in the message. The legacy assertion stays.
- The missing-Git fixture validates its isolated child's actual status and
  captures its output, so the parent has one libtest result line. No proof parser
  tolerance was changed.

All four affected targets passed 89887, exit 0: 62 cases. The owned loose-object
fixture now clears only its Windows read-only flag before removal; current native
testkit has no public arbitrary existing Git-object deletion API. The Linux
substitution control passed 7406, exit 0; Windows runtime remains CI-owned.
This fixture helper is scoped to its one target, avoiding dead code in the other
consumer. Final Source all-target Clippy passed 43658, exit 0.

Actual full bound completion passed 96746, exit 0. It removed the record after
the complete bound inventory; two existing Windows disk cases were explicitly
not applicable on Linux from actual compiler facts. The matching temporary
inventory row is removed, leaving no net docs_contract change. Source/index is
staged without stage artifacts. Mandatory two-job make check is running as
88041, through fmt and Clippy into workspace tests. No full-gate pass, commit or
publication is claimed yet.

The make check run 88041 terminated exit 2 at REUSE after passing fmt, workspace
Clippy/tests, Source release tests, documentation and offline deny. The seven
native assessment replacements had omitted explicit copyright/licence options,
so their SPDX headers were absent. This is owned metadata repair, not a policy
exception. The records are being regenerated with native CLI SPDX options;
unchanged code/release/doc/deny evidence remains applicable. Affected metadata
gates and remaining mandatory targets still require actual passing results.

All seven records were regenerated successfully by the native CLI with explicit
CC-BY-4.0/SPDX copyright options (3516, exit 0), then restaged. Only those seven
metadata headers changed; product source stayed frozen. Affected docs_contract
and self_workplan execution is running, followed serially by unchanged REUSE,
MSRV, fuzz, cross-check, licenses and issue-refs. The initial passing code phases
are reused rather than needlessly replayed for comment-only licence repair.
Unchanged substantive reviews and pinned provider-route receipts are reused;
full original source behavior and actual Census provider execution remain open.

The continuation 84342 is terminal, exit 0. Affected docs_contract and
self_workplan passed, followed by REUSE, workspace/fuzz MSRV, fuzz
formatting/Clippy/offline deny and all five 5,000-run fuzz targets, all three
required cross-target checks, licences and issue-reference gates. Logs/status:
`raw/queue-source-licensed-metadata.{log,status}` and
`raw/queue-source-final-remaining-gates.{log,status}`. Together with the valid
preceding make check phases, mandatory local coverage is complete. The original
make check exit 2 is preserved; no single uninterrupted pass is claimed.
Normal signed commit hooks and publication remain pending. No runtime or source
process from this lane remains active. Every recovery stash remains retained.

Normal signed commit f601a80 succeeded (64255, exit 0), including all configured
staged-snapshot hooks: formatting, REUSE, 31 document-contract cases, 76
repository-gate cases and the self-workplan gate. No hook was bypassed. Push and
PR publication are next; hosted qualification and final merge remain pending.

Push passed, exit 0. PR 58 is open at
https://github.com/Blackcat-Informatics/rust-agentic-foundations/pull/58 with
head f601a80. `stagectl pr-create` returned exit 1 after actual creation because
its GitHub JSON response was unparseable. A raw remote inventory confirmed the
single open PR before retrying; there was no duplicate publication. Exact remote
body/head are preserved in `raw/queue-source-publication-remote.json`.
Hosted CI, feedback and final merge remain pending with the parent owner.
The Source candidate is frozen and all recovery stashes remain retained.
