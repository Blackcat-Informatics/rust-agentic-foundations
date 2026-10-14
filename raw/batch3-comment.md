**Final remediation batch: R13, R5c, R11a, R5d, R11b** (all pushed)

The owner's scope decision, recorded verbatim in the stage validation index: "Everythign - you have 6 hours - capped". Every remaining item stays in this PR.

**R13: licence exceptions pinned to the audited versions** (d145545)
- `unicode-ident@=1.0.27` and `libfuzzer-sys@=0.4.13`.
- A deliberately wrong pin fails `make deny`, which proves the pin binds.

**R5c: glob scope tables have a byte budget** (be71d40)
- **Design:** shared per-visibility tables, built once; only the watching globs are woken; a 512 MiB glob-bytes budget, charged before allocating, with a typed refusal over it.
- **web-sys** now **loads** in about 5 s at 144 MiB, where it used to run out of the 8 GiB cap.
- **166 KB hostile file:** this was 7.3 GiB before. It is now refused, typed, at about 414 MiB.
- **Found in review and fixed:** repeated identical globs allocated before charging. They are now deduplicated and charged first, so 40,000×1,500 loads in 0.5 s, held ≤ charged.
- **Checks:** registry outcomes unchanged (1,204 OK, 0 budget refusals). Resolve dumps of this repository, katamari and purrdf are byte-identical.

**R11a: `items_under` matches whole segments; doc-contract checks binding roles** (df71a42)
- `items_under("demo::a")` no longer returns `demo::ab` or `demo::a1`.
- A `tests` binding must name a `#[test]` function, and a `code` binding must name a shipping item.

**R5d: resolve-path validation** (d8aba1e)
- `crate`, `self`, `super` or `Self` mid-path is now `Invalid`.
- The swallowed `SuperAboveRoot` arm is removed.
- A glob of a missing local module is the typed `UnresolvedGlob`, and the load still succeeds.
- The error order is fixed and documented.
- A 50,000-segment path takes milliseconds in release, against seconds before.
- `ResolveError` text is escaped.
- **Pre-existing misreport, found and fixed:** two dependency globs in one module reported a missing name as `Cyclic`. It is now `NotFound`, and real glob cycles stay `Cyclic`.

**R11b: public API sealed** (bf08e45, `feat(raf-source)!`)
- `#[non_exhaustive]` on the types consumers bind to, and the per-file facts are crate-private.
- `Target` gains `harness`, `required_features`, `test` and `doctest`, with Cargo's defaults. A library `required-features` is refused, as Cargo refuses it.
- `SpanFact` line ranges are half-open.
- `LoadOptions` is no longer `Copy`.
- `Outcome` and `LinkTarget` are exhaustive.

**For every item:** an independent review is under `tasks/`, and every one is PASS. Fixtures fail on the old code or carry a recorded mutation, and each commit passed its gates and hooks.

**Hosted CI:** fully green on 44e18b7 (all 12 jobs, including native macOS and Windows). The run on bf08e45 is in progress.

**Running now:** the final `make check` and a fresh completion audit. Then Stage 3.
