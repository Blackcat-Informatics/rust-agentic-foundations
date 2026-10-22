<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Archive compatibility and reconstruction preparation

Read-only inventory, before final freeze. No capture, build, deletion, movement, new archive, or publication was performed. Other evidence writers were active, so counts below are observations, not a stable capture verdict.

## Concrete compatibility

The selected stage currently occupies about 924 MiB: scratch 753 MiB, raw 167 MiB, tasks 3 MiB and reviews 108 KiB. Inventory observed 34,468 regular files (877,693,896 logical bytes), 33 symbolic links and no special files. Regular modes: 1,392 at 0400; 29,382 at 0600; 3,371 at 0644; 323 at 0700. Git capture retains the executable bit, not these private/read-only permission distinctions.

**Blocker:** twenty historical scratch clones contain a directory named `.git`. Installed `ghprsq-stage.py::inventory` rejects any such entry before capture; ignore rules do not exclude it. Renaming or hiding those databases is not an acceptable repair.

The 33 symlinks are target-directory links into external `/opt/.cargo/target/...` caches. Capture records their literal link bytes, never the targets. Historical comparison scripts also name absolute cache binaries directly. These links are build-location receipts, not independently archived executable evidence. Sources, manifests, commands and full comparison output must remain available to rebuild the tools. Compiled `.rmeta` outputs are likewise rebuildable products; fuzz corpus octet streams and original fixture bytes are necessary inputs, not disposable binaries.

The largest observed file is `raw/queue-rdf-census-all.log` (61,386,396 bytes); four historical answer dumps are about 25–26 MiB each. No current file exceeds ghprsq's 104,857,600-byte push-blob bound. Aggregate stage size alone is not a documented refusal. Do not truncate those logs or create compression/encoding to reduce their guard visibility.

Installed capture first scans stable copies for credentials, writes ordinary Git blobs through a temporary index, verifies complete names/modes/bytes and rejects concurrent changes. It has a local `capture GIT STAGE TEMPORARY` operation; it does not itself publish refs. Final ghprsq atomically publishes the evidence ref with the merge and enforces the complete push-closure blob limit. A filename-only scan using the installed credential pattern found no match in current ordinary evidence outside nested Git metadata; this is not a final capture or attribution/content guard pass.

## Existing reconstruction evidence

The original producers already preserve the important inputs:

- `tasks/T3-review.md` records main 95c384c versus head 7d35393, clone roles, commands, raw output and target locations. `scratch/t3review/cmp{,2}.sh` retains the actual command matrix.
- T4 review/recheck and `scratch/t4review-tools/cmp3.sh` identify 7d35393, da83cdc and df26e24 comparisons, probes, mutant edits, restoration and full outputs.
- T5 identifies da83cdc versus a26e36d, the three oracle families, exact answer-dump sources, target/impl/reach probes and mutant findings. Sources live in `scratch/t5review-tools/`; full outputs remain in its logs.
- T6/T7 reviews and rechecks identify their commits, source/fixture changes, selected checks, refusals and repairs. Their ordinary source snapshots and logs remain present.
- `raw/donor-t8/` already retains tracked/untracked path manifests, original tracked patch and original tar copies, with the recovery rationale in `tasks/queue-integration-r5.md`. `raw/queue-rdf-before-main.tar` and `raw/pre-purrdf-untracked.tar` preserve separate pre-integration inputs. These are existing evidence, not proposed packaging; keep them unchanged.

All twenty scratch HEADs are present in the authoritative repository and are ancestors of current delivery HEAD. The explicitly referenced review/base revisions 03edbca, d2046a2, 9bfdaf6, 224ac69, 645f9e7, 68837e4, a34db9a, 9533c6e, 98b0a9c, 7d35393, 7a3a5f2, b3e6b7d, 01799a9, da83cdc, 87280b0, 1f66acc, a26e36d, ff8b258, fec6c21, 4449ceb, dc1115b, 973d7ab, df26e24, 95c384c and 88d9e42 were each checked for existence and delivery ancestry, not merely the current clone HEAD. Thus the eventual preserved PR-head ancestry carries those historical commit/tree bytes without requiring the clone object databases.

A broader clone-ref audit found the unrelated cached Source branch revision ffd5bf28cae35724bdcacf34ed40ff8e9eb08e0b outside delivery ancestry. No inspected historical producer claim or comparison script uses it. Do not silently turn every cached remote ref into a replay requirement or claim all clone refs are delivery ancestors. If a required claim using that ref is discovered, stop and resolve its actual source-history dependency before database removal.

Observed clone HEAD mapping, relative to `scratch/` (the preparation must record full resolved IDs):

| Clone paths | Recorded revision |
| --- | --- |
| `t2review/mut` | 68837e4 |
| `t3review/head`, `t3review/adv`, `t3review/mut`, `t4old` | 7d35393 |
| `t3review/main` | 95c384c |
| `t4-states`, `t4review`, `t4adv`, `t5-base`, `t5review-base` | da83cdc |
| `t5-states` | 87280b0 |
| `t5-states3` | 1f66acc |
| `t5review`, `t5review-adv` | a26e36d |
| `t4r2`, `t4adv2` | df26e24 |
| `t5fix-state` | 973d7ab |
| `t6review` | fec6c21 |
| `t7review` | dc1115b |

“Authoritative history” here means the canonical repository's object database plus current delivery ancestry, which ghprsq preserves through its PR-head parent; it does not mean these unlanded feature commits are already ancestors of `origin/main`.

The scratch trees are not all pristine: t3review/adv, t4adv, t5-states, t5-states3, t5review-base, t5review-adv, t4adv2 and t5fix-state retain modifications or untracked probes. Keep every ordinary source/probe file. Their current file bytes alone do not describe tracked membership, deleted paths or staged-versus-working content, so capture the narrow Git projections below before any removal.

## Metadata-only preparation plan

Run only after root freezes all selected evidence writers. This is a plan, not an executed receipt. Use the actual installed Git and existing source-history objects; create no new archive or blanket checkout copy.

For each of the twenty existing scratch clones, create a correspondingly named ordinary metadata directory below `raw/archive-reconstruction/`. Store only replay-relevant Git facts using maintained structured commands:

```sh
git --no-optional-locks -C "$clone" rev-parse --verify HEAD
git --no-optional-locks -C "$clone" symbolic-ref -q HEAD
git --no-optional-locks -C "$clone" symbolic-ref -q refs/remotes/origin/HEAD
git --no-optional-locks -C "$clone" ls-files --stage
git --no-optional-locks -C "$clone" ls-files --others --exclude-standard
git --no-optional-locks -C "$clone" status --porcelain=v1 --untracked-files=all
git --no-optional-locks -C "$clone" diff --cached --binary --full-index HEAD --
git --no-optional-locks -C "$clone" diff --binary --full-index --
```

A detached HEAD or absent origin symref is explicit state: distinguish Git's expected absence status from another failure. Do not blanket `|| true` these commands. Preserve normal quoted Git path output, including modes and index stages; it is ordinary explicit metadata, not a copied index database. Store a staged or working patch only if nonempty; an explicit empty-state field is sufficient otherwise. The binary-capable maintained diff format protects real binary changes; the original ordinary source bytes remain visible to normal guards.

Also record the exact value of origin's symbolic HEAD target and any actually used `refs/remotes/origin/main` target, plus `git merge-base --all HEAD <used-ref>` for each present used ref. Preserve the HEAD branch name when not detached. Record only relevant non-secret configuration such as `core.hooksPath` if a replay actually depends on it; do not copy credential-bearing config, reflogs, unrelated refs, hook samples or pack files. Existing reports provide revision roles and command interpretation; do not generate unrelated upstream commit prose.

For every recorded required commit, validate against the authoritative delivery repository:

```sh
git -C "$authority" rev-parse --verify "$revision^{commit}"
git -C "$authority" merge-base --is-ancestor "$revision" "$delivery_head"
```

Treat an absent object, non-ancestor required base, unmerged index stage, unexplained special state or required unrecorded ref as a blocker, not a reason to omit it.

Reconstruction is concrete: create one task-owned historical replay checkout below the canonical repository's `.worktrees/` from the retained authoritative history; use an independent clone when reproducing origin refs, because remote refs in a linked worktree are shared with its repository. Select the recorded HEAD, restore only the recorded used refs/symrefs, apply a nonempty staged patch with `git apply --index`, then apply the working patch without staging. Restore untracked probes from their already retained ordinary file bytes. Compare the resulting index membership/status to the captured projections. Rebuild binaries from these sources with the recorded manifests and commands into a newly owned build directory, substituting that directory for the historical absolute target paths. Do not mutate the authoritative repository's shared origin refs or revive historical binaries as acceptance evidence.

After the root writer validates those projections and reconstruction dependencies, removing redundant clone databases is supported evidence normalization: their required commit/tree history is already retained by the source PR ancestry, and their unique replay state has an ordinary explicit representation. It is not a relocation or renaming to evade the reserved-metadata rule. Do not remove or move anything during this review, and do not remove any ordinary source, fixture, manifest, probe, donor archive, command script or raw log.

Then freeze the final stage, perform the existing helper's local-only capture preview if still needed for this concrete compatibility risk, inspect its complete result, and keep required credential/attribution/content failures blocking. Any artifact change invalidates that preview. Only actual ghprsq publication establishes the durable evidence ref, which must be confirmed before cleanup.

## Executed normalization disposition

Authorized metadata normalization subsequently ran through `tasks/archive-normalize.sh`. The actual command completed with exit 0; its complete output is `raw/archive-normalization.log`. `raw/archive-reconstruction/validated.tsv` has exactly twenty validated clone rows.

For all twenty clones the projections were regenerated with explicit `--src-prefix=a/ --dst-prefix=b/`, full-index binary-capable native Git diffs, and disabled color/external diff/text conversion. Each recorded HEAD was read into a separate temporary index using the authoritative repository's object store. Its nonempty staged patch was applied with `git apply --cached`; the resulting complete mode/blob/stage/path listing compared exactly with that clone's original index. Every recorded used origin ref and every computed merge base was checked for object existence and delivery ancestry. Every temporary reconstruction index and lock was removed.

Only after all twenty index comparisons succeeded did the script remove the exact twenty redundant clone `.git` directories. No ordinary clone file or symlink, external target directory, donor archive or log was removed. `ordinary-before.txt` and `ordinary-after.txt` compare exactly for the ordinary scratch file/link names, modes, sizes, modification times and link targets. These are preservation receipts, not newly computed content-hash attestations. No command in this normalization wrote ordinary clone source bytes. The stage now occupies about 647 MiB, has zero nested `.git` directories and no remaining temporary reconstruction indexes. Product files, the authoritative live index and refs were not modified; no build, test, capture or publication ran.

**Verification limitation:** the instruction to additionally inspect clone `refs/stash` and in-progress merge/rebase/cherry-pick state arrived after the normalization process had completed its removals. That late inspection did not run. There is no fresh operation-state receipt, and those removed local pseudoref/state files cannot now be claimed accounted for by this projection.

The earlier read-only audit did enumerate all refs in all twenty clones and test their object IDs against current delivery ancestry. Its only reported non-ancestral ref was the unrelated cached Source branch identified above. That prior ref scan is relevant to possible unique stash commits, but it was an ancestry-filtered audit, not an explicit fresh stash inventory and not a scan of operation-state files. No stronger conclusion is inferred. The preserved authoritative donor stashes were untouched.

The exact index replay, used-ref/base projections and ordinary-source preservation receipts remain valid within this explicit scope. Final capture compatibility, remaining guards, full gates and publication are separate results. No speculative recovery or further review was started.
