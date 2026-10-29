// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `Workplan::reconcile`: one fixture for each obligation kind the documents
//! can show, each asserting the kind, the location and the evidence text and
//! that fixing it removes exactly that line; the order and determinism of the
//! list; `contract_effect: none`; the refusals with a passing neighbour; and
//! hostile text.

mod support;

use std::collections::BTreeSet;

use raf_okf::{Actor, AssessmentVerdict};
use raf_workplan::ChangeLookupError;
use raf_workplan::obligation::{Needed, ObligationKind, ReconcileError, Reconciled};
use support::changes::{
    BOUND, DONE, OMEGA, SHAPE, base, capability, record, replaced, repo, without,
};
use support::disk::{Scratch, on_disk, snapshot, write_repo};
use support::evidence::{Evidence, OTHER_DIGEST, claim, real_digest};
use support::synced::{self, assessed, assessed_as, document, with_plan};
use support::{Repo, line_of};

const SIGMA: &str = "docs/capabilities/sigma.md";
const OTHER: &str = "work/changes/other.md";
const INDEX: &str = synced::INDEX;

/// The list of `change` once the sections it affects are assessed as the
/// documents stand ([`assessed`]): the fixtures that are about another kind
/// owe no assessment.
fn reconcile_err(repo: &Repo, change: &str) -> Result<Reconciled, ReconcileError> {
    reconcile_as_is(&assessed(repo, change), change)
}

/// The list of `change` over `repo` as it is, assessments and all.
fn reconcile_as_is(repo: &Repo, change: &str) -> Result<Reconciled, ReconcileError> {
    with_plan(repo, |plan| plan.reconcile(change))
}

fn reconcile(repo: &Repo) -> Reconciled {
    reconcile_err(repo, "shape").unwrap_or_else(|e| panic!("`reconcile` should succeed: {e}"))
}

/// The lines `reconcile` prints.
fn lines(repo: &Repo) -> Vec<String> {
    reconcile(repo)
        .to_string()
        .lines()
        .map(str::to_string)
        .collect()
}

// --- the cases ---------------------------------------------------------------

/// What one obligation of a case must be.
struct Want {
    kind: ObligationKind,
    /// The file the obligation points at.
    path: &'static str,
    /// The line, for an obligation about text.
    line: Option<usize>,
    /// The whole line `reconcile` prints.
    text: String,
}

/// A repository with one fault, the obligations the fault owes and the
/// repository with the fault put right.
struct Case {
    name: &'static str,
    broken: Repo,
    fixed: Repo,
    want: Vec<Want>,
}

impl Case {
    fn run(&self) {
        let list = reconcile(&self.broken);
        assert_eq!(list.len(), self.want.len(), "{}: {list}", self.name);
        for (obligation, want) in list.obligations().iter().zip(&self.want) {
            assert_eq!(obligation.kind, want.kind, "{}", self.name);
            assert_eq!(obligation.location.path, want.path, "{}", self.name);
            assert_eq!(obligation.location.line, want.line, "{}", self.name);
            assert_eq!(obligation.to_string(), want.text, "{}", self.name);
        }
        // The fix removes the line and puts no other in its place.
        let fixed = reconcile(&self.fixed);
        assert!(fixed.is_empty(), "{}: still owed:\n{fixed}", self.name);
    }
}

const REASON_NOT_FOUND: &str = "`demo::missing` names nothing in the workspace";

const PROPOSAL_EVIDENCE: &str =
    "the block is replaced by plain contract text, or removed because the claim was dropped";
const OUTCOME_EVIDENCE: &str = "an outcome of implemented, dropped or moved:change:<slug>";
const BINDING_EVIDENCE: &str = "a binding that resolves through the source index";
const SYNC_EVIDENCE: &str = "a docs sync run";
const TENSE_EVIDENCE: &str = "text rewritten as the supported behaviour";
const CLAIM_EVIDENCE: &str = "a passing record within its limits for each evidence id the claim names, or the claim reworded or removed";
const STALE_EVIDENCE: &str = "the evidence recorded again with evidence record";

const BLOCK: &str = "\n> [!PROPOSED change:shape]\n> The candidate contract.\n";

fn cases() -> Vec<Case> {
    let base = base();
    let mut cases = Vec::new();
    proposal_cases(&base, &mut cases);
    item_cases(&mut cases);
    binding_cases(&base, &mut cases);
    stale_cases(&base, &mut cases);
    tense_cases(&base, &mut cases);
    unloaded_proposal_cases(&base, &mut cases);
    trace_cases(&mut cases);
    review_cases(&base, &mut cases);
    cases
}

/// Missing reviews and persisted reviews of changed source both owe a current
/// review; a findings verdict supplies coverage without claiming clean code.
fn review_cases(base: &Repo, cases: &mut Vec<Case>) {
    use raf_okf::{DocType, RecordHeader, ReviewKind, ReviewRecord, ReviewScope, ReviewVerdict};
    use raf_source::MemoryTree;
    use raf_workplan::Workplan;

    let required = replaced(
        base,
        OMEGA,
        "depends:",
        "review_required: [security]\ndepends:",
    );
    let scope = ReviewScope::Capability("cap:omega".to_string());
    let current_inputs = with_plan(&required, |plan| plan.review_inputs(&scope).unwrap());
    let mut header = RecordHeader::new(DocType::Review, "omega-security");
    header.timestamp = Some("2026-01-01T00:00:00Z".to_string());
    let mut review = ReviewRecord::new(
        header,
        scope.clone(),
        "a".repeat(40),
        ReviewKind::Security,
        ReviewVerdict::Findings,
        Actor::Human,
        current_inputs,
    );
    review.findings_ref = Some("https://example.invalid/review".to_string());
    let path = "work/reviews/omega-security.md".to_string();
    let mut current = required.clone();
    current.insert(path.clone(), review.to_document().unwrap().render());
    let evidence = "a completed current review of the required kind recorded for the capability";
    cases.push(Case {
        name: "review-missing",
        broken: required.clone(),
        fixed: current.clone(),
        want: vec![Want {
            kind: ObligationKind::ReviewMissing,
            path: OMEGA,
            line: None,
            text: format!(
                "cap:omega  document  review-missing  {evidence} (no completed security review)"
            ),
        }],
    });
    // Compute the prior record from actual changed source, rather than inventing
    // a mismatched digest. The current source is the shared demo workspace.
    let mut prior_files: Vec<_> = support::demo_files()
        .into_iter()
        .map(|(path, text)| (path, text.to_string()))
        .collect();
    let (_, lib) = prior_files
        .iter_mut()
        .find(|(path, _)| *path == "src/lib.rs")
        .unwrap();
    assert!(
        lib.contains("pub fn real() {}"),
        "the prior source edit matches"
    );
    *lib = lib.replace("pub fn real() {}", "pub fn real() { let _value = 1; }");
    let prior = support::workspace_of(&prior_files);
    let tree: MemoryTree = required.iter().collect();
    review.inputs = Workplan::load(&tree, &prior).review_inputs(&scope).unwrap();
    let mut stale = required;
    stale.insert(path, review.to_document().unwrap().render());
    cases.push(Case {
        name: "review-stale",
        broken: stale,
        fixed: current,
        want: vec![Want {
            kind: ObligationKind::ReviewStale,
            path: OMEGA,
            line: None,
            text: format!(
                "cap:omega  document  review-stale  {evidence} (security review `review:omega-security` has changed inputs: [ReviewChange {{ item: \"demo::real\", kind: Changed }}])"
            ),
        }],
    });
}

fn proposal_cases(base: &Repo, cases: &mut Vec<Case>) {
    // unresolved-proposal
    {
        let with_block = repo(&[
            (
                OMEGA,
                capability("omega", "implementing", BOUND, &[("Contract", BLOCK)]),
            ),
            (SHAPE, base[SHAPE].clone()),
        ]);
        let line = line_of(&with_block[OMEGA], "[!PROPOSED change:shape]");
        cases.push(Case {
            name: "unresolved-proposal",
            fixed: replaced(
                &with_block,
                OMEGA,
                "> [!PROPOSED change:shape]\n> The candidate contract.\n",
                "The contract, stated plainly.\n",
            ),
            want: vec![Want {
                kind: ObligationKind::UnresolvedProposal,
                path: OMEGA,
                line: Some(line),
                text: format!(
                    "cap:omega  contract  unresolved-proposal  line {line}: {PROPOSAL_EVIDENCE}"
                ),
            }],
            broken: with_block,
        });
    }
}

fn item_cases(cases: &mut Vec<Case>) {
    // item-without-outcome: no outcome, a move to a change that does not
    // exist, and a move to the record itself.
    {
        let item = |outcome: &str| format!("{DONE}  - claim: \"omega is documented\"\n{outcome}");
        let other = record(
            "other",
            "[cap:omega]",
            &["cap:omega#contract"],
            "changes",
            DONE,
        );
        let repo_with = |items: String| {
            repo(&[
                (OMEGA, capability("omega", "implementing", BOUND, &[])),
                (
                    SHAPE,
                    record(
                        "shape",
                        "[cap:omega]",
                        &["cap:omega#contract"],
                        "changes",
                        &items,
                    ),
                ),
                (OTHER, other.clone()),
            ])
        };
        let fixed = repo_with(item(
            "    outcome: moved:change:other\n    reason: The other change owns this work.\n",
        ));
        let wanted = |evidence: &str| {
            vec![Want {
                kind: ObligationKind::ItemWithoutOutcome,
                path: SHAPE,
                line: None,
                text: format!("change:shape  item 2  item-without-outcome  {evidence}"),
            }]
        };
        cases.push(Case {
            name: "item-without-outcome",
            broken: repo_with(item("")),
            fixed: fixed.clone(),
            want: wanted(OUTCOME_EVIDENCE),
        });
        cases.push(Case {
            name: "item-moved-to-a-missing-change",
            broken: repo_with(item("    outcome: moved:change:ghost\n    reason: The named change is intended to own this work.\n")),
            fixed: fixed.clone(),
            want: wanted(&format!(
                "{OUTCOME_EVIDENCE} (`moved:change:ghost` names no change record)"
            )),
        });
        cases.push(Case {
            name: "item-moved-to-itself",
            broken: repo_with(item("    outcome: moved:change:shape\n    reason: The named change is intended to own this work.\n")),
            fixed,
            want: wanted(&format!(
                "{OUTCOME_EVIDENCE} (the item moves to the change that is being completed)"
            )),
        });
    }
    // outcome-without-reason: under strict mode, a dropped item and a moved
    // one, each without a reason.
    for (outcome, detail) in [
        ("dropped", "`outcome: dropped` has no `reason`"),
        (
            "moved:change:other",
            "`outcome: moved:change:other` has no `reason`",
        ),
    ] {
        let reasoned = |reason: &str| strict_items(&format!("    outcome: {outcome}\n{reason}"));
        cases.push(Case {
            name: if outcome == "dropped" {
                "outcome-without-reason-dropped"
            } else {
                "outcome-without-reason-moved"
            },
            broken: reasoned(""),
            fixed: reasoned("    reason: \"The other change carries it.\"\n"),
            want: vec![Want {
                kind: ObligationKind::OutcomeWithoutReason,
                path: SHAPE,
                line: None,
                text: format!(
                    "change:shape  item 2  outcome-without-reason  a reason for the outcome ({detail})"
                ),
            }],
        });
    }
}

/// The coverage record that says `strictness: strict` and enforces nothing.
const STRICT_COVERAGE: &str =
    "---\ntype: Coverage\nid: coverage:default\nscope: []\nexclusions: []\n---\n";

/// The repository of the fixtures under a strict coverage record, with a
/// second item of `shape` whose lines after its claim are `rest`.
fn strict_items(rest: &str) -> Repo {
    repo(&[
        (OMEGA, capability("omega", "implementing", BOUND, &[])),
        (
            SHAPE,
            record(
                "shape",
                "[cap:omega]",
                &["cap:omega#contract"],
                "changes",
                &format!("{DONE}  - claim: \"omega is documented\"\n{rest}"),
            ),
        ),
        (
            OTHER,
            record(
                "other",
                "[cap:omega]",
                &["cap:omega#contract"],
                "changes",
                DONE,
            ),
        ),
        (support::COVERAGE, STRICT_COVERAGE.to_string()),
    ])
}

/// Under `default` a dropped or moved item needs no reason, and a reason is
/// accepted beside any outcome or none under either strictness.
#[test]
fn a_dropped_or_moved_item_needs_a_reason_and_an_explained_outcome_is_accepted() {
    for outcome in ["dropped", "moved:change:other"] {
        let strict = strict_items(&format!("    outcome: {outcome}\n"));
        assert_eq!(
            reconcile(&strict)
                .of_kind(ObligationKind::OutcomeWithoutReason)
                .count(),
            1,
            "{outcome}"
        );
        let explained = strict_items(&format!(
            "    outcome: {outcome}\n    reason: The outcome follows the documented contract.\n"
        ));
        assert!(reconcile(&explained).is_empty(), "{outcome}");
    }
    for rest in [
        "    outcome: implemented\n    reason: \"It is built.\"\n",
        "    reason: \"Written before the outcome.\"\n",
    ] {
        let list = reconcile(&strict_items(rest));
        assert_eq!(
            list.of_kind(ObligationKind::OutcomeWithoutReason).count(),
            0,
            "{rest}: {list}"
        );
    }
}

fn binding_cases(base: &Repo, cases: &mut Vec<Case>) {
    // binding-missing
    let binding = |state: &str, bindings: [&str; 3]| {
        repo(&[
            (OMEGA, capability("omega", state, bindings, &[])),
            (SHAPE, base[SHAPE].clone()),
        ])
    };
    let owed = |detail: &str| {
        vec![Want {
            kind: ObligationKind::BindingMissing,
            path: OMEGA,
            line: None,
            text: format!(
                "cap:omega  verification  binding-missing  {BINDING_EVIDENCE} ({detail})"
            ),
        }]
    };
    cases.push(Case {
        name: "implementing-with-an-empty-tests-list",
        broken: binding("implementing", [BOUND[0], "[]", BOUND[2]]),
        fixed: base.clone(),
        want: owed("bindings.tests is empty"),
    });
    cases.push(Case {
        name: "implementing-with-an-empty-examples-list",
        broken: binding("implementing", [BOUND[0], BOUND[1], "[]"]),
        fixed: base.clone(),
        want: owed("bindings.examples is empty"),
    });
    // The Contract names no bound symbol, so every code binding is an input
    // of its assessment, and one that does not resolve leaves the section
    // with no assessment until it does.
    let mut want = vec![assessment_missing()];
    want.extend(owed(&format!(
        "bindings.code `demo::missing` does not resolve: {REASON_NOT_FOUND}"
    )));
    cases.push(Case {
        name: "implementing-with-a-path-that-does-not-resolve",
        broken: binding("implementing", ["[demo::missing]", BOUND[1], BOUND[2]]),
        fixed: base.clone(),
        want,
    });
    cases.push(Case {
        name: "implementing-with-a-test_that_is_not_a_test",
        broken: binding("implementing", [BOUND[0], "[demo::real]", BOUND[2]]),
        fixed: base.clone(),
        want: owed(
            "bindings.tests `demo::real` names an item that is not a function marked `#[test]`",
        ),
    });
    // A delivered capability: an empty list of code or tests is a rule of the
    // profile, so the document does not load and `check` reports
    // `okf-binding-required`; it owes the binding all the same. The catalogue
    // reads the loaded capabilities only, so it is stale until the document
    // loads again.
    let delivered = binding("delivered", BOUND);
    let index_line = |repo: &Repo| line_of(&repo[INDEX], "raf:generated:catalogue");
    let unbound = |from: &str, to: &str, detail: &str| {
        let broken = replaced(&delivered, OMEGA, from, to);
        let mut want = owed(detail);
        want.push(Want {
            kind: ObligationKind::GeneratedStale,
            path: INDEX,
            line: Some(index_line(&broken)),
            text: format!(
                "docs/capabilities/index.md  document  generated-stale  line {}: {SYNC_EVIDENCE} (the `catalogue` block is stale)",
                index_line(&broken)
            ),
        });
        (broken, want)
    };
    let (broken, want) = unbound(
        "  tests: [demo::test_crate::it::case]",
        "  tests: []",
        "bindings.tests is empty",
    );
    cases.push(Case {
        name: "delivered-with-an-empty-tests-list",
        broken,
        fixed: delivered.clone(),
        want,
    });
    let (broken, want) = unbound(
        "  code: [demo::real]",
        "  code: []",
        "bindings.code is empty",
    );
    cases.push(Case {
        name: "delivered-with-an-empty-code-list",
        broken,
        fixed: delivered.clone(),
        want,
    });
    cases.push(Case {
        name: "delivered-with-a-path-that-does-not-resolve",
        broken: binding("delivered", [BOUND[0], "[demo::missing]", BOUND[2]]),
        fixed: delivered,
        want: owed(&format!(
            "bindings.tests `demo::missing` does not resolve: {REASON_NOT_FOUND}"
        )),
    });
}

fn stale_cases(base: &Repo, cases: &mut Vec<Case>) {
    // generated-stale: a block of a listed capability, the catalogue of the
    // index, and an index that does not exist.
    {
        let stale = replaced(
            base,
            OMEGA,
            "| Item | Kind | Section |",
            "| Item | Kind | Where |",
        );
        let line = line_of(&stale[OMEGA], "raf:generated:verification-matrix");
        cases.push(Case {
            name: "generated-stale-block",
            fixed: base.clone(),
            want: vec![Want {
                kind: ObligationKind::GeneratedStale,
                path: OMEGA,
                line: Some(line),
                text: format!(
                    "cap:omega  verification  generated-stale  line {line}: {SYNC_EVIDENCE} (the `verification-matrix` block is stale)"
                ),
            }],
            broken: stale,
        });
        let stale_index = replaced(base, INDEX, "| Shaped. |", "| Reshaped. |");
        let line = line_of(&stale_index[INDEX], "raf:generated:catalogue");
        cases.push(Case {
            name: "generated-stale-index",
            fixed: base.clone(),
            want: vec![Want {
                kind: ObligationKind::GeneratedStale,
                path: INDEX,
                line: Some(line),
                text: format!(
                    "docs/capabilities/index.md  document  generated-stale  line {line}: {SYNC_EVIDENCE} (the `catalogue` block is stale)"
                ),
            }],
            broken: stale_index,
        });
        cases.push(Case {
            name: "generated-stale-index-missing",
            fixed: base.clone(),
            want: vec![Want {
                kind: ObligationKind::GeneratedStale,
                path: INDEX,
                line: None,
                text: format!(
                    "docs/capabilities/index.md  document  generated-stale  {SYNC_EVIDENCE} (docs/capabilities/index.md does not exist)"
                ),
            }],
            broken: without(base, INDEX),
        });
    }
}

/// `omega` with a claim on `bound` in its Contract, the record of `record`
/// beside it when there is one, and the change `shape`.
fn claimed(record: Option<&Evidence>) -> Repo {
    let marker = claim("bound");
    let mut files = vec![
        (
            OMEGA,
            capability("omega", "implementing", BOUND, &[("Contract", &marker)]),
        ),
        (SHAPE, base()[SHAPE].clone()),
    ];
    let path;
    if let Some(record) = record {
        path = record.path();
        files.push((path.as_str(), record.text()));
    }
    repo(&files)
}

fn trace_cases(cases: &mut Vec<Case>) {
    let passing = Evidence::passing("bound", "[cap:omega]");
    let fixed = claimed(Some(&passing));
    let line = line_of(&fixed[OMEGA], "raf:claim");
    let unsupported = |detail: &str| Want {
        kind: ObligationKind::ClaimUnsupported,
        path: OMEGA,
        line: Some(line),
        text: format!(
            "cap:omega  contract  claim-unsupported  line {line}: {CLAIM_EVIDENCE} ({detail})"
        ),
    };
    // claim-unsupported: no record, a failing record, and one over its limit.
    cases.push(Case {
        name: "claim-unsupported-missing",
        broken: claimed(None),
        fixed: fixed.clone(),
        // A claim on a record that does not load leaves its section with no
        // assessment: an assessment keeps the digest of each record it read.
        want: vec![
            unsupported("`evidence:bound` is not a loaded evidence record"),
            assessment_missing(),
        ],
    });
    cases.push(Case {
        name: "claim-unsupported-failing",
        broken: claimed(Some(&passing.clone().verdict("fail"))),
        fixed: fixed.clone(),
        want: vec![unsupported("`evidence:bound` has the verdict fail")],
    });
    cases.push(Case {
        name: "claim-unsupported-over-limit",
        broken: claimed(Some(
            &passing
                .clone()
                .verdict("fail")
                .values(&[("peak", "80")])
                .limits(&[("peak", "64")]),
        )),
        fixed: claimed(Some(
            &passing
                .clone()
                .values(&[("peak", "41")])
                .limits(&[("peak", "64")]),
        )),
        // A record that fails and holds a value over its limit owes both.
        want: vec![
            unsupported("`evidence:bound` has the verdict fail"),
            unsupported("`evidence:bound`: `peak` is 80, over its limit 64"),
        ],
    });
    // evidence-stale: a record made against another text of `demo::real`,
    // reached by a claim and by `for` alone.
    let stale = passing
        .clone()
        .inputs(vec![("demo::real".to_string(), OTHER_DIGEST.to_string())]);
    let stale_want = || Want {
        kind: ObligationKind::EvidenceStale,
        path: "work/evidence/bound.md",
        line: None,
        text: format!(
            "cap:omega  document  evidence-stale  {STALE_EVIDENCE} (`evidence:bound` was made against `demo::real`, which has changed since)"
        ),
    };
    cases.push(Case {
        name: "evidence-stale-claimed",
        broken: claimed(Some(&stale)),
        fixed: fixed.clone(),
        want: vec![stale_want()],
    });
    let by_for = |record: &Evidence| {
        repo(&[
            (OMEGA, capability("omega", "implementing", BOUND, &[])),
            (SHAPE, base()[SHAPE].clone()),
            (record.path().as_str(), record.text()),
        ])
    };
    cases.push(Case {
        name: "evidence-stale-by-for",
        broken: by_for(&stale),
        fixed: by_for(&passing),
        want: vec![stale_want()],
    });
    // An input that resolves to nothing now.
    let gone = passing.clone().inputs(vec![
        ("demo::real".to_string(), real_digest()),
        ("demo::vanished".to_string(), OTHER_DIGEST.to_string()),
    ]);
    cases.push(Case {
        name: "evidence-stale-unresolved",
        broken: by_for(&gone),
        fixed: by_for(&passing),
        want: vec![Want {
            kind: ObligationKind::EvidenceStale,
            path: "work/evidence/bound.md",
            line: None,
            text: format!(
                "cap:omega  document  evidence-stale  {STALE_EVIDENCE} (`evidence:bound` was made against `demo::vanished`, which does not resolve now: `demo::vanished` names nothing in the workspace)"
            ),
        }],
    });
}

fn tense_cases(base: &Repo, cases: &mut Vec<Case>) {
    // tense-mismatch
    {
        let with_tense = |text: &str| {
            repo(&[
                (
                    OMEGA,
                    capability("omega", "implementing", BOUND, &[("Contract", text)]),
                ),
                (SHAPE, base[SHAPE].clone()),
            ])
        };
        let broken = with_tense("\nThe parser will return an error.\n");
        let line = line_of(&broken[OMEGA], "will return");
        cases.push(Case {
            name: "tense-mismatch",
            fixed: with_tense("\nThe parser returns an error.\n"),
            want: vec![Want {
                kind: ObligationKind::TenseMismatch,
                path: OMEGA,
                line: Some(line),
                text: format!(
                    "cap:omega  contract  tense-mismatch  line {line}: {TENSE_EVIDENCE} (the text says \"will\")"
                ),
            }],
            broken,
        });
        // The phrase wrapped over two lines, found at the line it starts on.
        let wrapped = with_tense("\nStreaming support is\nplanned for the parser.\n");
        let line = line_of(&wrapped[OMEGA], "Streaming support is");
        cases.push(Case {
            name: "tense-mismatch-wrapped",
            fixed: with_tense("\nStreaming support is available.\n"),
            want: vec![Want {
                kind: ObligationKind::TenseMismatch,
                path: OMEGA,
                line: Some(line),
                text: format!(
                    "cap:omega  contract  tense-mismatch  line {line}: {TENSE_EVIDENCE} (the text says \"is planned\")"
                ),
            }],
            broken: wrapped,
        });
        // A delivered capability with a PROPOSED block owes the block twice:
        // it is unresolved, and a delivered capability states no proposal.
        let delivered = |extra: &[(&str, &str)]| {
            repo(&[
                (OMEGA, capability("omega", "delivered", BOUND, extra)),
                (SHAPE, base[SHAPE].clone()),
            ])
        };
        let broken = delivered(&[("Contract", BLOCK)]);
        let line = line_of(&broken[OMEGA], "[!PROPOSED change:shape]");
        cases.push(Case {
            name: "tense-mismatch-proposed-block-in-a-delivered-capability",
            fixed: delivered(&[]),
            want: vec![
                Want {
                    kind: ObligationKind::UnresolvedProposal,
                    path: OMEGA,
                    line: Some(line),
                    text: format!(
                        "cap:omega  contract  unresolved-proposal  line {line}: {PROPOSAL_EVIDENCE}"
                    ),
                },
                Want {
                    kind: ObligationKind::TenseMismatch,
                    path: OMEGA,
                    line: Some(line),
                    text: format!(
                        "cap:omega  contract  tense-mismatch  line {line}: {TENSE_EVIDENCE} (a PROPOSED block of change:shape stands in a delivered capability)"
                    ),
                },
            ],
            broken,
        });
    }
}

/// A block of the change in a capability document that does not load is owed
/// all the same: `check` reports the document, and the block is still there.
fn unloaded_proposal_cases(base: &Repo, cases: &mut Vec<Case>) {
    // An unlisted capability that has no title, with a block.
    let zeta = |block: &str, title: bool| {
        let text = capability(
            "zeta",
            "implementing",
            ["[]", "[]", "[]"],
            &[("Contract", block)],
        );
        let mut repo = base.clone();
        let text = if title {
            text
        } else {
            text.replace("title: \"Title of zeta\"\n", "")
        };
        repo.insert(SIGMA.replace("sigma", "zeta"), text);
        repo
    };
    let broken = zeta(BLOCK, false);
    let line = line_of(
        &broken["docs/capabilities/zeta.md"],
        "[!PROPOSED change:shape]",
    );
    cases.push(Case {
        name: "unresolved-proposal-in-a-capability-that-does-not-load",
        fixed: zeta("", false),
        want: vec![Want {
            kind: ObligationKind::UnresolvedProposal,
            path: "docs/capabilities/zeta.md",
            line: Some(line),
            text: format!(
                "cap:zeta  document  unresolved-proposal  line {line}: {PROPOSAL_EVIDENCE}"
            ),
        }],
        broken,
    });

    // A delivered capability that only binds no test: its block is owed with
    // the binding, in the document as a whole.
    let delivered = |block: &[(&str, &str)], tests: &str| {
        repo(&[
            (
                OMEGA,
                capability("omega", "delivered", [BOUND[0], tests, BOUND[2]], block),
            ),
            (SHAPE, base[SHAPE].clone()),
        ])
    };
    let unbound = replaced(
        &delivered(&[("Contract", BLOCK)], BOUND[1]),
        OMEGA,
        "  tests: [demo::test_crate::it::case]",
        "  tests: []",
    );
    let line = line_of(&unbound[OMEGA], "[!PROPOSED change:shape]");
    let index_line = line_of(&unbound[INDEX], "raf:generated:catalogue");
    cases.push(Case {
        name: "unresolved-proposal-in-an-unbound-delivered-capability",
        fixed: delivered(&[], BOUND[1]),
        want: vec![
            Want {
                kind: ObligationKind::UnresolvedProposal,
                path: OMEGA,
                line: Some(line),
                text: format!("cap:omega  document  unresolved-proposal  line {line}: {PROPOSAL_EVIDENCE}"),
            },
            Want {
                kind: ObligationKind::BindingMissing,
                path: OMEGA,
                line: None,
                text: format!(
                    "cap:omega  verification  binding-missing  {BINDING_EVIDENCE} (bindings.tests is empty)"
                ),
            },
            Want {
                kind: ObligationKind::GeneratedStale,
                path: INDEX,
                line: Some(index_line),
                text: format!(
                    "docs/capabilities/index.md  document  generated-stale  line {index_line}: {SYNC_EVIDENCE} (the `catalogue` block is stale)"
                ),
            },
        ],
        broken: unbound,
    });
}

#[test]
fn a_block_of_another_change_in_a_capability_that_does_not_load_is_not_owed() {
    let mut repo = base();
    let zeta = capability(
        "zeta",
        "implementing",
        ["[]", "[]", "[]"],
        &[("Contract", "\n> [!PROPOSED change:other]\n> Theirs.\n")],
    )
    .replace("title: \"Title of zeta\"\n", "");
    repo.insert("docs/capabilities/zeta.md".to_string(), zeta);
    // The document is broken and the block is not ours.
    assert!(
        synced::check_all(&repo)
            .iter()
            .any(|f| f.kind == "okf-missing-key")
    );
    assert_eq!(lines(&repo), Vec::<String>::new());
}

#[test]
fn each_fault_is_owed_with_its_kind_location_and_evidence_and_fixing_it_removes_the_line() {
    for case in cases() {
        case.run();
    }
}

// --- the assessments ------------------------------------------------------------

/// The `assessment-missing` line of the Contract of `omega`.
fn assessment_missing() -> Want {
    Want {
        kind: ObligationKind::AssessmentMissing,
        path: OMEGA,
        line: None,
        text: format!(
            "cap:omega  contract  assessment-missing  {ASSESSED_EVIDENCE} (`cap:omega#contract` has no assessment `assessment:omega-contract`)"
        ),
    }
}

const ASSESSMENT: &str = "work/assessments/omega-contract.md";
const ASSESSED_EVIDENCE: &str = "an assessment of the section recorded with assess";
const ASSESS_AGAIN_EVIDENCE: &str = "the section assessed again with assess";
const ACCEPTED_EVIDENCE: &str =
    "the section revised and assessed again with assess, with the verdict accepted";

/// A repository judged as it is, assessments and all: the section `omega`'s
/// Contract, the one `shape` affects, with no assessment, a stale one or a
/// rejected one, beside the repository whose assessment is current and
/// accepted.
fn assessment_cases() -> Vec<Case> {
    let base = base();
    let current = assessed(&base, "shape");
    let mut cases = Vec::new();
    cases.push(Case {
        name: "assessment-missing",
        broken: base.clone(),
        fixed: current.clone(),
        want: vec![assessment_missing()],
    });
    let stale = |detail: &str| Want {
        kind: ObligationKind::AssessmentStale,
        path: ASSESSMENT,
        line: None,
        text: format!(
            "cap:omega  contract  assessment-stale  {ASSESS_AGAIN_EVIDENCE} (`assessment:omega-contract`: {detail})"
        ),
    };
    // The section edited after the assessment.
    let edited = replaced(
        &current,
        OMEGA,
        "Text for Contract of omega.\n",
        "Text for Contract of omega, edited.\n",
    );
    cases.push(Case {
        name: "assessment-stale-section",
        fixed: assessed(&edited, "shape"),
        broken: edited,
        want: vec![stale("the section `cap:omega#contract` has changed since")],
    });
    // The record was made against another text of the bound symbol.
    let real = real_digest();
    let other_symbol = replaced(&current, ASSESSMENT, &real, OTHER_DIGEST);
    cases.push(Case {
        name: "assessment-stale-symbol",
        broken: other_symbol,
        fixed: current.clone(),
        want: vec![stale("the symbol `demo::real` has changed since")],
    });
    // A current assessment with the verdict rejected, by a human or an agent.
    for by in Actor::ALL {
        cases.push(Case {
            name: if by == Actor::Human {
                "assessment-rejected-by-a-human"
            } else {
                "assessment-rejected-by-an-agent"
            },
            broken: assessed_as(&base, "shape", AssessmentVerdict::Rejected, by),
            fixed: current.clone(),
            want: vec![Want {
                kind: ObligationKind::AssessmentRejected,
                path: ASSESSMENT,
                line: None,
                text: format!(
                    "cap:omega  contract  assessment-rejected  {ACCEPTED_EVIDENCE} (`assessment:omega-contract` is current and has the verdict rejected, by {})",
                    by.name()
                ),
            }],
        });
    }
    cases
}

impl Case {
    /// As [`Case::run`], with the repositories judged as they are.
    fn run_as_is(&self) {
        let list = reconcile_as_is(&self.broken, "shape").expect("reconciles");
        assert_eq!(list.len(), self.want.len(), "{}: {list}", self.name);
        for (obligation, want) in list.obligations().iter().zip(&self.want) {
            assert_eq!(obligation.kind, want.kind, "{}", self.name);
            assert_eq!(obligation.location.path, want.path, "{}", self.name);
            assert_eq!(obligation.location.line, want.line, "{}", self.name);
            assert_eq!(obligation.to_string(), want.text, "{}", self.name);
        }
        let fixed = reconcile_as_is(&self.fixed, "shape").expect("reconciles");
        assert!(fixed.is_empty(), "{}: still owed:\n{fixed}", self.name);
    }
}

#[test]
fn each_assessment_fault_is_owed_at_the_section_naming_the_input_and_a_current_acceptance_clears_it()
 {
    for case in assessment_cases() {
        case.run_as_is();
    }
}

#[test]
fn contract_effect_none_keeps_the_assessments_and_another_section_is_not_judged() {
    let none = replaced(
        &base(),
        SHAPE,
        "contract_effect: changes",
        "contract_effect: none",
    );
    let owed = reconcile_as_is(&none, "shape").expect("reconciles");
    assert_eq!(
        owed.obligations()
            .iter()
            .map(|o| o.kind)
            .collect::<Vec<_>>(),
        [ObligationKind::AssessmentMissing],
        "{owed}"
    );
    // An edit of a section the change does not affect leaves the assessment
    // of the affected one current.
    let current = assessed(&base(), "shape");
    let other = replaced(
        &current,
        OMEGA,
        "Text for Usage of omega.\n",
        "Text for Usage of omega, edited.\n",
    );
    assert!(
        reconcile_as_is(&other, "shape")
            .expect("reconciles")
            .is_empty()
    );
}

#[test]
fn the_baseline_owes_nothing_and_check_agrees() {
    let base = base();
    assert!(reconcile(&base).is_empty());
    assert_eq!(reconcile(&base).to_string(), "");
    assert_eq!(synced::check_all(&base), []);
}

// --- every kind, in both directions ------------------------------------------

#[test]
fn every_static_kind_is_raised_by_a_fixture_and_the_kinds_that_need_a_process_by_none() {
    let mut raised: BTreeSet<&str> = BTreeSet::new();
    let mut names = BTreeSet::new();
    for case in cases() {
        assert!(names.insert(case.name), "{} is named twice", case.name);
        let list = reconcile(&case.broken);
        raised.extend(list.obligations().iter().map(|o| o.kind.name()));
    }
    for case in assessment_cases() {
        assert!(names.insert(case.name), "{} is named twice", case.name);
        let list = reconcile_as_is(&case.broken, "shape").expect("reconciles");
        raised.extend(list.obligations().iter().map(|o| o.kind.name()));
    }
    let known: BTreeSet<&str> = ObligationKind::ALL.iter().map(|k| k.name()).collect();
    let unknown: Vec<&&str> = raised.difference(&known).collect();
    assert!(
        unknown.is_empty(),
        "fixtures produce unlisted kinds {unknown:?}"
    );
    let statics: BTreeSet<&str> = ObligationKind::ALL
        .iter()
        .filter(|k| k.is_static())
        .map(|k| k.name())
        .collect();
    let unraised: Vec<&&str> = statics.difference(&raised).collect();
    assert!(unraised.is_empty(), "no fixture raises {unraised:?}");
    // The kinds that start a process are in the closed set and are not raised
    // from the documents.
    let dynamic: BTreeSet<&str> = known.difference(&statics).copied().collect();
    assert_eq!(
        dynamic,
        BTreeSet::from([
            "section-untouched",
            "contract-effect-understated",
            "example-failing",
            "evidence-changed"
        ])
    );
    assert!(raised.is_disjoint(&dynamic), "{raised:?}");
}

// --- derived, not re-implemented ---------------------------------------------

#[test]
fn the_binding_obligations_are_the_gaps_of_the_roadmap() {
    // Every list empty, then three entries that do not resolve.
    let none = repo(&[
        (
            OMEGA,
            capability("omega", "implementing", ["[]", "[]", "[]"], &[]),
        ),
        (SHAPE, base()[SHAPE].clone()),
    ]);
    let unresolved = repo(&[
        (
            OMEGA,
            capability(
                "omega",
                "implementing",
                ["[demo::nowhere]", "[demo::missing]", "[demo::absent]"],
                &[],
            ),
        ),
        (SHAPE, base()[SHAPE].clone()),
    ]);
    for repo in [none, unresolved] {
        let list = reconcile(&repo);
        let gaps = with_plan(&repo, |plan| {
            plan.status()
                .get("omega")
                .map(|entry| {
                    entry
                        .gaps()
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                })
                .expect("an entry")
        });
        assert_eq!(gaps.len(), 3);
        let owed: Vec<String> = list
            .of_kind(ObligationKind::BindingMissing)
            .map(|o| o.evidence.to_string())
            .collect();
        assert_eq!(owed.len(), gaps.len(), "{list}");
        for gap in &gaps {
            let prefix = format!("{BINDING_EVIDENCE} ({gap}");
            assert_eq!(
                owed.iter().filter(|text| text.starts_with(&prefix)).count(),
                1,
                "no single obligation for `{gap}` in {owed:#?}"
            );
        }
    }
}

#[test]
fn a_delivered_capability_owes_code_and_tests_but_not_examples() {
    let repo = repo(&[
        (
            OMEGA,
            capability("omega", "delivered", [BOUND[0], BOUND[1], "[]"], &[]),
        ),
        (SHAPE, base()[SHAPE].clone()),
    ]);
    // `check` is satisfied with the bound code and test, and so is reconcile.
    assert!(reconcile(&repo).is_empty());
    // The same capability while implementing owes the empty examples list.
    let implementing = synced::synced(&repo_with_state(&repo, "implementing"));
    assert_eq!(
        lines(&implementing),
        [format!(
            "cap:omega  verification  binding-missing  {BINDING_EVIDENCE} (bindings.examples is empty)"
        )]
    );
}

fn repo_with_state(repo: &Repo, state: &str) -> Repo {
    replaced(repo, OMEGA, "state: delivered", &format!("state: {state}"))
}

#[test]
fn capabilities_that_bind_nothing_owe_no_binding() {
    for state in ["designed", "considered"] {
        let proposal = "\n> [!PROPOSED change:shape]\n> The candidate.\n";
        // A designed capability states no block, a considered one needs it.
        // Wording that an implementing or delivered capability may not use is
        // allowed while the capability is only planned.
        let wording = "\nIt will start, and it is not yet done.\n";
        let extra: &[(&str, &str)] = if state == "considered" {
            &[("Purpose", proposal), ("Contract", wording)]
        } else {
            &[("Contract", wording)]
        };
        let repo = repo(&[
            (
                OMEGA,
                document("omega", "Shaped.", state, ["[]", "[]", "[]"], "[]", extra),
            ),
            (
                SHAPE,
                record(
                    "shape",
                    "[cap:omega]",
                    &["cap:omega#purpose"],
                    "changes",
                    DONE,
                ),
            ),
        ]);
        let kinds: Vec<ObligationKind> = reconcile(&repo)
            .obligations()
            .iter()
            .map(|o| o.kind)
            .collect();
        assert!(
            !kinds.contains(&ObligationKind::BindingMissing)
                && !kinds.contains(&ObligationKind::TenseMismatch),
            "{state}: {kinds:?}"
        );
    }
}

#[test]
fn the_proposals_are_those_of_the_graph_for_this_change_only() {
    // A block of another change in the same capability is not this change's.
    let both = repo(&[
        (
            OMEGA,
            capability(
                "omega",
                "implementing",
                BOUND,
                &[(
                    "Contract",
                    "\n> [!PROPOSED change:shape]\n> Ours.\n\n> [!PROPOSED change:other]\n> Theirs.\n",
                )],
            ),
        ),
        (
            SHAPE,
            record(
                "shape",
                "[cap:omega]",
                &["cap:omega#contract"],
                "changes",
                DONE,
            ),
        ),
        (
            OTHER,
            record(
                "other",
                "[cap:omega]",
                &["cap:omega#contract"],
                "changes",
                DONE,
            ),
        ),
    ]);
    let ours = with_plan(&both, |plan| {
        plan.graph()
            .proposals()
            .iter()
            .filter(|p| p.change() == "shape")
            .count()
    });
    assert_eq!(ours, 1);
    assert_eq!(lines(&both).len(), 1);
    let theirs = reconcile_err(&both, "other").expect("other reconciles");
    assert_eq!(theirs.len(), 1);
    assert_eq!(theirs.change(), "other");
}

// --- order and determinism ---------------------------------------------------

/// A repository in which `omega` and `sigma` are both listed and each owes
/// several kinds, so the order has something to do.
fn untidy() -> Repo {
    let blocks = "\n> [!PROPOSED change:shape]\n> One.\n\n> [!PROPOSED change:shape]\n> Two.\n";
    let items =
        format!("{DONE}  - claim: \"sigma is shaped\"\n  - claim: \"omega is documented\"\n");
    repo(&[
        (
            OMEGA,
            capability(
                "omega",
                "implementing",
                ["[demo::zebra, demo::aardvark]", "[]", BOUND[2]],
                &[
                    ("Contract", blocks),
                    ("Usage", "\nIt will start, and later it is not yet done.\n"),
                ],
            ),
        ),
        (
            SIGMA,
            capability(
                "sigma",
                "delivered",
                [BOUND[0], BOUND[1], "[]"],
                &[("Purpose", "\nThis is proposed.\n")],
            ),
        ),
        (
            SHAPE,
            record(
                "shape",
                "[cap:sigma, cap:omega]",
                &["cap:omega#contract", "cap:sigma#purpose"],
                "changes",
                &items,
            ),
        ),
    ])
}

#[test]
fn two_blocks_in_one_section_print_in_document_order_and_every_line_is_in_order() {
    let repo = untidy();
    let list = reconcile(&repo);
    let proposals: Vec<usize> = list
        .of_kind(ObligationKind::UnresolvedProposal)
        .map(|o| o.location.line.expect("a line"))
        .collect();
    assert_eq!(proposals.len(), 2);
    assert!(proposals[0] < proposals[1]);
    assert_eq!(
        proposals,
        [
            line_of(&repo[OMEGA], "> One."),
            line_of(&repo[OMEGA], "> Two.")
        ]
        .map(|line| line - 1)
    );

    // Two entries that fail in one list differ only in their evidence text,
    // and print in the order of that text, not of the list.
    let missing: Vec<String> = list
        .of_kind(ObligationKind::BindingMissing)
        .map(ToString::to_string)
        .collect();
    let zebra = missing.iter().position(|l| l.contains("demo::zebra"));
    let aardvark = missing.iter().position(|l| l.contains("demo::aardvark"));
    assert!(aardvark < zebra, "{missing:#?}");

    // The whole list is in the order of its sort key.
    let text = list.to_string();
    let heads: Vec<(&str, &str)> = text
        .lines()
        .map(|line| {
            let mut parts = line.splitn(4, "  ");
            (
                parts.next().expect("subject"),
                parts.next().expect("section"),
            )
        })
        .collect();
    let subjects: Vec<&str> = heads.iter().map(|(subject, _)| *subject).collect();
    let mut sorted = subjects.clone();
    sorted.sort_unstable();
    assert_eq!(subjects, sorted);
    // Within omega the sections come in document order.
    let order = [
        "purpose",
        "contract",
        "usage",
        "examples",
        "constraints",
        "explanation",
        "verification",
    ];
    let omega: Vec<usize> = heads
        .iter()
        .filter(|(subject, _)| *subject == "cap:omega")
        .map(|(_, section)| order.iter().position(|s| s == section).expect("a section"))
        .collect();
    assert!(omega.windows(2).all(|pair| pair[0] <= pair[1]), "{omega:?}");
    assert!(
        text.contains("change:shape  item 3  item-without-outcome"),
        "{text}"
    );
}

#[test]
fn equal_inputs_print_equal_output() {
    // The order of the sort key is proved by the unit test, which feeds a
    // shuffled and a reversed list; a memory tree lists in sorted order, so a
    // load order cannot be varied here.
    let repo = untidy();
    let first = reconcile(&repo);
    assert_eq!(first, reconcile(&repo), "two runs differ");
    assert_eq!(first.to_string(), reconcile(&repo).to_string());
    assert!(first.len() > 6);
}

#[test]
fn the_change_is_found_with_or_without_its_prefix() {
    let repo = untidy();
    assert_eq!(
        reconcile_err(&repo, "change:shape").expect("with the prefix"),
        reconcile(&repo)
    );
}

// --- the footprint of the change ---------------------------------------------

#[test]
fn only_the_capabilities_the_change_lists_are_judged() {
    // `sigma` is stale and untidy and not listed by `shape2`.
    let stale = repo(&[
        (OMEGA, capability("omega", "implementing", BOUND, &[])),
        (
            SIGMA,
            capability(
                "sigma",
                "implementing",
                ["[]", "[]", "[]"],
                &[("Contract", "\nIt will start.\n")],
            ),
        ),
        (
            SHAPE,
            record(
                "shape",
                "[cap:omega]",
                &["cap:omega#contract"],
                "changes",
                DONE,
            ),
        ),
        (
            OTHER,
            record(
                "other",
                "[cap:sigma]",
                &["cap:sigma#contract"],
                "changes",
                DONE,
            ),
        ),
    ]);
    let stale = replaced(
        &stale,
        SIGMA,
        "| Item | Kind | Section |",
        "| Item | Kind | Where |",
    );
    assert!(
        reconcile(&stale).is_empty(),
        "unlisted capability judged:\n{}",
        reconcile(&stale)
    );
    // The same faults are owed by the change that lists the capability.
    let other = reconcile_err(&stale, "other").expect("reconciles");
    let kinds: BTreeSet<ObligationKind> = other.obligations().iter().map(|o| o.kind).collect();
    assert_eq!(
        kinds,
        BTreeSet::from([
            ObligationKind::BindingMissing,
            ObligationKind::TenseMismatch,
            ObligationKind::GeneratedStale
        ])
    );
}

#[test]
fn fixing_one_fault_removes_exactly_its_lines() {
    let repo = untidy();
    let all = lines(&repo);
    // Each fix keeps the number of lines of its file, so no other obligation
    // is renumbered.
    let fixes: [(&str, &str, &str, &str); 3] = [
        (
            OMEGA,
            "It will start, and later it is not yet done.",
            "It starts.",
            "cap:omega  usage  tense-mismatch",
        ),
        (
            SIGMA,
            "This is proposed.",
            "This is stated.",
            "cap:sigma  purpose  tense-mismatch",
        ),
        (
            SHAPE,
            "  - claim: \"omega is documented\"\n",
            "  - claim: \"omega is documented\"\n    outcome: dropped\n    reason: The documented contract already covers this work.\n",
            "change:shape  item 3  item-without-outcome",
        ),
    ];
    for (path, from, to, prefix) in fixes {
        let after = lines(&replaced(&repo, path, from, to));
        let expected: Vec<&String> = all.iter().filter(|l| !l.starts_with(prefix)).collect();
        assert!(expected.len() < all.len(), "no line starts with `{prefix}`");
        assert_eq!(
            after.iter().collect::<Vec<_>>(),
            expected,
            "fixing `{from}`"
        );
    }
    // The two phrases of one sentence are two lines, and the fix removed both.
    assert_eq!(
        all.iter()
            .filter(|l| l.starts_with("cap:omega  usage  tense-mismatch"))
            .count(),
        2
    );
}

// --- contract_effect: none ---------------------------------------------------

#[test]
fn contract_effect_none_keeps_every_obligation_the_documents_can_show() {
    // The untidy repository, with a claim on evidence that is not there and a
    // record that is stale.
    let mut changes = replaced(
        &untidy(),
        SIGMA,
        "This is proposed.\n",
        &format!("This is proposed.\n{}", claim("ghost")),
    );
    let stale = Evidence::passing("bound", "[cap:sigma]")
        .inputs(vec![("demo::real".to_string(), OTHER_DIGEST.to_string())]);
    changes.insert(stale.path(), stale.text());
    let none = replaced(
        &changes,
        SHAPE,
        "contract_effect: changes",
        "contract_effect: none",
    );
    let with = reconcile(&changes);
    let without = reconcile(&none);
    assert_eq!(with.obligations(), without.obligations());
    let kinds: BTreeSet<ObligationKind> = without.obligations().iter().map(|o| o.kind).collect();
    assert_eq!(
        kinds,
        BTreeSet::from([
            ObligationKind::UnresolvedProposal,
            ObligationKind::ItemWithoutOutcome,
            ObligationKind::BindingMissing,
            ObligationKind::TenseMismatch,
            ObligationKind::ClaimUnsupported,
            ObligationKind::EvidenceStale,
            // A claim on a record that does not load, and code bindings that
            // do not resolve, leave the affected sections with no assessment.
            ObligationKind::AssessmentMissing,
        ])
    );
}

// --- refusals and their neighbours -------------------------------------------

#[test]
fn an_unknown_change_is_not_a_change_that_did_not_load() {
    let base = base();
    // The neighbour: the change that exists reconciles.
    assert!(reconcile_err(&base, "shape").is_ok());

    match reconcile_err(&base, "nope") {
        Err(ReconcileError::Change(ChangeLookupError::Unknown { slug, known })) => {
            assert_eq!(slug, "nope");
            assert_eq!(known, ["shape"]);
        }
        other => panic!("expected an unknown change, got {other:?}"),
    }

    // A record that exists and does not load is a different error.
    let broken = replaced(
        &base,
        SHAPE,
        "contract_effect: changes",
        "contract_effect: sometimes",
    );
    match reconcile_err(&broken, "shape") {
        Err(ReconcileError::Change(ChangeLookupError::NotLoaded { slug, findings })) => {
            assert_eq!(slug, "shape");
            assert_ne!(findings.len(), 0);
        }
        other => panic!("expected a record that did not load, got {other:?}"),
    }
    let message = reconcile_err(&broken, "shape")
        .expect_err("refused")
        .to_string();
    assert!(message.contains("did not load"), "{message}");
    let message = reconcile_err(&base, "nope")
        .expect_err("refused")
        .to_string();
    assert!(message.contains("the open changes are shape"), "{message}");
}

#[test]
fn a_listed_capability_that_is_not_loaded_is_an_error_and_not_a_skipped_one() {
    let base = base();
    // Missing altogether.
    let missing = without(&base, OMEGA);
    let error = reconcile_err(&missing, "shape").expect_err("refused");
    match &error {
        ReconcileError::CapabilityNotLoaded {
            change,
            capability,
            findings,
        } => {
            assert_eq!(
                (change.as_str(), capability.as_str()),
                ("shape", "cap:omega")
            );
            assert_eq!(findings.len(), 0);
        }
        other => panic!("expected an unloaded capability, got {other:?}"),
    }
    assert!(
        error.to_string().contains("cap:omega, which is not loaded"),
        "{error}"
    );
    // Present and broken: the findings of its document say why.
    let broken = replaced(&base, OMEGA, "state: implementing", "state: finished");
    match reconcile_err(&broken, "shape") {
        Err(ReconcileError::CapabilityNotLoaded {
            capability,
            findings,
            ..
        }) => {
            assert_eq!(capability, "cap:omega");
            assert!(findings.iter().all(|f| f.file == OMEGA) && !findings.is_empty());
        }
        other => panic!("expected an unloaded capability, got {other:?}"),
    }
    assert!(reconcile_err(&base, "shape").is_ok());
}

#[test]
fn a_delivered_capability_that_breaks_another_rule_too_is_not_read_as_unbound() {
    let delivered = repo(&[
        (OMEGA, capability("omega", "delivered", BOUND, &[])),
        (SHAPE, base()[SHAPE].clone()),
    ]);
    // Only the empty list: the binding is owed.
    let unbound = replaced(
        &delivered,
        OMEGA,
        "  tests: [demo::test_crate::it::case]",
        "  tests: []",
    );
    assert_eq!(
        reconcile(&unbound)
            .of_kind(ObligationKind::BindingMissing)
            .count(),
        1
    );
    // The empty list and a missing section: the document is broken in a way
    // reconcile cannot read past, and the change is refused.
    let mut broken = unbound.clone();
    let text = broken.get_mut(OMEGA).expect("the fixture capability");
    let start = text.find("## Usage\n").expect("the authored Usage section");
    let end = start
        + text[start..]
            .find("## Examples\n")
            .expect("the next section");
    text.replace_range(start..end, "");
    match reconcile_err(&broken, "shape") {
        Err(ReconcileError::CapabilityNotLoaded { findings, .. }) => {
            let kinds: BTreeSet<&str> = findings.iter().map(|f| f.kind).collect();
            assert!(
                kinds.contains("okf-binding-required") && kinds.contains("okf-section-missing")
            );
        }
        other => panic!("expected an unloaded capability, got {other:?}"),
    }
    // Both lists empty are two obligations.
    let both = replaced(&unbound, OMEGA, "  code: [demo::real]", "  code: []");
    let owed: Vec<String> = reconcile(&both)
        .of_kind(ObligationKind::BindingMissing)
        .map(|o| o.evidence.to_string())
        .collect();
    assert_eq!(owed.len(), 2, "{owed:#?}");
}

// --- nothing is written ------------------------------------------------------

#[test]
fn reconcile_writes_nothing() {
    let scratch = Scratch::new("reconcile-read-only");
    let root = scratch.path();
    write_repo(root, &untidy());
    let before = snapshot(root);
    let list = on_disk(root, |plan| plan.reconcile("shape").expect("reconciles"));
    assert!(!list.is_empty());
    assert_eq!(before, snapshot(root));
}

// --- hostile text ------------------------------------------------------------

/// Reconcile `text` in the Contract of an implementing `omega`.
fn contract_with(text: &str) -> Result<Reconciled, ReconcileError> {
    let repo = replaced(
        &base(),
        OMEGA,
        "Text for Contract of omega.\n",
        &format!("Text for Contract of omega.\n{text}"),
    );
    reconcile_err(&repo, "shape")
}

#[test]
fn a_very_long_line_is_one_obligation() {
    let line = "will ".repeat(180_000);
    let list = contract_with(&format!("\n{line}\n")).expect("reconciles");
    assert_eq!(list.len(), 1, "one phrase on one line is one obligation");
}

#[test]
fn a_hundred_thousand_lines_are_a_hundred_thousand_obligations_in_order() {
    let text = "will\n".repeat(100_000);
    let list = contract_with(&format!("\n{text}")).expect("reconciles");
    assert_eq!(list.len(), 100_000);
    let lines: Vec<usize> = list
        .obligations()
        .iter()
        .map(|o| o.location.line.expect("a line"))
        .collect();
    assert!(lines.windows(2).all(|pair| pair[0] < pair[1]));
}

/// Reconcile `text` at the end of the last section of an implementing
/// `omega`, after its generated blocks.
fn last_section_with(text: &str) -> Result<Reconciled, ReconcileError> {
    let mut repo = base();
    let omega = repo.get_mut(OMEGA).expect("omega");
    omega.push_str(text);
    reconcile_err(&repo, "shape")
}

#[test]
fn an_unterminated_fence_hides_the_rest_of_the_section() {
    let list = last_section_with("\nStable text.\n\n```text\nwill is hidden\nnot yet either\n")
        .expect("reconciles");
    assert!(list.is_empty(), "{list}");
    // The neighbour: the same text with the fence closed, and wording after it.
    let list =
        last_section_with("\nStable text.\n\n```text\nwill is hidden\n```\n\nIt will start.\n")
            .expect("reconciles");
    assert_eq!(list.len(), 1);
}

#[test]
fn an_unterminated_fence_in_an_earlier_section_hides_the_headings_after_it() {
    // The sections after the fence are not headings, so the document does
    // not load, and the change that lists it is refused and not misread.
    let result = contract_with("\nStable text.\n\n```text\nwill is hidden\n");
    match result {
        Err(ReconcileError::CapabilityNotLoaded { findings, .. }) => {
            assert!(findings.iter().any(|f| f.kind == "okf-section-missing"));
        }
        other => panic!("expected an unloaded capability, got {other:?}"),
    }
    // So is text the reader refuses: a link destination over its bound.
    let result = contract_with(&format!("\n[x]({})\n", "a".repeat(5000)));
    assert!(matches!(
        result,
        Err(ReconcileError::CapabilityNotLoaded { .. })
    ));
}

#[test]
fn text_that_is_read_as_wording_at_a_hostile_size_owes_nothing() {
    // No space at all, link openers that never close, and a wrapped phrase
    // that never completes.
    for text in [
        "will".repeat(200_000),
        "](".repeat(200_000),
        "is\n".repeat(150_000),
        "a [x](#y) b ](#z)\n".repeat(30_000),
    ] {
        let result = contract_with(&format!("\n{text}\n"));
        assert!(result.expect("reconciles").is_empty());
    }
}

#[test]
fn a_capability_over_the_size_bound_is_not_loaded_and_refused() {
    let text = "x".repeat(2 * 1024 * 1024);
    let documents = [
        (
            OMEGA,
            capability(
                "omega",
                "implementing",
                BOUND,
                &[("Contract", text.as_str())],
            ),
        ),
        (SHAPE, base()[SHAPE].clone()),
    ];
    let mut repo = Repo::new();
    for (path, body) in &documents {
        repo.insert((*path).to_string(), body.clone());
    }
    match reconcile_err(&repo, "shape") {
        Err(ReconcileError::CapabilityNotLoaded { findings, .. }) => {
            assert_ne!(findings.len(), 0);
        }
        other => panic!("expected an unloaded capability, got {other:?}"),
    }
}

// --- near misses --------------------------------------------------------------

#[test]
fn wording_that_only_looks_like_a_phrase_owes_nothing() {
    let text = "
Goodwill is not wording, and William wrote it. A willing reader is fine.
A `will` in a code span is named, not said, and <!-- will in a comment --> too.
The PROPOSED marker is the name of a block.
See [the page](proposed-will.md) for more.

```text
It will be fenced.
```

<!-- a comment
that will span lines -->
";
    let result = contract_with(text);
    let list = result.expect("reconciles");
    assert!(list.is_empty(), "{list}");
    // The neighbour: the same lines with wording in them are read.
    let result = contract_with("\nIt will be fenced no longer.\n");
    assert_eq!(result.expect("reconciles").len(), 1);
}

#[test]
fn text_of_a_proposed_block_is_not_read_for_wording() {
    let result =
        contract_with("\n> [!PROPOSED change:shape]\n> The parser will return an error.\n");
    let list = result.expect("reconciles");
    assert_eq!(list.len(), 1);
    assert_eq!(
        list.obligations()[0].kind,
        ObligationKind::UnresolvedProposal
    );
}

#[test]
fn wording_is_read_in_every_section_of_an_implementing_capability() {
    for section in ["Purpose", "Usage", "Examples", "Constraints", "Explanation"] {
        let repo = repo(&[
            (
                OMEGA,
                capability(
                    "omega",
                    "implementing",
                    BOUND,
                    &[(section, "\nStreaming is planned.\n")],
                ),
            ),
            (SHAPE, base()[SHAPE].clone()),
        ]);
        let lines = lines(&repo);
        assert_eq!(lines.len(), 1, "{section}: {lines:?}");
        assert!(
            lines[0].starts_with(&format!(
                "cap:omega  {}  tense-mismatch  line ",
                section.to_lowercase()
            )),
            "{lines:?}"
        );
    }
}

#[test]
fn needed_is_a_typed_value_whose_text_starts_with_the_contract_string() {
    let list = reconcile(&untidy());
    for obligation in list.obligations() {
        let text = obligation.evidence.to_string();
        assert!(
            text.starts_with(obligation.evidence.contract_text()),
            "{text}"
        );
        assert!(!matches!(
            obligation.evidence,
            Needed::SectionDiff | Needed::PassingRun { .. }
        ));
    }
}
