<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Crate-root record input review

PASS for this narrow delta. No concrete schema/identity regression found. Read-only review; no builds, product edits or assessment publication.

`input_name_fits(Symbol, name)` now uses the existing identifier-segment validator without imposing the separate two-segment item-path minimum. It admits one valid crate identifier, retaining rejection of empty segments, a lone underscore, initial digits, hyphens and malformed separators. Section and record input forms are unchanged. Evidence binding subjects still use the two-segment `is_item_path`; coverage exclusion/exception rules are unchanged. This fixes record reload for an already legitimate crate-root symbol input rather than widening those distinct binding/exclusion policies. The BadInputName diagnostic now describes the accepted form accurately.

Actual OKF tests pin crate/member/method acceptance and malformed-path refusals with exact inputs-key/BadInputName ownership. The old invalid-name neighbor is correctly changed from a newly valid single identifier to a leading-separator path. Quoting the trailing-colon fixture ensures the refusal reaches the symbol-name validator rather than failing YAML framing.

The new Workplan integration uses the real on-disk record_assessment path, reads back the written root input, reconstructs Workplan on each currency call, observes Current, edits an actual function in the root source, observes exactly the changed Symbol/demo input, then republishes and observes Current. It does not broaden crate-root digest ownership to descendant files; the existing own-file digest rule remains. Rule-tag test ordinal/count adjustments are the previously diagnosed exhaustive-map repair, not a new schema variant or accepted value.

Inspected the passing receipts: `recovery-root-input-profile.log` has 37 record tests, and `recovery-root-input-publication.log` has the one actual root publication/reload/invalidation test. These are focused acceptance, not full gates.

The exact new contexts under `raw/recovery-root-input-contexts` support **accepted** judgments for both `okf-profile#contract` (Symbol input names) and `workplan-prose-assessment#contract` (Crate-root inputs). Each new marker has the specific actual assertion owner. Their newly stated semantics agree with the source and execution above; prior unchanged paragraphs retain their earlier judgments. The obvious impacted contexts are these two Contracts, matching the primary's 92-context comparison. No independent claim is made that every other untracked or future consumer is unaffected. Full lifecycle, required inventory, broad gates and native-host/CI acceptance remain separate.
