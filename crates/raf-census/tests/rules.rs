// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Rule admission executes both source sets through the real bounded parser.

use raf_census::{Error, FileCtx, Rule, RuleFixtures, RuleHit};

struct Flagged;

impl Rule for Flagged {
    fn id(&self) -> &'static str {
        "flagged"
    }

    fn check(&self, ctx: &FileCtx, file: &syn::File, out: &mut Vec<RuleHit>) {
        assert!(ctx.shipping);
        assert_eq!(ctx.package, "fixture");
        for item in &file.items {
            if let syn::Item::Fn(item) = item
                && item.sig.ident == "flagged"
            {
                out.push(RuleHit {
                    line: item.sig.ident.span().start().line,
                    message: "use the approved implementation".into(),
                    symbol: "fixture::flagged".into(),
                });
            }
        }
    }

    fn fixtures(&self) -> RuleFixtures {
        RuleFixtures {
            positive: vec!["fn flagged() {}"],
            negative: vec!["fn ordinary() {}"],
        }
    }
}

#[test]
fn the_rule_runs_its_positive_and_negative_source_cases() {
    Flagged.fixtures().validate(&Flagged).unwrap();
}

#[test]
fn empty_source_sets_are_refused_with_a_passing_neighbour() {
    let valid = Flagged.fixtures();
    for invalid in [
        RuleFixtures {
            positive: vec![],
            negative: valid.negative.clone(),
        },
        RuleFixtures {
            positive: valid.positive.clone(),
            negative: vec![],
        },
    ] {
        assert!(
            invalid
                .validate(&Flagged)
                .unwrap_err()
                .to_string()
                .contains("both positive and negative")
        );
    }
    valid.validate(&Flagged).unwrap();
}

#[test]
fn positive_misses_and_negative_hits_are_refused() {
    for (invalid, reason) in [
        (
            RuleFixtures {
                positive: vec!["fn ordinary() {}"],
                negative: vec!["fn ordinary() {}"],
            },
            "positive fixture produced no hit",
        ),
        (
            RuleFixtures {
                positive: vec!["fn flagged() {}"],
                negative: vec!["fn flagged() {}"],
            },
            "negative fixture produced a hit",
        ),
    ] {
        assert!(
            invalid
                .validate(&Flagged)
                .unwrap_err()
                .to_string()
                .contains(reason)
        );
    }
    Flagged.fixtures().validate(&Flagged).unwrap();
}

#[test]
fn invalid_rust_is_a_parse_failure_and_does_not_count_as_a_negative_case() {
    let invalid = RuleFixtures {
        positive: vec!["fn flagged() {}"],
        negative: vec!["fn broken("],
    };
    assert!(
        matches!(invalid.validate(&Flagged), Err(Error::Input { source, .. }) if matches!(*source, Error::Load(_)))
    );
    Flagged.fixtures().validate(&Flagged).unwrap();
}
