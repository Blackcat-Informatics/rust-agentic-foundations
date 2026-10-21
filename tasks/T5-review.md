VERDICT: BLOCKED

# T5 review (commit a8452e5)

## Scope
Task 5: crates/raf-census/src/report.rs, examples/baseline.rs, tests/support/builtin.rs, tests/reports.rs, serde derives in group.rs/job.rs, lib.rs exports. Checked against plan Rev 1 "Reports"/"Example", rows R10/R14, and the designed consumers census-migration, census-cli, census-debt-ratchet. Reviewed in a clone at scratch/t5review (worktree untouched). No timing runs, no mutation passes.

## Commands (exit status)
- cargo test -p raf-census --locked: 0 (22 unit, 19, 4, 15, 9 reports, 20, 1 doctest)
- make clippy: 0
- cargo run -q --frozen -p raf-census --example baseline, twice, cmp: 0, identical (3428 bytes); same output as the pinned BUILTIN golden (test passes through the shared builtin.rs code)
- same with `-- crates` from the repo root, twice, cmp: 0, identical (about 2.0 MB)
- `-- .` 0 and equals absolute repo root; `-- crates/nonexistent`, `-- /nonexistent`, `-- ../x`, `-- crates/raf-census/src/lib.rs`: each exit 1 with a named error (fail hard, good)
- `-- <absolute>/crates`: exit 1 (see finding 2)
- probe test (deleted afterwards): Index::from_json with `body` and `shim` keys absent: Ok (see finding 1)

## Verified
- Members equal the plan exactly. Baseline: normal_form, min_tokens, units, body_groups, shim_groups, hex_tables, shared_constants. Index: normal_form, units (symbol, package, file, line, shipping, tokens, body, shim), groups (kind, fingerprint, members), jobs (id, home, matches{symbol,class}, copies). All types have deny_unknown_fields; sorted keys and one trailing newline are tested. Negative fixtures cover unknown key (top and nested), missing key, wrong type, unknown class, index-as-baseline. Round trip tested for both.
- Byte determinism on fixtures and on crates/ (R14) tested and confirmed by hand.
- The no-argument example matches the form `cargo run --frozen --package=raf-census --example=baseline` (raf-workplan/src/bound/plan.rs:359): no argument, any cwd. Its asserted fingerprints (body:6bb022e241a79a28 with 21 tokens >= min 20, shim:203616cb54a3e53d) are the real output, and the golden string is compared against the same builtin() the example prints.
- Index units carry symbol, file, line, package, body and shim, so census-migration ("members of a fingerprint are the units that list it") can map old fingerprints through symbols.

## Adjudications
(a) Index `shim` = shim_key: not a problem. Grouping (group.rs:186) and job matching (job.rs:88) use shim_key only, so a trait-impl or Self:: forwarder is in no shim group and matches no forbidden shim fingerprint; an old ledger or index at the same normal form cannot have mapped through its shim print either. Migration therefore loses nothing. `body` for a shim candidate matches job.rs:85 (jobs do match on its body fingerprint), so listing it is correct. The CLI `fingerprint <SYM>` command must not take its shim from the index; it should read the library Print, and it should say when the unit's shim is not grouped (trait-impl/Self::), otherwise a user will forbid a shim fingerprint that can never match. This belongs to the census-cli change; not a T5 defect. The doc comment on IndexUnit.shim ("the fingerprint the unit groups under") is accurate.
(b) Constants as lowercase `0x...` strings: correct (u128 exceeds JSON numbers; round trip tested; matches the ledger's "integer literals compared by value" once parsed). Accept.
(c) Directory argument: relative = root of a tree at cwd, absolute = its own tree with root ".". Works for a workspace root in both spellings, but not for a subdirectory: see finding 2.

## Findings
1. (required) crates/raf-census/src/report.rs:120-123 (IndexUnit.body, .shim): `Option<String>` fields deserialize as None when the key is absent (serde default), so `Index::from_json` accepts a unit with no `body` and no `shim` key. This contradicts the from_json docs (report.rs:216-217 area: "a missing key ... is not a report"), the plan's closed-schema requirement, and census-migration reading an older index report: a truncated or wrong-shape file is read as units with no fingerprints and maps nothing, silently. No test covers a missing optional key. Fix: require the key, e.g. `#[serde(deserialize_with = "...")]` using `Option::<String>::deserialize` (a deserialize_with function disables the missing-key default), on both fields; add a negative fixture removing `"body": null` and one removing `"shim": null`, with the null-carrying neighbour as the valid case.
2. (non-blocking, fix with finding 1) crates/raf-census/examples/baseline.rs:22-28: the same directory gives different results by spelling. `crates` (relative) loads and reports; `<abs>/crates` fails with `cargo-workplan/Cargo.toml: invalid manifest: edition.workspace = true but no [workspace.package] edition`, because the absolute form makes `crates` the tree root so the workspace manifest above it is invisible. It fails hard rather than producing wrong output, but the docs say "the workspace under that directory". Fix: for an absolute path use its parent as the tree and its file name as the root (fall back to root "." only when there is no parent), so both spellings see the same tree; add a note or test for the chosen rule.

## Required work
Finding 1 (with its two negative fixtures). Finding 2 recommended in the same commit. Re-run cargo test -p raf-census --locked and make clippy after.
