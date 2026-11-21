<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Use Census in a Rust repository

Census groups Rust helpers with the same normalized body or forwarding pattern. Use a baseline to see what already matches, a ledger to declare the repository's policy, and a check in CI to catch new copies. Start with structural duplication. Add named jobs when you need to enforce a particular implementation as the home of a helper.

## Inspect the workspace

On native Linux with Rust 1.98.1 or newer, install the CLI from a pinned RAF revision. Use the same revision for developer machines, CI and any RAF dev-dependency.

```sh
cargo install --locked --git https://github.com/Blackcat-Informatics/rust-agentic-foundations.git \
  --rev 05138a5ea278377d745fca9809cae116ae37a667 cargo-census
cargo census self-test
cargo census baseline --format turtle
```

Make sure Cargo's install directory (normally `$HOME/.cargo/bin`) is on `PATH`. Run the baseline in the consumer workspace, or select it with `cargo census --workspace /path/to/workspace baseline`. Baseline discovery needs a Cargo manifest and readable sources. It prints an RDF 1.2 report using the default Census options and makes no policy decision. To save a report, add `--out baseline.ttl`. The self-test checks the installed detectors against seeded cases independently of your workspace.

## Install the ledger

Create `helpers-ledger.ttl` at the workspace root. This complete starter ledger selects the current workspace (`.`), normal form 1 and report form 5. It lets you enforce structural duplicate groups immediately, with no named jobs. Copy all six lines exactly, including the final newline.

```turtle
VERSION "1.2"
BASE <https://blackcatinformatics.ca/.well-known/genid/raf-report/b6b4b0160ca994e7e3840c07807d4adb56ba5a2e003a910ad5efdf316f211c54>
<> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://blackcatinformatics.ca/raf/report/1#Ledger> .
<> <https://blackcatinformatics.ca/raf/report/1#normalForm> "1"^^<http://www.w3.org/2001/XMLSchema#integer> .
<> <https://blackcatinformatics.ca/raf/report/1#reportForm> "5"^^<http://www.w3.org/2001/XMLSchema#integer> .
<> <https://blackcatinformatics.ca/raf/report/1#root> "." .
```

```sh
cargo census dump-ledger --format turtle
cargo census check
```

The first command validates and prints the ledger. The second evaluates it against the source and debt record. An absent `census-debt.ttl` counts as empty. A clean workspace passes immediately. A finding includes the group or policy violation to inspect. For `check`, exit status 0 means clean. Status 1 signals findings or an evaluation error. Status 2 signals invalid command usage or a ledger that requires Rust registrations the CLI lacks.

The ledger reader requires canonical bytes. Even a whitespace edit can invalidate the file, and a policy edit changes the content-derived `BASE`. For a custom policy, construct the graph with `raf_rdf::Builder` and serialize it with `Graph::to_text(Format::Turtle)`. The example constructs a job, names its home and attaches an extension using that writer. [Ledger construction example](crates/raf-census/examples/support/ledger.rs) · [Ledger fields](docs/capabilities/census-ledger.md)

## Adopt existing duplication

Inspect the reported occurrences together. Consolidate copies that should share one implementation. When you deliberately accept some existing duplication as the starting point, publish and review the exact record.

```sh
cargo census debt --write
cargo census check
git add helpers-ledger.ttl census-debt.ttl
```

Commit the ledger and any adopted debt. Run **only the read-only `cargo census check`** as the CI gate. A new duplicate group or a count above its record fails. Removing a copy can also produce a finding because the record now grants more than the code uses. Rerun `debt --write` to lower it, review the diff and run `check` again. The writer refuses growth against an existing record of the same normal form. Hard policy errors need a code or policy repair before debt can be written. [Debt ratchet](docs/capabilities/census-debt-ratchet.md)

For a small demonstration, use this `src/lib.rs` in a scratch library with the starter ledger. With an empty debt record, `check` finds the two bodies in one group. Rename the locals and the match persists. Remove the second helper and the empty-debt check passes.

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

To use the same structural policy from your test suite, add the library to a package's `Cargo.toml`. In a virtual workspace, choose a member package that owns the integration test. Census discovers the workspace root from the member directory.

```toml
[dev-dependencies]
raf-census = { git = "https://github.com/Blackcat-Informatics/rust-agentic-foundations.git", rev = "05138a5ea278377d745fca9809cae116ae37a667" }
```

```rust
// tests/census.rs
#[test]
fn duplicate_helpers_match_the_record() {
    raf_census::Gate::workspace(env!("CARGO_MANIFEST_DIR")).assert_clean();
}
```

Run `cargo test --test census` from the owning package to resolve the new dependency and check the gate. Commit the updated lockfile for a reproducible `cargo test --locked --test census` in CI. For a policy with custom rules or required providers, share the gate configuration across the enforcing entry points. RAF's own test shows a locked registry check over the workspace and its separate fuzz root. [RAF's Census gate](tests/self_census.rs) · [CLI reference](docs/capabilities/census-cli.md)
