Closes #29

## What changes

Judging coverage no longer tests every exclusion against every entry. A private `ExclusionIndex` (exact item paths and prefix texts in two hash maps, plus the matched flags) is built once; an entry costs one lookup per path and one per proper ancestor of it. Every matching exclusion is still marked, so an exclusion no entry uses is still reported, and the smallest position in the record still gives the reason. With no exclusions nothing is looked up. The old `matches` and `exclusion_for` are gone from production; a product reference implementation lives in the test module as the oracle.

* **work.rs**: `ExclusionsIndexed`, `IndexLookups` and `ExclusionMarks` count the work at the insert, lookup and mark sites.
* **Tests**: exact counts; growth in entries, in exclusions, and in both together (the product fails the last, the index does not); a ceiling linear in entries plus exclusions on every kind; the marking and unused-exclusion behaviour through `Workplan::check`; an enumerated cross product of 240 exclusions over 120 paths compared with the oracle on the first match and the full matched vector, on a rebuilt index too.
* **Docs**: the capability's bindings and Verification text, the changelog, the assessments.

## Validation

`make fmt clippy test test-release doc deny reuse msrv fuzz licenses issue-refs` exit 0. Mutants in scratch clones, each fails a named test: stop at the first match, whole path as the prefix key, skipping the nearest ancestor, choosing the largest position, building the index per entry, removing the marking loop, and the old product code against the growth tests (a growth failure at units 1, 2, 3; the numbers are in the test output). An independent reviewer compared the index with the old code over single exclusions, all ordered pairs and the full list forward and reversed, and ran five mutants of their own: three killed, one that only adds work the counts cannot see and one that changes nothing for any record that can load survived, as expected; the report output on a nine-exclusion scratch record was byte-identical to the old code's.

## Disclosures

* `make cross-check` cannot run on this host: the machine selects one dated nightly through `RUSTUP_TOOLCHAIN`, which has the Windows standard library but not the Apple or FreeBSD ones, and installing a toolchain is forbidden here. The Windows check of `raf-source` was run by hand and passes; the Apple and FreeBSD targets are unrun locally and only the hosted `cross-check` job covers them. This change does not touch `raf-source`.
* The workflow ran `rustup target add x86_64-pc-windows-gnu` against the pinned nightly while investigating `cross-check`, which changed that toolchain's installed components. Adding standard libraries to the Stage toolchain is the machine owner's action, not the workflow's; it was not authorised and a guard hook stopped any further change. The hand-run Windows check above depends on it. Whether to keep the target (or remove it with `rustup target remove --toolchain <the pinned nightly> x86_64-pc-windows-gnu`) is the owner's decision.
* The index differs from the old code only for paths with an odd run of colons (`a:::b`), which the record loader and Rust paths do not admit.
* `check` still loops over exclusions x crate targets to find the packages a record needs; that is a product over crates, not entries, and is stated in the capability.

## Least confident

That the counted work reflects all of judging: the exclusion side is counted at its three sites, but a loop added elsewhere that scans the exclusions per entry would not be seen by a count (a limit of every count-based bound in this repository).
