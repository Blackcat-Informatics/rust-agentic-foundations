<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Configured-budget error fixture review

Verdict: PASS for the scoped test-only correction in `crates/cargo-workplan/src/result_graph/tests.rs` against HEAD 89f219c. No product edits, builds or tests were performed by this reviewer. Current focused execution and hosted qualification remain the primary writer's responsibility.

The Windows job log records the same LinesError graph shape with an actual configured ItemBytes limit of 22,986 versus the representative fixture's 22,810. The correction does not skip Windows or weaken the real refusal into an arbitrary string match. It now requires the exact full domain error with the actual configured ceiling, zero crate rows and zero orphan rows, beside the unchanged admitted-source neighbor. Giving the fixture one spare byte also exercises a non-golden ceiling on Linux; failure to trigger the refusal still fails the test.

Both original golden formats remain byte-exact canonical round trips. The original N-Triples golden, sorted, still equals the unchanged external oracle fixture. For the actual input, the expected graph starts from that original golden; native-backed Builder must remove the exact original error assertion (asserting it exists), insert precisely the new configured-limit assertion and finish a valid graph. The actual projection must then equal the complete instantiated serialization, including command, classes, ordinals, links and regenerated content identity. All original assertions other than this input-dependent literal remain pinned, rather than being rebuilt from the report under test.

Source inspection confirms `Builder::from_graph` retains the authoritative native dataset; `remove` uses exact expanded assertion membership, `insert` validates roles and refuses duplicates, and `finish` freezes and validates the closed graph. The helper does not invoke the production command projection to manufacture its expected result. Existing actual-output parse/render round trips and valid/refused LinesError presence checks remain.

The dynamic expectation intentionally shares the native graph renderer with production; it is not a new independent oracle for every possible budget value. Independent sorted-golden pinning, exact unchanged fixture round trips and previously qualified kernel identity/serialization tests remain separate evidence. No new universal serializer claim follows from this test. No concrete masking of a projection field, extra/missing row, wrong error text or platform refusal was found in this delta.
