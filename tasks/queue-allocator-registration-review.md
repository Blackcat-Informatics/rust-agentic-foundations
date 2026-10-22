<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Compiler allocator registration: bounded review

Read coverage/lines.rs, the five allocator-named lines integration tests,
workplan-strict Contract/Constraints/bindings, OKF Verification registration
prose, adoption item and changelog. No builds or product edits. No whole
adoption or native-allocation migration acceptance is implied.

No concrete correctness defect found in the bounded delta. The actual bounded
parse_source callback visits native syn nodes, records direct bare attributes
and complete static byte spans, and rejects attribute arguments, repeated
static attributes or attributes not attached to a static. Macro token bodies
and cfg_attr are not expanded by this visitor. The complete source spans are
matched against actual indexed Static groups; all source-path copies retain
their own target/test identities. Missing or wrong-kind matches return an
actionable source error rather than selecting a nearby same-named item.

The added roots are gated using actual bound roots plus existing explicit
extra roots: exact indexed target participation and a separate participating
test context for test-only items. There is no exemption, line exception,
source-role mutation or dummy reference. The real static is added to roots
before the ordinary Workspace::reachable closure, which discovers its actual
type/initializer dependencies. A target reached only transitively through a
reference is not itself added to this initial participation set; current
prose correctly scopes the rule to bound participating targets, without
claiming every compiler-linked target is inferred.

Failure collection happens before coverage totals. The actual malformed and
unindexed integration cases assert error identity/path context and empty
crates/orphans (malformed also empty modules), with real module-static valid
neighbors. The implementation does not use an empty result as success.

Assertion ownership inspected:

- a_bound_test_reaches_its_registered_allocator_and_real_initializer_dependencies:
  actual integration-target static/type-alias/initializer coverage; removing
  registration or spelling it through cfg_attr leaves precisely those three
  units orphan. Registered result stays test-classified, not shipping.
- a_registered_allocator_is_not_a_root_of_an_unbound_target:
  two real targets with identical static source; only the bound target is
  covered, then binding both makes the report complete.
- a_test_only_allocator_requires_a_bound_test_context_not_a_shipping_binding:
  shipping function alone does not cover its library's cfg(test) allocator;
  adding the actual test binding covers it and its dependencies.
- malformed_allocator_registrations_refuse_without_partial_line_results:
  non-static, argument-bearing and repeated attributes refuse; actual valid
  registered static succeeds.
- an_allocator_without_an_exact_indexed_static_refuses_beside_a_module_static:
  nested function-local static, absent from the source index, refuses by exact
  indexed-definition diagnostic; module static succeeds.

The Contract's target/test/dependency statements have meaningful direct owners.
The Constraints marker follows its actual item and binds malformed/index-
missing and cfg_attr neighbors. Macro non-expansion remains source-inspected
and composes the existing source-token no-expansion law; these five new tests
do not contain an allocator-producing macro literal. Do not label that
specific boundary an independently executed macro-expansion proof. None of
these assertions claims rustc compiles every malformed fixture; they exercise
the actual source/Workplan refusal path, which is the intended boundary.

OKF Verification accurately distinguishes native calling-thread windows and
the separate heading process cap, and describes actual global_allocator
registration rather than claiming an ordinary test-body call to the static.
The adoption item and changelog do not claim the broader policy is complete.

Inspected raw/queue-allocator-registration-lines-current.log terminal result:
28 passed, zero failed. The current Clippy log ends with successful dev-profile
completion; owner reports terminal exit zero. No commands were rerun here.
The docs-sync log inspected has an invocation without a terminal verdict;
owner reported a prior document load failure and is diagnosing/repairing it.
No docs sync, candidate, contexts or broader gate pass is inferred. This
bounded source slice is released to its owner.
