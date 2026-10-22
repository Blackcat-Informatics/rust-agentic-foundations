<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Edition-aware unresolved re-export origins

The original actual coverage test failed in Rust 2015: `gen::Made` declared
inside `api` was attributed to `made::api::gen::Made`, instead of
`made::gen::Made`. Exact original failure:
`raw/queue-coverage-reexport-edition-repro.log` (session 2004, exit 101).
This was an adapter defect, not missing source resolver semantics.

The repair routes known prefixes through `Workspace::resolve_from` in every
actual declaring-module context. The missing generated tail is appended after
that lookup. An unknown head remains outside known workspace coverage; every
native error other than `NotFound` becomes a target surface error. All origins
are resolved before any target item is judged, avoiding partial target counts
and exclusion matches when origin resolution refuses. Absolute heads retain
their leading separator; repeated `super` is resolved as one native prefix,
including when its following generated namespace is absent. No edition API,
source resolver or source-index document was changed.

The real paired compiler witness is `scratch/reexport-origin.rs`. Four commands
use `rustc --edition {2015|2021} --cfg {root_origin|module_origin}
--crate-type lib --emit metadata`; the two correct edition/origin pairs succeed
and both wrong pairs refuse with type mismatch. The enclosing assertion command
returned exit 0 (session 12458); exact diagnostics/statuses are retained in
`raw/queue-coverage-reexport-origin-rustc.log`. A separate actual Rust 2015
`--cfg absolute --cfg root_origin` compile returned exit 0, retained in
`raw/queue-coverage-reexport-absolute-rustc.log`. These are compiler semantics
witnesses, not new source expansion support.

The existing inline `super` fixture also exposed an invalid Rust 2015 glob:
`pub use deep::*` inside `outer` cannot express the intended module-relative
import there. It now uses `self::deep::*`. The actual generated root `Made`
gap remains when only `outer` is bound; the exact gap is asserted rather than
filtered. Binding the real `gen` origin covers those lines and the re-export.
Earlier family failures and diagnostic logs remain historical evidence.

The expanded surface fixture separately pins trait associated type, constant
and method ownership; defining-item order under opposite alias spellings; a
real path-dependent workspace re-export; an external standard re-export; and
binding repair. It has a specific public-surface claim owner. The module-input
digest clause now names the real evidence and assessment owners asserting
own-module and child-file edits move inputs while sibling edits remain current.

Independent source recheck is `tasks/queue-coverage-origin-review.md`. It closes
the leading-absolute finding and records no further bounded source defect.
The final semantic family passed all 30 tests in
`raw/queue-coverage-origin-final-family.log`, including actual repository-wide
zero-debt coverage. The enclosing session 84197 exited 101 because the following
Clippy step found return-type complexity and one missing semicolon, not a test
failure. A small owned `ListedSurface` result and the semicolon repair changed
no selection or judgment rule. Session 6669 then returned terminal exit 0 for
current warnings-denied all-target Clippy and both affected origin regressions:
`raw/queue-coverage-origin-clippy-current.log`,
`raw/queue-coverage-origin-style-editions.log` and
`raw/queue-coverage-origin-style-super.log`. Original failed logs remain intact.
No assessment acceptance, lifecycle completion, integration or full-gate result
is inferred from these focused passes.
