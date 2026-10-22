// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One end-to-end fixture per rule of the `raf-okf` profile: each breaks that
//! rule in one document of a valid repository and is reported with the file and
//! the location, with the neighbouring documents clean.

mod support;

use raf_okf::Rule;
use support::cases::rule_case;
use support::fixtures::valid;

#[test]
fn every_rule_has_a_fixture_reported_with_file_and_location() {
    // The neighbours are the valid repository the fixtures break.
    support::assert_clean(&support::check(&valid()));
    for tag in Rule::ALL {
        let case = rule_case(tag);
        assert_eq!(case.kind, format!("okf-{}", tag.name()));
        case.assert_exact();
    }
}
