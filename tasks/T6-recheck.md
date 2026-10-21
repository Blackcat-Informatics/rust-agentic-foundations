VERDICT: PASS

# T6 recheck (commits ab62795 and be93118, head be93118)

Verified in the scratch clone `scratch/t6review` checked out at be93118. The worktree was not edited.

## Commands

| Command | Exit | Result |
|---|---|---|
| `cargo test -p raf-census --locked` | 0 | 22 unit, 20 forms, 4 golden, 16 groups, 5 page, 11 reports, 20 units, 1 doctest; all pass (the extra "1 passed" line in the log is my own throwaway probe file, not part of the branch) |
| `cargo test --locked --test docs_contract --test self_workplan --test repo_gates` | 0 | 28, 68 and 1 passed |
| `cargo run --locked -q -p cargo-workplan -- --root . check` | 0 | clean; the assessment digests are current |
| `cargo run --locked -q -p raf-census --example baseline` | 0 | output byte-identical to the earlier run, so the capability Examples still match |

## Findings 1-7

1. Constants and statics have no print: the page Units bullet, the Contract bullet and the Constraints bullet now say so, and name what the initialiser contributes (integer constants, strings, tables). Fixed.
2. Doc comments:
   - My probe `/// doc\nlet x = f(a); x` now gives `# [ doc = " doc" ] let $1 = f ( $0 ) ; $1`. The word `doc` is kept, where before it was `$1`.
   - `is_attribute` skips `#[..]` and `#![..]` groups in both `rename_params` and `rename_locals`.
   - The page Token source and the Constraints now state that a doc comment is the attribute tokens, that its text counts, and that attribute names are never renamed.
   - Pinned by fixtures `doc_comment_renamed_local` and `doc_comment_text_differs`, and by the forms test `the_names_of_an_attribute_are_kept_beside_a_local_of_the_same_name`. Fixed.
3. Shipping now reads "some target that reaches it ships and the item is not test code in that reach", in both the page and the Contract. Fixed.
4. Shim predicate: the page and the Contract say it is judged at the top level only, and the closure example is stated. Pinned by the fixture `shim_closure_body_with_let` (equal at the shim level, with a shim print). Fixed.
5. R1 at load level: `groups::renamed_locals_inside_macro_arguments_land_in_one_body_group` loads three functions through `Census::load`. Two differ only in locals renamed inside `format!`, `vec!` and `assert_eq!` arguments and land in one body group (at least 20 tokens). The third calls a different method and is not in the group. Fixed.
6. The contract, constraints and verification assessments are accepted afresh. Their digests changed, and `workplan check` is clean, so they are current. I re-read the three sections against the code; the claims named in findings 1 to 4 and 7 are now true, the new test names exist and are bound (98 tests), and Verification lists the new fixtures. Purpose, usage and examples are unchanged and still sound. Fixed.
7. `min_tokens` now comes from `CensusOptions::min_tokens`, with the ledger key attributed to the ledger capability. The macro-argument sentence now says a file whose macro arguments do not lex cannot reach the census (`raf-source` must parse the file), and that a slice that does not lex is an error. "`ref` and `ref mut`" is stated. Fixed.

## Goldens

`git diff 7a49256 be93118 -- crates/raf-census/tests/golden.rs` shows no pinned value changed. Six fixture lines were added (the three new pairs). The only removed line is the last line, `empty_body 0 - -`, re-emitted with its closing quote moved onto its own line. The manifest is unchanged, so no `normal_form` bump is needed and the stated "no version bump" holds. The new attribute rule changes the fingerprint only of a body that holds an attribute, which no previous fixture had.

## R1-R18 status

R2 to R10 and R12 to R18 are unchanged from T6-review and hold. R1 is now demonstrated at load level (finding 5) in addition to the `Normalizer` registry pairs. R11 holds, with the page text corrected.

No new findings.
