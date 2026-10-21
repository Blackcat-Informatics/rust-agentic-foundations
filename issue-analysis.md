# Requirements ledger: issue #3: raf-census core: normal form v1, fingerprints, grouping, reports

Evidence: issue.md, requirements-candidates.md, brief.json (coverage: issue #3 + 0 comment(s) via github; link crawl: 2 issue(s) within 2 hop(s) of `#N` references; forge search on ['raf-census', 'core:', 'normal']: 2 other item(s); GhpRsq-Defect-Class trailers: 1 distinct class(es) in the last 200 commits; prior attempts: 0 commit(s) citing #3, 0 other branch(es) named for it; PRs citing #3: 0; requirement candidates: 15 line(s); 0 comment(s) matched no rule; referenced artifacts: 2 path(s) (2 on disk), 13 identifier(s), 0 workspace package(s), 0 URL(s))

Line convention: "body L<n>" is the candidates' numbering, where body L1 is issue.md line 8. issue.md line = n + 7.

## Requirements
| R-id | Kind | Source | Quote (verbatim) | From candidate |
|------|------|--------|------------------|----------------|
| R1 | checkbox | body L7 | "- [ ] `normalize`: `proc_macro2` token streams with bodies re-lexed, so macro arguments are normalized too" | B1 |
| R2 | list-item | body L8 | "- **structural form**: keywords, punctuation and delimiters kept; names of items the body uses kept; locals, parameters and lifetimes become `$0..` by first appearance; literals become their kind" | B2 |
| R3 | list-item | body L9 | "- **value form**: drops `&`, `&mut`, `*`, `ref`, the value adapters (`.clone()`, `.as_str()`, `.to_owned()`…), full-range reindex, `let` type annotations and type-only turbofishes" | B3 |
| R4 | list-item | body L10 | "- **literal tier**: bodies under 30 tokens keep their literals verbatim" | B4 |
| R5 | list-item | body L11 | "- **shim form**: single-expression forwarders (6–30 tokens), with only the function's own parameters renamed; katamari's `shim_key` refinement excludes trait-impl methods and `Self::` forwarders" | B5 |
| R6 | checkbox | body L12 | "- [ ] Fingerprints: `<kind>:` + hex of the first 8 bytes of BLAKE3 (`blake3` crate); kinds are `body`, `shim`, `table`, `rule`" | B6 |
| R7 | checkbox | body L13 | "- [ ] Units: fn, method, const, static, with symbol, package, file, line, shipping flag, print, integer constants (compared by value regardless of spelling), tables and strings" | B7 |
| R8 | checkbox | body L14 | "- [ ] Grouping: `BTreeMap<fingerprint, members>`, groups of two or more, deterministic ordering (member count, then tokens, then fingerprint)" | B8 |
| R9 | checkbox | body L15 | "- [ ] Hex-table detection (`table:hex-lower` / `table:hex-upper` across string, byte-string and array forms); shared large constants (>0xffff in two or more packages)" | B9 |
| R10 | checkbox | body L16 | "- [ ] Reports (`serde_json`): baseline JSON (`min_tokens`, units, body/shim groups, hex tables, shared constants) and index JSON (symbols, groups, job matches); byte-deterministic output" | B10 |
| R11 | checkbox | body L17 | "- [ ] `docs/architecture/normal-form.md` updated: the normative description of v1" | B11 |
| R12 | checkbox | body L20 | "- [ ] **Golden tests** pin the fingerprint of every fixture; a comment states that any change needs a `normal_form` bump" | B12 |
| R13 | checkbox | body L21 | "- [ ] Fixtures prove that renamed locals, reference/adapter noise and different literal spellings collapse, while different called items do not" | B13 |
| R14 | checkbox | body L22 | "- [ ] Running twice gives identical bytes" | B14 |
| R15 | checkbox | body L23 | "- [ ] `cap:census-normal-form` reconciled with the result" | B15 |
| R16 | decision | body L2 (issue.md 9, Context) | "this issue defines a **fresh, versioned normal form v1**" | prose |
| R17 | prose | body L4 (issue.md 11) | "Capability delivered: `cap:census-normal-form`." | prose |
| R18 | dependency | body L26 (issue.md 33, Depends on) | "- #2" | prose |

Kinds: checkbox, modal, list-item, scope-out, dependency, question, decision, prose.

## Candidate dispositions
| Candidate | Disposition |
|-----------|-------------|
| B1 | kept → R1 |
| B2 | kept → R2 |
| B3 | kept → R3 |
| B4 | kept → R4 |
| B5 | kept → R5 |
| B6 | kept → R6 |
| B7 | kept → R7 |
| B8 | kept → R8 |
| B9 | kept → R9 |
| B10 | kept → R10 |
| B11 | kept → R11 |
| B12 | kept → R12 |
| B13 | kept → R13 |
| B14 | kept → R14 |
| B15 | kept → R15 |

No candidate dropped. B14 ("Running twice gives identical bytes") overlaps the "byte-deterministic output" clause of B10 but is a separate line with its own checkbox, so it is kept, not dropped as a duplicate.

Prose not turned into a row (background, no requirement): issue.md line 9 "Today there are seven diverged copies of purrdf's `crates/helper-census`..." and "They use different hashes (BLAKE3 vs SHA-256), thresholds (20 vs 30)...". Headings (`## Scope`, `## Acceptance criteria`) are structure only.

Prose additions: 3 (R16, R17, R18).

## Comment coverage
The issue has 0 comments (`## Comments (0)`; brief coverage "0 comment(s)"; silent comments list empty). Nothing to cover: comments covered 0/0.

| Comment | Author | Rows / disposition |
|---------|--------|--------------------|
| (none) | | N=0 |

## Dependencies (from brief.json relations)
- #2 mentions [CLOSED]: "Shared crates: raf-okf + raf-source"
- #1 2-hop [CLOSED]: "Scaffold workspace, project docs and branding, plus spec-first capability docs"

## NEEDS-JUDGMENT
1. R18 vs brief relation: "- #2" (issue body "Depends on") / brief.json `link_relations` "2": "mentions": the issue declares a dependency on #2 but the link is recorded as a mention. Which relation is authoritative is not decided here.
2. R16 vs brief ADRs: "Per the decision, this issue defines a **fresh, versioned normal form v1**" / brief.json `adrs: []`: the "decision" the issue cites is not among the linked records. Which decision record (if any) binds this is not established by the inputs.
3. R4 vs R5: "**literal tier**: bodies under 30 tokens keep their literals verbatim" / "**shim form**: single-expression forwarders (6–30 tokens)": both apply to small bodies (under 30 tokens); whether a single-expression forwarder in that range takes the literal tier or the shim form is not stated.
4. R2 vs R3: "punctuation and delimiters kept" / "drops `&`, `&mut`, `*`, `ref`": the structural form keeps punctuation while the value form drops `&` and `*`, which are punctuation. Which form each fingerprint uses is not stated for every kind (`body`, `shim`, `table`, `rule`).
5. R6 vs R13: "kinds are `body`, `shim`, `table`, `rule`" / "Fixtures prove that ... different called items do not": the checkbox in R13 requires fixtures to show non-collapse, but no fixture set is named. Not a conflict, noted for the planner.

## Self-check
Quotes checked: 18. Misses: 0. Comments covered: 0/0.
(Quotes checked by grep -cF against issue.md; every quote returned at least 1 hit.)
