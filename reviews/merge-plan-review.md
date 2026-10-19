# Merge-plan review: PR 31 / issue 27 (squash notes at HEAD 56ffc9f), recheck

* **Reviewed:** `/home/paudley/Active/rust-agentic-foundations/.worktrees/27-raf-source-the-registry-scan-example.squash_notes`
  (revised).
* **Method:** reading only. Nothing was run, and nothing was timed.
* **Scope:** this recheck settles each finding from the first pass and reviews only the text the fixes introduced.
* **New evidence read:** `/tmp/f27-scan.txt`, the summary block and the ERR lines of the local registry scan.

## Prior findings

| Finding | Status | Evidence |
|---|---|---|
| **B1** ("findings fixed" in process wording; the integration clause repeated) | DISCHARGED | The sentence is gone. The Validation paragraph now ends at the scratch-registry run (notes :32-36). The only remaining mention of main not moving is in `## Conflicts` (:73). |
| **P1** (hosted CI pending; the merge precondition) | MET, as reported by the coordinator | 12 of 12 checks pass on 56ffc9f, and `git rev-list HEAD..origin/main` is 0, so main is still at 679ca0a. I had no shell to re-run `gh pr checks`; this rests on the coordinator's report. The claim on notes :33 is now true, and CI on 56ffc9f covers the local `make ...` claim and `cross-check`. |
| **A1** ("shipped" run) | DISCHARGED | :34 now reads "the example was run over a scratch registry". |
| **A2** ("admits every load" of 1271) | DISCHARGED | :20-21 now say "of which 1251 load and the other 20 fail for reasons other than a budget". The new claim is checked below. |
| **A3** ("pin") | DISCHARGED | :14 reads "growth tests bound the doubling ratios". |
| **A4** ("in a scratch clone") | DISCHARGED | :34 reads "every counted claim has a mutant that fails it". |
| **A5** (not a conventional subject) | DISCHARGED | :1 reads `feat(raf-source): bound the registry scan by counted work instead of a clock`, which matches CONTRIBUTING.md:123-125. |
| **A6** ("and" should be "or") | DISCHARGED | :26 reads "stated in the capability or the example". |
| **A7** (diffstat leading space) | STILL-OPEN, accepted | :51 is unchanged. The coordinator checked it. It is cosmetic, does not change what lands, and does not block. |

## New text reviewed

* **"of which 1251 load and the other 20 fail for reasons other than a budget"** (notes :20-21) is VERIFIED.
  * `/tmp/f27-scan.txt:1272-1280` reads `# crates=1271`, `# ok=1251`, and then `err Syntax=2`, `Target=5`, `Tree=2`, `UnresolvedMod=7` and `UnresolvedTrait=4`.
  * Those error counts sum to 20, and there is no `err Budget` line.
  * The 20 ERR lines in the file (for example `:10`, `:385`, `:494`, `:933` and `:1230`) are all of those five kinds.
  * `# largest work=34343004 (web-sys@0.3.106)` is at `:1288`.
* **The scan was made before the final head.** gap-analysis.md:76 records that `src` has changed only in a comment since then, so the figures stand for HEAD.
* Line 21 is longer than the paragraph's other lines. This is cosmetic only.
* **Issue and PR numbers:** the only one is still `Closes #27`.
* **Process wording:** none is left. The phrases the gate refuses ("this issue", "per the plan", "this branch") do not occur, and the reviews and gap-analysis narrative is gone.

## Coverage, unchanged from the first pass

* **Numbers:** every count, closed form and registry figure matches the evidence.
* **Requirements:** none of R1-R6 is silently missing.
* **Declined options and residual limits:** they match plan.md, the capability (`source-index.md:774-777`, `:1720`), the example doc (`registry_scan.rs:31-42`) and the issue 28 disposition recorded in gap-analysis.md.
* **Integration:** sufficient. Main has not moved, so the squash tree is the 56ffc9f tree that CI passed.

## Findings

### BLOCKING
None.

### ADVISORY
* A7 is still open. It is cosmetic and accepted.

VERDICT: PASS
