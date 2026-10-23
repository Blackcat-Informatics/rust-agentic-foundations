<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Hosted binding verification acceptance

This queue delivery changes CI, repository guidance and two Git fixture path arguments exposed by the new Windows command. The compiler-observed platform exclusion implementation, distinct BoundOutcome receipts, failure/valid neighbors, capability contracts and reconciled assessments were delivered at d500d97. No production Rust, capability section, assessment or dependency bytes change here. Their qualifying full gates and twelve hosted jobs from that delivery remain applicable to unchanged inputs, not proof of the newly added no-slug command or the corrected fixtures.

| Requirement | Implementation or evidence | Current disposition |
| --- | --- | --- |
| Platform-specific bindings have a declared, truthful rule | Existing completion Constraints and compiler-observed exclusion implementation; actual unit, library and CLI refusal/valid-neighbor tests already landed | Reused unchanged qualification |
| Each platform judges all capability bindings | Existing Linux, macOS and Windows native CI jobs each fetch locked dependencies, install the current workspace debug command into their ephemeral Cargo bin and run literal `cargo workplan verify`, no slug or filter | All three actual commands passed on current head |
| Excluded bindings name their reason without becoming passes | Existing explicit `not-applicable` outcome, execution-pass count exclusion, text/RDF receipts and no-evidence-publication refusal | Reused unchanged qualification plus actual distinct hosted receipts |
| Local actual production entry point | Task-owned debug install, external retained target/build caches; actual `cargo workplan verify` on Linux | Install exit 0; verification exit 0 with 2,754 pass receipts, two explicit not-applicable receipts and no failures (`raw/verify-linux.log`) |
| Capability documents stay reconciled | No capability, assessment input or Rust change; current delivered guidance corrected without altering designed Census scope | Normal commit hook exit 0 (`raw/commit.log`), including the actual clean and assessed self-workplan gate |
| Hosted Linux, macOS and Windows each exit 0 | Newly mandatory CI steps; raw step output includes every binding outcome and reason | Observed on all three native hosts at bb27f3b |

Local installation used `cargo --config 'build.build-dir="/opt/.cargo/e6/19757045cb853d"' install --path crates/cargo-workplan --debug --locked --target-dir /opt/.cargo/target/16-raf-workplan-strict-mode-force-coverage-858ffaca --root /opt/.cargo/target/16-raf-workplan-strict-mode-force-coverage-858ffaca/verify-platform-tools`. The global installed binary was untouched. The verification command prepended that root's `bin` only to its own PATH and reused the retained target directory.

Source inspection confirms the CLI's optional slug reaches `lifecycle::verify`; `None` selects every loaded capability's tests and examples. It reports each actual bound Evidence outcome. Only Passed or NotApplicable are successful report records; unexplained missing/ignored/zero-list outcomes remain findings. The command neither limits selection to a named change nor uses quiet mode.

The original macOS process-group race note remains part of the issue's acceptance history. Current platform implementation and real process-group callers passed the preceding all-platform delivery; this new workflow must still surface any recurrence during the actual all-bindings run rather than waive it. Separate filesystem/process adapter extraction remains in its existing deliveries; this CI unit does not claim it.

No optional matrix, new exclusion, test binding removal, changed executor rule or full discretionary local suite is introduced. All three commits passed normal hooks completely. Independent source review accepts publication; final root merge preflight remains separate.

The single local verification completed in about 53 minutes. The prior successful hosted Linux test job took 24 minutes 18 seconds before this command was added. The three expanded native jobs therefore receive a 180-minute overall job budget; all existing tests and individual executor/build/example limits remain unchanged. This is capacity for the added mandatory work, not a runtime assertion or an observed macOS/Windows result. The independent production caller review is `tasks/queue-source-review.md`.

At initial publication, commit `c96a6b4a2c87d6282b419fbb15084614e05123b3` was pushed successfully. PR 56 is open at https://github.com/Blackcat-Informatics/rust-agentic-foundations/pull/56. The stagectl creation command created it but then reported an unparseable gh JSON response; a separate raw `gh pr list --head paudley/23-raf-workplan-verify-platform-specific --state open --json number,url,title` confirmed the unique PR. Hosted acceptance was pending at that checkpoint.

Hosted run 39879483433 judges initial head c96a6b4. The macOS job ended in failure: 2,442 passes, 14 truthful not-applicable receipts and 300 failures, each from Cargo's frozen refusal to download the unfetched OKF dev dependency `cap v0.1.2`. Full exact output is `raw/hosted-macos-failure.log`. Nine other jobs passed; Linux and Windows verification remain active. Reviews, PR comments, inline comments and review threads are empty.

The concrete repair adds `cargo fetch --locked` before the install/verification steps in each native job. The executor remains frozen and all bindings, limits and assertions remain unchanged. The actual prerequisite command exited 0 locally (`raw/fetch-locked.log`); this populated cache does not prove the fresh hosted scenario. No second local verification was launched. Repair commit 768290e passed the full normal hook (`raw/commit-fetch.log`) and was held until Windows completed to avoid automatic cancellation.

The original hosted Linux job is terminal SUCCESS. Exact output in `raw/hosted-linux-pass.log` contains 2,754 passes, two not-applicable receipts and zero failures. The original Windows job is terminal FAILURE; `raw/hosted-windows-failure.log` contains 2,258 passes, 196 not-applicable receipts and 302 failures. Of those, 300 have the same missing cap dependency cause; two actual linked-worktree tests also fail, with panic bodies truncated by the bound outcome summary. These require separate diagnosis; the fetch repair alone is not claimed sufficient. The original run is fully terminal, with ten successful and two failed jobs. Hosted all-platform acceptance remains unfulfilled.

The two additional failures share an absolute Git worktree destination formed from the canonical temporary root. The fixture now passes `../linked` relative to its existing `work` repository, preserving the exact canonical Rust-side target and every original prefix/head/ref/blob/index assertion. This avoids putting Rust's Windows verbatim filesystem prefix in a Git destination argument. It is an inferred cause pending actual Windows confirmation. Exactly those two existing tests passed locally (`raw/linked-worktree-current.log`); no full binding rerun was performed. Their normal commit hook passed (`raw/commit-linked-worktree.log`), including the clean assessed self-workplan gate.

Both repairs were pushed only after the original run was fully terminal. At the next intermediate checkpoint, head bb27f3b24366c8c2ddfe86feee11a6af307db728 had eleven successful checks and Windows 120209641607 was still running. Every native prerequisite, including locked fetch, had succeeded. Final acceptance below supersedes this pending checkpoint.

## Final hosted acceptance

Run 39890744811 is terminal SUCCESS at exact head bb27f3b24366c8c2ddfe86feee11a6af307db728. All twelve current checks pass (`raw/hosted-checks-current.txt`, stagectl exit 0). The actual no-slug commands produced:

| Host | Passed | Not applicable | Failed | Exact raw evidence |
| --- | ---: | ---: | ---: | --- |
| Linux | 2,754 | 2 | 0 | `raw/hosted-linux-current.log` |
| macOS | 2,742 | 14 | 0 | `raw/hosted-macos-current.log` |
| Windows | 2,560 | 196 | 0 | `raw/hosted-windows-current.log` |

Each host accounts for all 2,756 bound outcomes. Both corrected linked-worktree fixtures explicitly executed and passed on Windows, with one listed and one passed each; they were not skipped or relabeled. Final current PR metadata confirms OPEN/MERGEABLE at the same head, no reviews/comments, no inline comments and no review threads (`raw/hosted-pr-final.json`, `raw/hosted-inline-final.json`, `raw/hosted-threads-final.json`). The tracked working tree is clean. No local process or metadata writer remains after this final receipt. Ready for root's ghprsq preflight; no merge or cleanup is performed by this lane.
