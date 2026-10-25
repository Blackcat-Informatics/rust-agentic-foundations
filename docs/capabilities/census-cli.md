---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-cli
title: "cargo census command line"
description: "Repository-owned checks, native migration and explicit debt recording."
state: implementing
bindings:
  code:
    - cargo_census::bin_crate::cargo_census::main
    - cargo_census::bin_crate::cargo_census::cli
    - cargo_census::bin_crate::cargo_census::command
    - cargo_census::bin_crate::cargo_census::registered
    - cargo_census::bin_crate::cargo_census::error
    - raf_census::registered
    - raf_census::RegisteredCheck
    - raf_census::RegisteredDebtWrite
    - raf_census::RegisteredOperation
    - raf_census::DEBT_WRITE_PROTOCOL_VERSION
    - raf_census::CHECK_PROTOCOL_VERSION
    - raf_census::MAX_CHECK_RESPONSE_BYTES
    - raf_workplan::exec::Invocation::example_debt_write
  tests:
    - raf_workplan::test_crate::exec::every_invocation_is_the_row_of_its_table
    - raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors
    - raf_census::registered::tests::protocol_version_fields_and_outcomes_are_closed_with_valid_neighbors
    - raf_census::registered::tests::membership_ordinals_and_orphans_cannot_drop_findings
    - raf_census::registered::tests::canonical_and_size_refusals_do_not_accept_partial_carriers
    - cargo_census::bin_crate::cargo_census::registered::tests::malformed_target_responses_preserve_the_actual_stderr_cause
    - cargo_census::bin_crate::cargo_census::registered::tests::process_failure_protocol_disagreement_and_gate_error_remain_distinct
    - cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity
    - cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate
    - cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy
    - cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record
  examples:
    - raf_census::example_crate::registered
depends: [cap:census-ledger, cap:census-debt-ratchet, cap:census-reference-scan, cap:census-migration, cap:census-rule-api]
---

# cargo census command line

## Purpose

The command runs a repository-owned Census check with its actual Rule and
LedgerExtension registrations, and exposes native migration and explicit debt
recording. The selected target owns its policy and locked source inputs; the host
owns process execution and complete response validation.

## Contract

### Registered read-only check
<!-- raf:tested-by cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->

The command cargo census --workspace DIR check --package PACKAGE --example EXAMPLE
discovers the requested workspace root and runs cargo run --frozen
--package=PACKAGE --example=EXAMPLE there. Package and example are explicit
validated Cargo names. The repository example registers its own Gate and evaluates
the full requested policy. The CLI never substitutes a built-in-only Gate.
The existing bounded SystemRunner supplies null stdin and C locale, build
execution limits and a 16 MiB cap on each captured output stream. Cargo may create
build artifacts. The protocol grants no source or debt write and the host performs
none. The target is trusted repository executable code, not a sandbox.

### Closed native response
<!-- raf:tested-by raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors,raf_census::registered::tests::protocol_version_fields_and_outcomes_are_closed_with_valid_neighbors,raf_census::registered::tests::membership_ordinals_and_orphans_cannot_drop_findings,raf_census::registered::tests::canonical_and_size_refusals_do_not_accept_partial_carriers,cargo_census::bin_crate::cargo_census::registered::tests::malformed_target_responses_preserve_the_actual_stderr_cause,cargo_census::bin_crate::cargo_census::registered::tests::process_failure_protocol_disagreement_and_gate_error_remain_distinct -->

RegisteredCheck projects an actual Result<GateReport, Error> into native Graph
storage and canonical N-Triples1.2. Its fixed protocol IRI is
urn:raf:census:registered-check:v1. The sole RegisteredCheck root states
protocolVersion, status and findingCount. Clean has zero findings; findings has a
positive count; error has zero findings and exactly one errorMessage. Each finding
is an owned CensusFinding node at #finding/ORDINAL with a contiguous zero-based
integer ordinal and message. Order and duplicate messages are preserved.
Unknown fields/classes/versions, incomplete or orphan nodes, inconsistent
statuses, noncanonical bytes and responses larger than 16 MiB refuse. The size
bound governs wire bytes, not total native graph allocation.

### Failure and exit agreement
<!-- raf:tested-by cargo_census::bin_crate::cargo_census::registered::tests::malformed_target_responses_preserve_the_actual_stderr_cause,cargo_census::bin_crate::cargo_census::registered::tests::process_failure_protocol_disagreement_and_gate_error_remain_distinct,cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity -->

The target emits exactly one response on stdout and exits0 for clean or1 for
findings/evaluation error. The CLI prints census: clean or every finding in original
order and returns0 or1. A framed Gate error, missing target, build/process failure,
missing/malformed/oversized response, invalid UTF-8, protocol mismatch or exit
disagreement returns1 with an actionable diagnostic. Bounded stderr survives
process and response-decoding refusals. No partial successful verdict exists.
Invalid command usage returns2 through clap.

### Native migration and explicit debt recording
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate,cargo_census::test_crate::delivery::migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy,cargo_census::test_crate::delivery::command_usage_and_bounded_input_failures_do_not_publish_a_record -->

The command debt --write uses Gate::write_debt for ledgers needing no registered
code; registration-required policy returns2 and writes nothing. A repository's
own registered Gate remains its explicit native writing path.
<!-- raf:tested-by cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity,raf_census::registered::tests::protocol_version_fields_and_outcomes_are_closed_with_valid_neighbors,raf_census::registered::tests::native_responses_preserve_order_multiplicity_and_evaluation_errors -->

With both --package PACKAGE and --example EXAMPLE, debt --write instead invokes
the selected registered target using exactly cargo run --frozen --package=PACKAGE
--example=EXAMPLE -- --census-debt-write-v1. Partial selectors or missing --write
are usage errors. RegisteredOperation::from_args selects Check for no target
arguments and WriteDebt for exactly that one versioned token; every other
sequence refuses before evaluation. The target calls its actual registered Gate
writer, including write_debt_locked when configured providers are required.

Invocation::example_debt_write constructs that exact fixed argument sequence
from validated package/example names, exposing no arbitrary target arguments.
Workplan bound verification keeps its existing example_run invocation.
<!-- raf:tested-by raf_workplan::test_crate::exec::every_invocation_is_the_row_of_its_table -->

RegisteredDebtWrite projects Result<DebtWrite, Error> into one native graph root
of class RegisteredDebtWrite, protocol urn:raf:census:registered-debt-write:v1.
Status written carries exactly groupCount and jobCount as canonical nonnegative
integers; status error carries exactly errorMessage as a string and no counts.
The response has no destination or additional subjects. Unknown fields, missing
or repeated scalars, wrong classes/versions, status contradictions, noncanonical
bytes and oversized carriers refuse. It uses the existing per-stream size bound.
Exit zero agrees only with written, exit one only with error. A check response
cannot acknowledge publication. No process, protocol or backend failure retries
or falls back to ordinary evaluation; success prints only after full validation.
The command migrate --ledger FILE --index FILE reads older native RDF; migrate
--legacy --ledger FILE --baseline FILE imports the named foreign dialect.
Migration writes stdout and preserves both inputs. Unmapped rows or execution
failures return1; complete mapping returns0; invalid usage returns2. The
[census-debt-ratchet](./census-debt-ratchet.md) and
[census-migration](./census-migration.md) capabilities specify those operations.

> [!PROPOSED change:census-cli]
> The complete query surface includes baseline with --out and RDF format,
> index with RDF format, job ID, count ID, fingerprint SYM and dump-ledger.
> Their output is the corresponding native library report with deterministic
> ordering.

## Usage

Create a repository example that builds the same Gate used by its census test,
including rules, extensions and any explicitly supplied locked provider inputs.
Evaluate it and call RegisteredCheck::from_check(&result). Write to_ntriples()
bytes to stdout and return ExitCode::from(response.exit_code()). Build diagnostics
belong on stderr. Prepare the actual Cargo.lock before using the frozen command.
The library example registered demonstrates response generation on an owned
MemoryTree; the real-target integration test demonstrates Cargo execution.

~~~text
cargo census --workspace . check --package demo --example registered
cargo census debt --write
cargo census --workspace . debt --write --package demo --example registered
cargo census migrate --ledger old-ledger.ttl --index old-index.ttl
~~~

A target supporting both operations decodes args_os().skip(1) through
RegisteredOperation::from_args before executing either path. For WriteDebt it
passes the actual writer result to RegisteredDebtWrite::from_write, emits the
canonical carrier and returns its exit_code, just as the check path does with
RegisteredCheck. The real Cargo fixture demonstrates both operations with one
registered Rule, owner extension and configured local provider.

> [!PROPOSED change:census-cli]
> Query commands print baseline/index reports, job roles, copy counts and
> body/shim fingerprints without changing the policy they inspect.

## Examples

A repository target registers NoLocalCopy and the owner extension once. The same
registrations run in its test. With only the documented home, check prints
census: clean and exits0. Adding local_copy produces the same ordered findings
as the direct Gate and exits1. Changing the owner payload from demo to other
returns the extension's evaluation refusal. Selecting an absent example or one
that does not compile reports the actual Cargo cause and exits1. All runs use
the requested root even when the caller's current directory is elsewhere.

Explicit registered writing adopts the actual configured debt, preserves equal
bytes, records strict shrink and refuses growth, a custom-rule hit, an invalid
owner or an unlisted native competitor. Direct target and CLI publication produce
identical canonical debt bytes from equivalent roots. Check with empty target
arguments remains read-only after adoption.

## Constraints
<!-- raf:tested-by cargo_census::test_crate::registered::actual_registered_target_preserves_rule_extension_root_and_read_only_parity,raf_census::registered::tests::canonical_and_size_refusals_do_not_accept_partial_carriers,cargo_census::bin_crate::cargo_census::registered::tests::process_failure_protocol_disagreement_and_gate_error_remain_distinct -->

The command operates on one workspace and executes trusted code from it. The
frozen Cargo invocation requires a current lock and available dependencies;
missing prerequisites fail. The response is the sole stdout document and uses
the shared native RDF1.2 ground profile. No schema negotiation, partial-result
fallback or implicit debt adoption exists. Provider identities/configurations
are supplied by the target; this transport does not authenticate a compiler
invocation or acquire its complete source closure. Unsupported proof syntax
retains the native Gate refusal.

An authorized trusted target may publish successfully before its receipt is
lost, malformed, exceeds capture limits or cannot be rendered by the host.
A transport failure therefore does not imply rollback. The host does not retry;
inspect the debt record before explicitly invoking another write. Receipt counts
summarize trusted execution rather than authenticating target honesty.

## Explanation

Repository policy is executable Rust. Running its example keeps custom rules and
extensions on the same path as its tests. A closed response separates a policy
finding from a failed evaluation or build. The CLI validates the whole response
before publishing findings. Native storage, parser, canonical writer and the
existing Runner avoid parallel graph or process implementations; the adapter
adds only Census schema and exit agreement.

Publication is a separate operation because a clean check is not authority to
write. Its fixed token and distinct response preserve that boundary while sharing
the native Gate ratchet, runner, RDF storage and canonical codec.

## Verification

The real Cargo fixture shares exact registration source with its direct Gate
oracle. It compares clean/planted findings, extension rejection and absent/broken
target diagnostics from an unrelated working directory, while checking source,
manifest, lock and ledger bytes and absence of debt writes. Codec controls cover
order/duplicates, unknown/missing/mistyped fields, node ownership, ordinals,
protocol/status disagreement, canonical bytes and wire size. Scripted Runner
controls retain stderr/typed causes across malformed UTF-8 and native RDF,
unexpected exits and response/exit disagreement.

The same real Cargo case executes configured registered adoption, equal bytes,
strict shrink, growth/nondebt/owner/provider and malformed-prior refusals. It
compares direct-target and CLI bytes, checks fixed-token/CLI authorization and
known input preservation, and proves a malformed receipt after publication
reports failure without falsely claiming rollback. Existing codec and host
cases also validate the distinct write response and cross-operation refusal.

> [!PROPOSED change:census-cli]
> Every additional query command has an actual-binary fixture for output and
> refusal neighbours; named output writing is checked against native reports.
