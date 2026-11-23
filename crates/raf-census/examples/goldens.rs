// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Prints the pinned values of normal form 1: the manifest and the fingerprint
//! table of every registered fixture, in the form `tests/golden.rs` pins.
//!
//! Run it with no arguments. A new normal form is the output of this example
//! pasted over the pinned constants.

#[path = "../tests/support/fixtures.rs"]
mod fixtures;

#[expect(
    clippy::print_stdout,
    reason = "this executable writes its results to the caller's standard output"
)]
fn main() {
    println!("manifest");
    print!("{}", raf_census::manifest());
    println!("table");
    print!("{}", fixtures::table(&fixtures::registry()));
}
