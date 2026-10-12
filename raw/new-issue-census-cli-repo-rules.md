## Context
The census specs make the Rust `Gate` the authority for every ledger. `cargo census check` evaluates only ledgers that need no repository-registered code. When a ledger has a non-empty `[job.ext]`, or a `rule:` id that is not a built-in detector or pattern, the CLI exits 2 and names the Gate path (the repository's census test). Only the Gate writes `census-debt.json` for those ledgers. See `cap:census-cli`, `cap:census-rule-api`, `cap:census-ledger` and `cap:census-debt-ratchet`.

This issue adds a second mode: `cargo census` runs the repository's own registered rules and ledger extensions. A dialect ledger, such as one migrated from the purrdf or decodes formats, can then be checked and its debt record written from the CLI without a hand-run test.

## Scope
- [ ] Design and specify, as a change record with PROPOSED blocks in the affected capability docs, how `cargo census` reaches registered rules and extensions. The candidate approach is a mode that builds and invokes a repository test or example target, with a defined report and exit-code protocol between `cargo-census` and that target.
- [ ] `check` gives the same findings as `Gate::assert_clean` for ledgers that use `[job.ext]` and custom `rule:` ids. The mode never silently drops a registered rule.
- [ ] `debt --write` works for those ledgers through the same path, and keeps the existing refusal rules (no recorded growth, no write while non-debt findings exist).
- [ ] Hard failure when the repository target is missing, fails to build, or speaks a different protocol version. Never fall back to the built-in-only check.
- [ ] Spawned cargo follows the census subprocess rules: no shell, argv only, `--` before document-derived arguments, the workspace treated as trusted build input.
- [ ] Verification: fixtures for a dialect ledger checked through the CLI, a custom `rule:` id enforced through the CLI, and each protocol failure case.

## Acceptance criteria
- [ ] For a fixture repository with a registered `LedgerExtension` and a custom `Rule`, `cargo census check` and `Gate::assert_clean` report identical findings.
- [ ] `cargo census debt --write` produces the same `census-debt.json` as the Gate write path for that fixture.
- [ ] Every failure mode above exits non-zero with a message naming the cause.

## Depends on
The census CLI (`cargo-census`) and the census Rule, Gate and ledger crates.
