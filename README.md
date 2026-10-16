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

`raf-okf`, `raf-source`, `raf-workplan` and `cargo-workplan` are **delivered**.
`raf-workplan` delivers its document model, generated documentation blocks,
reconcile step, `verify` and completion gate, and `cargo-workplan` runs `check`,
`status`, `docs sync|preview`, `change new|impact|reconcile|complete` and
`verify`. Its commands `history`, `evidence record|claims|for`, `assess` and
`coverage code|tests` are built and **implementing**: their contract is still
proposed in [workplan-cli](docs/capabilities/workplan-cli.md). The other crates are
**designed — not yet published**. The linked
capability documents are the specification the crates are built against.

| Crate | Kind | Purpose | Status | Specification |
|---|---|---|---|---|
| `raf-okf` | library | Reads and writes the RAF profile of OKF documents. | **delivered** | [okf-profile](docs/capabilities/okf-profile.md) |
| `raf-source` | library | Indexes Rust source so tools can bind documents to code. | **delivered** | [source-index](docs/capabilities/source-index.md) |
| `raf-census` | library | The duplicate-helper census, ledger and debt ratchet, with a test gate. | **designed — not yet published** | [census-normal-form](docs/capabilities/census-normal-form.md) |
| `cargo-census` | cargo subcommand | Runs the census from the command line. | **designed — not yet published** | [census-cli](docs/capabilities/census-cli.md) |
| `raf-workplan` | library | The workplan document model, generated documentation blocks, reconcile step and completion gate. | **delivered** for the capabilities listed — not yet published | [workplan-document-model](docs/capabilities/workplan-document-model.md), [workplan-generated-docs](docs/capabilities/workplan-generated-docs.md), [workplan-reconcile](docs/capabilities/workplan-reconcile.md), [workplan-complete-gate](docs/capabilities/workplan-complete-gate.md) |
| `cargo-workplan` | cargo subcommand | Runs the workplan from the command line. | **delivered** — not yet published | [workplan-cli](docs/capabilities/workplan-cli.md) |

## Usage

A project consumes RAF as dev-dependencies. The workplan gate below is built; the
census gate is designed and not available yet. Neither crate is published yet.

```toml
[dev-dependencies]
raf-census = "0.1"
raf-workplan = "0.1"
```

```rust
// tests/census.rs
#[test]
fn no_new_duplicate_helpers() {
    raf_census::Gate::workspace(env!("CARGO_MANIFEST_DIR"))
        .assert_clean();
}
```

```rust
// tests/workplan.rs
#[test]
fn workplan_is_reconciled() {
    raf_workplan::Gate::workspace(env!("CARGO_MANIFEST_DIR"))
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
