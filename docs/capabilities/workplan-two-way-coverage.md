---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-two-way-coverage
title: "Two-way coverage"
description: "Checks from code to documentation and from tests to claims, complementing the documentation-to-code and documentation-to-tests checks."
state: considered
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:workplan-document-model, cap:source-index, cap:workplan-evidence-claims]
---

# Two-way coverage

## Purpose

The document model checks that what is documented exists: every binding resolves to
code and to a test. The opposite directions ask whether what exists is documented.
This capability adds code to documentation (public items that no capability explains)
and tests to claims (tests that support no documented guarantee), so that built
functionality and its documentation stay aligned in both directions.

Open questions:

- Which items count as public surface when a crate re-exports from a private module.
- Whether a test that supports no claim is a warning or an error, since some tests
  guard internals.
- Whether an item may be excluded with a recorded reason, and where that reason lives.

## Contract

> [!PROPOSED change:workplan-two-way-coverage]
> **The direction table.**
>
> | Direction | Question | Check |
> | --- | --- | --- |
> | Documentation -> code | Where is this documented capability implemented? | Every code binding resolves (`workplan check`) |
> | Code -> documentation | Where is this supported behaviour explained? | Every public item is bound by a capability, or excluded with a reason |
> | Documentation -> tests | Which tests address this guarantee? | Every test binding resolves (`workplan check`) |
> | Tests -> claims | Which documented guarantees does this test support? | Every test is bound by a capability, or excluded with a reason |
> | Change -> documentation | Which explanations, examples and contracts must change? | The reconcile obligation list |
> | Evidence -> claims | Which current statements depend on this result? | The [evidence](./workplan-evidence-claims.md) dependency view |
>
> **Code to documentation.** `cargo workplan coverage code` lists the public items of
> the workspace, found through the [source index](./source-index.md), that no
> capability binds. Each is reported with its path.
>
> **Tests to claims.** `cargo workplan coverage tests` lists the tests that no
> capability binds.
>
> **Exclusions.** An item or test excluded from coverage is listed in a workspace file
> with a reason. An exclusion with an empty reason is an error, and an exclusion for an
> item that no longer exists is an error.
>
> **The account.** For a repository revision the coverage report names every gap in
> either direction, so the current account is complete or states what is missing.

## Usage

> [!PROPOSED change:workplan-two-way-coverage]
> - `cargo workplan coverage code` prints the public items without a capability.
> - `cargo workplan coverage tests` prints the tests without a bound guarantee.
> - `cargo workplan check` includes both reports once the capability is delivered, and
>   a gap fails the check.

## Examples

> [!PROPOSED change:workplan-two-way-coverage]
> Adding a public function `parse_partial` to a crate and running the code report:
>
> | Gap | Path |
> | --- | --- |
> | Public item without a capability | `raf_okf::parse_partial` |
>
> The gap closes when a capability lists the path in `bindings.code`, or when the
> workspace exclusion file lists it with a reason.

## Constraints

> [!PROPOSED change:workplan-two-way-coverage]
> - Coverage only reports. It never edits a capability document or a binding.
> - Visibility is read from the source index, not guessed from naming.
> - Every exclusion carries a reason, and the reasons are readable in the repository.
> - A capability still in `considered` or `designed` binds nothing, so the checks that
>   need bindings apply from `implementing` onward.

## Explanation

A design promise needs an accounted-for realisation, and functionality that has been
built needs documentation that follows it. This is the two-way duty of the GMEOW
[Grounding Contract](https://github.com/Blackcat-Informatics/gmeow-ontology/blob/a631f73e6e39b4493d295bcd83b4deec80637f54/docs/GROUNDING.md). Checking only from documentation to code catches promises that were
not kept and misses behaviour nobody wrote down. Checking from code to documentation
and from tests to claims closes the other half.

The direction table also separates facts that become true at different times: a design
is accepted, code exists, tests address claims, executions provide evidence, and the
documentation describes the supported result. A planned feature stays visibly planned
until its implementation and documentation justify a stronger description. The
coverage reports keep the unfinished part visible, and they turn documentation
maintenance into work the system can identify: when a referenced API disappears,
the affected section receives a `binding-missing` obligation, and when a bound example
stops compiling or running it receives an `example-failing` obligation.

## Verification

> [!PROPOSED change:workplan-two-way-coverage]
> - A fixture crate with one bound and one unbound public item: the code report lists
>   only the unbound item.
> - A fixture with a private item: it does not appear in the code report.
> - A test bound by a capability is absent from the tests report, and an unbound test
>   appears.
> - An exclusion with an empty reason, and an exclusion for a missing item, each fail
>   with the file named.
> - Reports are identical across two runs on the same tree.
