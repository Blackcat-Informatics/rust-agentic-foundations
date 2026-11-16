# Issue #269: workplan: let evidence cite an artifact by path and digest

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

## Problem

Evidence can only point at records and source symbols inside the workspace (`crates/raf-okf/src/profile/records.rs`). A claim cannot cite a measurement or a log by path and content digest, so nothing fails when that artifact is gone or has changed. The evidence `revision` key is validated as commit-id text and is never read (`crates/raf-workplan/src/obligation/trace.rs`).

## Smallest design

- A static check: an evidence-only input row names a repository-relative path and a byte digest. The check hashes the file in the worktree against the recorded digest. It reports a missing file and a changed file, reusing the existing evidence-stale finding.
- `revision` stays informational. It is recorded with the row but not read from Git, so the workplan keeps its no-process, read-only rule.

## Limits this design has, stated plainly

- Reads are text-only today: `Tree::read` returns text, so a binary artifact needs a new read path that returns bytes through the same verified handle.
- An artifact under an ignored directory (for example local scratch space) is not tracked, so it may exist on one machine and not on another, and cannot be recovered from Git. An artifact outside the workspace conflicts with the source index's containment rule and is refused.
- Because the revision is informational, the check proves that the file matches the recorded digest, not that the file existed at that revision.

## What it changes for existing users

The row is optional and evidence-only, so existing records are unaffected. `InputKind` is an exhaustive enum, so every match gains an arm. An older RAF rejects a record that states the row.

## Acceptance

- An evidence record citing an existing artifact with a matching digest is accepted; a missing artifact and a changed artifact each fail, side by side with the accepted one.
- A path that escapes the workspace is refused.
- Records without the row behave exactly as before.


## Comments (0)

