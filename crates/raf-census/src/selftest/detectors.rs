// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A positive and a negative case for every registered built-in detector.

use raf_rdf::TermValue as Term;

use super::{Row, SelfTestCase, check, clean, copies, finding, fixtures, job, ledger, text};
use purrdf_jsonschema::ecma::MatchLimits;

const HOME: &str = "/// Home.\npub fn home() {}";

/// How a detector is selected: by a job's `rule:` fingerprint, or required on
/// the Gate without a job.
#[derive(Clone, Copy)]
pub(super) enum Selection {
    Job,
    Required,
}

/// One fixture workspace: the home source, the consumer source and the
/// extra job rows the detector needs.
pub(super) struct Probe {
    pub(super) home: String,
    pub(super) consumer: &'static str,
    pub(super) rows: Vec<Row>,
}

impl Probe {
    fn consumer(consumer: &'static str) -> Self {
        Self {
            home: HOME.into(),
            consumer,
            rows: Vec::new(),
        }
    }
}

/// Every probe of one detector, by expected outcome.
pub(super) struct Family {
    pub(super) selection: Selection,
    pub(super) positive: Vec<Probe>,
    pub(super) negative: Vec<Probe>,
}

pub(super) fn cases(limits: MatchLimits, out: &mut Vec<SelfTestCase>) {
    for id in crate::detectors::ids() {
        match family(id) {
            Some(family) => {
                out.push(super::case(
                    format!("detector/{id}/positive"),
                    holds(limits, id, &family, true),
                ));
                out.push(super::case(
                    format!("detector/{id}/negative"),
                    holds(limits, id, &family, false),
                ));
            }
            None => out.push(super::case(
                format!("detector/{id}"),
                Err("the detector has no self-test fixtures".into()),
            )),
        }
    }
}

/// Whether every probe of one side produces its expected outcome.
pub(super) fn holds(
    limits: MatchLimits,
    id: &str,
    family: &Family,
    positive: bool,
) -> Result<(), String> {
    let (side, probes) = if positive {
        ("positive", &family.positive)
    } else {
        ("negative", &family.negative)
    };
    if probes.is_empty() {
        return Err(format!("no {side} fixture"));
    }
    for (index, probe) in probes.iter().enumerate() {
        evaluate(limits, id, family.selection, probe, positive)
            .map_err(|reason| format!("{side} fixture {index}: {reason}"))?;
    }
    Ok(())
}

fn evaluate(
    limits: MatchLimits,
    id: &str,
    selection: Selection,
    probe: &Probe,
    positive: bool,
) -> Result<(), String> {
    let (gate, rows, prefix) = match selection {
        Selection::Job => {
            let forbidden = [("#job", "forbiddenFingerprint", text(&format!("rule:{id}")))];
            let mut rows = job(true, forbidden);
            rows.extend(probe.rows.iter().cloned());
            (super::gate(limits), rows, "rule")
        }
        Selection::Required => (
            super::gate(limits).require_rules([id]),
            Vec::new(),
            "required rule",
        ),
    };
    let tree = super::workspace(&probe.home, probe.consumer, ledger(rows)?);
    let report = check(&gate, &tree)?;
    if !positive {
        return clean(&report);
    }
    if matches!(selection, Selection::Job) {
        copies(&report, 1)?;
    }
    finding(&report, &format!("{prefix} {id}:"))
}

/// The fixture family of one registered detector id.
fn family(id: &str) -> Option<Family> {
    let pair = |positive: &'static str, negative: &'static str| Family {
        selection: Selection::Job,
        positive: vec![Probe::consumer(positive)],
        negative: vec![Probe::consumer(negative)],
    };
    if let Some((_, positive, negative)) = fixtures::LAYOUT.iter().find(|row| row.0 == id) {
        return Some(pair(positive, negative));
    }
    if let Some((_, positive, negative, _)) = fixtures::FACTS
        .iter()
        .chain(&fixtures::HEX)
        .find(|row| row.0 == id)
    {
        return Some(pair(positive, negative));
    }
    if let Some((_, home, positive, negative)) = fixtures::CONTEXTUAL.iter().find(|row| row.0 == id)
    {
        let probes = |consumers: &[&'static str]| {
            consumers
                .iter()
                .map(|consumer| Probe {
                    home: (*home).into(),
                    consumer,
                    rows: Vec::new(),
                })
                .collect()
        };
        return Some(Family {
            selection: Selection::Job,
            positive: probes(positive),
            negative: probes(negative),
        });
    }
    if fixtures::RDF.iter().any(|row| row.0 == id) {
        let (positive, negative) = fixtures::RDF
            .iter()
            .chain(&fixtures::RDF_NESTED)
            .filter(|row| row.0 == id)
            .map(|(_, positive, negative)| (Probe::consumer(positive), Probe::consumer(negative)))
            .unzip();
        return Some(Family {
            selection: Selection::Required,
            positive,
            negative,
        });
    }
    if id == fixtures::HASHER_ID {
        return Some(Family {
            selection: Selection::Job,
            positive: fixtures::HASHER_POSITIVE
                .into_iter()
                .map(Probe::consumer)
                .collect(),
            negative: fixtures::HASHER_NEGATIVE
                .into_iter()
                .map(Probe::consumer)
                .collect(),
        });
    }
    (id == fixtures::SOLE_NAME_ID).then(sole_name)
}

/// `sole-name` flags a second item with the forbidden name in the home
/// package itself; its negative names that item as a declared entry point.
fn sole_name() -> Family {
    let name: Row = ("#job", "forbiddenName", text("^copy$"));
    let positive = fixtures::SOLE_NAME_POSITIVE
        .iter()
        .map(|copy| Probe {
            home: format!("{HOME}\n{copy}"),
            consumer: "",
            rows: vec![name.clone()],
        })
        .collect();
    let negative = fixtures::SOLE_NAME_NEGATIVE
        .iter()
        .map(|(home, entry_point)| Probe {
            home: (*home).into(),
            consumer: "",
            rows: vec![
                name.clone(),
                ("#job", "entryPoint", Term::simple_literal(*entry_point)),
            ],
        })
        .collect();
    Family {
        selection: Selection::Job,
        positive,
        negative,
    }
}
