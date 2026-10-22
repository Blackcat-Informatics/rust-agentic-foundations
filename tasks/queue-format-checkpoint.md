<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Source-format capture checkpoint

The source-format repair is frozen for primary independent review. The whole
RDF/default/adoption delivery is unfinished; no completion, full-gate, commit or
push claim accompanies this checkpoint. Existing donor stashes and evidence are
preserved. No sibling settling behavior was imported or reimplemented.

## Actual production path

`Workspace::reachable` now lexes candidates through `reach::format` on its owned
worker, places invocations into the same innermost source units as ordinary
references, proves standard-macro provenance in the actual unit context, and
resolves decoded captures through the value namespace with ordinary unresolved
read and symbol-ambiguity checks. Real `Workplan::lines_report` reaches those
constants through the existing References edges; it propagates source refusals.

The rule decodes literal-leading standard `format!` fields, escaped/raw/Unicode
names and named width/precision. Explicit named bindings suppress their label and
implicit capture without hiding expression references. Ordinary literal strings,
doubled braces, numeric positions, unknown imports and custom token inputs add no
inferred capture. It is a bounded syntactic rule, not general macro expansion or
local value binding analysis. The existing module-as-code approximation for local
values sharing an indexed name remains explicit in source-index Constraints.

Both independent findings in `queue-format-identity-review.md` were repaired:
every enclosing macro input needs positive supported standard expression-context
provenance; imported aliases prove their std head at the import's own lexical
scope. Real compiler neighbors distinguish discarded malformed tokens from nested
standard expressions, and a custom local std alias from an outer standard import
that remains valid beside an inner std binding. Every admitted enclosing macro
has an actual compiler/public-reach fixture. An external parent's textual macro
also defeats an imported standard alias; qualified standard invocation remains a
positive neighbor.

Argument expressions and lexical imports pass pre-scan before recursive syn
parsing. Public API fixtures prove exactly the recursion ceiling, one unit above
it, actionable original source/line, and opaque custom input with the same large
token body. No span escapes the worker. Owned maps, strings, scope and placement
rows are charged to retained item bytes; the new counted fixture demonstrates
linear increments at three repeated-call sizes and the actual one-byte refusal
through `walk_charge`. Token and source/decoded-byte tallies return from the worker
to the calling thread instead of disappearing into worker-local counters.

## Current terminal checks

All commands below completed with exit 0. Historical failed development logs
remain preserved and are not claimed as success.

- `cargo test -p raf-source --test format_reach --test reachability`:
  16 new public API cases and all 24 existing cases;
  `raw/queue-format-contexts-qualified.log`.
- `cargo test -p raf-source --lib reach::tests`: all 16 cases, including exact
  old counts and the new work/retained-budget fixture;
  `raw/queue-format-reach-units-final.log`.
- `cargo test -p raf-workplan --test lines`: all 20 cases, including the actual
  capture/suppression/opaque-input/source-error consumer;
  `raw/queue-format-lines-final.log`.
- `cargo clippy -p raf-source -p raf-workplan -p cargo-workplan --all-targets -- -D warnings`:
  `raw/queue-format-callers-clippy-final.log`. Actual callers were also compiled
  early in `raw/queue-format-callers-first.log`.
- Production `make docs-sync`: `raw/queue-format-docs-sync.log`.
- `git diff --check`: exit 0 after final source and generated-document updates.

The root-cause named-argument probe now prints VALUE false / OTHER true for the
override, and VALUE true / OTHER false for its direct positional neighbor:
`raw/queue-format-named-current.log`. Original failing probes remain intact.

## Review surface and limits

New scanner files are `crates/raf-source/src/reach/format.rs`,
`reach/format/arguments.rs`, `reach/format/identity.rs` and
`tests/format_reach.rs`. Wiring and accounting are in `reach.rs`, `resolve.rs`
and `work.rs`. The production consumer fixture is the last test in
`crates/raf-workplan/tests/lines.rs`. Capability contract/bindings/limits are in
`source-index.md`; actual line ownership is documented in `workplan-strict.md`.
Other dirty changes predate this repair and belong to the preserved typed-policy
and adoption checkpoint.

The supported certificate deliberately does not identify transitive reexports,
crate aliases, unknown external glob exports, conditional imports/preludes or
qualified standard paths in edition 2015. Fields of other formatting macros are
not decoded by this rule. No debt, exception, exclusion, permissive policy switch
or artificial constant reference was introduced to obtain coverage. Actual
repository adoption counts must be recollected after review; the earlier
188-uncovered-claim/56-orphan-line diagnostic is historical, not a present
qualification result. All remaining adoption/lifecycle work stays owned here.
