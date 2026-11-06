// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Document refusal fixtures report their file and line beside documents
//! that read. Native operational failures have native-result unit fixtures
//! in the reader error and finding modules.

mod support;

use raf_okf::ErrorKind;
use support::cases::parse_case;

#[test]
fn every_reader_error_has_a_fixture_reported_with_file_and_line() {
    for tag in ErrorKind::ALL {
        let Some(case) = parse_case(tag) else {
            assert_eq!(
                tag,
                raf_okf::ErrorTag::Markdown,
                "only tested native operational refusals have no document fixture"
            );
            continue;
        };
        case.assert_exact();
    }
}
