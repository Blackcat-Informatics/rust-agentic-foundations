<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

Run literal `cargo workplan verify` without a slug in the existing Linux, macOS and Windows jobs. Each job fetches the full locked dependency graph and installs the workspace command using its existing target directory, then judges every capability binding with the delivered platform exclusion rules. The expanded native jobs have enough overall time for this additional work; individual execution limits and existing checks are unchanged.

Correct the contributor guidance and stale delivered-state entries in AGENTS.md.

Use relative sibling destinations in the two linked-worktree fixtures, preserving their original assertions while avoiding canonical Windows filesystem path spelling in Git arguments.

Validation: all 12 checks pass on the current head, including actual no-slug verification on Linux (2,754 pass, two not-applicable), macOS (2,742 pass, 14 not-applicable) and Windows (2,560 pass, 196 not-applicable), each with zero failures. Both corrected linked-worktree fixtures execute and pass on Windows. All commits passed the normal hooks. The actual local no-slug command and the two changed fixtures also passed; unchanged implementation qualification from the preceding delivery is reused.

Closes #23.
