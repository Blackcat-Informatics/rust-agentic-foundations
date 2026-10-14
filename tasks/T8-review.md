# T8 review: `Workplan::preview`

## Scope
Staged diff on top of 45ff252: `crates/raf-workplan/src/preview.rs`, `tests/preview.rs`, `lib.rs` (module, re-export, `# Preview` doc), `CHANGELOG.md`. Checked against plan Task 8 / G14 / list U (preview-limit bullet), the document-model PROPOSED grammar, the generated-docs preview text, the CLI capability, and the raf-okf marker scan (`body/marker.rs`, `body/blocks.rs`). Experiments ran in a separate worktree copy (`/tmp/t8rev`, own index); the staged tree was not touched.

## Evidence
1. Fidelity (scratch harness over realistic documents): the document-model Examples shape (`Para.` / blank / PROPOSED block) reads `Para.\n\nThe parser also accepts input.` with no stray blank and no marker. Block at EOF without final newline, empty-content block, CRLF, mixed CRLF/LF, lone CR, bare `>` line, UTF-8, adjacent blocks of two changes, list-item block, nested quote (one level removed) all give the expected text; every line keeps its ending; other changes' blocks stay byte-identical.
2. Fence limit, 19 layouts (backtick and tilde fences, fence first/middle/last in the quote, fence open and closed inside the quote with the quote continuing after, fence without a space after `>`, fence followed by a new marker, fence inside a list-item quote, nested-quote fence, longer outer fence, fence at EOF, unquoted fence after a quote line): every layout where raf-okf cuts the block at a fence and the quote continues returns `Prefix` with the fence line. Layouts where the block legitimately ends (unquoted fence, blank line, next marker) are returned faithfully (the trailing `> text` is then ordinary text outside the block, kept as written). I found no layout that yields silently wrong text. Reason: the block always ends either at a non-`>` line, at a marker, or at a fenced line; the fenced-line case is exactly "next line starts with `>`", which the check refuses unless it is a marker.
3. Hostile input (release): 100k-line block, 30k blocks with and without blanks, 30k blocks of another change, 900 KB single line, marker without `]` (load finding -> `CapabilityNotLoaded`, no panic), marker at EOF, over-1 MiB document (load finding): no panic, whole run about 3 s. Pure: DiskTree test asserts bytes and mtimes unchanged; two calls equal.
4. Mutants (all killed): marker kept; prefix over-stripped (`trim_start`); prefix under-stripped (no space strip); strip-always-one-byte; other changes reconciled; capability skipped when it has no block; first capability skipped; empty line keeps indent; fence guard disabled (the plain `if false` form fails to compile on unused variables, a variant with `false &&` was killed by the fence test).
5. Errors: unknown slug names it and lists open changes (empty list worded); broken change record -> `ChangeNotLoaded` with its findings; broken listed capability -> `CapabilityNotLoaded` with findings; listed capability without document -> `CapabilityMissing`; nothing is skipped silently.
6. API/docs: items documented, `#[non_exhaustive]` error, SPDX present, no process wording or issue tokens (`make issue-refs` passes), CHANGELOG entry matches behaviour.
7. Gates (in the copy with the staged diff applied): `cargo fmt --check`, `clippy --workspace --all-targets -D warnings`, `cargo test -p raf-workplan` (19 preview tests, 6 doctests) and `--release`, `docs_contract` + `repo_gates`, `make reuse`, `make msrv`, `make issue-refs`, `make doc`: all pass.

## Findings
REQUIRED: none.

ADVISORY:
- A1. A tab after the quote marker is kept: `>\ta` becomes `\ta`, which at column 0 is an indented code block, while CommonMark reads `>\ta` as a paragraph with two spaces of indent. raf-okf's own `content()` strips only a space, so preview is consistent with the model, but the reconciled text can render differently from the quote. Either refuse a tab right after `>` with `Prefix`, or document it. Rare in practice.
- A2. The `preview` doctest only shows the `is_err` path; a success example (a PROPOSED block reconciled) would document the main behaviour. Not blocking since the integration tests cover it.
- A3. A block placed directly under a text line (no blank) merges with it when unquoted. This is a property of the author's layout, not a defect.
- A4. `reconcile` runs `proposed_blocks().iter().any(..)` per block, so it is quadratic in the blocks of one document; at the 1 MiB limit (30k blocks) it took well under a second, so no action.

Reminder for Task 9 (list U): the generated-docs Constraint bullet for the preview limit must be added; the docs still carry the old "printed and never written" wording and the `cargo workplan docs preview` block.

VERDICT: PASS
