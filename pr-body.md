<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

## Delivered behavior

Closes #4.

Adds one strict native RDF ledger and the production Census Gate, including home/variant/copy classification, exact distinct sanctions, staleness failures and registered rule/extension admission. The counted debt record refuses growth and stale slack; its writer uses the same evaluation and preserves the previous record on failure.

Adds real `cargo census debt --write` and `cargo census migrate` commands. Migration reads older native indexes or foreign TOML/JSON inputs without modifying them, retains dialect metadata in the native extension boundary, and exposes every unmapped row with a nonzero result. RDF parsing, canonicalization, bounded JSON and ECMA matching reuse PurRDF.

Census report layout advances to 3 for counted debt. Normal form stays at 1 and its fingerprint goldens remain unchanged. Ordinary readers reject earlier report layouts; regenerate baseline and index reports from source. Native index migration requires the supported report layout; foreign dialect migration retains its explicit TOML/JSON input route.

Both workspaces pin the existing transitive `ctutils` dependency to 0.4.4, the patched version identified by [RUSTSEC-2026-0385](https://github.com/RustSec/advisory-db/blob/main/crates/ctutils/RUSTSEC-2026-0385.md).

Native test jobs retain backtrace line tables and disable incremental caches after the Linux runner exhausted disk during binding verification. Test commands, assertions and the release/fuzz profiles are unchanged.

The ledger, debt and migration capability documents describe the delivered APIs and commands. Pattern, reference and preferred-provider declarations are validated here; their full execution remains in #5, and unavailable execution refuses rather than producing partial clean results. The complete CLI and registered-target protocol remain in #6 and #12. Existing partial detector implementation is preserved without claiming those parents complete.

## Validation

- All required local gates are qualified on the patched lockfiles. The full run passed fmt, warnings-denied all-target Clippy, workspace/release tests, docs, deny, REUSE, MSRV and all five 5,000-run fuzz targets, then stopped because the managed host lacked cross targets. After installing those exact targets on the existing compiler, `make cross-check licenses issue-refs` passed with exit 0. The original stopped invocation remains recorded as failed; its completed coverage is reused. Both dependency gates, all 195 Census/CLI consumer tests and normal signed hooks also passed.
- Native Workplan completion passed for the ledger, debt, migration and affected report contract. All 133 assessments are current and accepted; all nine first-party packages have complete coverage without debt or exclusions.
- The focused Census/library/real-command qualification executed 195 tests successfully. Independent review findings were corrected, including report-layout versioning and retained oracle provenance.
- Current main is integrated. The affected Git fixture binary passed all 22 tests, and normal signed commit hooks passed, including the integrated repository policy and assessment gates. All twelve hosted checks pass on 3521144. Every host accounts for all 2,816 capability bindings: Linux 2,814 passed / 2 platform-not-applicable; macOS 2,802 / 14; Windows 2,620 / 196, with zero failures.
