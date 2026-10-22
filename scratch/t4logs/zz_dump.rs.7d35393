// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Review dump of every case under default (scratch only).

mod support;

#[test]
fn dump() {
    let mut out = String::new();
    for mut case in support::cases::all_cases() {
        case.strictness = raf_workplan::Strictness::Default;
        out.push_str(&format!("=== {}\n{:#?}\n", case.name, case.run()));
    }
    std::fs::write(std::env::var("DUMP_TO").expect("DUMP_TO"), out).expect("write");
}
