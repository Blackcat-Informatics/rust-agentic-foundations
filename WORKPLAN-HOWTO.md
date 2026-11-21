<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Use Workplan in a Rust repository

Workplan answers two questions about a documented guarantee. Does its named implementation still exist, and does the document identify the test that supports it? When the guarantee changes, Workplan tracks which sections need rewriting and assessment. Your normal Cargo test job establishes whether the implementation passes those tests.

## Install the command

On native Linux with Rust 1.98.1 or newer, install the CLI from a pinned RAF revision. Keep the same revision for developer machines, CI and any RAF dev-dependency.

```sh
cargo install --locked --git https://github.com/Blackcat-Informatics/rust-agentic-foundations.git \
  --rev 05138a5ea278377d745fca9809cae116ae37a667 cargo-workplan
```

Make sure Cargo's install directory (normally `$HOME/.cargo/bin`) is on `PATH`. Run the commands below from your workspace root. From another directory, pass the root itself, for example `cargo workplan --root /path/to/workspace check`.

## Bind one guarantee

Start with a small API whose ordinary and boundary behaviour you can explain. The example uses a library package named `demo`. In an existing repository, substitute your package, symbols, example and test wherever they appear.

`src/lib.rs`

```rust
//! Helpers for choosing port numbers.

/// Returns the next port number, or None at the upper limit.
pub fn next_port(port: u16) -> Option<u16> {
    port.checked_add(1)
}

#[cfg(test)]
mod tests {
    #[test]
    fn advances_until_the_last_port() {
        assert_eq!(super::next_port(443), Some(444));
        assert_eq!(super::next_port(65_534), Some(65_535));
        assert_eq!(super::next_port(65_535), None);
    }
}
```

`examples/ports.rs`

```rust
fn main() {
    assert_eq!(demo::next_port(443), Some(444));
}
```

Create `work/coverage/default.md`. Use Cargo **package names** in `scope`. The source bindings below use Rust item paths.

```yaml
---
type: Coverage
id: coverage:default
scope: [demo]
exclusions: []
---
```

Create `docs/capabilities/next-port.md`. A delivered capability has seven sections in this order. Write at least twenty words of authored prose in each. Name the bound example in a code span in Usage. Attach the test marker directly to the guarantee or constraint it supports. Leave the verification matrix delimiters in place for Workplan to fill.

```markdown
---
type: Capability
id: cap:next-port
title: "Next port number"
description: "Advances a port number within the unsigned sixteen-bit range."
state: delivered
bindings:
  code:
    - demo::next_port
  tests:
    - demo::tests::advances_until_the_last_port
  examples:
    - demo::example_crate::ports
depends: []
---

# Next port number

## Purpose

A caller walking a range of port numbers needs a way to advance one position and to recognise when it has reached the end of the range.

## Contract

**Advance one port.** Given any port below the maximum, the function returns the next number wrapped in Some, including the step from the penultimate port to the last one.
<!-- raf:tested-by demo::tests::advances_until_the_last_port -->

## Usage

Pass the current port to the function as shown in `demo::example_crate::ports`. Use the returned number for the next iteration and end the iteration on None.

## Examples

Starting at port 443 gives the caller port 444. The ports example demonstrates this step with the same public function an application uses.

## Constraints

- At port 65535 the function returns None. A caller that needs to restart its scan at zero must make that choice explicitly after reaching the upper limit.
<!-- raf:tested-by demo::tests::advances_until_the_last_port -->

## Explanation

Checked addition expresses the boundary in the return type. Every valid port number, including zero, therefore remains available to the caller as an ordinary input.

## Verification

The bound unit test covers an ordinary port, the last successful increment, and the first input with no successor. The example shows the ordinary case through the public API.

<!-- raf:generated:verification-matrix -->
<!-- raf:generated:end -->
```

## Generate the catalogue and run the gate

```sh
cargo workplan docs sync --copyright "2026 Demo" --licence CC-BY-4.0
git add src/lib.rs examples/ports.rs docs/capabilities work/coverage/default.md
cargo workplan check
cargo workplan verify
```

Use your project's copyright and documentation licence. Sync creates `docs/capabilities/index.md` and fills the matrix with resolved bindings. Git tracking matters because Workplan validates relative links against the index. Commit the authored documents and generated output together. A clean check exits zero. In CI, run `cargo workplan check` on the checkout alongside the project's existing Rust tests.

| Finding | What to inspect |
| --- | --- |
| Broken link in the generated catalogue | Stage the capability file and generated index in Git. |
| Unresolved binding | Check the Rust crate, module, target and item name. An integration test uses `crate_name::test_crate::target_name::test_name`. |
| Missing coverage or unclaimed code | Set the coverage scope, then use `cargo workplan coverage code`, `coverage tests` and `coverage lines` to locate gaps. Bind each item to its contract or give a supported exclusion a specific reason. |
| Existing strict debt during adoption | Fix document and binding errors first. Run `cargo workplan debt --write` deliberately to record admissible debt in `work/debt/default.md`. Review and commit it. Run the writer again when debt shrinks. |

### Run the check from a Rust test

For a package at the workspace root, add the library to its `Cargo.toml` and use this integration test. In a virtual workspace, put the dev-dependency in a member package and pass the directory containing the workspace's `docs/capabilities` to `Gate::workspace`. If the package is in the coverage scope, account for the gate test in the repository's own verification bindings.

```toml
[dev-dependencies]
raf-workplan = { git = "https://github.com/Blackcat-Informatics/rust-agentic-foundations.git", rev = "05138a5ea278377d745fca9809cae116ae37a667" }
```

```rust
// tests/workplan.rs
#[test]
fn documented_contract_is_current() {
    raf_workplan::Gate::workspace(env!("CARGO_MANIFEST_DIR")).assert_clean();
}
```

Run `cargo test --test workplan` from the owning package to resolve the new dependency and check the gate. Commit the updated lockfile for a reproducible `cargo test --locked --test workplan` in CI. RAF's own repository also requires every recorded assessment to be current and accepted. Its test shows how to add that policy. [RAF's Workplan gate](tests/self_workplan.rs)

## Change a delivered contract

With the initial documents committed, open a change against the capability and name each affected section. Reconciliation reads the merge base with the symbolic default branch `refs/remotes/origin/HEAD`. Use a clone that has that reference and the necessary commit history.

```sh
cargo workplan change new port-range --capability cap:next-port \
  --affects 'cap:next-port#contract'
cargo workplan change impact port-range
```

Edit `work/changes/port-range.md` to state the intended outcome. Put proposed wording in the affected section under `> [!PROPOSED change:port-range]`. As the behaviour lands, replace the proposal with current prose, update the source and test bindings, and give each item an outcome. Use `implemented`, `dropped` with a reason, or `moved:change:<slug>` with a reason and an existing destination record.

```sh
cargo workplan docs sync --copyright "2026 Demo" --licence CC-BY-4.0
cargo workplan assess 'cap:next-port#contract' --context
```

Read the section against the inputs the command prints. Record `accepted` only when the prose matches those inputs. Repeat for each affected authored section and address any other obligations that reconciliation reports.

```sh
cargo workplan assess 'cap:next-port#contract' --verdict accepted --by human
cargo workplan change reconcile port-range
cargo workplan verify port-range
cargo workplan change complete port-range
cargo workplan check
```

Successful completion removes the change record and returns a capability to delivered when this was its last implementing change. Review and commit the resulting documents and assessments. Verification and completion inspect the repository as data. Building Rustdoc is a separate operation that requires explicit `--trust-docs` for that invocation and runs build scripts and procedural macros with the caller's permissions. [Completion contract](docs/capabilities/workplan-complete-gate.md) · [Command reference](docs/capabilities/workplan-cli.md)
