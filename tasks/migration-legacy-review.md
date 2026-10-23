<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Foreign migration source review

Bounded read-only review of the new foreign importer, baseline/fields/rows helpers, shared UnmappedRow/finish/remap_members delta and five proposed runtime cases. No builds, tests, production edits or subagents were used. The initial owner compile receipt is not treated as runtime acceptance; the current runtime batch was still pending when assigned.

## Concrete findings

1. **Invalid all-mapped success, repaired in current source.** Initially a missing summary produced an empty placeholder, no unmapped row, and a policy rejected by the real ledger reader; the initial test explicitly accepted this state. The governing contract requires preserving the placeholder. Current header construction now reports empty/blank/multiline summaries as UnmappedRow while retaining their value. Shared finish now invokes Ledger::from_turtle when no unmapped rows remain. This closes the source finding without removing the placeholder contract; execution after the repair remains owner qualification.
2. **Native container adoption, repaired in current source.** `legacy.rs::check_depth` now uses pinned `WorkList<_, 32>` with fallible `try_push` for initial and child entries. LIFO traversal and the existing depth rule are preserved. `Error::ImportStorage` retains the original TryReserveError in its source chain. The new 64-field fixture exceeds inline capacity and asserts that all 64 unknown values remain available; its execution is owner qualification, not a pass observed here.
3. **Narrow invariant-comment hygiene, repaired in current source.** `baseline::decode` now explicitly documents that Record::new established object shape before `as_object().expect`. This closes the comment finding without changing parsing behavior.

## Reviewed dispositions

Flat/nested forbidden aliases and dialect/ext collisions refuse instead of choosing a precedence. Native strict JSON Records reject unknown/duplicate fields; canonical native hex decoding rejects wrong kind/width. Repeated historical member symbols intentionally deduplicate for hash union, while current source resolution still expands physical alternatives. Distinct-row repetitions become explicit multiplicities. No source cfg filtering or legacy reader was added to production Gate.

Unknown fields retain their complete TOML value in structured rows and escaped comments. Projectable dialect metadata retains strings, booleans, integers and ordered/nested JSON arrays/tables through native builders; nonfinite floats/date values retain the complete unprojectable original instead of becoming null. Unmapped fingerprints retain an assertion and a review row. Shared finish caps bytes after comments. Inputs and Tree remain read-only; no output file is published by these APIs.

The ledger byte cap precedes TOML decoding; the application container-depth check follows the maintained TOML parser. Native JSON receives explicit byte/value/string/depth limits and unique-members checking. Current/future declared ledger versions refuse, and the legacy function accepts only the explicitly selected JSON baseline shape; the modern RDF index entrypoint remains separate. Parser/record failures preserve typed import causes and named input paths. No further concrete mapping/refusal defect was found in this bounded review.

Legacy runtime outcomes, final Clippy/gates and complete delivery acceptance are not asserted here. Pending patterns/detectors/reference/CLI work is not a regression of this delta and was not reviewed again.
