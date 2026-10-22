<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Platform guard patch handoff

`recovery-source-platform-guard.patch` is metadata only. `git apply --check` passed against the current strict checkout; rustfmt parsed the companion draft. No product files were changed, no caller was compiled, and no proposed test was executed.

The patch adds borrowed retained-source guards and an owned compiler-platform value. Bare `unix` and `windows` come from their exact printed atoms, never from `target_family`. Compiler output missing an OS, containing multiple OS entries, or containing malformed/non-atomic metadata is refused. Features, custom predicates, `test`, and empty logical constants remain unknown. `cfg_attr` is implication. Every definite false result therefore requires a platform-false certificate. Unknown source-route alternatives cannot authorize a skip.

Own item, containing inline modules, external module declarations, external file inner attributes, and target-root inner attributes are conjoined. Multiple declaration sites or conditional path alternatives remain unknown. Unsupported member/foreign/exported-macro ownership also remains unknown. Invalid IDs and inconsistent retained source context are actionable errors. Source inventory is never filtered.

The guard pointer vector uses native `Memory` and the existing cumulative ItemBytes account, with typed native/budget failures. Metadata parsing reuses existing `syn`/`CfgExpr`; their internal parsing allocations have no native admission callback. Guard evaluation reparses validated retained metadata and has the same parser-temporary boundary. This patch must not be described as complete physical parser admission or complete cfg evaluation.

Proposed exact bindings under `raf_source::platform::tests`:

- `parent_platform_guards_do_not_filter_the_source_inventory`
- `inner_crate_and_external_module_guards_are_both_observed`
- `nonplatform_conditions_and_cfg_attr_do_not_invent_exclusion`
- `compiler_platform_refuses_incomplete_or_malformed_output`
- `conditional_module_routes_remain_unknown`
- `invalid_item_is_an_actionable_error`

The parent-module fixture uses the names of both actual Windows disk tests and retains all four test functions. Workplan must obtain actual configured-target compiler output, merge resolved source alternatives by OR, and report NotApplicable only when every alternative is definitively false. True/unknown alternatives retain ordinary exact-listing verification. NotApplicable is never Passed. The original three-host verification requirement remains unqualified by this metadata patch.

Before acceptance: compile actual Source and Workplan callers, run these proposed tests and adjacent exact-listing/unknown/refusal controls, qualify original ItemBytes/Work query failures, update capability bindings, and run the required gates. No source mutation or executable session is owned by this reviewer.
