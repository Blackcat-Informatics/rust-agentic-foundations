# Validation

## Task 10 real use

Binary: `target/debug/cargo-workplan` built from HEAD 6f7afe1 (`cargo build -q -p cargo-workplan`), run as `cargo-workplan --root <worktree> ...`. No code change in this task.

### Authored edits

* `docs/architecture/normal-form.md`: `## Decision` (after the introduction) and `## Conditions to revisit` (4 items, end of page).
* `docs/architecture/okf-profile.md`: `## Decision` (after the introduction) and `## Conditions to revisit` (4 items, end of page).
* `docs/capabilities/workplan-evidence-claims.md` Explanation: `### Decision` (digest-based staleness) and `### Conditions to revisit` (3 items); the record `work/changes/workplan-evidence-claims.md` gains `cap:workplan-evidence-claims#explanation` in `affects`.
* `docs/capabilities/workplan-complete-gate.md` Contract, Bounds: the timeout sentence split out as its own paragraph with `<!-- raf:claim evidence=gate-kills-a-test-past-its-timeout -->`.
* `tests/docs_contract.rs` INVENTORY `records` lists the two new records (and the pin test).

### check before the evidence existed (claim names a missing record) - exit 1
```text
docs/capabilities/workplan-complete-gate.md: line 421: the claim names `evidence:gate-kills-a-test-past-its-timeout`, which is not an evidence record [claim-evidence-missing]; record the evidence as work/evidence/gate-kills-a-test-past-its-timeout.md, or name an existing record
```

### Evidence (R2)

`evidence record --id gate-kills-a-test-past-its-timeout --for cap:workplan-complete-gate --by agent --copyright '2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>' --licence CC-BY-4.0 --from-binding raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran` - exit 0 (22 s)
```text
recorded: work/evidence/gate-kills-a-test-past-its-timeout.md (pass)
```
The record holds 16 `symbol` inputs (the code bindings of `cap:workplan-complete-gate`), subject `binding:...`, verdict pass, by agent, SPDX lines.

`evidence claims gate-kills-a-test-past-its-timeout` - exit 0
```text
docs/capabilities/workplan-complete-gate.md:421: cap:workplan-complete-gate#contract
    A bound test that runs past its timeout is killed and fails its binding, and the
    reason says that it was killed, with the time it ran and its limit.
```
exit 0

`evidence for cap:workplan-complete-gate`
```text
evidence:gate-kills-a-test-past-its-timeout  pass  work/evidence/gate-kills-a-test-past-its-timeout.md  (in its for list; claims at lines 421)
```
exit 0

Neighbour check (working copy only, restored byte for byte): with the record's verdict edited to `fail`, `change reconcile workplan-complete-gate` printed
```text
cap:workplan-complete-gate  contract  claim-unsupported  line 421: a passing record within its limits for each evidence id the claim names, or the claim reworded or removed (`evidence:gate-kills-a-test-past-its-timeout` has the verdict fail)
```
and with the record restored, no claim or evidence line.

`change reconcile workplan-complete-gate` - exit 1 (only unresolved-proposal, assessment-missing and item-without-outcome, which Task 11 resolves; no claim-unsupported, evidence-stale or evidence-changed)
```text
cap:workplan-complete-gate  contract  unresolved-proposal  line 262: the block is replaced by plain contract text, or removed because the claim was dropped
cap:workplan-complete-gate  contract  unresolved-proposal  line 295: the block is replaced by plain contract text, or removed because the claim was dropped
cap:workplan-complete-gate  contract  unresolved-proposal  line 331: the block is replaced by plain contract text, or removed because the claim was dropped
cap:workplan-complete-gate  contract  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-complete-gate#contract` has no assessment `assessment:workplan-complete-gate-contract`)
cap:workplan-complete-gate  examples  unresolved-proposal  line 462: the block is replaced by plain contract text, or removed because the claim was dropped
cap:workplan-complete-gate  examples  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-complete-gate#examples` has no assessment `assessment:workplan-complete-gate-examples`)
cap:workplan-complete-gate  verification  unresolved-proposal  line 590: the block is replaced by plain contract text, or removed because the claim was dropped
cap:workplan-complete-gate  verification  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-complete-gate#verification` has no assessment `assessment:workplan-complete-gate-verification`)
change:workplan-complete-gate  item 1  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:workplan-complete-gate  item 2  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:workplan-complete-gate  item 3  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:workplan-complete-gate  item 4  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
```
exit 1

### Coverage (R4)

`work/coverage/default.md`: `scope: [raf-workplan, cargo-workplan]`, `exclusions: []` (unchanged).

`coverage code` - exit 1, 212 gaps: raf-okf 177; raf-source 35; 
```text
| Gap | Crate | Path | Location |
| --- | --- | --- | --- |
| public item | raf-okf | `raf_okf::Actor` | crates/raf-okf/src/profile/typed.rs:348 |
| public item | raf-okf | `raf_okf::Actor::ALL` | crates/raf-okf/src/profile/typed.rs:357 |
...
| public item | raf-source | `raf_source::TreeError::path` | crates/raf-source/src/error.rs:95 |
| public item | raf-source | `raf_source::Vis` | crates/raf-source/src/facts.rs:66 |
```

`coverage tests` - exit 1, 440 gaps: raf-okf 140; raf-source 215; rust-agentic-foundations 85; 
```text
| Gap | Crate | Path | Location |
| --- | --- | --- | --- |
| test | raf-okf | `raf_okf::error::tag_tests::all_lists_every_error_kind_once_in_variant_order` | crates/raf-okf/src/error.rs:871 |
| test | raf-okf | `raf_okf::error::tag_tests::names_are_unique_lowercase_kebab_case` | crates/raf-okf/src/error.rs:885 |
...
| test | rust-agentic-foundations | `rust_agentic_foundations::test_crate::repo_gates::lints_rejects_an_added_or_removed_lint_in_the_fuzz_tables` | tests/repo_gates.rs:890 |
| test | rust-agentic-foundations | `rust_agentic_foundations::test_crate::repo_gates::lints_the_root_denies_missing_docs_unsafe_code_and_unreachable_pub` | tests/repo_gates.rs:819 |
```

`coverage code --crate raf-workplan` - exit 0
```text
| Gap | Crate | Path | Location |
| --- | --- | --- | --- |
```
`coverage tests --crate raf-workplan` - exit 0
```text
| Gap | Crate | Path | Location |
| --- | --- | --- | --- |
```
`coverage code --crate cargo-workplan` - exit 0
```text
| Gap | Crate | Path | Location |
| --- | --- | --- | --- |
```
`coverage tests --crate cargo-workplan` - exit 0
```text
| Gap | Crate | Path | Location |
| --- | --- | --- | --- |
```

### History (R1)

Run on the working tree before commit; every document and symbol history comes from HEAD. `git log -- <path>` lists the same document commits.

`history cap:workplan-reconcile` - exit 0
```text
| Commit | Date | Subject | Source |
| --- | --- | --- | --- |
| 6b317fc | 2026-10-16 | feat(raf-workplan): two-way coverage of public items and tests | document |
| bb415d3 | 2026-10-16 | feat(raf-workplan): prose assessments, their obligations and a sixth completion condition | document, `raf_workplan::obligation::Needed`, `raf_workplan::obligation::ObligationKind`, `raf_workplan::obligation::suppressed` |
| 9bacdb3 | 2026-10-16 | feat(raf-workplan): evidence records, claims and their obligations | document, `raf_workplan::Workplan::reconcile`, `raf_workplan::lifecycle::Lifecycle::reconcile`, `raf_workplan::obligation::ClaimProblem`, `raf_workplan::obligation::EvidenceChange`, `raf_workplan::obligation::InputMoved`, `raf_workplan::obligation::Needed`, `raf_workplan::obligation::ObligationKind`, `raf_workplan::obligation::OverLimit`, `raf_workplan::obligation::suppressed` |
| de0b842 | 2026-10-15 | feat(raf-workplan): load evidence, assessment and coverage records with a shared digest | `raf_workplan::Workplan::new_change` |
| a72c5f5 | 2026-10-14 | docs(capabilities): name the git commands of the default-branch lookup | document |
| 92e4de8 | 2026-10-14 | docs(workplan): deliver the reconcile, completion gate and command line capabilities | document |
| 41eba82 | 2026-10-14 | docs(workplan): state the built contract of the lifecycle capabilities | document |
| ed5ec73 | 2026-10-14 | docs(workplan): propose the contract edits of the lifecycle capabilities | document |
| 0e999a1 | 2026-10-14 | docs(workplan): bind the reconcile, completion gate and command line capabilities | document |
| 0668be1 | 2026-10-14 | feat(cargo-workplan): change and verify commands | `raf_workplan::Workplan::new_change`, `raf_workplan::lifecycle::NewChange`, `raf_workplan::lifecycle::NewChangeError`, `raf_workplan::lifecycle::NewChangeRequest` |
| 3f6cc3b | 2026-10-14 | feat(raf-workplan): section-untouched, example-failing and verify | `raf_workplan::git::MergeBase`, `raf_workplan::git::blob_at`, `raf_workplan::git::default_merge_base`, `raf_workplan::git::show_prefix`, `raf_workplan::lifecycle::Lifecycle::open`, `raf_workplan::lifecycle::Lifecycle::reconcile`, `raf_workplan::lifecycle::section_digest`, `raf_workplan::obligation::Needed` |
| 80c303e | 2026-10-14 | feat(raf-workplan): reconcile obligations that need no process | `raf_workplan::Workplan::reconcile`, `raf_workplan::obligation`, `raf_workplan::obligation::Needed`, `raf_workplan::obligation::Obligation`, `raf_workplan::obligation::ObligationKind`, `raf_workplan::obligation::Reconciled`, `raf_workplan::obligation::suppressed`, `raf_workplan::obligation::tense::PHRASES` |
| 4c97695 | 2026-10-14 | feat(raf-workplan): open a change and report what it touches | `raf_workplan::Workplan::impact`, `raf_workplan::Workplan::new_change`, `raf_workplan::lifecycle`, `raf_workplan::lifecycle::Impact`, `raf_workplan::lifecycle::NewChange`, `raf_workplan::lifecycle::NewChangeError`, `raf_workplan::lifecycle::NewChangeRequest` |
| 8c5eeba | 2026-10-14 | feat(raf-workplan): git operations, Gate::open and workspace root discovery | `raf_workplan::git`, `raf_workplan::git::blob_at`, `raf_workplan::git::default_merge_base`, `raf_workplan::git::require_version` |
| 6120029 | 2026-10-12 | docs(security): state the trust model and harden the designed parsing and subprocess contracts | document |
| f711aa2 | 2026-10-12 | docs(workplan): define the seven sections and close the workplan contract gaps | document |
| 9ce066f | 2026-10-11 | docs(capabilities): add workplan capability specs | document |
```

`history arch:normal-form` - exit 0
```text
| Commit | Date | Subject | Source |
| --- | --- | --- | --- |
| edf93d5 | 2026-10-12 | docs: resolve contradictions in the census and workplan contracts | document |
| 25e1978 | 2026-10-11 | docs(capabilities): add census capability specs and architecture pages | document |
```

`history arch:okf-profile` - exit 0
```text
| Commit | Date | Subject | Source |
| --- | --- | --- | --- |
| 9bacdb3 | 2026-10-16 | feat(raf-workplan): evidence records, claims and their obligations | document |
| de0b842 | 2026-10-15 | feat(raf-workplan): load evidence, assessment and coverage records with a shared digest | document |
| 6e75ac7 | 2026-10-15 | feat(raf-okf): evidence, assessment and coverage records and claim markers | document |
| a3d60dd | 2026-10-14 | docs(capabilities): deliver the workplan document model and generated docs | document |
| 4e9c2f3 | 2026-10-12 | test(docs-contract): read documents through raf-okf and resolve bindings | document |
| 1a67fa3 | 2026-10-12 | feat(raf-okf): profile validation with typed documents | document |
| e94cd8d | 2026-10-12 | feat(raf-okf): body model, transactional splicing and capability index | document |
| 3f8697b | 2026-10-12 | feat(raf-okf): bounded frontmatter reader and deterministic writer | document |
| edf93d5 | 2026-10-12 | docs: resolve contradictions in the census and workplan contracts | document |
| dba307f | 2026-10-12 | test(docs): harden the document contract | document |
| 6120029 | 2026-10-12 | docs(security): state the trust model and harden the designed parsing and subprocess contracts | document |
| f711aa2 | 2026-10-12 | docs(workplan): define the seven sections and close the workplan contract gaps | document |
| 9ce066f | 2026-10-11 | docs(capabilities): add workplan capability specs | document |
| 25e1978 | 2026-10-11 | docs(capabilities): add census capability specs and architecture pages | document |
```

`history cap:workplan-entity-history --since 2026-01-01` - exit 0
```text
| Commit | Date | Subject | Source |
| --- | --- | --- | --- |
| 6f7afe1 | 2026-10-16 | feat(cargo-workplan): history, evidence, assess and coverage commands | document, `raf_workplan::Workplan::history` |
| 6b317fc | 2026-10-16 | feat(raf-workplan): two-way coverage of public items and tests | document |
| 73662ab | 2026-10-16 | feat(raf-workplan): history of a capability, change or architecture page from git | document, `raf_workplan::Workplan::history`, `raf_workplan::git::AuthorDate`, `raf_workplan::git::LastChange`, `raf_workplan::git::LogEntry`, `raf_workplan::git::document_log`, `raf_workplan::git::is_shallow`, `raf_workplan::git::last_change`, `raf_workplan::git::symbol_log`, `raf_workplan::history` |
| 8c5eeba | 2026-10-14 | feat(raf-workplan): git operations, Gate::open and workspace root discovery | `raf_workplan::git::resolve_commit` |
| 6120029 | 2026-10-12 | docs(security): state the trust model and harden the designed parsing and subprocess contracts | document |
| 9f93e5d | 2026-10-11 | docs(capabilities): add considered workplan capabilities and retire the essay | document |
```

`history cap:workplan-entity-history --since 2026-10-15` - exit 0
```text
| Commit | Date | Subject | Source |
| --- | --- | --- | --- |
| 6f7afe1 | 2026-10-16 | feat(cargo-workplan): history, evidence, assess and coverage commands | document, `raf_workplan::Workplan::history` |
| 6b317fc | 2026-10-16 | feat(raf-workplan): two-way coverage of public items and tests | document |
| 73662ab | 2026-10-16 | feat(raf-workplan): history of a capability, change or architecture page from git | document, `raf_workplan::Workplan::history`, `raf_workplan::git::AuthorDate`, `raf_workplan::git::LastChange`, `raf_workplan::git::LogEntry`, `raf_workplan::git::document_log`, `raf_workplan::git::is_shallow`, `raf_workplan::git::last_change`, `raf_workplan::git::symbol_log`, `raf_workplan::history` |
```

`history evidence:gate-kills-a-test-past-its-timeout` - exit 0
```text
| Commit | Date | Subject | Source |
| --- | --- | --- | --- |
```

`history evidence:gate-kills-a-test-past-its-timeout` lists no commit because the record is not committed yet; it lists its commit once the record is committed.

### Assessment (R3)

`assess cap:workplan-current-rationale#explanation --context` - exit 0 (344 lines: the section text, `raf_workplan::rationale` with its source, and the inputs). Inputs printed:
```text
inputs
  section cap:workplan-current-rationale#explanation edd528665b156daab80499f66ab5bbce8be6a1eee5a612e267254bf5e35699f1
  symbol raf_workplan::rationale 8959f873b524e3f0a3d68e0fd0c4006843776efbec11d658503ab0826fb915d9
```

`assess cap:workplan-current-rationale#explanation --verdict accepted --by agent --copyright '2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>' --licence CC-BY-4.0` - exit 0
```text
recorded: work/assessments/workplan-current-rationale-explanation.md (accepted by agent)
```

Its inputs: the section `cap:workplan-current-rationale#explanation` and the symbol `raf_workplan::rationale` (the whole module, since the section names no bound symbol). It goes stale on an edit of that Explanation or of anything in `crates/raf-workplan/src/rationale.rs` (and modules below it), or when the capability's code bindings change. The section is not in the `affects` of `workplan-current-rationale`.

`assess --list` - exit 0
```text
cap:workplan-current-rationale#explanation  assessment:workplan-current-rationale-explanation  accepted  agent  current
```

`assess cap:workplan-current-rationale#explanation --list` - exit 0
```text
cap:workplan-current-rationale#explanation  assessment:workplan-current-rationale-explanation  accepted  agent  current
```

### check and reconcile

`check` - exit 0
```text
```

`change reconcile workplan-evidence-claims` - exit 1; trace lines (claim-unsupported, evidence-stale, evidence-changed, assessment-stale, assessment-rejected): 0; the rest are unresolved-proposal, assessment-missing, binding-missing (examples) and item-without-outcome.
```text
cap:workplan-evidence-claims  contract  unresolved-proposal  line 75: the block is replaced by plain contract text, or removed because the claim was dropped
cap:workplan-evidence-claims  contract  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-evidence-claims#contract` has no as
cap:workplan-evidence-claims  usage  unresolved-proposal  line 137: the block is replaced by plain contract text, or removed because the claim was dropped
cap:workplan-evidence-claims  usage  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-evidence-claims#usage` has no assessme
cap:workplan-evidence-claims  examples  unresolved-proposal  line 153: the block is replaced by plain contract text, or removed because the claim was dropped
cap:workplan-evidence-claims  examples  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-evidence-claims#examples` has no as
cap:workplan-evidence-claims  constraints  unresolved-proposal  line 168: the block is replaced by plain contract text, or removed because the claim was dropped
cap:workplan-evidence-claims  constraints  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-evidence-claims#constraints` has
cap:workplan-evidence-claims  explanation  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-evidence-claims#explanation` has
cap:workplan-evidence-claims  verification  unresolved-proposal  line 221: the block is replaced by plain contract text, or removed because the claim was droppe
cap:workplan-evidence-claims  verification  binding-missing  a binding that resolves through the source index (bindings.examples is empty)
cap:workplan-evidence-claims  verification  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-evidence-claims#verification` h
change:workplan-evidence-claims  item 1  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:workplan-evidence-claims  item 2  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:workplan-evidence-claims  item 3  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:workplan-evidence-claims  item 4  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:workplan-evidence-claims  item 5  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
```

`change reconcile workplan-current-rationale` - exit 1; trace lines (claim-unsupported, evidence-stale, evidence-changed, assessment-stale, assessment-rejected): 0; the rest are unresolved-proposal, assessment-missing, binding-missing (examples) and item-without-outcome.
```text
cap:workplan-current-rationale  contract  unresolved-proposal  line 53: the block is replaced by plain contract text, or removed because the claim was dropped
cap:workplan-current-rationale  contract  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-current-rationale#contract` has n
cap:workplan-current-rationale  usage  unresolved-proposal  line 112: the block is replaced by plain contract text, or removed because the claim was dropped
cap:workplan-current-rationale  usage  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-current-rationale#usage` has no asse
cap:workplan-current-rationale  examples  unresolved-proposal  line 124: the block is replaced by plain contract text, or removed because the claim was dropped
cap:workplan-current-rationale  examples  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-current-rationale#examples` has n
cap:workplan-current-rationale  constraints  unresolved-proposal  line 143: the block is replaced by plain contract text, or removed because the claim was dropp
cap:workplan-current-rationale  constraints  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-current-rationale#constraints`
cap:workplan-current-rationale  verification  unresolved-proposal  line 173: the block is replaced by plain contract text, or removed because the claim was drop
cap:workplan-current-rationale  verification  binding-missing  a binding that resolves through the source index (bindings.examples is empty)
cap:workplan-current-rationale  verification  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-current-rationale#verificatio
change:workplan-current-rationale  item 1  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:workplan-current-rationale  item 2  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:workplan-current-rationale  item 3  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:workplan-current-rationale  item 4  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:workplan-current-rationale  item 5  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
```

`change reconcile okf-profile` - exit 1; trace lines (claim-unsupported, evidence-stale, evidence-changed, assessment-stale, assessment-rejected): 0; the rest are unresolved-proposal, assessment-missing, binding-missing (examples) and item-without-outcome.
```text
cap:okf-profile  contract  unresolved-proposal  line 272: the block is replaced by plain contract text, or removed because the claim was dropped
cap:okf-profile  contract  assessment-missing  an assessment of the section recorded with assess (`cap:okf-profile#contract` has no assessment `assessment:okf-p
cap:okf-profile  verification  unresolved-proposal  line 444: the block is replaced by plain contract text, or removed because the claim was dropped
cap:okf-profile  verification  binding-missing  a binding that resolves through the source index (bindings.examples is empty)
cap:okf-profile  verification  assessment-missing  an assessment of the section recorded with assess (`cap:okf-profile#verification` has no assessment `assessme
change:okf-profile  item 1  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:okf-profile  item 2  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:okf-profile  item 3  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:okf-profile  item 4  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:okf-profile  item 5  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
change:okf-profile  item 6  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>
```

### Gates (Task 10, working tree with every change staged)

| Gate | Exit | Result |
| --- | --- | --- |
| `cargo test --workspace --locked` (CARGO_BUILD_JOBS=6) | 0 | 1915 passed, 0 failed; docs_contract 28, repo_gates 57, raf-workplan rationale 14, raf-okf schema 6 |
| `cargo fmt --all --check` | 0 | clean |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | clean (only `tests/docs_contract.rs` changed among Rust files) |
| `make reuse` | 0 | 441 / 441 files with licence information |
| `make issue-refs` | 0 | repo_gates 57 passed |
| `make docs-sync` twice | 0, 0 | `git diff` digest identical before, after the first and after the second run (no generated block changed) |
| `cargo-workplan check` | 0 | no finding |
| `.githooks/pre-commit` on the staged index | 0 | fmt, reuse, repo_gates, docs_contract green |

Commit order: the claim marker needs its record in the same commit (`check` reports `claim-evidence-missing` otherwise), so the order is (1) the architecture pages, the Explanation pair and the `affects` entry; (2) the complete-gate claim, the evidence record, the assessment record and the INVENTORY `records` entries. Each is green alone: (1) touches no record and no claim; (2) adds the records the claim and INVENTORY name.

## Task 11A: resolve the PROPOSED blocks of okf-profile, source-index, workplan-document-model, workplan-generated-docs

Commits (each through the pre-commit hook: fmt, reuse, repo_gates 57 passed, docs_contract 28 passed; each pushed):

| Commit | Capability | Files |
| --- | --- | --- |
| `3bd9e56` | `cap:okf-profile` | `docs/capabilities/okf-profile.md`, `work/changes/okf-profile.md`, `docs/capabilities/index.md` |
| `dbc26f7` | `cap:source-index` | `docs/capabilities/source-index.md`, `work/changes/source-index.md`, `crates/raf-source/src/lib.rs`, `docs/capabilities/index.md` |
| `e76c4cf` | `cap:workplan-document-model` | `docs/capabilities/workplan-document-model.md`, `work/changes/workplan-document-model.md`, `crates/raf-workplan/src/gate.rs` |
| `928ca4d` | `cap:workplan-generated-docs` | `docs/capabilities/workplan-generated-docs.md`, `work/changes/workplan-generated-docs.md` |

Each commit was built from HEAD plus only its own files (the other capabilities' files restored to HEAD and `make docs-sync` run), so the catalogue row of each commit matches its own bindings.

### Bindings made valid for `change complete`

* `cap:okf-profile`: `bindings.examples` was empty; it now binds the runnable documentation tests `raf_okf::EvidenceRecord::to_document` (build, write, parse and validate a record back to an equal one) and `raf_okf::Claim` (read a claim marker). Verification names them.
* `cap:source-index`: `bindings.examples` was empty; it now binds the runnable documentation test `raf_source::Workspace::exported_items`. The `registry_scan` example needs a local registry and an argument, so it is not hermetic and is not bound. The two `#[cfg(windows)]` tests `tree::disk::tests::windows::the_final_path_of_a_handle_is_its_canonical_path_and_lies_under_the_root` and `tree::disk::tests::windows::a_disk_tree_refuses_a_reserved_device_name_or_a_trailing_dot_or_space_beside_a_plain_name` are unbound; their Linux-run siblings stay bound (`the_handle_path_verifier_refuses_a_path_outside_the_root_beside_one_inside`, `reserved_device_names_and_trailing_dots_or_spaces_are_ambiguous_on_windows_beside_plain_names`, `test_crate::workspace_containment::a_tree_with_windows_device_names_loads_off_windows`) and `linux_verifier_reads_the_real_descriptor_of_an_open_file` is added. A Constraints bullet states that the Windows-only proofs are compiled on every host by `make cross-check` and run only on Windows, in the Windows CI job.
* `cap:workplan-document-model`: the `Gate` struct documentation test was `no_run`. It now runs: it writes a repository of one package and one delivered capability (plus a coverage record with `scope: []`) to a scratch directory and runs `Gate::workspace(dir).tracked_paths([]).try_check()`, asserting the one finding `index-missing` (the bindings resolve; no `git` is started). The `tests/workplan.rs` usage block moved to `Gate::workspace` as a `no_run` block that says it is compiled and not run. The binding `raf_workplan::Gate` stays.
* `cap:workplan-generated-docs`: no binding change; the Verification section now describes what the newly bound tests prove, which clears `section-untouched`.

### Delivered sentences removed or reworded (for the PR body)

`cap:okf-profile`
1. Contract opening: "...the body rules and the PROPOSED and generated block syntax, are stated once..." -> "...the body rules and the syntax of the PROPOSED blocks, the generated blocks and the claim markers, are stated once..." (the architecture page now also states the claim marker syntax).
2. Validate: "returns a typed result: the typed document (`RafDocument`, ...), or the list of every violation" -> the same with the record variants (`RafDocument::Evidence|Assessment|Coverage` holding `EvidenceRecord|AssessmentRecord|CoverageRecord` with a `RecordHeader`) inserted (widened).
3. Validate rule: "a missing `type`, or one other than `Capability`, `Change` or `Architecture`, is the only violation reported for that document" -> "a missing `type`, or one outside `DocType::ALL`, is the only violation ..." (superseded: six types; a new "Document types" paragraph lists them).
4. Validate rule: "...an Architecture body is not empty." -> "...an Architecture body is not empty; a record body is empty." (widened).
5. Body model: "So are block quotes and list items nested over 64 levels." -> extended with the claim marker parse errors (malformed, unattached, unknown `raf:` comment) (widened; the following `Body::replace_spans` sentence only reflowed).
   Removed: nothing. Added: Document types, the record rules list, the `claims` bullet of the body model, the rule "a claim marker appears only in a Capability", the Writing records paragraph, and the Verification items of the change plus the bound-examples line.

`cap:source-index`
1. Contract (use-alias cycles): "that place is left to the glob for the paths that read it, though not yet for the globs of other modules, and the cycle resolves through it" -> "...for the paths that read it, and not for the globs of other modules, and the cycle resolves through it" (tense scan; the fallback answers the paths of the cycle only, per `settle.rs` `Act::Fallback`).
2. API surface: one sentence appended (`Exports`, `ExportedItem`, `Reexport`, `ExportOptions` and `ExportError` are `#[non_exhaustive]`; `ExportOptions` is built from `default()`); no sentence removed.
3. Proposed text reworded on resolution: the Usage sentence "The crate documentation of `raf-source` shows the listing over a real tree ... and over a `MemoryTree` ..." is replaced by the two blocks themselves (Usage: the real-tree `no_run` block; Examples: the `MemoryTree` block), which equal crate doctests 2 and 4 in `lib.rs` (reordered).
   Removed: nothing. Added: the Exported items Contract text (placed before API surface), the Usage paragraph and block, the Examples block and paragraph, the Constraints bullet of the proposal and the Windows-proofs bullet, and the Verification items.

`cap:workplan-document-model`
1. The four states: "the current account stays as written, and the proposed text sits beside it in PROPOSED blocks" -> "...and the new text of the change sits beside it in PROPOSED blocks" (tense scan; same meaning).
2. Graph Nodes bullet: "...proposals, and the code symbols, tests and examples that bindings name." -> adds "the records, the crates the coverage record scopes and the symbols the inputs of records name" (widened); a new "Record edges" bullet holds the five record edges.
3. Graph queries: one sentence appended listing the record queries (`Graph::covers`, `evidence_of`, `claims`, `claims_on`, `assesses`, `assessments_of`, `scope`, `inputs`, `inputs_of`).
4. Checks list: the last bullet ends with `;` and a new bullet lists the record and claim reference findings.
5. Usage: "`Gate::workspace` reads `docs/capabilities`, `docs/architecture` and `work/changes` under the directory, loads..." -> adds "and the records of `work/evidence`, `work/assessments` and `work/coverage`"; "...and a missing `work/changes` is the empty set of changes." -> "..., a missing `work/changes` is the empty set of changes, and a missing record directory holds no record."
6. Proposed text reworded on resolution: "a repository without records loads and checks as it did before" was untrue as written (a repository with no coverage record has the `coverage-record-missing` finding of two-way coverage) and history wording -> "loads and reports no finding of the record rules; that it has no coverage record is the one finding two-way coverage gives it". "An input whose name resolves to nothing now makes no edge" -> "...resolves to nothing the repository holds makes no edge and is no finding of the load: an input that moved is a matter of the record's currency" (wording of `Graph::inputs`).
   Removed: nothing. Added: the Records, Digests and Writing records paragraphs, the Verification items of the change and a line for the `Gate` documentation example.

`cap:workplan-generated-docs`
   Removed or reworded: nothing. Each Verification bullet gains the sentences for the newly bound tests (relation listed twice, unresolved example, backtick code spans, state heading capital; check/sync agreement, frontmatter kept; line ending of the first break, LF frontmatter over CRLF body, CRLF idempotence; index header kept, licence-only index, no index without capabilities; index only where its directory exists, permissions kept, temporary name never touched, the `docs_sync` example's gate open, SPDX flags, git checkout and argument refusals; the preview edge cases and slug lookup).

### Record items

All 21 items (okf-profile 6, source-index 6, workplan-document-model 7, workplan-generated-docs 2) are `implemented`, each checked against code and tests (`raf-okf` `profile/records.rs` limit rule and `body_rules.rs` `ClaimOutsideCapability`; `raf-source` `exports.rs`; `raf-workplan` `graph.rs`, `digest.rs`, `record.rs`, `finding.rs`; the bound tests listed in each capability). No item reworded, none moved, none dropped; no new record created. `affects` already listed every edited section (okf-profile: contract, verification; source-index: contract, usage, examples, constraints, verification; workplan-document-model: contract, usage, verification; workplan-generated-docs: verification); no widening needed.

### Verify (`cargo run -q -p cargo-workplan -- --root <worktree> verify <slug>`, the built binary, at HEAD `928ca4d`, clean tree, run one after another)

| Capability | Exit | Bindings | Wall time |
| --- | --- | --- | --- |
| `okf-profile` | 0 | 88 pass, 0 fail (examples `raf_okf::EvidenceRecord::to_document`, `raf_okf::Claim` pass) | 85 s |
| `source-index` | 0 | 293 pass, 0 fail (example `raf_source::Workspace::exported_items` passes) | 584 s |
| `workplan-document-model` | 0 | 253 pass, 0 fail (example `raf_workplan::Gate` passes: 1 selected, 1 ok) | 335 s |
| `workplan-generated-docs` | 0 | 87 pass, 0 fail | 142 s |

Before the edits the same runs gave: okf-profile exit 0 (no example bound), source-index exit 1 (the two `tests::windows::*` bindings: the `--lib` list pass shows no such test), workplan-document-model `reconcile` `example-failing` on the `no_run` `gate::Gate` doc test.

### Reconcile at HEAD (each exit 1; the only lines are `assessment-missing`, recorded by a later part once all text is final)

```text
### okf-profile
cap:okf-profile  contract  assessment-missing  an assessment of the section recorded with assess (`cap:okf-profile#contract` has no assessme
cap:okf-profile  verification  assessment-missing  an assessment of the section recorded with assess (`cap:okf-profile#verification` has no 
exit 1
### source-index
cap:source-index  contract  assessment-missing  an assessment of the section recorded with assess (`cap:source-index#contract` has no assess
cap:source-index  usage  assessment-missing  an assessment of the section recorded with assess (`cap:source-index#usage` has no assessment `
cap:source-index  examples  assessment-missing  an assessment of the section recorded with assess (`cap:source-index#examples` has no assess
cap:source-index  constraints  assessment-missing  an assessment of the section recorded with assess (`cap:source-index#constraints` has no 
cap:source-index  verification  assessment-missing  an assessment of the section recorded with assess (`cap:source-index#verification` has n
exit 1
### workplan-document-model
cap:workplan-document-model  contract  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-document-model#c
cap:workplan-document-model  usage  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-document-model#usag
cap:workplan-document-model  verification  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-document-mod
exit 1
### workplan-generated-docs
cap:workplan-generated-docs  verification  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-generated-do
exit 1
```

### Gates at HEAD `928ca4d`

| Gate | Exit | Result |
| --- | --- | --- |
| `make docs-sync` | 0 | nothing rewritten |
| `cargo-workplan check` | 0 | no finding |
| `cargo test --test docs_contract --test repo_gates` | 0 | 28 and 57 passed (also in each pre-commit hook) |
| `cargo test -p raf-workplan -p raf-source -p raf-okf -p cargo-workplan` | 0 | 1831 passed, 0 failed (394 s) |
| `cargo test -p raf-source --doc`, `cargo test -p raf-workplan --doc -- Gate` | 0 | 9 passed; 2 passed |
| `make fmt` | 0 | clean |
| `make clippy` | 0 | clean |
| `make doc` | 0 | clean |
| `make reuse` | 0 | compliant |
| `make issue-refs` | 0 | 57 passed |

Not run here: `change complete`, any `delivered` state edit and any assessment (later parts).

## Task 11B: resolve the PROPOSED blocks of workplan-reconcile, workplan-complete-gate, workplan-cli

Commits (each through the pre-commit hook: fmt, reuse, docs_contract 28 passed, repo_gates 57 passed; each pushed):

| Commit | Capability | Files |
| --- | --- | --- |
| `cfec99f` | `cap:workplan-reconcile` | `docs/capabilities/workplan-reconcile.md`, `work/changes/workplan-reconcile.md` |
| `511d27a` | `cap:workplan-complete-gate` | `docs/capabilities/workplan-complete-gate.md`, `work/changes/workplan-complete-gate.md`, `docs/capabilities/index.md` (catalogue row regenerated by `make docs-sync`) |
| `1a5b28e` | `cap:workplan-cli` | `docs/capabilities/workplan-cli.md`, `work/changes/workplan-cli.md`, `AGENTS.md`, `README.md` |

No code changed. Every PROPOSED block of the three is resolved (`grep -n 'PROPOSED change:workplan-(reconcile|complete-gate|cli)'` finds none); the fenced `text` block inside the condition-6 Examples quote of complete-gate is now a normal fenced block; `docs preview workplan-complete-gate` and `docs preview workplan-cli` exit 0. Tense scan of the three bodies (`will`, `is planned`, `proposed`, `not yet`, `to be implemented`, `open questions`, `still`) outside code spans: no hit except `PROPOSED` marker names, the code-span phrase list of the reconcile Constraints, "every later command" (sequence, not deferral), and the Scenario B fixture description "a capability in `considered` with open questions" (no Open questions list) and the cli sentence "a bare `--list` still shows an assessment of it" (a present fact). None of the three held an "Open questions" list.

### Bindings valid for `change complete`

No binding edit was needed. Each capability binds code and tests; the bound examples are runnable documentation tests (`raf_workplan::Workplan::new_change|impact|reconcile`, `raf_workplan::lifecycle::section_digest`; `raf_workplan::bound`, `raf_workplan::exec`; `raf_workplan::Workplan::sync|preview|new_change|impact`): each `verify` line shows "1 selected, 1 ok lines", so none is `no_run` or `compile_fail`, and every bound test is listed once and ran on Linux ("1 listed, 1 passed").

### Verify (`cargo run -q -p cargo-workplan -- --root <checkout> verify <slug>`, run one after another, each on a clean detached checkout of the commit that resolved the capability, `.worktrees/verify-11b`, so editing the next document could not touch a run; build time excluded)

| Capability | Commit | Exit | Bindings | Wall time |
| --- | --- | --- | --- | --- |
| `workplan-reconcile` | `cfec99f` | 0 | 194 pass (190 tests, 4 examples), 0 fail | 294 s |
| `workplan-complete-gate` | `511d27a` | 0 | 200 pass (198 tests, 2 examples), 0 fail | 260 s |
| `workplan-cli` | `1a5b28e` | 0 | 201 pass (197 tests, 4 examples), 0 fail | 304 s |

### Delivered sentences removed or reworded (for the PR body)

`cap:workplan-reconcile`
1. Contract, obligation table: the six rows of the PROPOSED table (`claim-unsupported`, `evidence-stale`, `evidence-changed`, `assessment-missing`, `assessment-stale`, `assessment-rejected`) are rows of the one table after `tense-mismatch`, so it lists all 13 kinds in `ObligationKind::ALL` order; the pinned test `obligation::tests::the_evidence_strings_are_the_evidence_column_of_the_contract` passes (run alone and in `verify`). No delivered row changed.
2. Contract, `contract_effect: none`: one sentence appended (the assessment kinds apply under either effect); nothing removed.
3. Constraints: "a repository with no default branch reachable from `HEAD` makes `section-untouched` an error and not a silent pass" -> "...makes `section-untouched`, and `evidence-changed` for a change whose capabilities hold a claim, an error and not a silent pass" (widened to what is built and tested: `evidence::with_no_default_branch_a_claim_is_an_error_and_no_claim_asks_git_nothing`). `affects` gains `cap:workplan-reconcile#constraints` for it.
4. Verification: "An invocation fixture with stub `git` and `cargo` on `PATH` records each argv and asserts the forms above, and a default branch named `--output=x` is resolved to a commit id and never reaches `git` as an option." -> "An invocation fixture gives the library a stub `git` as the program it starts; the stub records each argv, working directory and locale, and the fixture asserts the commands above in order, each in the workspace root and in the C locale, and that a default branch named `--output=x` ... never reaches `git` as an option. The `cargo` argv forms of `example-failing` are the rows of the completion gate, whose stub fixture asserts each one." Reason: `untouched::stubs::reconcile_asks_git_exactly_the_commands_of_the_contract_in_order` passes the stub through `Programs::with_git`, not `PATH`, and starts no `cargo`; the cargo rows are asserted by `exec::stubs::every_row_reaches_a_process_as_exactly_its_argv` (bound to the completion gate). Not weakened: the cwd and locale assertions the test makes are now stated too.
   Proposed text reworded on resolution: the lead "Six kinds follow `tense-mismatch` in the table, and in the order of the list" is gone (the rows are in the table); the generated-only sentence "Every section of a capability that loads holds authored text, since the profile refuses one ..." -> "A section with nothing outside its generated blocks and comments cannot occur: the profile refuses such a section (`okf-section-empty`), so every section of a capability that loads holds authored text ..." (aligned with complete-gate condition 6 and the records); "They compare digests" -> "The assessment kinds compare digests"; Verification "One fixture per new kind" -> "One fixture per trace kind"; the two completion-gate clauses ("The completion gate refuses each of the three kinds ..." and "... and the completion gate refuses each of the three and completes once the section is assessed and accepted") -> one bullet "The per-kind fixture of the completion gate refuses each of the six trace and assessment kinds and completes once it is repaired: the record passing again, its digests current again or its limit restored, and the section assessed and accepted" (the repairs of `complete::trace_cases` and `assessment_cases`).
   Removed: nothing.

`cap:workplan-complete-gate`
1. Frontmatter `description`: "the five completion conditions" -> "the six completion conditions"; the catalogue row of `docs/capabilities/index.md` regenerated by `make docs-sync`.
2. Conditions: "`change complete <slug>` succeeds only when all five hold:" -> "all six hold:", and condition 6 is item 6 of the one list (superseding the PROPOSED "A sixth condition joins the five").
3. Order: "conditions 1 to 4 (the obligation list without `example-failing`, which needs a run) and the check of the repository once the record is removed" -> "conditions 1 to 4 (...), condition 6 (the currency of each assessment, from digests of the working tree) and the check of the repository once the record is removed"; "so a static failure starts no `cargo` process" -> "so a static failure, a section without a current accepted assessment among them, starts no `cargo` process". This is the real order of `Lifecycle::complete` (`evaluate`: conditions 1-4, 6 and the after-removal check, no early exit; then `run_bound`; then `gate.open()` and a second `evaluate` before `write`). The separate PROPOSED Order paragraph is merged in and gone.
4. "the repository before the removal is judged by the five conditions only" -> "by the six conditions only".
5. Obligations by effect: "All five conditions apply to every change." -> "All six conditions"; the condition-6 sentence of the PROPOSED block appended.
6. Verification: "A binding fixture with a stub named by `$CARGO` (and one on `PATH` when `$CARGO` is unset) records each argv and asserts it equals the matching row of the invocation table with `--frozen`, `--package=` and the pinned target, and that no shell is involved." -> "A binding fixture runs each row of the invocation table against a stub that the library is given as the program `$CARGO` names; the stub records each argv, and the fixture asserts it equals the row, with `--frozen`, `--package=` and the pinned target, and with `Wrapper<T>::get` arriving as one word, so no shell is involved. With `$CARGO` unset the program is `cargo`, and a stub named `cargo` in the directories searched as `PATH` is the one that runs." Reason: `exec::stubs::every_row_reaches_a_process_as_exactly_its_argv` and `a_stub_named_by_cargo_runs_for_cargo_and_a_stub_on_the_path_when_it_is_unset` give the stub through `Programs::with_cargo_var` and the bare-name lookup through `SystemRunner::search_path`, not the process `PATH`. Every assertion of the old sentence stays.
   Proposed text reworded on resolution: the generated-only sentence "A section with no text outside its generated blocks and comments would hold nothing an author wrote, but the profile refuses such a section ..." -> "A section with nothing outside its generated blocks and comments cannot occur: the profile refuses such a section (`okf-section-empty`), so its document does not load and `complete` is an error before any condition is judged, and every section of a capability that loads is authored" (same wording as reconcile); "each refuse" -> "each refuse condition 6"; Verification "Scenario B gains the step" -> "Scenario B holds the step"; the shared-helper bullet adds "a section whose assessment cannot be recorded gets none, so what it owes stays visible" (what `tests/support/assessed.rs` does). The condition-6 refusal example is placed after the fifth-condition example, its fence a normal block.
   Removed: nothing.

`cap:workplan-cli`
1. Contract, command table: the eight rows of the record commands are rows of the one table; the PROPOSED lead "Four more commands read and write the records of the workplan" is gone. The value-checking paragraph (now headed **Values of the record commands.**) and **The command of `evidence record`.** follow the per-command paragraph; **Exit codes of the record commands.** follows **Exit codes.**; **Output of the record commands.** follows **JSON.**
2. Exit codes of the record commands: "(for `assess --list` with a section too)" -> a sentence: a section given to `assess --list` must be one `assess --context` answers for, so a named section that can no longer be assessed is exit 2 while a bare `--list` still shows an assessment of it (carried reviewer advisory A8; `assess.rs` runs `assess_context` on the subject).
3. Constraints: "...and then in one of three forms: after `--`; inside a single `<rev>:<path>` or `-L<start>,<end>:<path>` argument; or as a validated test filter" -> "four forms: ...; as a validated identifier inside one `-S<identifier>` argument of the history; or as a validated test filter" (superseded by the PROPOSED fourth-form sentence). The PROPOSED "The binary still starts no process of its own." -> "The binary starts no process of its own: each process is started through `raf-workplan`." ("still" is change-relative). The write bullet of the record commands follows the delivered write bullet.
4. Usage: the PROPOSED bullets gain the lead "The record commands are used the same way:".
5. Verification: delivered bullets unchanged. Proposed "A stub-recorded fixture asserting that each such refusal starts no `git` and no `cargo` ..." -> "A fixture runs the binary with recording stubs of `git` and `cargo` first on `PATH` and `$CARGO` naming the `cargo` stub, and asserts ..." and "... and that `history` asks `git --version` before its log and refuses 2.29" -> "A recording `git` stub on `PATH` that answers the version shows that `history` asks `git --version` before its log, refuses 2.29 and goes on at 2.30" (what `process::an_id_a_day_or_a_value_of_the_record_commands_is_refused_before_any_process_starts` and `process::history_asks_the_git_version_before_its_log_and_refuses_a_git_older_than_2_30` do); "JSON pins of the members of each new kind" -> "of each kind of the record commands".
6. `AGENTS.md` "their contract still proposed in the command-line capability" -> "their contract stated in the command-line capability"; `README.md` "their contract is still proposed in [workplan-cli]" -> "their contract is stated in [workplan-cli]" (both false once the blocks are resolved; the commands stay **implementing**).
   Removed: nothing.

### Record items

All 19 items (workplan-reconcile 7, workplan-complete-gate 4, workplan-cli 8) are `implemented`, each checked against the code: `obligation::trace::raise` and `trace::assessments` (claim-unsupported at the claim line for a missing, failing or over-limit record; evidence-stale per moved or unresolved symbol input, no git); `Lifecycle::process_obligations` used by `Lifecycle::reconcile` (`session.rs`) and `Lifecycle::complete` (`complete.rs`), with `lifecycle::changed` reading each record at the merge base; `obligation::suppressed` (only `section-untouched` of three sections under `contract_effect: none`); `Condition::Assessments` (number 6) with `Failure::Assessment` from `assessment_failures` inside `evaluate`, judged before `run_bound` and again before `write`, count label "section(s) without a current accepted assessment"; the cli `history`, `evidence`, `assess` and `coverage` modules, the JSON kinds in `cargo-workplan` `main.rs`, `raf_workplan::root` in `bindings.code`, and `coverage tests --crate cargo-workplan` empty at Task 10 with no code change since. No item reworded, none moved, none dropped; no new record created. The earlier rewordings ("exactly one test ran", doc tests run by the bare item path) are delivered complete-gate text that matches `bound.rs`; none of the three records restates them. `changes` lists the one capability of each record. `affects`: workplan-reconcile gains `#constraints` (edited); workplan-complete-gate (contract, examples, verification) and workplan-cli (contract, usage, examples, constraints, verification) already listed every edited section (the frontmatter `description` is no section).

### Reconcile at HEAD `1a5b28e` (each exit 1; the only lines are `assessment-missing`, recorded by a later part once all text is final)

```text
### workplan-reconcile
cap:workplan-reconcile  contract  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-reconcile#contract` h
cap:workplan-reconcile  constraints  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-reconcile#constrai
cap:workplan-reconcile  verification  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-reconcile#verific
exit 1
### workplan-complete-gate
cap:workplan-complete-gate  contract  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-complete-gate#con
cap:workplan-complete-gate  examples  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-complete-gate#exa
cap:workplan-complete-gate  verification  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-complete-gate
exit 1
### workplan-cli
cap:workplan-cli  contract  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-cli#contract` has no assess
cap:workplan-cli  usage  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-cli#usage` has no assessment `
cap:workplan-cli  examples  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-cli#examples` has no assess
cap:workplan-cli  constraints  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-cli#constraints` has no 
cap:workplan-cli  verification  assessment-missing  an assessment of the section recorded with assess (`cap:workplan-cli#verification` has n
exit 1
```

The claim `gate-kills-a-test-past-its-timeout` in the complete-gate Contract stays supported: no `claim-unsupported`, `evidence-stale` or `evidence-changed` line.

### Gates at HEAD `1a5b28e`

| Gate | Exit | Result |
| --- | --- | --- |
| `make docs-sync` | 0 | nothing rewritten after the complete-gate commit |
| `cargo run -q -p cargo-workplan -- check` | 0 | no finding |
| `cargo test --locked --test docs_contract --test repo_gates` | 0 | 28 and 57 passed (also in each pre-commit hook) |
| `cargo test -p raf-workplan --lib -- obligation::tests` | 0 | 21 passed (the evidence-column pin among them) |
| `make reuse` | 0 | compliant |
| `make issue-refs` | 0 | 57 passed |
| `docs preview workplan-complete-gate`, `docs preview workplan-cli` | 0, 0 | the fenced block inside the former PROPOSED quote no longer stops the preview |

Crate tests not rerun: no code changed (the bound tests ran in `verify`). The `tests/roadmap.rs` guard is left for the final part. Not run here: `change complete`, any `delivered` state edit, any assessment.

## Task 11C: resolve the PROPOSED blocks of workplan-entity-history, workplan-evidence-claims, workplan-prose-assessment, workplan-two-way-coverage, workplan-current-rationale

Commits (each through the pre-commit hook: fmt, reuse, docs_contract 28 passed, repo_gates 57 passed; each pushed):

| Commit | Capability | Files |
| --- | --- | --- |
| `74783a0` | `cap:workplan-entity-history` | `docs/capabilities/workplan-entity-history.md`, `work/changes/workplan-entity-history.md`, `docs/capabilities/index.md` (catalogue row regenerated: the `bindings.examples is empty` note gone) |
| `5536582` | `cap:workplan-evidence-claims` | `docs/capabilities/workplan-evidence-claims.md`, `work/changes/workplan-evidence-claims.md`, `docs/capabilities/index.md` (catalogue note gone) |
| `180ec00` | `cap:workplan-prose-assessment` | `docs/capabilities/workplan-prose-assessment.md`, `work/changes/workplan-prose-assessment.md`, `docs/capabilities/index.md` (catalogue note gone) |
| `6a9c116` | `cap:workplan-two-way-coverage` | `docs/capabilities/workplan-two-way-coverage.md`, `work/changes/workplan-two-way-coverage.md`, `docs/capabilities/index.md` (catalogue note gone) |
| `d6f8804` | `cap:workplan-current-rationale` | `docs/capabilities/workplan-current-rationale.md`, `work/changes/workplan-current-rationale.md`, `docs/capabilities/index.md` (catalogue note gone), `crates/raf-workplan/src/rationale.rs` (a module documentation test, no code change) |

### Sentences removed or reworded (for the PR body)

`cap:workplan-entity-history`
1. Purpose: the "Open questions:" list (three questions) is removed; the text answers each: one timeline (a commit several sources list is one entry), a move of the document followed by `git log --follow` on its path, and an entry shows commit, date, subject and sources. The Purpose prose stays. `affects` gains `cap:workplan-entity-history#purpose`.
2. Contract, Document history: "a merge that changed nothing in it is not an entry" -> "a merge whose result equals one of its parents is not an entry" (carried reviewer edit: a `-s ours` merge changes the document relative to one parent and is still not listed; under `--cc` a merge is path-less exactly when its result equals one parent).
3. Contract: every other PROPOSED sentence is the current contract unchanged, the fenced `text` block of the three git rows now a normal fenced block (`docs preview workplan-entity-history` exits 0).
4. Constraints: one bullet added: the paths of an entry have two bases when the workspace lies below the top of its work tree (document names from `git log --name-only` relative to the top, or to the workspace root under `diff.relative=true`, checked with real git; symbol files relative to the workspace root). The partial-clone lazy fetch sentence was already in the Constraints and stays.
5. Verification: "are still found" -> "are found"; "a stub `git` on `PATH` records ..." -> "A recording stub `git`, given to the library as the program it starts (`Programs::with_git`), shows ..." (what `history::stubs::a_path_named_like_an_option_reaches_git_after_the_separator_or_the_colon` does); "a stub `git --version` reporting 2.29 exits 2 before any history is read" -> the library stub stops the history before any log, a scripted runner shows the same for 2.29 and a shallow answer, and through the binary a recording `git` stub first on `PATH` makes `history` exit 2 (the `process::history_asks_the_git_version_...` fixture of `cap:workplan-cli`); "an `<id>` or `--since` value that begins with `-` or does not match its pattern is a usage error with exit 2" -> the library refuses an id of another kind or one no document has and a bad `--since`, with no `git` started, and the command line reports each as exit 2 (exit codes belong to the binary); "a merge that changed nothing in it is not" -> "a clean merge, whose result equals one of its parents, is not"; "A shallow clone: the command fails" -> "the history fails ..., and a full clone of the same origin answers". Added bullets for bound tests the section did not describe: the order fixture (`every_question_is_asked_in_order_as_exactly_its_argv`, `since_reaches_both_logs_before_the_last_separator`, `since_limits_the_logs_and_not_the_removal_lookup`), the binding refusal (`a_binding_that_names_no_single_symbol_is_refused_before_any_process`), each id kind (`each_kind_of_id_has_the_history_of_its_file`) and the documentation example.
   Removed: the Open questions list only.

### Record items

`change:workplan-entity-history`: all 7 items `implemented`. Reworded to what is built: item 3 "listed with the last commit in which it resolved" -> "listed with the commit that removed it, the newest commit that changed how often its last path segment occurs in its crate's directory, and that commit's parent as the last commit in which it resolved" (D11; `history::RemovedBinding`, `git::last_change`); item 4 "A repository whose history is incomplete" -> "A shallow clone, whose history is incomplete, produces an error that names the missing history and no partial list" (`HistoryError::Shallow`; a partial clone fetches its objects and is not incomplete); item 6 "with a path after --" -> "with a path after -- or inside the single -L argument" (the symbol row). Nothing unbuilt, so nothing moved.

### Bindings

`cap:workplan-entity-history`: `bindings.examples` was empty (`binding-missing`); it binds the runnable documentation test `raf_workplan::Workplan::history` (an id no document has is refused before any process starts), and Verification names it.

`cap:workplan-evidence-claims`
1. Purpose: the "Open questions:" list (three questions) is removed; the text answers each: one directory, `work/evidence/<id>.md`; the verdict with the measured values and limits; a claim may name several records and holds only while every one holds. `affects` gains `cap:workplan-evidence-claims#purpose`.
2. Contract, Claims: "The document names the evidence a claim rests on, so the dependency is stated in the text and not inferred." -> the same sentence continued with the claim marker `<!-- raf:claim evidence=<id>[,<id>...] -->` (in a code span, so it is not read as a marker), a link to the marker syntax of the OKF profile page, "a claim may rest on several records and holds only while every one of them holds", "a marker inside a PROPOSED block is a claim once the block is resolved", and the three `check` findings `claim-evidence-missing`, `claim-evidence-not-for`, `evidence-for-missing` (what `finding.rs` and the record rules raise). The claim marker is stated, as the instruction to keep it asked.
3. Contract, Invalidation: "a record one of whose input symbols has another digest now than the one it keeps is reported as stale" -> "..., or no longer resolves, is reported as stale" (widened to what `obligation::trace` raises).
4. Examples: the claim-less table rows "Statement | Parsing a 10 MiB input ..." and "Evidence id | `memory-bound`" (a duplicate of the claim without its marker) -> a fenced `markdown` block holding the statement and its claim marker (the example of the OKF profile page); the "Record" row -> rows Subject, Verdict, Values, Limits, Inputs (what an `EvidenceRecord` holds). "Re-recording the evidence with a printed `peak=80` records the verdict fail and makes the statement an obligation: the bound is still documented and no longer supported." -> "Recording the evidence again with `--update` when the command prints `peak=80` records the verdict fail, and `change reconcile` lists the statement as `claim-unsupported`, and as `evidence-changed` when the record passed at the merge base: the bound is documented and no longer supported." (a replacement needs `--update`; the obligation kinds named).
5. Constraints: removed the bullet "Every `name=number` line of the output is stored, not only those a limit names, up to 1024 values; more is an error." (a duplicate of the Contract sentences "Every `name=number` line is stored as a value, up to 1024 values" and "more than 1024 values are printed" -> exit 2). The staleness bullet "Evidence is stale when the digest of one of its input symbols differs from the digest the record keeps. Staleness reads no revision ...; the revision a record states is for its readers." -> "Staleness reads no revision and asks Git nothing, so it holds through squash merges and shallow clones (the decision below gives the reasons)." (the first and last clauses duplicated the Contract and the Explanation `### Decision`, which stays with its `### Conditions to revisit`).
6. Verification: "Removing a record reports the claim as unsupported." -> "..., and as changed against the merge base." (D6); "A claim that names an evidence id that does not exist is an error naming the file." -> "is a finding of `check` naming the file and the line of the marker" (`claim-evidence-missing`, as Task 10 showed). Added bullets for bound tests the section did not describe: the float-precision limit (`a_claim_on_a_value_over_its_limit_by_less_than_a_float_tells_apart_is_unsupported`, `evidence::tests::a_value_over_its_limit_by_less_than_a_float_tells_apart_records_fail`), the for list without a claim and the view refusals, the hostile record text, `--from-binding`, every refusal of the request, replacement, 1024 values, the output bound, the limit parser, the trusted argv tests, and the two documentation examples.
   Removed: the Open questions list, the duplicate 1024-values bullet and the duplicated clauses of the staleness bullet.

`change:workplan-evidence-claims`: all 5 items `implemented`. Reworded to what is built: item 1 "evidence record persists the command, revision, verdict and values of an execution." -> "evidence record runs the command after -- or a bound test or example and persists the source of its verdict (the command text or the binding), the revision when one is given, the verdict, the measured values, the limits and the digests of its inputs." (a binding source has no command; the revision is optional; D4); item 4 "A changed or removed record gives each dependent claim an obligation." -> "A record that passed at the merge base and now fails, holds a value over a limit, has lost a limit, or is gone gives each claim that names it an evidence-changed obligation." (a record whose values change within its limits gives none; `EvidenceChange`); item 5 "A claim without evidence is reported as unsupported." -> "A claim whose evidence does not exist, fails or holds a value over its limit is reported as unsupported." (`trace::raise`). Nothing unbuilt, so nothing moved.

Bindings: `bindings.examples` was empty; it binds the runnable documentation tests `raf_workplan::Workplan::record_evidence` (an unknown capability is refused with no process started and nothing written) and `raf_workplan::evidence::parse_limit` (rustdoc names it `evidence::measure::parse_limit`; the executor selects it: "34 listed, 1 selected, 1 ok lines").

`cap:workplan-prose-assessment`
1. Purpose: the "Open questions:" list is removed; the text answers each: the verdict is `accepted` or `rejected` and no other (a sentence added to the Contract), an assessment covers a whole section, and an agent assessment satisfies the gate by itself (The gate). `affects` gains `cap:workplan-prose-assessment#purpose`.
2. Contract, Mechanical and authored text: "The profile refuses such a section in a capability document, so every section of a capability that loads is authored and may be assessed." -> "Such a section cannot occur in a capability that loads: the profile refuses it (`okf-section-empty`), so every section of a capability that loads is authored and may be assessed." (aligned with the reconcile and complete-gate wording of Task 11B); added "The verdict is `accepted` or `rejected`, and no other." The strong/weak naming rule stays in the Contract as written (strong: a whole path or a `::`-suffix of two segments or more; weak: the last segment alone; a suffix two bindings end in names both; a weak name only adds). The record names the assessor by kind only (`by: human|agent`), as the record item says.
3. Examples: the table "Update the usage example | The final API differs from the stated API in two places." / "Update the lifecycle explanation | ..." / "Support the memory bound | The documented bound has no current evidence." (obligation names no command prints) -> a fenced `text` block of three `change reconcile` lines in the exact form the tool prints (`assessment-stale` with the moved symbol, `assessment-missing`, `assessment-rejected` by human), with the lead "`change reconcile` lists each authored section in `affects` that owes an assessment, with the input that moved or the verdict that stands in the way". "An assessment that moved is listed with the input" -> "`assess --list` shows an assessment that moved with the input" (the line shown is the `--list` form of `AssessmentState`). The fenced block of that line is a normal fenced block (`docs preview workplan-prose-assessment` exits 0).
4. Constraints: "a bound symbol it describes only in prose, or reaches through another path such as a re-export, is not an input" -> "a bound symbol it describes only in prose or in fenced code, or reaches ..., is not an input" (carried reviewer edit: code spans in fenced code are not read).
5. Verification: "For generated sections and bindings, adding code spans that hold only last segments never removes an input." -> "Over sections and bindings that the test generates, adding code spans that hold only last segments never removes an input, and a weak name never ends the rule that, with no strong name, every bound symbol is an input." (`widening_a_section_with_weak_names_never_removes_an_input`, `a_weak_name_adds_an_input_and_never_ends_the_every_binding_rule`). Added bullets for bound tests the section did not describe (subject and record slug, a capability that does not load, authored text, a moved input's message) and the two documentation examples.
   Removed: the Open questions list and the invented obligation table.

`change:workplan-prose-assessment`: all 6 items `implemented`. Reworded to what is built: item 4 "An assessment is invalidated when the section, a bound symbol or depended-on evidence changes." -> "... when the section, a bound symbol that is one of its inputs or an evidence record a claim of the section names changes, or when an input is gone or new." (a bound symbol the section does not name, once it names one strongly, is no input and moves nothing: `a_symbol_the_section_names_is_an_input_and_a_bound_symbol_it_does_not_name_is_not`); item 6 "requires current assessments" -> "requires a current accepted assessment for each affected authored section" (condition 6; a rejected one never satisfies it). The item with the assessor named by kind only was reworded in Task 6 and is unchanged. Nothing unbuilt, so nothing moved.

Bindings: `bindings.examples` was empty; it binds the runnable documentation tests `raf_workplan::Workplan::assess_context` and `raf_workplan::assessment::record_slug`.

`cap:workplan-two-way-coverage`
1. Purpose: the "Open questions:" list is removed; the text answers each: a `pub` item of a private module is public only through a `pub` re-export (The public surface); an unbound test is a gap like an unbound item and a finding for a crate of `scope`, and a test that only guards internals is excluded by its module prefix with the reason; exclusions with reasons live in `work/coverage/default.md`. `affects` gains `cap:workplan-two-way-coverage#purpose`.
2. Contract, The public surface: "so a binary's public surface is empty and the code direction asks nothing of it" -> "so the public surface of a binary target is empty in practice and the code direction asks nothing of it; the tests of a binary package count in the tests direction like those of a library" (carried edit: the module documentation says the surface is usually empty; no fixture pins a `pub` item of a binary, so "in practice").
3. Contract, Covered: "unit tests in `#[cfg(test)]` modules included" -> "unit tests in the `#[cfg(test)]` modules of a library or a binary included, which count like any other test", and one sentence added: "A unit test that only guards internals is a gap like any other until it is bound or its module is excluded with the reason."
4. Contract, The coverage record: the forms stated: `scope` holds package names (`raf-okf`, not `raf_okf`) and an entry that is no package is `scope-crate-missing` naming the package of a crate name (`a_scope_entry_is_a_package_name_and_a_crate_name_says_which_package_it_means`); "the item is an item path ..., or `<module>::*` below a crate root" -> "the item has one of two forms: an item path of two segments or more (`raf_okf::Body`) ... or a prefix `<module>::*` below a crate root (`raf_okf::body::*`)"; "A whole crate is never excluded: it is left out of `scope`." -> "A whole crate is never excluded: an exclusion of a crate root (`raf_okf` or `raf_okf::*`) is refused when the record loads, and the crate is left out of `scope` instead." (`raf_okf` `exclusion_item`: one segment is `CrateRootExclusion`). `coverage-record-missing` and `scope: []` stay as written.
5. Usage: "`scope: []` ... enforces nothing; the reports still name every gap." -> "enforces nothing, and the reports name every gap all the same."
6. Examples: the YAML exclusion example listed only `raf_okf::body::tests::*`, which does not close the `raf_okf::parse_partial` gap the example is about -> it lists `raf_okf::parse_partial` with a reason and then the unit-test prefix, with the lead "as it lists the prefix of a module of unit tests that guard internals". The fenced YAML block is a normal fenced block (`docs preview workplan-two-way-coverage` exits 0).
7. Constraints: "A capability still in `considered` or `designed` binds nothing" -> "A capability in `considered` or `designed` binds nothing" ("still" is change-relative).
8. Verification: added bullets for bound tests the section did not describe (scope entry by package name, exclusion by exported or defining path, the budget error, table and JSON escaped, this repository's two crates covered) and the documentation example `Workplan::coverage_report`.
   Removed: the Open questions list.

`change:workplan-two-way-coverage`: all 5 items `implemented`. Reworded to what is built: item 3 "Exclusions carry a reason, and stale or empty exclusions are errors." -> "Exclusions carry a reason, and an exclusion with an empty reason, of a whole crate, or that matches nothing is an error."; item 5 "check includes both reports and fails on a gap." -> "check enforces both directions for the crates the coverage record lists in scope and fails on a gap there, and a repository with no coverage record has one coverage-record-missing finding." (the owner decision on the record: `check` enforces the scope crates, the reports cover the workspace). Nothing is moved: enforcement of any crate is built (a crate is enforced by listing it in `scope`); which crates this repository lists is the content of its record, which the owner decision fixes at the workplan crates, and widening it is an edit of the record that this change does not claim.

Bindings: `bindings.examples` was empty; it binds the runnable documentation test `raf_workplan::Workplan::coverage_report`.

`cap:workplan-current-rationale`
1. Purpose: the "Open questions:" list is removed; the text answers each: a conditions list is required only on a decision page (an architecture page with `## Decision`), a link counts from any section, and no check recognises a met condition (a person judges it with the history command). `affects` gains `cap:workplan-current-rationale#purpose`.
2. Contract, Conditions to revisit: "A page or an Explanation that records a significant decision ends it with a list of conditions to revisit." -> "A recorded decision carries a list of conditions to revisit, under the headings the decision pages paragraph below names." and "An architecture page that records no decision needs no such list." added (edit list: the decision page and `### Decision` rule, D10; "significant" and "ends it" were not what is checked: the conditions heading may stand anywhere on a decision page).
3. Contract, Headings and lists: two sentences added before the matching rule: the headings are ATX headings (up to three spaces, one to six `#`, a space or a tab and the text), and a setext heading is not a heading of these rules (carried reviewer edit; `raf_okf` `body::heading::atx`).
4. Contract, Checks: "A link counts when it is written outside fenced code, code spans and comments and resolves to the page" -> "A link counts when it is written in any section of the document, outside fenced code, code spans and comments, and resolves to the page" (answers the open question; link edges are read over the whole body).
5. Verification: added bullets for bound tests the section did not describe (exact level match, no unlinked report while a document does not load, the list-item rule, this repository and the valid fixture) and the documentation example.
   Removed: the Open questions list.

`change:workplan-current-rationale`: all 5 items `implemented`. Reworded to what is built: item 3 "A decision page ends with a list of conditions to revisit." -> "A decision page, an architecture page with a ## Decision heading, has a ## Conditions to revisit heading whose list holds at least one item." (`decision_page`: the heading is required, not its place). Nothing unbuilt, so nothing moved.

Bindings: `bindings.examples` was empty and no runnable documentation test exercised the rules except the crate-level example of `lib.rs`, which has no item path to bind. A module documentation test is added to `crates/raf-workplan/src/rationale.rs` (the module bound in `bindings.code`): two linked architecture pages, one a decision page whose conditions list is empty, give exactly one `decision-conditions-missing` through `Workplan::check`, and with one condition the check is empty. It binds as `raf_workplan::rationale` ("35 listed, 1 selected, 1 ok lines"). The `## Decision` heading text of the fixture is passed as a `format!` argument because rustdoc reads a documentation line that starts with `##` as an escaped `#`. Consequence: the module digest moved, so the existing assessment `assessment:workplan-current-rationale-explanation` (input `raf_workplan::rationale`) is stale (`assess --list`: "stale: the symbol `raf_workplan::rationale` has changed since"); the Explanation is not in `affects`, so it is no obligation, and the assessments of the later part record it again.

### Verify (`cargo run -q -p cargo-workplan -- --root <checkout> verify <slug>`, run one after another, each on a clean detached checkout of the commit that resolved the capability, `.worktrees/verify-11c`, so editing the next document could not touch a run; wall time includes the incremental build)

| Capability | Commit | Exit | Bindings | Wall time |
| --- | --- | --- | --- | --- |
| `workplan-entity-history` | `74783a0` | 0 | 30 pass (29 tests, 1 example), 0 fail | 72 s |
| `workplan-evidence-claims` | `5536582` | 0 | 34 pass (32 tests, 2 examples), 0 fail | 35 s |
| `workplan-prose-assessment` | `180ec00` | 0 | 28 pass (26 tests, 2 examples), 0 fail | 44 s |
| `workplan-two-way-coverage` | `6a9c116` | 0 | 23 pass (22 tests, 1 example), 0 fail | 31 s |
| `workplan-current-rationale` | `d6f8804` | 0 | 17 pass (16 tests, 1 example), 0 fail | 48 s |

Each example line reads "N listed, 1 selected, 1 ok lines", so every bound example is a runnable documentation test (no `no_run`, no `compile_fail`), and every bound test line reads "1 listed, 1 passed": all run on Linux. Each capability binds code and tests. The evidence-claims and current-rationale bindings were also verified on the working tree before their commits (exit 0, 34 and 17 pass), to confirm that the re-exported `evidence::parse_limit` (rustdoc names it `evidence::measure::parse_limit`) and the private-module test `rationale` are selected.

### Reconcile at HEAD `d6f8804` (each exit 1; the only lines are `assessment-missing`, recorded by a later part once all text is final)

* `workplan-entity-history`: `assessment-missing` for purpose, contract, usage, examples, constraints, verification (6 lines; the two `section-untouched` lines of usage and examples and the `binding-missing` of examples before this part are gone).
* `workplan-evidence-claims`: `assessment-missing` for purpose, contract, usage, examples, constraints, explanation, verification (7 lines).
* `workplan-prose-assessment`: `assessment-missing` for purpose, contract, usage, examples, constraints, verification (6 lines).
* `workplan-two-way-coverage`: `assessment-missing` for purpose, contract, usage, examples, constraints, verification (6 lines).
* `workplan-current-rationale`: `assessment-missing` for purpose, contract, usage, examples, constraints, verification (6 lines).
* `workplan-complete-gate` (the record whose Contract holds the claim): the three `assessment-missing` lines of Task 11B only; the claim `gate-kills-a-test-past-its-timeout` stays supported (no `claim-unsupported`, `evidence-stale` or `evidence-changed`).

No `unresolved-proposal`, `binding-missing`, `section-untouched`, `item-without-outcome`, `tense-mismatch`, trace or evidence line remains in any of the five. `grep -c 'PROPOSED change:'` is 0 in each of the five documents and `grep -i 'open questions'` finds nothing; `docs preview <slug>` exits 0 for each (the three fenced blocks that sat inside PROPOSED quotes are normal fenced blocks). Tense scan of the five bodies (`will`, `is planned`, `proposed`, `not yet`, `to be implemented`, `open questions`) outside code spans: no hit except the marker name `PROPOSED block` (evidence-claims Contract, prose-assessment Contract, current-rationale Contract and Verification) and a bound test path holding `proposed_blocks`.

### Gates at HEAD `d6f8804`

| Gate | Exit | Result |
| --- | --- | --- |
| `make docs-sync` | 0 | nothing rewritten |
| `cargo run -q -p cargo-workplan -- check` | 0 | no finding |
| `cargo test --locked --test docs_contract --test repo_gates` | 0 | 28 and 57 passed (also in each pre-commit hook) |
| `cargo test -p raf-workplan --doc` | 0 | 35 passed (the new `rationale` module test among them) |
| `cargo test -p raf-workplan --test rationale` | 0 | 14 passed |
| `make fmt`, `make clippy`, `make doc` | 0 | clean (`rationale.rs` changed in a documentation comment only) |
| `make reuse` | 0 | compliant |
| `make issue-refs` | 0 | 57 passed |

Not run here: `change complete`, any `delivered` state edit, any assessment, and no edit of `tests/roadmap.rs`. For the later part: the assessment `assessment:workplan-current-rationale-explanation` is stale (its `raf_workplan::rationale` input moved with the module documentation test) and is recorded again with the others.

## Task 11D1: assess the affected sections of okf-profile, source-index, workplan-document-model, workplan-generated-docs

Each section was read in full through `assess cap:<name>#<section> --context` (section text, bound symbols with their source, evidence), checked against the code and the bound tests, and recorded with `assess ... --verdict accepted --by agent --copyright '2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>' --licence CC-BY-4.0`. No git command that changes state was run.

### okf-profile

| Section | Verdict | Text fixes |
| --- | --- | --- |
| `cap:okf-profile#contract` | accepted | 1 |
| `cap:okf-profile#verification` | accepted | none |

Fix 1 (Contract, Body model): "So are block quotes and list items nested over 64 levels, a malformed claim marker, a claim marker with no statement directly above it, and a `raf:` comment that is neither ..." -> "So are block quotes and list items nested over 64 levels and, outside fenced code, code spans and PROPOSED blocks, a malformed claim marker, a claim marker with no statement directly above it, and a `raf:` comment that is neither ...". Why: `body/marker.rs` `scan_markers` calls `raf_marker` only for lines that are not fenced and not in a PROPOSED block, and `raf_comment` removes code spans first; `tests/claims.rs` `markers_in_fences_code_spans_and_proposed_blocks_are_not_read` parses `<!-- raf:anything -->` in a fence, `<!-- raf:note -->` in a code span and `<!-- raf:claim evidence=Bad -->` in a PROPOSED block without error. The unqualified sentence said those are parse errors.

Checked and true as written: the bounds (`limits.rs` 1 MiB, 32, 64; `link.rs` `MAX_LINK_PART` 4096), `DocType::ALL` of six, the record rules of `profile/records.rs` (64 lowercase hex digest; revision 40 or 64 hex; evidence inputs `symbol` only via `input_kind_allowed`; `command` exactly when the subject is `command`; a limit of a failing record may name no value; scope package names distinct; a one-segment exclusion, with or without `::*`, is `CrateRootExclusion`), the exhaustive enums (no `#[non_exhaustive]` on `DocType`, `InputKind`, `Actor`, `EvidenceVerdict`, `AssessmentVerdict`, `EvidenceSubject`, `ExclusionItem`, `Outcome`, `LinkTarget`), the body rules of `body_rules.rs` (record body empty, `ClaimOutsideCapability`), and the writer of `profile/write.rs` (numbers plain, empty list `[]`, absent measures and optional keys left out). Verification: each bullet matches a test of `crates/raf-okf/tests` (the 100 ms `RELEASE_BOUND` of `validate_cost.rs`, the eight-times allocation bound, the record fixtures each beside an accepted neighbour, `a_raf_generated_comment_keeps_its_own_rules`). Note: the fixtures `claims::a_claim_may_rest_on_several_records` and `claims::a_claim_after_a_list_item_supports_that_item_only` that the "with several ... after a list item" clause describes exist and run in `cargo test`, but are not in `bindings.tests` (a binding is not section text; not changed here).

`change reconcile okf-profile`: no output, exit 0.

### source-index

| Section | Verdict | Text fixes |
| --- | --- | --- |
| `cap:source-index#contract` | accepted | 2 |
| `cap:source-index#usage` | accepted | none |
| `cap:source-index#examples` | accepted | none |
| `cap:source-index#constraints` | accepted | 1 |
| `cap:source-index#verification` | accepted | none |

Fix 2 (Contract, API surface): "...are `#[non_exhaustive]` like `Item`, so a later field is not a breaking change" -> "...so adding a field is not a breaking change", and "so a later option that is not `Copy` is not a breaking change" -> "so adding an option that is not `Copy` is not a breaking change". Why: "a later field" reads as forward-looking; the rule is a present property of the types (`#[non_exhaustive]` on `Package`, `Target`, `SourceFile`, `Reach`, `Entry`, `SpanFact`; `LoadOptions` `Clone` and not `Copy`). Same meaning.

Fix 3 (Contract, Use-alias index): "...an `extern crate std;` at the root adds it back. The import binds nothing and keeps its name indeterminate" -> "... The ambiguous import binds nothing and keeps its name indeterminate". Why: the sysroot sentence stands between the E0659 ambiguity sentence and "The import", so "The import" had no clear referent; the sentence describes the ambiguous `use` head of the sentence before.

Fix 4 (Constraints): "Every opened path must be, after canonicalisation, inside the canonical workspace root and a regular file" -> "Every opened path must lead inside the canonical workspace root, as the disk resolves it (canonicalised on Linux and Windows, walked beneath the root on other Unix systems), and every file read must be a regular file". Why: the Contract (Path containment) and `tree/disk/other_unix.rs` walk the path with `openat` on macOS and the BSDs and canonicalise nothing there, and directories are opened for listing, so "a regular file" holds for files read only.

Checked and true as written: the budget table `BUDGETS` of `budget.rs` (128, 64, 256, 128, 256, `RECURSION` = 64 MiB / (2 x 17 KiB) = 1927, module depth 128, module paths 262144, directory depth 128, 2000000 tokens, 256/512/256/512 MiB), `PARSE_BYTES_PER_TOKEN` 1664, `PARSE_STACK_BYTES` 64 MiB, `DEFAULT_MAX_FILE_BYTES` 8 MiB, `MAX_MEMBERS`/`MAX_EXCLUDES` 4096, `pub use syn;`; the Exported items text against `exports.rs` (breadth-first shown queue before the hidden one, items sorted by path then `sort_key`, re-exports by path, file, line; `ExportError` variants; `ExportOptions::include_hidden`; the Cooper-Harvey-Kennedy dominator tree) and `tests/exported_items.rs`; the Usage blocks equal crate doctests 1 and 2 of `lib.rs` (compiled) and the Examples blocks doctests 3 and 4 (run); the Examples prose equals `the_example_of_the_contract_lists_one_item_and_more_through_a_glob_or_a_pub_module` (three surfaces); Verification counts recomputed from the tests: the differential family is 166 cases (3 x 3 x 6 x 2 x 2 less the skipped shapes of `carried_case`), the random differential 3 seeds x 150 = 450, the time bounds 150 ms / 1 s, 500 ms / 4 s, 300 ms / 3 s with a 32 MiB budget, 10,000 within 1 s; the module-path chain `chain(2, 30)` is 2^31 - 1 walks refused by `ModulePaths` within 30 s in release; `this_repository.rs` asserts the `raf-source` surface and the empty `cargo-workplan` binary surface.

`change reconcile source-index`: no output, exit 0.

### workplan-document-model

| Section | Verdict | Text fixes |
| --- | --- | --- |
| `cap:workplan-document-model#contract` | accepted | 2 |
| `cap:workplan-document-model#usage` | accepted | 1 |
| `cap:workplan-document-model#verification` | accepted | none |

Fix 5 (Contract, Digests): "the kind ... keeps the domains apart and the decimal length ends each field." -> "... and the decimal length says where each field ends." Why: the length precedes the text in `<kind> NUL <name> NUL <length> NUL <text>` (`digest.rs` `field`); it delimits the field, it is not its last part.

Fix 6 (Contract, Writing records): "...opened with the two SPDX comment lines of `RecordOptions` when both are given, and refuse, before anything is written, a record whose text would not load with no finding." -> "...when both are given, and refuse, before anything is written, one of the two lines given without the other and a record whose text would not read back as a record with no finding." Why: `Record::text` refuses half an SPDX header (`RecordError::SpdxUnset`), which the Verification states ("half an SPDX header are refused with nothing written") and the Contract did not; "would not load with no finding" read ambiguously; the wording of `RecordError::Invalid` is "does not read back as a record with no finding".

Fix 7 (Usage): "judges links against the paths `git ls-files` lists, and runs the checks" -> "judges links against the paths `git ls-files` lists (every file under the directory when `git` answers that the directory is not inside a work tree, and the caller's list with `Gate::tracked_paths`), and runs the checks". Why: `gate.rs` documents and implements the one fallback (outside a git work tree every file stands for a tracked path) and the `tracked_paths` override, which the documentation example of `Gate` uses; the sentence stated `git ls-files` as the only source.

Checked and true as written: the graph queries (`Graph::covers`, `claims`, `assesses`, `scope`, `inputs`, `inputs_of`, `evidence_of`, `claims_on`, `assessments_of`, `code_of`, `capabilities_of`, `tests_for`, `sections_of`, `linking_sections` in `graph.rs`), the finding kinds of the Records and Checks paragraphs in `finding.rs`, the SHA-256 implemented in `digest.rs` with no dependency (`Cargo.toml` has none), the module digest over the files below it, the roadmap group order in `roadmap.rs`, `record::create`/`record::replace` and `RecordOptions` in `record.rs`, `Gate::workspace` roots `["."]` and `NoCapabilities`. Verification: the NIST vectors of `sha256_gives_the_standard_test_vectors` (empty, `abc`, the 448-bit message, a million `a`), the `sha256sum`-pinned encodings of `tests/digest.rs`, the read-only-directory replace fixture (target unchanged, no temporary file), the size-bound record within its time bound.

`change reconcile workplan-document-model`: no output, exit 0.

## Task 11D2: assess the affected sections of workplan-reconcile, workplan-complete-gate, workplan-cli

Each section was assessed with `assess cap:<name>#<section> --context` (section text, bound symbols with source, inputs) and checked statement by statement against the code and the bound tests (`obligation.rs`, `obligation/sources.rs`, `obligation/trace.rs`, `obligation/tense.rs`, `git.rs`, `exec.rs`, `exec/invocation.rs`, `lifecycle/{create,session,untouched,changed,completion,complete}.rs`, `bound/*`, `cargo-workplan/src/*.rs`, `tests/{obligations,untouched,evidence,complete,lifecycle,scenarios}.rs`, `cargo-workplan/tests/*.rs`), then recorded with `--verdict accepted --by agent --copyright '2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>' --licence CC-BY-4.0`. Cheap commands were run and compared (`status`, `--format json check`, the `history` and `coverage` Usage lines, `docs preview workplan-reconcile`). No section was rejected. Text only; no code changed.

| Section | Verdict | Fixes |
| --- | --- | --- |
| `cap:workplan-reconcile#contract` | accepted | 10 |
| `cap:workplan-reconcile#constraints` | accepted | 2 |
| `cap:workplan-reconcile#verification` | accepted | 2 |
| `cap:workplan-complete-gate#contract` | accepted | 4 |
| `cap:workplan-complete-gate#examples` | accepted | 1 |
| `cap:workplan-complete-gate#verification` | accepted | 3 |
| `cap:workplan-cli#contract` | accepted | 4 |
| `cap:workplan-cli#usage` | accepted | 0 |
| `cap:workplan-cli#examples` | accepted | 0 |
| `cap:workplan-cli#constraints` | accepted | 4 |
| `cap:workplan-cli#verification` | accepted | 0 |

### workplan-reconcile

Contract:
1. `item-without-outcome` row: "An item has no `outcome`" -> "An item has no `outcome`, or its `moved` outcome names no change record or the record itself". Why: `sources::items` raises `OutcomeProblem::MovedToMissing` and `MovedToItself` as the same kind.
2. `binding-missing` row: "... or a binding does not resolve" -> "... or a binding does not resolve or names an item its list does not take". Why: `sources::bindings` raises `BindingWhy::WrongRole` (`wrong_role`).
3. `generated-stale` row: "A generated block differs from what `docs sync` writes" -> "A generated block of a listed capability or of the capability index differs from what `docs sync` writes, or the index does not exist". Why: `stale_blocks` judges only listed capabilities and the index, and raises `SyncProblem::IndexMissing`.
4. `tense-mismatch` row: "Delivered or implementing contract text uses ..." -> "The prose of a listed delivered or implementing capability uses ..., or a listed delivered capability has a PROPOSED block". Why: `wording` scans every section (`wording_is_read_in_every_section_of_an_implementing_capability`), not the Contract only, and only listed capabilities.
5. `evidence-stale` row: "An input of an evidence record that a listed capability's `for` list or claims reach" -> "An input of an evidence record whose `for` lists a listed capability, or that a claim of one names". Why: `for` is a key of the record (`Graph::evidence_of`), not of the capability.
6. Counting: "once for each record the marker names (and, for `evidence-changed`, once for each change)" -> "for each record the marker names: `claim-unsupported` once for each thing wrong with the record (one that fails and holds a value over its limit is owed twice), and `evidence-changed` once for each change". Why: `trace::raise` pushes one obligation per `ClaimProblem` (fails, and each over-limit value).
7. "A removed record is both `claim-unsupported` and `evidence-changed`" -> "A record that passed at the merge base and has been removed since is both ...". Why: `changed::raise` compares only a record that is at the merge base with the verdict `pass`.
8. "..., and a listed capability that does not load is an error." -> "A listed capability that does not load is an error, except a `delivered` one whose only fault is an empty `code` or `tests` list: it owes that `binding-missing` and takes part in no other kind until it loads." Why: `sources::listed` and `load::note_unbound`.
9. "When `HEAD` is the default branch's commit, the merge base is `HEAD` itself and every affected section is reported." -> "..., the merge base is `HEAD` itself, which is not an error: every affected section the working tree has not edited is reported." Why: the working tree is compared, so an uncommitted edit on the default branch is touched (`on_the_default_branch_itself_every_affected_section_is_owed_and_it_is_not_an_error` asserts exactly that).
10. "... each read up to 16 MiB, and a command that writes more or runs past 300 seconds is killed" -> "... each read up to 16 MiB. A command that writes more than its bound or runs past 300 seconds is killed and is an error." Why: `blob_at` uses `Limits::query()` (300 s) with the 1 MiB bound, so the timeout applies to `cat-file blob` too.

Constraints:
1. "`reconcile` and `impact` never write a file in the repository." -> "... never write a document, a record or any other file of the repository, apart from the build output of `example-failing` below."
2. "...besides `git`; it still writes no file in the repository." -> "...besides `git`. Cargo keeps what it builds in its target directory; `reconcile` writes nothing else." Why (both): the `cargo` builds of bound examples write build output; the tool itself writes nothing (`reconcile_writes_nothing*`).

Verification:
1. Assessment-kind bullet: "an edit of the section, of a symbol it names, of a bound symbol when it names none, and of an evidence record a claim of it names each make its assessment `assessment-stale` ..., while a regenerated block and an edit of another section do not" -> "an edit of the section, and a record made against another text of its bound symbol, each make its assessment `assessment-stale` naming that input, while an edit of another section does not; ... Which edits move an input (a symbol the section names, every bound symbol when it names none, an evidence record a claim of it names, and never a regenerated block) is verified by the fixtures of the prose assessment, whose judgement these kinds report." Why: the reconcile fixtures (`assessment_cases` in `tests/obligations.rs`, `contract_effect_none_keeps_the_assessments_and_another_section_is_not_judged`) cover the section, the symbol digest and another section; the other input cases are the `tests/assessments.rs` fixtures of `cap:workplan-prose-assessment`, which judge through the same `judge_section`.
2. A blank line that split the bullet list in two removed (no wording change).

Checked and true as written: the 13 kinds in `ObligationKind::ALL` order with the evidence column pinned by `obligation::tests::the_evidence_strings_are_the_evidence_column_of_the_contract` (only the "Raised when" column was edited); the sort key (subject, part, kind, line, evidence); exit 0/1; the `change new` contract (`is_kebab`, exclusive create, reload and removal, template `> [!PROPOSED change:<slug>]` and one placeholder line, state guidance, the `git add` / `docs sync` steps, `link-broken`); the eight `git` argv rows in order, in the root and the C locale (`untouched::stubs::reconcile_asks_git_exactly_the_commands_of_the_contract_in_order`, including `refs/remotes/origin/--output=x`); Git 2.30 minimum; 1 MiB blob bound; `$CARGO` or `cargo`; `contract_effect: none` (`suppressed`); the tense phrase list and its exclusions (`tense.rs`); the per-kind fixture covering all 13 kinds (`the_cases_cover_every_kind_of_obligation_in_both_directions`).

### workplan-complete-gate

Contract:
1. Condition 2: "a `moved` outcome names a change that exists" -> "names another change that exists". Why: a move to the change being completed is refused (`OutcomeProblem::MovedToItself`; `an_item_without_outcome_and_a_move_to_nowhere_or_to_itself_refuse_condition_two`).
2. "Four patterns validate the placeholders, and a value that fails its pattern is reported as an unresolved binding and is not passed to `cargo`:" -> "Four patterns validate the values the gate handles. A placeholder value that fails its pattern is reported as an unresolved binding and is not passed to `cargo`, and a line of the list output whose file part fails its pattern is not read as an entry:". Why: the fourth pattern judges the doc-test list output, whose failing line `bound/parse.rs` (`doc_entry`) skips.
3. Bounds: "The standard output and the standard error of every process are each read up to 16 MiB." -> "... of every process of a bound run ...". Why: `git cat-file blob` is bounded at 1 MiB and `cargo metadata` at 64 MiB (`exec.rs`); `Limits::build` and `Limits::example` are 16 MiB.
4. The Bounds paragraph rewrapped after fix 3 (no wording change).

Examples:
1. "Each fault prints one line, under the count of its condition." -> "Each fault prints one line, and the headline counts it under each condition that states it: the item without outcome is an obligation and a fault of condition 2, and the stale catalogue an obligation and a fault of condition 4." Why: the headline "2 obligations, 1 item without outcome, 1 stale generated block" counts each fault twice (`completion.rs` `summary`, `is_repeated_in`; `an_item_without_outcome_and_a_stale_block_print_the_counts_of_the_contract`).

Verification:
1. Zero-test bullet: "... is not a `#[test]` runs zero tests, so `complete` refuses" -> "... would run zero tests and is refused before any process starts, so `complete` refuses". Why: `Refusal::NotATest` at planning (`a_function_that_is_not_a_test_is_refused_before_any_process`).
2. Doc-test bullet: "an item path that begins with `-` is rejected by its pattern" -> "an item path that begins with `-` never reaches `cargo`: bound, it does not resolve, and given to the plan, it is refused by its pattern". Why: the fixture `an_item_path_that_begins_with_a_hyphen_never_reaches_cargo` asserts `Refusal::Unresolved`; the pattern refusal is the unit test `a_doc_item_path_is_refused_unless_it_matches_its_pattern`.
3. Scenario B: "which refuses with condition 6 alone" -> "which refuses only for the missing assessments, each an `assessment-missing` obligation and a fault of condition 6". Why: conditions 1 and 6 both report it (`assessments_missing()` in `tests/scenarios.rs`).

Checked and true as written: the six conditions and their order (`evaluate`: 1-4, 6 and the after-removal check without early exit, then `run_bound`, then a second open and evaluation before `write`); the count labels; the five `cargo` argv rows (`exec/invocation.rs`, `tests/exec.rs`); the pattern texts, ref-name rules (1024 bytes, `.lock`) and 40- or 64-digit ids; 1800 s for test passes, 300 s for examples, stdin closed, process group killed; the proof-that-it-ran rules; the claim `gate-kills-a-test-past-its-timeout` (its record passes).

### workplan-cli

Contract:
1. Root discovery: "When `--root` is absent, the binary spawns exactly this command" -> "When `--root` is absent, root discovery in `raf-workplan` spawns exactly this command". Why: `command::workspace_root` calls `root::discover` with a `SystemRunner`; the binary starts no process of its own (Constraints).
2. `status` row: "Print the roadmap: planned, implementing with gaps, delivered" -> "Print the roadmap: the capabilities of each state, the gaps of each implementing one, and the open changes". Why: "planned" is no state; a real `status` run prints delivered, implementing, designed, considered and the open-changes row.
3. "... is a usage error, ..., and so is a process whose program name is `--`" -> "... and a process whose program name is `--` is refused with exit 2 before anything starts". Why: that refusal is a `Failure` of `evidence::line`, not a parser usage error; the crate docs of `main.rs` list it apart.
4. The `--by` sentence rewrapped (no wording change).

Constraints:
1. "`check`, `status`, `docs preview`, `change impact` and `change reconcile` write nothing." -> "... write nothing in the repository; a bound example that `change reconcile` builds leaves only the build output of cargo." Why: `example-failing` builds.
2. "Each spec that spawns lists its exact invocation, and runs `cargo` with `--frozen`." -> "..., and every `cargo` command the tool composes carries `--frozen`." Why: the caller's own `evidence record` command (such as `cargo bench`) runs exactly as given.
3. "A slug starts with a lowercase letter or digit; any other slug is a usage error." -> "A slug is lowercase letters and digits in parts joined by single `-`, so it starts with a letter or a digit; any other slug, one that starts with `-` included, is a usage error." Why: `cli::slug` checks the whole kebab-case pattern.
4. "A command that takes a slug fails with exit 2 when `work/changes/<slug>.md` does not exist, except `change new`." -> "A command that takes the slug of a change record (`docs preview`, `change impact`, `change reconcile`, `change complete` and `verify <slug>`) fails with exit 2 when `work/changes/<slug>.md` does not exist; `change new` fails with exit 2 when it exists." Why: `evidence record --id` takes a slug that is no change record; `new::a_second_run_for_the_same_slug_is_exit_two_and_leaves_the_record_as_it_is`.

Checked and true as written: the 11 non-rule JSON kinds and the 11 record-command JSON kinds; the history id prefixes, `--since` 1970-01-01 to 2099-12-31, `--revision` of 40 or 64 lowercase hex digits, `--limit` without exponent, both `--crate` forms; root discovery reading only `workspace_root` within 64 MiB; the `-L`, `-S` and `--` argv forms; the record-command exit codes; the `evidence record` example table (matches `tests/evidence.rs`); the status row format and the `link-broken` message and fix; each Verification bullet against its bound test.

Found outside these documents (code documentation, not edited here: no code edits in this part): the crate documentation of `crates/cargo-workplan/src/main.rs`, which the cli Contract names as the reference for the JSON fields, describes the `refusal` object with `condition` "`obligations`, `outcomes`, `proposals`, `generated-blocks`, `bound-runs`, `after-removal` or `changed-during-run`" and `number` "1 to 5", while `Condition::name` also gives `assessments` and `Condition::number` gives 6 for it (`lifecycle/completion.rs`). That doc comment needs `assessments` and "1 to 6".

### Result

Assessments recorded: 13 (11 sections, plus 2 re-records: `cap:workplan-complete-gate#contract` and `cap:workplan-cli#contract`, after the rewrap of a line each staled the first records). New files: `work/assessments/workplan-reconcile-{contract,constraints,verification}.md`, `work/assessments/workplan-complete-gate-{contract,examples,verification}.md`, `work/assessments/workplan-cli-{contract,usage,examples,constraints,verification}.md`; each `assess --list` line reads `accepted  agent  current`.

| Gate | Exit | Result |
| --- | --- | --- |
| `change reconcile workplan-reconcile` | 0 | no output |
| `change reconcile workplan-complete-gate` | 0 | no output |
| `change reconcile workplan-cli` | 0 | no output |
| `cargo run -q -p cargo-workplan -- --root <worktree> check` | 0 | no finding |
| `docs preview workplan-reconcile` | 0 | |
| `make docs-sync` | 0 | `git diff` and `git status --porcelain` identical before and after |

### workplan-generated-docs

| Section | Verdict | Text fixes |
| --- | --- | --- |
| `cap:workplan-generated-docs#verification` | accepted | 1 |

Fix 8 (Verification, line-ending bullet): "The line ending of a document is its first break, an LF frontmatter over a CRLF body gets CRLF blocks" -> "The line ending of a body is its first break, so a CRLF frontmatter over an LF body gets LF blocks and an LF frontmatter over a CRLF body gets CRLF blocks". Why: the Contract says a block takes the line ending of the body, read from the body's first break, and the bound fixture `a_crlf_frontmatter_over_an_lf_body_gets_lf_blocks_and_stays_clean` shows the document's first break (in the frontmatter) does not decide it; "of a document" was false for that fixture.

Checked and true as written: every other clause matches a bound test of `tests/generated.rs`, `tests/sync.rs`, `tests/preview.rs` or `generated::tests` (the code-span fence, the capital state heading, `line_ending`), and the `docs_sync` refusals of `the_docs_sync_example_in_a_git_checkout_and_with_bad_arguments` (`--copyright` without a value, `--frobnicate`, an extra argument: exit 2, a `docs_sync:` message, empty standard output).

`change reconcile workplan-generated-docs`: no output, exit 0.

### Totals and gates

* Assessments recorded: 11, all `accepted`, all by `agent`, none rejected (okf-profile 2, source-index 5, workplan-document-model 3, workplan-generated-docs 1). `assess --list` shows each `current`. No assessment was re-recorded: every text fix was made before the assessment of its section.
* Text fixes: 8 (okf-profile Contract 1; source-index Contract 2, Constraints 1; workplan-document-model Contract 2, Usage 1; workplan-generated-docs Verification 1). No contract was narrowed; no binding, code or change record was edited.
* `change reconcile <slug>` for each of the four: no output, exit 0.
* `cargo run -q -p cargo-workplan -- --root <worktree> check`: exit 0, no finding.
* `make docs-sync`: exit 0; `docs/capabilities/index.md` unchanged and no generated block rewritten (`check` reports no stale block).
* Observation, not changed (a binding is not section text): `raf_okf` `tests/claims.rs` `a_claim_may_rest_on_several_records` and `a_claim_after_a_list_item_supports_that_item_only` back the okf-profile Verification clause "with one record and with several, after a paragraph and after a list item" and run under `cargo test`, but are not listed in `bindings.tests` of `cap:okf-profile`.

## Task 11D3: assess the affected sections of workplan-entity-history, workplan-evidence-claims, workplan-prose-assessment, workplan-two-way-coverage, workplan-current-rationale

Each section was read in full through `assess cap:<slug>#<section> --context` and checked against the bound code and tests, with the cheap commands run on this repository and their output compared with the text. That included `history cap:workplan-reconcile` with and without `--since`, `history arch:normal-form`, `history cap:nope` and a bad `--since` (each exit 2), `evidence claims gate-kills-a-test-past-its-timeout`, `evidence for cap:workplan-complete-gate|cap:workplan-evidence-claims`, `assess --list`, `coverage code|tests` with and without `--crate`, and `check`. No `evidence record` was run. Every assessment is `--verdict accepted --by agent` with the SPDX copyright and `CC-BY-4.0`, recorded on the final text of its section. **No section is rejected.** No code, binding, change record or frontmatter was edited.

### `cap:workplan-entity-history` (6 accepted)

| Section | Verdict | Fix |
| --- | --- | --- |
| purpose | accepted | 1 |
| contract | accepted | 2, 3 |
| usage | accepted | none (the three commands run and answer as stated) |
| examples | accepted | 4 |
| constraints | accepted | 5 |
| verification | accepted | none (each bullet matches a bound test) |

1. "Some readers still need to know ..." -> "Some readers nevertheless need to know ...". "still" could read as relative to the change.
2. "The command first runs `git --version` ..." -> "Loading the repository lists its tracked paths with `git`, as it does for every command of the [command line](./workplan-cli.md). Then, before any question of its own, the history runs `git --version` ...". "first" was false: opening the repository runs `git rev-parse --is-inside-work-tree` and `git ls-files -z` before the version check.
3. "`<identifier>` is the last segment of the binding's path without a generic argument list" -> "... without a generic argument list or a raw-identifier prefix `r#`". The code strips `r#`, and the identifier pattern would refuse it.
4. Examples: "A table for one capability with one bound function", whose row source was `raf_workplan::reconcile`, -> "... with one bound method. The oldest commit wrote both the document and the method, so it is one entry with both sources". The rows now use `raf_workplan::Workplan::reconcile`, and the oldest row's source is "document, `raf_workplan::Workplan::reconcile`". `raf_workplan::reconcile` does not exist, and the new row shows the de-duplicated form the tool prints.
5. "The command reads Git only." -> "The command only reads, from Git and the loaded documents." It also reads the documents.

### `cap:workplan-evidence-claims` (7 accepted)

| Section | Verdict | Fix |
| --- | --- | --- |
| purpose | accepted | none |
| contract | accepted | 1, 2 |
| usage | accepted | 3, 4 |
| examples | accepted | 5 |
| constraints | accepted | 6 |
| explanation | accepted | none |
| verification | accepted | none |

1. "... of the form `name=number`, where the name is letters, digits, `_` and `-` from the start of the line; every other line is discarded" -> "... of the form `name=number`: a line whose text before its first `=` is a name, letters, digits, `_` and `-` from the start of the line, is a value, and the rest of it must be a number; every other line is discarded". `measure::printed` takes `status=ok` as a value and refuses it with `NotNumber`; it does not discard it.
2. "a marker that names no evidence record (`claim-evidence-missing`)" -> "a marker that names an evidence record that does not exist (`claim-evidence-missing`)". A marker with no id is a parse error, not this finding.
3. "A command whose standard output holds other `key=value` lines, such as `status=ok`, fails the recording, because each such line is read as a value" -> "... other lines that start with a name and `=`, such as `status=ok`, has each of them read as a value: one that is not a number, as here, makes the recording an error that writes nothing, and a number is stored as one more value". The old text was false for a numeric line such as `n=3`, which is stored.
4. "... includes an obligation for every claim whose evidence changed." -> "... for every claim of a capability the change lists whose evidence changed." `lifecycle/changed.rs` raises `evidence-changed` only for the change's own capabilities.
5. "| Subject | command `cargo bench parse` |" -> "| Subject | `command`, with the command text `cargo bench parse` |". A record stores `subject: command`, with the text under its own `command` key.
6. "The stored command is never run, and only the command line after `--` reaches a process." -> "The stored command is never run, and no text of a record reaches a process: the command `evidence record` runs is the command line after `--`, and a `--from-binding` path reaches `cargo` only as the test filter of the rows the completion gate runs." The old text was false for `--from-binding`.

### `cap:workplan-prose-assessment` (6 accepted)

| Section | Verdict | Fix |
| --- | --- | --- |
| purpose | accepted | 1 |
| contract | accepted | 2, 3 |
| usage | accepted | 4, 5 |
| examples | accepted | 6, 7 |
| constraints | accepted | none |
| verification | accepted | 8, 9, 10 |

1. "records what was assessed, by whom and against which inputs" -> "records what was assessed, whether a human or an agent assessed it and against which inputs". The record names the assessor by kind only.
2. "A bound symbol that does not resolve, or a claim on a record that does not load, is refused" -> "A bound symbol that is an input and does not resolve, ...". `inputs_now` judges only the bindings that are inputs.
3. "receives an obligation in the reconcile list, naming the input that moved" -> "receives obligations in the reconcile list: `assessment-missing`, one `assessment-stale` for each input that moved, naming it, or `assessment-rejected`". Only the stale kind names an input, and there is one obligation per moved input (`trace::assessments`).
4. Usage `--context`: "prints the section, the bound symbols and the evidence" -> adds "and the inputs with their digests". This is what `AssessContext` prints.
5. Usage `--verdict`: "replacing the one the section had" -> "... had, if any; given together, `--copyright <text>` and `--licence <id>` open the record with its two SPDX lines". A first record is created, not replaced, and the CLI requires the two SPDX flags together.
6. Examples: the three `change reconcile` lines were in the order usage, explanation, constraints -> usage, constraints, explanation, with the lead "The lines come in the order of the sections." Obligations sort by section in document order, so the old order could not be printed. The line formats match the `Display` code exactly.
7. "Each resolved item ends in an assessment record, and the record stays current only until the section, a bound symbol or the evidence changes." -> "Each is resolved by recording an assessment, after revising the section for the rejected one, and the record stays current only until one of its inputs moves: the section, a bound symbol that is an input or an evidence record a claim of the section names." "item" could be confused with change items, and only a bound symbol that is an input moves the assessment.
8. Refusals bullet: "a missing section, an unknown capability, ..." -> "a subject not written `cap:<slug>#<section>`, a section that is not one of the seven, an unknown capability, ...; one SPDX field alone is refused and the two together are written". This is what `each_refusal_writes_nothing_and_its_neighbour_records` tests.
9. Bounds bullet (fork): appended "the record of ten thousand inputs is over the document bound and is refused with nothing written, and one of eight thousand records". The test proves this and the bullet did not say it.
10. Bounds bullet (lead review of 9, which read ambiguously): "... the record of ten thousand inputs is over the document bound and is refused with nothing written, and one of eight thousand records." -> "... the record of ten thousand named symbols is over the document bound and is refused with nothing written, and the record of eight thousand is written, is current, and goes stale naming the one symbol edited." This is what `a_huge_section_and_ten_thousand_named_symbols_are_recorded_and_judged_within_bounds` asserts. The Verification assessment was recorded again afterwards.

### `cap:workplan-two-way-coverage` (6 accepted)

| Section | Verdict | Fix |
| --- | --- | --- |
| purpose | accepted | 1 |
| contract | accepted | 2, 3, 4, 5 |
| usage | accepted | 6, 7 |
| examples | accepted | 8 |
| constraints | accepted | none |
| verification | accepted | 9, 10, 11, 12 |

1. "every binding resolves to code and to a test" -> "every code binding resolves to an item and every test binding to a test". Each binding kind resolves to its own target.
2. "reported under the first of them" -> "... the first of them in path order". Exports are sorted by path.
3. "a surface that cannot be listed is `coverage-surface-unreadable`" -> adds "at the record's `scope`" (`Location::Key("scope")`).
4. "no coverage record has exactly one finding, `coverage-record-missing`" -> "has one coverage finding and no other, `coverage-record-missing`, ...; a record that does not load is reported by its own findings and not as missing". The old text read as one finding for the whole repository, and the new clause is the `is_present` branch and its test.
5. Contract: "`scope: []` enforces nothing" -> "enforces no crate". An exclusion that matches nothing is still a finding under an empty scope.
6. Usage: added "`cargo workplan coverage code --crate raf-okf` prints the report of one package, named by its package name or its crate name (`raf_okf`)" (`selected()`).
7. Usage: "`scope: []` ... enforces nothing" -> "enforces no crate". Same reason as 5.
8. Examples: "Adding a public function `parse_partial` to a crate and running the code report:" -> "Adding to `raf-okf` a public function `parse_partial`, and a `#[cfg(test)]` module `body::tests` of unit tests of the private functions of `body`, gives the code report a row for the function among its gaps:". Also, "as it lists the prefix of a module of unit tests that guard internals" -> "The record takes the new unit tests out of the tests report by the prefix of their module", and the reason "A wrapper of parse that the parse capability explains with it." -> "A wrapper of Document::parse that the OKF profile explains with it.". `raf_okf::body::tests` does not exist here, `raf-okf` has no free `parse` and there is no "parse capability".
9. "A fixture crate with one bound and one unbound public item: the code report lists only the unbound item." -> "A bound public item is absent from the code report, and its unbound neighbour is listed." The fixture has nine public items.
10. "binding a re-export covers its definition under every path" -> "Binding a module or a type covers the items below it and nothing beside it; binding an item by its exported or its defining path covers it, and what is below it, under its exported path". This is what the re-export test and the "nothing beside it" test assert.
11. No-record bullet: gains "a record that does not load gives only its own findings, and a repository without capability documents owes no record" (`without_a_record_check_has_exactly_one_finding_that_says_to_add_it`). The repository bullet "This repository's `check` is clean with `scope: [...]`" -> "This repository's coverage record has `scope: [raf-workplan, cargo-workplan]`, its `check` has no coverage finding, both directions of those crates are covered, and `raf-okf` and `raf-source`, outside the scope, are reported with their gaps". The test asserts only the absence of `coverage-*` findings.
12. Lead review of 11: "... `coverage-record-missing`, a record that does not load gives only its own findings, and a repository without capability documents owes no record;" -> "... `coverage-record-missing`, while a record that does not load gives only its own findings and a repository without capability documents owes no record;". This fixes a comma splice inside the semicolon list. The Verification assessment was recorded again afterwards.

### `cap:workplan-current-rationale` (7 accepted: the six of `affects` and the Explanation, recorded again)

| Section | Verdict | Fix |
| --- | --- | --- |
| purpose | accepted | none |
| contract | accepted | 1, 2 |
| usage | accepted | none (`history arch:normal-form` answers) |
| examples | accepted | none (both `docs/architecture` pages carry `## Decision` and `## Conditions to revisit`) |
| constraints | accepted | 3 |
| explanation | accepted, replacing the stale record (input `raf_workplan::rationale` moved with the module documentation test of 11C) | none |
| verification | accepted | 4, 5 |

1. "its text after the `#` marks, with the surrounding spaces removed, is exactly ..." -> "... with the surrounding spaces and a closing run of `#` marks removed, is exactly ...". `atx` strips a closing run, and `headings_match_exactly_at_their_level` counts `## Decision ##`.
2. "A link counts when it is written in any section of the document" -> "A link counts when it is written anywhere in the body of the document, before its first section or in any section". `add_links` reads the whole body, including text before the first `##`.
3. "An architecture page states a resolved decision. It does not narrate how the decision was reached." -> "A decision an architecture page records is resolved. The page does not narrate how the decision was reached." Not every architecture page records a decision; the Contract says one with none needs no conditions list.
4. "The findings are the same on every run." -> "... on every run and come in the order of files and lines." (`the_findings_are_the_same_on_every_run_in_file_and_line_order`).
5. "a list item is a marker, a space and text" -> "a marker, a space or a tab and text". `list_item_text` accepts a tab, and the unit test holds `-\tTabbed.`.

### Totals and gates (working tree on HEAD `d6f8804`, nothing committed)

* Assessments recorded: 32 files: 31 for the sections the five records affect and the Explanation of current-rationale. The prose-assessment and two-way-coverage Verification records were written twice (after the lead's fixes 10 and 12), so there were 34 recording runs. All 32 are `accepted`, by agent, and `current` in `assess --list`; none is rejected.
* Text fixes: 38 (entity-history 5, evidence-claims 6, prose-assessment 10, two-way-coverage 12, current-rationale 5). No contract was narrowed.
* `change reconcile <slug>` for each of the five: no output, exit 0. `change reconcile workplan-complete-gate`: no `claim-unsupported`, `evidence-stale` or `evidence-changed` line, so `gate-kills-a-test-past-its-timeout` still supports its claim.
* `cargo run -q -p cargo-workplan -- --root <worktree> check`: exit 0, no finding.
* `make docs-sync`: exit 0; the working tree (diff and untracked files) is byte-identical before and after.
* `docs preview <slug>` for each of the five: exit 0.
* For the commit (outside these five capabilities' files): `INVENTORY.records` in `tests/docs_contract.rs` lists only `assessment:workplan-current-rationale-explanation`, `coverage:default` and `evidence:gate-kills-a-test-past-its-timeout`. `check_inventory` flags every record file it does not list, so the 31 new `assessment:<slug>-<section>` ids of this part (and those of 11D1 and 11D2) are added there in the same commit.
* Observation, not changed (a binding is not section text): `raf_workplan::git::resolve_commit` is bound in `cap:workplan-entity-history` `bindings.code`, but the history code never calls it (it serves the merge-base lookup). It only adds an input to these assessments.

## Task 11E: integrate the assessments, fix the loose ends, commit

Working tree on HEAD `d6f8804` with the uncommitted work of 11D1 to 11D3.

1. `git status --short` and `git diff` of the twelve capability documents: body text only (no frontmatter, no code); 13 modified files and 53 new assessment files (54 assessment records in all, the Explanation of current-rationale modified).
2. `tests/docs_contract.rs`: `INVENTORY.records` now lists all 56 records, sorted (54 `assessment:<slug>-<section>` ids taken from the file names of `work/assessments/`, `coverage:default`, `evidence:gate-kills-a-test-past-its-timeout`). The pin of `inventory_table_has_the_documented_shape` no longer repeats the list: it asserts the list is sorted with each id once, 54 assessments, 1 coverage record, 1 evidence record, 56 in all. `git add work/assessments`; `cargo test --test docs_contract --test repo_gates`: exit 0 (28 and 57 passed).
3. Loose ends:
   a. `crates/cargo-workplan/src/main.rs` crate docs, the `refusal` row of the JSON table: `condition` now lists `assessments` (between `bound-runs` and `after-removal`, the order of `Condition`), `number` reads "1 to 6" (`Condition::Assessments` is 6 in `lifecycle/completion.rs`). No test pins that text (searched `crates/`, `tests/`, `docs/`). Doc comment only.
   b. `docs/capabilities/okf-profile.md` `bindings.tests`: added `raf_okf::test_crate::claims::a_claim_may_rest_on_several_records` and `raf_okf::test_crate::claims::a_claim_after_a_list_item_supports_that_item_only` (both in `crates/raf-okf/tests/claims.rs`). `check` resolves them (exit 0).
   c. `docs/capabilities/workplan-entity-history.md` `bindings.code`: removed `raf_workplan::git::resolve_commit` (no call in `history.rs`, `history/` or `cargo-workplan`; it serves `default_merge_base`). It stays covered by the `raf_workplan::git` module binding of `cap:workplan-reconcile`: `check` exit 0, no coverage finding.
   `make docs-sync`: exit 0; `docs/capabilities/index.md` and the generated blocks unchanged.
4. `change reconcile <slug>` for the twelve: eleven printed nothing (exit 0); `workplan-entity-history` printed five `assessment-stale` lines (purpose, contract, usage, examples, constraints), each "the symbol `raf_workplan::git::resolve_commit` is no longer an input" (exit 1). The Verification and Explanation were not stale (they name symbols of their own). Each of the five was read again with `assess cap:workplan-entity-history#<section> --context`: the text is the one 11D3 accepted, and no sentence of it rests on `resolve_commit` (the removed-binding lookup is `last_change`, `git log -S`). Re-recorded `--verdict accepted --by agent` with the SPDX copyright and `CC-BY-4.0` (each `replaced:`, exit 0; each record lost only its `resolve_commit` input). The okf-profile test bindings and the `main.rs` doc comment staled nothing (test bindings are no assessment input, and no capability's input digest covers the binary's crate docs). Second pass over the twelve: no output, exit 0 each.
   Evidence: `evidence claims gate-kills-a-test-past-its-timeout` exit 0 (the claim at `workplan-complete-gate.md:410`); `evidence for cap:workplan-complete-gate`: `pass`; `change reconcile workplan-complete-gate` shows no `claim-unsupported`, `evidence-stale` or `evidence-changed`, so the record was not re-recorded (no code it depends on changed).
5. Gates (working tree after steps 1 to 4):

| Gate | Exit | Result |
| --- | --- | --- |
| `make docs-sync` | 0 | `git diff` hash identical before and after; no untracked file |
| `cargo run -q -p cargo-workplan -- --root <worktree> check` | 0 | no finding |
| `cargo test --workspace` (first run) | 101 | one failure: `raf-source` `module_tree::inline_modules_with_path_alternatives_are_bounded_by_the_module_path_budget`, its debug-mode 10 s bound missed at 10.149 s, host load average 65; nothing under `crates/raf-source` changed |
| that test alone | 0 | 5.69 s |
| `cargo test --workspace` (second run) | 0 | 111 `test result: ok`, no failure |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | |
| `cargo fmt --check` | 0 | |
| `make reuse` | 0 | 494 / 494 files compliant |
| `make issue-refs` | 0 | |
| `assess --list` | 0 | 54 lines, each `accepted  agent  current` |

6. Commits (hooks on, `core.hooksPath=.githooks`): the first commit with only the capability documents and `main.rs` staged was refused by the pre-commit gate (exit 1, nothing committed): `repository_documents_satisfy_the_profile` reported the 53 new assessment files "the document is missing", because the `INVENTORY` of the working-tree test binary lists records the staged index did not hold. So both parts were committed together under the first message: `f88d1dd docs: fix statements found false when assessing the workplan capabilities` (68 files: 12 capability documents, `crates/cargo-workplan/src/main.rs`, `tests/docs_contract.rs`, 54 assessment records; `docs/capabilities/index.md` unchanged). Pre-commit gate exit 0. Pushed (exit 0); `origin/paudley/10-m2-rest-of-the-planning-essay-entity` is `f88d1dd`.

Totals: 54 assessment records (okf-profile 2, source-index 5, workplan-document-model 3, workplan-generated-docs 1, workplan-reconcile 3, workplan-complete-gate 3, workplan-cli 5, workplan-entity-history 6, workplan-evidence-claims 7, workplan-prose-assessment 6, workplan-two-way-coverage 6, workplan-current-rationale 7), all accepted by agent and current; 5 re-recorded in this part (entity-history purpose, contract, usage, examples, constraints, for the removed `resolve_commit` input); none rejected; the evidence record unchanged and passing. 56 records in `INVENTORY.records`.

## Task 11F1: complete the records okf-profile, source-index, workplan-document-model, workplan-generated-docs with the tool

Start: HEAD `f88d1dd`, clean tree, hooks on (`core.hooksPath=.githooks`). Each record listed only its own capability in `changes` (no other open record lists any of the four), so `state-delivered-open-record` cannot refuse. One `complete` at a time, each committed and pushed before the next.

### okf-profile

1. `docs/capabilities/okf-profile.md` frontmatter `state: implementing` -> `state: delivered` (only change in the document). `tests/docs_contract.rs` `INVENTORY`: `("okf-profile", "delivered")`; `inventory_table_has_the_documented_shape` pins `implementing` 11 and `delivered` 1 (designed 9, records 56 unchanged). `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (a `## Delivered` group with the okf-profile row before `## Implementing`).
2. `cargo run -q -p cargo-workplan -- --root <worktree> change reconcile okf-profile`: no output, exit 0 (17 s).
3. `cargo run -q -p cargo-workplan -- --root <worktree> change complete okf-profile`: exit 0, 95 s, output:

   ```text
   completed: removed work/changes/okf-profile.md
   ```

   No `rewrote` line: `docs/capabilities/okf-profile.md` has no generated relationships block, so no `Changed by` row named the record. No refusal.
4. Verify: `work/changes/okf-profile.md` absent; `check` exit 0 (no finding); `make docs-sync` exit 0, nothing rewritten (no diff); `assess --list` 54 lines, each `accepted  agent  current`. Before staging, `docs_contract` (`repository_documents_satisfy_the_profile`, `repository_bindings_resolve_in_the_source_index`) and `repo_gates` (`issue_refs_tracked_files_are_clean`) failed with "cannot read tracked file work/changes/okf-profile.md": the gates read the index's file list, which still held the deleted record. After `git add` of the four paths: `cargo test --test docs_contract --test repo_gates` exit 0 (28 and 57 passed); `make reuse` exit 0; `make issue-refs` exit 0 (57 passed).
5. Commit `ef62049` `docs(capabilities): deliver the okf-profile capability` (4 files: the capability document, the deleted record, `docs/capabilities/index.md`, `tests/docs_contract.rs`); pre-commit gate exit 0; pushed (`f88d1dd..ef62049`).

### source-index

1. `docs/capabilities/source-index.md` `state: implementing` -> `state: delivered` (only change). `INVENTORY`: `("source-index", "delivered")`; the shape test pins `implementing` 10, `delivered` 2. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (the row moves to `## Delivered`).
2. `cargo run -q -p cargo-workplan -- --root <worktree> change reconcile source-index`: no output, exit 0 (17 s).
3. `cargo run -q -p cargo-workplan -- --root <worktree> change complete source-index`: exit 0, 367 s, output:

   ```text
   completed: removed work/changes/source-index.md
   ```

   No `rewrote` line (no generated relationships block in the document). No refusal.
4. Verify: record absent; `check` exit 0; `make docs-sync` nothing rewritten; after `git add` of the four paths `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57 passed); `make reuse` exit 0; `make issue-refs` exit 0; `assess --list` 54 lines, all `accepted  agent  current`.
5. Commit `a8bd498` `docs(capabilities): deliver the source-index capability` (4 files); pre-commit gate exit 0; pushed (`ef62049..a8bd498`).

### workplan-document-model

1. `docs/capabilities/workplan-document-model.md` `state: implementing` -> `state: delivered` (only change by hand). `INVENTORY`: `("workplan-document-model", "delivered")`; the shape test pins `implementing` 9, `delivered` 3. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md`.
2. `cargo run -q -p cargo-workplan -- --root <worktree> change reconcile workplan-document-model`: no output, exit 0 (24 s).
3. `cargo run -q -p cargo-workplan -- --root <worktree> change complete workplan-document-model`: exit 0, 115 s, output:

   ```text
   completed: removed work/changes/workplan-document-model.md
   rewrote docs/capabilities/workplan-document-model.md
   ```

   The rewrite removed the one row `| Changed by | [workplan-document-model](../../work/changes/workplan-document-model.md) |` from the relationships block. No refusal.
4. Verify: record absent; `check` exit 0; `make docs-sync` nothing rewritten; after `git add` `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57 passed); `make reuse` exit 0; `make issue-refs` exit 0; `assess --list` 54 lines, all `accepted  agent  current` (the rewritten block is generated, blanked in the section digests).
5. Commit `6a132df` `docs(capabilities): deliver the workplan-document-model capability` (4 files, including the rewrite); pre-commit gate exit 0 (28 and 57 passed); pushed (`a8bd498..6a132df`).

### workplan-generated-docs

1. `docs/capabilities/workplan-generated-docs.md` `state: implementing` -> `state: delivered` (only change by hand). `INVENTORY`: `("workplan-generated-docs", "delivered")`; the shape test pins `implementing` 8, `delivered` 4. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md`.
2. `cargo run -q -p cargo-workplan -- --root <worktree> change reconcile workplan-generated-docs`: no output, exit 0 (11 s).
3. `cargo run -q -p cargo-workplan -- --root <worktree> change complete workplan-generated-docs`: exit 0, 49 s, output:

   ```text
   completed: removed work/changes/workplan-generated-docs.md
   rewrote docs/capabilities/workplan-generated-docs.md
   ```

   The rewrite removed the one row `| Changed by | [workplan-generated-docs](../../work/changes/workplan-generated-docs.md) |`. No refusal.
4. Verify: record absent; `check` exit 0; `make docs-sync` nothing rewritten; after `git add` `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57 passed); `make reuse` exit 0; `make issue-refs` exit 0; `assess --list` 54 lines, all `accepted  agent  current`.
5. Commit `0b88c66` `docs(capabilities): deliver the workplan-generated-docs capability` (4 files, including the rewrite); pre-commit gate exit 0 (28 and 57 passed); pushed (`6a132df..0b88c66`).

### Summary of 11F1

| Record | reconcile | complete | Duration | Rewrites | Commit |
| --- | --- | --- | --- | --- | --- |
| `okf-profile` | empty, exit 0 | exit 0 | 95 s | none | `ef62049` |
| `source-index` | empty, exit 0 | exit 0 | 367 s | none | `a8bd498` |
| `workplan-document-model` | empty, exit 0 | exit 0 | 115 s | `docs/capabilities/workplan-document-model.md` | `6a132df` |
| `workplan-generated-docs` | empty, exit 0 | exit 0 | 49 s | `docs/capabilities/workplan-generated-docs.md` | `0b88c66` |

No `complete` refused. The four capabilities are `delivered` at HEAD `0b88c66`; `git log --diff-filter=D` lists each record's deletion commit. After the four completions every remaining record (`workplan-reconcile`, `workplan-complete-gate`, `workplan-cli`, `workplan-entity-history`, `workplan-evidence-claims`, `workplan-prose-assessment`, `workplan-two-way-coverage`, `workplan-current-rationale`) still reconciles empty (exit 0), and `assess --list` shows all 54 assessments current: no assessment went stale. The other capabilities' states and records and `tests/roadmap.rs` were not touched. Working tree clean after the last push.

## Task 11F2: complete the records workplan-reconcile, workplan-complete-gate, workplan-cli with the tool

Start: HEAD `0b88c66`, clean tree, hooks on (`core.hooksPath=.githooks`). Each of the three records lists only its own capability in `changes` (no other open record lists any of the three), so `state-delivered-open-record` cannot refuse. One `complete` at a time, each committed and pushed before the next.

### workplan-reconcile

1. `docs/capabilities/workplan-reconcile.md` frontmatter `state: implementing` -> `state: delivered` (only change by hand). `INVENTORY`: `("workplan-reconcile", "delivered")`; the shape test pins `implementing` 7, `delivered` 5. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md`.
2. `cargo run -q -p cargo-workplan -- --root <worktree> change reconcile workplan-reconcile`: no output, exit 0 (32 s).
3. `cargo run -q -p cargo-workplan -- --root <worktree> change complete workplan-reconcile`: exit 0, 257 s, output:

   ```text
   completed: removed work/changes/workplan-reconcile.md
   rewrote docs/capabilities/workplan-reconcile.md
   ```

   The rewrite removed the one row `| Changed by | [workplan-reconcile](../../work/changes/workplan-reconcile.md) |`. No refusal.
4. Verify (after `git add` of the four paths, the deletion staged): record absent; `check` exit 0 (no finding); `make docs-sync` exit 0, nothing rewritten (no diff); `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57 passed); `make reuse` exit 0 (compliant); `make issue-refs` exit 0 (57 passed); `assess --list` exit 0, 54 lines, all `accepted  agent  current`.
5. Commit `e22ac7d` `docs(capabilities): deliver the workplan-reconcile capability` (4 files, including the rewrite); pre-commit gate exit 0 (28 and 57 passed); pushed (`0b88c66..e22ac7d`).

### workplan-complete-gate

1. `docs/capabilities/workplan-complete-gate.md` `state: implementing` -> `state: delivered` (only change by hand). `INVENTORY`: `("workplan-complete-gate", "delivered")`; the shape test pins `implementing` 6, `delivered` 6. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md`.
2. `cargo run -q -p cargo-workplan -- --root <worktree> change reconcile workplan-complete-gate`: no output, exit 0 (19 s).
3. `cargo run -q -p cargo-workplan -- --root <worktree> change complete workplan-complete-gate`: exit 0, 231 s, output:

   ```text
   completed: removed work/changes/workplan-complete-gate.md
   rewrote docs/capabilities/workplan-complete-gate.md
   ```

   The rewrite removed the one row `| Changed by | [workplan-complete-gate](../../work/changes/workplan-complete-gate.md) |`. No refusal.
4. Verify (after `git add` of the four paths): record absent; `check` exit 0; `make docs-sync` exit 0, nothing rewritten; `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57 passed); `make reuse` exit 0; `make issue-refs` exit 0 (57 passed); `assess --list` exit 0, 54 lines, all `accepted  agent  current`; `evidence for cap:workplan-complete-gate` exit 0: `evidence:gate-kills-a-test-past-its-timeout  pass` (claim at line 410, unchanged by the rewrite).
5. Commit `ba34ce9` `docs(capabilities): deliver the workplan-complete-gate capability` (4 files, including the rewrite); pre-commit gate exit 0 (28 and 57 passed); pushed (`e22ac7d..ba34ce9`).

### workplan-cli

1. `docs/capabilities/workplan-cli.md` `state: implementing` -> `state: delivered` (only change by hand). `INVENTORY`: `("workplan-cli", "delivered")`; the shape test pins `implementing` 5, `delivered` 7. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md`.
2. `cargo run -q -p cargo-workplan -- --root <worktree> change reconcile workplan-cli`: no output, exit 0 (24 s).
3. `cargo run -q -p cargo-workplan -- --root <worktree> change complete workplan-cli`: exit 0, 157 s, output:

   ```text
   completed: removed work/changes/workplan-cli.md
   rewrote docs/capabilities/workplan-cli.md
   ```

   The rewrite removed the one row `| Changed by | [workplan-cli](../../work/changes/workplan-cli.md) |`. No refusal.
4. Verify (after `git add` of the four paths): record absent; `check` exit 0; `make docs-sync` exit 0, nothing rewritten; `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57 passed); `make reuse` exit 0; `make issue-refs` exit 0 (57 passed); `assess --list` exit 0, 54 lines, all `accepted  agent  current`.
5. Commit `11ce02d` `docs(capabilities): deliver the workplan-cli capability` (4 files, including the rewrite); pre-commit gate exit 0 (28 and 57 passed); pushed (`ba34ce9..11ce02d`).

### Summary of 11F2

| Record | reconcile | complete | Duration | Rewrites | Commit |
| --- | --- | --- | --- | --- | --- |
| `workplan-reconcile` | empty, exit 0 (32 s) | exit 0 | 257 s | `docs/capabilities/workplan-reconcile.md` | `e22ac7d` |
| `workplan-complete-gate` | empty, exit 0 (19 s) | exit 0 | 231 s | `docs/capabilities/workplan-complete-gate.md` | `ba34ce9` |
| `workplan-cli` | empty, exit 0 (24 s) | exit 0 | 157 s | `docs/capabilities/workplan-cli.md` | `11ce02d` |

No `complete` refused. The three capabilities are `delivered` at HEAD `11ce02d` (seven delivered, five implementing). After the three completions each remaining record (`workplan-entity-history`, `workplan-evidence-claims`, `workplan-prose-assessment`, `workplan-two-way-coverage`, `workplan-current-rationale`) reconciles empty (exit 0), and `assess --list` shows all 54 assessments current. The other capabilities' states and records and `tests/roadmap.rs` were not touched. Working tree clean after the last push.

## Task 11F3: complete the records workplan-entity-history, workplan-evidence-claims, workplan-prose-assessment, workplan-two-way-coverage, workplan-current-rationale with the tool; restore the roadmap guard

Start: HEAD `11ce02d`, clean tree, hooks on (`core.hooksPath=.githooks`). Each of the five records lists only its own capability in `changes`. One `complete` at a time, each committed and pushed before the next.

### workplan-entity-history

1. `docs/capabilities/workplan-entity-history.md` frontmatter `state: implementing` -> `state: delivered` (only change by hand). `INVENTORY`: `("workplan-entity-history", "delivered")`; the shape test pins `implementing` 4, `delivered` 8. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md`.
2. `cargo run -q -p cargo-workplan -- --root <worktree> change reconcile workplan-entity-history`: no output, exit 0 (10 s).
3. `cargo run -q -p cargo-workplan -- --root <worktree> change complete workplan-entity-history` (background, output to a file): exit 0, 28 s, output:

   ```text
   completed: removed work/changes/workplan-entity-history.md
   ```

   No `rewrote` line (no generated relationships block in the document). No refusal.
4. Verify (after `git add` of the four paths, the deletion staged): record absent; `check` exit 0 (no finding); `make docs-sync` exit 0, nothing rewritten; `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57 passed); `make reuse` exit 0 (compliant); `make issue-refs` exit 0 (57 passed); `assess --list` exit 0, 54 lines, all `accepted  agent  current`.
5. Commit `d561881` `docs(capabilities): deliver the entity-history capability` (4 files; the subject names the capability without its `workplan-` prefix, unlike the other deliveries; the commit was already pushed, so it was not rewritten); pre-commit gate exit 0 (28 and 57 passed); pushed (`11ce02d..d561881`).

### workplan-evidence-claims

1. `docs/capabilities/workplan-evidence-claims.md` `state: implementing` -> `state: delivered` (only change by hand). `INVENTORY`: `("workplan-evidence-claims", "delivered")`; the shape test pins `implementing` 3, `delivered` 9. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md`.
2. `change reconcile workplan-evidence-claims`: no output, exit 0 (13 s).
3. `change complete workplan-evidence-claims` (background): exit 0, 47 s, output:

   ```text
   completed: removed work/changes/workplan-evidence-claims.md
   ```

   No `rewrote` line. No refusal.
4. Verify (after `git add` of the four paths): record absent; `check` exit 0; `make docs-sync` exit 0, nothing rewritten; `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57 passed); `make reuse` exit 0; `make issue-refs` exit 0 (57 passed); `assess --list` exit 0, 54 lines, all `accepted  agent  current`; `evidence for cap:workplan-complete-gate` exit 0: `evidence:gate-kills-a-test-past-its-timeout  pass` (claim at line 410).
5. Commit `fcd69d1` `docs(capabilities): deliver the workplan-evidence-claims capability` (4 files); pre-commit gate exit 0 (28 and 57 passed); pushed (`d561881..fcd69d1`).

### workplan-prose-assessment

1. `docs/capabilities/workplan-prose-assessment.md` `state: implementing` -> `state: delivered` (only change by hand). `INVENTORY`: `("workplan-prose-assessment", "delivered")`; the shape test pins `implementing` 2, `delivered` 10. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md`.
2. `change reconcile workplan-prose-assessment`: no output, exit 0 (23 s).
3. `change complete workplan-prose-assessment` (background): exit 0, 49 s, output:

   ```text
   completed: removed work/changes/workplan-prose-assessment.md
   ```

   No `rewrote` line. No refusal.
4. Verify (after `git add` of the four paths): record absent; `check` exit 0; `make docs-sync` exit 0, nothing rewritten; `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57 passed); `make reuse` exit 0; `make issue-refs` exit 0 (57 passed); `assess --list` exit 0, 54 lines, all `accepted  agent  current`.
5. Commit `7b8ae50` `docs(capabilities): deliver the workplan-prose-assessment capability` (4 files); pre-commit gate exit 0 (28 and 57 passed); pushed (`fcd69d1..7b8ae50`).

### workplan-two-way-coverage

1. `docs/capabilities/workplan-two-way-coverage.md` `state: implementing` -> `state: delivered` (only change by hand). `INVENTORY`: `("workplan-two-way-coverage", "delivered")`; the shape test pins `implementing` 1, `delivered` 11. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md`.
2. `change reconcile workplan-two-way-coverage`: no output, exit 0 (14 s).
3. `change complete workplan-two-way-coverage` (background): exit 0, 67 s, output:

   ```text
   completed: removed work/changes/workplan-two-way-coverage.md
   ```

   No `rewrote` line. No refusal.
4. Verify (after `git add` of the four paths): record absent; `check` exit 0; `make docs-sync` exit 0, nothing rewritten; `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57 passed); `make reuse` exit 0; `make issue-refs` exit 0 (57 passed); `assess --list` exit 0, 54 lines, all `accepted  agent  current`.
5. Commit `68a3342` `docs(capabilities): deliver the workplan-two-way-coverage capability` (4 files); pre-commit gate exit 0 (28 and 57 passed); pushed (`7b8ae50..68a3342`).

### workplan-current-rationale

1. `docs/capabilities/workplan-current-rationale.md` `state: implementing` -> `state: delivered` (only change by hand). `INVENTORY`: `("workplan-current-rationale", "delivered")`; the shape test pins `implementing` 0, `delivered` 12. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (the row moves to `## Delivered` and the now empty `## Implementing` group is dropped).
2. `change reconcile workplan-current-rationale`: no output, exit 0 (10 s).
3. `change complete workplan-current-rationale` (background): exit 0, 63 s, output:

   ```text
   completed: removed work/changes/workplan-current-rationale.md
   ```

   No `rewrote` line. No refusal.
4. Verify (after `git add` of the four paths): record absent; `check` exit 0; `make docs-sync` exit 0, nothing rewritten; `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57 passed); `make reuse` exit 0; `make issue-refs` exit 0 (57 passed); `assess --list` exit 0, 54 lines, all `accepted  agent  current`.
5. Commit `023df72` `docs(capabilities): deliver the workplan-current-rationale capability` (4 files); pre-commit gate exit 0 (28 and 57 passed); pushed (`68a3342..023df72`).

### Summary of the five completions

| Record | reconcile | complete | Duration | Rewrites | Commit |
| --- | --- | --- | --- | --- | --- |
| `workplan-entity-history` | empty, exit 0 (10 s) | exit 0 | 28 s | none | `d561881` |
| `workplan-evidence-claims` | empty, exit 0 (13 s) | exit 0 | 47 s | none | `fcd69d1` |
| `workplan-prose-assessment` | empty, exit 0 (23 s) | exit 0 | 49 s | none | `7b8ae50` |
| `workplan-two-way-coverage` | empty, exit 0 (14 s) | exit 0 | 67 s | none | `68a3342` |
| `workplan-current-rationale` | empty, exit 0 (10 s) | exit 0 | 63 s | none | `023df72` |

No `complete` refused. No assessment went stale; the evidence record still passes.

### The roadmap guard

`crates/raf-workplan/tests/roadmap.rs`, `this_repository_lists_every_capability_once_in_its_state_group`: the weakened check (delivered plus implementing not zero, "nothing is built") is replaced. The test now names the twelve capabilities this repository has delivered (`okf-profile`, `source-index` and the ten `workplan-*`) and asserts that each is in the roadmap with state `Delivered`, has no gap, and is in the delivered group. It asserts the delivered group is not empty again (`assert_ne!(..., 0, "nothing is delivered")`, as before the weakening) and that the nine census capabilities are listed, without pinning their state, so later census work does not break the guard. `cargo test -p raf-workplan --test roadmap`: exit 0 (14 passed). Negative check: with `docs/capabilities/workplan-cli.md` set to `state: implementing` in the working tree, the test failed (`assertion left == right failed: workplan-cli is not delivered`); the file was then restored with `git checkout`.

Prose made false by the deliveries: `README.md` (the Crates paragraph and the `raf-workplan` table row) and `AGENTS.md` (the Crates paragraph of section 1) still called `history`, `evidence record|claims|for`, `assess` and `coverage code|tests` "built and **implementing**". Both now list those commands as delivered with the other commands, list the five library capabilities among what `raf-workplan` delivers, and the README table row links the five capability documents. No test pins that text; the crate docs of `raf-workplan` and `cargo-workplan` mention `implementing` only as a state name.

### Final repository state

- `INVENTORY`: 21 capabilities, `delivered` 12, `implementing` 0, `designed` 9, `considered` 0; 56 records; 2 architecture pages. The only `"implementing"` left in `tests/docs_contract.rs` is the pin `count("implementing") == 0`.
- `work/changes/`: the nine `census-*` records only; no workplan record is open.
- `check`: exit 0, no finding.
- `status`: exit 0; `delivered 12` (okf-profile, source-index, the ten `workplan-*`), `implementing 0`, `designed 9` (the census capabilities, each with its own open change), `considered 0`, `open changes 9`.
- `assess --list`: exit 0, 54 lines, all `accepted  agent  current`.
- `coverage code --crate raf-workplan`: exit 0, no gap row; `coverage code --crate cargo-workplan`: exit 0, no gap row.

### Gates (working tree on HEAD `023df72` with the guard, `README.md` and `AGENTS.md` edits; run once)

| Gate | Exit | Result |
| --- | --- | --- |
| `cargo test --workspace` | 0 | 248 s; no failure |
| `cargo test -p raf-workplan -p raf-okf -p cargo-workplan --release` | 0 | 320 s; no failure (188 `test result: ok` lines over both runs) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | run again after touching `roadmap.rs`: exit 0 |
| `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps` | 0 | run again with the variable set directly: exit 0 |
| `cargo fmt --check` | 0 | |
| `make reuse` | 0 | compliant |
| `make deny` | 0 | advisories, bans, licenses, sources ok |
| `make msrv` | 0 | |
| `make issue-refs` | 0 | |
| `make licenses` | 0 | |
| `make docs-sync` (twice) | 0, 0 | nothing rewritten; `git diff --stat` identical (only the three hand edits) |

### Commits of 11F3

| Commit | Subject |
| --- | --- |
| `d561881` | `docs(capabilities): deliver the entity-history capability` |
| `fcd69d1` | `docs(capabilities): deliver the workplan-evidence-claims capability` |
| `7b8ae50` | `docs(capabilities): deliver the workplan-prose-assessment capability` |
| `68a3342` | `docs(capabilities): deliver the workplan-two-way-coverage capability` |
| `023df72` | `docs(capabilities): deliver the workplan-current-rationale capability` |
| `1c2f6a6` | `docs: state the workplan commands and capabilities as delivered` (`README.md`, `AGENTS.md`) |
| `a51977f` | `test(raf-workplan): require the delivered capabilities in the roadmap guard` |

Each pre-commit gate exit 0 (28 and 57 passed); all pushed (`11ce02d..a51977f`). Working tree clean. The census capabilities' states and records were not touched.

## Task 12 fix F1-F2

Start: HEAD `a51977f`, clean tree, hooks on (`core.hooksPath=.githooks`). Findings F1 (the history of a module binding covered only its `mod` line) and F2 (`--since` filtered on the committer date while entries show and order by the author date) of `reviews/completion-audit.md`, carried through the lifecycle on the delivered `cap:workplan-entity-history`.

### Reopening

1. `cargo run -q -p cargo-workplan -- --root <worktree> change new workplan-entity-history --capability cap:workplan-entity-history --affects ...#contract --affects ...#usage --affects ...#constraints --affects ...#verification --affects ...#explanation --copyright '2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>' --licence CC-BY-4.0`: exit 0, `created work/changes/workplan-entity-history.md`; guidance: set `state: implementing` in the same commit (`state-delivered-proposal`, `state-delivered-open-record`). `#explanation` was then removed from `affects` by hand: the Explanation stays true (the module rationale is stated in the Contract instead), and it has no assessment record, so listing it would have needed a new one with no text change behind it.
2. Record authored: title "Module history and an exact since", two items (module-binding history covers the files the digest reads; `--since` is exact by author date), summary. `git add` of the record before `docs sync`.
3. `docs/capabilities/workplan-entity-history.md` `state: delivered` -> `implementing`; `INVENTORY` `("workplan-entity-history", "implementing")`, shape pins `implementing` 1, `delivered` 11.

### Decisions

- F2: EXACT semantics chosen. `--since` no longer reaches `git`: `Arg::Since` is removed, `Invocation::git_document_log(path)` and `Invocation::git_symbol_log(lines, file)` lose their date parameter, `git::document_log` / `git::symbol_log` likewise, and `Workplan::history` keeps the entries whose author-date instant is >= 00:00:00 UTC of the day (`git::start_of_day`, crate-private). Cost: every log is read whole (bounded at 16 MiB as before); the `LogTooLarge` message no longer says "`--since` narrows it". `Date` and `Pattern::Date` stay (the CLI and `HistoryOptions` still validate the day, 1970-01-01..2099-12-31).
- F1: a code binding that resolves to a module below its crate root keeps the `-L` log of its own span (the `mod name;` line, or inline braces) and adds the document-log row (`--follow --cc --root --name-only -- <file>`) for each file `digest::module_files` returns (refactored to return `(path, text)` pairs, used by both the digest and the history), sorted by path and de-duplicated per binding; source = the binding path; paths = the names git lists. A crate root and non-module items keep `-L` only; an inline module with no module file below it keeps `-L` only. A module file whose full log is empty (no commit holds it) is the new `HistoryError::Uncommitted { binding, file }` ("the file `<file>` of the bound module `<binding>` has no commit ...; commit `<file>` and ask again"); because no date reaches git any more, an empty log can only mean no commit holds the file.

### Code, tests and documents (commit `bc3f15f`)

- `crates/raf-workplan/src/history.rs`: `FileQuery`, `Plan.files`, the module-file loop after the span logs, `HistoryError::Uncommitted`, the in-process `--since` filter; module docs, `HistoryOptions::with_since`, `Source::Symbol` and `Entry::paths` docs. `digest.rs`: `module_files` returns `Vec<(&str, &str)>` (same set and filter as before; the digest maps it to its sites, so no digest changes). `git/log.rs`: `start_of_day` (crate-private) and the date parameters dropped. `exec/invocation.rs`: `Arg::Since` removed, two rows without a date. `exec/pattern.rs`, `git.rs`: docs; the `LogTooLarge` message loses "and `--since` narrows it". `cargo-workplan` `cli.rs` help: "Only the commits whose author date is from 00:00:00 UTC of this day on."; `crates/cargo-workplan/tests/history.rs` comment "(`--since` reads the author date)".
- Pinned rows: `crates/raf-workplan/tests/exec.rs` (the two "since a date" rows became "of a path named like an option" rows, still 23 rows; the separator check now asserts no row carries `--since`); `exec/tests.rs` (variant walk without `since`, 15 variants); `tests/history.rs` stub argv rows without the date.
- New tests (all bound in `bindings.tests` of `cap:workplan-entity-history`): `a_module_binding_lists_the_commits_of_its_files_and_not_of_a_sibling` (real git: own-file and child-file edits listed for `demo::outer`; sibling, crate-root function and inline-module edits not; crate-root binding `demo` lists `lib.rs` edits only; inline `demo::inline` its braces), `a_module_file_that_is_not_committed_names_the_binding_and_the_file_until_it_is` (real git, the child file kept out of the commit by `.gitignore`; `Uncommitted { demo::outer, outer/inner.rs }`; neighbour after committing), `since_selects_by_the_author_date_and_not_by_the_committer_date` (real git, `GIT_AUTHOR_DATE` 2026-01-01 / `GIT_COMMITTER_DATE` 2026-10-16 excluded under `--since 2026-10-15`, author 2026-10-15T00:30Z / committer 2026-01-02 included; neighbours 2025-06-01 all three, 2026-10-16 none), `since_reaches_no_process_and_keeps_each_entry_by_its_author_date` (fake; replaces `since_reaches_both_logs_before_the_last_separator`, whose claim is now false), `a_module_binding_asks_its_span_and_then_the_log_of_each_of_its_files_in_order` (fake: exact argv order, no sibling, empty file log is `Uncommitted`), `git::log::tests::a_since_day_starts_at_midnight_utc_on_the_scale_of_an_author_date`.
- Negative check: with the file-log loop disabled (`take(0)`) and the filter disabled (`|| true`), 7 tests of `--test history` failed (the five new history tests plus the two existing `--since` real-git tests); restored from a copy, 23 passed.
- Live: `history cap:workplan-current-rationale` (exit 0) now lists `d6f8804` with sources `document, raf_workplan::rationale` and path `crates/raf-workplan/src/rationale.rs` (the audit saw `document` only).
- Capability text written directly as current text (no PROPOSED block): Contract (module paragraph, argv block without `--since`, question order, `--since` by author date and in process, the shown day in the author's offset), Usage (`--since` "authored from that day on"; the rationale example), Constraints (two path bases incl. module files; `--since` reaches no process and does not narrow the read; whole-file module history and the uncommitted-file error), Verification (module, uncommitted module file, date fixtures; the order fixture). `SECURITY.md` (the history paragraph: module-file `--follow` rows; `--since` reaches no process) and `CHANGELOG.md` (history entry; `Date`).
- Gates on that tree: `check` exit 0; `coverage code|tests --crate raf-workplan|cargo-workplan` exit 0, no gap row (4 runs); `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0 after two fixes in the new tests (`redundant_closure_for_method_calls`; `too_many_lines`, a helper `module_workspace` extracted); `cargo fmt --check` exit 0; `make reuse` exit 0 (483/483); `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps` exit 0; `make msrv` exit 0; `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57). `cargo test -p raf-workplan -p cargo-workplan`: exit 101, the one failure `roadmap::this_repository_lists_every_capability_once_in_its_state_group` ("workplan-entity-history is not delivered"), which the guard demands of a reopened capability by design; it passes again after the delivery (below). Every other test passed.
- Commit `bc3f15f` `fix(raf-workplan): history of a module binding covers its files and --since filters by author date` (17 files incl. the new record and the `index.md` rewrite); pre-commit gate exit 0 (28 and 57 passed).

### Reconcile and assessments (commit `b4ca6c8`)

1. `change reconcile workplan-entity-history`: exit 1, 21 lines: 19 `assessment-stale` (contract, usage, constraints, verification: the section and the symbols `Workplan::history`, `git::document_log`, `git::symbol_log`, `history`), 1 `tense-mismatch` (constraints line 238, "not yet"), 2 `item-without-outcome`. Fixed: "such as one not yet added" -> "such as a new file never committed" (and the same wording in the `history.rs` module doc); both items `outcome: implemented`. A re-read of the Contract against the code found one imprecision, fixed: the file logs follow the logs of all spans, so the Contract now states the full order (document, spans by binding path, module files by binding then file path, removal lookups).
2. Second reconcile: only `assessment-stale` lines. `assess --list` also showed purpose and examples stale (bound-symbol inputs), though outside `affects`.
3. `assess cap:workplan-entity-history#<section> --context` read for purpose, contract, usage, examples, constraints, verification (exit 0 each; 1316 to 1537 lines). Each statement checked against the code and the tests (module set and order = `digest::module_files` sorted by path; crate root and inline module keep `-L`; `Uncommitted` message; `start_of_day`; no `--since` in any argv; `Entry` paths from git for module files). Purpose and Examples unchanged and still true. Re-recorded all six `--verdict accepted --by agent --copyright '2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>' --licence CC-BY-4.0`: each `replaced: ... (accepted by agent)`, exit 0.
4. `change reconcile workplan-entity-history`: no output, exit 0. `check` exit 0; `make docs-sync` nothing rewritten.
5. Commit `b4ca6c8` `docs(workplan-entity-history): assess the sections again for module history and the author-date filter` (9 files); pre-commit gate exit 0 (28, 57 passed).

### Stale assessments and evidence of other capabilities (delivered, no open record)

`assess --list` after the re-recording: 54 lines, 48 current, 6 stale, none of `cap:workplan-entity-history`:

| Assessment | Stale input |
| --- | --- |
| `assessment:workplan-complete-gate-contract` | `raf_workplan::exec`, `raf_workplan::exec::Pattern` |
| `assessment:workplan-complete-gate-examples` | `raf_workplan::exec`, `raf_workplan::exec::Pattern` |
| `assessment:workplan-complete-gate-verification` | `raf_workplan::exec`, `raf_workplan::exec::Pattern` |
| `assessment:workplan-reconcile-contract` | `raf_workplan::git` |
| `assessment:workplan-reconcile-constraints` | `raf_workplan::git` |
| `assessment:workplan-reconcile-verification` | `raf_workplan::git` |

Cause: `Arg::Since` and the two row signatures (module `exec`), the `Pattern::Date` doc (`exec::Pattern`), and `git.rs` / `git/log.rs` (module `git`). A grep of both documents for the changed rows, `--since`, `Arg::`, `document_log`, `symbol_log` and `--follow` finds no statement the change made false; they are stale by digest only. The evidence record `evidence:gate-kills-a-test-past-its-timeout` has `raf_workplan::exec`, `exec::Invocation` and `exec::Pattern` among its inputs, so it is stale by digest too (inferred from those inputs, which `assess --list` reports changed for the complete-gate assessments; no reconcile of a change listing `cap:workplan-complete-gate` was run); `evidence for cap:workplan-complete-gate` still shows `pass` (verdict unchanged; staleness is an obligation of a change that lists the capability). `check` does not report any of these (exit 0). Per the instruction they were not re-recorded and the two capabilities were not reopened: a later change that lists `cap:workplan-complete-gate` or `cap:workplan-reconcile` will meet them as `assessment-stale` / `evidence-stale`.

### Delivery (commit `09a2c8d`)

1. `state: implementing` -> `delivered`; `INVENTORY` `("workplan-entity-history", "delivered")`, shape pins `implementing` 0, `delivered` 12. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (row back to `## Delivered`, the empty `## Implementing` group dropped).
2. `change reconcile workplan-entity-history`: no output, exit 0.
3. `change complete workplan-entity-history` (background, output to a file, no polling): exit 0, 27 s, output `completed: removed work/changes/workplan-entity-history.md`. No `rewrote` line (no relationships block in the document). No refusal.
4. `git add` of the four paths, the deletion staged. `docs/capabilities/index.md`, `tests/docs_contract.rs` and `work/changes/` are then byte-identical to `a51977f`.
5. Verify on that tree: `check` exit 0; `make docs-sync` nothing rewritten (diff hash identical, no untracked file); `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57); `make reuse` exit 0 (compliant); `make issue-refs` exit 0 (57 passed); `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0; `cargo fmt --check` exit 0; `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps` exit 0; `make msrv` exit 0; `status`: delivered 12, implementing 0, designed 9, considered 0, open changes 9 (census only); `assess --list` exit 0, 54 lines, 48 current, the 6 stale ones of the table above (none of entity-history); `history cap:workplan-entity-history --since 2026-10-16` exit 0. No attribution in any commit message; `.deficiencies` unchanged; no PROPOSED block left.
6. Commit `09a2c8d` `docs(capabilities): deliver the workplan-entity-history capability again` (explicit paths, 4 files); pre-commit gate exit 0 (28, 57 passed).
7. `cargo test --workspace` on the tree of `09a2c8d` (started with the delivery staged, same content): exit 0, 184 s, 111 `test result: ok`, no failure; the roadmap guard `this_repository_lists_every_capability_once_in_its_state_group` passes again.
8. Pushed `a51977f..09a2c8d` (three commits: `bc3f15f`, `b4ca6c8`, `09a2c8d`) once, after the delivery, so the intermediate state with the capability `implementing` (in which the roadmap guard fails by design) was never the remote head.

### Commits of Task 12 fix F1-F2

| Commit | Subject |
| --- | --- |
| `bc3f15f` | `fix(raf-workplan): history of a module binding covers its files and --since filters by author date` |
| `b4ca6c8` | `docs(workplan-entity-history): assess the sections again for module history and the author-date filter` |
| `09a2c8d` | `docs(capabilities): deliver the workplan-entity-history capability again` |

Unresolved, reported: the 6 stale assessments of `cap:workplan-complete-gate` (contract, examples, verification) and `cap:workplan-reconcile` (contract, constraints, verification), and the evidence record `evidence:gate-kills-a-test-past-its-timeout` whose `exec` inputs moved; stale by digest only, `check` clean, not re-recorded and the capabilities not reopened, per the instruction. Hosted CI has still not run on the branch.

## Task 12 stale cleanup

Start: HEAD `09a2c8d`, clean tree, hooks on (`core.hooksPath=.githooks`). Goal: no record stale after the history change (`bc3f15f..09a2c8d`), found with the shipped tool and re-recorded honestly.

### Probe before (complete list of stale records)

- `assess --list` (exit 0): 54 lines, 48 current, 6 stale: `workplan-complete-gate` contract, examples, verification (`raf_workplan::exec`, `raf_workplan::exec::Pattern` changed) and `workplan-reconcile` contract, constraints, verification (`raf_workplan::git` changed).
- Probe loop, scratch worktree `git worktree add --detach /tmp/stale-check HEAD`, binary `target/debug/cargo-workplan --root /tmp/stale-check`, script kept outside the scratch tree (`/tmp/stale-probe-scripts/probe.sh`). For each of the 12 delivered capabilities: `change new stale-probe-<cap> --capability cap:<cap> --affects cap:<cap>#<s>` for all seven sections (so every assessed section is in `affects`; reconcile judges assessments only for affected sections) `--copyright ... --licence CC-BY-4.0`, `git add` of the record, `change reconcile stale-probe-<cap>`, grep of `evidence-stale|assessment-stale`, then `git rm` of the record and `checkout`/`reset --hard` of the scratch tree (the first pass left a `Changed by` row for a probe record in `workplan-generated-docs.md` of the scratch tree, so the probe was rerun with each capability starting from a clean scratch tree; same result). Every `change new` exit 0; the other lines were the expected `assessment-missing` (sections with no record), `section-untouched`, `generated-stale` (the probe record itself).
- Result: 11 stale lines in 2 capabilities, 0 in the other 10:
  - `cap:workplan-complete-gate` (8): `document evidence-stale` x2 (`evidence:gate-kills-a-test-past-its-timeout` made against `raf_workplan::exec::Pattern` and `raf_workplan::exec`); `assessment-stale` x2 each for contract, examples, verification (`exec::Pattern`, `exec`).
  - `cap:workplan-reconcile` (3): `assessment-stale` for contract, constraints, verification (`raf_workplan::git`).
  - okf-profile, source-index, workplan-cli, workplan-current-rationale, workplan-document-model, workplan-entity-history, workplan-evidence-claims, workplan-generated-docs, workplan-prose-assessment, workplan-two-way-coverage: none.
  - So the complete set is the 6 assessments and the 1 evidence record named in Task 12 fix F1-F2; no other record (for instance an evidence record of `cap:workplan-entity-history` or `cap:workplan-cli`) was stale.

### Re-reading against the code

- `assess <subject> --context` read for the six sections (exit 0 each; 6342 to 9336 lines). The diff (`git diff a51977f bc3f15f -- crates/raf-workplan/src/exec* crates/raf-workplan/src/git*`): `Arg::Since` removed, `git_document_log(path)` / `git_symbol_log(lines, file)` without a date, the `Pattern::Date` and module docs (date reaches no process), the `git` module doc (a file of a bound module, no date reaches git, logs read whole), `LogTooLarge` loses "and `--since` narrows it", `start_of_day` re-exported crate-private.
- `workplan-complete-gate`: the Contract states the five `cargo` argv rows, the four gate patterns, the two validators backing reconcile's `git` commands (ref name, commit id), the bounds (1800 s, 300 s, 16 MiB) and the timeout claim; none names `--since`, `Date`, the history rows or a row count of the exec table. The Examples are completion output lines; the Verification states the stub fixture over each row of the gate's invocation table (still true: `every_row_reaches_a_process_as_exactly_its_argv`; the exec test table still has 23 rows, a number no document states). No statement false; no text edit.
- `workplan-reconcile`: the Contract lists reconcile's eight `git` commands in order, the 1 MiB blob bound and the 16 MiB bound of every other `git` command, 300 s; the Constraints say no process receives document text except a validated `<commit>:<path>` path or a resolved binding; the Verification states the stub-`git` invocation, version, default-branch and blob fixtures. None names the history logs, `--since` or `LogTooLarge`; the changed functions are history questions only. No statement false; no text edit.

### Re-recorded

1. `assess cap:workplan-complete-gate#{contract,examples,verification}` and `assess cap:workplan-reconcile#{contract,constraints,verification}` `--verdict accepted --by agent --copyright '2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>' --licence CC-BY-4.0`: each `replaced: work/assessments/<slug>.md (accepted by agent)`, exit 0.
2. `evidence record --id gate-kills-a-test-past-its-timeout --for cap:workplan-complete-gate --by agent --update --from-binding raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran --copyright ... --licence CC-BY-4.0`: exit 0, 14 s, `recorded: work/evidence/gate-kills-a-test-past-its-timeout.md (pass)` (the bound test ran through `cargo test` and passed). Diff of the record: only the digests of `raf_workplan::exec` and `raf_workplan::exec::Pattern` changed; verdict `pass`, `by: agent`, `for` unchanged.
3. That re-recording moved the input `record evidence:gate-kills-a-test-past-its-timeout` of the Contract assessment (its claim at line 410 names the record): `assess --list` then showed `cap:workplan-complete-gate#contract` stale ("the evidence record ... has changed since"). The claim sentence (a test past its timeout is killed and fails its binding, the reason names the time it ran and the limit) is what the passing test asserts; re-assessed `accepted` by agent, exit 0.
4. `assess --list`: exit 0, 54 lines, 54 current.

### Probe after

- The seven changed records were copied into a fresh scratch worktree of `09a2c8d` and committed there on its detached HEAD (the commit stayed in the scratch tree and was discarded with it). A first attempt at that scratch commit was made with `-c core.hooksPath=/dev/null`; it was undone at once (`reset --soft HEAD~1`) and made again with the hooks on, pre-commit gate passed (57), so each probe's reset keeps them. The same 12-capability loop: every `change new` exit 0, 0 `evidence-stale` and 0 `assessment-stale` lines in every capability; the only kinds left are `assessment-missing` 30, `section-untouched` 31 and `generated-stale` 5, all caused by the probe records themselves (no `claim-unsupported`, `evidence-changed` or `assessment-rejected`). Scratch worktree removed (`git worktree remove --force /tmp/stale-check`, `prune`).
- `evidence for cap:workplan-complete-gate`: exit 0, `evidence:gate-kills-a-test-past-its-timeout  pass ... (in its for list; claims at lines 410)`.

### Gates

`check` exit 0; `make docs-sync` exit 0, nothing rewritten (status and diff hash identical before and after); `cargo test --locked --test docs_contract --test repo_gates` exit 0 (28, 57); `make reuse` exit 0 (482/482, compliant); `make issue-refs` exit 0 (57 passed); `assess --list` exit 0, 54/54 current. No capability text edited; no capability reopened; no change record created in the repository; `.deficiencies` unchanged.

### Commit of Task 12 stale cleanup

| Commit | Subject |
| --- | --- |
| `8e5f568` | `docs: record the assessments and evidence that the history change made stale` |

Explicit paths (the six `work/assessments/*.md` and `work/evidence/gate-kills-a-test-past-its-timeout.md`, 7 files, 12+/12-), hooks on, no stash; pre-commit gate exit 0 (reuse, 28 and 57 passed). No attribution in the message. Pushed `09a2c8d..8e5f568`. Hosted CI has not been observed on the branch.

## Stage 2 remediation A

Start: HEAD `43c7e02`, clean tree, hooks on (`core.hooksPath=.githooks`). Scope: the reopening commit, S1 (record writer link-following), G1 (condition-6 headline count), G5 (output-kill time). No hook bypass, no stash.

### C1: reopening (commit `0ebc1f7`)

1. `cargo run -q -p cargo-workplan -- --root <worktree> change new <slug> --capability cap:<slug> --affects ... --copyright '2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>' --licence CC-BY-4.0` for the ten slugs, each exit 0 (`created work/changes/<slug>.md`), affects as Revision 2 lists: workplan-document-model (contract, usage, verification), workplan-evidence-claims (contract, constraints, verification), workplan-prose-assessment (constraints, verification), workplan-complete-gate (contract, examples, constraints, verification), okf-profile (contract, verification), workplan-cli (contract, constraints, verification), workplan-entity-history (contract, verification), workplan-two-way-coverage (constraints), workplan-generated-docs (verification), source-index (verification). `git add work/changes` before `docs sync`.
2. Each record authored: title, description, summary, items = the claims of this remediation about the capability (document-model 3, evidence-claims 5, prose-assessment 2, complete-gate 3, okf-profile 2, cli 2, entity-history 1, two-way-coverage 1, generated-docs 1, source-index 1); no outcome.
3. `state: delivered` -> `state: implementing` in the ten capability documents; `INVENTORY` states; shape pins `implementing` 10, `delivered` 2 (records list unchanged, 56).
4. Roadmap guard (`crates/raf-workplan/tests/roadmap.rs`): the delivered-only loop is replaced by `built_capabilities_hold(&roadmap, &BUILT)`: each of the 12 is delivered, or implementing with the open change record named after it (`open_change_slugs` contains the slug); no gap in either case; in the group of its state. The final assert is "delivered + implementing not zero" ("nothing is built"). New fixture test `a_built_capability_is_delivered_or_reopened_under_its_own_change_and_has_no_gap`: delivered with no record holds; implementing with no record fails ("built is implementing with no open change named after it"); implementing with only `change:grow-built` fails; implementing with `change:built` (and `grow-built`) holds; implementing with its record and an empty tests list fails ("built has gaps"); designed fails; an absent slug fails. Bound in `bindings.tests` of `cap:workplan-document-model`; its Verification gets a `> [!PROPOSED change:workplan-document-model]` block restating the Roadmap fixtures sentence.
5. Negative check on the real repository: `work/changes/source-index.md` removed from the working tree, `cargo test -p raf-workplan --test roadmap this_repository`: FAILED ("source-index is implementing with no open change named after it (open: [])"); restored, passes.
6. `make docs-sync`: exit 0, rewrote `index.md`, `workplan-cli.md`, `workplan-complete-gate.md`, `workplan-document-model.md`, `workplan-generated-docs.md` (Changed by rows, verification matrix row). `check` exit 0.
7. Gates after the commit: `cargo test -p raf-workplan --test roadmap` exit 0 (15 passed); `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57); `check` exit 0. Pre-commit gate exit 0 (reuse 492/492, 28, 57). Pushed `43c7e02..0ebc1f7`.

### S1: record writes through real directories (commit `16d3648`)

Shared judgement: `files::target(root, directory, path) -> Result<Target, Refused>` (`crates/raf-workplan/src/files.rs`): each directory on the way (`work`, then `work/<type>`; `docs`, then `docs/capabilities`) must be a directory, judged by the root's `DiskTree::entry`; a symbolic link ("it is a symbolic link, and nothing is written through a link"), a file or anything else ("it is not a directory") is refused naming that directory; a missing directory ends the walk (nothing below exists); the kind at the path itself is then read, a final link not followed. Users:

| Write path | Before | After |
| --- | --- | --- |
| `files::create_new` (`record::create`, `change new`) | own per-directory loop | `files::target`; a file already there is `Exists` before the open, `create_new` open kept |
| `record::replace` | `tree.entry(path)` (followed in-root parent links) | `files::target`; target must be `File`, else `Missing` |
| `evidence::target_fits` (pre-check before the run) | `symlink_metadata(root.join(path))` | `files::target`; `Create`+none / `Replace`+`File` fit |
| `Workplan::record_assessment` (replace or create) | `symlink_metadata(root.join(path))` | `files::target`; `File` replace, none create, else `Exists` |
| `sync::judge` (`Workplan::write`) | `tree.entry(path)` and parent entry | `files::target` over the parent of each edit; reachable because `SyncReport.edits` is public and a report can be computed before a link appears |
| `change complete` removal (`lifecycle/complete.rs`) | `fs::remove_file(dir.join(record))` | `judge_record` in `evaluate` (both evaluations, before any cargo process and before the writes): `work`/`work/changes` judged, record must be `File`, else new `LifecycleError::RecordRefused { path, source }` (exit 2). Reachable through the library: `Lifecycle::open_with` with a link made after the open (the loader reads `work/changes` through a linked `work`); a linked `work/changes` itself is refused by the loader's listing, so the change is not found (`ChangeLookupError::Unknown`): that case is unreachable for the removal and is pinned as such |

Fixtures (all bound in the same commit; `coverage code|tests --crate raf-workplan` exit 0, no gap row):

- `records::a_directory_link_inside_the_root_on_the_way_to_a_record_is_refused_and_a_real_directory_is_not` (document-model): links `work` (each of the three record types), `work/evidence`, `work/assessments`, `work/coverage`, each relative and inside the root, with the record's own file behind it; `create` and `replace` each `RecordError::Write { path == link }`, message names "symbolic link", `snapshot` byte-identical; neighbour: the real directory, `replace` then `create` succeed.
- `records::a_replacement_through_a_link_to_the_documents_never_overwrites_a_document` (document-model): reproduction 1 (`work/coverage -> ../docs`, `docs/default.md`): `replace` refused naming `work/coverage`, `docs/default.md` unchanged.
- `evidence::tests::a_directory_link_on_the_way_to_the_record_is_refused_before_the_command_runs` (evidence-claims): plan loaded, then `work` or `work/evidence` linked in root with `bound.md` behind it; `Create` and `Replace` each refused (`Write { path == link }`), `FakeRunner` has no call, tree identical; neighbour replace and create record.
- `evidence::tests::evidence_through_the_gate_never_replaces_a_capability_document_through_a_link` (evidence-claims): reproduction 2 (member `pkg`, `docs/capabilities/fixture.md`, `work/evidence -> ../docs/capabilities`, `Gate::workspace(root).source_roots(&["pkg"]).tracked_paths(vec![]).open()`, `record_evidence` id `fixture` with `WriteMode::Replace`): refused naming `work/evidence`, no process, the capability document byte-identical; neighbour with a real `work/evidence` records `work/evidence/fixture.md`.
- `assessments::an_assessment_is_never_written_through_a_linked_directory_and_a_real_one_records` (prose-assessment): `work` and `work/assessments` linked with `omega-contract.md` behind; refused naming the link, tree identical; neighbour replaces; a file at `work/assessments` refused.
- `sync::write_refuses_a_target_behind_a_directory_link_inside_the_root` (generated-docs): report computed, then `docs` or `docs/capabilities` linked; `Workplan::write` refused ("the directory <link> is refused"), tree identical; neighbour writes the report.
- `complete::a_linked_work_directory_is_refused_before_anything_runs_and_a_real_one_completes` (complete-gate): lifecycle opened, then `work` linked: `LifecycleError::RecordRefused { path: "work" }`, "nothing was run or written", no cargo call, tree identical; `work/changes` linked: `ChangeLookupError::Unknown` (the loader refuses a linked document directory), no cargo call; neighbour completes.

Mutants (each restored from a copy, `cmp`/grep confirmed):

| Mutant | Tests that failed |
| --- | --- |
| `record::replace` judges with `files::target(root, "", &path)` (no directory judged, the old behaviour) | both `records` link tests |
| `sync::judge` without its directories | `write_refuses_a_target_behind_a_directory_link_inside_the_root` |
| `judge_record` not called in `complete` | `a_linked_work_directory_is_refused_before_anything_runs_and_a_real_one_completes` |
| `evidence::target_fits` without its directories | both evidence link tests (a process started) |

The pre-check of `record_assessment` with its directories dropped is not caught on its own: both paths it then chooses (`create`, `replace`) judge the directories again and refuse, so the result is the same refusal; it is defence in depth, not a distinct behaviour.

Text (PROPOSED blocks in the reopened capabilities): document-model Contract "Writing records" restated with "Each judges the way to the record before anything is written, as every write of a record does: `work` and the directory of the record type must each be a directory, and a symbolic link there, whether it leads inside the root or out of it, a file or anything else is refused, naming that directory. A record is therefore written in the record directory of the root and never through a link onto another file, such as a document." and a Verification bullet (writer link fixtures); evidence-claims Constraints "Evidence is written only through real directories: before the command or the binding runs, `work` and `work/evidence` must each be a directory, and a symbolic link there, inside the root or out of it, or a file is refused with nothing run and nothing written, so a record never lands on a document through a link." and a Verification bullet; prose-assessment Constraints "An assessment is written only through real directories: `work` and `work/assessments` must each be a directory, and a symbolic link there, inside the root or out of it, or a file is refused before anything is written, so the record replaced is the record of the section and never another file." and a Verification bullet; generated-docs Constraints `Workplan::write` sentence restated with "below directories that are each a directory and not a symbolic link ... a path below a linked directory, inside the root or out of it" and a Verification bullet (`#constraints` added to the record's `affects`, one item added; `#contract` not added: no Contract sentence states the refusals of `Workplan::write`); complete-gate Contract "The record is deleted only through real directories. ... Otherwise it is an error (exit 2) that names the refused directory, and nothing is run or written. A linked `work/changes` is never read as the directory of change records, so its change is not found." and a Verification bullet; the complete-gate item reworded to "change complete refuses a linked work directory before it runs or writes anything, and a linked work/changes holds no change record, ...". `SECURITY.md` (plain text, no workplan document): the write paragraph gains "every write of a record, of a change record, of a generated block and the removal of a completed record first judges each directory on the way (`work` and `work/<type>`, or `docs` and `docs/capabilities`) and refuses one that is a symbolic link, whether it leads inside the root or out of it, or that is not a directory, before anything runs or is written, so no write lands on another file through a directory link". `CHANGELOG.md`: "`raf-workplan` writes through real directories" entry, including the new `LifecycleError::RecordRefused` (the enum is `#[non_exhaustive]`). Rustdoc of `record::create`, `record::replace`, `record_evidence`, `record_assessment`, `Workplan::write` and `Lifecycle::complete` updated.

Gates before the commit: `cargo fmt` (applied), `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0 (after one `assert_is_empty` fix in the new test), `RUSTDOCFLAGS=-Dwarnings cargo doc -p raf-workplan --no-deps` exit 0, `cargo test -p raf-workplan -p cargo-workplan` exit 0 (53 `test result: ok`), `make docs-sync` (rewrote the three matrices), `check` exit 0. Commit `16d3648`, pre-commit gate exit 0 (reuse, 28, 57). After: roadmap exit 0 (15), `docs_contract`+`repo_gates` exit 0 (28, 57), `check` exit 0. Pushed `0ebc1f7..16d3648`.

### G1: sections in the condition-6 headline (commit `a0ccac0`)

- `crates/raf-workplan/src/lifecycle/completion.rs`: `summary()` counts condition 6 with `sections_without_assessment`, the number of distinct `(subject, section)` pairs of the `Failure::Assessment` failures (a `BTreeSet`), in place of one per failure. The JSON `refused` record of `cargo-workplan` takes its `message` from the same `Completion::headline()`, so text and JSON agree (no separate count). The obligation count, the lines printed and `of_condition(Condition::Assessments)` are unchanged (one per obligation). Doc comments of `Failure::Assessment` and `Lifecycle::complete` restate it.
- Fixtures (bound in `cap:workplan-complete-gate`): `lifecycle::completion::tests::a_section_with_three_moved_inputs_is_one_section_in_the_headline` (three `assessment-stale` for `cap:shape#usage`: `refused: 3 obligations, 1 section without a current accepted assessment`, 4 lines, 3 condition-6 failures; neighbour two sections: `2 obligations, 2 sections`; the same section of another capability: 2 sections) and `complete::a_section_with_three_moved_inputs_is_one_section_in_the_refusal_headline` (scripted lifecycle, the Contract bound to `fx_life::add` and `fx_life::plain`, assessed, then the Contract text and both bodies edited: the headline exactly `refused: 3 obligations, 1 section without a current accepted assessment`, kinds `[assessment-stale]`, 3 condition-6 failures, 4 lines, tree identical; neighbour Contract and Usage each edited: `refused: 2 obligations, 2 sections without a current accepted assessment`).
- Mutant (count back to one per failure, the helper kept used): both new tests fail, the lifecycle one with `left: "refused: 3 obligations, 3 sections without a current accepted assessment"` (the false headline the audit saw); restored.
- Text: complete-gate Contract PROPOSED block after the condition-6 paragraph, its sentence reworded from "Each section without a current accepted assessment is also an `assessment-missing`, `assessment-stale` or `assessment-rejected` obligation, so conditions 1 and 6 both report it, and the refusal prints the line once, under its obligation, with the count `sections without a current accepted assessment`." to "A section without a current accepted assessment is one or more `assessment-missing`, `assessment-stale` or `assessment-rejected` obligations, one `assessment-stale` for each input that moved, so conditions 1 and 6 both report it. The refusal prints each of those lines once, under its obligation, and its headline counts the section once, as `sections without a current accepted assessment`, however many obligations it has: one section with three moved inputs is `3 obligations, 1 section without a current accepted assessment`."; Examples PROPOSED block (two moved inputs of the Usage section: two `assessment-stale` lines, `refused: 2 obligations, 1 section without a current accepted assessment`); Verification PROPOSED bullet. `CHANGELOG.md` condition-6 entry: "a refusal counts the sections without one, each section once however many of its inputs moved".
- Gates: clippy `-D warnings` exit 0; `cargo test -p raf-workplan --lib` exit 0 (216), `--test complete --test lifecycle` exit 0 (35, 31); `coverage tests --crate raf-workplan` no gap row; `make docs-sync` rewrote the complete-gate matrix; `check` exit 0. Pre-commit gate exit 0 (28, 57). After: roadmap 0 (15), `docs_contract`+`repo_gates` 0 (28, 57), `check` 0. Pushed `16d3648..a0ccac0`.

### G5: the time of an output kill (commit `2c422d5`)

- `ExecError::OutputTooLarge` gains `elapsed: Duration` (`crates/raf-workplan/src/exec.rs`); its message is now "`<command>` wrote more than <limit> bytes to <stream> in <t> s and was killed; the run has no verdict". `SystemRunner` fills it with `started.elapsed()`, `FakeRunner` with the scripted `Output::elapsed`. `bound/run.rs` keeps it as the run's time (`Run::new(invocation, Ended::Overflowed, elapsed)`, where it was `Duration::ZERO`) and the binding fails with the error's text as its reason. The enum is `#[non_exhaustive]`, the variant's fields are not: an outside struct pattern without `..` breaks, as `crates/raf-workplan/tests/exec.rs` did (fixed; it now also asserts the time is above zero and is in the message); `CHANGELOG.md` states it ("`raf-workplan` the time of an output kill").
- Fixture `bound::tests::a_kill_at_either_limit_fails_the_binding_with_the_time_it_ran_and_its_limit` (bound in `cap:workplan-complete-gate`): the timeout reason exactly "`cargo test` ran 5.1 s of its 5 s limit and was killed; the run has no verdict" (`Ended::TimedOut`, run time 5.1 s) and the output reason exactly "`cargo test` wrote more than 16777216 bytes to standard output in 2.3 s and was killed; the run has no verdict" (`Ended::Overflowed`, run time 2.3 s), `wall()` at least the time. Real-runner fixture `exec::...never_stops_writing_is_killed_at_the_bound` now asserts `0 < elapsed <= the call's own time`.
- Mutants: the run's time back to `Duration::ZERO` -> the new test fails (`left: 0ns, right: 2.3s`); the message without the time -> fails (`left: "... to standard output and was killed; ..."`); both restored.
- Text: complete-gate Contract PROPOSED block after the claim marker of the Bounds paragraph (the claim on `gate-kills-a-test-past-its-timeout` keeps its paragraph): "A process that runs past its timeout or writes more than the bound is killed, its process group with it on Unix, and the binding counts as failing with the time it ran; ... A bound test that writes more than the bound is killed and fails its binding, and the reason says that it was killed, with the time it ran and the bound, as the reason of a kill at the timeout gives the time it ran and the timeout." (restores the delivered clause "with the time it ran" for both kills); Verification PROPOSED bullet. `SECURITY.md`: the trusted command's "more is an error and no record" -> "more is an error that says the time it ran, and no record"; the bound-run sentence "a process runs with its input closed, its output bounded and a wall-clock limit" gains ", and a bound test or example killed at either limit fails its binding with the time it ran and the limit".
- Gates: `cargo fmt` applied; clippy `-D warnings` exit 0; `make docs-sync` rewrote the complete-gate matrix; `check` exit 0; `cargo test -p raf-workplan -p cargo-workplan` exit 0 (53 `test result: ok`). Pre-commit gate exit 0 (28, 57). After: roadmap 0 (15), `docs_contract`+`repo_gates` 0 (28, 57), `check` 0. Pushed `a0ccac0..2c422d5`.

### Final gates of part A (HEAD `2c422d5`, clean tree)

| Gate | Exit | Result |
| --- | --- | --- |
| `cargo test --workspace` | 0 | 111 `test result: ok`, no failure (host load average up to 37) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | |
| `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps` | 0 | |
| `cargo fmt --check` | 0 | |
| `cargo test -p raf-workplan --test roadmap` | 0 | 15 passed |
| `cargo test --test docs_contract --test repo_gates` | 0 | 28, 57 |
| `cargo run -q -p cargo-workplan -- --root <worktree> check` | 0 | no finding |

### Commits of part A

| Commit | Subject |
| --- | --- |
| `0ebc1f7` | `docs(capabilities): reopen ten workplan capabilities for the hardening fixes` |
| `16d3648` | `fix(raf-workplan): never write a record or a generated block through a linked directory` |
| `a0ccac0` | `fix(raf-workplan): count sections, not obligations, in the condition-six headline` |
| `2c422d5` | `fix(raf-workplan): a kill for too much output records the time it ran` |

Every commit with hooks on (pre-commit exit 0 each), no bypass, no stash; pushed after each green commit (`43c7e02..2c422d5`). Nothing re-assessed or completed (part C). Left for later parts by scope: the PROPOSED blocks are unresolved; assessments and the evidence record `gate-kills-a-test-past-its-timeout` whose inputs moved (`files`, `record`, `sync`, `evidence`, `assessment`, `lifecycle`, `exec`, `bound` modules and the edited sections) are stale and are re-read and re-recorded in step 2; sections in `affects` that part A did not edit (document-model `#usage`, evidence-claims `#contract`, complete-gate `#constraints`, and the sections of okf-profile, workplan-cli, workplan-entity-history, workplan-two-way-coverage, source-index and generated-docs `#verification` beyond the S1 bullet) carry the work of part B.

## Stage 2 remediation B

Start: HEAD `2c422d5`, clean tree, hooks on (`core.hooksPath=.githooks`). Scope: G2, G3, A3, the SECURITY.md/Contract statements (security A1, A2, A4, A5, A6, A7), the carried items (C1) and G4. New text went into `> [!PROPOSED change:<slug>]` blocks of the reopened capabilities (restating the whole paragraph or bullet it replaces, as part A did); `SECURITY.md`, `CHANGELOG.md` and the architecture page `docs/architecture/okf-profile.md` (no PROPOSED blocks outside a Capability) were edited directly. Every new test and public item was bound in the commit that added it. No hook bypass, no stash. After every commit: `cargo test -p raf-workplan --test roadmap` exit 0 (15 passed), `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57), `cargo run -q -p cargo-workplan -- --root <worktree> check` exit 0; pre-commit gate exit 0 each time (fmt, reuse 492/492, 28, 57); pushed after each.

### G2: a `raf:` marker split over lines (commit `27c566c`, okf-profile)

- `crates/raf-okf/src/body/marker.rs`: `family_of` (the `raf:` word after `<!--`, any letter case, `generated:` exempt) shared by `raf_comment` (one line) and the new `split_raf_comment`: a line whose text outside code spans ends with `<!--` reads the first non-blank line below it (a fenced line ends the search: not read); `raf_marker` uses `raf_comment(raw).or_else(split_raf_comment)`. Claim family -> `ErrorKind::MalformedClaim` (its message already says "write exactly ... on its own line"), other word -> `UnknownMarker`, both at the line of `<!--`. PROPOSED lines are skipped as before (the `<!--` line is in the block).
- Fixtures: `raf_okf::test_crate::claims::a_raf_comment_split_over_lines_is_a_marker_error_and_other_split_comments_are_text` (3-line claim, blank line + `  RAF:Claim`, comment opened after a statement, `raf:foo`, `<!--   ` + tab `raf:note`: errors with lines; neighbours plain: split `raf:generated:x` and `RAF:Generated:catalogue`, multi-line SPDX header, ordinary comment, empty comment, `<!--` in a code span, fenced, fence inside the comment, PROPOSED block) and `cargo_workplan::test_crate::check::a_raf_comment_split_over_lines_fails_check_and_other_split_comments_pass` (through the binary: the three error cases exit 1 with `[okf-parse-malformed-claim]`/`[okf-parse-unknown-marker]` naming `docs/capabilities/alpha.md`, equal to the library gate, tree unchanged; neighbours generated/SPDX/ordinary exit 0). Both bound in `cap:okf-profile`.
- Mutant (`split_raf_comment(..).and(None)`, helper kept used): the library test fails; restored.
- Text: okf-profile Contract PROPOSED (the marker-error paragraph restated with "A marker is one line: ... is the same error at the line of `<!--`, `okf-parse-malformed-claim` for `raf:claim` and `okf-parse-unknown-marker` for another word ... A multi-line `raf:generated:` comment, an SPDX header written over several lines and every other multi-line comment are plain comments."); Verification PROPOSED bullet. Architecture page (direct, after the claim-marker rules): "A marker is one line. A comment that `<!--` opens at the end of a line, outside its code spans, and whose first non-blank text on the lines below is `raf:` in any letter case, other than `raf:generated:`, is the error of that word on the line of `<!--` ...". Key-order test and docs_contract green. CHANGELOG entry.

### G3: `evidence claims` while a capability does not load (commit `99d5487`, evidence-claims, cli)

- `Workplan::evidence_claims`: after the id check, `unloaded()` filtered to `cap:`; non-empty -> new `EvidenceLookupError::UnloadedCapabilities { ids }` ("cap:x did not load, so a claim in it is unknown; fix its document first"); the enum is `#[non_exhaustive]`. The CLI maps it to exit 2 (stderr, nothing on stdout).
- Fixtures: `raf_workplan::test_crate::evidence::the_claims_of_a_record_are_refused_while_a_capability_document_does_not_load` (bound in evidence-claims: a second capability `zeta` with `state: unknown` -> refused for `bound`, `evidence:bound`, `nothing`; `Bad Id` still `BadId`; neighbours an unloaded change record and an unloaded evidence record (asserted not loaded) and the clean repository answer one claim) and `cargo_workplan::test_crate::evidence::claims_exit_two_naming_a_capability_that_does_not_load_and_answer_beside_other_records` (bound in cli: exit 2 text and JSON naming `cap:beta`, stdout empty, tree unchanged; unloaded change and evidence record exit 0 while `check` names them).
- Mutant (the refusal never taken): both tests fail; restored.
- Text: evidence-claims Contract PROPOSED ("The dependency view" restated with the refusal and the exit), Verification PROPOSED bullet; cli Contract PROPOSED ("Exit codes of the record commands" restated with "`evidence claims` also exits 2, naming each capability document that exists and does not load ..."), Verification PROPOSED bullet. CHANGELOG entry.

### A3: escaped text of `evidence claims` and `assess --context` (commit `6efed08`, cli)

- New `pub mod raf_workplan::escape`: `hidden` and `shown` moved out of `history/render.rs` (`shown` is `pub(crate)`, used by `history::render::table_cell`/`table_code`, which coverage uses), new `pub fn shown_lines` (keeps line feed and tab, escapes every other control incl. CR, ESC, BEL, and the bidi formatting characters). Three users: history tables, coverage tables (through `table_cell`), and the CLI (`evidence::claim_record`, `assess::context_record` text). JSON unchanged (serde escapes C0).
- Fixtures (bound in cli): `raf_workplan::escape::tests::each_control_and_bidirectional_formatting_character_is_escaped`, `...::lines_keep_their_line_feeds_and_tabs_and_lose_every_other_control`, `cargo_workplan::test_crate::evidence::a_claimed_statement_reaches_the_terminal_escaped_and_json_escapes_it_as_json` (ESC `[2J`, OSC `]0;PWNED` BEL, U+202E in a claimed statement with a tab-indented line: exact text output, JSON has `\u001b` and reads back raw), `cargo_workplan::test_crate::assess::the_text_of_a_section_reaches_the_terminal_escaped_and_json_escapes_it_as_json` (same in the Contract section: no raw ESC/BEL/U+202E, equals `shown_lines(library context)`, line count kept; plain neighbour unchanged). Code binding `raf_workplan::escape`, example binding `raf_workplan::escape::shown_lines` (doctest).
- Mutants (escape dropped in each CLI renderer): each fixture fails; restored.
- Text: cli Constraints PROPOSED bullet ("Document text reaches the terminal escaped ..."), Verification PROPOSED bullet. Crate docs of `raf-workplan` name `escape`. CHANGELOG entry. SECURITY.md (commit `0379a26`): the text output of `history`, `coverage`, `evidence claims` and `assess --context` escapes control and bidi characters; JSON escapes as JSON does.

### SECURITY.md and the trusted-command statements (commit `38486a5`, evidence-claims)

- A6 code: `exec::trusted::text_of` (the quoting of `TrustedArgv::text`) now also writes the `Display` of a trusted `Invocation`, which every `ExecError` message quotes, so `sh -c 'yes >&2'` and `sh -c yes '>&2'` read apart. Fixture `raf_workplan::exec::trusted::tests::an_error_about_the_command_quotes_its_words_as_the_record_stores_them` (bound in evidence-claims): both commands, `Display == text()`, the `OutputTooLarge` message starts with the quoted command, a real `SystemRunner` spawn of `./no such tool` gives `Spawn { command == text() }` and "cannot start `'./no such tool' 'a b' ''`", plain neighbour. Mutant (Display back to the unquoted join): fails; restored. All evidence/exec tests pass (no test pinned the unquoted form).
- evidence-claims Constraints PROPOSED bullets: supervision only while `cargo-workplan` runs (Ctrl-C/hang-up reaches the CLI and not the command's own process group; then no limit and no record); a `setsid`/`setpgid` descendant escapes the kill at the limit; elsewhere than Unix only the process is killed; a relative program (`./tool.sh` typed below the root) and a bare name with `.` on `PATH` resolve in the root while the record keeps the command as typed; error messages quote the words as `TrustedArgv::text` writes them; at most 1024 values bounds what `evidence record` writes, a hand-written record may hold more, bounded by the 1 MiB document bound; the way to a record is judged by path, not held handles (a concurrent local writer is outside the trust model). Verification PROPOSED bullet for the quoting fixture.
- SECURITY.md (direct): the trusted-command paragraph gains the Ctrl-C/process-group and `setsid` limits, the relative-program rule, the quoted messages and the hand-written-record bound; the write paragraph gains the by-path note; the Bounds paragraph gains the exported-items carry cost with the figures of reviews/security.md A4: "a glob chain of 200 modules, each with 40 re-exports of another crate, is refused by the budget after 75 s at a peak of 138 MB in a release build, and 500 such modules (about 0.5 MB of source) had no answer after 4 minutes of CPU at 53 MB" (charged to the item-bytes budget left after the load; ends in a typed error, never a wrong list; no time bound). CHANGELOG entry.

### Carried items

| Item | Commit | Capability (record) | What |
| --- | --- | --- | --- |
| `binding_site` first glob | `0738a45` | source-index | `raf_source::test_crate::exported_items::a_re_export_two_globs_carry_is_written_as_the_first_glob_that_carries_it` (two globs of `p_one` bring the same `f`: written as the first in either order, with its line; one-glob neighbour); mutant `.min()` -> `.max()` fails; Verification PROPOSED bullet; bound |
| `exports.rs` wording | `0738a45`, `a378f46` | source-index | "a later field is not a breaking change" -> "adding a field is not a breaking change" (three structs) and "a later option" -> "adding an option" (`ExportOptions`), the Contract's words; then the same in `target.rs`, `modtree.rs`, `workspace.rs`, `tree/mod.rs`, `facts.rs` and `LoadOptions` (`options.rs`); record item added and widened |
| SymbolLog wording | `3e9bc88` | entity-history | message now "`git log -L` could not read the lines of the bound symbol `<b>` in `<f>` from `HEAD`: <git>; when `<f>` is not committed, or `HEAD` holds fewer of its lines than the working copy, one way to read them is to commit `<f>` with the symbol at the lines it has now and ask again"; library test pins start and end, CLI test pins both parts; Contract PROPOSED (Binding history paragraph gains the failure), Constraints PROPOSED (the `git log -L` bullet restated), Verification PROPOSED; `#constraints` added to the record's `affects` |
| Crate-root digest | `f6095fe` | prose-assessment | `raf_workplan::test_crate::digest::a_crate_root_stands_for_its_own_file_and_an_edit_below_it_leaves_its_digest` (edits in `src/tools.rs` and `src/tools/deep.rs` leave `demo`'s digest and move `demo::tools`'s; an edit of `src/lib.rs` moves it); mutant D4 (root treated as a module) fails; Constraints PROPOSED (digest bullet restated with the crate root), Verification PROPOSED; bound in prose-assessment |
| `has_unloaded_documents` comment | `f767223` | document-model | comment: the id noted per file is prefix + file stem, compared with header ids; exact because `okf-id-file-name` refuses a document whose id is not its stem (a mismatching file does not load) |
| `docs_sync` example lock | `3d8928a` | generated-docs | `static EXAMPLE: Mutex<()>` and `serial()` in `tests/sync.rs`, held by the four tests that run the example from first process to last; generated-docs Verification PROPOSED bullet (test only) |
| `super::` re-exports | `7ade138` | two-way-coverage | see below: fixed, not stated |
| G4 coverage cascade | `7ade138` | two-way-coverage | built (cheap), see below |

### `super::` and G4 (commit `7ade138`, two-way-coverage)

- The plan said "state the limit (a false gap row, never a missed gap)". Checking it showed the second half false: `named_paths` read the path from the module of the declaration's file and kept `super::` as written, so `pub use super::gen::Made;` in `src/api.rs` named `made::api::super::gen::Made`; its ancestors include `made::api`, so a binding of the declaring module covered it (a missed gap) while a binding of `made::gen` did not (a false gap). Inline modules were also read as the file's module (a bare `deep::*` in inline `mod api` was judged not local and never counted). Fixed instead of stated: new `Judge::declaring_modules` (the innermost inline `mod` of the file whose braces hold the declaration's line, else the file's module, per reach); `named_paths` strips `self::`, climbs one module per `super::` (a climb above the root gives no path), and `is_local` resolves the first segment from the declaring module. Fixture `raf_workplan::test_crate::coverage::a_re_export_written_through_super_names_the_path_super_reaches` (file module: item `made::gen::Made`, covered by `made::gen`, not by `made::api`; inline two levels with `super::super::` and a glob of an inline module counted; `self::` neighbour). Mutants (no climb; inline modules ignored): each fails; restored.
- G4: `coverage::check` passes `gaps = !(a cap: id in unloaded())` to `findings`; while a capability document does not load, no `coverage-item-unbound`/`-test-unbound`/`-reexport-unbound`; surface-unreadable and exclusion-unmatched still reported. Capability-only (not `has_unloaded_documents`, which also counts change and architecture documents that bind nothing). Fixture `raf_workplan::test_crate::coverage::a_capability_that_does_not_load_gives_its_own_findings_and_no_cascade_of_gaps` (broken `zeta`: its finding and no gap; unmatched exclusion still reported; unloaded change record: gaps stay; `zeta` loaded with one binding dropped: exactly that gap). Mutant (gaps always on): fails; restored.
- Text: two-way-coverage Contract PROPOSED ("Unresolved re-exports" restated with how the named path is read and the `src/api.rs` example; "What `check` enforces" restated with the G4 rule), Verification PROPOSED (two bullets). Record: `affects` now #contract and #verification (#constraints removed: not edited), items and summary rewritten to the fix. Module docs of `coverage.rs`; the misplaced `is_local` doc comment moved back onto `is_local`. CHANGELOG entry.

### Final gates of part B (HEAD `0379a26`, clean tree; `a378f46` changes doc comments only and was followed by `cargo test -p raf-source` exit 0, `cargo doc -p raf-source --no-deps` with `-Dwarnings` exit 0, `cargo fmt --check` exit 0 and the three per-commit gates)

| Gate | Exit | Result |
| --- | --- | --- |
| `cargo test --workspace` | 0 | 111 `test result: ok`, no failure |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | |
| `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps` | 0 | |
| `cargo fmt --check` | 0 | |
| `cargo test -p raf-workplan --test roadmap` | 0 | 15 passed |
| `cargo test --test docs_contract --test repo_gates` | 0 | 28, 57 |
| `cargo run -q -p cargo-workplan -- --root <worktree> check` | 0 | no finding |
| `coverage code|tests --crate raf-workplan|cargo-workplan` | 0 | no gap row in any of the four |

### Commits of part B

| Commit | Subject |
| --- | --- |
| `27c566c` | `fix(raf-okf): a raf marker split over lines is a marker error, never a plain comment` |
| `99d5487` | `fix(raf-workplan): evidence claims fails hard while a capability document does not load` |
| `6efed08` | `fix(cargo-workplan): escape control and bidirectional characters in the text of evidence claims and assess --context` |
| `38486a5` | `fix(raf-workplan): quote the caller's command in messages and state the limits of the trusted command` |
| `0738a45` | `test(raf-source): pin the first carrying glob as the site of a re-export two globs bring in` |
| `3e9bc88` | `fix(raf-workplan): say that git log -L could not read the lines, and name committing as one way to read them` |
| `f6095fe` | `test(raf-workplan): pin that an edit below a crate root leaves the digest of its binding` |
| `f767223` | `docs(raf-workplan): say why has_unloaded_documents may compare file stems with ids` |
| `3d8928a` | `test(raf-workplan): hold one lock in every test that runs the docs_sync example` |
| `7ade138` | `fix(raf-workplan): read the path an unresolved re-export names from its module and report no coverage gap while a capability does not load` |
| `0379a26` | `docs(security): state that the text output escapes control and bidirectional characters` |
| `a378f46` | `docs(raf-source): word every non-exhaustive note as adding a field or an option` |

### Consciously accepted (for the PR body, with reasons)

- G4 is built, not accepted (above).
- Security A4: the exported-items carry has no time bound; stated in SECURITY.md with the measured figures; the result is always a typed error or a correct list, and the trust model makes the workspace trusted build input.
- Security A5: a hand-written evidence record may hold more than 1024 values; bounded by the 1 MiB document bound (60000 values load and validate in bounded time); stated in SECURITY.md and the evidence-claims Constraints.
- Security A7: write checks judge paths, not held handles; a concurrent local writer can already edit the workspace and is outside the trust model; stated.
- Security A1/A2: kept as behaviour (no signal forwarding, no cwd-relative resolution) and stated in SECURITY.md and the evidence-claims Constraints.
- Gap A5 maintainability: module sizes (`graph.rs`, `exports.rs`, `assessment.rs`, `coverage.rs`, `cli.rs`), JSON rendered two ways (hand-rolled in the library, serde in the binary), one `INVENTORY` edit per record, type bindings that do not hash method bodies (stated in prose-assessment Constraints: name the method to include it), and no gate that keeps records current between changes (a change that lists the capability surfaces the staleness).
- Gap A6: a completed change's history is not reachable through `history change:` (the record is deleted; Git keeps it) and `history cap:` lists no change records.
- Validation A4: the final local gates and hosted CI cover it.

### Left for part C by scope

All PROPOSED blocks added here are unresolved; no assessment or evidence was recorded and nothing was completed. Inputs moved in this part (stale until step 2): `raf-okf` `body/marker.rs`; `raf-workplan` `evidence.rs`, `escape.rs` (new), `history.rs`, `history/render.rs`, `exec/invocation.rs`, `exec/trusted.rs`, `coverage.rs`, `model.rs`, `lib.rs` docs; `cargo-workplan` `evidence.rs`, `assess.rs`; `raf-source` `exports.rs`, `target.rs`, `modtree.rs`, `workspace.rs`, `tree/mod.rs`, `facts.rs`, `options.rs` (doc comments only, so the digests of the items they document); and the edited sections of okf-profile, evidence-claims, workplan-cli, entity-history, prose-assessment, source-index, generated-docs and two-way-coverage. Pre-existing and unrelated: `cargo doc --document-private-items` warns on `lifecycle/session.rs` and `lookup.rs` (private items; the gate documents public items only and passes). The same "a later field/option" wording in the other `raf-source` files (`target.rs`, `modtree.rs`, `workspace.rs`, `tree/mod.rs`, `facts.rs`, `options.rs`) was reworded too (commit below), since the Contract words it "adding a field/option"; no delivered capability or evidence record binds a `raf-source` item, so only reopened capabilities' records move.

## Stage 2 remediation C1

Start: HEAD `a378f46`, clean tree, hooks on. Scope: resolve every `> [!PROPOSED change:<slug>]` block of the ten reopened capabilities into plain current text, one commit per capability; tense scan; bindings; record items and `affects`; `change reconcile`. No assessment recorded, no `delivered` set, nothing completed (part C2). Per commit: pre-commit hook exit 0, then `cargo test -p raf-workplan --test roadmap`, `cargo test --test docs_contract --test repo_gates`, `cargo run -q -p cargo-workplan -- --root <worktree> check`, then push. Tense scan: `will`, `is planned`, `proposed`, `not yet`, `to be implemented`, `open questions`, `still`, `now`, `later`, `follow-up`, `no longer` outside fences, code spans and generated blocks.

### workplan-document-model (commit `2a8a0b9`)

- Contract, **Writing records.**: the delivered paragraph is replaced by the resolved block. Reworded: "creating the directory of its type when it is missing and refusing a link or a file on the way;" -> "creating the directory of its type when it is missing;" plus the new sentences "Each judges the way to the record before anything is written, as every write of a record does: `work` and the directory of the record type must each be a directory, and a symbolic link there, whether it leads inside the root or out of it, a file or anything else is refused, naming that directory. A record is therefore written in the record directory of the root and never through a link onto another file, such as a document." (superseded: the refusal of `create` alone becomes the judgement of every record write, `files::target`). The rest of the paragraph is unchanged.
- Verification: "Roadmap fixtures: the group and slug order, the gaps of an `implementing` capability, and this repository with every capability once." -> the same bullet continued "... once and each capability it has built either `delivered`, or `implementing` with the open change record named after it, with no gap in either case. The rule refuses ... and accepts it `delivered` with no record or reopened under its own record." (widened: the guard of `tests/roadmap.rs`, `built_capabilities_hold`, and its fixture). New bullet after Writer fixtures: "Link fixtures for the writer ...". Removed: nothing.
- Record: `affects` drops `#usage` (Usage was not edited); items 1-3 `implemented` (`files::target` used by `record::create`/`record::replace`, `evidence::target_fits`, `record_assessment`; `built_capabilities_hold` and `a_built_capability_is_delivered_or_reopened_under_its_own_change_and_has_no_gap`; the `has_unloaded_documents` comment in `model.rs`).
- Tense scan: only pre-existing system terms (`PROPOSED` marker names, "open questions" of the `considered` row, "what applies now", "now carries the claim"); nothing in the edited text. `make docs-sync`: nothing rewritten. `check` exit 0. Reconcile exit 1, only `assessment-stale` (contract 4, verification 6).
- Gates after the commit: roadmap exit 0 (15), `docs_contract`+`repo_gates` exit 0 (28, 57), `check` exit 0; pre-commit hook exit 0; pushed.

### workplan-evidence-claims (commit `d0585ea`)

- Contract, **The dependency view.**: the delivered paragraph is replaced by the resolved block; its three delivered sentences stay word for word, and between the `evidence claims` and the `evidence for` sentence the refusal is inserted ("While a capability document exists and does not load, ... is an error (`EvidenceLookupError::UnloadedCapabilities`, exit 2) naming each such document ...; a change record or an evidence record that does not load holds no claim and leaves the answer as it is."). Removed: nothing.
- Constraints: the six resolved bullets are appended to the list (real directories; supervision only while `cargo-workplan` runs, Ctrl-C and the process group, `setsid`/`setpgid`, non-Unix; relative program and `.` on `PATH`; quoted words in error messages; hand-written records over 1024 values; judgement by path). No delivered bullet removed or reworded.
- Verification: the three resolved bullets are appended. Reworded on resolution: "while a malformed id is still refused as one" -> "while a malformed id is refused as malformed first" (tense scan: "still"; the test checks `BadId` before the unloaded refusal, `evidence.rs` line 769 before 806). Removed: nothing.
- Record: `affects` (contract, constraints, verification) = the edited sections; items 1-5 `implemented` (`evidence::target_fits` through `files::target`; `Workplan::evidence_claims` `UnloadedCapabilities` and the CLI exit 2; the Constraints bullets now plain text; `exec::trusted::text_of` for `Invocation`'s `Display`; the 1024-values and by-path bullets).
- Tense scan: "no longer resolves"/"no longer supported" and "another digest now" are delivered statements of a state, not of the change; nothing else. `make docs-sync` nothing rewritten; `check` exit 0; reconcile exit 1, only `assessment-stale` (contract 4, constraints 4, verification 3).
- Gates after the commit: roadmap exit 0 (15), `docs_contract`+`repo_gates` exit 0 (28, 57), `check` exit 0; pre-commit hook exit 0; pushed.

### workplan-prose-assessment (commit `fdae8ec`)

- Constraints: the real-directories bullet is placed after "An assessment never edits the section it assesses; recording writes only its own record." The delivered digest bullet is replaced by the resolved one: "... so any edit inside it moves the input." gains "The binding of a crate root stands for its own span, the whole of its file, and for nothing below it, so an edit of a module file below the root leaves the input and an edit of the root's own file moves it." (widened; pinned by `test_crate::digest::a_crate_root_stands_for_its_own_file_and_an_edit_below_it_leaves_its_digest`). Removed: nothing.
- Verification: the two resolved bullets are appended after the completion-gate bullet. Removed or reworded: nothing.
- Record: `affects` (constraints, verification) = the edited sections; items 1-2 `implemented` (`Workplan::record_assessment` through `files::target`, file at `work/assessments` refused in `assessments::an_assessment_is_never_written_through_a_linked_directory_and_a_real_one_records`; the crate-root digest test).
- Tense scan: hits only in delivered text that states a state ("digests now", "no longer resolves", "the evidence still missing", "an edit under them still makes it stale"), none in the edited text. `make docs-sync` nothing rewritten; `check` exit 0; reconcile exit 1, only `assessment-stale` (constraints 3, verification 1).
- Gates after the commit: roadmap exit 0 (15), `docs_contract`+`repo_gates` exit 0 (28, 57), `check` exit 0; pre-commit hook exit 0; pushed.

### workplan-complete-gate (commit `6f086fc`)

- Contract, condition-6 paragraph: replaced by the resolved block. Reworded (delivered): "Each section without a current accepted assessment is also an `assessment-missing`, `assessment-stale` or `assessment-rejected` obligation, so conditions 1 and 6 both report it, and the refusal prints the line once, under its obligation, with the count `sections without a current accepted assessment`." -> "A section without a current accepted assessment is one or more ... obligations, one `assessment-stale` for each input that moved, so conditions 1 and 6 both report it. The refusal prints each of those lines once, under its obligation, and its headline counts the section once, ... one section with three moved inputs is `3 obligations, 1 section without a current accepted assessment`." (superseded: G1, `Completion::summary` counts distinct sections). The other sentences of the paragraph are unchanged.
- Contract, record removal: the resolved paragraph "The record is deleted only through real directories. ..." follows the paragraph that ends "Git holds the record's history." Reworded on resolution: "an error (exit 2) that names the refused directory" -> "names the refused path" (`judge_record` names the record itself when it is not a regular file, a directory otherwise).
- Contract, **Bounds.**: "... and the binding counts as failing; a process that cannot be started ..." -> "... and the binding counts as failing with the time it ran; ..." (restores the delivered clause for both kills, G5). The claim paragraph "A bound test that runs past its timeout ..." and its marker `gate-kills-a-test-past-its-timeout` keep their text and place. Reworded on resolution: the proposed last sentence "A bound test that writes more than the bound is killed and fails its binding, and the reason says that it was killed, with the time it ran and the bound, as the reason of a kill at the timeout gives the time it ran and the timeout." becomes its own paragraph after the claim marker, "A bound test that writes more than the bound is killed and fails its binding in the same way, and the reason says that it was killed, with the time it ran and the bound." (the comparison clause is the claim paragraph just above it; the claim stays attached to the timeout paragraph only).
- Examples: the resolved paragraph (two moved inputs of Usage: two `assessment-stale` lines, `refused: 2 obligations, 1 section without a current accepted assessment`) follows the condition-6 example. Removed: nothing.
- Constraints (in `affects` since the reopening and not yet edited, so `section-untouched`): "A document edited between the second judgement and the writes is not detected: the rewrite replaces a file without comparing it with the text it was computed from, which is sound only because the workspace is trusted input." -> "A document edited, or a directory on the way to the record replaced by a link, between the second judgement and the writes is not detected: the rewrite replaces a file without comparing it with the text it was computed from, and the removal goes by the path judged, which is sound only because the workspace is trusted input." (widened to what is built: `judge_record` runs inside both `evaluate` calls, `write` then removes by path with `fs::remove_file`; the by-path note of `SECURITY.md` and the evidence-claims Constraints).
- Verification: the three resolved bullets are appended after "The completion writes only the rewrite and the removal ...". Removed: nothing.
- Record: `affects` (contract, examples, constraints, verification) = the edited sections; items 1-3 `implemented` (`summary()` with `sections_without_assessment`; `OutputTooLarge.elapsed` kept as the run time in `bound/run.rs`; `judge_record` and `RecordRefused`, `ChangeLookupError::Unknown` for a linked `work/changes`).
- Tense scan: hits only in delivered text (`still` as "nevertheless", "no longer builds", "what applies now", the Scenario B "open questions"); none in the edited text. `docs preview workplan-complete-gate` exit 0. `make docs-sync` nothing rewritten; `check` exit 0; reconcile exit 1: `evidence-stale` 4 (the claim's record, inputs `bound`, `exec`, `Failure`, `Lifecycle::complete` moved in part A), `assessment-stale` contract 5, examples 5, verification 5, `assessment-missing` constraints 1; the earlier `section-untouched` of constraints is gone.
- Gates after the commit: roadmap exit 0 (15), `docs_contract`+`repo_gates` exit 0 (28, 57), `check` exit 0; pre-commit hook exit 0; pushed.

### okf-profile (commit `84e9f09`)

- Contract, the marker-error paragraph of the body model: replaced by the resolved block. Its delivered sentences stay word for word; inserted after "... so a misspelt claim marker never reads as a plain comment.": "A marker is one line: an HTML comment that `<!--` opens at the end of a line, outside its code spans, and whose first non-blank text on the lines below is `raf:` in any letter case and not `raf:generated:`, is the same error at the line of `<!--`, `okf-parse-malformed-claim` for `raf:claim` and `okf-parse-unknown-marker` for another word, so a claim marker split over lines never reads as a plain comment either. A multi-line `raf:generated:` comment, an SPDX header written over several lines and every other multi-line comment are plain comments." The `Body::replace_spans` sentences only reflowed. Removed: nothing.
- Verification: the split-marker bullet is appended after the `raf:generated:` letter-case bullet. Reworded on resolution: "Through `check`, the first two in a capability are `okf-parse-malformed-claim` and the third `okf-parse-unknown-marker`" -> "Through `check`, the three-line claim marker and the one with a blank line in a capability are `okf-parse-malformed-claim` and the `raf:foo` comment `okf-parse-unknown-marker`" ("the first two"/"the third" did not match the order of the bullet, whose third case is a malformed claim; the CLI fixture `check::a_raf_comment_split_over_lines_fails_check_and_other_split_comments_pass` runs exactly those three).
- Record: `affects` (contract, verification) = the edited sections; items 1-2 `implemented` (`body/marker.rs`: `raf_comment` reads `<!-- raf:claim ...` left open on its line as a malformed claim, `split_raf_comment` reads `<!--` at the end of a line; `family_of` exempts `raf:generated:`; neighbours in both fixtures).
- Tense scan: "a later `---` line" and "no longer parses" are delivered statements of order and state; nothing in the edited text. `make docs-sync` nothing rewritten; `check` exit 0; reconcile exit 1, only `assessment-stale` (contract 1, verification 1).
- Gates after the commit: roadmap exit 0 (15), `docs_contract`+`repo_gates` exit 0 (28, 57), `check` exit 0; pre-commit hook exit 0; pushed.

### workplan-cli (commit `f96ff1c`)

- Contract, **Exit codes of the record commands.**: replaced by the resolved block. Its delivered sentences stay word for word; inserted before "A section given to `assess --list` ...": "`evidence claims` also exits 2, naming each capability document that exists and does not load, with nothing on standard output, because a claim in such a document may name the record; a change record or an evidence record that does not load holds no claim and leaves its answer and exit 0 as they are." Removed: nothing.
- Constraints: the escaped-text bullet ("Document text reaches the terminal escaped. ...", `raf_workplan::escape::shown_lines`, the bidi set U+061C, U+200E, U+200F, U+202A-U+202E, U+2066-U+2069 as in `escape::hidden`) is appended. Removed or reworded: nothing.
- Verification: the two resolved bullets are appended after the JSON-pins bullet. Removed or reworded: nothing.
- Record: `affects` (contract, constraints, verification) = the edited sections. Item 2 reworded to what is built: "The text output of evidence claims and assess --context shows each control and bidirectional formatting character of a document escaped, as the history and coverage tables do." -> "... shows each control character of a document other than the line feed and the tab, and each bidirectional formatting character, escaped, as the history and coverage tables escape their cells." (`shown_lines` keeps LF and tab; the tables use `shown`, which escapes them too). Items 1-2 `implemented` (`evidence.rs` of `cargo-workplan` maps `UnloadedCapabilities` to exit 2; `evidence::claim_record` and `assess::context_record` call `shown_lines`).
- Tense scan: hits only in the delivered `assess --list` sentence ("can no longer be assessed", "no longer resolving", "a bare `--list` still shows", a present fact); nothing in the edited text. `make docs-sync` nothing rewritten; `check` exit 0; reconcile exit 1, only `assessment-stale` (contract 2, constraints 2, verification 2).
- Gates after the commit: roadmap exit 0 (15), `docs_contract`+`repo_gates` exit 0 (28, 57), `check` exit 0; pre-commit hook exit 0; pushed.

### workplan-entity-history (commit `7355d1a`)

- Contract, **Binding history.**: replaced by the resolved block; its delivered sentences stay word for word, and the failure sentence is appended: "When `git log -L` cannot read the lines of a symbol from `HEAD`, as for a bound file that is not committed or that `HEAD` holds fewer lines of, the whole history fails with an error (`HistoryError::SymbolLog`) that names the binding and the file, says that `git log -L` could not read the lines from `HEAD` with what `git` said, and names committing the file with the symbol at the lines it has now as one way to read them, not as the cause." Removed: nothing.
- Constraints, the `git log -L` bullet: "... with an error that names the binding and the file and says to commit it." -> "... with an error that names the binding and the file, says that `git log -L` could not read the lines from `HEAD`, and names committing the file as one way to read them." (superseded: the `SymbolLog` message, `history.rs` line 422, gives committing as one way and not as the cause). The rest of the bullet is unchanged.
- Verification: "A bound file whose lines `git log -L` cannot read from `HEAD` (as for a file that is not committed): the error names the binding and the file and says to commit it, while a failing log of the document stays a failure of `git`." -> the resolved bullet (library: the message opens with what `git log -L` could not read, naming binding and file, quotes `git` and ends with committing as one way; binary: exit 2 with that message until committed). Reworded on resolution: "starts with `git log -L` could not read the lines from `HEAD`" -> "starts by saying that `git log -L` could not read the lines of the binding in the file from `HEAD`, naming both" (the pinned text is "`git log -L` could not read the lines of the bound symbol `<b>` in `<f>` from `HEAD`: "), and "(a fixture of the [command line](./workplan-cli.md))" added to the binary half, since `cargo_workplan::test_crate::history::a_bound_file_that_is_not_committed_is_exit_two_naming_it_until_it_is` is bound in `cap:workplan-cli`, as the version fixture bullet already says of its binary half.
- Record: `affects` (contract, constraints, verification) = the edited sections; item 1 `implemented` (`HistoryError::SymbolLog` message; `tests/history.rs` line 1150 pins the start, the CLI test both parts).
- Tense scan: "what applies now", "no longer resolves", "a later commit", "reasons that still govern" are delivered statements; "the lines it has now" (new) names the working copy, not the change. `make docs-sync` nothing rewritten; `check` exit 0; reconcile exit 1, only `assessment-stale` (contract 2, constraints 2, verification 2).
- Gates after the commit: roadmap exit 0 (15), `docs_contract`+`repo_gates` exit 0 (28, 57), `check` exit 0; pre-commit hook exit 0; pushed.

### workplan-two-way-coverage (commit `824f80e`)

- Contract, **Unresolved re-exports.**: replaced by the resolved block; delivered sentences word for word, the reading of the named path appended. Reworded on resolution: "as `rustc` reads a `use` path: `crate::` from the crate root, `self::` and a bare path from that module, and each `super::` one module up" -> "...: `crate::` from the crate root, a path that begins with the name of a crate of the workspace from that crate, `self::` and any other bare path from that module, and each `super::` one module up" (`Judge::named_paths`, `coverage.rs` line 949-958, takes a path whose first segment is a workspace crate as written before it climbs; the proposed text left that rule out).
- Contract, **What `check` enforces.**: replaced by the resolved block; delivered sentences word for word, the G4 sentences inserted after "A crate outside `scope` is not enforced, and its gaps are in the reports." ("While a capability document exists and does not load, `check` reports none of the three unbound kinds ... the gaps are judged once it loads. A change record that does not load ... reported either way."). Removed: nothing.
- Verification: the two resolved bullets are appended after the `Workplan::coverage_report` example bullet. Reworded on resolution: "`check` reports its load finding and no `coverage-item-unbound`, `coverage-test-unbound` or `coverage-reexport-unbound`, and still reports an exclusion that matches nothing" -> "`check` reports its load finding and an exclusion that matches nothing, and no `coverage-item-unbound`, ..." (tense scan: "still").
- Record: `affects` (contract, verification) = the edited sections; items 1-2 `implemented` (`declaring_modules`, `named_paths`, `is_local`; `coverage::check` passing `gaps = !(a cap: id in unloaded())`; fixtures `a_re_export_written_through_super_names_the_path_super_reaches` and `a_capability_that_does_not_load_gives_its_own_findings_and_no_cascade_of_gaps`).
- Tense scan: nothing outside the `PROPOSED` marker names. `make docs-sync` nothing rewritten; `check` exit 0; reconcile exit 1, only `assessment-stale` (contract 2, verification 2).
- Gates after the commit: roadmap exit 0 (15), `docs_contract`+`repo_gates` exit 0 (28, 57), `check` exit 0; pre-commit hook exit 0; pushed.

### workplan-generated-docs (commit `1da446b`)

- Constraints, the `Workplan::write` bullet: "`Workplan::write` writes only to a regular file that already exists or to the new index in an existing `docs/capabilities`, and refuses a symbolic link, a directory, a path outside the root and a path listed twice." -> "... in an existing `docs/capabilities`, below directories that are each a directory and not a symbolic link, and refuses a symbolic link, a directory, a path outside the root, a path below a linked directory, inside the root or out of it, and a path listed twice. Every target is judged before the first byte is written." (widened: `sync::judge` through `files::target` for every edit in the first loop of `Workplan::write`, before any `stage`). The temporary-file and rename sentences are unchanged.
- Verification: the two resolved bullets are inserted after "Write fixtures on disk ..." and before "Preview fixtures ...". Removed or reworded: nothing.
- Record: `affects` (constraints, verification) = the edited sections. Item 2 reworded to what is built: "The test that runs the docs sync example takes the lock the other process tests of the crate take, so no other test rewrites the executable while it runs." -> "The four tests that run the docs_sync example hold one lock of their test binary from their first process to their last, as the other process tests of the crate hold the lock of theirs, so no test of that binary rewrites the example while another starts it." (`tests/sync.rs` `static EXAMPLE` and `serial()`, taken at lines 398, 460, 697, 719; each test binary has its own lock, `SPAWN` in `exec.rs` and `gate.rs` among them, so there is no one shared lock). Items 1-2 `implemented`.
- Tense scan: "a later file" (order) only. `make docs-sync` nothing rewritten; `check` exit 0; reconcile exit 1: `assessment-missing` constraints 1, `assessment-stale` verification 10 (the section now names `Workplan::write` strongly, so the other bound symbols are no longer inputs: for C2 to re-assess).
- Gates after the commit: roadmap exit 0 (15), `docs_contract`+`repo_gates` exit 0 (28, 57), `check` exit 0; pre-commit hook exit 0; pushed.

### source-index (commit `6690803`)

- Verification: the resolved bullet ("A re-export of an item of another crate of the workspace that two `pub` globs bring in is written as the glob that comes first, with its line, in either order of the two, beside one glob alone, written as that glob.") follows the exported-item fixtures bullet. Removed or reworded: nothing.
- Record: `affects` (verification) = the edited section; items 1-2 `implemented` (`exported_items::a_re_export_two_globs_carry_is_written_as_the_first_glob_that_carries_it`; `grep 'later field|later option'` over `crates/raf-source/src` finds nothing, ten "adding a field/an option" notes, as Contract lines 1514-1519 word them).
- Tense scan: the hits are delivered statements of the resolution algorithm ("may still bind", "still undecided") and of editions ("later editions"); none in the edited text. `make docs-sync` nothing rewritten; `check` exit 0; reconcile exit 1, only `assessment-stale` (verification 3).
- Gates after the commit: roadmap exit 0 (15), `docs_contract`+`repo_gates` exit 0 (28, 57), `check` exit 0; pre-commit hook exit 0; pushed.

### Bindings

No binding was edited in C1. Every test added in parts A and B is a `#[test]` bound in its capability (the seven link fixtures are `#[cfg(unix)]` and run on Linux; none is Windows-only), `check` prints nothing (no unbound test, no unresolved binding), and the one example binding added in part B, `raf_workplan::escape::shown_lines`, is a plain ```` ```rust ```` doc test (neither `no_run` nor `compile_fail`). The other bound examples are those `verify` ran in 11A-11C. `verify <slug>` is left to C2 at completion time.

### Reconcile at HEAD `6690803` (each exit 1; only assessment and evidence kinds)

| Capability | Lines |
| --- | --- |
| workplan-document-model | 10 `assessment-stale` |
| workplan-evidence-claims | 11 `assessment-stale` |
| workplan-prose-assessment | 4 `assessment-stale` |
| workplan-complete-gate | 15 `assessment-stale`, 1 `assessment-missing` (constraints), 4 `evidence-stale` (`gate-kills-a-test-past-its-timeout`) |
| okf-profile | 2 `assessment-stale` |
| workplan-cli | 6 `assessment-stale` |
| workplan-entity-history | 6 `assessment-stale` |
| workplan-two-way-coverage | 4 `assessment-stale` |
| workplan-generated-docs | 10 `assessment-stale`, 1 `assessment-missing` (constraints) |
| source-index | 3 `assessment-stale` |

No `unresolved-proposal`, `item-without-outcome`, `section-untouched`, `binding-missing` or `tense-mismatch` line in any of the ten. `grep 'PROPOSED change:<slug>]'` over every tracked Markdown file finds none of the ten slugs (the `incremental-parsing` blocks of document-model are examples of the syntax, inside fences). Item outcomes: all 22 items `implemented` (document-model 3, evidence-claims 5, prose-assessment 2, complete-gate 3, okf-profile 2, cli 2, entity-history 1, two-way-coverage 2, generated-docs 2, source-index 2); two reworded to what is built (cli item 2, generated-docs item 2); nothing unbuilt, so no new record and no `moved:`; none `dropped`. `affects` changes: document-model drops `#usage` (not edited); complete-gate keeps `#constraints`, now edited.

### Commits of C1

| Commit | Subject |
| --- | --- |
| `2a8a0b9` | `docs(workplan-document-model): state the hardening fixes as the current contract` |
| `d0585ea` | `docs(workplan-evidence-claims): state the hardening fixes as the current contract` |
| `fdae8ec` | `docs(workplan-prose-assessment): state the hardening fixes as the current contract` |
| `6f086fc` | `docs(workplan-complete-gate): state the hardening fixes as the current contract` |
| `84e9f09` | `docs(okf-profile): state the hardening fixes as the current contract` |
| `f96ff1c` | `docs(workplan-cli): state the hardening fixes as the current contract` |
| `7355d1a` | `docs(workplan-entity-history): state the hardening fixes as the current contract` |
| `824f80e` | `docs(workplan-two-way-coverage): state the hardening fixes as the current contract` |
| `1da446b` | `docs(workplan-generated-docs): state the hardening fixes as the current contract` |
| `6690803` | `docs(source-index): state the hardening fixes as the current contract` |

Every commit through the pre-commit hook (exit 0), no bypass, no stash; each followed by roadmap exit 0 (15 passed), `docs_contract`+`repo_gates` exit 0 (28, 57) and `check` exit 0, then pushed (`a378f46..6690803`). Not done here by scope (C2): assessments, the evidence record `gate-kills-a-test-past-its-timeout`, `verify`, `delivered` states and `change complete`.

## Stage 2 remediation C2-A

Start: HEAD `6690803`, clean tree. Scope: the stale or missing assessments of workplan-document-model, okf-profile, source-index and workplan-generated-docs. `change reconcile <slug>` named seven sections (document-model contract, verification; okf-profile contract, verification; source-index verification; generated-docs constraints `assessment-missing`, verification). Each was read in full through `assess cap:<slug>#<section> --context` and checked against the bound code and tests (`files::target`, `record::create`, `record::replace`, `sync::judge`, `Workplan::write`, `body/marker.rs` `family_of`/`raf_comment`/`split_raf_comment`/`raf_marker`, `exports.rs` `binding_site`, `tests/roadmap.rs` `built_capabilities_hold`), with the diffs of the documents and of `crates/raf-source/src`, `record.rs` and `digest.rs` since the last records (`f88d1dd`); the `raf-source` and `digest.rs` changes since then are doc comments and a refactor of `module_files` with the same result. Bound fixtures run: `raf-okf --test claims` (13), `cargo-workplan --test check split` (1), `raf-source --test exported_items two_globs` (1), `raf-workplan --test records link` (3), `--test roadmap` (15), `--test sync` (19), all passing. Each assessment recorded with `--verdict accepted --by agent --copyright '2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>' --licence CC-BY-4.0` on the final text of its section. No git command that changes state was run; no code, binding, change record, state or frontmatter edited.

| Section | Verdict | Text fixes |
| --- | --- | --- |
| `cap:okf-profile#contract` | accepted | none |
| `cap:okf-profile#verification` | accepted | 1 |
| `cap:source-index#verification` | accepted | 2, 3 |
| `cap:workplan-document-model#contract` | accepted | 4 |
| `cap:workplan-document-model#verification` | accepted | 5 |
| `cap:workplan-generated-docs#constraints` | accepted | 6, 7 |
| `cap:workplan-generated-docs#verification` | accepted | 8 |

No section is rejected.

1. okf-profile Verification: the blank line after the split-marker bullet, which left the list loose before "Cost fixture", removed (no wording change).
2. source-index Verification, the two-globs bullet: "is written as the glob that comes first, with its line, in either order of the two, beside one glob alone, written as that glob." -> "is written as the glob that comes first in the file, with its line, whichever of the two that is; beside it, the re-export that one glob alone brings in is written as that glob." Why: "first" named no order (`binding_site` takes the smallest `uses` index of the carrying routes, the source order of the module's file, as `a_re_export_two_globs_carry_is_written_as_the_first_glob_that_carries_it` asserts for both orders), and the trailing "beside one glob alone, written as that glob" had no subject. The blank line after the bullet removed.
3. source-index Verification: the blank line between the line-and-span bullet and "Exported-item fixtures" removed (no wording change).
4. document-model Contract, **Writing records.**: "`work` and the directory of the record type must each be a directory, and a symbolic link there, ..." -> "`work` and the directory of the record type must each be a directory (`create` makes one that is missing), and a symbolic link there, ...". Why: `files::target` ends the walk at a missing directory and `files::create_new` then creates it, so "must each be a directory" was false for `create` as stated beside "creating the directory of its type when it is missing"; for `replace` a missing directory is `RecordError::Missing`.
5. document-model Verification: the blank line before "Record fixtures in their directories" removed (no wording change).
6. generated-docs Constraints, the `Workplan::write` bullet: "a path below a linked directory, inside the root or out of it," -> "a path below a directory link, whether the link leads inside the root or out of it,". Why: "inside the root or out of it" read as the place of the path, which is always below the root; it is where the link leads (`files::target` refuses an `EntryKind::Symlink` directory whatever it points at).
7. generated-docs Constraints: "Sync is idempotent: running it twice changes nothing the second time." -> "Sync is idempotent: once the report of `Workplan::sync` is written, a second `Workplan::sync` reports nothing, so running it twice changes nothing the second time." Why: the bullet describes `Workplan::sync` and named it nowhere, so with `Workplan::write` and `preview` named the assessment of the section had only those two as inputs and an edit of `Workplan::sync` would not move it (the prose-assessment rule: a section that describes a method names it); the statement is `write_puts_every_edit_on_disk_and_a_second_run_writes_nothing` (`second.is_empty()`). Inputs now `Workplan::preview`, `Workplan::sync`, `Workplan::write`.
8. generated-docs Verification, the directory-link bullet: "is refused by `Workplan::write` before anything is written, though the files behind it are regular files, and the same report through the real directories is written." -> "is refused when the report is written, with nothing written, though the files behind it are regular files; the same report through the real directories is written." Why: that code span was the only strong name of the section, so it narrowed the inputs of a Verification that describes the fixtures of every bound symbol (sync, preview, the lookup and the errors) to `Workplan::write` alone; reconcile reported eight bound symbols as "no longer an input". Without it, all nine `code` bindings are inputs again, as before the bullet was added. Same meaning (`write_refuses_a_target_behind_a_directory_link_inside_the_root`: "the directory <link> is refused", snapshot unchanged, neighbour writes).

Checked and true as written: okf-profile Contract, the split-marker sentences (`split_raf_comment` reads the first non-blank line below a line that ends with `<!--` outside code spans and stops at a fenced line; `family_of` exempts `raf:generated:` in any case; the claim family is `MalformedClaim` because `parse_claim` fails on the `<!--` line, another word `UnknownMarker`; `error_at` the `<!--` line; the codes `okf-parse-malformed-claim`/`okf-parse-unknown-marker` of `finding.rs`), the rest unchanged since its 11D1 record; okf-profile Verification, the split-marker bullet against both fixtures (five error cases with lines, nine plain neighbours, the CLI's three exit-1 and three exit-0 cases with the snapshot); document-model Contract, the rest of **Writing records.** (`create` exclusive with `create_new`, a final link refused by `O_EXCL` or by `replace`'s `EntryKind::File` test, every record write through `files::target`: `record::create`, `record::replace`, `evidence::target_fits`, `record_assessment`); document-model Verification, the Roadmap bullet against `a_built_capability_is_delivered_or_reopened_under_its_own_change_and_has_no_gap` (no record, another name only, a gap, `designed`, absent refused; `delivered` without a record and reopened under its own accepted) and the writer-link bullet against both `records` link fixtures; generated-docs Constraints, "Every target is judged before the first byte is written" (`Workplan::write` judges every edit in its first loop before any `stage`), the `preview` refusals (`PreviewError::Prefix`); generated-docs Verification, the lock bullet (`static EXAMPLE` and `serial()` held at `tests/sync.rs` lines 398, 460, 697, 719, the four tests that call `docs_sync`; `SPAWN` of `root.rs`, `git.rs`, `gate.rs`, `exec.rs`).

Assessments recorded: 7 (6 replaced, 1 new: `work/assessments/workplan-generated-docs-constraints.md`), all `accepted` by `agent`, none re-recorded (every fix was made before the record of its section).

| Gate | Exit | Result |
| --- | --- | --- |
| `change reconcile workplan-document-model` | 0 | no output |
| `change reconcile okf-profile` | 0 | no output |
| `change reconcile source-index` | 0 | no output |
| `change reconcile workplan-generated-docs` | 0 | no output |
| `cargo run -q -p cargo-workplan -- --root <worktree> check` | 0 | no finding |
| `make docs-sync` | 0 | `git diff` and `git status --porcelain` identical before and after |
| `cargo test --test docs_contract --test repo_gates` | 0 | 28, 57 |

## Stage 2 remediation C2-C

Start: HEAD `6690803`; the working tree held only the files of C2-A and C2-B, none of the three capabilities below. Scope: assess `cap:workplan-prose-assessment`, `cap:workplan-entity-history` and `cap:workplan-two-way-coverage`, and probe `cap:workplan-current-rationale` and `cap:workplan-reconcile`. No git command that changes the state of the worktree was run, no state was changed and nothing was completed. Every assessment was recorded with `assess cap:<name>#<section> --verdict accepted --by agent --copyright '2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>' --licence CC-BY-4.0`, on the final text of its section.

### Probe before

`change reconcile <slug>` (each exit 1, `assessment-stale` lines only): prose-assessment constraints 3 (section, `Workplan::record_assessment`, `assessment`) and verification 1 (section); entity-history contract, constraints and verification, 2 each (section, `raf_workplan::history`); two-way-coverage contract and verification, 2 each (section, `raf_workplan::coverage`). `assess --list` also showed as stale, because of the same moved symbols, the sections outside `affects`: prose-assessment purpose, contract, usage and examples; entity-history purpose, usage and examples; two-way-coverage purpose, usage, examples and constraints. The completion gate does not ask for them, but each is a stale record of these capabilities, so they were read and recorded again too. The Explanation of each of the three was `current`.

### How the sections were checked

Each section was read in full in `assess --context` and checked against the code that moved: `files::target` and `Workplan::record_assessment` (`assessment.rs`), `digest::module_files` and `tests/digest.rs`, `HistoryError::SymbolLog` and where `Workplan::history` raises it (`history.rs`), `escape.rs` with `history/render.rs`, and `coverage::check`, `Judge::named_paths`, `declaring_modules` and `is_local` (`coverage.rs`), along with the bound fixtures (`assessments::an_assessment_is_never_written_through_a_linked_directory_and_a_real_one_records`, `digest::a_crate_root_stands_for_its_own_file_and_an_edit_below_it_leaves_its_digest`, `history::a_symbol_log_that_git_refuses_names_the_binding_and_the_file_to_commit`, `coverage::a_capability_that_does_not_load_gives_its_own_findings_and_no_cascade_of_gaps`, `coverage::a_re_export_written_through_super_names_the_path_super_reaches`). Spot checks:

- On this repository: `check` exit 0; `coverage code --crate raf_workplan` and `coverage tests --crate cargo-workplan` exit 0 with no gap row; `coverage code --crate raf-okf` lists gaps (outside the scope); `coverage code --crate nope` exit 2 naming the packages; `history cap:workplan-entity-history` (the de-duplicated "document, `raf_workplan::history`" source on `3e9bc88`); `history cap:workplan-current-rationale --since 2026-10-01`; `history cap:nope` and `--since 2026-13-01`, each exit 2.
- On a copy of the tracked files in `/tmp` (outside the worktree): with `work/assessments` a link to a real directory, `assess ... --verdict` exits 2 and leaves the record byte-identical. On the command line package discovery refuses the linked directory before `record_assessment` is called, which is why the library fixture reads the plan before it makes the link. With the two-way-coverage document set to `state: unknown`, `check` reports `okf-bad-state` and no `coverage-*-unbound`. Neighbour: the document loaded without its `raf_workplan::coverage` binding gives 43 `coverage-item-unbound`. Restored, `check` exit 0.
- In a throwaway git repository made from that copy: padding `crates/raf-workplan/src/git/log.rs` with 6000 lines gives exit 2, "`git log -L` could not read the lines of the bound symbol `raf_workplan::git::AuthorDate` in `crates/raf-workplan/src/git/log.rs` from `HEAD`: ... fatal: file ... has only 729 lines; when ... one way to read them is to commit ...", as the Contract states. Git 2.56: `-L729,800` on a 729-line file answers (the end is clamped) and `-L730,800` fails, so `git log -L` fails when the copy in `HEAD` ends before the first line of the span, not whenever it is shorter (fixes 3 and 4).

### Text fixes (text only)

1. prose-assessment Constraints, real directories. Before: "`work` and `work/assessments` must each be a directory, and a symbolic link there, inside the root or out of it, or a file is refused before anything is written, so the record replaced is the record of the section and never another file." After: "`work` and `work/assessments` must each be a directory, or be missing and then be created, and a symbolic link there, inside the root or out of it, a file or anything else that is not a directory is refused before anything is written, so the record replaced or created is the record of the section and never another file." Why: `files::target` ends the walk at a missing directory (`directories: false`) and `record::create` then creates it through `files::create_new`, so a missing directory is not refused. Any other kind that is not a directory is refused ("it is not a directory"), not only a file.
2. prose-assessment Verification, crate-root digest bullet. Before: "... unchanged by an edit in the file of a module below the root and in the file of a module below that, while each edit moves the digest of the module it is in, ..." After: "... unchanged by an edit in the file of a module below the root and by one in the file of a module below that, while both edits move the digest of the module below the root, whose content holds both files, ..." Why: for the edit of `src/tools/deep.rs` the fixture asserts that the digest of `demo::tools` moves, and `demo::tools` is not the module that file is in (`demo::tools::deep`).
3. entity-history Contract, Binding history. Before: "as for a bound file that is not committed or that `HEAD` holds fewer lines of, ... says that `git log -L` could not read the lines from `HEAD` with what `git` said, and names ..." After: "as for a bound file that is not committed or whose copy in `HEAD` ends before the line the symbol starts at in the working copy, ... says that `git log -L` could not read the lines from `HEAD`, quoting what `git` said, and names ...". Why: a `HEAD` copy that is shorter but still reaches the start of the span is answered, because git clamps the end (spot check above). "with what `git` said" also read as if it belonged to "read the lines"; the message quotes git's error after a colon.
4. entity-history Constraints, the `git log -L` bullet. Before: "A bound file that is not committed, or is shorter in `HEAD`, makes `git log -L` fail," After: "A bound file that is not committed, or whose copy in `HEAD` ends before the line the symbol starts at in the working copy, makes `git log -L` fail,". Why: as for fix 3.

No fix was needed in two-way-coverage. Its Contract (the `super::`/`self::`/workspace-crate reading of a named path, the inline declaring module, the `src/api.rs` example, and no unbound kind while a capability document does not load, with unmatched exclusions and unreadable surfaces still reported) and its Verification bullets match `coverage.rs` and the two fixtures statement by statement.

Not section text, so not edited (code documentation is outside this part): the rustdoc of `HistoryError::SymbolLog` and of the module `history.rs` say that `git log -L` fails "for a file that is not committed or is shorter in `HEAD`", and the conditional of the `SymbolLog` message reads "or `HEAD` holds fewer of its lines than the working copy". Both name the same broader condition that fixes 3 and 4 corrected in the documents. The message offers it as a condition of the hint and not as the cause, so it is not false; the rustdoc is imprecise in the same way.

### Verdicts

| Section | Verdict | Fix |
| --- | --- | --- |
| `cap:workplan-prose-assessment#purpose` | accepted | none |
| `cap:workplan-prose-assessment#contract` | accepted | none |
| `cap:workplan-prose-assessment#usage` | accepted | none |
| `cap:workplan-prose-assessment#examples` | accepted | none |
| `cap:workplan-prose-assessment#constraints` | accepted | 1 |
| `cap:workplan-prose-assessment#verification` | accepted | 2 |
| `cap:workplan-entity-history#purpose` | accepted | none |
| `cap:workplan-entity-history#contract` | accepted | 3 |
| `cap:workplan-entity-history#usage` | accepted | none |
| `cap:workplan-entity-history#examples` | accepted | none |
| `cap:workplan-entity-history#constraints` | accepted | 4 |
| `cap:workplan-entity-history#verification` | accepted | none |
| `cap:workplan-two-way-coverage#purpose` | accepted | none |
| `cap:workplan-two-way-coverage#contract` | accepted | none |
| `cap:workplan-two-way-coverage#usage` | accepted | none |
| `cap:workplan-two-way-coverage#examples` | accepted | none |
| `cap:workplan-two-way-coverage#constraints` | accepted | none |
| `cap:workplan-two-way-coverage#verification` | accepted | none |

None was rejected. Assessments recorded: 18 (each `replaced:`, exit 0); no section needed a second record, because every fix was made before the first recording.

### current-rationale and reconcile (delivered, no open record; text not edited)

`assess --list`: all seven `cap:workplan-current-rationale` assessments are `current`. `cap:workplan-reconcile` has three stale assessments: `#contract`, `#constraints` and `#verification`, each "the symbol `raf_workplan::git` has changed since; the symbol `raf_workplan::lifecycle` has changed since". Since their records (`8e5f568`), the commits `16d3648`, `a0ccac0` and `2c422d5` changed `git/tests.rs` (an `elapsed` field in two `OutputTooLarge` fixtures), `lifecycle/complete.rs` (`judge_record`), `lifecycle/completion.rs` (the condition-6 section count), `lifecycle/session.rs` (`LifecycleError::RecordRefused`) and `lifecycle/tests.rs`. The lead decides whether to reopen `cap:workplan-reconcile`.

### Probe after and gates

| Gate | Exit | Result |
| --- | --- | --- |
| `change reconcile workplan-prose-assessment` | 0 | no output |
| `change reconcile workplan-entity-history` | 0 | no output |
| `change reconcile workplan-two-way-coverage` | 0 | no output |
| `cargo run -q -p cargo-workplan -- --root <worktree> check` | 0 | no finding |
| `assess --list` (the five capabilities) | 0 | every record `current` except the three of `cap:workplan-reconcile` above |
| `make docs-sync` | 0 | `git diff` and `git status --porcelain` hashes identical before and after |
| `cargo test --test docs_contract` | 0 | 28 passed |

Files of this part: `docs/capabilities/workplan-prose-assessment.md` and `docs/capabilities/workplan-entity-history.md` (body text only), and the 18 records `work/assessments/workplan-{prose-assessment,entity-history,two-way-coverage}-{purpose,contract,usage,examples,constraints,verification}.md`. No new record id, so `INVENTORY.records` is unchanged. Nothing committed.

## Stage 2 remediation C2-B

Start: HEAD `6690803`, clean tree. Scope: the re-record of `evidence:gate-kills-a-test-past-its-timeout` and the assessments of `workplan-complete-gate`, `workplan-cli` and `workplan-evidence-claims`. No git command that changes state was run. Every section was read in full through `assess cap:<slug>#<section> --context` and checked against the code. Spot checks were run with `target/debug/cargo-workplan` on this tree and on a scratch export (`git archive HEAD` into `/tmp/c2b/scratch`, no `.git`). Every assessment is `--verdict accepted --by agent --copyright '2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>' --licence CC-BY-4.0`, recorded on the final text of its section. **No section is rejected.** No code, binding, frontmatter or change record was edited.

### Step 1: evidence re-record

`cargo run -q -p cargo-workplan -- --root <worktree> evidence record --id gate-kills-a-test-past-its-timeout --for cap:workplan-complete-gate --by agent --update --from-binding raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran --copyright ... --licence CC-BY-4.0`: exit 0 in 21.6 s, `recorded: work/evidence/gate-kills-a-test-past-its-timeout.md (pass)`. The bound test ran through a real `cargo test` and passed. Diff of the record: only the four digests that had moved changed (`raf_workplan::bound`, `raf_workplan::exec`, `raf_workplan::lifecycle::Failure`, `raf_workplan::lifecycle::Lifecycle::complete`). `verdict: pass`, `by: agent` and `for` are unchanged. `evidence claims gate-kills-a-test-past-its-timeout`: exit 0, one claim, `cap:workplan-complete-gate#contract`, the timeout paragraph ("A bound test that runs past its timeout is killed ... with the time it ran and its limit."). After the text fixes below, the marker is at line 425 and still attached to that paragraph. `evidence for cap:workplan-complete-gate`: `pass ... (in its for list; claims at lines 425)`. After the re-record, reconcile shows no `evidence-stale`, `claim-unsupported` or `evidence-changed` line.

### workplan-complete-gate

| Section | Verdict | Text fixes |
| --- | --- | --- |
| `cap:workplan-complete-gate#contract` | accepted | 1 |
| `cap:workplan-complete-gate#examples` | accepted | none |
| `cap:workplan-complete-gate#constraints` | accepted (first record: was `assessment-missing`) | 2 |
| `cap:workplan-complete-gate#verification` | accepted | none |

Fix 1 (Contract, record removal):
- Before: "With the static conditions, before any `cargo` process or write, `complete` judges the way to `work/changes/<slug>.md`: ... Otherwise it is an error (exit 2) that names the refused path, and nothing is run or written."
- After: "With each judgement of the static conditions, the first before any `cargo` process and the second immediately before the writes, `complete` judges the way to `work/changes/<slug>.md`: ... Otherwise it is an error (exit 2) that names the refused path, and nothing is written; refused at the first judgement, nothing has run either."
- Why: `judge_record` is called inside `evaluate` (`lifecycle/complete.rs`), and `complete` calls `evaluate` twice. The second call comes after `run_bound`, so a link that appears while `cargo` runs is refused after processes have run. "Nothing is run" was false for that case. The Constraints bullet on the window after the second judgement already assumes this second judgement.

Fix 2 (Constraints, compile bullet):
- Before: "`complete` runs the bound tests and examples with `cargo`, so a repository whose tests or examples cannot compile cannot complete a change."
- After: "`complete` runs the bound tests and examples with `cargo`, which builds each of them first, so a bound test or example that cannot compile fails its binding and the change that lists its capability cannot complete."
- Why: the old sentence claimed too much. Only the bound targets are built (`cargo test --package=<pkg> <target>`, `cargo run --example=`), and a change with no bound runs starts no `cargo` at all. A list pass that does not build fails the binding (`a_list_pass_that_does_not_build_fails_the_binding`).

Fix 3 (Constraints, empty lists):
- Before: "A capability with empty `bindings.tests` and `bindings.examples` lists satisfies the fifth condition only while it is `designed` or `considered`."
- After: "... lists gives the fifth condition nothing to run, which suffices only while it is `designed` or `considered`: in `implementing` and `delivered` an empty list is a `binding-missing` obligation of condition 1 (the `examples` list only in `implementing`)."
- Why: `obligation/sources.rs` `bindings` raises `Gap::EmptyList` as `binding-missing` for `implementing` and `delivered`, and skips an empty `examples` list when `delivered`. Condition 5 itself runs nothing for empty lists, so the refusal comes from condition 1, not condition 5. The rustdoc of `Lifecycle::complete` says the same.

Checked and true as written:
- The condition-6 paragraph: `trace::assessments` raises one `assessment-missing`, one `assessment-stale` per moved input, or one `assessment-rejected`. `summary()` counts condition 6 with `sections_without_assessment` (a `BTreeSet` of subject and section). `is_repeated_in` prints `Failure::Assessment` once, under its obligation.
- Bounds: `bound/run.rs` `start` keeps `elapsed` for both `Ended::TimedOut` and `Ended::Overflowed` and fails the binding with the error text. `exec/system.rs` uses `process_group(0)` and `kill_process_group` on Unix.
- The output-kill paragraph: the `OutputTooLarge` message "`<command>` wrote more than <limit> bytes to <stream> in <t> s and was killed".
- `RecordRefused` maps to exit 2 (`exit.rs`, `From<E: Error>`).
- `files::target` refuses a link ("it is a symbolic link ...") and a non-directory. A missing directory or a non-file record is "it is not a regular file".
- Examples: the two-moved-inputs headline matches the `summary` format.
- Verification: the three new bullets match `bound::tests::a_kill_at_either_limit_fails_the_binding_with_the_time_it_ran_and_its_limit` (exact reasons, run time, `wall()`), `test_crate::exec::stubs::a_process_that_never_stops_writing_is_killed_at_the_bound` (`0 < elapsed <= ran`, bound), the two three-moved-inputs fixtures, and `a_linked_work_directory_is_refused_before_anything_runs_and_a_real_one_completes`.

Found in code, not edited (text only in this part): the `LifecycleError::RecordRefused` message in `lifecycle/session.rs` always says "nothing was run or written". When the second judgement refuses, the bound tests have already run, so that half of the message is false there. The message needs "nothing was written" (and "nothing was run" only for the first judgement).

### workplan-cli

| Section | Verdict | Text fixes |
| --- | --- | --- |
| `cap:workplan-cli#contract` | accepted | none |
| `cap:workplan-cli#constraints` | accepted | 1 |
| `cap:workplan-cli#verification` | accepted | 1 |
| `cap:workplan-cli#usage` (not in `affects`; stale through the new input `raf_workplan::escape`) | accepted | none |
| `cap:workplan-cli#examples` (not in `affects`; same) | accepted | none |

Fix 4 (Constraints, escaped text):
- Before: "The JSON form carries the text itself, with its control characters escaped as JSON escapes them."
- After: "The JSON form carries the text itself: JSON escapes the control characters U+0000 to U+001F, and every other character, a bidirectional formatting character included, is written as it is, for the reader of the JSON to show safely."
- Why: a probe in scratch put ESC, U+202E, DEL and U+0085 in a claimed statement. `--format json evidence claims` wrote `\u001b`, but U+202E, DEL and U+0085 came out raw (`od -c`: `342 200 256 177 302 205`). The text form escaped all four (`\u{1b}[2J\u{202e}\u{7f}\u{85}`). The old sentence suggested every control and bidi character is escaped in JSON.

Fix 5 (Verification, the escape and unloaded-claims bullets):
- Before: "the JSON form escapes them as JSON does and reads back as the text" and "exit 2 in text and JSON, naming the document on standard error, with nothing on standard output and nothing written".
- After: "the JSON form holds no raw escape, which it writes as JSON escapes a control character, and reads back as the text" and "exit 2 in text and JSON, naming the document on standard error, in text with nothing on standard output, and nothing written". The last lines were rewrapped.
- Why: both fixtures assert only that `\u001b` replaces the raw ESC in JSON (`!contains('\u{1b}') && contains("\\u001b")`), not that the override is escaped. `claims_exit_two_naming_a_capability_that_does_not_load_and_answer_beside_other_records` asserts empty stdout for the text runs only, and for JSON only the exit and `cap:beta` on stderr. The probe also showed empty stdout for JSON, but the fixture does not pin it.

Checked and true as written:
- Contract: the `evidence claims` exit-2 sentence. Probe: `state: bogus` in a capability gives `cap:workplan-cli did not load, so a claim in it is unknown; fix its document first`, exit 2, for an id with claims and for one without, in text and JSON. An evidence record that does not load leaves the answer, exit 0, while `check` names it with `okf-bad-verdict`.
- The 11 non-rule JSON kinds are in `cargo-workplan/src`. The `history-*` and `coverage-*` kinds are in `raf-workplan`.
- `shown_lines` keeps LF and tab, and both renderers call it on the whole record text.
- Usage: the commands and the `status` row format, compared with a real `status` run.
- Examples: the `evidence record` table, reproduced in scratch: `peak=41` gives pass with exit 0, `peak=80 --update` gives `(fail): over the limit: peak` with exit 1, and `1e-3` gives exit 2 with no record.

### workplan-evidence-claims

| Section | Verdict | Text fixes |
| --- | --- | --- |
| `cap:workplan-evidence-claims#contract` | accepted | none |
| `cap:workplan-evidence-claims#constraints` | accepted | none |
| `cap:workplan-evidence-claims#verification` | accepted | none |
| purpose, usage, examples, explanation (not in `affects`; stale through the moved `evidence`, `evidence_claims`, `record_evidence`) | accepted | none |

Checked and true as written:
- Contract: `Workplan::evidence_claims` checks the id first (`BadId`), then returns `UnloadedCapabilities` for unloaded `cap:` ids only, as `evidence_for` returns `Capability { not_loaded }`.
- Constraints:
  - `target_fits` through `files::target` runs before the run (`evidence.rs` line 483).
  - Probe: a linked `work/evidence` in scratch gave exit 2 before the command ran, and the probe's marker file was not created.
  - `process_group(0)` on Unix.
  - The relative-program probe: `./tool.sh` typed in `sub/` ran the root's `tool.sh` (`where: 1`), and the record keeps `command: ./tool.sh`.
- Usage: `status=ok` gives exit 2 with no record, and `n=3` is stored as a value (probed).
- Verification: each new bullet matches its bound test, as listed in Stage 2 remediation B.

### Totals and gates

- Assessments recorded: 16, all `accepted` by agent. 10 sections reported by reconcile (complete-gate 4, of them 1 new; cli 3; evidence-claims 3). 6 stale records of sections outside `affects` (cli usage and examples; evidence-claims purpose, usage, examples, explanation) were read and re-recorded so that no record of these three capabilities is stale. Every text fix was made before its section's assessment, so nothing was recorded twice. `assess --list` for the three capabilities: 16 lines, all `current`.
- Text fixes: 5 (complete-gate Contract 1 and Constraints 2; cli Constraints 1 and Verification 1). No contract was narrowed.
- Evidence: 1 re-record, `pass`.
- `change reconcile workplan-complete-gate|workplan-cli|workplan-evidence-claims`: each printed nothing, exit 0.
- `check`: exit 0.
- `make docs-sync`: exit 0, with `git diff` and `git status --porcelain` identical before and after.
- `cargo test --locked --test docs_contract`: exit 0 (28).
- For the commit (outside these files): `work/assessments/workplan-complete-gate-constraints.md` is a new record id, `assessment:workplan-complete-gate-constraints`. `INVENTORY.records` in `tests/docs_contract.rs` does not list it. `docs_contract` passes now only because the file is untracked, so it must be added to that list when the file is staged. The same applies to `work/assessments/workplan-generated-docs-constraints.md`, which is in another part's scope.
- The scratch export `/tmp/c2b/scratch` was left in place; it is not a worktree.

## Stage 2 remediation C3a

Start: HEAD `6690803` with the uncommitted work of C2-A, C2-B and C2-C. Step 1: `git status --short` showed only body-text changes in 8 capability documents (okf-profile, source-index, workplan-cli, workplan-complete-gate, workplan-document-model, workplan-entity-history, workplan-generated-docs, workplan-prose-assessment; no frontmatter line changed), 47 modified assessment records, the re-recorded `work/evidence/gate-kills-a-test-past-its-timeout.md` and 2 new records (`workplan-complete-gate-constraints`, `workplan-generated-docs-constraints`). Nothing else.

### Code fixes

- `LifecycleError::RecordRefused` gains `ran: bool`; `evaluate`/`judge_record` take it (`false` for the first evaluation, `true` for the second). Message: "`<path>` is refused on the way to the change record at the first judgement, before anything ran: <reason>; nothing was written, so make ..." or "... at the second judgement, after the bound tests and examples ran: ...". Rustdoc of the variant and of `Lifecycle::complete` errors updated; CHANGELOG `RecordRefused` line updated.
- Tests: `complete::a_linked_work_directory_is_refused_before_anything_runs_and_a_real_one_completes` now asserts `ran: false` and the first-judgement text; new `complete::a_record_linked_while_the_tests_ran_is_refused_at_the_second_judgement` (the record replaced by a link to a file in the root after the first `cargo` process: the change still loads, `RecordRefused { path: work/changes/shape.md, ran: true }`, second-judgement text, cargo ran, link and file kept, the capability document unchanged; neighbour with a run that changes nothing completes). A linked `work` after the run cannot reach the second judgement: the second `Gate::open` refuses it in discovery (`LoadErrorKind::DirSymlink`), observed while writing the test. Bound in `workplan-complete-gate` `bindings.tests`; Contract sentence and a Verification bullet added as plain text (no PROPOSED block).
- `history.rs` module doc and `HistoryError::SymbolLog` rustdoc: "is shorter in `HEAD`" -> "whose copy in `HEAD` ends before the line the symbol starts at in the working copy"; the hint of the `SymbolLog` message reworded the same way (its fixture in `tests/history.rs` updated); `git::symbol_log` rustdoc "or has fewer lines there" -> "or whose copy there ends before the first line of `lines`"; CHANGELOG history line aligned.
- `cargo test -p raf-workplan --test complete --test history`: 36 and 23 passed. `make docs-sync` rewrote the complete-gate verification matrix; `check` exit 0.

### INVENTORY

`tests/docs_contract.rs` `INVENTORY.records` gains `assessment:workplan-complete-gate-constraints` and `assessment:workplan-generated-docs-constraints` (sorted); pins `records("assessment:")` 54 -> 56, `INVENTORY.records.len()` 56 -> 58. With both files `git add`ed: `cargo test --test docs_contract --test repo_gates` exit 0 (28, 57).

### Probe before (after the code fixes)

`assess --list`: 56 records, 17 stale: source-index contract, usage, examples, constraints (the symbols `raf_source::ExportOptions`, `ExportedItem`, `Exports`, `LoadOptions`, `Reexport`, `Target` changed; these were already stale at `6690803`, outside `affects`, and not re-read in C2-A); workplan-complete-gate contract, examples, constraints, verification (`Lifecycle::complete`, and the Contract and Verification text); workplan-entity-history all six authored sections except explanation (`raf_workplan::history`, `git::symbol_log`); workplan-reconcile contract, constraints, verification (`raf_workplan::git`, `raf_workplan::lifecycle`). `change reconcile`: exit 1 for workplan-complete-gate (`evidence-stale` for `gate-kills-a-test-past-its-timeout` plus 6 `assessment-stale`) and workplan-entity-history (5 `assessment-stale`); exit 0 with no output for the other 8 open changes.

### A flaky fixture found by the full test run

`cargo test --workspace` failed on 4 of 5 runs: `cargo_workplan::test_crate::evidence::output_the_tool_cannot_read_is_exit_two_and_writes_no_record` compares the two argument shapes byte for byte, and since the output kill gives the time it ran (`... in 0.1 s and was killed` beside `... in 0.0 s ...`) the two runs differ. Fix: `Repo::both_timed` and `Run::without_run_time` in `crates/cargo-workplan/tests/support/mod.rs` write the seconds before ` s and was killed` as `<t>` in both runs before comparing; the fixture uses it and now also asserts the kill line `wrote more than 16777216 bytes to standard output in <t> s and was killed`. Five runs of the fixture pass. The workplan-cli Verification bullet on both argument shapes now says that the seconds of an output-kill message are left out of the comparison.

### Re-records

- `evidence record --id gate-kills-a-test-past-its-timeout --for cap:workplan-complete-gate --by agent --update --from-binding raf_workplan::test_crate::bound::a_test_that_never_ends_is_killed_and_refused_with_the_time_it_ran ...`: exit 0 (41.8 s), `recorded: ... (pass)`; only the digests of `raf_workplan::bound`, `raf_workplan::exec`, `lifecycle::Failure` and `Lifecycle::complete` changed.
- Assessments, each `--verdict accepted --by agent --copyright '2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>' --licence CC-BY-4.0`, exit 0, each read against its `assess --context` first:
  - workplan-complete-gate contract, examples, constraints, verification: the new Contract sentences (the judgement named, nothing written in either case, the tests ran when the second refuses) match `judge_record(.., ran)` and the message; the two Verification bullets match the two fixtures; the Constraints bullet on the window after the second judgement still holds; Examples quote no record refusal.
  - workplan-entity-history purpose, contract, usage, examples, constraints, verification: only rustdoc and the hint of the `SymbolLog` message moved; the Contract, Constraints and Verification sentences (lines 95-98, 238-242, 341-343) state the narrower condition and that committing is one way to read the lines, which the reworded hint and `tests/history.rs` now say too; the other sections say nothing of this limit.
  - source-index contract, usage, examples, constraints: stale since `f88d1dd` through rustdoc-only edits in `raf-source` (`a378f46`: "a later field" -> "adding a field", "a later option" -> "adding an option"); the Contract already says "adding a field is not a breaking change" and "adding an option that is not `Copy`"; nothing else moved.
  - workplan-reconcile contract, constraints, verification (see the decision below).
  - workplan-cli verification, after the shapes bullet was qualified.

### workplan-reconcile decision: re-recorded, not reopened

Since its records (`8e5f568`), `raf_workplan::git` changed by an `elapsed` field in two `OutputTooLarge` fixtures of `git/tests.rs` and by the `symbol_log` rustdoc of this part; `raf_workplan::lifecycle` changed in `complete.rs` (`judge_record`, the `ran` flag), `completion.rs` (`sections_without_assessment` for the condition-6 headline, and its unit test), `session.rs` (`RecordRefused`) and `lifecycle/tests.rs` (`elapsed` in one fixture). Each statement of the three sections was read in `assess --context`: none describes the completion headline, the record refusal or the kill message; the statements that touch these modules ("A command that writes more than its bound or runs past 300 seconds is killed and is an error"; the 1 MiB `cat-file blob` bound and its fixture; "reconcile and impact never write"; the per-kind fixture of the completion gate refusing each trace and assessment kind and completing once repaired) still hold. Every statement holds, so the three were re-recorded as accepted; `cap:workplan-reconcile` stays `delivered`.

### Probe after

Pass 2 (after the re-records) and pass 3 (after the flaky-fixture fix and the cli re-record): `assess --list` 56 records, all `current`; `change reconcile <slug>` exit 0 with no output for each of the 10 open changes; in a scratch worktree (`git worktree add --detach /tmp/probe HEAD`, the working-tree diff applied with `git apply`, nothing committed, no hook touched, removed with `git worktree remove --force /tmp/probe`) a throwaway `change new stale-probe` listing all seven sections, then `change reconcile stale-probe`: for workplan-reconcile only `section-untouched` (4), `assessment-missing` (4, the never-assessed purpose, usage, examples, explanation) and one `generated-stale` (the relationships block naming the throwaway record); for workplan-current-rationale one `section-untouched`. No `assessment-stale`, `assessment-rejected`, `evidence-stale`, `evidence-changed` or `claim-unsupported` for any of the 12. (Pass 1, after the code fixes and before the re-records, gave for workplan-reconcile 6 `assessment-stale` lines over #contract, #constraints, #verification, and for current-rationale nothing stale.)

### A second flaky fixture

The first full-gate run after the probe passed (111 binaries, 1948 passed); the second failed `raf_workplan::test_crate::exec::stubs::a_descendant_that_holds_the_pipes_after_the_exit_is_not_waited_for_past_the_limit` with `Spawn { .. kind: ExecutableFileBusy, "Text file busy (os error 26)" }`. Cause: `tests/exec.rs` holds its `SPAWN` lock in every test that writes a stub, but four tests outside the `stubs` module start real processes without it (`the_real_git_and_cargo_start`, `a_program_that_does_not_exist_is_a_spawn_error`, `a_directory_that_does_not_exist_is_a_spawn_error`, `a_name_missing_from_the_search_path_is_a_spawn_error`), so their fork can hold a stub's write descriptor while the stub test executes it. Fix: `SPAWN` and `serial()` moved to the top of the file, and the four tests hold the lock. `cargo test -p raf-workplan --test exec` 15 runs in a row: 43 passed each. The generated-docs lock bullet ("as the other process tests of the crate hold theirs") still holds; no document names the lock otherwise.

### Gates (final tree, before the commits)

| Gate | Exit | Result |
| --- | --- | --- |
| `cargo-workplan check` | 0 | no finding |
| `make docs-sync` | 0 | `git diff` + `git status --porcelain` hash identical before and after |
| `cargo fmt --all -- --check` | 0 | |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | no output |
| `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked` | 0 | |
| `cargo test --workspace --locked`, 3 runs after both fixture fixes | 0, 0, 0 | 111 binaries, 1948 passed, 0 failed each |
| `make reuse` | 0 | compliant |
| `make issue-refs` | 0 | 57 passed |
| `assess --list` | 0 | 56 records, all `current` |
| `cargo test --test docs_contract --test repo_gates` | 0 | 28, 57 (and in each pre-commit hook) |

### Commits (hooks on, explicit paths, no bypass)

The pre-commit gate judges the staged index with test binaries built from the working tree, whose `INVENTORY` lists the two new assessment records; a code-first commit without them fails `check_inventory`. So the docs commit came first, with the complete-gate and cli documents staged as `HEAD` plus the C2-B text fixes only (blobs written with `git hash-object -w` and staged with `git update-index --cacheinfo`); the sentences, binding and matrix row of this part went with the code and fixture commits.

1. `003b50c` docs: assess the sections the hardening fixes changed (8 capability documents' C2 text fixes, 56 assessment records, the evidence record, `tests/docs_contract.rs` INVENTORY): hook exit 0 (fmt, reuse, docs_contract 28, repo_gates 57).
2. `f137b7a` fix(raf-workplan): say which judgement refused a record write and describe the history limit precisely (`session.rs`, `complete.rs`, `history.rs`, `git/log.rs`, `tests/complete.rs`, `tests/history.rs`, CHANGELOG, complete-gate Contract sentence, Verification bullets, binding and matrix row): hook exit 0.
3. `6679168` test: compare the shapes without the run time of a kill and lock every process test (`cargo-workplan` `tests/evidence.rs` and `tests/support/mod.rs`, `raf-workplan` `tests/exec.rs`, the cli Verification bullet): hook exit 0.

After the last commit: `cargo test -p raf-workplan --test roadmap` 15 passed; `check` exit 0; `assess --list` all current. Pushed: `6690803..6679168`, remote head equals `git rev-parse HEAD` (`6679168`). No capability reopened.

## Stage 2 remediation C3b

Start: HEAD `6679168`, clean tree, hooks on (`core.hooksPath=.githooks`). Scope: complete the ten reopened change records again with the shipped tool (`change complete`), one at a time, each committed and pushed before the next, in the order okf-profile, source-index, workplan-generated-docs, workplan-document-model, workplan-complete-gate, workplan-evidence-claims, workplan-prose-assessment, workplan-two-way-coverage, workplan-entity-history, workplan-cli; then the final state, the gates and one `verify` over the whole repository. Every record lists only its own capability in `changes`; item outcomes at the start: all 24 `implemented` (okf-profile 2, source-index 2, generated-docs 2, document-model 3, complete-gate 3, evidence-claims 5, prose-assessment 2, two-way-coverage 2, entity-history 1, cli 2). No stash, no hook bypass.

### okf-profile (commit `92521b0`)

1. `docs/capabilities/okf-profile.md` frontmatter `state: implementing` -> `state: delivered` (the only change). `INVENTORY`: `("okf-profile", "delivered")`; shape pins `implementing` 9, `delivered` 3. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (14 s).
2. `change reconcile okf-profile`: no output, exit 0 (19 s). Items 2/2 `implemented`.
3. `change complete okf-profile` (background, output to a file): exit 0, 55 s, `completed: removed work/changes/okf-profile.md`. No `rewrote` line (no relationships block). No refusal.
4. After `git add` of the four paths (deletion staged): record absent; `check` exit 0; `make docs-sync` exit 0, nothing rewritten, status and diff hash identical; roadmap exit 0 (15); `docs_contract` + `repo_gates` exit 0 (28, 57); `make reuse` exit 0; `make issue-refs` exit 0 (57); `assess --list` exit 0, 56 lines, 56 current.
5. Commit `92521b0` `docs(capabilities): deliver the okf-profile capability again` (4 files, explicit paths); pre-commit hook exit 0 (reuse, 28, 57). Pushed `6679168..92521b0`.

### source-index (commit `11c7d0c`)

1. `state: implementing` -> `state: delivered` (the only change). `INVENTORY`: `("source-index", "delivered")`; pins `implementing` 8, `delivered` 4. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (9 s).
2. `change reconcile source-index`: no output, exit 0 (12 s). Items 2/2 `implemented`.
3. `change complete source-index` (background): exit 0, 249 s, `completed: removed work/changes/source-index.md`. No `rewrote` line. No refusal.
4. After `git add` of the four paths: record absent; `check` exit 0; `make docs-sync` nothing rewritten (status and diff hash identical); roadmap exit 0 (15); `docs_contract` + `repo_gates` exit 0 (28, 57); `make reuse` exit 0; `make issue-refs` exit 0 (57); `assess --list` exit 0, 56/56 current.
5. Commit `11c7d0c` `docs(capabilities): deliver the source-index capability again` (4 files); pre-commit hook exit 0 (reuse, 28, 57). Pushed `92521b0..11c7d0c`.

### workplan-generated-docs (commit `62d6c9b`)

1. `state: implementing` -> `state: delivered` (the only change by hand). `INVENTORY`: `("workplan-generated-docs", "delivered")`; pins `implementing` 7, `delivered` 5. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (6 s).
2. `change reconcile workplan-generated-docs`: no output, exit 0 (20 s). Items 2/2 `implemented`.
3. `change complete workplan-generated-docs` (background): exit 0, 77 s:

   ```text
   completed: removed work/changes/workplan-generated-docs.md
   rewrote docs/capabilities/workplan-generated-docs.md
   ```

   The rewrite removed the one row `| Changed by | [workplan-generated-docs](../../work/changes/workplan-generated-docs.md) |`. No refusal.
4. After `git add` of the four paths: record absent; `check` exit 0; `make docs-sync` nothing rewritten; roadmap exit 0 (15); `docs_contract` + `repo_gates` exit 0 (28, 57); `make reuse` exit 0; `make issue-refs` exit 0 (57); `assess --list` exit 0, 56/56 current (the rewritten block is generated, blanked in the digests).
5. Commit `62d6c9b` `docs(capabilities): deliver the workplan-generated-docs capability again` (4 files, the rewrite included); pre-commit hook exit 0 (reuse, 28, 57). Pushed `11c7d0c..62d6c9b`.

### workplan-document-model (commit `113357b`)

1. `state: implementing` -> `state: delivered` (the only change by hand). `INVENTORY`: `("workplan-document-model", "delivered")`; pins `implementing` 6, `delivered` 6. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (7 s).
2. `change reconcile workplan-document-model`: no output, exit 0 (18 s). Items 3/3 `implemented`.
3. `change complete workplan-document-model` (background): exit 0, 113 s:

   ```text
   completed: removed work/changes/workplan-document-model.md
   rewrote docs/capabilities/workplan-document-model.md
   ```

   The rewrite removed the one row `| Changed by | [workplan-document-model](../../work/changes/workplan-document-model.md) |`. No refusal.
4. After `git add` of the four paths: record absent; `check` exit 0; `make docs-sync` nothing rewritten; roadmap exit 0 (15); `docs_contract` + `repo_gates` exit 0 (28, 57); `make reuse` exit 0; `make issue-refs` exit 0 (57); `assess --list` exit 0, 56/56 current.
5. Commit `113357b` `docs(capabilities): deliver the workplan-document-model capability again` (4 files, the rewrite included); pre-commit hook exit 0 (reuse, 28, 57). Pushed `62d6c9b..113357b`.

### workplan-complete-gate (commit `f5a88ff`)

1. `state: implementing` -> `state: delivered` (the only change by hand). `INVENTORY`: `("workplan-complete-gate", "delivered")`; pins `implementing` 5, `delivered` 7. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (5 s).
2. `change reconcile workplan-complete-gate`: no output, exit 0 (13 s). Items 3/3 `implemented`.
3. `change complete workplan-complete-gate` (background): exit 0, 195 s:

   ```text
   completed: removed work/changes/workplan-complete-gate.md
   rewrote docs/capabilities/workplan-complete-gate.md
   ```

   The rewrite removed the one row `| Changed by | [workplan-complete-gate](../../work/changes/workplan-complete-gate.md) |`. No refusal.
4. After `git add` of the four paths: record absent; `check` exit 0; `make docs-sync` nothing rewritten; roadmap exit 0 (15); `docs_contract` + `repo_gates` exit 0 (28, 57); `make reuse` exit 0; `make issue-refs` exit 0 (57); `assess --list` exit 0, 56/56 current; `evidence for cap:workplan-complete-gate` exit 0: `evidence:gate-kills-a-test-past-its-timeout  pass ... (in its for list; claims at lines 427)`.
5. Commit `f5a88ff` `docs(capabilities): deliver the workplan-complete-gate capability again` (4 files, the rewrite included); pre-commit hook exit 0 (reuse, 28, 57). Pushed `113357b..f5a88ff`.

### workplan-evidence-claims (commit `fd98b01`)

1. `state: implementing` -> `state: delivered` (the only change by hand). `INVENTORY`: `("workplan-evidence-claims", "delivered")`; pins `implementing` 4, `delivered` 8. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (12 s).
2. `change reconcile workplan-evidence-claims`: no output, exit 0 (20 s). Items 5/5 `implemented`.
3. `change complete workplan-evidence-claims` (background): exit 0, 71 s, `completed: removed work/changes/workplan-evidence-claims.md`. No `rewrote` line (no relationships block). No refusal.
4. After `git add` of the four paths: record absent; `check` exit 0; `make docs-sync` nothing rewritten; roadmap exit 0 (15); `docs_contract` + `repo_gates` exit 0 (28, 57); `make reuse` exit 0; `make issue-refs` exit 0 (57); `assess --list` exit 0, 56/56 current.
5. Commit `fd98b01` `docs(capabilities): deliver the workplan-evidence-claims capability again` (4 files); pre-commit hook exit 0 (reuse, 28, 57). Pushed `f5a88ff..fd98b01`.

### workplan-prose-assessment (commit `734a3aa`)

1. `state: implementing` -> `state: delivered` (the only change by hand). `INVENTORY`: `("workplan-prose-assessment", "delivered")`; pins `implementing` 3, `delivered` 9. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (13 s).
2. `change reconcile workplan-prose-assessment`: no output, exit 0 (36 s). Items 2/2 `implemented`.
3. `change complete workplan-prose-assessment` (background): exit 0, 62 s, `completed: removed work/changes/workplan-prose-assessment.md`. No `rewrote` line. No refusal.
4. After `git add` of the four paths: record absent; `check` exit 0; `make docs-sync` nothing rewritten; roadmap exit 0 (15); `docs_contract` + `repo_gates` exit 0 (28, 57); `make reuse` exit 0; `make issue-refs` exit 0 (57); `assess --list` exit 0, 56/56 current.
5. Commit `734a3aa` `docs(capabilities): deliver the workplan-prose-assessment capability again` (4 files); pre-commit hook exit 0 (reuse, 28, 57). Pushed `fd98b01..734a3aa`.

### workplan-two-way-coverage (commit `91804c0`)

1. `state: implementing` -> `state: delivered` (the only change by hand). `INVENTORY`: `("workplan-two-way-coverage", "delivered")`; pins `implementing` 2, `delivered` 10. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (7 s).
2. `change reconcile workplan-two-way-coverage`: no output, exit 0 (13 s). Items 2/2 `implemented`.
3. `change complete workplan-two-way-coverage` (background): exit 0, 41 s, `completed: removed work/changes/workplan-two-way-coverage.md`. No `rewrote` line. No refusal.
4. After `git add` of the four paths: record absent; `check` exit 0; `make docs-sync` nothing rewritten; roadmap exit 0 (15); `docs_contract` + `repo_gates` exit 0 (28, 57); `make reuse` exit 0; `make issue-refs` exit 0 (57); `assess --list` exit 0, 56/56 current.
5. Commit `91804c0` `docs(capabilities): deliver the workplan-two-way-coverage capability again` (4 files); pre-commit hook exit 0 (reuse, 28, 57). Pushed `734a3aa..91804c0`.

### workplan-entity-history (commit `f8a4e7d`)

1. `state: implementing` -> `state: delivered` (the only change by hand). `INVENTORY`: `("workplan-entity-history", "delivered")`; pins `implementing` 1, `delivered` 11. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (12 s).
2. `change reconcile workplan-entity-history`: no output, exit 0 (17 s). Items 1/1 `implemented`.
3. `change complete workplan-entity-history` (background): exit 0, 42 s, `completed: removed work/changes/workplan-entity-history.md`. No `rewrote` line. No refusal.
4. After `git add` of the four paths: record absent; `check` exit 0; `make docs-sync` nothing rewritten; roadmap exit 0 (15); `docs_contract` + `repo_gates` exit 0 (28, 57); `make reuse` exit 0; `make issue-refs` exit 0 (57); `assess --list` exit 0, 56/56 current.
5. Commit `f8a4e7d` `docs(capabilities): deliver the workplan-entity-history capability again` (4 files); pre-commit hook exit 0 (reuse, 28, 57). Pushed `91804c0..f8a4e7d`.

### workplan-cli (commit `4dcdebc`)

1. `state: implementing` -> `state: delivered` (the only change by hand). `INVENTORY`: `("workplan-cli", "delivered")`; pins `implementing` 0, `delivered` 12. `make docs-sync`: exit 0, `rewrote docs/capabilities/index.md` (10 s; the row moves to `## Delivered` and the now empty `## Implementing` group is dropped).
2. `change reconcile workplan-cli`: no output, exit 0 (27 s). Items 2/2 `implemented`.
3. `change complete workplan-cli` (background): exit 0, 202 s:

   ```text
   completed: removed work/changes/workplan-cli.md
   rewrote docs/capabilities/workplan-cli.md
   ```

   The rewrite removed the one row `| Changed by | [workplan-cli](../../work/changes/workplan-cli.md) |`. No refusal.
4. After `git add` of the four paths: record absent; `check` exit 0; `make docs-sync` nothing rewritten; roadmap exit 0 (15); `docs_contract` + `repo_gates` exit 0 (28, 57); `make reuse` exit 0; `make issue-refs` exit 0 (57); `assess --list` exit 0, 56/56 current.
5. Commit `4dcdebc` `docs(capabilities): deliver the workplan-cli capability again` (4 files, the rewrite included); pre-commit hook exit 0 (reuse, 28, 57). Pushed `f8a4e7d..4dcdebc`.

### Summary of the ten completions

| Record | reconcile | complete | Duration | Rewrites | Commit |
| --- | --- | --- | --- | --- | --- |
| `okf-profile` | empty, exit 0 (19 s) | exit 0 | 55 s | none | `92521b0` |
| `source-index` | empty, exit 0 (12 s) | exit 0 | 249 s | none | `11c7d0c` |
| `workplan-generated-docs` | empty, exit 0 (20 s) | exit 0 | 77 s | `docs/capabilities/workplan-generated-docs.md` | `62d6c9b` |
| `workplan-document-model` | empty, exit 0 (18 s) | exit 0 | 113 s | `docs/capabilities/workplan-document-model.md` | `113357b` |
| `workplan-complete-gate` | empty, exit 0 (13 s) | exit 0 | 195 s | `docs/capabilities/workplan-complete-gate.md` | `f5a88ff` |
| `workplan-evidence-claims` | empty, exit 0 (20 s) | exit 0 | 71 s | none | `fd98b01` |
| `workplan-prose-assessment` | empty, exit 0 (36 s) | exit 0 | 62 s | none | `734a3aa` |
| `workplan-two-way-coverage` | empty, exit 0 (13 s) | exit 0 | 41 s | none | `91804c0` |
| `workplan-entity-history` | empty, exit 0 (17 s) | exit 0 | 42 s | none | `f8a4e7d` |
| `workplan-cli` | empty, exit 0 (27 s) | exit 0 | 202 s | `docs/capabilities/workplan-cli.md` | `4dcdebc` |

No `complete` refused; no reconcile printed anything, so no text, assessment or item outcome was changed. No item was moved or dropped.

### Final state (HEAD `4dcdebc`, clean tree)

- `INVENTORY`: 12 `delivered`, 0 `implementing`, 9 `designed`; pins `count("implementing") == 0`, `count("delivered") == 12`.
- `work/changes/`: the nine `census-*` records only.
- `status`: exit 0; `delivered 12` (okf-profile, source-index, the ten `workplan-*`), `implementing 0`, `designed 9` (each census capability with its own open change), `considered 0`, `open changes 9`.
- `assess --list`: exit 0, 56 records, 56 current.
- Stale probe (scratch worktree `git worktree add --detach /tmp/c3b-stale HEAD`, binary `target/debug/cargo-workplan --root /tmp/c3b-stale`, script kept outside the scratch tree in `/tmp/c3b-probe-scripts/probe.sh`): for each of the 12 delivered capabilities, starting from `reset --hard HEAD` of the scratch tree, `change new stale-probe-<cap> --capability cap:<cap> --affects cap:<cap>#<s>` for all seven sections (exit 0 each), `git add` of the record, `change reconcile stale-probe-<cap>` (exit 1 for 11, exit 0 with no output for workplan-evidence-claims, whose seven sections all have current assessments). 62 lines in total: `section-untouched` 29, `assessment-missing` 28 (sections never assessed), `generated-stale` 5 (the relationships blocks of cli, complete-gate, document-model, generated-docs and reconcile naming the probe record). **0** `assessment-stale`, `assessment-rejected`, `evidence-stale`, `evidence-changed` or `claim-unsupported` lines in every capability. 7 min 11 s. Scratch worktree removed with `git worktree remove --force /tmp/c3b-stale` and `git worktree prune`.

### Gates (HEAD `4dcdebc`, clean tree; each target of `make check` run by itself, in order)

| Target | Exit | Time | Result |
| --- | --- | --- | --- |
| `make fmt` | 0 | 3 s | |
| `make clippy` | 0 | 1 s | no warning (cached build of the same sources) |
| `make test` | 0 | 306 s | 111 `test result: ok`, 1948 passed, 0 failed |
| `make test-release` | 0 | 207 s | 33 `test result: ok`, 524 passed, 0 failed |
| `make doc` | 0 | 2 s | |
| `make deny` | 0 | 1 s | advisories, bans, licenses, sources ok |
| `make reuse` | 0 | 2 s | compliant |
| `make msrv` | 0 | 26 s | |
| `make fuzz` | 0 | 206 s | fmt, clippy, deny and the three targets over their corpus |
| `make licenses` | 0 | 2 s | |
| `make issue-refs` | 0 | 1 s | |
| `make docs-sync` (twice) | 0, 0 | | nothing rewritten; status and diff hash identical before and after each |

`make cross-check` was not run, by owner decision: the hosted CI jobs count for it.

### `verify` over the whole repository

`cargo run -q -p cargo-workplan -- --root <worktree> verify` (no slug, so every capability; background, output to a file, no polling) at HEAD `4dcdebc`: exit 0, 756 s. 1481 lines, **1481 `pass`, 0 `fail`**: 1454 test bindings and 27 example bindings over the 12 delivered capabilities. The nine `designed` census capabilities have no bound tests or examples.

| Capability | Tests | Examples |
| --- | --- | --- |
| okf-profile | 90 | 2 |
| source-index | 293 | 1 |
| workplan-cli | 202 | 5 |
| workplan-complete-gate | 203 | 2 |
| workplan-current-rationale | 16 | 1 |
| workplan-document-model | 253 | 3 |
| workplan-entity-history | 34 | 1 |
| workplan-evidence-claims | 36 | 2 |
| workplan-generated-docs | 85 | 3 |
| workplan-prose-assessment | 28 | 2 |
| workplan-reconcile | 190 | 4 |
| workplan-two-way-coverage | 24 | 1 |

No binding failed, so nothing was fixed. This closes the gap that `verify` had never been run over the whole repository. The tree was clean after the run.

### Commits of C3b

| Commit | Subject |
| --- | --- |
| `92521b0` | `docs(capabilities): deliver the okf-profile capability again` |
| `11c7d0c` | `docs(capabilities): deliver the source-index capability again` |
| `62d6c9b` | `docs(capabilities): deliver the workplan-generated-docs capability again` |
| `113357b` | `docs(capabilities): deliver the workplan-document-model capability again` |
| `f5a88ff` | `docs(capabilities): deliver the workplan-complete-gate capability again` |
| `fd98b01` | `docs(capabilities): deliver the workplan-evidence-claims capability again` |
| `734a3aa` | `docs(capabilities): deliver the workplan-prose-assessment capability again` |
| `91804c0` | `docs(capabilities): deliver the workplan-two-way-coverage capability again` |
| `f8a4e7d` | `docs(capabilities): deliver the workplan-entity-history capability again` |
| `4dcdebc` | `docs(capabilities): deliver the workplan-cli capability again` |

Each commit has 4 files given as explicit paths and passed the pre-commit hook (exit 0: fmt, reuse, `docs_contract` 28, `repo_gates` 57). Each was pushed right after it was made (`6679168..4dcdebc`), and the remote head equals `HEAD` (`4dcdebc`). No hook was bypassed and no stash was used. No commit message carries attribution. `.deficiencies` holds only its notice and marker. No refusal, no text edit, no re-assessment, and no item outcome changed. Hosted CI was not observed here.

## Stage 3 integration assessment

The PR base (main) equals the merge base of the branch, so the integration candidate is the branch tip itself: `git merge-tree --write-tree` exits 0 with the branch tree, and no change of main exists to combine. Hosted CI (12 of 12) on the tip is the qualifying evidence for the candidate; no merge of main into the branch, push or rerun is needed. The final Stage 2 audit passed every criterion except the PR body, whose statements were corrected afterwards; the only code change since is a test-only edit covered by hosted CI. No further audit was run (owner decision).

### Stage 3 delta at the branch tip

The tip is one commit past the audited tree (a test-only edit of `exec/system.rs`: a blocking `waitid` replaces a sleep). That module is an input of the four complete-gate assessments and the timeout-kill evidence record, which were re-recorded in the same commit; at the tip `assess --list` shows 56 of 56 current, `evidence for cap:workplan-complete-gate` passes and `check` exits 0. The whole-repository `verify` log (1486 pass, 0 fail, exit 0) was taken on the tree before that commit.
