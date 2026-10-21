# C2 report: GAP-05, one parse

Status: DONE (one environment limit on `make cross-check`, below).

Commits
* `0f7aaa3` feat(raf-source): `Item::parameter_bindings()` and `Item::initializer()`.
* the following `fix(raf-census)` commit: the census takes both from raf-source.

Decisions
* Exposed the facts (no fallback helper needed). Both live in one shared `Arc<ItemDetail>` on the
  item fact and the `Item` (one pointer), because two separate fields took `Item` to 264 bytes and
  the 256-byte test bound holds at 232. Read in `facts.rs` on the parse thread, charged to the
  fact meter.
* `self` is listed first for a receiver (as assigned); the census filters it out, so `Unit::params`
  and every golden are unchanged.
* `units/sig.rs` and its `syn` re-parse are deleted. `collect` clears the lexer's span table with
  a drop guard (`proc_macro2::extra::invalidate_current_thread_spans`, available with
  `span-locations`), on success and on error.
* Constraint of `cap:census-normal-form` and the page's Token source state the bound that holds:
  the census parses no signature or item; a too-deep signature is `Error::Load` naming the
  recursion budget.
* Docs: source-index Contract, bindings and tests; census-normal-form tests, Error text, Constraint;
  assessments re-accepted: source-index contract/usage/examples/constraints, census-normal-form
  contract/constraints. `--verdict` strips the SPDX header of an assessment file, restored by hand.

Tests
* raf-source `symbol_index.rs`: bindings (self, mut, ref, tuple, struct, slice, raw ident, type
  names excluded, trait default and required), initializer ranges, sharing across targets.
* raf-census `units.rs`, on a 2 MiB thread: 8 x 60 signature Ok with one unit; 24 x 60 is
  `Error::Load` naming the recursion budget; 8 x 60 constant type loads; one tree loaded twice on
  one thread leaves the span table empty (first token offset 0 before, between and after).

Checks (exit status)
* `cargo test -p raf-source -p raf-census --locked`: 0
* `cargo test -p raf-source --release --locked`: 0
* `cargo test --locked --test docs_contract --test self_workplan --test repo_gates`: 0
* `make fmt clippy doc reuse issue-refs`: 0
* `cargo workplan --root . check`: 0; `assess --list` has no stale entry; `make docs-sync` is a no-op
  on a second run.
* `make cross-check`: 1 in this environment, targets `x86_64-apple-darwin`, FreeBSD and
  Windows-gnu are not installed and the nightly toolchain cannot be installed here. The change adds
  no platform code. Not a code failure.

Blockers: none for the work; cross-check needs the targets (CI has them).
