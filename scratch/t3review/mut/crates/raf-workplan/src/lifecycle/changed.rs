// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `evidence-changed`: an evidence record that a claim of a listed capability
//! names, which passed at the merge base and has changed since.
//!
//! The record of each id the claims name is read once at the merge base, at
//! `work/evidence/<slug>.md` below the workspace's prefix, with the one
//! `git cat-file blob` row of `section-untouched` and its 1 MiB bound. A record
//! that is not there, or whose verdict there is `fail`, gives nothing to
//! compare: the claim is judged by `claim-unsupported` alone. A record there
//! that does not read as an evidence record is an error. Otherwise each change
//! is owed at each claim that names the record: the verdict went from `pass`
//! to `fail`, a value is now over a limit the record or the record at the
//! merge base states, a limit of the record at the merge base is gone, or no
//! record loads under the id now.

use std::collections::{BTreeMap, BTreeSet};

use raf_okf::{
    Capability, Document, EvidenceRecord, EvidenceVerdict, Measure, Profile, RafDocument,
};

use crate::evidence::record_path;
use crate::exec::Runner;
use crate::git::{MergeBase, Prefix};
use crate::obligation::trace::claim_part;
use crate::obligation::{
    EvidenceChange, Needed, Obligation, ObligationKind, OverLimit, Place, Subject,
};
use crate::{Loaded, Opened};

use super::Impact;
use super::session::{LifecycleError, Side};
use super::untouched;

/// The `evidence-changed` obligations of a change.
///
/// Nothing is asked of `git` when no claim of a listed capability names an
/// evidence record.
pub(super) fn raise(
    opened: &Opened,
    runner: &dyn Runner,
    prefix: &Prefix,
    impact: &Impact,
    base: &mut Option<MergeBase>,
) -> Result<Vec<Obligation>, LifecycleError> {
    let plan = opened.workplan();
    let listed: Vec<&Loaded<Capability>> = plan
        .capabilities()
        .iter()
        .filter(|cap| impact.capabilities.contains(&cap.value().header.id))
        .collect();
    let named: BTreeSet<String> = listed
        .iter()
        .flat_map(|cap| cap.body().claims())
        .flat_map(|claim| claim.evidence().iter())
        .map(|slug| format!("evidence:{slug}"))
        .collect();
    if named.is_empty() {
        return Ok(Vec::new());
    }
    let base = untouched::merge_base(base, runner)?;
    let mut changes: BTreeMap<&str, Vec<EvidenceChange>> = BTreeMap::new();
    for id in &named {
        let path = record_path(id);
        let Some(then) = record_at(runner, base, prefix, &path)? else {
            continue;
        };
        if then.verdict != EvidenceVerdict::Pass {
            continue;
        }
        let now = plan
            .evidence()
            .iter()
            .find(|record| record.value().header.id == *id)
            .map(Loaded::value);
        changes.insert(id, compare(&then, now));
    }
    let mut owed = Vec::new();
    for cap in listed {
        for claim in cap.body().claims() {
            for slug in claim.evidence() {
                let id = format!("evidence:{slug}");
                for change in changes.get(id.as_str()).into_iter().flatten() {
                    owed.push(Obligation {
                        kind: ObligationKind::EvidenceChanged,
                        subject: Subject::Capability(cap.value().header.id.clone()),
                        section: claim_part(claim.section()),
                        location: Place {
                            path: cap.path().to_string(),
                            line: Some(claim.line()),
                        },
                        evidence: Needed::ReviewedClaim {
                            evidence: id.clone(),
                            change: change.clone(),
                        },
                    });
                }
            }
        }
    }
    Ok(owed)
}

/// What changed from the passing record `then` to `now`, or that it is gone.
fn compare(then: &EvidenceRecord, now: Option<&EvidenceRecord>) -> Vec<EvidenceChange> {
    let Some(now) = now else {
        return vec![EvidenceChange::Removed];
    };
    let mut changes = Vec::new();
    if now.verdict == EvidenceVerdict::Fail {
        changes.push(EvidenceChange::Failed);
    }
    let limit_of = |limits: &[Measure], name: &str| -> Option<Measure> {
        limits.iter().find(|limit| limit.name == name).cloned()
    };
    for value in &now.values {
        // The tighter of the two limits a value has, the record's own and the
        // one of the merge base.
        let tightest = [
            limit_of(&now.limits, &value.name),
            limit_of(&then.limits, &value.name),
        ]
        .into_iter()
        .flatten()
        .filter(|limit| value.value.exceeds(&limit.value))
        .min_by(|a, b| a.value.compare(&b.value));
        if let Some(limit) = tightest {
            changes.push(EvidenceChange::OverLimit(OverLimit {
                name: value.name.clone(),
                value: value.value.text().to_string(),
                limit: limit.value.text().to_string(),
            }));
        }
    }
    for limit in &then.limits {
        if limit_of(&now.limits, &limit.name).is_none() {
            changes.push(EvidenceChange::LimitRemoved(limit.name.clone()));
        }
    }
    changes
}

/// The evidence record at `path` at the merge base, or `None` when no file is
/// there.
fn record_at(
    runner: &dyn Runner,
    base: &MergeBase,
    prefix: &Prefix,
    path: &str,
) -> Result<Option<EvidenceRecord>, LifecycleError> {
    let Some(bytes) = untouched::blob(runner, base, prefix, path)? else {
        return Ok(None);
    };
    let refused = |reason: String| LifecycleError::Document {
        path: path.to_string(),
        side: Side::MergeBase,
        reason,
    };
    let text = String::from_utf8(bytes).map_err(|_| refused("is not UTF-8 text".to_string()))?;
    let document = Document::parse(&text).map_err(|error| refused(error.to_string()))?;
    match Profile.validate(&document, path) {
        Ok(RafDocument::Evidence(record)) => Ok(Some(record)),
        Ok(_) => Err(refused("is not an evidence record".to_string())),
        Err(violations) => Err(refused(format!(
            "is not a valid evidence record: {}",
            violations
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        ))),
    }
}

#[cfg(test)]
mod tests {
    use raf_okf::{
        Actor, EvidenceRecord, EvidenceSubject, EvidenceVerdict, Input, InputKind, Measure, Number,
    };

    use super::compare;
    use crate::obligation::{EvidenceChange, OverLimit};

    fn measure(name: &str, value: &str) -> Measure {
        Measure::new(name, Number::parse(value).expect("a number"))
    }

    fn record(
        verdict: EvidenceVerdict,
        values: &[(&str, &str)],
        limits: &[(&str, &str)],
    ) -> EvidenceRecord {
        let mut record = EvidenceRecord::new(
            "bound",
            EvidenceSubject::Command,
            vec![Input::new(InputKind::Symbol, "demo::run", "0".repeat(64))],
            verdict,
            Actor::Agent,
            vec!["cap:demo".to_string()],
        );
        record.values = values.iter().map(|(n, v)| measure(n, v)).collect();
        record.limits = limits.iter().map(|(n, v)| measure(n, v)).collect();
        record
    }

    fn over(name: &str, value: &str, limit: &str) -> EvidenceChange {
        EvidenceChange::OverLimit(OverLimit {
            name: name.to_string(),
            value: value.to_string(),
            limit: limit.to_string(),
        })
    }

    #[test]
    fn a_value_over_a_limit_by_less_than_a_float_tells_apart_is_a_change() {
        let one_e400 = format!("1{}", "0".repeat(400));
        let two_e400 = format!("2{}", "0".repeat(400));
        for (over_text, limit, under) in [
            ("9007199254740993", "9007199254740992", "9007199254740991"),
            ("0.30000000000000001", "0.3", "0.29999999999999999"),
            ("1.0000000000000000001", "1", "0.9999999999999999999"),
            (two_e400.as_str(), one_e400.as_str(), &one_e400[..400]),
        ] {
            let then = record(
                EvidenceVerdict::Pass,
                &[("peak", under)],
                &[("peak", limit)],
            );
            // The limit of the merge base is loosened, and the value passes it.
            let now = record(
                EvidenceVerdict::Pass,
                &[("peak", over_text)],
                &[("peak", &format!("{limit}1"))],
            );
            assert_eq!(compare(&then, Some(&now)), [over("peak", over_text, limit)]);
            for neighbour in [limit, under] {
                let now = record(
                    EvidenceVerdict::Pass,
                    &[("peak", neighbour)],
                    &[("peak", limit)],
                );
                assert_eq!(compare(&then, Some(&now)), [], "{neighbour}");
            }
        }
    }

    #[test]
    fn an_unchanged_or_better_record_owes_nothing() {
        let then = record(EvidenceVerdict::Pass, &[("peak", "41")], &[("peak", "64")]);
        assert_eq!(compare(&then, Some(&then)), []);
        let tighter = record(EvidenceVerdict::Pass, &[("peak", "30")], &[("peak", "32")]);
        assert_eq!(compare(&then, Some(&tighter)), []);
    }

    #[test]
    fn each_change_is_named() {
        let then = record(EvidenceVerdict::Pass, &[("peak", "41")], &[("peak", "64")]);
        assert_eq!(compare(&then, None), [EvidenceChange::Removed]);
        let failing = record(EvidenceVerdict::Fail, &[("peak", "80")], &[("peak", "64")]);
        assert_eq!(
            compare(&then, Some(&failing)),
            [EvidenceChange::Failed, over("peak", "80", "64")]
        );
        let unbounded = record(EvidenceVerdict::Pass, &[("peak", "41")], &[]);
        assert_eq!(
            compare(&then, Some(&unbounded)),
            [EvidenceChange::LimitRemoved("peak".to_string())]
        );
        // A limit loosened past the old one: the value is over the limit of
        // the merge base, which is named.
        let loosened = record(EvidenceVerdict::Pass, &[("peak", "80")], &[("peak", "100")]);
        assert_eq!(compare(&then, Some(&loosened)), [over("peak", "80", "64")]);
    }
}
