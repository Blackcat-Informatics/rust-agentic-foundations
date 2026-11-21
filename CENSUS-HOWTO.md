<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Use Census in a Rust repository

Census groups Rust helpers with the same normalized body or forwarding pattern. Use a baseline to see what already matches, a ledger to declare the repository's policy, and a check in CI to catch new copies. Start with structural duplication. Add named jobs when you need to enforce a particular implementation as the home of a helper.

## Inspect the workspace

On native Linux with Rust 1.98.1 or newer, select a RAF checkout containing this guide. From that checkout, record its commit and install the CLI at that revision. Use the same commit on developer machines, in CI and for the RAF dev-dependency.

```sh
RAF_CHECKOUT=$(pwd -P)
RAF_REV=$(git rev-parse HEAD)
cargo install --locked --git https://github.com/Blackcat-Informatics/rust-agentic-foundations.git \
  --rev "$RAF_REV" cargo-census
```

From the consumer workspace, run the self-test and inspect its baseline.

```sh
cargo census self-test
cargo census baseline --format turtle
```

Make sure Cargo's install directory (normally `$HOME/.cargo/bin`) is on `PATH`. Run the baseline in the consumer workspace, or select it with `cargo census --workspace /path/to/workspace baseline`. Baseline discovery needs a Cargo manifest and readable sources. It prints an RDF 1.2 report using the default Census options and makes no policy decision. To save a report, add `--out baseline.ttl`. The self-test checks the installed detectors against seeded cases independently of your workspace.

## Capture the production lint result

Use the Cargo lint tables below for a standalone package. In a workspace, put them under `[workspace.lints.rust]` and `[workspace.lints.clippy]` in the root manifest. Each member inherits them with `workspace = true` under `[lints]`.

```toml
[lints.rust]
unused = { level = "deny", priority = -1 }
dead_code = "deny"
unreachable_pub = "deny"
unused_crate_dependencies = "warn"

[lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
allow_attributes = "deny"
allow_attributes_without_reason = "deny"
used_underscore_binding = "deny"
dbg_macro = "deny"
todo = "deny"
unimplemented = "deny"
```

If the workspace has no lockfile yet, run `cargo generate-lockfile` from its root. Run the production check from the trusted build or hook. In the shell where you selected `RAF_CHECKOUT`, run these commands from the consumer workspace. Use a temporary directory outside the inspected repository.

```sh
evidence_dir=$(mktemp -d)
export RAF_CENSUS_LINT_EVIDENCE="$evidence_dir/production.ttl"
"$RAF_CHECKOUT/.githooks/production-clippy" "$PWD"
```

The helper runs Clippy on the workspace libraries and binaries with the lockfile and required denied lints. A private helper called only from a test can be dead in that build. A passing `--all-targets` result cannot satisfy the check. A dev-dependency used only in tests can remain in `[dev-dependencies]`.

Use a fixed checkout and a trusted compiler and build environment. Generate the required lockfile with `cargo generate-lockfile` if it is missing. The receipt covers member manifests, the lockfile, workspace compiler configuration, Clippy configuration from each member up to the workspace root, shipping modules and directly named literal file inclusions. Standalone tests, examples and benches do not supply source roots for the production pass.

Place literal file inclusions in the consuming source. Census refuses a file-inclusion name in a declarative macro definition or one forwarded as a macro argument because the expansion can resolve the path at another call site. The scan skips directly `#[cfg(test)]`-guarded items and statements where it can prove they are inactive. Inputs to an unknown attribute macro still count because the macro can rewrite a nested declaration. Census also reports first-party build scripts, unverified target feature requirements, renamed or dynamic inclusions, and local configuration that changes the compiler invocation. Keep dependency sources and toolchain fixed when producing and consuming a receipt.

Keep the receipt in the build-owned directory. The CLI accepts its path through the exported variable or `check --lint-evidence /absolute/path/to/production.ttl`. Missing evidence, changed inputs, a weakened manifest or a failed compiler run fails the default check. Repeat the capture with a fresh destination after changing the sources or policy. Use `check --no-production-lints` for an explicitly structural check. Baseline inspection and debt recording need no compiler receipt.

## Install the ledger

From the workspace or one of its members, initialize the structural policy, inspect the resulting ledger, then check the source.

```sh
cargo census init
cargo census dump-ledger --format turtle
cargo census check
```

`init` creates canonical `helpers-ledger.ttl` at the workspace root. Its sole scan root is `.` and it declares no jobs or allowances. An existing ledger path makes initialization fail and preserves the existing entry. Initialization leaves any debt record untouched.

`dump-ledger` validates and prints the ledger. `check` evaluates it against the source and debt record. An absent `census-debt.ttl` counts as empty. A clean workspace passes immediately. A finding includes the group or policy violation to inspect. For `check`, exit status 0 means clean. Status 1 signals findings or an evaluation error. Status 2 signals invalid command usage or a ledger that requires Rust registrations the CLI lacks.

The ledger reader requires canonical bytes. Even a whitespace edit can invalidate the file, and a policy edit changes the content-derived `BASE`. For a custom policy, construct the graph with `raf_rdf::Builder` and serialize it with `Graph::to_text(Format::Turtle)`. The example constructs a job, names its home and attaches an extension using that writer. [Ledger construction example](crates/raf-census/examples/support/ledger.rs) · [Ledger fields](docs/capabilities/census-ledger.md)

## Adopt existing duplication

Inspect the reported occurrences together. Consolidate copies that should share one implementation. When you deliberately accept some existing duplication as the starting point, publish and review the exact record.

```sh
cargo census debt --write
cargo census check
git add helpers-ledger.ttl census-debt.ttl
```

Commit the ledger and any adopted debt. In CI, capture the production Clippy result, then run the read-only `cargo census check`. A new duplicate group or a count above its record fails. Removing a copy can also produce a finding because the record now grants more than the code uses. Rerun `debt --write` to lower it, review the diff and run `check` again. The writer refuses growth against an existing record of the same normal form. Hard policy errors need a code or policy repair before debt can be written. [Debt ratchet](docs/capabilities/census-debt-ratchet.md)

To exercise structural matching, use this `src/lib.rs` in a scratch library with the starter ledger and an empty debt record. Run `cargo census check --no-production-lints`. It finds the two bodies in one group. Rename the locals and the match persists. Remove the second helper and the structural check passes. For a full check, capture fresh production evidence after each source edit.

```rust
pub fn scaled_total(samples: &[u64]) -> u64 {
    let mut sum = 0;
    for sample in samples {
        sum += sample * 3;
    }
    sum
}

pub fn batch_total(values: &[u64]) -> u64 {
    let mut total = 0;
    for value in values {
        total += value * 3;
    }
    total
}
```

## Find the source of a finding

| Question | Command |
| --- | --- |
| Which sources, groups and classifications did the gate see? | `cargo census index` |
| Which occurrences and fingerprints belong to a symbol? | `cargo census fingerprint demo::scaled_total` (use your crate and item name) |
| Which occurrences matched a declared job? | `cargo census job JOB_ID` |
| How many copies did that job classify? | `cargo census count JOB_ID` |

The starter ledger declares no job IDs. For policy beyond structural grouping, a job can identify a home symbol and forbidden competitors. A reference scan can compare local code with sources sealed from the Cargo lockfile and cached registry archives. Supply consumer-owned Rust rules and extension validators through the library gate when the policy requires them. The native CLI never launches the consumer to discover registrations. [Rule registration](docs/capabilities/census-rule-api.md) · [Reference scans](docs/capabilities/census-reference-scan.md)

## Run the gate from a Rust test

To use the same structural policy from your test suite, add the library to a package at the commit selected above. In a virtual workspace, run this command in the member that owns the integration test. Census discovers the workspace root from the member directory.

```sh
cargo add --dev raf-census --git https://github.com/Blackcat-Informatics/rust-agentic-foundations.git \
  --rev "$RAF_REV"
```

```rust
// tests/census.rs
#[test]
fn duplicate_helpers_match_the_record() {
    let gate = raf_census::Gate::workspace(env!("CARGO_MANIFEST_DIR"));
    let receipt = std::env::var_os("RAF_CENSUS_LINT_EVIDENCE")
        .expect("capture the production Clippy run before running this test");
    let run = raf_census::ProductionLintRun::read_external(
        &gate.root().expect("resolve the workspace"),
        std::path::Path::new(&receipt),
    ).expect("read the build-owned production lint result");
    gate.with_production_lint_run(run).assert_clean();
}
```

Resolve the new dependency, commit the updated lockfile and capture fresh production Clippy evidence. Run `cargo test --locked --test census` from the owning package with the receipt in the environment. For a policy with custom rules or required providers, share the gate configuration across the enforcing entry points. RAF's own test shows a locked registry check over the workspace and its separate fuzz root. [RAF's Census gate](tests/self_census.rs) · [CLI reference](docs/capabilities/census-cli.md)
