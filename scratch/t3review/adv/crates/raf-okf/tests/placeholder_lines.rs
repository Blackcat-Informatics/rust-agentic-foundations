// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `Body::placeholder_lines` applies the placeholder rule of the profile to any body.

use raf_okf::Body;

const TOKEN: &str = concat!("t", "bd");

#[test]
fn reports_each_token_in_prose_with_its_line() {
    let text = format!("# Notes\n\nStill {TOKEN} here.\n\nT.B.D. and {TOKEN}s.\n");
    let body = Body::parse(&text).expect("parses");
    assert_eq!(body.placeholder_lines(), [(3, TOKEN), (5, TOKEN)]);
}

#[test]
fn does_not_read_code_or_comments() {
    let text = format!(
        "# Notes\n\n`{TOKEN}` and <!-- {TOKEN} -->\n\n```text\n{TOKEN}\n```\n\n<!--\n{TOKEN}\n-->\n"
    );
    let body = Body::parse(&text).expect("parses");
    assert_eq!(body.placeholder_lines(), []);
}
