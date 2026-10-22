// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
use purrdf_core::SerializeGraph;
use purrdf_rdf::{parse_dataset, serialize_dataset};
fn main() {
    let mut args = std::env::args().skip(1);
    let output = std::path::PathBuf::from(args.next().expect("output directory"));
    std::fs::create_dir_all(&output).unwrap();
    for path in args {
        if path.ends_with(".oracle.nt") { continue; }
        let media = if path.ends_with(".ttl") { "text/turtle" } else { "application/n-triples" };
        let bytes = std::fs::read(&path).unwrap();
        let dataset = parse_dataset(&bytes, media, None).expect("registry parser accepts graph with no caller base");
        let nt = serialize_dataset(&*dataset, "application/n-triples", SerializeGraph::Dataset).unwrap();
        let text = String::from_utf8(nt).unwrap();
        let mut triples: Vec<_> = text.lines().collect();
        triples.sort_unstable();
        let normalized = format!("{}\n", triples.join("\n"));
        let name = path.replace(['/', '\\'], "_");
        std::fs::write(output.join(format!("{name}.nt")), normalized).unwrap();
        let pin = std::path::Path::new(&path).with_extension("oracle.nt");
        assert_eq!(std::fs::read_to_string(output.join(format!("{name}.nt"))).unwrap(), std::fs::read_to_string(pin).unwrap(), "{path}: registry graph pin");
        println!("{path}: {} registry-parsed ground triples match the pin", triples.len());
    }
}
