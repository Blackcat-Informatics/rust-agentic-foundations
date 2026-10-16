// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The parts of the obligation module that need no repository: the kinds, the
//! suppression rule, the sort, the evidence strings and the tense scan.

use std::collections::BTreeSet;
use std::path::Path;

use raf_okf::{Body, CapabilitySection, ContractEffect};

use super::tense::{self, PHRASES};
use super::{
    BindingWhy, ClaimProblem, EvidenceChange, Found, InputMoved, Needed, Obligation,
    ObligationKind, OutcomeProblem, OverLimit, Part, Place, Reconciled, RunProblem, Subject,
    SyncProblem, suppressed,
};
use crate::BindingKind;

// --- the kinds ---------------------------------------------------------------

#[test]
fn all_lists_every_kind_once_in_order_with_unique_kebab_names() {
    let mut names = BTreeSet::new();
    for (index, kind) in ObligationKind::ALL.iter().enumerate() {
        assert_eq!(kind.rank(), index, "{kind} is out of place in ALL");
        assert_eq!(kind.to_string(), kind.name());
        assert!(
            kind.name()
                .split('-')
                .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase())),
            "{kind}"
        );
        assert!(names.insert(kind.name()), "{kind} is named twice");
    }
    assert!(ObligationKind::ALL.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn the_kinds_are_the_ten_of_the_contract() {
    let names: Vec<&str> = ObligationKind::ALL.iter().map(|kind| kind.name()).collect();
    assert_eq!(
        names,
        [
            "unresolved-proposal",
            "item-without-outcome",
            "binding-missing",
            "section-untouched",
            "example-failing",
            "generated-stale",
            "tense-mismatch",
            "claim-unsupported",
            "evidence-stale",
            "evidence-changed",
        ]
    );
}

#[test]
fn only_the_three_kinds_that_start_a_process_are_not_static() {
    let dynamic: Vec<ObligationKind> = ObligationKind::ALL
        .into_iter()
        .filter(|kind| !kind.is_static())
        .collect();
    assert_eq!(
        dynamic,
        [
            ObligationKind::SectionUntouched,
            ObligationKind::ExampleFailing,
            ObligationKind::EvidenceChanged,
        ]
    );
}

// --- contract_effect: none ---------------------------------------------------

#[test]
fn contract_effect_none_suppresses_section_untouched_for_three_sections_and_nothing_else() {
    let sections = CapabilitySection::ALL
        .into_iter()
        .map(Part::Section)
        .chain([Part::Document, Part::Item(1)]);
    for section in sections {
        for kind in ObligationKind::ALL {
            let expected = kind == ObligationKind::SectionUntouched
                && matches!(
                    section,
                    Part::Section(
                        CapabilitySection::Contract
                            | CapabilitySection::Usage
                            | CapabilitySection::Examples
                    )
                );
            assert_eq!(
                suppressed(kind, section, ContractEffect::None),
                expected,
                "{kind} in {section}"
            );
            // A change that changes the contract is never lighter.
            assert!(
                !suppressed(kind, section, ContractEffect::Changes),
                "{kind} in {section}"
            );
        }
    }
}

// --- order and rendering -----------------------------------------------------

fn obligation(
    kind: ObligationKind,
    subject: Subject,
    section: Part,
    line: Option<usize>,
    evidence: Needed,
) -> Obligation {
    Obligation {
        kind,
        subject,
        section,
        location: Place {
            path: "docs/capabilities/x.md".to_string(),
            line,
        },
        evidence,
    }
}

fn cap(id: &str) -> Subject {
    Subject::Capability(id.to_string())
}

fn tense_at(id: &str, section: CapabilitySection, line: usize, phrase: &'static str) -> Obligation {
    obligation(
        ObligationKind::TenseMismatch,
        cap(id),
        Part::Section(section),
        Some(line),
        Needed::SupportedBehaviour(Found::Phrase(phrase)),
    )
}

#[test]
fn the_list_is_sorted_by_subject_part_kind_line_then_evidence_and_is_without_repeats() {
    use CapabilitySection::{Contract, Purpose, Verification};
    let binding = |why: &str| {
        obligation(
            ObligationKind::BindingMissing,
            cap("cap:a"),
            Part::Section(Verification),
            None,
            Needed::ResolvedBinding {
                list: BindingKind::Code,
                path: Some("demo::x".to_string()),
                why: BindingWhy::DoesNotResolve(why.to_string()),
            },
        )
    };
    let shuffled = vec![
        tense_at("cap:b", Purpose, 3, "will"),
        binding("z reason"),
        tense_at("cap:a", Contract, 9, "proposed"),
        tense_at("cap:a", Contract, 4, "will"),
        tense_at("cap:a", Contract, 4, "not yet"),
        tense_at("cap:a", Contract, 4, "will"),
        binding("a reason"),
        obligation(
            ObligationKind::UnresolvedProposal,
            cap("cap:a"),
            Part::Section(Contract),
            Some(20),
            Needed::ProposalResolved,
        ),
        obligation(
            ObligationKind::ItemWithoutOutcome,
            Subject::Change("change:g".to_string()),
            Part::Item(2),
            None,
            Needed::Outcome(OutcomeProblem::Missing),
        ),
        obligation(
            ObligationKind::GeneratedStale,
            Subject::Index,
            Part::Document,
            Some(5),
            Needed::SyncRun(SyncProblem::Stale("catalogue".to_string())),
        ),
    ];
    let mut reversed = shuffled.clone();
    reversed.reverse();
    let list = Reconciled::new("g".to_string(), shuffled);
    assert_eq!(list, Reconciled::new("g".to_string(), reversed));

    let lines: Vec<String> = list.to_string().lines().map(str::to_string).collect();
    assert_eq!(lines.len(), list.len());
    assert_eq!(list.len(), 9, "the repeated hit is one obligation");
    assert_eq!(
        lines,
        [
            // cap:a: contract before verification; within the contract the
            // kinds in table order, then lines, then evidence text.
            "cap:a  contract  unresolved-proposal  line 20: the block is replaced by plain contract text, or removed because the claim was dropped",
            "cap:a  contract  tense-mismatch  line 4: text rewritten as the supported behaviour (the text says \"not yet\")",
            "cap:a  contract  tense-mismatch  line 4: text rewritten as the supported behaviour (the text says \"will\")",
            "cap:a  contract  tense-mismatch  line 9: text rewritten as the supported behaviour (the text says \"proposed\")",
            "cap:a  verification  binding-missing  a binding that resolves through the source index (bindings.code `demo::x` does not resolve: a reason)",
            "cap:a  verification  binding-missing  a binding that resolves through the source index (bindings.code `demo::x` does not resolve: z reason)",
            "cap:b  purpose  tense-mismatch  line 3: text rewritten as the supported behaviour (the text says \"will\")",
            "change:g  item 2  item-without-outcome  an outcome of implemented, dropped or moved:change:<slug>",
            "docs/capabilities/index.md  document  generated-stale  line 5: a docs sync run (the `catalogue` block is stale)",
        ]
    );
    assert_eq!(list.of_kind(ObligationKind::TenseMismatch).count(), 4);
    assert!(!list.is_empty());
    assert_eq!(list.change(), "g");
}

#[test]
fn a_part_is_ordered_document_then_sections_in_document_order_then_items() {
    let order = [
        Part::Document,
        Part::Section(CapabilitySection::Purpose),
        Part::Section(CapabilitySection::Contract),
        Part::Section(CapabilitySection::Verification),
        Part::Item(1),
        Part::Item(2),
        Part::Item(10),
    ];
    for (index, left) in order.iter().enumerate() {
        for (other, right) in order.iter().enumerate() {
            assert_eq!(left.cmp(right), index.cmp(&other), "{left} against {right}");
        }
    }
}

#[test]
fn an_empty_list_prints_nothing() {
    let list = Reconciled::new("g".to_string(), Vec::new());
    assert!(list.is_empty());
    assert_eq!(list.to_string(), "");
}

// --- the evidence strings ----------------------------------------------------

/// The evidence of each kind, taken from the table of the reconcile contract
/// in the capability document: backticks removed and the first letter in
/// lower case. A row of a table inside a PROPOSED block (a line that starts
/// with `>`) is a row too, so the kinds a change adds are pinned in the same
/// commit as the code, while their text is still a proposal.
fn contract_evidence() -> Vec<(String, String)> {
    let root = std::env::var_os("RAF_GATE_ROOT").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        std::path::PathBuf::from,
    );
    let path = root.join("docs/capabilities/workplan-reconcile.md");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let mut rows = Vec::new();
    for line in text.lines() {
        let line = line.strip_prefix('>').map_or(line, str::trim_start);
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        // `| `kind` | Raised when | Evidence |` splits into five parts.
        let [_, kind, _, evidence, _] = cells.as_slice() else {
            continue;
        };
        let Some(kind) = kind.strip_prefix('`').and_then(|k| k.strip_suffix('`')) else {
            continue;
        };
        if ObligationKind::ALL.iter().any(|k| k.name() == kind) {
            let plain = evidence.replace('`', "");
            let mut chars = plain.chars();
            let lowered = chars
                .next()
                .map_or_default(|first| first.to_lowercase().chain(chars).collect::<String>());
            rows.push((kind.to_string(), lowered));
        }
    }
    rows
}

/// One evidence value for each kind.
fn evidence_of(kind: ObligationKind) -> Needed {
    match kind {
        ObligationKind::UnresolvedProposal => Needed::ProposalResolved,
        ObligationKind::ItemWithoutOutcome => Needed::Outcome(OutcomeProblem::Missing),
        ObligationKind::BindingMissing => Needed::ResolvedBinding {
            list: BindingKind::Test,
            path: None,
            why: BindingWhy::Empty,
        },
        ObligationKind::SectionUntouched => Needed::SectionDiff,
        ObligationKind::ExampleFailing => Needed::PassingRun {
            binding: "demo::example_crate::show".to_string(),
            problem: RunProblem::Fails { code: Some(101) },
        },
        ObligationKind::GeneratedStale => Needed::SyncRun(SyncProblem::IndexMissing),
        ObligationKind::TenseMismatch => Needed::SupportedBehaviour(Found::Phrase("will")),
        ObligationKind::ClaimUnsupported => Needed::SupportingEvidence {
            evidence: "evidence:bound".to_string(),
            problem: ClaimProblem::Fails,
        },
        ObligationKind::EvidenceStale => Needed::RecordedAgain {
            evidence: "evidence:bound".to_string(),
            input: "demo::run".to_string(),
            moved: InputMoved::Moved,
        },
        ObligationKind::EvidenceChanged => Needed::ReviewedClaim {
            evidence: "evidence:bound".to_string(),
            change: EvidenceChange::Removed,
        },
    }
}

#[test]
fn the_evidence_strings_are_the_evidence_column_of_the_contract() {
    let rows = contract_evidence();
    let kinds: Vec<&str> = rows.iter().map(|(kind, _)| kind.as_str()).collect();
    let all: Vec<&str> = ObligationKind::ALL.iter().map(|k| k.name()).collect();
    assert_eq!(kinds, all, "the contract table lists the kinds in order");
    for (kind, text) in rows {
        let kind = ObligationKind::ALL
            .into_iter()
            .find(|k| k.name() == kind)
            .expect("a kind");
        assert_eq!(evidence_of(kind).contract_text(), text, "{kind}");
    }
}

#[test]
fn the_evidence_names_the_thing_it_is_about_in_parentheses() {
    let moved = Needed::Outcome(OutcomeProblem::MovedToMissing("ghost".to_string()));
    assert_eq!(
        moved.to_string(),
        "an outcome of implemented, dropped or moved:change:<slug> (`moved:change:ghost` names no change record)"
    );
    let wrong = Needed::ResolvedBinding {
        list: BindingKind::Test,
        path: Some("demo::real".to_string()),
        why: BindingWhy::WrongRole("names an item that is not a function marked `#[test]`"),
    };
    assert_eq!(
        wrong.to_string(),
        "a binding that resolves through the source index (bindings.tests `demo::real` names an item that is not a function marked `#[test]`)"
    );
    assert_eq!(
        Needed::SupportedBehaviour(Found::ProposedBlock("grow".to_string())).to_string(),
        "text rewritten as the supported behaviour (a PROPOSED block of change:grow stands in a delivered capability)"
    );
    assert_eq!(
        Needed::SyncRun(SyncProblem::IndexMissing).to_string(),
        "a docs sync run (docs/capabilities/index.md does not exist)"
    );
    // Evidence with nothing to name has no parentheses.
    assert_eq!(
        Needed::SectionDiff.to_string(),
        Needed::SectionDiff.contract_text()
    );
}

#[test]
fn the_trace_evidence_names_the_record_and_what_is_wrong_with_it() {
    let over = || OverLimit {
        name: "peak".to_string(),
        value: "80".to_string(),
        limit: "64".to_string(),
    };
    let supporting = |problem| {
        Needed::SupportingEvidence {
            evidence: "evidence:bound".to_string(),
            problem,
        }
        .to_string()
    };
    let lead = Needed::SupportingEvidence {
        evidence: String::new(),
        problem: ClaimProblem::Missing,
    }
    .contract_text();
    for (problem, detail) in [
        (
            ClaimProblem::Missing,
            "`evidence:bound` is not a loaded evidence record",
        ),
        (ClaimProblem::Fails, "`evidence:bound` has the verdict fail"),
        (
            ClaimProblem::OverLimit(over()),
            "`evidence:bound`: `peak` is 80, over its limit 64",
        ),
    ] {
        assert_eq!(supporting(problem), format!("{lead} ({detail})"));
    }
    let again = |moved| {
        Needed::RecordedAgain {
            evidence: "evidence:bound".to_string(),
            input: "demo::run".to_string(),
            moved,
        }
        .to_string()
    };
    assert!(
        again(InputMoved::Moved)
            .ends_with("(`evidence:bound` was made against `demo::run`, which has changed since)")
    );
    assert!(
        again(InputMoved::Unresolved("gone".to_string()))
            .ends_with("which does not resolve now: gone)")
    );
    let reviewed = |change| {
        Needed::ReviewedClaim {
            evidence: "evidence:bound".to_string(),
            change,
        }
        .to_string()
    };
    for (change, detail) in [
        (
            EvidenceChange::Failed,
            "`evidence:bound` passed at the merge base and fails now",
        ),
        (
            EvidenceChange::OverLimit(over()),
            "`evidence:bound`: `peak` is 80, over its limit 64",
        ),
        (
            EvidenceChange::LimitRemoved("peak".to_string()),
            "`evidence:bound` no longer has the limit `peak` it had at the merge base",
        ),
        (
            EvidenceChange::Removed,
            "`evidence:bound` passed at the merge base and is not a loaded record now",
        ),
    ] {
        assert!(
            reviewed(change).ends_with(&format!("({detail})")),
            "{detail}"
        );
    }
}

// --- tense -------------------------------------------------------------------

/// The phrases found in `text`, one line to a row.
fn found(text: &str) -> Vec<(usize, &'static str)> {
    tense::scan(
        text.lines()
            .enumerate()
            .map(|(index, line)| (index + 1, line, ())),
    )
    .into_iter()
    .map(|(line, phrase, ())| (line, phrase))
    .collect()
}

/// A sentence with each phrase in it, and the phrase it must report.
const POSITIVES: [(&str, &str); 12] = [
    ("The parser will return an error.", "will"),
    ("Will it work?", "will"),
    ("Streaming is planned.", "is planned"),
    ("Both formats are planned for the parser.", "are planned"),
    ("This is the proposed behaviour.", "proposed"),
    ("Proposed: a new flag.", "proposed"),
    (
        "Retry is to be implemented by the runner.",
        "to be implemented",
    ),
    ("It may change in the future.", "in the future"),
    ("The gate is going to refuse it.", "going to"),
    ("Support is not yet complete.", "not yet"),
    ("The text says **will** loudly.", "will"),
    ("A \"will\" in quotes still reads as the future.", "will"),
];

#[test]
fn every_phrase_has_a_sentence_that_finds_it_and_every_hit_is_a_phrase() {
    let mut covered = BTreeSet::new();
    for (sentence, phrase) in POSITIVES {
        let hits = found(sentence);
        assert_eq!(hits, [(1, phrase)], "{sentence}");
        assert!(PHRASES.contains(&phrase));
        covered.insert(phrase);
    }
    let listed: BTreeSet<&str> = PHRASES.into_iter().collect();
    assert_eq!(covered, listed, "a phrase has no sentence that finds it");
    assert_eq!(listed.len(), PHRASES.len(), "a phrase is listed twice");
}

#[test]
fn only_the_marker_name_is_exempt_in_capitals() {
    assert_eq!(found("It WILL work."), [(1, "will")]);
    assert_eq!(
        found("NOT YET and IN THE FUTURE."),
        [(1, "not yet"), (1, "in the future")]
    );
    assert_eq!(
        found("It is GOING TO be IS PLANNED."),
        [(1, "going to"), (1, "is planned")]
    );
    assert_eq!(found("A PROPOSED block."), []);
    assert_eq!(
        found("A Proposed block and a proposed block."),
        [(1, "proposed")]
    );
}

#[test]
fn near_misses_are_not_hits() {
    for sentence in [
        "The goodwill of the maintainers.",
        "William wrote the parser.",
        "A willing reader.",
        "Willow and willows.",
        "Unproposed text and reproposed text.",
        "The notyet token and not-yet-written code.",
        "Going tomorrow, to begin.",
        "A plan is plain.",
        "The planned examples of a designed capability.",
        "The future of the project.",
        "in the futures",
        "to be implementation",
        // Words of a phrase must be separated by one space or one line break.
        "It is  planned with two spaces.",
        "It is, planned after a comma.",
        "It is-planned with a hyphen.",
        // The name of the marker is not wording.
        "A PROPOSED block holds the candidate text.",
        // A link destination is not wording.
        "See [the page](will-proposed.md) for the details.",
        "See [the page](https://example.org/proposed/will) for the details.",
        "",
    ] {
        assert_eq!(found(sentence), [], "{sentence}");
    }
    // The text before a link destination and after it is read.
    assert_eq!(
        found("See [the page](x.md) which will help."),
        [(1, "will")]
    );
    // An unclosed destination leaves the rest of the line out.
    assert_eq!(found("See [the page](x.md will not be read"), []);
}

#[test]
fn a_phrase_continues_over_one_line_break_and_not_over_anything_else() {
    assert_eq!(found("Streaming is\nplanned."), [(1, "is planned")]);
    assert_eq!(found("Streaming is\n  planned."), [(1, "is planned")]);
    assert_eq!(found("in the\nfuture"), [(1, "in the future")]);
    assert_eq!(found("going\nto"), [(1, "going to")]);
    // A blank line ends the paragraph.
    assert_eq!(found("Streaming is\n\nplanned."), []);
    // A line that ends with punctuation or starts with a list marker does not
    // continue the sentence.
    assert_eq!(found("Streaming is,\nplanned."), []);
    assert_eq!(found("Streaming is\n- planned"), []);
    // A line that is not prose between the two (a fence, a comment) leaves a
    // gap in the line numbers.
    let gapped = tense::scan([(1, "is", ()), (3, "planned", ())]);
    assert_eq!(gapped.len(), 0);
    let joined = tense::scan([(1, "is", ()), (2, "planned", ())]);
    assert_eq!(joined, [(1, "is planned", ())]);
}

#[test]
fn several_hits_of_a_phrase_on_one_line_are_one_hit_and_two_lines_are_two() {
    assert_eq!(found("will and will and will"), [(1, "will")]);
    assert_eq!(found("will\nwill"), [(1, "will"), (2, "will")]);
    assert_eq!(
        found("It will, not yet, and is going to be."),
        [(1, "will"), (1, "not yet"), (1, "going to")]
    );
}

/// A capability-shaped body: a section heading, then the text.
fn body(text: &str) -> Body {
    Body::parse(&format!("# Title\n\n## Contract\n\n{text}")).expect("a body")
}

#[test]
fn only_prose_is_read_and_the_hit_names_its_section_and_line() {
    let text = "\
The parser will start.

```text
will in a fence
```

A `will` in a code span and an <!-- inline will --> comment.

<!-- a comment
that will span lines -->

> [!PROPOSED change:grow]
> The parser will change.

Streaming is planned.
";
    let hits = tense::find(&body(text));
    let rows: Vec<(usize, &str, Option<CapabilitySection>)> = hits
        .iter()
        .map(|hit| (hit.line, hit.phrase, hit.section))
        .collect();
    assert_eq!(
        rows,
        [
            (5, "will", Some(CapabilitySection::Contract)),
            (19, "is planned", Some(CapabilitySection::Contract)),
        ]
    );
}

#[test]
fn text_before_the_first_section_has_no_section() {
    let body = Body::parse("# Title\n\nIt will start.\n\n## Contract\n\nPlain.\n").expect("a body");
    let hits = tense::find(&body);
    assert_eq!(hits.len(), 1);
    assert_eq!((hits[0].line, hits[0].section), (3, None));
}

#[test]
fn generated_blocks_are_not_read() {
    let text = "Plain.\n\n<!-- raf:generated:relationships -->\nIt will start.\n<!-- raf:generated:end -->\n";
    assert_eq!(tense::find(&body(text)), []);
}

#[test]
fn an_unterminated_fence_hides_the_rest_and_does_not_panic() {
    let text = "It will start.\n\n```text\nwill is hidden\nand so is not yet\n";
    let hits = tense::find(&body(text));
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].phrase, "will");
}

#[test]
fn hostile_text_is_read_in_linear_time() {
    // One huge line of the phrase, one of no space at all, a line of link
    // openers without a close, and many short lines.
    let long = "will ".repeat(180_000);
    assert_eq!(found(&long), [(1, "will")]);
    let solid = "will".repeat(200_000);
    assert_eq!(found(&solid), []);
    let openers = "](".repeat(200_000);
    assert_eq!(found(&openers), []);
    let many = "will\n".repeat(100_000);
    assert_eq!(found(&many).len(), 100_000);
    let wrapped = "is\n".repeat(100_000);
    assert_eq!(found(&wrapped), []);
    let alternating = "is\nplanned\n".repeat(50_000);
    assert_eq!(found(&alternating).len(), 50_000);
}

#[test]
fn a_failing_run_names_the_binding_and_what_happened_without_a_time_or_output() {
    let binding = || "demo::example_crate::show".to_string();
    let text = |problem: RunProblem| {
        Needed::PassingRun {
            binding: binding(),
            problem,
        }
        .to_string()
    };
    let lead = Needed::PassingRun {
        binding: binding(),
        problem: RunProblem::TimedOut,
    }
    .contract_text();
    for (problem, detail) in [
        (
            RunProblem::Fails { code: Some(101) },
            "`demo::example_crate::show` fails to build or run (exit code 101)",
        ),
        (
            RunProblem::Fails { code: None },
            "`demo::example_crate::show` fails to build or run (ended by a signal)",
        ),
        (
            RunProblem::TimedOut,
            "`demo::example_crate::show` ran past its time limit and was killed",
        ),
        (
            RunProblem::Overflowed,
            "`demo::example_crate::show` wrote more output than the bound and was killed",
        ),
        (
            RunProblem::Unresolved(crate::bound::Refusal::NoDocItem {
                path: "demo".to_string(),
            }),
            "`demo::example_crate::show` proves nothing: `demo` has no item part",
        ),
    ] {
        let shown = text(problem);
        assert!(shown.starts_with(lead), "{shown}");
        assert!(shown.contains(&format!("({detail}")), "{shown}");
    }
}
