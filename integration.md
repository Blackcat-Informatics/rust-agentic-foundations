<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Current base integration

The delivery commit eff9b6d passed the full make check and normal commit hook.
The fetched main is d957883. Its eleven-file delta adds native-host binding
verification and locked fetching, portable relative Git worktree fixture paths,
the Python-file repository gate, and matching policy/documentation assessments.

The actual merge-tree inspection found only AGENTS.md conflicts. The resolution
retains main's default Workplan policy and removal of its duplicate RDF status,
alongside the delivered Census ledger/debt/migration and partial CLI inventory.
Both sides were inspected; no implementation side was discarded.

Main changes no Census implementation, report fixtures, Cargo lock, dependency
policy or fuzz inputs. The complete delivery qualification therefore remains
applicable. The affected real Git fixture binary passed all 22 tests with no
ignored or filtered cases in raw/closure-base-git-fixtures.log. Normal integration
commit hooks passed against the combined index, including the new Python gate,
document contract and assessment gate. The integrated commit is e8215d7. Hosted
checks remain pending; no hosted pass is claimed.

The integrated workflow runs all bound tests/examples on Linux, macOS and Windows.
Head 5c0e789 adds only the ctutils 0.4.4 patch in both lockfiles, addressing the
fresh hosted advisory without changing source or policy. Its affected 195 Census
and CLI tests, both deny checks and normal commit hook passed. All required local
make-check targets are now qualified; validation.md records the cross-target
prerequisite repair and successful remaining gates. Hosted qualification remains pending.

The earlier origin/main d957883 and head 5c0e789 merge-tree computation exited 0
with tree 24507c6a58dc955b76ff05decc22ecf36ad0b498. That is a historical candidate
before the workflow repair, not the final expected tree. Refresh final base/head
and the candidate after hosted qualification; confirm actual landing inputs/result.

Head 3521144 adds only native test-job storage settings after the Linux runner
reported disk exhaustion. Source, locks and local make-check environment remain
identical to 5c0e789, so the completed local qualification remains applicable.
The workflow change passed normal hooks and independent focused review in
tasks/ci-storage-review.md. Its new hosted run must prove sufficient storage;
the failed run is not counted as a pass.

Current final candidate: fetched main remains d957883, PR head is 3521144 and
GitHub reports MERGEABLE. Read-only merge-tree exits 0 with tree
9e4788795fc8333a173b4ff67fd58f448b5cd0b2. Main is already incorporated; no new base
delta invalidates qualification. Eleven hosted checks pass, including actual
Linux and macOS no-slug binding verification. Windows remains pending. Refresh
before ghprsq and compare its actual reported inputs/tree after publication.

Final refresh confirms the same base/head and clean tree. All twelve hosted
checks pass; Windows accounts for 2,620 pass and 196 platform-not-applicable
bindings, zero failures. Paginated reviews, inline comments, issue comments and
threads are all empty. No uncovered integration delta or open feedback remains.
