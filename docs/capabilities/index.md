<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

<!-- raf:generated:catalogue -->
## Delivered

| Capability | Description | Gaps |
| --- | --- | --- |
| [okf-profile](./okf-profile.md) | The raf-okf crate: parse OKF Markdown, validate the RAF profile with closed key sets, and write documents back deterministically. |  |
| [source-index](./source-index.md) | The raf-source crate: a Tree abstraction, package discovery, mod-tree following with #\[path\], and a symbol and use-alias index. |  |
| [workplan-current-rationale](./workplan-current-rationale.md) | Shared architecture pages that state resolved decisions, and the conditions to revisit them. |  |
| [workplan-generated-docs](./workplan-generated-docs.md) | Generated blocks for the verification matrix, relationship table and capability catalogue, and the deterministic sync. |  |
| [workplan-reconcile](./workplan-reconcile.md) | change new, change impact and change reconcile: opening a change, listing what it touches, and producing the documentation obligations. |  |

## Implementing

| Capability | Description | Gaps |
| --- | --- | --- |
| [workplan-cli](./workplan-cli.md) | The cargo workplan subcommands, options and exit codes. |  |
| [workplan-complete-gate](./workplan-complete-gate.md) | change complete and verify: the six completion conditions, lighter obligations for changes that leave the contract alone, deletion of the record, and the acceptance scenarios. |  |
| [workplan-document-model](./workplan-document-model.md) | Capability and change documents, the four states, PROPOSED blocks, the graph, binding resolution and the check rules. |  |
| [workplan-entity-history](./workplan-entity-history.md) | A history command for a capability or decision and its bindings, built on git log --follow and -L. |  |
| [workplan-evidence-claims](./workplan-evidence-claims.md) | Persisted execution evidence, the claims that depend on each result, and invalidation when the evidence changes. |  |
| [workplan-prose-assessment](./workplan-prose-assessment.md) | Recorded assessments of explanatory prose by humans and agents, invalidated when their inputs change. |  |
| [workplan-two-way-coverage](./workplan-two-way-coverage.md) | Checks from code to documentation and from tests to claims, complementing the documentation-to-code and documentation-to-tests checks. |  |

## Designed

| Capability | Description | Gaps |
| --- | --- | --- |
| [census-cli](./census-cli.md) | The cargo census subcommands, their options and exit codes. |  |
| [census-debt-ratchet](./census-debt-ratchet.md) | A recorded set of existing duplication that can only shrink, stored in census-debt.json. |  |
| [census-detectors](./census-detectors.md) | The fourteen generic detectors the RAF census ships, each with a positive and a negative fixture. |  |
| [census-ledger](./census-ledger.md) | The helpers-ledger.toml schema: jobs, variants, distinct sets, extensions and the staleness checks. |  |
| [census-migration](./census-migration.md) | Move a legacy census baseline and ledger, or a ledger at an older normal form, to the implemented normal form and the unified ledger schema. |  |
| [census-normal-form](./census-normal-form.md) | How the RAF census normalises function bodies, fingerprints them with BLAKE3 and groups the duplicates. |  |
| [census-patterns](./census-patterns.md) | Ledger-declared \[\[pattern\]\] matchers that flag code by call, path, macro, type, literal or token sequence. |  |
| [census-reference-scan](./census-reference-scan.md) | Compare first-party code with the source of named dependency packages, resolved through cargo metadata. |  |
| [census-rule-api](./census-rule-api.md) | The Rust Rule trait for repository-specific detectors and the Gate that runs the census as a dev-dependency test. |  |
<!-- raf:generated:end -->
