// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The embedded example's production run, asserted through its own helper.

#[allow(
    dead_code,
    reason = "the example's entrypoint prints the witnesses and is not invoked by these tests"
)]
#[path = "../examples/embedded.rs"]
mod embedded;

use embedded::{COPY_SOURCE, NEIGHBOUR_SOURCE, Pair, Side};

/// Start of the copied statements in the source; they begin the verified region.
fn copied_start(source: &str, marker: &str) -> usize {
    source
        .find(marker)
        .expect("the fixture contains the copied statement")
}

/// The helper side and the larger side of a pair, in that order.
fn helper_and_larger(pair: &Pair) -> (&Side, &Side) {
    if pair.left.function == "helper" {
        (&pair.left, &pair.right)
    } else {
        (&pair.right, &pair.left)
    }
}

#[test]
fn copied_helper_pairs_with_larger_at_its_verified_spans() {
    let pairs = embedded::witnesses(COPY_SOURCE).expect("the production run analyzes the copy");
    let pair = pairs
        .iter()
        .find(|pair| {
            let mut names = [pair.left.function.as_str(), pair.right.function.as_str()];
            names.sort_unstable();
            names == ["helper", "larger"]
        })
        .expect("a verified pair joins helper and larger");
    let (helper, larger) = helper_and_larger(pair);

    // The verified region is the helper's three `let` statements and tail: the
    // span starts at its first statement, and its text matches the source.
    let helper_start = copied_start(COPY_SOURCE, "let scaled");
    assert_eq!(helper.bytes.start, helper_start);
    assert!(helper.text.starts_with("let scaled = input * 3 + 7;"));
    assert!(helper.text.ends_with("total % 11 + total"));
    assert_eq!(helper.lines, 2..=5);
    assert!(helper.body.start <= helper.bytes.start && helper.bytes.end <= helper.body.end);

    // The larger side is its renamed copy inside `larger`'s own body.
    assert!(larger.text.starts_with("let amount = value * 3 + 7;"));
    assert!(larger.text.ends_with("sum % 11 + sum"));
    assert_eq!(larger.lines, 11..=14);
    assert!(larger.body.start <= larger.bytes.start && larger.bytes.end <= larger.body.end);
    assert_eq!(larger.bytes.start, copied_start(COPY_SOURCE, "let amount"));
}

#[test]
fn a_neighbouring_operator_change_yields_no_witness() {
    let pairs =
        embedded::witnesses(NEIGHBOUR_SOURCE).expect("the production run analyzes the neighbour");
    assert!(
        pairs.is_empty(),
        "a changed operator refuses the correspondence, found {} pair(s)",
        pairs.len()
    );
}
