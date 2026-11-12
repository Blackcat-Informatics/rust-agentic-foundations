// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The self-test holds on this build and names a case the census stops holding.

use purrdf_jsonschema::ecma::MatchLimits;

use super::detectors::{Family, Probe, Selection, holds};
use super::engine::{CONTROL_FLOW, RENAMED, groups_as_one, separates};
use crate::{FileCtx, Gate, Rule, RuleFixtures, RuleHit};

#[test]
fn every_case_holds_on_this_build_and_two_runs_print_the_same_text() {
    let first = Gate::workspace("/nonexistent").self_test();
    let text = first.to_string();
    assert!(first.passed(), "{text}");
    assert_eq!(
        text,
        Gate::workspace("/nonexistent").self_test().to_string()
    );
    let ids: Vec<_> = first.cases().iter().map(super::SelfTestCase::id).collect();
    for id in crate::detectors::ids() {
        assert!(
            ids.iter()
                .any(|case| case.starts_with(&format!("detector/{id}/"))),
            "no case for detector {id}"
        );
    }
}

fn probe(consumer: &'static str) -> Probe {
    Probe {
        home: "/// Home.\npub fn home() {}".into(),
        consumer,
        rows: Vec::new(),
    }
}

#[test]
fn a_detector_that_stops_firing_or_starts_over_firing_fails_its_side() {
    let limits = MatchLimits::default();
    let family = |positive, negative| Family {
        selection: Selection::Job,
        positive: vec![probe(positive)],
        negative: vec![probe(negative)],
    };
    let flags = "fn copy(n:usize,d:usize)->usize {(n+d-1)/d}";
    let spares = "fn copy(n:usize,d:usize)->usize {n.div_ceil(d)}";
    let good = family(flags, spares);
    assert_eq!(holds(limits, "div-ceil-by-hand", &good, true), Ok(()));
    assert_eq!(holds(limits, "div-ceil-by-hand", &good, false), Ok(()));
    // A detector that missed its positive is the positive fixture it ignores.
    let blind = family(spares, spares);
    assert!(holds(limits, "div-ceil-by-hand", &blind, true).is_err());
    // A detector that flags the negative is the negative fixture it flags.
    let eager = family(flags, flags);
    assert!(holds(limits, "div-ceil-by-hand", &eager, false).is_err());
}

#[test]
fn a_normal_form_that_merges_or_splits_wrongly_fails_its_case() {
    let limits = MatchLimits::default();
    assert_eq!(groups_as_one(limits, RENAMED), Ok(()));
    // A pair the form must keep apart, offered as a copy, models a form that
    // merged too much; a copy offered as a pair to keep apart models one that
    // split too much.
    assert!(groups_as_one(limits, CONTROL_FLOW).is_err());
    assert_eq!(separates(limits, CONTROL_FLOW), Ok(()));
    assert!(separates(limits, RENAMED).is_err());
}

struct Broken;
struct Healthy;
impl Rule for Healthy {
    fn id(&self) -> &'static str {
        "healthy-rule"
    }
    fn check(&self, _: &FileCtx, file: &syn::File, out: &mut Vec<RuleHit>) {
        for item in &file.items {
            if let syn::Item::Fn(item) = item
                && item.sig.ident == "copy"
            {
                out.push(RuleHit {
                    line: 1,
                    message: "use home".into(),
                    symbol: "fixture::copy".into(),
                });
            }
        }
    }
    fn fixtures(&self) -> RuleFixtures {
        Broken.fixtures()
    }
}

impl Rule for Broken {
    fn id(&self) -> &'static str {
        "broken-rule"
    }
    fn check(&self, _: &FileCtx, _: &syn::File, _: &mut Vec<RuleHit>) {}
    fn fixtures(&self) -> RuleFixtures {
        RuleFixtures {
            positive: vec!["fn copy() {}"],
            negative: vec!["fn home() {}"],
        }
    }
}

#[test]
fn a_registered_rule_whose_fixture_fails_is_a_named_failing_case() {
    let report = Gate::workspace("/nonexistent")
        .rules(vec![Box::new(Broken) as Box<dyn Rule>])
        .self_test();
    assert!(!report.passed());
    let failed: Vec<_> = report
        .cases()
        .iter()
        .filter(|case| !case.passed())
        .collect();
    assert_eq!(failed.len(), 1, "{report}");
    assert_eq!(failed[0].id(), "rule/broken-rule/fixtures");
    assert!(
        report
            .to_string()
            .contains("fail rule/broken-rule/fixtures: ")
    );
}

#[test]
fn a_registered_rule_whose_fixtures_hold_is_a_passing_case() {
    let report = Gate::workspace("/nonexistent")
        .rules(vec![Box::new(Healthy) as Box<dyn Rule>])
        .self_test();
    assert!(report.passed(), "{report}");
    assert!(
        report
            .to_string()
            .contains("pass rule/healthy-rule/fixtures\n")
    );
}

const CLI_DOC: &str = include_str!("../../../../docs/capabilities/census-cli.md");

/// The documented detector list and case counts equal the registry's run, so adding, removing
/// or renaming a detector, or changing the case count, fails until the capability document
/// states the new list.
#[test]
fn the_documented_detector_list_and_case_count_equal_the_registry() {
    let report = Gate::workspace("/nonexistent").self_test();
    let total = report.cases().len();

    let (_, after) = CLI_DOC
        .split_once("The detectors are ")
        .expect("the cases list names the detectors");
    let sentence = after.split_once(".\n").expect("the sentence ends").0;
    let documented: Vec<&str> = sentence.split('`').skip(1).step_by(2).collect();
    let mut registered: Vec<&str> = Vec::new();
    for case in report.cases() {
        if let Some(id) = case
            .id()
            .strip_prefix("detector/")
            .and_then(|rest| rest.strip_suffix("/positive"))
        {
            registered.push(id);
        }
    }
    assert_eq!(documented, registered);

    let counts: Vec<&str> = CLI_DOC
        .match_indices(" cases, 0 failed")
        .map(|(end, _)| {
            let head = &CLI_DOC[..end];
            &head[head
                .rfind(|c: char| !c.is_ascii_digit())
                .map_or(0, |at| at + 1)..]
        })
        .collect();
    assert!(!counts.is_empty(), "the document states the case count");
    for count in counts {
        assert_eq!(count, total.to_string(), "documented case count");
    }
}
