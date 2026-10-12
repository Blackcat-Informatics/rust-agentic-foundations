---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-rule-api
title: "Census Rule trait and Gate"
description: "The Rust Rule trait for repository-specific detectors and the Gate that runs the census as a dev-dependency test."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:census-detectors, cap:census-ledger]
---

# Census Rule trait and Gate

## Purpose

Some checks need structure that a declarative pattern cannot express, or concern
one repository's own types. The `Rule` trait lets a repository write such a
detector in Rust, and the `Gate` runs the whole census, rules included, from an
ordinary `#[test]`. Together they make the census a dev-dependency of the
repository it protects.

## Contract

`raf_census` exposes the `Rule` trait:

```rust
pub trait Rule {
    /// The rule id; hits report as `rule:<id>`.
    fn id(&self) -> &'static str;
    /// Inspect one parsed file and push a hit for each finding.
    fn check(&self, ctx: &FileCtx, file: &syn::File, out: &mut Vec<RuleHit>);
    /// Source fixtures that prove what the rule flags and what it leaves alone.
    fn fixtures(&self) -> RuleFixtures;
}
```

- `RuleFixtures` holds source snippets in two sets, `positive` and `negative`. A
  rule returns at least one of each.
- `FileCtx` describes the file being checked: its package, repository-relative
  path, and whether it is shipping code or a test, bench or example target.
- `RuleHit` carries the line, a message and the symbol the hit belongs to.
- A rule id is unique across the rules registered on a gate and the built-in
  detector ids.
- A rule's hits are `rule:<id>` fingerprints, and a ledger job forbids them like any
  other fingerprint.

`Gate` is a builder:

- `Gate::workspace(manifest_dir)` loads the workspace that contains
  `manifest_dir` and its `helpers-ledger.toml`.
- `.rules(iter)` registers `Rule` implementations as boxed trait objects.
- `.extensions(iter)` registers `LedgerExtension` implementations (see
  [census-ledger](./census-ledger.md)).
- `.assert_clean()` runs the census and panics with every finding in one message
  when there is any, and returns when there is none.

A rule with no negative fixture is not enforced. Before it runs a registered rule,
the gate runs the rule's `fixtures()` through `check`. It refuses, with a hard
failure that names the rule, when any of these holds:

- the negative set is empty;
- a negative fixture produces a hit;
- a positive fixture produces no hit.

The built-in detectors and the pattern kinds follow the same rule: the census
crate's own tests run their positive and negative fixtures (see
[census-detectors](./census-detectors.md) and
[census-patterns](./census-patterns.md)).

## Usage

Add the crate as a dev-dependency and run the gate from an integration test:

```toml
[dev-dependencies]
raf-census = "1"
```

```rust
// tests/census.rs
use raf_census::Gate;

#[test]
fn helper_census() {
    Gate::workspace(env!("CARGO_MANIFEST_DIR"))
        .rules(my_rules())
        .extensions(my_extensions())
        .assert_clean();
}
```

The designed shape of a repository rule with its fixtures:

```rust
struct NoStdSleep;

impl Rule for NoStdSleep {
    fn id(&self) -> &'static str { "no-std-sleep" }
    fn check(&self, ctx: &FileCtx, file: &syn::File, out: &mut Vec<RuleHit>) {
        // Walk `file` and push a RuleHit for each call to std::thread::sleep.
    }
    fn fixtures(&self) -> RuleFixtures {
        RuleFixtures {
            positive: vec!["fn wait() { std::thread::sleep(D); }"],
            negative: vec!["fn wait(d: Duration) { d.is_zero(); }"],
        }
    }
}
```

## Examples

Registering the rule on a gate. The gate runs `NoStdSleep::fixtures()` first and
fails by name if they do not hold:

```rust
fn my_rules() -> Vec<Box<dyn Rule>> {
    vec![Box::new(NoStdSleep)]
}
```

## Constraints

- A `Rule` sees one parsed file at a time and holds no state between files.
- `Gate` panics on findings, as a test assertion does; it is not a runtime API for
  production binaries.
- Rule ids are `&'static str` and follow lowercase kebab case.
- The gate reads the workspace from disk through the source index and does not run
  the compiler, so it needs no build of the code under test.

## Explanation

The `Rule` trait is the escape hatch that keeps the shared crate generic. The old
census copies carried rules about RDF structure and hash domains because there was
no other place for them. With the trait, those rules live in the repository whose
types they know, and the shared crate carries none of them.

The negative-fixture requirement comes from the same lesson as the built-in
detectors. A rule is only trusted when its authors showed what it must not flag.
The gate makes that a precondition of running the rule, not a review comment.

## Verification

- A test registers a rule whose fixtures hold and shows the gate runs it, reports the
  hit on a positive source and passes the negative source.
- Three tests register a rule whose negative set is empty, a rule whose negative
  fixture produces a hit, and a rule whose positive fixture produces none, and show
  the gate fails each time, naming the rule.
- The census crate's own tests run the positive and negative fixtures of every
  built-in detector and pattern kind.
- A test registers two rules with one id and shows the gate fails.
- A test shows `.assert_clean()` returns on a clean fixture workspace and panics
  with every finding on a fixture workspace that contains a planted duplicate
  helper.
- A doctest shows the dev-dependency usage above compiles.
