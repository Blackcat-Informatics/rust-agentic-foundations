---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-debt-ratchet
title: "Census debt ratchet"
description: "An exact RDF record of duplication that can only shrink."
state: implementing
bindings:
  code:
    - raf_census::report::debt
    - raf_census::Gate::write_debt
    - raf_census::Gate::write_debt_locked
    - raf_census::gate::Gate::write_debt_with
    - raf_census::gate::Gate::evaluate_locked
    - raf_census::publication
    - raf_census::gate::read_debt
    - raf_census::DebtRecord
    - raf_census::DebtGroup
    - raf_census::DebtChange
    - raf_census::DebtWrite
    - raf_census::DEBT_FILE
    - raf_census::MAX_DEBT_BYTES
  tests:
    - raf_census::publication::tests::published_records_and_unowned_temporaries_survive_typed_finalization_failures
    - cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target
    - cargo_census::test_crate::delivery::debt_publication_preserves_unowned_entries_and_busy_records
    - cargo_census::test_crate::delivery::embedded_debt_publication_never_creates_a_declared_missing_rust_target
    - cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records

    - raf_census::test_crate::debt::new_groups_supersets_and_rising_copy_counts_are_growth
    - raf_census::test_crate::debt::a_present_superset_cannot_hide_a_missing_exact_recorded_group
    - raf_census::test_crate::debt::resolved_groups_strict_subsets_and_falling_counts_require_a_lower_record
    - raf_census::test_crate::debt::a_group_of_a_different_kind_is_new_debt
    - raf_census::test_crate::debt::both_native_spellings_round_trip_and_input_order_does_not_change_bytes
    - raf_census::test_crate::debt::commas_percent_signs_and_unicode_do_not_alias_member_sets
    - raf_census::test_crate::debt::other_versions_can_be_read_but_cannot_be_compared
    - raf_census::test_crate::debt::invalid_groups_and_zero_copy_rows_are_refused
    - raf_census::test_crate::debt::recomputing_a_digest_does_not_admit_foreign_classes_or_unknown_properties
    - raf_census::test_crate::debt::counted_cfg_members_round_trip_and_lower_without_changing_symbol_identity
    - raf_census::test_crate::debt::counted_nodes_refuse_redundant_counts_and_unowned_assertions
    - raf_census::test_crate::debt::oversized_records_refuse_before_parsing_beside_a_readable_record
    - cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate
    - cargo_census::test_crate::delivery::registered_code_is_required_explicitly_without_recording_partial_debt
    - cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets
  examples:
    - raf_census::example_crate::debt
depends: [cap:census-normal-form, cap:census-ledger, cap:rdf-report-profile]
---

# Census debt ratchet

## Purpose

The ratchet records present duplication while refusing both new debt and stale
slack. Removing duplication must lower the record, so an old allowance cannot
hide a regression.

## Contract

### Configured locked writing
<!-- raf:tested-by raf_census::example_crate::debt -->

Gate::write_debt_locked constructs its own DiskTree at the Gate's discovered
workspace root. It evaluates that root's complete ledger through the same
configured locked evaluation as Gate::check_locked, using the caller's
LockedCatalog, local LockedManifest associations and sealed GitPackageRoots.
Registered Rules, Extensions and preferred-provider proofs run before the debt
ratchet. Only first-party packages under the ledger roots supply consumer debt;
sealed provider source remains definition evidence.
The writer receives evaluation before check_debt adds debt-difference findings.
It accepts no caller-built GateReport and has no structural fallback.

### Exact counted relations
<!-- raf:tested-by raf_census::test_crate::debt::counted_cfg_members_round_trip_and_lower_without_changing_symbol_identity,raf_census::test_crate::debt::both_native_spellings_round_trip_and_input_order_does_not_change_bytes,raf_census::test_crate::debt::commas_percent_signs_and_unicode_do_not_alias_member_sets,raf_census::test_crate::debt::invalid_groups_and_zero_copy_rows_are_refused -->

The workspace-root census-debt.ttl is a canonical native RDF 1.2 DebtRecord
with normalForm and reportForm, body/shim groups and positive per-job copy counts.
Group identity is its kind and counted canonical symbols. Physical cfg
alternatives sharing a symbol retain multiplicity. Counts above one use owned
MemberMultiplicity nodes; ordinary one-count symbols keep memberSymbol.
Source lines and byte offsets are not debt identity. Equal duplication
relations coalesce, even if separate fingerprint groups project to that relation.
Both canonical RDF formats represent the same record; the stored file is Turtle.

### Growth and shrinking records
<!-- raf:tested-by raf_census::test_crate::debt::new_groups_supersets_and_rising_copy_counts_are_growth,raf_census::test_crate::debt::a_present_superset_cannot_hide_a_missing_exact_recorded_group,raf_census::test_crate::debt::resolved_groups_strict_subsets_and_falling_counts_require_a_lower_record,raf_census::test_crate::debt::a_group_of_a_different_kind_is_new_debt,cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate -->

New groups, counted supersets and rising copy counts are growth. Missing groups,
counted strict subsets and falling copy counts are shrinkage. Both fail a check.
A strict subset counts as shrinkage only when its recorded superset disappeared;
a surviving superset cannot hide a missing exact row. Growth cannot be written.
After shrinkage, debt --write or the matching Gate::write_debt or
Gate::write_debt_locked records exactly present debt.
The record never carries unused allowance.

### Exclusive writer coordination
<!-- raf:tested-by raf_census::example_crate::debt,cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target,cargo_census::test_crate::delivery::debt_publication_preserves_unowned_entries_and_busy_records,cargo_census::test_crate::delivery::embedded_debt_publication_never_creates_a_declared_missing_rust_target,cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records -->

On native Linux, both writers acquire an exclusive nonblocking kernel lock on
the retained workspace directory before evaluating policy or reading prior
debt. No coordination file is created. A manifest can name any filename as Rust
source; a reserved lock filename cannot authorize creating inspected source.
The directory lock remains held through comparison, serialization and publication,
including error cleanup. A competing writer refuses with an actionable busy
error; retry after the active writer finishes. Locking failures are hard errors.
Closing the owned directory description releases the lock. This prevents cooperating writers from comparing against
the same old allowance and publishing stale debt after a smaller record.
The filesystem must support exclusive directory locking and directory
synchronization; unavailable operations refuse without a weaker fallback.
Older writers using `.census-debt.ttl.lock` do not coordinate with this protocol.
Other platform adapters refuse publication without the native directory
capability. Advisory locking does not authenticate arbitrary concurrent directory
mutation or make source contents an immutable snapshot.

### Adoption and version replacement
<!-- raf:tested-by raf_census::test_crate::debt::other_versions_can_be_read_but_cannot_be_compared,cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate,raf_census::example_crate::debt -->

A missing record means no debt during checking. An explicit write creates it
from present debt. A mismatched normal form refuses comparison; the explicit
write may replace that old-version record after the ledger is current. A
malformed existing record still refuses. Creation and version replacement are
visible adoption operations; deleting the record and adopting again is not a
proof that debt did not grow.

### One publication path
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate,cargo_census::test_crate::delivery::registered_code_is_required_explicitly_without_recording_partial_debt,raf_census::example_crate::debt,raf_census::test_crate::debt::oversized_records_refuse_before_parsing_beside_a_readable_record -->

Gate::write_debt and cargo census debt --write use ordinary evaluation;
Gate::write_debt_locked uses configured locked evaluation. Both Gate writers
share one ratchet and publication path. Non-debt findings prevent every write. Registered rules or nonempty
extensions require a repository's registered Gate; the built-in CLI exits 2
naming Gate::write_debt rather than skipping them. Required callbacks cannot be omitted from the debt evaluation. A current-version record can only stay equal or shrink.


Registered policy is supplied directly to the library Gate through its rule and
extension registrations. The shipped CLI installs no repository callbacks and
refuses policy requiring them; it does not launch a repository target.
The writer serializes completely, checks the shared 16-MiB record cap, creates
an exclusive temporary file and synchronizes it before publication. Initial
adoption links the complete temporary to an absent destination atomically and
refuses a concurrent creation. An existing record is replaced atomically.
Existing debt is read through a pinned no-follow regular-file handle; a symlink
or nonregular record refuses before its contents are opened. The retained
directory is synchronized after publication. Equal records are left unchanged, including when an unrelated
temporary file is already present. A preexisting .census-debt.ttl.write blocks
publication and is left untouched. If writing an owned temporary or replacing
the record fails, the writer attempts to remove only its own temporary and
reports a cleanup failure as well as the original failure. A refusal preserves
the prior record; source, manifest, lock and ledger inputs are not rewritten.
An error after successful atomic publication explicitly reports that the record
already exists at its destination. Cleanup and directory-synchronization errors
retain their native types, including both causes when finalization fails twice.
<!-- raf:tested-by raf_census::publication::tests::published_records_and_unowned_temporaries_survive_typed_finalization_failures -->

## Usage

Run cargo census debt --write in a workspace with a native ledger and no required
repository registration. For registered rules/extensions, call Gate::write_debt
from a repository binary or xtask that installs those implementations. For
preferred providers, construct the same complete typed source associations used
by the locked check and call Gate::write_debt_locked(&catalog, &local, &sources).
The Gate reads first-party files and publishes census-debt.ttl at its own root;
the caller supplies the catalog, configurations and sealed dependency roots.
Check with Gate::check or Gate::assert_clean for ordinary evaluation, and
Gate::check_locked for configured locked evaluation.
The `raf_census::example_crate::debt` source reference includes ordinary and
locked disk fixtures with registered policy. Its fixture operations are explicit
host calls; RAF does not initiate the example's execution. Native writers lock
the retained workspace directory and create no coordination file.
<!-- raf:tested-by raf_census::example_crate::debt -->

For custom policy, construct the registered Gate in the repository's own binary
or xtask and call its explicit debt writer. The shipped CLI has no package or
example selector for loading callbacks.
<!-- raf:tested-by raf_census::example_crate::debt,cargo_census::test_crate::delivery::registered_code_is_required_explicitly_without_recording_partial_debt,cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets -->

## Examples

An explicit Cargo library path naming a missing `.census-debt.ttl.lock`,
temporary or debt record refuses as missing source and creates none of them.
A held directory lock refuses a competing writer; releasing it permits the
same ordinary debt publication. Preexisting temporary files and symlink or
directory entries remain untouched on refusal.
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target,cargo_census::test_crate::delivery::debt_publication_preserves_unowned_entries_and_busy_records -->

A recorded body group contains demo_a::len, demo_b::len and demo_c::len.
Removing demo_c leaves a strict subset. The next Gate check reports shrinkage.
Running cargo census debt --write records the two remaining members and the
next check passes. Adding a fourth member instead refuses both checking and
writing; an exact documented distinct sanction or removal resolves it.

The same debt example configures a local provider, a registered no-local-copy
rule and a nonempty owner extension. Ordinary writing refuses its provider
ledger. The locked writer adopts two copies, preserves an unchanged record,
refuses a third copy and records a reduction to one. A planted rule hit, an
unlisted provider competitor, a rejected owner or malformed prior debt refuses
without replacing that record. Version replacement and preexisting-temporary
controls exercise the same publication path.
The same executable holds a real kernel lock to verify contention refusal and
successful retry after release. A registered policy callback attempts a nested
writer and requires the busy refusal, proving evaluation occurs under the lock.
<!-- raf:tested-by raf_census::example_crate::debt -->

The native CLI fixture adopts two copies, preserves equal bytes, records a
reduction to one and refuses growth without replacing prior debt. It uses the
same ordinary Gate writer. A policy requiring a custom rule or owner validator
instead refuses with the library writer named in the diagnostic and creates no
partial record. The debt example covers the configured registered writer.
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate,cargo_census::test_crate::delivery::registered_code_is_required_explicitly_without_recording_partial_debt,raf_census::example_crate::debt -->


## Constraints

- The native CLI cannot supply repository-specific rule or extension callbacks.
  Such policy requires an explicitly configured library Gate. Repository target
  selectors are rejected before any consumer code is built or executed.
<!-- raf:tested-by cargo_census::test_crate::delivery::registered_code_is_required_explicitly_without_recording_partial_debt,cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets -->


- Coordination is advisory between writers locking the same retained directory.
  It does not prevent external source edits or authenticate concurrent directory
  mutation. Older file-lock writers do not participate. Native Linux directory
  locking and synchronization are required; other adapters refuse publication.
<!-- raf:tested-by raf_census::example_crate::debt -->

- The ratchet covers duplicate groups and unenforced job-copy counts. Detector
  findings and unavailable policy execution cannot be hidden in a record.
  Readers and writers share a 16-MiB serialized cap. Unknown shapes, invalid
  counts, unowned nodes and noncanonical/digest-invalid text refuse. Persistent
  repository debt is written by a binary or xtask; fixture tests write only their
  owned scratch workspaces.
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate,cargo_census::test_crate::delivery::registered_code_is_required_explicitly_without_recording_partial_debt,raf_census::test_crate::debt::counted_nodes_refuse_redundant_counts_and_unowned_assertions,raf_census::test_crate::debt::oversized_records_refuse_before_parsing_beside_a_readable_record -->

- The locked writer owns the first-party disk read and publication root, not
  acquisition of its catalog or dependency source. The caller supplies complete
  associations and declared configurations; the operation does not authenticate
  a compiler invocation or acquire its complete source closure. It preserves
  Source's locked identity, authority and configuration refusals.
<!-- raf:tested-by raf_census::example_crate::debt -->
- Debt publication is an explicit library writer operation. Installing custom
  callbacks on a Gate does not grant the shipped CLI authority to acquire or
  execute those callbacks from repository source.
<!-- raf:tested-by raf_census::example_crate::debt,cargo_census::test_crate::delivery::registered_code_is_required_explicitly_without_recording_partial_debt -->

## Explanation

The [shared debt decision](../architecture/debt-ratchet.md) requires both sides
of the ratchet. Blocking only growth leaves slack after code improves: an old
record of three copies would permit two copies to return when only one remains.
Requiring a lowering operation makes each improvement visible and keeps that
allowance equal to current code.

Counted symbol relations separate debt identity from source-line movement while
retaining distinct cfg alternatives. The exact-set check avoids treating one
remaining duplicate group as an excuse for a different vanished group. Explicit
adoption and version replacement stay reviewable because they create or replace
the tracked record. Complete serialization before publication prevents an
oversized or failed rendering from replacing a readable record. Registered code
uses the same evaluator and writer, so a command that cannot install it refuses
instead of quietly producing a smaller partial record.

## Verification

Controlled CLI fixtures refuse missing declared source at each closed publication
pathname without creating it. They preserve unowned files, directories and
symlinks, refuse a held directory lock and admit publication after release.
The finalization fixture exercises both atomic creation and replacement with an
injected synchronization failure; complete records survive, and a replaced
unowned temporary remains intact with both failure causes retained as typed
errors. Fixture programs and example consumers are never initiated by RAF.
<!-- raf:tested-by cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target,cargo_census::test_crate::delivery::debt_publication_preserves_unowned_entries_and_busy_records,raf_census::publication::tests::published_records_and_unowned_temporaries_survive_typed_finalization_failures -->

Bound cases cover both ratchet directions, kind changes, counted cfg members,
canonical formats, shape/digest refusal and version comparison. Actual command
fixtures prove adoption, growth refusal, lowering, registration refusal and
preservation of the previous file. The existing executable example includes
ordinary and configured locked disk writing with registered Rules, a nonempty
Extension and an admitted local provider. Its assertions cover adoption,
unchanged bytes, strict shrink, growth and policy refusal, malformed prior data,
version replacement and owned versus preexisting temporary files. Each locked
writer also exercises real lock contention, release and evaluation under the
shared guard. Each locked
write snapshots source, manifests, lock and ledger, and refusal controls compare
prior debt bytes. The fixture's declared target and local provider establish
the controlled policy path; they do not claim an observed compiler configuration
or execution of an external native implementation.
<!-- raf:tested-by raf_census::example_crate::debt -->

<!-- raf:generated:verification-matrix -->
| Item | Kind | Section |
| --- | --- | --- |
| `cargo_census::test_crate::delivery::debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate` | test | [Contract](#contract), [Examples](#examples), [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::debt_publication_never_creates_a_declared_missing_rust_target` | test | [Contract](#contract), [Examples](#examples) |
| `cargo_census::test_crate::delivery::debt_publication_preserves_unowned_entries_and_busy_records` | test | [Contract](#contract), [Examples](#examples) |
| `cargo_census::test_crate::delivery::embedded_debt_publication_never_creates_a_declared_missing_rust_target` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::embedded_debt_publication_preserves_unowned_entries_and_busy_records` | test | [Contract](#contract) |
| `cargo_census::test_crate::delivery::native_index_never_builds_or_executes_consumer_targets` | test | [Usage](#usage), [Constraints](#constraints) |
| `cargo_census::test_crate::delivery::registered_code_is_required_explicitly_without_recording_partial_debt` | test | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::example_crate::debt` | example target | [Contract](#contract), [Usage](#usage), [Examples](#examples), [Constraints](#constraints) |
| `raf_census::publication::tests::published_records_and_unowned_temporaries_survive_typed_finalization_failures` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::a_group_of_a_different_kind_is_new_debt` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::a_present_superset_cannot_hide_a_missing_exact_recorded_group` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::both_native_spellings_round_trip_and_input_order_does_not_change_bytes` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::commas_percent_signs_and_unicode_do_not_alias_member_sets` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::counted_cfg_members_round_trip_and_lower_without_changing_symbol_identity` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::counted_nodes_refuse_redundant_counts_and_unowned_assertions` | test | [Constraints](#constraints) |
| `raf_census::test_crate::debt::invalid_groups_and_zero_copy_rows_are_refused` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::new_groups_supersets_and_rising_copy_counts_are_growth` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::other_versions_can_be_read_but_cannot_be_compared` | test | [Contract](#contract) |
| `raf_census::test_crate::debt::oversized_records_refuse_before_parsing_beside_a_readable_record` | test | [Contract](#contract), [Constraints](#constraints) |
| `raf_census::test_crate::debt::recomputing_a_digest_does_not_admit_foreign_classes_or_unknown_properties` | test | [Verification](#verification) |
| `raf_census::test_crate::debt::resolved_groups_strict_subsets_and_falling_counts_require_a_lower_record` | test | [Contract](#contract) |
<!-- raf:generated:end -->
<!-- raf:generated:relationships -->
| Relation | Capability |
| --- | --- |
| Depends on | [census-ledger](./census-ledger.md) |
| Depends on | [census-normal-form](./census-normal-form.md) |
| Depends on | [rdf-report-profile](./rdf-report-profile.md) |
| Depended on by | [census-cli](./census-cli.md) |
| Depended on by | [census-embedded-clones](./census-embedded-clones.md) |
| Depended on by | [census-rule-api](./census-rule-api.md) |
| Depended on by | [repository-contract](./repository-contract.md) |
| Changed by | [embedded-clones](../../work/changes/embedded-clones.md) |
<!-- raf:generated:end -->
