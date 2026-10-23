<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Bounded forge feedback

## Current correction: indexed cache published

Comment 4364372990 is corrected in published signed head
af8b27a428e63dda80eb44e41de72d833d0ff06b, tree
dbb6535321a4a97c7e7a973f766ad35393b0d278, signature G. The locked native
OrderedTable replaces linear lookup; Source APIs, dependency pin and frozen
normal-form algorithm are unchanged. A 4,000-binding production case fails on
003de42 (raw/indexed-cache-before.log) and passes with a clean neighbor plus
smaller-grant refusal after correction. All 24 library and 60 Gate cases,
workspace lint, native completion, normal hooks, affected package tests/doctests,
MSRV and docs pass; push/update/meta exited 0. Review is closed in
tasks/indexed-owner-cache-review.md. Native physical arena and Source scratch
boundaries are explicit; retained vector capacity is state-admitted. Prior raw
review/thread captures below remain historical observations at 003de42, not
current thread-resolution claims. Hosted CI and landing remain with the owner.

## Current-head review capture and unresolved performance finding

Review 5634616918 is COMMENTED on exact head
003de42594489c6aba89da3dfab9f39f0217350e, submitted 2026-10-23T21:26:31Z.
Its sole new inline finding is comment 4364372990 on paths.rs line 182, thread
PRRT_kwDOVFffUs6vx1tl (unresolved, current). The earlier documentation-comment
thread is now resolved. Commit routing uses review id plus original_commit_id;
GitHub has rebased the earlier inline comment's current commit_id to this head.

Complete paginated capture:

- raw/pr-reviews-head-003de42.json: both reviews, including their complete bodies.
- raw/pr-inline-comments-head-003de42.json: both complete inline comments.
- raw/pr-conversation-comments-head-003de42.json: the complete updated conversation comment.
- raw/pr-review-threads-head-003de42.json: both threads, resolution/outdated
  state and all nested comment bodies. Top-level and both nested comment
  pageInfo values have hasNextPage=false; no omitted comment page remains.
- raw/review-5634616918.txt and raw/comment-4364372990.txt: complete new bodies.

The concrete performance concern is valid by source inspection. WorkList's
eight slots are inline capacity, not an entry limit. OwnerCache::get searches
with position(), then traverses again with nth() or last(). Its explicit cache
charge is 2 times the number of retained entries. For N distinct misses, the
cache-walk charge alone is N times (N minus 1); N=3163 costs 10,001,406 steps,
exceeding the default 10,000,000-step grant before native query and traversal
costs. This is an analytical bound from current code, not a newly executed
large-source regression. The existing two-site fixture establishes reuse and
bounded refusal, but does not establish scalable lookup. The earlier note that
linear cache lookup is charged remains true and does not remove this cost.

At capture this was an unresolved delivery performance blocker. Its correction
uses indexed, fallible lookup and insertion without
the second traversal, preserving quota admission and allocation-error handling.
The bot's proposed push-return-index change alone is insufficient: the inspected
public WorkList interface exposes iteration, not random-access retrieval.
This was verified directly in the locked 5db00aa checkout at
/home/paudley/.cargo/git/checkouts/purrdf-97037606e49ad484/5db00aa/crates/lex/src/walk.rs;
the same checkout's crates/jsonschema/src/ecma/vm.rs confirms default steps
10,000,000 and states 65,536. Current PurRDF main's WorkList also exposes no
indexed retrieval in the inspected interface.
Inspect the native library's applicable indexed storage APIs before selecting
the smallest implementation; do not copy embedded bot CLI instructions.

This pass captured evidence and read current code only. No Rust/source changes,
new tests, forge replies, thread-resolution mutations or merges were performed.

## Prior corrected findings

Full paginated forge records: raw/pr-review-comments.json and raw/pr-reviews.json.
Review 5634206152 is also preserved verbatim in raw/review-5634206152.txt.
Embedded CLI prompts were not followed.

Comment 4364042826 is valid: proc-macro2 desugars documentation comments into
string leaves, while the literalRegex contract selects written ordinary string
literals. The before-fix paired test failed with a prose COPY (raw/doc-comments-before.log).
The fix checks the original source range and excludes comment-origin leaves,
retaining explicit doc attributes, raw/escaped strings, macros and attributes.
No Form or normal-form fingerprint production path changed.

The ownership-cache nitpick is valid: item_at rescanned file candidates for each
pattern, and same_source repeated root classification. OwnerCache retains native
lookup results once per file/byte, shares the slice with path patterns and
constant observations, precharges scan/file-key work, and bounds retained entries
and owner references. Native WorkList supplies cache storage. The Source accessor
item_count_in reads existing by_file metadata; no parallel span index was added.
Actual native item_at/same_source retain the source/root ownership policy.
The native WorkList cache lookup is linear and explicitly charged; this is
source-query reuse and bounded work, not a universal asymptotic speed guarantee
or a new generic interval index.

Focused runtime: 22 Census lib cases plus all 59 Gate cases passed, including all
four documentation-comment forms, explicit doc attributes and strings, and cache
step refusal/reuse/state refusal beside sufficient-grant neighbors. Workspace
all-target lint passed after extracting the original-span predicate to keep run
below the existing line limit. Native change completion exited 0 and removed the
record normally (raw/review-feedback-native-complete.log). Affected Census/CLI
tests and doctests, MSRV workspace/fuzz, warnings-denied docs and all three local
Source cross-target checks exited 0. Normal signed commit hooks, push and PR
update exited 0 for signed head 003de42594489c6aba89da3dfab9f39f0217350e,
tree 03699091e95e396c5a6dedc6c2984a0a6f8e4baf. Windows-only native tests retain explicit observed
Linux guard not-applicable evidence; no Windows runtime pass is claimed.
