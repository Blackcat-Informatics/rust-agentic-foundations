<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Lifecycle owner repairs: bounded source review

Read the actual OKF allocation tests, Census selection example and integration
owner, CLI coverage owner, dependency declarations and AGENTS descriptive
allocator sentence. No builds or product changes were performed.

The allocation migration preserves the process-wide `Cap<CountingAllocator>`
1 GiB hard ceiling in bounds.rs, the 64 KiB fixed allowance and four-times
heading coefficient. alloc_bound.rs retains eight-times-input plus 16 KiB
for both actual anchor and late-alias refusal paths. Input construction is
outside the window; the parse result remains alive when it closes. The native
CurrentThreadWindow measures allocations/frees on its opening thread. Actual
Document and Body parse paths are synchronous and do not delegate work to
other threads, so excluding sibling harness activity fits these calls. The
native peak span can account for a trough below the opening level; this is
not a weaker substitute for the former high-water bound. This is not a
parallel-parser measurement claim. The dependency is real test execution,
and AGENTS now accurately describes the absence of a local unsafe allocator.

The Census integration imports the actual example, including its existing
fixture test. That test compares each policy key to actual Tower::print/form
results and pins corpus/group counts. The module-local dead_code allowance
documents the unused command/rendering entrypoints; it does not skip the
assertions or create a replacement selection implementation.

The CLI repository test now requires exactly eight named scoped package rows,
no gaps/exclusions/errors and equal total/covered counts in both directions.
These are substantive whole-repository assertions, replacing an obsolete
outside-scope expectation rather than permitting uncovered packages.

One remaining narrow assertion gap was sent to the owner: the final selector
loop asserts exit zero and no gaps only. Ignoring `--crate` would satisfy that
loop because all eight packages are now covered. Assert the exact selected
CoveragePackage identity list for each spelling; the existing one-package demo
fixture cannot independently distinguish ignored selection. This does not
invalidate the whole-repository coverage assertions above.

Owner-reported selected allocation tests, Census selection and affected
warnings-denied Clippy passed. Those commands were not rerun in this review.
No lifecycle, broader candidate or adoption acceptance is inferred.

## Current narrow recheck

The owner removed the example's nested test entirely after actual coverage
exposed duplicate test ownership. The integration file now owns the single
test body, retaining the same per-fixture production comparison and exact
261/3900/22/54 assertions. It imports the actual example's Body, Policy and
groups through crate-visible helpers; no second policy implementation was
introduced. The example no longer contains a test attribute. The earlier
description of importing the nested test is superseded by this source state.

The selector assertion gap is closed in current source: each invocation now
requires exactly its selected normalized CoveragePackage identity, no errors
and no exclusions, in addition to no gaps and exit zero. Fresh execution of
these final edits remains pending; previous passes describe the earlier state.
