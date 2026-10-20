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

`raf-okf`, `raf-source`, `raf-workplan` and `cargo-workplan` are **delivered**:
twelve capabilities, each with resolved bindings and a generated verification
matrix. This repository runs the workplan gate on itself
(`tests/self_workplan.rs`), and the `use`-path resolution of `raf-source` is
checked against `rustc` on generated programs
(`crates/raf-source/tests/rustc_oracle.rs`).
`raf-workplan` delivers its document model, generated documentation blocks,
reconcile step, `verify` and completion gate, entity history, execution evidence
and its claims, prose assessments, two-way coverage and the current-rationale
check, and `cargo-workplan` runs `check`, `status`, `docs sync|preview`,
`change new|impact|reconcile|complete`, `verify`, `history`,
`evidence record|claims|for`, `assess` and `coverage code|tests`: their contract
is stated in [workplan-cli](docs/capabilities/workplan-cli.md). Strict mode
([workplan-strict](docs/capabilities/workplan-strict.md)) is **designed**: an opt-in
set of harder rules with a debt record that can only shrink, which adds the global
option `--strict` and the commands `coverage claims`, `coverage lines` (also spelled
`coverage --reverse`) and `debt --write`. The census crates
are **designed — not yet published**: their nine capability documents are the
specification they will be built against, and no census code exists yet.

| Crate | Kind | Purpose | Status | Specification |
|---|---|---|---|---|
| `raf-okf` | library | Reads and writes the RAF profile of OKF documents. | **delivered** | [okf-profile](docs/capabilities/okf-profile.md) |
| `raf-source` | library | Indexes Rust source so tools can bind documents to code. | **delivered** | [source-index](docs/capabilities/source-index.md) |
| `raf-census` | library | The duplicate-helper census, ledger and debt ratchet, with a test gate. | **designed — not yet published** | [census-normal-form](docs/capabilities/census-normal-form.md) |
| `cargo-census` | cargo subcommand | Runs the census from the command line. | **designed — not yet published** | [census-cli](docs/capabilities/census-cli.md) |
| `raf-workplan` | library | The workplan document model, generated documentation blocks, reconcile step, completion gate, entity history, evidence and claims, prose assessments, two-way coverage and current rationale. | **delivered** for the capabilities listed — not yet published | [workplan-document-model](docs/capabilities/workplan-document-model.md), [workplan-generated-docs](docs/capabilities/workplan-generated-docs.md), [workplan-reconcile](docs/capabilities/workplan-reconcile.md), [workplan-complete-gate](docs/capabilities/workplan-complete-gate.md), [workplan-entity-history](docs/capabilities/workplan-entity-history.md), [workplan-evidence-claims](docs/capabilities/workplan-evidence-claims.md), [workplan-prose-assessment](docs/capabilities/workplan-prose-assessment.md), [workplan-two-way-coverage](docs/capabilities/workplan-two-way-coverage.md), [workplan-current-rationale](docs/capabilities/workplan-current-rationale.md) |
| `cargo-workplan` | cargo subcommand | Runs the workplan from the command line. | **delivered** — not yet published | [workplan-cli](docs/capabilities/workplan-cli.md) |

## Usage

A project consumes RAF as dev-dependencies. The workplan gate is built and runs
on this repository; the census gate is designed and does not exist yet. Neither
crate is published yet.

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
