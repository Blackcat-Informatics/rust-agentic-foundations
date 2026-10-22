<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Adoption authored-impact correction

The adjacent patch removes only 35 unchanged authored-section declarations from `repository-adoption.affects` (112 to 77 entries). A read-only comparison against current merge base `1b2291298a86bc18d472c5676cde2d965014bf31`, retaining section headings and generated markers but excluding generated contents, reconfirmed the same earlier list after the latest root-helper/repository Contract+Examples and OKF changes. Actual production adoption reconciliation must confirm the exact untouched list before application; no shell comparison substitutes for its verdict.

| Capability | Removed unchanged sections |
| --- | --- |
| okf-profile | Purpose, Examples |
| source-index | Examples |
| workplan-cli | Purpose, Explanation |
| workplan-complete-gate | Purpose, Examples |
| workplan-current-rationale | Purpose, Examples, Verification |
| workplan-document-model | Purpose, Examples, Explanation, Verification |
| workplan-entity-history | Purpose, Examples, Explanation |
| workplan-evidence-claims | Purpose, Examples, Explanation, Verification |
| workplan-generated-docs | Purpose, Explanation, Verification |
| workplan-prose-assessment | Purpose, Examples, Explanation, Verification |
| workplan-reconcile | Purpose, Examples, Explanation, Verification |
| workplan-two-way-coverage | Purpose, Examples, Explanation |

`affects` declares authored text changed, not every section read, judged or tested. The patch leaves all 16 `changes` capabilities, every implemented item, `contract_effect: changes`, all 113 assessment inventory entries and zero code/test/claim/orphan/debt adoption requirements intact. No assessment, test binding, source, document section or required outcome is removed. Full-section judgments remain valid required acceptance even where their already-correct prose needed no edit. This is explicit impact correction, not padding, effect suppression or a lighter policy.

No product edit, build, index mutation or lifecycle invocation was performed. The patch is prepared metadata for the owner to apply only after the actual reconcile result agrees. The independent selected root snapshot and actual completion/default-gate qualification remain separately required.
