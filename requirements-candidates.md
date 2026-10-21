# Requirement candidates — issue #3

Lines from the issue body and comments that matched a lexical rule, copied
verbatim with the rule that fired. This is a CANDIDATE list: rules both
over-match (a "should" in passing) and under-match (a requirement stated
without a keyword). The issue-analyzer keeps or drops each row, adds what
the rules missed, and accounts for every silent comment. Code fences are
skipped.

| ID | Source | Line | Rules | Text |
|---|---|---|---|---|
| B1 | body | 7 | checkbox | - [ ] `normalize`: `proc_macro2` token streams with bodies re-lexed, so macro arguments are normalized too |
| B2 | body | 8 | list-under-requirement-heading | - **structural form**: keywords, punctuation and delimiters kept; names of items the body uses kept; locals, parameters and lifetimes become `$0..` by first appearance; literals become their kind |
| B3 | body | 9 | list-under-requirement-heading | - **value form**: drops `&`, `&mut`, `*`, `ref`, the value adapters (`.clone()`, `.as_str()`, `.to_owned()`…), full-range reindex, `let` type annotations and type-only turbofishes |
| B4 | body | 10 | list-under-requirement-heading | - **literal tier**: bodies under 30 tokens keep their literals verbatim |
| B5 | body | 11 | list-under-requirement-heading | - **shim form**: single-expression forwarders (6–30 tokens), with only the function's own parameters renamed; katamari's `shim_key` refinement excludes trait-impl methods and `Self::` forwarders |
| B6 | body | 12 | checkbox | - [ ] Fingerprints: `<kind>:` + hex of the first 8 bytes of BLAKE3 (`blake3` crate); kinds are `body`, `shim`, `table`, `rule` |
| B7 | body | 13 | checkbox | - [ ] Units: fn, method, const, static, with symbol, package, file, line, shipping flag, print, integer constants (compared by value regardless of spelling), tables and strings |
| B8 | body | 14 | checkbox | - [ ] Grouping: `BTreeMap<fingerprint, members>`, groups of two or more, deterministic ordering (member count, then tokens, then fingerprint) |
| B9 | body | 15 | checkbox | - [ ] Hex-table detection (`table:hex-lower` / `table:hex-upper` across string, byte-string and array forms); shared large constants (>0xffff in two or more packages) |
| B10 | body | 16 | checkbox | - [ ] Reports (`serde_json`): baseline JSON (`min_tokens`, units, body/shim groups, hex tables, shared constants) and index JSON (symbols, groups, job matches); byte-deterministic output |
| B11 | body | 17 | checkbox | - [ ] `docs/architecture/normal-form.md` updated: the normative description of v1 |
| B12 | body | 20 | checkbox | - [ ] **Golden tests** pin the fingerprint of every fixture; a comment states that any change needs a `normal_form` bump |
| B13 | body | 21 | checkbox | - [ ] Fixtures prove that renamed locals, reference/adapter noise and different literal spellings collapse, while different called items do not |
| B14 | body | 22 | checkbox | - [ ] Running twice gives identical bytes |
| B15 | body | 23 | checkbox | - [ ] `cap:census-normal-form` reconciled with the result |

## Comments that matched no rule

None.
