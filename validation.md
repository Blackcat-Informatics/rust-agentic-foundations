# Validation for the artifact-evidence change at 5da1f0b6

## Head and base

- Merging head: 5da1f0b6 on paudley/269-workplan-let-evidence-cite-an-artifact, after 9259237f, f94a6788 and the review fixes.
- Base: origin/main f94c75bd, unchanged at merge preparation (0 commits behind). `git merge-tree --write-tree origin/main HEAD` exits 0.
- Net change: raf-okf (profile row kind and closed artifact names), raf-source (Tree::read_bytes, with disk and memory trees implementing it and delegating trees forwarding it), raf-workplan (artifact digests at load, stale and refused currency, reconcile refusal, diagram and strict fix text), documents and assessments.

## CI (checked by me)

- CI run 43881936374 on 5da1f0b6: 24 of 24 required checks passed. CodeRabbit is skipped and not required. The PR is CLEAN.
- The earlier run 43873157221 at f94a6788 failed in the doc job (an unresolved link to a moved method) and in the sync test. Both are fixed at 5da1f0b6, and the new run passed.

## Commit hooks (reported by the coding agent; the commits exist on the branch)

- The pre-commit hook for 5da1f0b6 reported docs_contract 29, repo_gates 81, self_census 4 and self_workplan 1, all passed. I did not read the hook log myself.

## Focused runs

- Checked by me at 9259237f, before the review fixes: raf-workplan obligations 34 passed and evidence 5 passed, exit 0.
- Reported by the coding agent at 5da1f0b6, not rerun by me: raf-workplan sync 23 passed; raf-okf records 39 passed; the strict and assessments named cases; the rustdoc build of raf-source under `RUSTDOCFLAGS=-Dwarnings`, exit 0; clippy with warnings denied on raf-okf, raf-source and raf-workplan. Logs are under .stage/workplan-let-evidence-cite-an-artifact/.

## Independent review

- One review at 9259237f: BLOCKED on two required items (assess treated artifact paths as symbols; the complete-check artifact path had no test), plus failure-path and wording items. All fixed at f94a6788 or 5da1f0b6; the code deltas were read once by me.
- Review items on PR 279 at 5da1f0b6: drive-designator refusal addressed; digest wording addressed; typed refusal and the staged `..` item declined with replies. The staged `..` decline rests on RelPath::new normalising dot-dot and refusing paths above the root (crates/raf-source/src/path.rs), which I read.

## Merging reading

- I read the net source diff of raf-okf, raf-source and raf-workplan, and the deltas of each fix commit. See delta.md.

## Checks not run

- No whole-workspace gate was run by me; the hook and CI cover the wider surface.
- No mutation testing and no timing measurement.
- I did not re-read every renewed assessment end to end; the coding agent re-read the sections whose text it changed, and the rest went stale from changed bound symbols.

## Known limits (stated in the capability documents)

- An artifact is read as a whole file up to 256 MiB; a larger file is stale, not read in part.
- A file under an ignored directory is read like any other; whether it is tracked is not judged.
- The revision is informational; the check shows the file matches the recorded digest, not that it existed at that revision.
- A Windows drive designator is refused in artifact paths on every platform; the primary target is Linux.
