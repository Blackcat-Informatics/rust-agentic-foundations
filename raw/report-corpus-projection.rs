// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Temporary projection of canonical corpus carriers to the current report layout.

use raf_rdf::{Builder, Format, Graph, TermValue, vocab};
use std::{error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus/census-report-read");
    let version = TermValue::iri(format!("{}reportForm", vocab::REPORT));
    for name in [
        "baseline.ttl", "baseline.nt", "index.ttl", "index.nt", "individual.ttl",
        "class.ttl", "unknown.ttl", "repeated.ttl", "orphan.ttl", "copies.ttl",
        "single.ttl", "name.ttl", "mistyped.ttl", "missing.ttl", "datatype.ttl",
    ] {
        let path = directory.join(name);
        let format = if name.ends_with(".nt") { Format::NTriples } else { Format::Turtle };
        let graph = Graph::from_text(&fs::read_to_string(&path)?, format)?;
        let mut builder = Builder::new();
        let mut replaced = 0;
        for mut triple in graph.assertions() {
            if triple.predicate == version {
                assert_eq!(triple.object, TermValue::integer(2_u32), "{name}");
                triple.object = TermValue::integer(raf_census::REPORT_FORM);
                replaced += 1;
            }
            builder.insert(triple)?;
        }
        assert_eq!(replaced, 1, "{name}");
        fs::write(path, builder.finish()?.to_text(format)?)?;
        println!("projected {name}");
    }
    Ok(())
}
