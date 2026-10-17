# Branch under review — issue #9

Branch `paudley/9-acceptance-scenarios-a-b-workplan-self` against `origin/main`. Gathered by `stagectl brief
--mode review`; every hash and path came from this repository.

## Commits (2)

- `1da058c79` test: run the workplan gate on this repository from its own test binary  _2026-10-17_
- `202392621` docs(capabilities): every delivered capability holds a generated verification matrix  _2026-10-17_

## Files changed

```
.githooks/pre-commit                               |  11 +-
 AGENTS.md                                          |  10 +-
 CONTRIBUTING.md                                    |   4 +-
 Makefile                                           |   3 +-
 docs/capabilities/okf-profile.md                   |  97 +++++++
 docs/capabilities/source-index.md                  | 299 +++++++++++++++++++++
 docs/capabilities/workplan-current-rationale.md    |  22 ++
 docs/capabilities/workplan-entity-history.md       |  40 +++
 docs/capabilities/workplan-evidence-claims.md      |  43 +++
 docs/capabilities/workplan-prose-assessment.md     |  35 +++
 docs/capabilities/workplan-two-way-coverage.md     |  30 +++
 tests/docs_contract.rs                             | 141 ++++++++--
 tests/self_workplan.rs                             |  65 +++++
 work/assessments/okf-profile-verification.md       |   2 +-
 work/assessments/source-index-verification.md      |   2 +-
 .../workplan-current-rationale-verification.md     |   2 +-
 .../workplan-entity-history-verification.md        |   2 +-
 .../workplan-evidence-claims-verification.md       |   2 +-
 .../workplan-prose-assessment-verification.md      |   2 +-
 .../workplan-two-way-coverage-verification.md      |   2 +-
 20 files changed, 777 insertions(+), 37 deletions(-)
```

## ADRs the changed files cite

None of the changed files cite an ADR.

These are the settled decisions this branch touches. The question
for stage2 is whether the change fits them, not merely whether it
compiles.

