# Issue #42: raf-workplan: root discovery without a cargo metadata child process

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: enhancement, crate:raf-source, crate:raf-workplan

## Body

## Context
Root discovery in `raf-workplan` spawns `cargo metadata --format-version 1` and hand-scans its JSON for `workspace_root` (`crates/raf-workplan/src/root/json.rs:4-40`, `crates/raf-workplan/src/exec.rs:74,84`, `docs/capabilities/workplan-cli.md:255-270`). That costs a child process, a bespoke RFC 8259 validator and a 64 MiB / 300 s spawn bound, only to find a directory that `raf-source` can already find: it parses `Cargo.toml` workspaces and members (`crates/raf-source/src/manifest.rs`, `workspace.rs`).

Goals: no JSON; subsume and extend (`raf-source` should own this lookup); wasmable core (a process spawn is not available in a wasm host); performance.

## Scope
- [ ] Add workspace-root discovery to `raf-source`: nearest ancestor `Cargo.toml` with a `[workspace]` table, resolved through the `Tree` trait so it runs on `MemoryTree`.
- [ ] Switch `raf-workplan` root discovery to it and delete `root/json.rs` and the metadata spawn with its size and time bound.
- [ ] Decide and document the behaviour for a package that is a member versus a standalone package and for `exclude`d directories, with hard failure on ambiguity.
- [ ] Update `workplan-cli.md` (root discovery section and constraints).

## Acceptance criteria
- [ ] A fixture workspace tree on `MemoryTree` resolves the root from a nested member directory without any `Runner` invocation (the `FakeRunner` records zero calls).
- [ ] Standalone package, nested workspace and excluded-directory fixtures each resolve to the documented root or fail hard with a finding.
- [ ] `crates/raf-workplan/src/root/json.rs` is gone and no `cargo metadata` string remains in `raf-workplan` sources.
- [ ] The existing root-discovery CLI tests pass unchanged against the new implementation.

## Related
- #40 (JSON removal), #46 (wasm core), #45 (host process adapter)
- #5 (census reference scan has the same `cargo metadata` dependency)


## Comments (0)

