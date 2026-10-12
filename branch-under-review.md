# Branch under review — issue 1 / PR 11

Generated manually: `stagectl brief --mode review --base origin/main` crashes (UnicodeDecodeError in resolve.git → `git show HEAD:docs/social-preview.png`, binary blob decoded as UTF-8). Automation gap recorded in validation.md.

- Branch: paudley/1-scaffold-workspace-project-docs-and
- Base: origin/main @ 720262d
- Head: d6b6572 (PR 11 headRefOid d6b6572)
- Merge base: 720262d

## Commits
```
d6b6572 docs(capabilities): use a neutral example package in the reference scan spec
9f93e5d docs(capabilities): add considered workplan capabilities and retire the essay
b499578 test: enforce the OKF document profile in cargo test
9ce066f docs(capabilities): add workplan capability specs
25e1978 docs(capabilities): add census capability specs and architecture pages
49eb42b docs(brand): add RAF brand, logo, social preview and README
c7edb76 docs: add agent guide, contributing, conduct, security and GitHub templates
b7db728 ci: add make check, pre-commit hook and CI workflow
fc6f531 docs(license): add licensing texts and REUSE metadata
a4eacdc build: scaffold RAF workspace
```

## Diffstat
```
 .deficiencies                                   |   20 ++
 .githooks/pre-commit                            |   38 +++
 .github/ISSUE_TEMPLATE/bug_report.yml           |   64 ++++++
 .github/ISSUE_TEMPLATE/config.yml               |    7 +
 .github/ISSUE_TEMPLATE/feature_request.yml      |   40 ++++
 .github/PULL_REQUEST_TEMPLATE.md                |   17 ++
 .github/workflows/ci.yaml                       |  144 ++++++++++++
 .gitignore                                      |    7 +
 AGENTS.md                                       |  203 +++++++++++++++++
 CHANGELOG.md                                    |   26 +++
 CITATION.cff                                    |   29 +++
 CLAUDE.md                                       |   25 ++
 CODE_OF_CONDUCT.md                              |   47 ++++
 CONTRIBUTING.md                                 |  127 +++++++++++
 Cargo.lock                                      |    7 +
 Cargo.toml                                      |   50 ++++
 INITIAL_PLANNING_PLAN.md                        |  248 --------------------
 LICENSE-APACHE                                  |  202 ++++++++++++++++
 LICENSE-MIT                                     |   21 ++
 LICENSE-MULAN                                   |  130 +++++++++++
 LICENSES/Apache-2.0.txt                         |  202 ++++++++++++++++
 LICENSES/CC-BY-4.0.txt                          |  156 +++++++++++++
 LICENSES/MIT.txt                                |   21 ++
 LICENSES/MulanPSL-2.0.txt                       |  130 +++++++++++
 LICENSING.md                                    |  115 ++++++++++
 Makefile                                        |   89 ++++++++
 README.md                                       |   75 ++++++
 REUSE.toml                                      |   30 +++
 SECURITY.md                                     |  159 +++++++++++++
 cliff.toml                                      |   75 ++++++
 clippy.toml                                     |    6 +
 deny.toml                                       |   30 +++
 docs/BRAND.md                                   |   81 +++++++
 docs/LICENSING-BACKTRANSLATION.md               |   71 ++++++
 docs/LICENSING.zh-Hans.md                       |   23 ++
 docs/architecture/normal-form.md                |  142 ++++++++++++
 docs/architecture/okf-profile.md                |  127 +++++++++++
 docs/capabilities/census-cli.md                 |  118 ++++++++++
 docs/capabilities/census-debt-ratchet.md        |  126 ++++++++++
 docs/capabilities/census-detectors.md           |  134 +++++++++++
 docs/capabilities/census-ledger.md              |  179 +++++++++++++++
 docs/capabilities/census-migration.md           |  128 +++++++++++
 docs/capabilities/census-normal-form.md         |  178 +++++++++++++++
 docs/capabilities/census-patterns.md            |  137 +++++++++++
 docs/capabilities/census-reference-scan.md      |  106 +++++++++
 docs/capabilities/census-rule-api.md            |  159 +++++++++++++
 docs/capabilities/okf-profile.md                |  147 ++++++++++++
 docs/capabilities/source-index.md               |  134 +++++++++++
 docs/capabilities/workplan-cli.md               |  115 ++++++++++
 docs/capabilities/workplan-complete-gate.md     |  143 ++++++++++++
 docs/capabilities/workplan-current-rationale.md |  115 ++++++++++
 docs/capabilities/workplan-document-model.md    |  234 +++++++++++++++++++
 docs/capabilities/workplan-entity-history.md    |  115 ++++++++++
 docs/capabilities/workplan-evidence-claims.md   |  111 +++++++++
 docs/capabilities/workplan-generated-docs.md    |  132 +++++++++++
 docs/capabilities/workplan-prose-assessment.md  |  111 +++++++++
 docs/capabilities/workplan-reconcile.md         |  135 +++++++++++
 docs/capabilities/workplan-two-way-coverage.md  |  117 ++++++++++
 docs/raf-logo.svg                               |  133 +++++++++++
 docs/social-preview.png                         |  Bin 0 -> 93145 bytes
 docs/social-preview.svg                         |   84 +++++++
 rust-toolchain.toml                             |    9 +
 rustfmt.toml                                    |    6 +
 src/lib.rs                                      |    6 +
 tests/docs_contract.rs                          | 1799 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 work/changes/census-cli.md                      |   26 +++
 work/changes/census-debt-ratchet.md             |   27 +++
 work/changes/census-detectors.md                |   26 +++
 work/changes/census-ledger.md                   |   27 +++
 work/changes/census-migration.md                |   26 +++
 work/changes/census-normal-form.md              |   28 +++
 work/changes/census-patterns.md                 |   26 +++
 work/changes/census-reference-scan.md           |   26 +++
 work/changes/census-rule-api.md                 |   27 +++
 work/changes/okf-profile.md                     |   25 ++
 work/changes/source-index.md                    |   25 ++
 work/changes/workplan-cli.md                    |   24 ++
 work/changes/workplan-complete-gate.md          |   25 ++
 work/changes/workplan-current-rationale.md      |   25 ++
 work/changes/workplan-document-model.md         |   25 ++
 work/changes/workplan-entity-history.md         |   25 ++
 work/changes/workplan-evidence-claims.md        |   25 ++
 work/changes/workplan-generated-docs.md         |   24 ++
 work/changes/workplan-prose-assessment.md       |   25 ++
 work/changes/workplan-reconcile.md              |   25 ++
 work/changes/workplan-two-way-coverage.md       |   25 ++
 86 files changed, 8184 insertions(+), 248 deletions(-)
```

## PR feedback surfaces (raw/pr-11*.json)
- Reviews: 0; inline comments: 0; review threads: 0 (GraphQL).
- PR comments: 1 author handoff (Stage 1). Issue comments: 9 author workflow posts.
- Checks: all 8 jobs FAILURE in 2–3 s — job not started, org billing block (raw/pr-11-checks.txt).
