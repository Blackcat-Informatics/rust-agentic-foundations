// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Selectable tests of the actual corpus-analysis example and production forms.

#[allow(
    dead_code,
    reason = "the example's command entrypoint and source-corpus renderer are not invoked by its fixture-policy test"
)]
#[path = "../examples/selection.rs"]
mod selection;

use raf_census::{Level, Tower};
use selection::{Body, Policy, fixtures, groups};

#[test]
fn fixture_counts_are_pinned_and_default_selection_matches_production() {
    let registry = fixtures::registry();
    let mut bodies = Vec::new();
    let policy = Policy {
        floor: raf_census::DEFAULT_MIN_TOKENS,
        tier: raf_census::LITERAL_TIER,
        shim_min: raf_census::MIN_SHIM_TOKENS,
        shim_max: raf_census::SHIM_MAX_TOKENS,
    };
    for fixture in &registry.fixtures {
        let tower = Tower::new(&fixture.slice()).expect("fixture tower");
        let body = Body::new(fixture.id.clone(), &tower);
        let print = tower.print(raf_census::DEFAULT_MIN_TOKENS);
        let expected = if print.shim.is_some() {
            (!tower.calls_through_self()).then(|| (true, tower.form(Level::Shim).texts()))
        } else {
            print
                .body
                .as_ref()
                .map(|_| (false, tower.form(Level::Value).texts()))
        };
        assert_eq!(
            policy.key(&body).map(|(shim, form)| (shim, form.to_vec())),
            expected,
            "{}",
            fixture.id
        );
        bodies.push(body);
    }
    assert_eq!(
        (
            bodies.len(),
            bodies.iter().map(|body| body.tokens).sum::<usize>()
        ),
        (261, 3900)
    );
    let counts = groups(&bodies, policy);
    assert_eq!(counts.values().filter(|group| group.len() > 1).count(), 22);
    assert_eq!(
        counts
            .values()
            .map(|group| group.len() * group.len().saturating_sub(1) / 2)
            .sum::<usize>(),
        54
    );
}
