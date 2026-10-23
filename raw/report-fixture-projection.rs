// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Temporary report-version fixture projection through the native RDF carrier.

use raf_rdf::{Builder, Format, Graph, TermValue, vocab};
use std::{collections::BTreeMap, error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/fixtures/reports");
    let version = format!("{}reportForm", vocab::REPORT);
    let old_version = format!("<{version}> \"2\"^^<{}integer>", vocab::XSD);
    let new_version = format!("<{version}> \"3\"^^<{}integer>", vocab::XSD);
    let mut examples = BTreeMap::new();
    let document_path = root.join("../../docs/capabilities/census-normal-form.md");
    let mut document = fs::read_to_string(&document_path)?;
    for name in ["baseline", "index", "awkward"] {
        let path = fixtures.join(format!("{name}.ttl"));
        let old_text = fs::read_to_string(&path)?;
        let old = Graph::from_text(&old_text, Format::Turtle)?;
        let mut builder = Builder::new();
        for mut triple in old.assertions() {
            if triple.predicate == TermValue::iri(&version) {
                triple.object = TermValue::integer(3_u32);
            }
            builder.insert(triple)?;
        }
        let graph = builder.finish()?;
        let turtle = graph.to_text(Format::Turtle)?;
        let nt = graph.to_text(Format::NTriples)?;
        let oracle_path = fixtures.join(format!("{name}.oracle.nt"));
        let oracle = fs::read_to_string(&oracle_path)?
            .replace(&old.snapshot()?, &graph.snapshot()?)
            .replace(&old_version, &new_version);
        let mut oracle_rows: Vec<_> = oracle.lines().collect();
        oracle_rows.sort_unstable();
        let oracle = format!("{}\n", oracle_rows.join("\n"));
        let mut actual_rows: Vec<_> = nt.lines().collect();
        actual_rows.sort_unstable();
        assert_eq!(format!("{}\n", actual_rows.join("\n")), oracle);
        if name != "awkward" {
            assert!(document.contains(&old_text));
            document = document.replace(&old_text, &turtle);
            examples.insert(name, turtle.clone());
        }
        fs::write(path, turtle)?;
        fs::write(fixtures.join(format!("{name}.nt")), nt)?;
        fs::write(oracle_path, oracle)?;
    }
    fs::write(
        fixtures.join("example.txt"),
        format!("baseline\n{}index\n{}", examples["baseline"], examples["index"]),
    )?;
    fs::write(document_path, document)?;
    Ok(())
}
