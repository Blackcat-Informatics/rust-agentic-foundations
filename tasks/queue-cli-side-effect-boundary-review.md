<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# CLI trusted-side-effect prose review

Bounded read-only review; no builds or product edits. No concrete defect found
in the repaired judged Constraints or their production routing/proof composition.
This is not fresh context acceptance or adoption completion.

The CLI now distinguishes its own document/record publication from child
execution. Its explicit judged limitation retains trusted tests, examples,
build scripts and evidence programs, possible actual workspace writes and Git
partial-clone object fetches. No sandbox or global unchanged-repository promise
remains in those items, and the limitation has not moved to Explanation.

Actual routing supports composing the library proof: change::reconcile opens
Lifecycle and calls reconcile; change::verify opens Gate and calls the same
library verify with a real SystemRunner; history::history calls Workplan::history
with a real SystemRunner. No CLI wrapper suppresses child effects or copies a
workspace into a sandbox. Result rendering does not turn those operations into
different execution paths.

The existing CLI example-failure fixture invokes the built CLI against the
actual bound exiting example, requires exact example-failing obligation/exit
code and an unchanged benign worktree; its repaired successful neighbor removes
the obligation. This proves delegation and benign tool-publication behavior,
not a universal absence of child writes. The bound library trusted-effects
owner separately executes actual writing example, writing build.rs and ordinary
neighbor, comparing complete repository snapshots with precisely the expected
marker change. The previously reviewed partial-clone owner requires an actually
missing historical blob, actual history fetch into Git objects, unchanged index,
all snapshot differences confined to Git metadata, and complete-clone unchanged
neighbor. These are substantive side-effect producers composed with real CLI
callers, not dummy bindings or dependency presence.

The two new frontmatter owners and the limitation marker are present. Existing
publication-specific CLI owners retain their own exact write assertions; the
child limitation does not waive required atomicity/refusal/error behavior of
those tool-owned writes.

Minor internal wording consistency remains outside this prose-only repair:
change.rs's verify comment and history.rs's helper comment still say “Nothing
is written”/“writes nothing” without the publication qualifier. Their actual
implementations follow the repaired boundary above; these comments should use
the same tool-owned scope when next edited, rather than be treated as a sandbox
claim or behavioral proof.

No checks were rerun here. Earlier CLI contexts are historical inputs and must
not be accepted as fresh contexts for this prose. Fresh generation/context and
required gate evidence remain with the owner. The read slice is released.
