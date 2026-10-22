<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Model fixture and macro coverage recheck

Bounded read-only source recheck; no builds or product edits. Reviewed
`coverage/lines.rs::invoked_sites`, the strengthened existing macro test,
`support/cases.rs::coverage_cases` and binding rows, the shared binding builder,
and the kinds inventory/diagnostic loop. No concrete defect found in these
repairs. Full adoption and active Clippy/sync/candidate work remain separate.

The actual edition defect has a coherent narrow repair. The source resolver
documents `resolve_from_in` as reading a use path; its `Heads::Use` predicate
roots a bare 2015 head at the crate root. A single textual macro name instead
needs the current module first. `invoked_sites` now prefixes only that single
name with `self::`, then retains the existing ancestor search. Qualified macro
paths retain their original lookup. The actual macro test constructs a nested
definition/invocation plus an unresolvable generated-type reexport in both
2015 and 2021, checks local macro resolution, and requires no orphan lines
when only real functions and `made::touch` are bound. It does not bind the
macro definition or the whole module to hide the earlier orphan. Existing
unreached macro/invocation orphan-range assertions remain in the same test.
This does not establish new universal textual-scope ordering or macro-expansion
semantics beyond the existing model.

The small coverage fixtures now match their actual supplied source. They
remove the broad demo auxiliary document only when using `listed_workspace`,
which genuinely lacks its modules/tests/examples. Their clean neighbor runs
the complete check against that exact workspace with real and bare bound.
The item-gap negative then removes only bare's binding; the real-to-bare call
keeps its lines reached independently of its public documentation obligation.
The second-test negative similarly has case call other, so its expected missing
test binding is not conflated with orphan lines. Scope remains active through
real bindings even in explicit empty-scope rows.

The generated-reexport negative binds `made::touch`, whose actual body uses
`Made`; `make!(Made)` supplies that generated identifier to the existing
coverage heuristic. No whole-module binding is retained, so the reexport at
the asserted line 12 remains a genuine unresolved public surface entry.
The full demo surface-error fixture retains the auxiliary owner and changes
both private globs to public globs; it therefore still exercises the actual
ambiguous public surface. Existing exclusions in these fixtures were not
expanded to mask the new failure. No new ceiling or permissive case assertion
was found.

Binding rows keep a real first runnable example and retain invalid examples as
additional entries. Their helper uses the first example for valid prose and
claim markers, leaving each intended bad code/test/example binding present.
`Case::assert_exact` still requires its named kind/file/location and rejects
unlisted extra kinds and findings in neighboring files. The kinds loop catches
assertion panics only to collect diagnostics, then unconditionally fails when
any were caught before checking produced-kind coverage. It does not count
failed cases as passing, filter findings, or remove the negative cases.
Removing the obsolete `debt-without-strict` inventory entry matches the already
removed production kind; the exact inventory equality remains enforced.

The prior document-model review points are closed on source inspection: graph
nodes explicitly name evidence/assessment/coverage, and writer/header markers
include the actual Debt create/replace/SPDX owner. Its typed three-variant
round-trip scope remains as described in the earlier review.

Execution limits: owner reports the edition reproducer failed before repair
and its focused test passed afterward, all four registry cases and Debt 29
passed, and states 16/generated 40 passed. This reviewer did not run or re-run
those checks and does not infer a terminal result for active session 84439.

Final source refresh: the owner subsequently extracted
`generated_reexport_case` and `ambiguous_surface_case` to repair a Clippy length
finding. Inspected the resulting helper bodies and their actual callers: the
small/full document inputs, made::touch binding, public glob transformation,
expected kinds and locations are unchanged. This is not a new execution claim;
the owner reports exact registry replay 20353 is active before fresh Clippy.
