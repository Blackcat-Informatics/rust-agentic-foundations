// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One end-to-end fixture per way a document fails to read, reported with its
//! file and line beside documents that read.

mod support;

use raf_okf::ErrorKind;
use support::cases::parse_case;

#[test]
fn every_reader_error_has_a_fixture_reported_with_file_and_line() {
    for tag in ErrorKind::ALL {
        let case = parse_case(tag);
        case.assert_exact();
    }
}
