<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Independent source review

The reviewed delivery adds literal `cargo workplan verify` without a slug to
the existing Linux, macOS and Windows jobs after installing this workspace's
command. Existing test steps remain intact. CONTRIBUTING accurately explains
the resulting binding execution and platform exclusions. The AGENTS changes
repair stale delivered states and a duplicate RDF crate row.

The production command passes the absent slug to lifecycle verification. Its
`None` branch selects every capability's bindings and refuses unloaded
capabilities. It runs test and example bindings through the existing Executor;
failed and unresolved outcomes remain failures, and compiler-grounded
not-applicable outcomes are separately reported and excluded from pass counts.
No new filtering, outcome conversion or executor limit is introduced.

The platform mechanism and negative cases were qualified in the merged base.
The single actual local no-slug Linux command completed with exit 0: 2,754
passing bindings, two explicit not-applicable outcomes and zero failures. The
expanded native jobs now have 180-minute composition budgets, covering the
observed 53-minute verification plus their existing checks. Executor and test
limits remain unchanged. Required hosted Linux/macOS/Windows outcomes remain
pending; local execution does not establish the other hosts.

Source review: acceptable for publication and hosted acceptance. No additional
local test suite is requested. Merge remains contingent on current required
hosted outcomes and feedback.

The concrete macOS failure on c96a6b4 is an unprimed dependency cache: frozen
verification refused the missing raf-okf dev dependency cap. Its other 2,442
bindings passed and 14 were explicitly not applicable. Linux hosted verification
completed with 2,754 passes, two not-applicable and zero failures. Neither result
is relabelled as qualification of the repaired head.

Reviewed repair 768290e adds exactly cargo fetch --locked before the command
installation and verification in each native job. Fetching the root workspace
lock closes the observed cache prerequisite without altering frozen execution,
selection, exclusions, test limits or product code. Local fetch and the normal
commit hook both passed. Acceptable to publish once the existing Windows job is
terminal; publishing earlier would cancel that live evidence via concurrency.
The repaired head still requires its actual hosted acceptance and fresh feedback.

Reviewed bb27f3b independently: both linked-worktree fixtures now pass ../linked
to Git from the existing work repository. Their canonical Rust-side target paths,
linked-head/prefix/ref/blob/index assertions and actual Git operations remain.
This removes the verbatim Windows filesystem spelling from Git's argument without
weakening the fixture or changing library behavior. Both existing affected cases
passed locally and the normal hook passed; the Windows explanation remains an
inference until its actual native result. Publication is accepted. Run 39890744811
is active at bb27f3b, with nine checks passed and all three native verifications
still pending at review time. No additional local suite is requested.

Final acceptance: run 39890744811 completed successfully on bb27f3b. All twelve
required checks passed, covering every repository Makefile gate through CI's
documented equivalent recipes. Actual no-slug verification accounts for all
2,756 outcomes on each host: Linux 2,754/2, macOS 2,742/14 and Windows 2,560/196
passed/not-applicable, with zero failures. Both corrected Git fixtures actually
passed on Windows. Final current reviews, inline comments and threads are empty.

The fetched base d500d97 is an ancestor of the qualified head. merge-tree exited
0, and its candidate tree has no source delta from that head. There is no
uncovered base integration behavior. Reuse the unchanged production mechanism
qualification and these actual current-head hosted gates; no discretionary
local full-suite replay is requested. The selected stage contains 5.5 MiB of
regular evidence files and no symlinks. All lane writers and processes stopped.
Final source/requirement/feedback review accepts ghprsq merge without a waiver.
