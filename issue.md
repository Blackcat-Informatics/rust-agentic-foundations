# Issue #3: raf-census core: normal form v1, fingerprints, grouping, reports

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: enhancement, crate:raf-census

## Body

## Context
This is the heart of the unified helper-census. Today there are seven diverged copies of purrdf's `crates/helper-census` (purrdf, gmeow-ontology branch, lillith_memetics, purrassist, lillith_decodes, katamari, sparrow). They use different hashes (BLAKE3 vs SHA-256), thresholds (20 vs 30) and normalizer generations. Per the decision, this issue defines a **fresh, versioned normal form v1**.

Capability delivered: `cap:census-normal-form`.

## Scope
- [ ] `normalize`: `proc_macro2` token streams with bodies re-lexed, so macro arguments are normalized too
  - **structural form**: keywords, punctuation and delimiters kept; names of items the body uses kept; locals, parameters and lifetimes become `$0..` by first appearance; literals become their kind
  - **value form**: drops `&`, `&mut`, `*`, `ref`, the value adapters (`.clone()`, `.as_str()`, `.to_owned()`…), full-range reindex, `let` type annotations and type-only turbofishes
  - **literal tier**: bodies under 30 tokens keep their literals verbatim
  - **shim form**: single-expression forwarders (6–30 tokens), with only the function's own parameters renamed; katamari's `shim_key` refinement excludes trait-impl methods and `Self::` forwarders
- [ ] Fingerprints: `<kind>:` + hex of the first 8 bytes of BLAKE3 (`blake3` crate); kinds are `body`, `shim`, `table`, `rule`
- [ ] Units: fn, method, const, static, with symbol, package, file, line, shipping flag, print, integer constants (compared by value regardless of spelling), tables and strings
- [ ] Grouping: `BTreeMap<fingerprint, members>`, groups of two or more, deterministic ordering (member count, then tokens, then fingerprint)
- [ ] Hex-table detection (`table:hex-lower` / `table:hex-upper` across string, byte-string and array forms); shared large constants (>0xffff in two or more packages)
- [ ] Reports (`serde_json`): baseline JSON (`min_tokens`, units, body/shim groups, hex tables, shared constants) and index JSON (symbols, groups, job matches); byte-deterministic output
- [ ] `docs/architecture/normal-form.md` updated: the normative description of v1

## Acceptance criteria
- [ ] **Golden tests** pin the fingerprint of every fixture; a comment states that any change needs a `normal_form` bump
- [ ] Fixtures prove that renamed locals, reference/adapter noise and different literal spellings collapse, while different called items do not
- [ ] Running twice gives identical bytes
- [ ] `cap:census-normal-form` reconciled with the result

## Depends on
- #2

## Comments (0)

