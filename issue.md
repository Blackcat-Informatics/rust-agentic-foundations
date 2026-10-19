# Issue #27: raf-source: the registry scan example reads the clock

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

## Context
`crates/raf-source/examples/registry_scan.rs` is an example binary that scans a registry of crates. It measures time: it applies a per-crate timeout and polls with `sleep`. The repository rule is that time-based testing and wall-clock measurement are not allowed; the source-index crate's tests were converted to counted work and a repository gate keeps clock spellings out of the crate's source and tests, and the gate exempts `examples/`.

## Scope
- [ ] Replace the per-crate timeout and the sleep poll with a bound on counted work (the index already reports `Usage` and refuses at its budgets), or remove the example if its purpose is served by the differential test against rustc.
- [ ] Extend the gate to cover `examples/` once the example is clean, or state in the gate why it stays exempt.

## Acceptance criteria
- [ ] `grep -nE 'std::time|Instant|Duration|sleep|recv_timeout|SystemTime' crates/raf-source/examples` is empty, or the example is gone.
- [ ] The repository gate passes with `examples/` in scope.


## Comments (0)

