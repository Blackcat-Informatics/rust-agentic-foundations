Closes #3

## What this delivers
The `raf-census` library crate's core, and `cap:census-normal-form` delivered with its change record completed by `cargo workplan change complete`.

* **Normal form 1** as an ordered tower of pure passes over a span-carrying token tree (verbatim, shim, literal tier, structural, value). Bodies are re-lexed with `proc_macro2`, macro arguments included. The value form erases borrows, derefs, `ref`/`ref mut`, the eleven value adapters, full-range reindexes, `let` annotations and type-only turbofishes. The literal tier keeps literals below 30 structural tokens, and integers compare by value in any spelling. Shim forms are for single-expression forwarders of 6 to 29 tokens; `shim_key` excludes trait-impl methods and `Self::` callees at any depth. `explain(a, b)` names the finest level at which two bodies differ.
* **Fingerprints** `body:`/`shim:` are the first 8 bytes of BLAKE3 over a length-prefixed token encoding, as 16 hex digits; `table:` and `rule:` are names. `check_normal_form` refuses any other version and names `cargo census migrate`.
* **Units** come from `raf-source` by source identity, so a module reached by several targets is one unit. Kinds are fn, method, const and static. Each unit carries symbol, package, file, line, shipping flag, print, integer constants by value, strings with lines (str, byte and C strings) and hex tables. `CensusOptions` sets `min_tokens` and `max_file_bytes`.
* **Grouping** is deterministic (count desc, tokens desc, fingerprint asc). A 64-bit collision between different forms is an error. Hex tables are detected (`table:hex-lower`/`hex-upper`) and constants above 0xffff shared across packages are reported. **Job matching** classifies matches HOME (home or entry point), VARIANT or COPY over in-memory `Job`s, which the ledger change will build from the ledger file.
* **Reports**: baseline and index JSON with closed schemas (`deny_unknown_fields`, required keys), round-trip and byte-deterministic. The `baseline` example's output is pinned byte for byte in the capability Examples.
* `docs/architecture/normal-form.md` is the normative v1. `tests/page.rs` fails when the page and the code's constants, pass ids, adapters or report members drift apart.

Moved between tracked changes: the job-match classification item moved from `change:census-ledger` into this change and was built here. Reading `[[job]]` tables and the ledger-file `normal_form` refusal stay with `change:census-ledger`.

## Validation
* Focused checks at the head: `cargo test -p raf-census --locked` (97 tests incl. 187+ pinned golden fixtures), the `docs_contract`/`self_workplan`/`repo_gates` binaries, `make fmt clippy doc deny reuse issue-refs msrv`, `cargo workplan check`. All exit 0.
* The example was run twice on its fixture and on this repository's `crates`; the outputs are byte-identical.
* Every task was independently reviewed. Five of six reviews blocked first; each was fixed in a separate commit and rechecked to PASS.
* Pending: the single full `make check` (after Stage 2), hosted CI, and the Stage 2 gap analysis and completion audit.

## Residual risks
* Normal form 1 is now fixed, and any later change to a pass is a version bump. The choices I am least sure of are listed in the comment below.
