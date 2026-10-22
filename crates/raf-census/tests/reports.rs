// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Both canonical RDF formats preserve census domain refusals and NF1 identities.

use raf_census::{
    Baseline, Census, CensusOptions, Error, Index, Job, JobForbidden, MatchClass, ReportError,
};
use raf_rdf::{Builder, Format, Graph, TermValue, Triple, vocab};
use raf_source::{DiskTree, MemoryTree};
use std::path::Path;
#[path = "support/builtin.rs"]
mod builtin;
const BUILTIN: &str = include_str!("fixtures/reports/example.txt");
fn builtin_census() -> Census {
    Census::load(&builtin::tree(), &["."], &CensusOptions::default()).unwrap()
}
fn halves() -> (String, String) {
    let text = builtin::builtin().unwrap();
    let (a, b) = text
        .strip_prefix("baseline\n")
        .unwrap()
        .split_once("index\n")
        .unwrap();
    (a.into(), b.into())
}
fn property(name: &str) -> TermValue {
    TermValue::iri(format!("{}{name}", vocab::REPORT))
}
fn rewrite(text: &str, mut edit: impl FnMut(Triple) -> Option<Triple>) -> String {
    let original = Graph::from_text(text, Format::Turtle).unwrap();
    let mut graph = Builder::new();
    for triple in original.assertions() {
        if let Some(value) = edit(triple) {
            graph.insert(value).unwrap();
        }
    }
    let graph = graph.finish().unwrap();
    assert_ne!(
        graph.snapshot().unwrap(),
        original.snapshot().unwrap(),
        "the negative fixture must actually change a fact"
    );
    graph.to_text(Format::Turtle).unwrap()
}
fn refused(result: Result<impl std::fmt::Debug, Error>, needle: &str) {
    match result {
        Err(Error::Report(error)) => assert!(error.to_string().contains(needle), "{error}"),
        other => panic!("expected report refusal {needle}, got {other:?}"),
    }
}
#[test]
fn the_example_output_is_the_pinned_golden() {
    assert_eq!(builtin::builtin().unwrap(), BUILTIN);
}
#[test]
fn the_example_output_names_the_expected_group_fingerprints() {
    let census = builtin_census();
    let output = builtin::builtin().unwrap();
    assert_eq!(census.body_groups().count(), 1);
    assert_eq!(census.shim_groups().count(), 1);
    assert!(
        output.contains("body:6bb022e241a79a281e80d6e233c0b8f61a027761c52bd4b0c8b232de92544609")
    );
    assert!(
        output.contains("shim:203616cb54a3e53de4ca5100ce3cf2aad3b59e9421fc50c09bc93deb7c974725")
    );
    assert!(output.contains(&format!("<{}jobId> \"sum\"", vocab::REPORT)));
}
#[test]
fn the_capability_examples_hold_the_example_output() {
    let document = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/capabilities/census-normal-form.md"),
    )
    .unwrap();
    for text in [halves().0, halves().1] {
        assert!(document.contains(&format!("```turtle\n{text}```")));
    }
}
#[test]
fn both_reports_read_back_what_they_write() {
    let (baseline, index) = halves();
    let b = Baseline::from_turtle(&baseline).unwrap();
    let i = Index::from_turtle(&index).unwrap();
    assert_eq!(b, builtin_census().baseline());
    assert_eq!(b.to_turtle().unwrap(), baseline);
    assert_eq!(i.to_turtle().unwrap(), index);
    assert_eq!(
        Baseline::from_ntriples(&b.to_ntriples().unwrap()).unwrap(),
        b
    );
    assert_eq!(Index::from_ntriples(&i.to_ntriples().unwrap()).unwrap(), i);
    assert_eq!(
        (b.units, i.units.len(), b.normal_form, b.min_tokens),
        (7, 7, 1, 20)
    );
    for (ttl, nt) in [
        (baseline, b.to_ntriples().unwrap()),
        (index, i.to_ntriples().unwrap()),
    ] {
        assert_eq!(
            Graph::from_text(&ttl, Format::Turtle)
                .unwrap()
                .snapshot()
                .unwrap(),
            Graph::from_text(&nt, Format::NTriples)
                .unwrap()
                .snapshot()
                .unwrap()
        );
    }
}
#[test]
fn output_is_the_canonical_turtle() {
    for text in [halves().0, halves().1] {
        let graph = Graph::from_text(&text, Format::Turtle).unwrap();
        assert_eq!(graph.to_text(Format::Turtle).unwrap(), text);
        assert!(text.starts_with("VERSION \"1.2\"\nBASE <"));
        assert!(text.ends_with(" .\n") && !text.ends_with("\n\n"));
        assert!(!text.contains('\r') && !text.contains('\t') && !text.contains("_:"));
    }
}

#[test]
fn native_scalar_readers_preserve_canonical_report_refusals_in_both_formats() {
    let (baseline, index) = halves();
    for (text, property_name, datatype, invalid, is_index) in [
        (
            &baseline,
            "minTokens",
            vocab::INTEGER,
            &[
                "+20",
                "020",
                "-1",
                "-0",
                "20.0",
                "1000000000000000000000000000000000000000",
            ][..],
            false,
        ),
        (
            &index,
            "shipping",
            vocab::BOOLEAN,
            &["0", "1", "TRUE", "false "][..],
            true,
        ),
    ] {
        let original = Graph::from_text(text, Format::Turtle).expect("valid fixture graph");
        for format in [Format::Turtle, Format::NTriples] {
            let accepts = |text: &str| match (format, is_index) {
                (Format::Turtle, false) => Baseline::from_turtle(text).is_ok(),
                (Format::NTriples, false) => Baseline::from_ntriples(text).is_ok(),
                (Format::Turtle, true) => Index::from_turtle(text).is_ok(),
                (Format::NTriples, true) => Index::from_ntriples(text).is_ok(),
            };
            let valid = original.to_text(format).expect("canonical neighbor");
            assert!(accepts(&valid));
            for lexical in invalid {
                let key = property(property_name);
                let old = original
                    .assertions()
                    .find(|t| t.predicate == key)
                    .expect("actual scalar");
                let mut builder = Builder::from_graph(&original);
                assert!(builder.remove(old.clone()).unwrap());
                builder
                    .insert(Triple {
                        object: TermValue::typed_literal(*lexical, datatype),
                        ..old
                    })
                    .unwrap();
                let graph = builder
                    .finish()
                    .expect("ground graph permits domain lexicals");
                let invalid = graph.to_text(format).expect("canonical ground text");
                assert!(!accepts(&invalid), "{lexical:?} {format:?}");
            }
        }
    }
}
#[test]
fn unknown_terms_are_not_a_report() {
    let (baseline, index) = halves();
    for name in ["extra", "column"] {
        let original = Graph::from_text(&baseline, Format::Turtle).unwrap();
        let mut graph = Builder::from_graph(&original);
        graph
            .insert(Triple {
                subject: TermValue::iri(""),
                predicate: property(name),
                object: TermValue::integer(1),
            })
            .unwrap();
        refused(
            Baseline::from_turtle(&graph.finish().unwrap().to_text(Format::Turtle).unwrap()),
            name,
        );
    }
    let text = rewrite(&baseline, |mut t| {
        if t.object == property("Unit") {
            t.object = property("Widget");
        }
        Some(t)
    });
    refused(Baseline::from_turtle(&text), "Widget");
    let text = rewrite(&index, |mut t| {
        if t.predicate == property("homeSymbol") {
            t.predicate = TermValue::iri("https://foreign.example/owner");
        }
        Some(t)
    });
    refused(Index::from_turtle(&text), "foreign.example/owner");
}
#[test]
fn decoded_control_characters_are_escaped_in_report_diagnostics() {
    let (baseline, _) = halves();
    let edited = baseline.replacen("minTokens>", "minTokens\\u001B>", 1);
    let error = Baseline::from_turtle(&edited).unwrap_err().to_string();
    assert!(!error.contains('\u{1b}'));
    assert!(error.contains("\\u{1b}"));
    assert!(Baseline::from_turtle(&baseline).is_ok());
}
#[test]
fn missing_repeated_and_mistyped_properties_are_not_a_report() {
    let (baseline, index) = halves();
    let missing = rewrite(&baseline, |t| {
        (t.predicate != property("minTokens")).then_some(t)
    });
    refused(Baseline::from_turtle(&missing), "minTokens");
    let mistyped = rewrite(&baseline, |mut t| {
        if t.predicate == property("normalForm") {
            t.object = TermValue::simple_literal("one");
        }
        Some(t)
    });
    refused(Baseline::from_turtle(&mistyped), "normalForm");
    let original = Graph::from_text(&index, Format::Turtle).unwrap();
    let mut graph = Builder::from_graph(&original);
    let mut repeated = original
        .assertions()
        .find(|t| t.predicate == property("body"))
        .unwrap();
    repeated.object = TermValue::iri(format!(
        "{}{}",
        raf_census::report::vocab::fingerprint_namespace(1, raf_census::GroupKind::Body),
        "0".repeat(64)
    ));
    graph.insert(repeated).unwrap();
    refused(
        Index::from_turtle(&graph.finish().unwrap().to_text(Format::Turtle).unwrap()),
        "body",
    );
    for (key, value, needle) in [
        ("matchClass", property("Duplicate"), "matchClass"),
        ("copies", TermValue::integer(3), "copies"),
    ] {
        let text = rewrite(&index, |mut t| {
            if t.predicate == property(key) {
                t.object = value.clone();
            }
            Some(t)
        });
        refused(Index::from_turtle(&text), needle);
    }
}
#[test]
fn names_that_disagree_with_their_node_are_not_a_report() {
    let (baseline, _) = halves();
    let text = rewrite(&baseline, |mut t| {
        if t.predicate == property("line") {
            t.object = TermValue::integer(99);
        }
        Some(t)
    });
    refused(Baseline::from_turtle(&text), "node name");
    let text = rewrite(&baseline, |mut t| {
        if t.predicate == property("normalForm") {
            t.object = TermValue::integer(2);
        }
        Some(t)
    });
    refused(Baseline::from_turtle(&text), "normalForm");
}
#[test]
fn orphan_units_and_matches_without_a_named_reifier_are_not_a_report() {
    let (baseline, index) = halves();
    let text = rewrite(&baseline, |t| {
        (t.predicate != property("holds")).then_some(t)
    });
    refused(Baseline::from_turtle(&text), "orphan");
    let text = rewrite(&baseline, |mut t| {
        if t.object == property("BodyGroup") {
            t.object = property("ShimGroup");
        }
        Some(t)
    });
    refused(Baseline::from_turtle(&text), "fingerprint");
    let text = rewrite(&index, |mut t| {
        if t.predicate == property("copies") {
            t.object = TermValue::integer(0);
        }
        (!matches!(&t.subject, TermValue::Iri(s) if s.starts_with("#match/"))).then_some(t)
    });
    refused(Index::from_turtle(&text), "exactly one named reifier");
}
#[test]
fn constructs_outside_the_profile_are_not_a_report() {
    let (baseline, _) = halves();
    for construct in [
        "_:b",
        "[]",
        "()",
        "# comment",
        "\"\"\"x\"\"\"",
        "'x'",
        "<<( <> <urn:p> <> )>>",
        "{| <urn:p> 1 |}",
    ] {
        refused(
            Baseline::from_turtle(&baseline.replacen("<> ", &format!("{construct} "), 1)),
            "",
        );
    }
    for version in ["1.1", "1.2-basic"] {
        refused(
            Baseline::from_turtle(&baseline.replacen("1.2", version, 1)),
            "",
        );
    }
    refused(
        Baseline::from_turtle(baseline.strip_prefix("VERSION \"1.2\"\n").unwrap()),
        "canonical",
    );
    for suffix in ["@en", "^^<http://www.w3.org/2001/XMLSchema#string>"] {
        refused(
            Baseline::from_turtle(&baseline.replacen(
                "\"src/lib.rs\"",
                &format!("\"src/lib.rs\"{suffix}"),
                1,
            )),
            "",
        );
    }
}
#[test]
fn an_edited_or_respaced_text_is_not_a_report() {
    let (baseline, _) = halves();
    refused(
        Baseline::from_turtle(&baseline.replace(" .\n", "  .\n")),
        "canonical",
    );
    refused(
        Baseline::from_turtle(&baseline.replacen("\"20\"", "\"21\"", 1)),
        "digest",
    );
}
#[test]
fn canonical_line_diagnostics_preserve_crlf_and_the_final_newline() {
    let (baseline, _) = halves();
    assert!(matches!(
        Baseline::from_turtle(&baseline.replace('\n', "\r\n")),
        Err(Error::Report(ReportError::Ground(
            raf_rdf::Error::NotCanonical { line: 1 }
        )))
    ));
    assert!(
        matches!(Baseline::from_turtle(baseline.strip_suffix('\n').unwrap()), Err(Error::Report(ReportError::Ground(raf_rdf::Error::NotCanonical { line }))) if line == baseline.lines().count())
    );
}
#[test]
fn an_index_is_not_a_baseline() {
    let (b, i) = halves();
    refused(Baseline::from_turtle(&i), "expected Baseline");
    refused(Index::from_turtle(&b), "expected Index");
}
#[test]
fn duplicate_job_ids_are_refused() {
    let census = builtin_census();
    let job = Job::new("same", "demo::alpha");
    assert!(matches!(
        census.index(&census.match_jobs(&[job.clone(), job])),
        Err(Error::Report(ReportError::DuplicateJob(_)))
    ));
}
#[test]
fn a_report_of_another_report_form_is_refused_naming_both() {
    let (baseline, _) = halves();
    for version in [1, 3] {
        let text = rewrite(&baseline, |mut t| {
            if t.predicate == property("reportForm") {
                t.object = TermValue::integer(version);
            }
            Some(t)
        });
        let nt = Graph::from_text(&text, Format::Turtle)
            .unwrap()
            .to_text(Format::NTriples)
            .unwrap();
        for result in [Baseline::from_turtle(&text), Baseline::from_ntriples(&nt)] {
            assert!(
                matches!(result, Err(Error::Report(ReportError::Form { declared, implemented: 2 })) if declared == version)
            );
        }
    }
}
#[test]
fn an_index_of_another_normal_form_reads_and_keeps_its_version() {
    let mut report = builtin_census().index(&[]).expect("index");
    report.normal_form = 7;
    let text = report.to_turtle().expect("renders old version");
    assert!(text.contains("/nf/7/"));
    let read = Index::from_turtle(&text).expect("reader preserves version");
    assert_eq!(read, report);
    assert!(raf_census::check_normal_form(read.normal_form).is_err());
}

#[test]
fn awkward_names_round_trip() {
    let mut report = builtin_census().index(&[]).expect("index");
    report.groups.clear();
    report.units.truncate(1);
    let unit = &mut report.units[0];
    unit.symbol = "demo::r#type@x\"q\\é".into();
    unit.file = "a b/#%?:@\\é.rs".into();
    let text = report.to_turtle().expect("awkward terms");
    assert!(text.contains("%C3%A9") && text.contains("\\\"") && text.contains("\\\\"));
    assert_eq!(
        Index::from_turtle(&text).expect("reads escaped terms"),
        report
    );
    assert_eq!(text, include_str!("fixtures/reports/awkward.ttl"));
}

#[test]
fn two_cfg_alternatives_on_one_line_have_distinct_unit_nodes() {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='cfgs'\nversion='0.1.0'\nedition='2024'\n",
        ),
        (
            "src/lib.rs",
            "#[cfg(a)] pub fn f() {} #[cfg(b)] pub fn f() {}",
        ),
    ]);
    let census = Census::load(&tree, &["."], &CensusOptions::default()).expect("cfg alternatives");
    let report = census.index(&[]).expect("index");
    assert_eq!(report.units.len(), 2);
    assert_eq!(report.units[0].symbol, report.units[1].symbol);
    assert_eq!(report.units[0].line, report.units[1].line);
    assert_ne!(report.units[0].start_byte, report.units[1].start_byte);
    assert_eq!(
        Index::from_turtle(&report.to_turtle().expect("renders")).expect("reads"),
        report
    );
}

#[test]
fn the_writer_refuses_a_value_outside_the_shape() {
    let mut baseline = builtin_census().baseline();
    baseline.body_groups[0].members.truncate(1);
    assert!(matches!(
        baseline.to_turtle(),
        Err(Error::Report(ReportError::Missing(_)))
    ));
}

const JOB_SOURCE: &str = "\
pub fn home(a: u32) -> u32 { a + a + a + a + a + a + a + a + a + a + a }
pub fn entry(b: u32) -> u32 { b + b + b + b + b + b + b + b + b + b + b }
pub fn variant(c: u32) -> u32 { c + c + c + c + c + c + c + c + c + c + c }
pub fn copy(d: u32) -> u32 { d + d + d + d + d + d + d + d + d + d + d }
pub const BIG: u32 = 65536;
pub const OTHER: u32 = 7;
";

#[test]
fn the_index_renders_the_classified_matches_of_every_job() {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        ("src/lib.rs", JOB_SOURCE),
    ]
    .into_iter()
    .collect();
    let census = Census::load(&tree, &["."], &CensusOptions::default()).expect("loads");
    let fingerprint = census.groups()[0].fingerprint.clone();
    let job = Job::new("sum", "demo::home")
        .entry_points(["demo::entry"])
        .variants(["demo::variant"])
        .forbidding(JobForbidden {
            fingerprints: vec![fingerprint],
            constants: vec![0x1_0000],
        });
    let idle = Job::new("idle", "demo::home");
    let index = census
        .index(&census.match_jobs(&[job, idle]))
        .expect("unique jobs");
    assert_eq!(index.jobs.len(), 2);
    let sum = &index.jobs[1];
    let classes: Vec<(&str, MatchClass)> = sum
        .matches
        .iter()
        .map(|m| (m.symbol.as_str(), m.class))
        .collect();
    assert_eq!(
        classes,
        [
            ("demo::BIG", MatchClass::Copy),
            ("demo::copy", MatchClass::Copy),
            ("demo::entry", MatchClass::Home),
            ("demo::home", MatchClass::Home),
            ("demo::variant", MatchClass::Variant),
        ]
    );
    assert_eq!(sum.copies, 2);
    assert!(index.jobs[0].matches.is_empty() && index.jobs[0].copies == 0);
    let text = index.to_turtle().expect("renders");
    assert_eq!(Index::from_turtle(&text).expect("reads"), index);
    assert!(text.contains(&format!(
        "<{}matchClass> <{}Variant>",
        vocab::REPORT,
        vocab::REPORT
    )));
}

#[test]
fn a_shared_constant_is_an_integer() {
    let manifest = |name: &str| {
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
    };
    let signed_max = u128::try_from(i128::MAX).expect("a positive maximum");
    let mut last = None;
    for expected in [
        0x1_0000_0000_0000_0000_0000,
        signed_max,
        signed_max + 1,
        u128::MAX,
    ] {
        let tree: MemoryTree = [
            (
                "Cargo.toml",
                "[workspace]\nmembers = [\"a\", \"b\"]\nresolver = \"2\"\n".to_owned(),
            ),
            ("a/Cargo.toml", manifest("a")),
            ("a/src/lib.rs", format!("pub const X: u128 = {expected};\n")),
            ("b/Cargo.toml", manifest("b")),
            ("b/src/lib.rs", format!("pub const Y: u128 = {expected};\n")),
        ]
        .iter()
        .map(|(path, text)| (*path, text.as_str()))
        .collect();
        let census = Census::load(&tree, &["."], &CensusOptions::default()).expect("loads");
        let baseline = census.baseline();
        assert_eq!(baseline.shared_constants.len(), 1);
        let row = &baseline.shared_constants[0];
        assert_eq!(row.value, expected);
        assert_eq!(row.packages, 2);
        let text = baseline.to_turtle().expect("renders");
        assert_eq!(Baseline::from_turtle(&text).expect("reads"), baseline);
        let nt = baseline.to_ntriples().expect("N-Triples renders");
        assert_eq!(
            Baseline::from_ntriples(&nt).expect("N-Triples reads"),
            baseline
        );
        last = Some(text);
    }
    let overflow = rewrite(&last.expect("four width fixtures"), |mut t| {
        if t.predicate == property("value") {
            t.object =
                TermValue::typed_literal("340282366920938463463374607431768211456", vocab::INTEGER);
        }
        Some(t)
    });
    refused(Baseline::from_turtle(&overflow), "value");
}

#[test]
fn two_fresh_loads_of_the_fixtures_give_identical_bytes() {
    assert_eq!(
        builtin::builtin().expect("first"),
        builtin::builtin().expect("second")
    );
    let tree = builtin::tree();
    assert_eq!(
        builtin::of(&tree, ".").expect("first"),
        builtin::of(&tree, ".").expect("second")
    );
}

#[test]
fn two_fresh_loads_of_this_repository_give_identical_bytes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let run = || {
        let tree = DiskTree::new(&root).expect("the repository root opens");
        builtin::of(&tree, "crates").expect("the crates load and render")
    };
    let first = run();
    assert!(first.contains(&format!("<{}normalForm> \"1\"", vocab::REPORT)));
    identical_report_bytes(&first, &run());
}

fn identical_report_bytes(first: &str, second: &str) {
    let byte = first
        .bytes()
        .zip(second.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or(first.len().min(second.len()));
    assert!(
        first == second,
        "report bytes differ at byte {byte}; lengths {} and {}",
        first.len(),
        second.len()
    );
}

#[test]
fn an_absolute_directory_reports_the_same_as_the_relative_spelling() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = root.canonicalize().expect("the repository root exists");
    let absolute = builtin::of_dir(root.join("crates").to_str().expect("utf-8 path"))
        .expect("the absolute spelling reports");
    let tree = DiskTree::new(&root).expect("the repository root opens");
    identical_report_bytes(&absolute, &builtin::of(&tree, "crates").expect("relative"));
}
