# Branch under review — issue #3

Branch `paudley/3-raf-census-core-normal-form-v1` against `origin/main`. Gathered by `stagectl brief
--mode review`; every hash and path came from this repository.

## Commits (12)

- `be93118c2` docs(census): correct the normal form page and capability against the code  _2026-10-20_
- `ab6279542` fix(raf-census): keep the names inside an attribute, never rename them as locals  _2026-10-20_
- `7a49256eb` docs(census): normal form 1 as delivered, census-normal-form complete  _2026-10-20_
- `2ec6a1e2a` fix(raf-census): require the index body and shim keys, read an absolute directory from its parent  _2026-10-20_
- `a8452e586` feat(raf-census): baseline and index reports, the baseline example, byte-determinism  _2026-10-20_
- `e02a4443b` fix(raf-census): match shim ids by the shim key, group by the same field  _2026-10-20_
- `e9ea57e20` feat(raf-census): grouping, hex tables, shared constants and job matching  _2026-10-20_
- `e1073f94d` fix(raf-census): recognise turbofish calls, fail hard on bad spans, collect all string literals  _2026-10-20_
- `05d7be1c6` feat(raf-census): units by source identity and Census::load  _2026-10-20_
- `3f8f46968` fix(raf-census): number locals after every parameter a body uses, placeholders included  _2026-10-20_
- `08071b896` feat(raf-census): normal form 1 as a tower of passes, its fingerprints and pinned goldens  _2026-10-20_
- `cb08b0c6e` feat(raf-census): the crate, its options, errors and normal-form version gate  _2026-10-20_

## Files changed

```
AGENTS.md                                          |  12 +-
 CHANGELOG.md                                       |  12 +
 Cargo.lock                                         |  75 ++
 Cargo.toml                                         |   2 +-
 README.md                                          |  10 +-
 crates/raf-census/Cargo.toml                       |  32 +
 crates/raf-census/examples/baseline.rs             |  32 +
 crates/raf-census/examples/goldens.rs              |  18 +
 crates/raf-census/src/census.rs                    | 138 ++++
 crates/raf-census/src/error.rs                     | 127 ++++
 crates/raf-census/src/group.rs                     | 338 +++++++++
 crates/raf-census/src/job.rs                       | 188 +++++
 crates/raf-census/src/lib.rs                       |  71 ++
 crates/raf-census/src/normalize/fingerprint.rs     | 207 ++++++
 crates/raf-census/src/normalize/form.rs            | 271 ++++++++
 crates/raf-census/src/normalize/literal.rs         | 139 ++++
 crates/raf-census/src/normalize/mod.rs             | 340 +++++++++
 crates/raf-census/src/normalize/passes.rs          | 627 +++++++++++++++++
 crates/raf-census/src/normalize/shim.rs            | 109 +++
 crates/raf-census/src/normalize/words.rs           |  47 ++
 crates/raf-census/src/options.rs                   |  92 +++
 crates/raf-census/src/report.rs                    | 355 ++++++++++
 crates/raf-census/src/units/mod.rs                 | 291 ++++++++
 crates/raf-census/src/units/scan.rs                | 177 +++++
 crates/raf-census/src/units/sig.rs                 | 190 +++++
 crates/raf-census/src/version.rs                   |  57 ++
 crates/raf-census/tests/forms.rs                   | 519 ++++++++++++++
 crates/raf-census/tests/golden.rs                  | 305 +++++++++
 crates/raf-census/tests/groups.rs                  | 550 +++++++++++++++
 crates/raf-census/tests/page.rs                    | 186 +++++
 crates/raf-census/tests/reports.rs                 | 480 +++++++++++++
 crates/raf-census/tests/support/builtin.rs         |  85 +++
 crates/raf-census/tests/support/fixtures.rs        | 762 +++++++++++++++++++++
 crates/raf-census/tests/support/mod.rs             |  11 +
 crates/raf-census/tests/units.rs                   | 693 +++++++++++++++++++
 docs/architecture/normal-form.md                   | 291 ++++++--
 docs/capabilities/census-normal-form.md            | 706 +++++++++++++++++--
 docs/capabilities/index.md                         |   2 +-
 tests/docs_contract.rs                             |  17 +-
 work/assessments/census-normal-form-constraints.md | 325 +++++++++
 work/assessments/census-normal-form-contract.md    |  82 +++
 work/assessments/census-normal-form-examples.md    | 325 +++++++++
 work/assessments/census-normal-form-purpose.md     | 325 +++++++++
 work/assessments/census-normal-form-usage.md       |  40 ++
 .../assessments/census-normal-form-verification.md | 325 +++++++++
 work/changes/census-ledger.md                      |   1 -
 work/changes/census-normal-form.md                 |  28 -
 47 files changed, 9862 insertions(+), 153 deletions(-)
```

## ADRs the changed files cite

None of the changed files cite an ADR.

These are the settled decisions this branch touches. The question
for stage2 is whether the change fits them, not merely whether it
compiles.

