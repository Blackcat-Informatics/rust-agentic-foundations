# R11b review: public API sealing (A5, A6, A8, A9, A10, A14, A15, A18)

Reviewer: independent single pass. Subject: uncommitted change in /tmp/raf-par2/c-r11b (base 44e18b7), patch raw/R11b.patch, seal log raw/R11b-seal-check.txt. Nothing was committed or edited except this file.

## Scope

24 files: fact types made pub(crate), #[non_exhaustive] on Package, Target, SourceFile, Reach, Entry, SpanFact, Entry::new, Target test settings (harness, required_features, test, doctest), SpanFact::lines as a half-open Range, LoadOptions not Copy, raf-okf Outcome/LinkTarget exhaustive and the two docs_contract fallback arms removed, registry_scan reading #[macro_export] via syn, Contract/CHANGELOG text.

## Evidence

Gates, run by me in /tmp/raf-par2/c-r11b with CARGO_BUILD_JOBS=4:
- `cargo test --workspace --locked`: every `test result` line is `ok`, 0 failed. Doctests of raf_source: 6 ordinary (incl. Entry::new) + 6 compile_fail (Package, Target, SourceFile, Reach, Entry, SpanFact), all ok.
- `cargo test --locked --test docs_contract --test repo_gates`: 139 and 57 passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0 (results were cached from identical inputs).
- `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked`: exit 0, so no private-intra-doc-link and no private_interfaces warning (the latter is also covered by clippy -D warnings).
- `make msrv` (rust-validate 1.98.1, workspace and fuzz): both finished clean. `cargo fmt --check`: clean.

(1) Cargo defaults. Checked against Cargo's documented target keys and its target construction: `test` default true for lib/bin/test and false for bench/example; `harness` default true for every target table (bench included); `doctest` default true for lib only; `required-features` empty; build script harness/test/doctest false. `derive` in target.rs matches all of these. Fixture `harness_required_features_test_and_doctest_are_read_from_each_target_table` asserts, per kind, values away from default (including `test = false`, bench `test = true`, example `doctest = true`) beside table-less targets with defaults, the build script, and a plain package; the wrong-type test covers lib/bin/test/example tables and both scalar and array keys and asserts the key name appears in the error.

(2) Reachability. Public surface after the change is `FieldShape, SpanFact, TestRole, Vis` from facts; no public signature returns a pub(crate) type (compiler/clippy/rustdoc clean). `SourceFile::facts` is pub(crate) and the in-crate unit tests in facts.rs now carry the assertions that used it (the three integration tests moved, content preserved; no binding in source-index.md pointed at the removed names). `raf_source::syn` re-export is documented.

(3) Half-open lines. `Positions::span` builds `start.line..end.line + 1`; the crate-root item uses `1..lines + 1`. grep finds no remaining start_line/end_line in code or docs. Tests assert one-line `n..n+1`, a 3-line struct `3..6`, a crate root `1..6`, CRLF/BOM cases `1..2`, a 1-based wide-character line `1..2`. raf-okf Span tests (`5..6`, `3..7`) confirm that crate is the same convention. docs_contract only uses raf-okf spans (`.lines.start`), unchanged by this patch.

(4) compile_fail doctests. I wrote a scratch crate (/tmp/r11b-scratch, path dependency, own target dir) with all six patterns with every current field and no `..`: Package, Target, Reach, Entry, SpanFact each give exactly E0638; SourceFile gives E0638 plus the uncoded "pattern requires `..` due to inaccessible fields". The doctests name `E0638`, so rustdoc fails them unless that code is emitted. The seal-check log's step 1 (base fields on base) compiles, step 2 fails on the patch, step 3 (current fields, seals removed) compiles for the five types that can be named. So five of six would compile without the seal.

(5) struct_excessive_bools. `Target` has four bools (shipping, harness, test, doctest). The allow is item-scoped and has a reason, as AGENTS.md requires. A TestSettings sub-struct would add a second public type to seal and an extra pattern layer for three keys that mirror Cargo one to one. Justified.

(6) Text. CHANGELOG, source-index.md (Contract, Constraints with the A14 and A18 sentences, A10 sentence on Workspace::load, Verification) and okf-profile.md state what the code does. No issue or PR numbers, no process wording, no attribution.

## Findings

REQUIRED: none.

ADVISORY:
1. The SourceFile compile_fail doctest cannot meet the "compiles without the seal" half of the contract: with `facts` pub(crate) the pattern is rejected for the private field even unsealed. It is still discriminating, because E0638 is emitted only by the seal. The Verification bullet says "every field and no `..`" for all six; worth one clause ("SourceFile omits the crate-private facts").
2. `Target::doctest` doc says "cargo honours the key only for the library", but a `[[example]] doctest = true` is reported as `true` (the fixture asserts this). That is faithful to the manifest, so state in the field doc that the field reports the written key and only the library's is acted on.
3. `[lib] required-features` is read without complaint; cargo rejects it for a library. Harmless for an index, optional to note.
4. source-index.md lines were extended past the surrounding wrap width (the "names its key, such as `bin.harness`. The library is..." line and the `span`/`body` (a `SpanFact`) line). Cosmetic; no gate fails.

VERDICT: PASS
