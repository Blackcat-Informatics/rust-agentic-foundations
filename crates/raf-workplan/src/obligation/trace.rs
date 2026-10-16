// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The trace source: what the claims and the evidence records owe, from the
//! loaded documents and the source index alone.
//!
//! * `claim-unsupported`: for each claim marker of a listed capability and each
//!   evidence id it names, the record that loaded under the id; one that is not
//!   there, fails or holds a value over its limit leaves the claim unsupported,
//!   once for each of these that holds.
//! * `evidence-stale`: for each evidence record that a listed capability's `for`
//!   edge or claim reaches, each input whose digest now differs from the digest
//!   it keeps, or that resolves to nothing now. Nothing reads a revision or Git.
//! * `assessment-missing`, `assessment-stale` and `assessment-rejected`: for
//!   each authored section in `affects`, the assessment record under the id of
//!   the section ([`record_slug`](crate::assessment::record_slug)): none, one
//!   whose inputs moved (once for each input, naming it), or one that is
//!   current and rejected. A section that holds only generated blocks would owe
//!   nothing, and the profile refuses one, so a loaded capability has none.

use std::collections::{BTreeMap, BTreeSet};

use raf_okf::{
    AssessmentVerdict, Capability, CapabilitySection, DocType, EvidenceRecord, EvidenceVerdict,
    InputKind, Measure,
};

use crate::assessment::Judged;
use crate::digest::{Digest, SymbolDigestError};
use crate::graph::SectionId;
use crate::{Loaded, Workplan};

use super::{
    ClaimProblem, InputMoved, Needed, Obligation, ObligationKind, OverLimit, Part, Place, Subject,
};

/// Each value of `values` over its limit in `limits`, in the order of the
/// limits, compared exactly on the decimal texts.
pub(crate) fn over_limits<'a>(
    values: &'a [Measure],
    limits: &'a [Measure],
) -> impl Iterator<Item = OverLimit> + 'a {
    limits.iter().filter_map(|limit| {
        let value = values.iter().find(|value| value.name == limit.name)?;
        value.value.exceeds(&limit.value).then(|| OverLimit {
            name: limit.name.clone(),
            value: value.value.text().to_string(),
            limit: limit.value.text().to_string(),
        })
    })
}

/// The part of a capability a claim in `section` is about.
pub(crate) fn claim_part(section: Option<&str>) -> Part {
    section
        .and_then(CapabilitySection::from_slug)
        .map_or(Part::Document, Part::Section)
}

/// The trace obligations of the listed capabilities.
pub(super) fn raise(
    plan: &Workplan<'_>,
    listed: &[&Loaded<Capability>],
    found: &mut Vec<Obligation>,
) {
    let records: BTreeMap<&str, &Loaded<EvidenceRecord>> = plan
        .evidence()
        .iter()
        .map(|record| (record.value().header.id.as_str(), record))
        .collect();
    let prefix = DocType::Evidence.id_prefix();
    // The digest of each symbol now, taken once.
    let mut now: BTreeMap<String, Result<Digest, SymbolDigestError>> = BTreeMap::new();
    for cap in listed {
        let id = cap.value().header.id.as_str();
        let subject = || Subject::Capability(id.to_string());
        let mut reached: BTreeSet<&str> = plan.graph().evidence_of(id).into_iter().collect();
        for claim in cap.body().claims() {
            for slug in claim.evidence() {
                let evidence = format!("{prefix}{slug}");
                let mut problems = Vec::new();
                match records.get(evidence.as_str()) {
                    None => problems.push(ClaimProblem::Missing),
                    Some(record) => {
                        reached.insert(record.value().header.id.as_str());
                        let value = record.value();
                        if value.verdict == EvidenceVerdict::Fail {
                            problems.push(ClaimProblem::Fails);
                        }
                        problems.extend(
                            over_limits(&value.values, &value.limits).map(ClaimProblem::OverLimit),
                        );
                    }
                }
                for problem in problems {
                    found.push(Obligation {
                        kind: ObligationKind::ClaimUnsupported,
                        subject: subject(),
                        section: claim_part(claim.section()),
                        location: Place {
                            path: cap.path().to_string(),
                            line: Some(claim.line()),
                        },
                        evidence: Needed::SupportingEvidence {
                            evidence: evidence.clone(),
                            problem,
                        },
                    });
                }
            }
        }
        for evidence in reached {
            let Some(record) = records.get(evidence) else {
                continue;
            };
            for input in &record.value().inputs {
                // The profile gives an evidence record `symbol` inputs only.
                if input.kind != InputKind::Symbol {
                    continue;
                }
                let digest = now
                    .entry(input.name.clone())
                    .or_insert_with(|| Digest::symbol(plan.source(), &input.name));
                let moved = match digest {
                    Ok(digest) if digest.to_string() == input.digest => continue,
                    Ok(_) => InputMoved::Moved,
                    Err(error) => InputMoved::Unresolved(match error {
                        SymbolDigestError::Resolve { error, .. } => error.to_string(),
                        other => other.to_string(),
                    }),
                };
                found.push(Obligation {
                    kind: ObligationKind::EvidenceStale,
                    subject: subject(),
                    section: Part::Document,
                    location: Place {
                        path: record.path().to_string(),
                        line: None,
                    },
                    evidence: Needed::RecordedAgain {
                        evidence: evidence.to_string(),
                        input: input.name.clone(),
                        moved,
                    },
                });
            }
        }
    }
}

/// The assessment obligations of the sections in `affects`.
///
/// Each section of a loaded capability is judged. The listed capabilities are
/// loaded, or reconcile has stopped with an error, except a delivered one that
/// only binds nothing in a list it must bind: it owes that binding and takes
/// part in nothing else until it loads, as for every other kind.
pub(super) fn assessments(
    plan: &Workplan<'_>,
    sections: &[SectionId],
    found: &mut Vec<Obligation>,
) {
    for id in sections {
        let Some(cap) = plan
            .capabilities()
            .iter()
            .find(|cap| cap.value().header.id == id.capability())
        else {
            continue;
        };
        let owed = |kind, path: &str, evidence| Obligation {
            kind,
            subject: Subject::Capability(id.capability().to_string()),
            section: Part::Section(id.section()),
            location: Place {
                path: path.to_string(),
                line: None,
            },
            evidence,
        };
        match plan.judge_section(cap, id.section()) {
            Judged::Missing { id: assessment } => found.push(owed(
                ObligationKind::AssessmentMissing,
                cap.path(),
                Needed::Assessed {
                    subject: id.to_string(),
                    assessment,
                },
            )),
            Judged::Stale(record, moved) => {
                for input in moved {
                    found.push(owed(
                        ObligationKind::AssessmentStale,
                        record.path(),
                        Needed::AssessedAgain {
                            assessment: record.value().header.id.clone(),
                            input,
                        },
                    ));
                }
            }
            Judged::Current(record) if record.value().verdict == AssessmentVerdict::Rejected => {
                found.push(owed(
                    ObligationKind::AssessmentRejected,
                    record.path(),
                    Needed::AcceptedAssessment {
                        assessment: record.value().header.id.clone(),
                        by: record.value().by,
                    },
                ));
            }
            Judged::Exempt | Judged::Current(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use raf_okf::{Measure, Number};

    use super::over_limits;

    fn measure(name: &str, value: &str) -> Measure {
        Measure::new(name, Number::parse(value).expect("a number"))
    }

    #[test]
    fn a_value_over_its_limit_is_found_by_its_text_and_not_its_nearest_float() {
        let one_e400 = format!("1{}", "0".repeat(400));
        let two_e400 = format!("2{}", "0".repeat(400));
        for (over, limit, under) in [
            ("9007199254740993", "9007199254740992", "9007199254740991"),
            ("0.30000000000000001", "0.3", "0.29999999999999999"),
            ("1.0000000000000000001", "1", "0.9999999999999999999"),
            (two_e400.as_str(), one_e400.as_str(), &one_e400[..400]),
        ] {
            let limits = [measure("peak", limit)];
            let found: Vec<(String, String)> = over_limits(&[measure("peak", over)], &limits)
                .map(|over| (over.value, over.limit))
                .collect();
            assert_eq!(found, [(over.to_string(), limit.to_string())]);
            for neighbour in [limit, under] {
                assert_eq!(
                    over_limits(&[measure("peak", neighbour)], &limits).count(),
                    0,
                    "{neighbour}"
                );
            }
        }
    }
}
