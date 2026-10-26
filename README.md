<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

<p align="center">
  <img src="docs/raf-logo.svg" alt="RAF: a black cat biting a toolbox with a wrench" width="240">
</p>

# RAF — Rust Agentic Foundations

RAF is a set of dev-dependency libraries and cargo subcommands, not yet
published, that keep Rust
repositories free of duplicated helpers and keep their documentation an accurate,
current account of the system. A census finds near-duplicate helper code and
ratchets recorded debt down; a spec-first workplan reconciles capability documents
with the code, tests and examples that implement them. Both are built to be driven
by humans and coding agents alike, and both fail hard with a message that says how
to fix the problem.

## Crates

Delivered contracts and the remaining Census designs are listed in the
[capability catalogue](docs/capabilities/index.md), with resolved bindings and
generated verification matrices. This repository runs the workplan gate on itself
(`tests/self_workplan.rs`), and the `use`-path resolution of `raf-source` is
checked against `rustc` on generated programs
(`crates/raf-source/tests/rustc_oracle.rs`).
`raf-workplan` delivers its document model, generated documentation blocks,
reconcile step, `verify` and completion gate, entity history, execution evidence
and its claims, prose assessments, two-way coverage and the current-rationale
check, and `cargo-workplan` runs `check`, `status`, `docs sync|preview`,
`change new|impact|reconcile|complete`, `verify`, `history`,
`evidence record|claims|for`, `assess` and `coverage code|tests`: their contract
is stated in [workplan-cli](docs/capabilities/workplan-cli.md). The
[complete policy](docs/capabilities/workplan-strict.md) is delivered and applies
to every check, reconciliation and completion. It enforces section content,
package ownership, claims, line coverage and change outcomes, with explicit debt
recording that can only shrink.

`raf-census` delivers normal form 1, fingerprints, grouping, canonical RDF reports,
the strict ledger, the production Gate, counted debt and bounded migration.
`cargo-census` supplies registered `check`, `migrate` and direct `debt --write`. Complete detector/provider
execution and the full Census command interface retain their own capability states
in the catalogue.

| Crate | Kind | Purpose | Status | Specification |
|---|---|---|---|---|
| `raf-okf` | library | Reads and writes the RAF profile of OKF documents. | **delivered** | [okf-profile](docs/capabilities/okf-profile.md) |
| `raf-rdf` | library | Canonical ground RDF 1.2 reports over native PurRDF storage. | **delivered** | [rdf-report-profile](docs/capabilities/rdf-report-profile.md) |
| `raf-source` | library | Indexes Rust source so tools can bind documents to code. | **delivered** | [source-index](docs/capabilities/source-index.md) |
| `raf-census` | library | The duplicate-helper census, strict ledger, counted debt, migration and Gate. | **delivered** for the capabilities listed — not yet published | [census-normal-form](docs/capabilities/census-normal-form.md), [census-ledger](docs/capabilities/census-ledger.md), [census-debt-ratchet](docs/capabilities/census-debt-ratchet.md), [census-migration](docs/capabilities/census-migration.md) |
| `cargo-census` | cargo subcommand | Runs registered checks, migrates ledgers and writes admitted debt through the production Gate. | **complete command interface delivered** — not yet published | [census-migration](docs/capabilities/census-migration.md), [census-debt-ratchet](docs/capabilities/census-debt-ratchet.md), [census-cli](docs/capabilities/census-cli.md) |
| `raf-workplan` | library | The workplan document model, generated documentation blocks, reconcile step, completion gate, entity history, evidence and claims, prose assessments, two-way coverage and current rationale. | **delivered** for the capabilities listed — not yet published | [workplan-document-model](docs/capabilities/workplan-document-model.md), [workplan-generated-docs](docs/capabilities/workplan-generated-docs.md), [workplan-reconcile](docs/capabilities/workplan-reconcile.md), [workplan-complete-gate](docs/capabilities/workplan-complete-gate.md), [workplan-entity-history](docs/capabilities/workplan-entity-history.md), [workplan-evidence-claims](docs/capabilities/workplan-evidence-claims.md), [workplan-prose-assessment](docs/capabilities/workplan-prose-assessment.md), [workplan-two-way-coverage](docs/capabilities/workplan-two-way-coverage.md), [workplan-current-rationale](docs/capabilities/workplan-current-rationale.md) |
| `cargo-workplan` | cargo subcommand | Runs the workplan from the command line. | **delivered** — not yet published | [workplan-cli](docs/capabilities/workplan-cli.md) |

## Usage

A project consumes RAF as dev-dependencies. The workplan gate runs on this
repository. A repository binary or xtask can install the Census Gate with its
registered rules and extensions. For read-only registered checking, expose that
Gate in an example and run `cargo census check --package PACKAGE --example EXAMPLE`.
Migration and direct debt commands use the same library paths. The crates are not published yet.

```toml
[dev-dependencies]
raf-workplan = "0.1"
```

```rust
// tests/workplan.rs
#[test]
fn workplan_is_reconciled() {
    raf_workplan::Gate::workspace(env!("CARGO_MANIFEST_DIR"))
        .assert_clean();
}
```

The census gate, once built, takes the same shape:

```toml
[dev-dependencies]
raf-census = "0.1"
```

```rust
// tests/census.rs
#[test]
fn no_new_duplicate_helpers() {
    raf_census::Gate::workspace(env!("CARGO_MANIFEST_DIR"))
        .assert_clean();
}
```

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md). Run `make check` before sending a change
and `make hooks` once to install the pre-commit hook. Agents should also read
[AGENTS.md](AGENTS.md).

## Licensing

Code is licensed `MIT OR Apache-2.0 OR MulanPSL-2.0`, and documentation is
licensed CC-BY-4.0. See [LICENSING.md](LICENSING.md) for the full statement.

## Brand

The name, logo and colour system are described in [docs/BRAND.md](docs/BRAND.md).
