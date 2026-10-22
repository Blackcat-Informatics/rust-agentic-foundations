// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
//! Capture canonical report fixtures through the actual production API.
#[allow(dead_code, reason = "Fixture capture uses only the built-in workspace helpers.")]
#[path = "../tests/support/builtin.rs"]
mod builtin;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let census = raf_census::Census::load(&builtin::tree(), &["."], &raf_census::CensusOptions::default())?;
    let baseline = census.baseline().to_turtle()?;
    let index = census.index(&census.match_jobs(&builtin::jobs(&census)))?.to_turtle()?;
    let mut awkward = census.index(&[])?;
    awkward.groups.clear(); awkward.units.truncate(1);
    awkward.units[0].symbol = "demo::r#type@x\"q\\é".into();
    awkward.units[0].file = "a b/#%?:@\\é.rs".into();
    let docs = base.join("../../docs/capabilities/census-normal-form.md");
    let mut document = std::fs::read_to_string(&docs)?;
    for (name, text) in [("baseline", &baseline), ("index", &index)] {
        let path = base.join(format!("tests/fixtures/reports/{name}.ttl"));
        let previous = std::fs::read_to_string(&path)?;
        let previous_block = format!("```turtle\n{previous}```");
        assert!(document.contains(&previous_block), "old documented {name} fixture");
        document = document.replace(&previous_block, &format!("```turtle\n{text}```"));
        std::fs::write(path, text)?;
    }
    std::fs::write(docs, document)?;
    std::fs::write(base.join("tests/fixtures/reports/example.txt"), format!("baseline\n{baseline}index\n{index}"))?;
    std::fs::write(base.join("tests/fixtures/reports/awkward.ttl"), awkward.to_turtle()?)?;
    Ok(())
}
