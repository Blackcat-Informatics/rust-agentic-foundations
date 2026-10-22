<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Bound-root walk failure regression

Read-only diagnosis. No source edits, builds or tests by this reviewer. Parent's
exact reproduction is `raw/recovery-strict-line-error-runtime.log`, exit 101.
The assertion at `coverage/lines/tests.rs:285` expects the real walk's error
context; actual errors are `["a bound coverage root has no indexed item"]`.

## Cause and ownership

`demo_and_wide` creates a genuinely out-of-range root: wide::c's item place is
past the items of the small workspace. `raf-source::ReachError::UnknownItem`
explicitly owns that refusal (`reach.rs:160–177`), and Workspace::reachable
documents and delegates it (`workspace.rs:557–572`). This is a place/range check,
not a claim that ids carry workspace provenance; in-range foreign ids would not
be a valid negative fixture.

The new registration-context scan in `coverage/lines.rs:1447–1463` eagerly
rejects missing Item/Impl roots while determining participating targets and test
roles. It exits before the actual `source.reachable` call at lines 1481–1483,
which retains the original `the walk of the bound items failed:` context.
Consequently the current negative no longer executes the walk it claims to
exercise. The preceding table-format test does not catch this because it uses
the actual message as its expected table payload.

## Smallest coherent fix proposal

Keep root identity refusal in the actual Source walker. The registration scan
should inspect target/test context for roots that are indexed, without removing
any root from the subsequent `nodes` vector. An invalid root contributes no
registration context; it **must remain an input of the unconditional real walk**,
which refuses before a successful report can be produced. This avoids a second
walk, retains typed dependency ownership and restores the existing error wrapper.
It is not an ignored error or partial report: compute still returns exactly one
error-only report from the real failing operation.

Suggested local replacement (metadata only, not applied):

```rust
    for node in roots.nodes.iter().chain(extra) {
        // This scan only determines registration context. Every root,
        // including an invalid one, remains an input of the actual walk.
        let context = match node {
            Node::Item(id) => source.item(*id).map(|item| (item.target, item.test)),
            Node::Impl(id) => source.impl_block(*id).map(|block| (block.target, block.test)),
        };
        if let Some((target, test)) = context {
            participating.insert(target);
            if test != TestRole::None {
                tests.insert(target);
            }
        }
    }
```

Preserve the original test assertion and its passing no-foreign-root neighbor.
The implementation owner should add/assert the exact underlying out-of-range
message and cover an invalid Impl root too, while keeping both root variants in
the walker inputs. Existing allocator fixtures must retain participating-target,
test-role and initializer-dependency behavior. No ceiling, finding severity,
baseline or gate needs changing.

If eager registration validation is deliberately retained instead, it needs its
own explicit negative and this walk-specific test needs a genuine walker refusal
using valid roots, not a renamed expectation of the early error. A deterministic
existing pattern is `raf-source/tests/reachability.rs:895–925`: load the workspace
with max_item_bytes equal to measured successful load usage, then call reachable
on a valid root and require typed ReachError::Budget. Such a fixture needs a
separate admitted valid-workspace neighbor; an empty-root call can also allocate
walk scratch, so it must not assume the zero-headroom workspace counts empty
roots successfully. This is a larger fixture change than preserving the one
existing root-validation path above.

## Verification limit

The exact failing log and production/test paths were inspected. This proposal
has not been compiled or run. Parent retains the single writer/build slot and
must run the focused negative/control plus affected allocator/reachability
coverage before rerunning the actual strict completion. Earlier unit passes do
not qualify the current registration-scan change.
