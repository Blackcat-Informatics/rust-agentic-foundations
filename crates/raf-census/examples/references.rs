// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Full reference receipts and witnesses over an owned path dependency.

#[path = "../tests/support/reference.rs"]
mod reference;

use raf_census::{Gate, Index, Query, QueryReport};
use std::{error::Error, io::Write as _};

fn main() -> Result<(), Box<dyn Error>> {
    let tree = reference::fixture(
        "pub fn body(value:u32)->u32 { let doubled=value*2; doubled+11 }\npub fn shim(value:u32)->u32 { std::convert::identity(value) }",
        "pub mod api { pub fn first(value:u32)->u32 { let doubled=value*2; doubled+11 } pub fn second(input:u32)->u32 { let result=input*2; result+11 } pub fn shim(value:u32)->u32 { std::convert::identity(value) } }",
    );
    let report = Gate::workspace(".").check_tree(&tree)?;
    assert_eq!(report.jobs()[0].copies, 2);
    assert_eq!(report.census().groups(), []);
    let index = report.census().index(report.jobs())?;
    let text = index.to_ntriples()?;
    assert_eq!(Index::from_ntriples(&text)?, index);
    let query = index.query(Query::Job("arithmetic"))?.to_ntriples()?;
    assert_eq!(QueryReport::from_ntriples(&query)?.to_ntriples()?, query);
    std::io::stdout().lock().write_all(text.as_bytes())?;
    Ok(())
}
