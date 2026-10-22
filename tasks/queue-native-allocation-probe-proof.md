<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native allocation probe ownership

Pinned PurRDF revision `5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43`
supplies `purrdf_alloc_probe::{CountingAllocator, CurrentThreadWindow}`.
The two OKF allocation targets now use this actual global allocator and open
native windows only around the production parse. Results remain live until the
window closes. Signed native peak values convert with checked `usize::try_from`.
No local unsafe allocator, native-counter mirror, saturation or zero fallback
remains. The guide's descriptive unsafe inventory is updated; lints are unchanged.

## Scope and preserved bounds

`Document::parse` calls `read_head`, tag sorting and `Body::parse` directly.
`Body::parse` calls `parse_at`, whose block, heading, marker and link scanners
run on the same calling thread. The complete OKF source contains no worker,
thread-spawn or parallel traversal; its thread-local work tally is synchronous.
The shared ECHAR values are const. These measured paths neither transfer owned
buffers across threads nor release pre-window input allocations. Native
calling-thread windows therefore cover the relevant parse allocations while
excluding unrelated harness traffic. This is allocator layout demand, not RSS.

The heading target retains `Cap<CountingAllocator>` with the unchanged hard
`1 << 30` process ceiling. Cap supplies refusal, since the native probe has no
hard-limit API. Its optional statistics feature is removed; native windows
alone supply peak measurements. The heading allowance remains short-parse peak
plus four times heading bytes per section plus 64 KiB. Both refused-document
families retain eight times input bytes plus 16 KiB. No new universal memory
claim is inferred from these finite fixtures.

## Actual owner results

- `cargo check --offline -p raf-okf --all-targets`: exit 0;
  `raw/queue-native-allocation-callers.log` records the exact pinned dependency.
- `cargo test --locked --offline -p raf-okf --test bounds --test alloc_bound`:
  exit 0, both actual tests passed; `raw/queue-native-allocation-tests.log`.
- `cargo clippy --locked --offline -p raf-okf -p raf-census -p cargo-workplan
  --all-targets -- -D warnings`: exit 0;
  `raw/queue-lifecycle-owner-repairs-clippy.log`.
- `cargo fmt --all`: exit 0; `raw/queue-native-allocation-fmt.log`.
- Fuzz manifest offline metadata update and subsequent locked replay: exit 0;
  `raw/queue-native-allocation-fuzz-metadata.{json,log}` and
  `raw/queue-native-allocation-fuzz-locked.{json,log}`.
- Final edited-state all-target Clippy: exit 0 in session 83794;
  `raw/queue-lifecycle-owner-final-clippy-current.log`. The preceding failed
  Clippy found a duplicated fixture-module load in the test-owner relocation;
  both consumers now share the existing example module instead.

Independent source review is `queue-lifecycle-owner-native-probe-review.md`.

No full gate, assessment acceptance or lifecycle success is claimed here.
