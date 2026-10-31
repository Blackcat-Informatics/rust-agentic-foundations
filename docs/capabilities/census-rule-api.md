---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-rule-api
title: "Census Rule trait and Gate"
description: "The Rust Rule trait for repository-specific detectors and the Gate that runs the census as a dev-dependency test."
state: delivered
bindings:
  code:
    - raf_census::rule
    - raf_census::gate::rules
    - raf_census::Gate::require_rules
    - raf_census::Gate::workspace
    - raf_census::Gate::rules
    - raf_census::Gate::extensions
    - raf_census::Gate::assert_clean
  tests:
    - raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries
    - cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity
    - raf_census::test_crate::rules::the_rule_runs_its_positive_and_negative_source_cases
    - raf_census::test_crate::rules::empty_source_sets_are_refused_with_a_passing_neighbour
    - raf_census::test_crate::rules::invalid_rust_is_a_parse_failure_and_does_not_count_as_a_negative_case
    - raf_census::test_crate::rules::positive_misses_and_negative_hits_are_refused
    - raf_census::test_crate::census_gate::an_unknown_rule_is_hard_and_registered_hits_cannot_be_adopted_as_debt
    - raf_census::test_crate::census_gate::configured_provider_tracks_native_state_updates_and_refuses_parallel_mutation
    - raf_census::test_crate::census_gate::configured_provider_targets::every_selected_target_proves_native_calls_through_its_ordinary_library
  examples:
    - raf_census::example_crate::registered
depends: [cap:census-detectors, cap:census-ledger, cap:census-patterns, cap:census-debt-ratchet, cap:census-reference-scan]
---

# Census Rule trait and Gate

## Purpose

Some checks need structure that a declarative pattern cannot express, or concern
one repository's own types. The `Rule` trait lets a repository write such a
detector in Rust, and the `Gate` runs the whole census, rules included, from an
ordinary `#[test]`. Together they make the census a dev-dependency of the
repository it protects.

Required-rule selection also protects repository-wide source policies without
inventing a helper home or a ledger job for them.

Configured provider checks apply the same gate to each selected consumer target,
so a dependency available to one target cannot authorize another target's calls.

## Contract

### Required rules

`.require_rules(iter)` selects registered or executable built-in rule IDs whose
findings are required independently of ledger jobs. Every eligible hit is a
hard finding, including source subjects without body fingerprints. A home,
variant, unenforced job or debt record cannot exempt it. Detector-specific
source eligibility remains unchanged. Unknown IDs, duplicate required IDs and
home-dependent detectors refuse actionably.
<!-- raf:tested-by raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries -->

### Rule interface
<!-- raf:tested-by raf_census::test_crate::rules::the_rule_runs_its_positive_and_negative_source_cases,raf_census::test_crate::census_gate::an_unknown_rule_is_hard_and_registered_hits_cannot_be_adopted_as_debt -->

`raf_census` exposes the `Rule` trait:

```rust
pub trait Rule: Sync {
    /// The rule id; hits report as `rule:<id>`.
    fn id(&self) -> &'static str;
    /// Inspect one parsed file and push a hit for each finding.
    fn check(&self, ctx: &FileCtx, file: &syn::File, out: &mut Vec<RuleHit>);
    /// Source fixtures that prove what the rule flags and what it leaves alone.
    fn fixtures(&self) -> RuleFixtures;
}
```

- A rule runs on the parse thread, so `Rule: Sync`.
- `RuleFixtures` holds source snippets in two sets, `positive` and `negative`. A
  rule returns at least one of each.
- `FileCtx` describes the file being checked: its package, repository-relative
  path, and whether it is shipping code or a test, bench or example target.
- `RuleHit` carries the line, a message and the symbol the hit belongs to.
- A rule id is unique across the rules registered on a gate and the built-in
  detector ids.
- A rule's hits are `rule:<id>` fingerprints, and a ledger job forbids them like any
  other fingerprint.

### Gate operations
<!-- raf:tested-by cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity,raf_census::test_crate::census_gate::an_unknown_rule_is_hard_and_registered_hits_cannot_be_adopted_as_debt -->

`Gate` is a builder:

- `Gate::workspace(manifest_dir)` selects the workspace that contains
  `manifest_dir`. Checks read its canonical RDF ledger, `helpers-ledger.ttl`.
- `.rules(iter)` registers `Rule` implementations as boxed trait objects.
- `.extensions(iter)` registers `LedgerExtension` implementations (see
  [census-ledger](./census-ledger.md)).
  Each receives a borrowed `LedgerExtensionGraph` containing only its job's
  uniquely owned native subjects. The validator checks root and nested fields;
  `None` declines, `Some(Err(...))` refuses and exactly one `Some(Ok(()))` is
  required for nonempty content.
  <!-- raf:tested-by cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->
- `.assert_clean()` runs the census and panics with every finding in one message
  when there is any, and returns when there is none.
- `.write_debt()` has the signature `fn write_debt(&self) -> Result<DebtWrite, Error>`.
  It rewrites `census-debt.ttl` beside the ledger to the present debt (see
  [census-debt-ratchet](./census-debt-ratchet.md)) and refuses, with an `Error` that
  lists them, while any finding that is not debt exists, the same refusal as
  `cargo census debt --write`. `DebtWrite` reports the path written, the number of
  groups and the number of jobs with copies, and prints as the summary line that
  `cargo census debt --write` prints. Configured provider policy uses
  `write_debt_locked` with complete source associations. The registered CLI
  selects the repository's writer explicitly with --package and --example;
  its fixed versioned request and distinct native RDF receipt preserve this
  same policy and ratchet, without a fallback or arbitrary report publisher.
<!-- raf:tested-by cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->
- Validating the ledger, the gate resolves every `rule:<id>` that a job forbids
  against the built-in detectors, the patterns and the registered rules. An id that
  matches none is a hard ledger error that names the id and the job.

### Fixture admission
<!-- raf:tested-by raf_census::test_crate::rules::empty_source_sets_are_refused_with_a_passing_neighbour,raf_census::test_crate::rules::invalid_rust_is_a_parse_failure_and_does_not_count_as_a_negative_case,raf_census::test_crate::rules::positive_misses_and_negative_hits_are_refused -->

A rule with no negative fixture is not enforced. Before it runs a registered rule,
the gate runs the rule's `fixtures()` through `check`. It refuses, with a hard
failure that names the rule, when any of these holds:

- the positive set is empty;
- the negative set is empty;
- a negative fixture produces a hit;
- a positive fixture produces no hit.

The built-in detectors and the pattern kinds follow the same rule: the census
crate's own tests run their positive and negative fixtures (see
[census-detectors](./census-detectors.md) and
[census-patterns](./census-patterns.md)).

### Configured providers

Configured provider checks use `check_locked` with exact manifest associations,
sealed provider sources and declared target contexts. Build scripts require an
eligible build dependency; that route cannot supply an ordinary library's
missing provider. The provider proof follows returned native state and values,
including the pinned PurRDF Hasher alias, rather than accepting a provider name
or an unused call. The full policy is stated in
[census-ledger](./census-ledger.md#selected-target-contexts).
<!-- raf:tested-by raf_census::test_crate::census_gate::configured_provider_targets::every_selected_target_proves_native_calls_through_its_ordinary_library,raf_census::test_crate::census_gate::configured_provider_tracks_native_state_updates_and_refuses_parallel_mutation -->

## Usage

Run `cargo run -p raf-census --example registered` for the bound
`raf_census::example_crate::registered` example. It registers a rule and an
extension, accepts its valid source and refuses a planted copy and invalid owner.
<!-- raf:tested-by raf_census::example_crate::registered -->

Register custom implementations with `.rules(iter)` and select their IDs with
`.require_rules(iter)` when they must hold without a ledger job. Fixture
admission still applies. Check and debt-writing operations share this same
required-rule path.
<!-- raf:tested-by raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries -->

Use `check_locked` for configured provider checks, supplying exact catalog rows,
sealed Git source and complete target declarations. A build script selects its
`build-script-build` context with an independent host cfg and requests a build
dependency edge. The selected package library keeps its own ordinary context.
<!-- raf:tested-by raf_census::test_crate::census_gate::configured_provider_targets::every_selected_target_proves_native_calls_through_its_ordinary_library -->

Use `LedgerExtensionGraph::root` to inspect the owner declaration and `get` to
resolve its normalized local child ids. Reject missing fields, unknown fields
and invalid evidence in the validator; closure membership alone verifies no
policy meaning. Iteration through `nodes` is deterministic and borrowed.

Add the crate as a dev-dependency and run the gate from an integration test:

```toml
[dev-dependencies]
raf-census = "0.1"
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

`write_debt` is called from the repository's own binary or xtask, never from a
`#[test]`, because a test does not write tracked files. An xtask that the repository
runs as `cargo xtask census-debt`:

```rust
// xtask/src/main.rs
use raf_census::Gate;

fn census_debt() -> Result<(), raf_census::Error> {
    let written = Gate::workspace(env!("CARGO_MANIFEST_DIR"))
        .rules(my_rules())
        .extensions(my_extensions())
        .write_debt()?;
    println!("{written}");
    Ok(())
}
```

For a repository example supporting both operations, decode its arguments with
RegisteredOperation::from_args before evaluating the registered Gate. Empty
arguments select checking; the exact versioned token selects writing. Pass the
actual configured writer result to RegisteredDebtWrite::from_write, emit its
canonical response and matching exit. Invoke it explicitly with cargo census
debt --write --package demo --example registered. The command contract is in
[census-cli](./census-cli.md); the library's own registered example demonstrates
check response generation, while the real CLI fixture supplies the writing target.

A repository rule supplies its source fixtures:

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

`Gate::workspace(root).require_rules(["serde-json"]).assert_clean()` requires
the selected detector even when the admitted ledger declares no jobs. A
misspelled ID refuses rather than disabling the check. Job-dependent
`home-literal`, `vocabulary-literal` and `sole-name` require ledger homes and
cannot be selected as unscoped requirements.
<!-- raf:tested-by raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries -->

The runnable registered example shares its NoLocalCopy rule, owner extension and
native ledger fixture with the actual registered CLI test. It accepts the home,
reports a planted local_copy, reads the owner name from a nested native node,
refuses the wrong nested owner payload and emits the clean
canonical check response. Run it with `cargo run -p raf-census --example registered`.

The real CLI fixture additionally registers a configured local provider and an
unenforced copy job. Its repository target decodes the explicit write operation,
calls the same registered Gate's write_debt_locked and emits a native write
receipt. The direct target and shipped CLI create identical debt records; growth,
rule hits, a rejected owner and an unlisted provider competitor refuse without
replacing prior debt. The check operation remains read-only.
<!-- raf:tested-by cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->

The native hashing example creates `purrdf_hash::blake3::Hasher`, passes the
input to `Hasher::update(&mut state, input)` and returns
`Hasher::finalize(&state)`. Its typed caller consumes the returned `Hash` bytes
and compares a nonempty message with independently frozen BLAKE3 output. The
Gate judges that same source using the compiled dependency's exact committed
definitions. Source discovery uses `purrdf_testkit::paths::workspace_root()`;
missing Git objects are an actionable failure.
<!-- raf:tested-by raf_census::test_crate::census_gate::configured_provider_tracks_native_state_updates_and_refuses_parallel_mutation -->

Registering the rule on a gate. The gate runs `NoStdSleep::fixtures()` first and
fails by name if they do not hold:

```rust
fn my_rules() -> Vec<Box<dyn Rule>> {
    vec![Box::new(NoStdSleep)]
}
```

## Constraints

Required-rule selection does not grant provider authority or alter a
detector's declared eligibility. Its findings are not structural debt and
cannot be recorded by debt publication.
<!-- raf:tested-by raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries -->

- Each `Rule::check` call receives one parsed file; the Gate supplies no
  whole-workspace callback.
<!-- raf:tested-by raf_census::test_crate::rules::the_rule_runs_its_positive_and_negative_source_cases -->
- `assert_clean` panics on findings, as a test assertion does; it is not a runtime
  API for production binaries. `write_debt` returns a `Result` and is for the
  repository's own binary or xtask only.
<!-- raf:tested-by cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->
- The registered `cargo census check` invokes a repository example whose `Gate`
  owns every rule and extension. The host adds no registrations and never
  substitutes a partial census (see [census-cli](./census-cli.md)).
<!-- raf:tested-by cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->
- Rule ids are `&'static str` and follow lowercase kebab case.
<!-- raf:tested-by raf_census::test_crate::census_gate::an_unknown_rule_is_hard_and_registered_hits_cannot_be_adopted_as_debt -->
- The gate reads the workspace from disk through the source index and does not run
  the compiler, so it needs no build of the code under test.
<!-- raf:tested-by raf_census::test_crate::census_gate::an_unknown_rule_is_hard_and_registered_hits_cannot_be_adopted_as_debt -->

- A build dependency cannot supply an ordinary library's provider route. A shared
  body must prove every selected consumer context independently.
<!-- raf:tested-by raf_census::test_crate::census_gate::configured_provider_targets::every_selected_target_proves_native_calls_through_its_ordinary_library -->

## Explanation

The `Rule` trait is the escape hatch that keeps the shared crate generic. The old
census copies carried rules about RDF structure and hash domains because there was
no other place for them. With the trait, those rules live in the repository whose
types they know, and the shared crate carries none of them.

The negative-fixture requirement comes from the same lesson as the built-in
detectors. A rule is only trusted when its authors showed what it must not flag.
The gate makes that a precondition of running the rule, not a review comment.

## Verification

The existing detector Gate case checks required findings with no jobs, valid
neighboring source, unsupported IDs and home-dependent selection refusals.
The self-Census exercises the same builder path over the actual snapshot.

- A test registers a rule whose fixtures hold and shows the gate runs it, reports the
  hit on a positive source and passes the negative source.
- Four tests register a rule whose positive set is empty, a rule whose negative set
  is empty, a rule whose negative fixture produces a hit, and a rule whose positive
  fixture produces none, and show the gate fails each time, naming the rule.
- The census crate's own tests run the positive and negative fixtures of every
  built-in detector and pattern kind.
- A test registers two rules with one id and shows the gate fails.
- The registered example and Gate integration cases pair accepted source with
  planted rule findings. `assert_clean` returns for a clean Gate report and
  converts findings or an evaluation error into a panic.
- A test shows `.write_debt()` writes `census-debt.ttl` beside the ledger of a
  fixture workspace that registers a rule and an extension, and that two runs give
  byte-identical files.
- A test shows `.write_debt()` returns an `Error` that lists the findings, and writes
  nothing, while a non-debt finding exists.
- A test shows a job that forbids `rule:nope` fails the gate with a ledger error that
  names the id, and passes once a rule with that id is registered.
- The root integration test consumes `raf-census` as a dev-dependency and calls
  the production Gate directly. Source inspection by that Gate does not start
  another compiler.

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::example_crate::registered` | example target | [Usage](#usage) |
| `raf_census::test_crate::census_gate::an_unknown_rule_is_hard_and_registered_hits_cannot_be_adopted_as_debt` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::byte_layout_detectors_execute_with_shipping_and_home_package_boundaries` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::configured_provider_targets::every_selected_target_proves_native_calls_through_its_ordinary_library` | test | [Contract](#contract), [Usage](#usage), [Constraints](#constraints) |
| `raf_census::test_crate::census_gate::configured_provider_tracks_native_state_updates_and_refuses_parallel_mutation` | test | [Contract](#contract), [Examples](#examples) |
| `raf_census::test_crate::rules::empty_source_sets_are_refused_with_a_passing_neighbour` | test | [Contract](#contract) |
| `raf_census::test_crate::rules::invalid_rust_is_a_parse_failure_and_does_not_count_as_a_negative_case` | test | [Contract](#contract) |
| `raf_census::test_crate::rules::positive_misses_and_negative_hits_are_refused` | test | [Contract](#contract) |
| `raf_census::test_crate::rules::the_rule_runs_its_positive_and_negative_source_cases` | test | [Contract](#contract), [Constraints](#constraints) |
<!-- raf:generated:end -->
