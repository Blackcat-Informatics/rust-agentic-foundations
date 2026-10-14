**R5a fixed: extern-prelude aliases and stack-safe trait keys** (commit e472918, pushed)

- **Gaps:**
  - **G2 / CA-F1 (CRITICAL):** `pub use serde;`, `use std;` and `pub use dep::{self, ..}` resolved through their own binding, giving `Cyclic`. A trait impl through such an alias failed the whole load; this repository's own `raf_source::syn::File` was `Cyclic`.
  - **CA-F7:** trait-key recursion could abort the process on valid input.
- **Fix:**
  - An import never consults its own binding. A single-segment import of an outside crate binds nothing, so paths through it are `NotFound`, and trait keys follow it to the dependency path. An import of a workspace library binds its crate root.
  - Edition 2015 paths start at the crate root, and a head names a crate only through `extern crate` or the injected `std`/`core`. `#![no_std]` is honoured.
  - **rustc's rules, verified with the compiler:** an item or explicit import shadows a crate, and a glob-imported name collides with it (E0659). The plan's broader "local item vs crate → Ambiguous" was corrected to this.
  - **Order-independent:** a path waits for its module's globs and for unsettled explicit imports before reading a crate, and a final pass re-checks every import against the settled tables.
  - Trait keys are derived with an explicit worklist. 20,000 chained aliases or globs key on a 2 MiB thread, where the old code overflowed at about 2,000.
- **Checks:**
  - Fixtures from the plan plus every-order fixtures, over 24 orders and with or without globs. Each fails on the old code; the parts that are already correct have mutation checks.
  - The reviewer's random-program search (1,440 programs × 12 orders) found no remaining order dependence in R5a's scope. Two cycle shapes rustc rejects belong to R5b.
- **Registry:** 1,183 of 1,213 crates load. `UnresolvedTrait` went from 19 to 5, and the remaining 5 are correct refusals (traits from `include!` or macros, and a cfg'd-off invalid module).
  - Now load: bincode, ena, option-ext, core-foundation, rmp-serde, js-sys, wide, cx448, p256, p384, p521, hmac, ripemd, sha3.
- **Real workspaces:** item resolutions and trait keys are unchanged across 99k+ paths. Alias paths now resolve as rustc does; for example, 135 katamari test paths behind `use …testkit as support;` now resolve.
- **Gates:** 357 tests in debug and release, clippy, rustdoc, docs_contract 139, repo_gates 55, cross-check, msrv, reuse, `make check`, and the hook.
- **Known limit, stated in the Contract:** a glob carrying an alias of an outside crate's *type item* under a crate's name (`pub use serde::de as other;`) can't be classed by namespace, so it is not refused. rustc gives E0659 there.
- **Review:** `tasks/R5a-review.md`, VERDICT: PASS after two rechecks.
