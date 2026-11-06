// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! End-to-end fixtures for document rules of the `raf-okf` profile, with
//! clean neighbors. Native operational refusals have native-result unit
//! fixtures in the profile and finding modules.

mod support;

use raf_okf::Rule;
use support::cases::rule_case;
use support::fixtures::valid;

#[test]
fn every_rule_has_a_fixture_reported_with_file_and_location() {
    // The neighbours are the valid repository the fixtures break.
    support::assert_clean(&support::check(&valid()));
    for tag in Rule::ALL {
        let Some(case) = rule_case(tag) else {
            assert_eq!(
                tag,
                raf_okf::RuleTag::Markdown,
                "only tested native operational refusals have no document fixture"
            );
            continue;
        };
        assert_eq!(case.kind, format!("okf-{}", tag.name()));
        case.assert_exact();
    }
}
