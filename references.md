# Referenced artifacts — issue #10

Everything the issue names, checked against disk. Facts only: whether a
file is a viable port source, or may be relicensed, is decided in the plan.
Paths resolve against the main checkout, then against a sibling checkout
only when that repo is named (first path component, or in the prose just
before the reference). "not here; exists in sibling(s)" is NOT resolved.

## Paths

| Reference | Exists | Resolved in | Kind | Lines | Licence header |
|---|---|---|---|---|---|
| `.stage/m2-rest-of-the-planning-essay-entity` | NO | repo | — | — | — |
| `work/changes/` | yes | repo | dir | — | — |
| `work/coverage/default.md` | NO | repo | — | — | — |
| `work/` | yes | repo | dir | — | — |
| `docs/architecture/okf-profile.md` | yes | repo | file | 216 | SPDX: CC-BY-4.0 |
| `body/marker.rs` | NO | repo | — | — | — |
| `tests/docs_contract.rs::check_inventory` | NO | repo | — | — | — |
| `-rf.rs` | NO | repo | — | — | — |
| `docs/` | yes | repo | dir | — | — |
| `validation.md` | NO | repo | — | — | — |
| `tasks/TN-review.md` | NO | repo | — | — | — |
| `work/evidence` | NO | repo | — | — | — |
| `work/assessments` | NO | repo | — | — | — |
| `work/coverage` | NO | repo | — | — | — |
| `tests/docs_contract.rs` | yes | repo | file | 1355 | SPDX: MIT OR Apache-2.0 OR MulanPSL-2.0 |
| `tests/roadmap.rs` | NO | repo | — | — | — |
| `docs/capabilities/workplan-two-way-coverage.md` | yes | repo | file | 117 | SPDX: CC-BY-4.0 |

### History: `work/changes/`

- `92e4de8d3 docs(workplan): deliver the reconcile, completion gate and command line capabilities`
- `41eba8226 docs(workplan): state the built contract of the lifecycle capabilities`
- `a3d60dd0a docs(capabilities): deliver the workplan document model and generated docs`
- `d65e9f4b8 docs(changes): record the outcomes of the workplan model and generated docs`
- `6c744af63 docs(capabilities): deliver the OKF profile and source index`
- `376baf93b docs(changes): record the outcomes of the raf-okf and raf-source items`
- `1a67fa3e5 feat(raf-okf): profile validation with typed documents`
- `80c14d720 docs(census): map fingerprints through the index report at a normal-form bump`
- `edf93d5b5 docs: resolve contradictions in the census and workplan contracts`
- `6120029a9 docs(security): state the trust model and harden the designed parsing and subprocess contracts`

### History: `work/`

- `92e4de8d3 docs(workplan): deliver the reconcile, completion gate and command line capabilities`
- `41eba8226 docs(workplan): state the built contract of the lifecycle capabilities`
- `a3d60dd0a docs(capabilities): deliver the workplan document model and generated docs`
- `d65e9f4b8 docs(changes): record the outcomes of the workplan model and generated docs`
- `6c744af63 docs(capabilities): deliver the OKF profile and source index`
- `376baf93b docs(changes): record the outcomes of the raf-okf and raf-source items`
- `1a67fa3e5 feat(raf-okf): profile validation with typed documents`
- `80c14d720 docs(census): map fingerprints through the index report at a normal-form bump`
- `edf93d5b5 docs: resolve contradictions in the census and workplan contracts`
- `6120029a9 docs(security): state the trust model and harden the designed parsing and subprocess contracts`

### History: `docs/architecture/okf-profile.md`

- `a3d60dd0a docs(capabilities): deliver the workplan document model and generated docs`
- `4e9c2f397 test(docs-contract): read documents through raf-okf and resolve bindings`
- `1a67fa3e5 feat(raf-okf): profile validation with typed documents`
- `e94cd8df8 feat(raf-okf): body model, transactional splicing and capability index`
- `3f8697b35 feat(raf-okf): bounded frontmatter reader and deterministic writer`
- `edf93d5b5 docs: resolve contradictions in the census and workplan contracts`
- `dba307fae test(docs): harden the document contract`
- `6120029a9 docs(security): state the trust model and harden the designed parsing and subprocess contracts`
- `f711aa29e docs(workplan): define the seven sections and close the workplan contract gaps`
- `9ce066f56 docs(capabilities): add workplan capability specs`

### History: `docs/`

- `a72c5f51d docs(capabilities): name the git commands of the default-branch lookup`
- `92e4de8d3 docs(workplan): deliver the reconcile, completion gate and command line capabilities`
- `41eba8226 docs(workplan): state the built contract of the lifecycle capabilities`
- `ed5ec735a docs(workplan): propose the contract edits of the lifecycle capabilities`
- `0e999a1ab docs(workplan): bind the reconcile, completion gate and command line capabilities`
- `68fb6e9ca docs(capabilities): keep the command line out of the delivered workplan contracts`
- `a3d60dd0a docs(capabilities): deliver the workplan document model and generated docs`
- `45ff25260 test(docs-contract): check the repository through raf-workplan`
- `5bcd7411b fix(raf-source): index verbatim use groups and bodiless items`
- `bf08e459e feat(raf-source)!: seal the public types consumers bind to`

### History: `tests/docs_contract.rs`

- `92e4de8d3 docs(workplan): deliver the reconcile, completion gate and command line capabilities`
- `0e999a1ab docs(workplan): bind the reconcile, completion gate and command line capabilities`
- `c0685a4c2 feat(cargo-workplan): check, status and docs commands over raf-workplan`
- `a3d60dd0a docs(capabilities): deliver the workplan document model and generated docs`
- `45ff25260 test(docs-contract): check the repository through raf-workplan`
- `bf08e459e feat(raf-source)!: seal the public types consumers bind to`
- `df71a428c fix(raf-source): match items_under by whole segment and check binding roles`
- `20b59c5d1 test(raf-source): exercise caps, spawn failure and source text through a load`
- `d89b78f3d feat(raf-okf): expose a fenced block's content and whether it is closed`
- `061798548 fix(raf-source): place a module under its declared path`

### History: `docs/capabilities/workplan-two-way-coverage.md`

- `f711aa29e docs(workplan): define the seven sections and close the workplan contract gaps`
- `9f93e5de0 docs(capabilities): add considered workplan capabilities and retire the essay`

## Identifiers (files containing each at HEAD)

- `reconcile`: 51
- `complete`: 57
- `delivered`: 71
- `considered`: 52
- `exec`: 62
- `git`: 105
- `Gate::open`: 8
- `Workplan`: 62
- `Graph`: 10
- `section_digest`: 7
- `Lifecycle`: 21
- `obligation`: 46
- `Workplan::write`: 15
- `check`: 166
- `scope`: 30
- `make`: 91
- `Evidence`: 21
- `Assessment`: 0
- `subject`: 28
- `inputs`: 18
- `verdict`: 25
- `Coverage`: 1
- `exclusions`: 1
- `DocType`: 16
- `Rule::ALL`: 6
- `ErrorKind::ALL`: 6
- `section_of`: 10
- `raf_workplan::digest`: 0
- `section`: 104
- `symbol`: 74
- `record`: 147
- `Program::Trusted`: 0
- `bound::Executor`: 5
- `EvidenceRecord`: 0
- `bound::Evidence`: 4
- `revision`: 14
- `trace`: 3
- `Workplan::reconcile`: 7
- `accepted`: 54
- `affects`: 75
- `assess`: 12
- `true`: 136
- `exec::Invocation`: 4
- `cargo`: 114
- `fail`: 176
- `command`: 80
- `create_new`: 9
- `implemented`: 49
- `dropped`: 57
- `changes`: 123
- `implementing`: 57
- `Lifecycle::process_obligations`: 0
- `DocType::ALL`: 4
- `human`: 10
- `agent`: 29
- `records`: 79
- `ObligationKind::ALL`: 4
- `claims`: 27
- `none`: 117
- `digest`: 19
- `history`: 22
- `INVENTORY`: 3
- `Number`: 10
- `raf_okf`: 80
- `bindings.examples`: 18
- `verify`: 41
- `no_run`: 16
- `raf_workplan::Gate`: 8
- `binding_site`: 0
- `exported_items`: 0
- `TrustedArgv::from_command_line`: 0
- `rust`: 100
- `super::X`: 0
- `has_unloaded_documents`: 0
- `HistoryError::SymbolLog`: 0

## Workspace packages named

- `cargo-workplan`
- `raf-okf`
- `raf-source`
- `raf-workplan`

## URLs

None.
