# Branch under review — issue #10

Branch `paudley/10-m2-rest-of-the-planning-essay-entity` against `origin/main`. Gathered by `stagectl brief
--mode review`; every hash and path came from this repository.

## Commits (42)

- `43c7e020c` test(cargo-workplan): keep git maintenance and vanished files out of the tree snapshot  _2026-10-16_
- `8e5f568d1` docs: record the assessments and evidence that the history change made stale  _2026-10-16_
- `09a2c8dca` docs(capabilities): deliver the workplan-entity-history capability again  _2026-10-16_
- `b4ca6c87a` docs(workplan-entity-history): assess the sections again for module history and the author-date filter  _2026-10-16_
- `bc3f15f8c` fix(raf-workplan): history of a module binding covers its files and --since filters by author date  _2026-10-16_
- `a51977f37` test(raf-workplan): require the delivered capabilities in the roadmap guard  _2026-10-16_
- `1c2f6a68d` docs: state the workplan commands and capabilities as delivered  _2026-10-16_
- `023df72f9` docs(capabilities): deliver the workplan-current-rationale capability  _2026-10-16_
- `68a3342ca` docs(capabilities): deliver the workplan-two-way-coverage capability  _2026-10-16_
- `7b8ae50eb` docs(capabilities): deliver the workplan-prose-assessment capability  _2026-10-16_
- `fcd69d1d9` docs(capabilities): deliver the workplan-evidence-claims capability  _2026-10-16_
- `d561881e5` docs(capabilities): deliver the entity-history capability  _2026-10-16_
- `11ce02d06` docs(capabilities): deliver the workplan-cli capability  _2026-10-16_
- `ba34ce9c5` docs(capabilities): deliver the workplan-complete-gate capability  _2026-10-16_
- `e22ac7dfd` docs(capabilities): deliver the workplan-reconcile capability  _2026-10-16_
- `0b88c6601` docs(capabilities): deliver the workplan-generated-docs capability  _2026-10-16_
- `6a132df12` docs(capabilities): deliver the workplan-document-model capability  _2026-10-16_
- `a8bd498b3` docs(capabilities): deliver the source-index capability  _2026-10-16_
- `ef6204947` docs(capabilities): deliver the okf-profile capability  _2026-10-16_
- `f88d1dd2d` docs: fix statements found false when assessing the workplan capabilities  _2026-10-16_
- `d6f88048e` docs(workplan-current-rationale): state decision pages, conditions to revisit and their checks as the current contract  _2026-10-16_
- `6a9c116b8` docs(workplan-two-way-coverage): state the public surface, the coverage record and what check enforces as the current contract  _2026-10-16_
- `180ec0027` docs(workplan-prose-assessment): state assessment records, their inputs and the gate as the current contract  _2026-10-16_
- `5536582e0` docs(workplan-evidence-claims): state evidence records, claim markers and their invalidation as the current contract  _2026-10-16_
- `74783a059` docs(workplan-entity-history): state the history command, its git rows and their limits as the current contract  _2026-10-16_
- `1a5b28e7e` docs(workplan-cli): state the history, evidence, assess and coverage commands as the current contract  _2026-10-16_
- `511d27a22` docs(workplan-complete-gate): state the sixth completion condition and its order as the current contract  _2026-10-16_
- `cfec99fee` docs(workplan-reconcile): state the trace and assessment obligations as the current contract  _2026-10-16_
- `928ca4dba` docs(workplan-generated-docs): describe every bound test of the generated blocks, sync and preview  _2026-10-16_
- `e76c4cfcf` docs(workplan-document-model): state the records, digests and record writer as the current contract  _2026-10-16_
- `dbc26f719` docs(source-index): state the exported items of a target as the current contract  _2026-10-16_
- `3bd9e56d6` docs(okf-profile): state the record types, claim markers and record writer as the current contract  _2026-10-16_
- `21499bcf6` docs: cite recorded evidence for the time limit of a bound test and assess a rationale section  _2026-10-16_
- `6f7afe143` feat(cargo-workplan): history, evidence, assess and coverage commands  _2026-10-16_
- `c54db19d2` feat(raf-workplan): check current rationale pages and decisions  _2026-10-16_
- `6b317fc6b` feat(raf-workplan): two-way coverage of public items and tests  _2026-10-16_
- `bb415d37c` feat(raf-workplan): prose assessments, their obligations and a sixth completion condition  _2026-10-16_
- `9bacdb380` feat(raf-workplan): evidence records, claims and their obligations  _2026-10-16_
- `73662ab82` feat(raf-workplan): history of a capability, change or architecture page from git  _2026-10-16_
- `de0b842d2` feat(raf-workplan): load evidence, assessment and coverage records with a shared digest  _2026-10-15_
- `8844239ea` feat(raf-source): list the exported items of a target  _2026-10-15_
- `6e75ac715` feat(raf-okf): evidence, assessment and coverage records and claim markers  _2026-10-15_

## Files changed

```
AGENTS.md                                          |   11 +-
 CHANGELOG.md                                       |  230 +++
 README.md                                          |   11 +-
 SECURITY.md                                        |   81 +-
 crates/cargo-workplan/Cargo.toml                   |    2 +-
 crates/cargo-workplan/src/assess.rs                |  209 +++
 crates/cargo-workplan/src/cli.rs                   |  750 ++++++++-
 crates/cargo-workplan/src/command.rs               |   44 +-
 crates/cargo-workplan/src/coverage.rs              |   59 +
 crates/cargo-workplan/src/evidence.rs              |  320 ++++
 crates/cargo-workplan/src/history.rs               |   34 +
 crates/cargo-workplan/src/main.rs                  |  130 +-
 crates/cargo-workplan/src/render.rs                |   81 +-
 crates/cargo-workplan/tests/assess.rs              |  263 +++
 crates/cargo-workplan/tests/complete.rs            |   43 +-
 crates/cargo-workplan/tests/coverage.rs            |  227 +++
 crates/cargo-workplan/tests/evidence.rs            |  675 ++++++++
 crates/cargo-workplan/tests/history.rs             |  283 ++++
 crates/cargo-workplan/tests/json.rs                |   11 +
 crates/cargo-workplan/tests/process.rs             |  279 +++-
 crates/cargo-workplan/tests/reconcile.rs           |    9 +-
 crates/cargo-workplan/tests/support/mod.rs         |   43 +-
 crates/raf-okf/src/body/blocks.rs                  |    6 +
 crates/raf-okf/src/body/marker.rs                  |  207 ++-
 crates/raf-okf/src/body/mod.rs                     |   19 +-
 crates/raf-okf/src/error.rs                        |   54 +-
 crates/raf-okf/src/lib.rs                          |   54 +-
 crates/raf-okf/src/profile/body_rules.rs           |   16 +-
 crates/raf-okf/src/profile/mod.rs                  |  202 ++-
 crates/raf-okf/src/profile/records.rs              |  626 ++++++++
 crates/raf-okf/src/profile/schema.rs               |  159 +-
 crates/raf-okf/src/profile/typed.rs                |  533 +++++++
 crates/raf-okf/src/profile/violation.rs            |  500 +++++-
 crates/raf-okf/src/profile/write.rs                |  213 +++
 crates/raf-okf/tests/claims.rs                     |  306 ++++
 crates/raf-okf/tests/fixtures/assessment.md        |   17 +
 crates/raf-okf/tests/fixtures/coverage.md          |   12 +
 crates/raf-okf/tests/fixtures/evidence.md          |   26 +
 crates/raf-okf/tests/profile.rs                    |  104 +-
 crates/raf-okf/tests/record_writer.rs              |  299 ++++
 crates/raf-okf/tests/records.rs                    |  931 +++++++++++
 crates/raf-okf/tests/render.rs                     |   86 +-
 crates/raf-okf/tests/schema.rs                     |  128 +-
 crates/raf-okf/tests/validate_cost.rs              |   46 +
 crates/raf-source/src/exports.rs                   | 1537 ++++++++++++++++++
 crates/raf-source/src/facts.rs                     |   36 +-
 crates/raf-source/src/glob_scope.rs                |   32 +
 crates/raf-source/src/index.rs                     |   16 +
 crates/raf-source/src/item.rs                      |    4 +
 crates/raf-source/src/lib.rs                       |   52 +
 crates/raf-source/src/resolve.rs                   |   49 +-
 crates/raf-source/src/workspace.rs                 |  101 ++
 crates/raf-source/tests/exported_items.rs          | 1603 +++++++++++++++++++
 crates/raf-source/tests/this_repository.rs         |   71 +-
 crates/raf-workplan/src/assessment.rs              | 1112 +++++++++++++
 crates/raf-workplan/src/assessment/tests.rs        |  318 ++++
 crates/raf-workplan/src/coverage.rs                | 1061 +++++++++++++
 crates/raf-workplan/src/coverage/render.rs         |  163 ++
 crates/raf-workplan/src/digest.rs                  |  678 ++++++++
 crates/raf-workplan/src/evidence.rs                |  871 ++++++++++
 crates/raf-workplan/src/evidence/measure.rs        |  328 ++++
 crates/raf-workplan/src/evidence/tests.rs          |  819 ++++++++++
 crates/raf-workplan/src/exec.rs                    |   32 +-
 crates/raf-workplan/src/exec/fake.rs               |    6 +-
 crates/raf-workplan/src/exec/invocation.rs         |  139 +-
 crates/raf-workplan/src/exec/pattern.rs            |  167 +-
 crates/raf-workplan/src/exec/system.rs             |   20 +-
 crates/raf-workplan/src/exec/tests.rs              |  170 +-
 crates/raf-workplan/src/exec/trusted.rs            |  283 ++++
 crates/raf-workplan/src/files.rs                   |  100 ++
 crates/raf-workplan/src/finding.rs                 |  183 ++-
 crates/raf-workplan/src/gate.rs                    |   57 +-
 crates/raf-workplan/src/generated.rs               |    2 +-
 crates/raf-workplan/src/git.rs                     |   27 +
 crates/raf-workplan/src/git/log.rs                 |  729 +++++++++
 crates/raf-workplan/src/graph.rs                   |  845 +++++++++-
 crates/raf-workplan/src/history.rs                 |  836 ++++++++++
 crates/raf-workplan/src/history/render.rs          |  263 +++
 crates/raf-workplan/src/lib.rs                     |  141 +-
 crates/raf-workplan/src/lifecycle.rs               |   18 +-
 crates/raf-workplan/src/lifecycle/changed.rs       |  276 ++++
 crates/raf-workplan/src/lifecycle/complete.rs      |   53 +-
 crates/raf-workplan/src/lifecycle/completion.rs    |  112 +-
 crates/raf-workplan/src/lifecycle/create.rs        |  134 +-
 crates/raf-workplan/src/lifecycle/session.rs       |   61 +-
 crates/raf-workplan/src/lifecycle/untouched.rs     |   63 +-
 crates/raf-workplan/src/load.rs                    |   81 +-
 crates/raf-workplan/src/model.rs                   |   92 +-
 crates/raf-workplan/src/obligation.rs              |  234 ++-
 crates/raf-workplan/src/obligation/sources.rs      |    2 +
 crates/raf-workplan/src/obligation/tests.rs        |  135 +-
 crates/raf-workplan/src/obligation/trace.rs        |  250 +++
 crates/raf-workplan/src/plain.rs                   |   15 +-
 crates/raf-workplan/src/rationale.rs               |  360 +++++
 crates/raf-workplan/src/record.rs                  |  362 +++++
 crates/raf-workplan/src/spdx.rs                    |   79 +
 crates/raf-workplan/src/sync.rs                    |    2 +-
 crates/raf-workplan/tests/assessments.rs           |  898 +++++++++++
 crates/raf-workplan/tests/complete.rs              |  243 ++-
 crates/raf-workplan/tests/coverage.rs              |  935 +++++++++++
 crates/raf-workplan/tests/digest.rs                |  325 ++++
 crates/raf-workplan/tests/documents.rs             |   11 +-
 crates/raf-workplan/tests/evidence.rs              |  794 ++++++++++
 crates/raf-workplan/tests/exec.rs                  |  204 ++-
 crates/raf-workplan/tests/gate.rs                  |    4 +-
 crates/raf-workplan/tests/generated.rs             |   13 +-
 crates/raf-workplan/tests/graph.rs                 |   40 +-
 crates/raf-workplan/tests/history.rs               | 1673 ++++++++++++++++++++
 crates/raf-workplan/tests/kinds.rs                 |   18 +-
 crates/raf-workplan/tests/lifecycle.rs             |    4 +-
 crates/raf-workplan/tests/links.rs                 |   13 +-
 crates/raf-workplan/tests/obligations.rs           |  319 +++-
 crates/raf-workplan/tests/plain.rs                 |   23 +
 crates/raf-workplan/tests/rationale.rs             |  608 +++++++
 crates/raf-workplan/tests/records.rs               | 1115 +++++++++++++
 crates/raf-workplan/tests/roadmap.rs               |   53 +-
 crates/raf-workplan/tests/scenarios.rs             |  147 +-
 crates/raf-workplan/tests/support/assessed.rs      |   82 +
 crates/raf-workplan/tests/support/cases.rs         |  466 +++++-
 crates/raf-workplan/tests/support/changes.rs       |    2 +-
 crates/raf-workplan/tests/support/evidence.rs      |  131 ++
 crates/raf-workplan/tests/support/mod.rs           |   52 +-
 crates/raf-workplan/tests/support/project.rs       |    5 +
 crates/raf-workplan/tests/support/synced.rs        |   34 +-
 crates/raf-workplan/tests/untouched.rs             |   14 +-
 crates/raf-workplan/tests/verify.rs                |    3 +
 docs/architecture/normal-form.md                   |   25 +
 docs/architecture/okf-profile.md                   |  123 +-
 docs/capabilities/index.md                         |   17 +-
 docs/capabilities/okf-profile.md                   |  148 +-
 docs/capabilities/source-index.md                  |  403 ++++-
 docs/capabilities/workplan-cli.md                  |  488 +++++-
 docs/capabilities/workplan-complete-gate.md        |  373 ++++-
 docs/capabilities/workplan-current-rationale.md    |  201 ++-
 docs/capabilities/workplan-document-model.md       |  505 +++++-
 docs/capabilities/workplan-entity-history.md       |  395 +++--
 docs/capabilities/workplan-evidence-claims.md      |  283 +++-
 docs/capabilities/workplan-generated-docs.md       |  113 +-
 docs/capabilities/workplan-prose-assessment.md     |  264 ++-
 docs/capabilities/workplan-reconcile.md            |  379 ++++-
 docs/capabilities/workplan-two-way-coverage.md     |  270 +++-
 fuzz/fuzz_targets/workspace_input.rs               |   13 +-
 tests/docs_contract.rs                             |  241 ++-
 work/assessments/okf-profile-contract.md           |   58 +
 work/assessments/okf-profile-verification.md       |   25 +
 work/assessments/source-index-constraints.md       |   94 ++
 work/assessments/source-index-contract.md          |   79 +
 work/assessments/source-index-examples.md          |   94 ++
 work/assessments/source-index-usage.md             |   94 ++
 work/assessments/source-index-verification.md      |   43 +
 work/assessments/workplan-cli-constraints.md       |   58 +
 work/assessments/workplan-cli-contract.md          |   58 +
 work/assessments/workplan-cli-examples.md          |   58 +
 work/assessments/workplan-cli-usage.md             |   58 +
 work/assessments/workplan-cli-verification.md      |   58 +
 .../assessments/workplan-complete-gate-contract.md |   64 +
 .../assessments/workplan-complete-gate-examples.md |   61 +
 .../workplan-complete-gate-verification.md         |   61 +
 .../workplan-current-rationale-constraints.md      |   16 +
 .../workplan-current-rationale-contract.md         |   16 +
 .../workplan-current-rationale-examples.md         |   16 +
 .../workplan-current-rationale-explanation.md      |   16 +
 .../workplan-current-rationale-purpose.md          |   16 +
 .../workplan-current-rationale-usage.md            |   16 +
 .../workplan-current-rationale-verification.md     |   16 +
 .../workplan-document-model-contract.md            |   97 ++
 work/assessments/workplan-document-model-usage.md  |   28 +
 .../workplan-document-model-verification.md        |   34 +
 .../workplan-entity-history-constraints.md         |   40 +
 .../workplan-entity-history-contract.md            |   40 +
 .../workplan-entity-history-examples.md            |   40 +
 .../assessments/workplan-entity-history-purpose.md |   40 +
 work/assessments/workplan-entity-history-usage.md  |   40 +
 .../workplan-entity-history-verification.md        |   19 +
 .../workplan-evidence-claims-constraints.md        |   34 +
 .../workplan-evidence-claims-contract.md           |   34 +
 .../workplan-evidence-claims-examples.md           |   34 +
 .../workplan-evidence-claims-explanation.md        |   34 +
 .../workplan-evidence-claims-purpose.md            |   34 +
 work/assessments/workplan-evidence-claims-usage.md |   34 +
 .../workplan-evidence-claims-verification.md       |   16 +
 .../workplan-generated-docs-verification.md        |   40 +
 .../workplan-prose-assessment-constraints.md       |   28 +
 .../workplan-prose-assessment-contract.md          |   28 +
 .../workplan-prose-assessment-examples.md          |   28 +
 .../workplan-prose-assessment-purpose.md           |   28 +
 .../assessments/workplan-prose-assessment-usage.md |   25 +
 .../workplan-prose-assessment-verification.md      |   16 +
 work/assessments/workplan-reconcile-constraints.md |   97 ++
 work/assessments/workplan-reconcile-contract.md    |   97 ++
 .../assessments/workplan-reconcile-verification.md |   97 ++
 .../workplan-two-way-coverage-constraints.md       |   19 +
 .../workplan-two-way-coverage-contract.md          |   19 +
 .../workplan-two-way-coverage-examples.md          |   19 +
 .../workplan-two-way-coverage-purpose.md           |   19 +
 .../assessments/workplan-two-way-coverage-usage.md |   19 +
 .../workplan-two-way-coverage-verification.md      |   19 +
 work/changes/workplan-current-rationale.md         |   26 -
 work/changes/workplan-entity-history.md            |   28 -
 work/changes/workplan-evidence-claims.md           |   26 -
 work/changes/workplan-prose-assessment.md          |   26 -
 work/changes/workplan-two-way-coverage.md          |   26 -
 work/coverage/default.md                           |    8 +
 .../evidence/gate-kills-a-test-past-its-timeout.md |   59 +
 204 files changed, 37194 insertions(+), 1265 deletions(-)
```

## ADRs the changed files cite

None of the changed files cite an ADR.

These are the settled decisions this branch touches. The question
for stage2 is whether the change fits them, not merely whether it
compiles.

