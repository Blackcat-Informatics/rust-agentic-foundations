<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Finding and item-coverage registry repair

The owning-module test now requires exact equality between actual emitted terms,
the documented registry table and `raf_workplan::report::TERMS`. A documented or
registered term that no fixture emits fails this equality. A new emitted term
that is absent from either registry also fails.

Fixtures use the actual adapters: findings at all four location variants; public
item and test coverage with an exclusion; a macro-generated unresolved re-export;
and all four typed surface failures constructed in the owning coverage module.
The latter checks projection of existing typed failures; it does not claim that
backend resource exhaustion was forced. The separate real ambiguous-surface test
continues to exercise production surface refusal.

The collector visits top-level subjects and predicates, IRI objects and literal
datatype IRIs, plus all three positions of quoted Statements, including quoted
literal datatypes. Its focused neighbour test exercises the quoted IRI object
and both datatype positions explicitly.

`cargo test -p raf-workplan --lib report_projection_tests --locked --offline`
returned exit 0: all 3 tests passed. Exact output is retained at
`raw/queue-rdf-registry-repair.log`. Moving the test into its child module first
required correcting its relative documentation include; the current terminal log
is after that correction and the shared finding projection change.

Native Turtle/N-Triples goldens and independent parser pins were not conflated or
rewritten for this repair. The expanded command-result vocabulary is a separate
remaining check; this result does not qualify that newer surface or finish the
sole-policy/repository-adoption delivery.
