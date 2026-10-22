// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The baseline and index reports: canonical Turtle 1.2 over the built-in workspace of
//! the `baseline` example, the round trip, the closed schema, the job
//! classification, and byte-identical output from two fresh loads, on fixtures
//! and on this repository's own `crates`.

use std::path::Path;

use raf_census::{
    Baseline, Census, CensusOptions, Error, Fingerprint, Index, Job, JobForbidden, MatchClass,
    ReportError,
};
use raf_source::{DiskTree, MemoryTree};

#[path = "support/builtin.rs"]
mod builtin;

/// The output of `cargo run -p raf-census --example baseline`. A change to a
/// fingerprint here is a change of the normal form.
const BUILTIN: &str = include_str!("fixtures/reports/example.txt");

fn builtin_census() -> Census {
    Census::load(&builtin::tree(), &["."], &CensusOptions::default()).expect("the fixture loads")
}

/// The two reports of the built-in workspace, as separate texts.
fn halves() -> (String, String) {
    let text = builtin::builtin().expect("renders");
    let (baseline, index) = text
        .strip_prefix("baseline\n")
        .and_then(|rest| rest.split_once("index\n"))
        .expect("two headed reports");
    (baseline.to_owned(), index.to_owned())
}

#[test]
fn the_example_output_is_the_pinned_golden() {
    assert_eq!(builtin::builtin().expect("renders"), BUILTIN);
}

#[test]
fn the_example_output_names_the_expected_group_fingerprints() {
    let output = builtin::builtin().expect("renders");
    let census = builtin_census();
    let body: Vec<_> = census.body_groups().collect();
    let shim: Vec<_> = census.shim_groups().collect();
    assert_eq!((body.len(), shim.len()), (1, 1));
    assert!(matches!(body[0].fingerprint, Fingerprint::Body(_)));
    assert!(matches!(shim[0].fingerprint, Fingerprint::Shim(_)));
    assert!(output.contains(&body[0].fingerprint.to_string()));
    assert!(output.contains(&shim[0].fingerprint.to_string()));
    assert!(
        output.contains("body:6bb022e241a79a281e80d6e233c0b8f61a027761c52bd4b0c8b232de92544609"),
        "{output}"
    );
    assert!(
        output.contains("shim:203616cb54a3e53de4ca5100ce3cf2aad3b59e9421fc50c09bc93deb7c974725"),
        "{output}"
    );
    assert!(output.contains("census:jobId \"sum\""), "{output}");
}

/// The capability document shows the reports the example prints, byte for
/// byte, so its keys and fingerprints are the ones the code writes.
#[test]
fn the_capability_examples_hold_the_example_output() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/capabilities/census-normal-form.md");
    let document = std::fs::read_to_string(&path).expect("the capability document reads");
    let (baseline, index) = halves();
    for (name, report) in [("baseline", baseline), ("index", index)] {
        let block = format!("```turtle\n{report}```");
        assert!(
            document.contains(&block),
            "the Examples lack the {name} report"
        );
    }
}

#[test]
fn both_reports_read_back_what_they_write() {
    let (baseline, index) = halves();
    let b = Baseline::from_turtle(&baseline).expect("a baseline");
    assert_eq!(b.to_turtle().expect("renders"), baseline);
    assert_eq!(b, builtin_census().baseline());
    let i = Index::from_turtle(&index).expect("an index");
    assert_eq!(i.to_turtle().expect("renders"), index);
    assert_eq!((b.units, i.units.len()), (7, 7));
    assert_eq!((b.body_groups.len(), b.shim_groups.len()), (1, 1));
    assert_eq!(b.hex_tables.len(), 1);
    assert_eq!(b.normal_form, 1);
    assert_eq!(b.min_tokens, 20);
}

fn refused(result: Result<impl std::fmt::Debug, Error>, needle: &str) {
    match result {
        Err(Error::Report(reason)) => assert!(reason.to_string().contains(needle), "{reason}"),
        other => panic!("expected report refusal naming {needle}, got {other:?}"),
    }
}

#[test]
fn output_is_the_canonical_turtle() {
    for text in [halves().0, halves().1] {
        assert!(text.starts_with("VERSION \"1.2\"\nBASE <"));
        assert!(text.ends_with(" .\n") && !text.ends_with("\n\n"));
        assert!(!text.contains('\r') && !text.contains('\t') && !text.contains("_:"));
        assert!(text.lines().all(|line| !line.ends_with(' ')));
        let body = text
            .split_inclusive('\n')
            .enumerate()
            .filter_map(|(i, line)| (i != 1).then_some(line))
            .collect::<String>();
        assert!(
            text.lines().nth(1).expect("BASE").contains(
                purrdf_hash::hex::Lower(purrdf_hash::blake3::hash(body.as_bytes()).as_bytes())
                    .to_string()
                    .as_str()
            )
        );
    }
}

#[test]
fn unknown_terms_are_not_a_report() {
    let (baseline, index) = halves();
    assert!(Baseline::from_turtle(&baseline).is_ok());
    assert!(Index::from_turtle(&index).is_ok());
    for (from, to, needle) in [
        (
            "census:minTokens 20 ;",
            "census:extra 1 ;\n    census:minTokens 20 ;",
            "extra",
        ),
        (
            "census:symbol \"demo::alpha\"",
            "census:column 1 ;\n    census:symbol \"demo::alpha\"",
            "column",
        ),
        ("a census:Unit", "a census:Widget", "Widget"),
        (
            "census:minTokens 20",
            "<https://foreign.example/key> 20",
            "foreign.example",
        ),
    ] {
        refused(
            Baseline::from_turtle(&baseline.replacen(from, to, 1)),
            needle,
        );
    }
    refused(
        Index::from_turtle(&index.replacen("census:homeSymbol", "census:owner", 1)),
        "owner",
    );
}

#[test]
fn decoded_control_characters_are_escaped_in_report_diagnostics() {
    let (baseline, _) = halves();
    assert!(Baseline::from_turtle(&baseline).is_ok());
    let edited = baseline.replacen("census:minTokens", "<https://foreign.example/\\u001B>", 1);
    let error = Baseline::from_turtle(&edited)
        .expect_err("foreign predicate")
        .to_string();
    assert!(!error.contains('\u{1b}'));
    assert!(error.contains("\\u{1b}"));
}

#[test]
fn missing_repeated_and_mistyped_properties_are_not_a_report() {
    let (baseline, index) = halves();
    assert!(Baseline::from_turtle(&baseline).is_ok());
    assert!(Index::from_turtle(&index).is_ok());
    refused(
        Baseline::from_turtle(&baseline.replace("    census:minTokens 20 ;\n", "")),
        "minTokens",
    );
    refused(
        Baseline::from_turtle(
            &baseline.replace("census:normalForm 1", "census:normalForm \"one\""),
        ),
        "normalForm",
    );
    refused(
        Index::from_turtle(&index.replacen(
            "census:body body:",
            "census:body body:0000000000000000000000000000000000000000000000000000000000000000 ;\n    census:body body:",
            1,
        )),
        "body",
    );
    refused(
        Index::from_turtle(&index.replace(
            "census:matchClass census:Copy",
            "census:matchClass census:Duplicate",
        )),
        "matchClass",
    );
    refused(
        Index::from_turtle(&index.replace("census:copies 2", "census:copies 3")),
        "copies",
    );
}

#[test]
fn names_that_disagree_with_their_node_are_not_a_report() {
    let (baseline, _) = halves();
    assert!(Baseline::from_turtle(&baseline).is_ok());
    refused(
        Baseline::from_turtle(&baseline.replacen("census:line 1 ;", "census:line 9 ;", 1)),
        "node name",
    );
    refused(
        Baseline::from_turtle(&baseline.replace("/nf/1/", "/nf/2/")),
        "normalForm",
    );
    let missing = baseline.rfind("\n<#unit/").expect("unit block");
    refused(Baseline::from_turtle(&baseline[..missing]), "dangling");
}

#[test]
fn orphan_units_and_matches_without_a_named_reifier_are_not_a_report() {
    let (baseline, index) = halves();
    assert!(Baseline::from_turtle(&baseline).is_ok());
    assert!(Index::from_turtle(&index).is_ok());
    refused(
        Baseline::from_turtle(&baseline.replace("    census:holds table:hex-lower ;\n", "")),
        "orphan",
    );
    refused(
        Baseline::from_turtle(&baseline.replacen("a census:BodyGroup", "a census:ShimGroup", 1)),
        "fingerprint",
    );
    let missing = index.replacen(" ~ <#match/sum/demo::BIG@src/lib.rs:7:335> {| a census:Match ; census:matchClass census:Copy |}","",1).replace("census:copies 2","census:copies 1");
    refused(Index::from_turtle(&missing), "exactly one named reifier");
}

#[test]
fn constructs_outside_the_profile_are_not_a_report() {
    let (baseline, _) = halves();
    assert!(Baseline::from_turtle(&baseline).is_ok());
    for construct in [
        "_:b",
        "[]",
        "()",
        "# comment",
        "\"\"\"x\"\"\"",
        "'x'",
        "<<( <> census:p <> )>>",
        "{| census:p 1 |}",
    ] {
        refused(
            Baseline::from_turtle(&baseline.replacen("<> a", &format!("{construct} a"), 1)),
            "",
        );
    }
    for version in ["1.1", "1.2-basic"] {
        refused(
            Baseline::from_turtle(&baseline.replacen("1.2", version, 1)),
            "VERSION",
        );
    }
    refused(
        Baseline::from_turtle(baseline.strip_prefix("VERSION \"1.2\"\n").expect("header")),
        "VERSION",
    );
    refused(
        Baseline::from_turtle(&baseline.replacen("PREFIX", "@prefix", 1)),
        "",
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
    assert!(Baseline::from_turtle(&baseline).is_ok());
    assert!(matches!(
        Baseline::from_turtle(&baseline.replace("    census:minTokens", "     census:minTokens")),
        Err(Error::Report(ReportError::NotCanonical { .. }))
    ));
    assert!(matches!(
        Baseline::from_turtle(&baseline.replace("census:minTokens 20", "census:minTokens 21")),
        Err(Error::Report(ReportError::Digest { .. }))
    ));
}

#[test]
fn canonical_line_diagnostics_preserve_crlf_and_the_final_newline() {
    let (baseline, _) = halves();
    assert!(Baseline::from_turtle(&baseline).is_ok());
    assert!(matches!(
        Baseline::from_turtle(&baseline.replace('\n', "\r\n")),
        Err(Error::Report(ReportError::NotCanonical { line: 1 }))
    ));
    let last = baseline.lines().count();
    assert!(
        matches!(Baseline::from_turtle(baseline.strip_suffix('\n').expect("final newline")),Err(Error::Report(ReportError::NotCanonical{line})) if line==last)
    );
}

#[test]
fn an_index_is_not_a_baseline() {
    let (baseline, index) = halves();
    refused(Baseline::from_turtle(&index), "expected Baseline");
    refused(Index::from_turtle(&baseline), "expected Index");
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
    assert!(Baseline::from_turtle(&baseline).is_ok());
    assert!(matches!(
        Baseline::from_turtle(&baseline.replace("census:reportForm 1", "census:reportForm 2")),
        Err(Error::Report(ReportError::Form {
            declared: 2,
            implemented: 1
        }))
    ));
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
    assert!(text.contains("census:matchClass census:Variant"));
}

#[test]
fn a_shared_constant_is_an_integer() {
    let manifest = |name: &str| {
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
    };
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"a\", \"b\"]\nresolver = \"2\"\n".to_owned(),
        ),
        ("a/Cargo.toml", manifest("a")),
        (
            "a/src/lib.rs",
            "pub const X: u128 = 0x1_0000_0000_0000_0000_0000;\n".to_owned(),
        ),
        ("b/Cargo.toml", manifest("b")),
        (
            "b/src/lib.rs",
            "pub const Y: u128 = 0x1_0000_0000_0000_0000_0000;\n".to_owned(),
        ),
    ]
    .iter()
    .map(|(path, text)| (*path, text.as_str()))
    .collect();
    let census = Census::load(&tree, &["."], &CensusOptions::default()).expect("loads");
    let baseline = census.baseline();
    assert_eq!(baseline.shared_constants.len(), 1);
    let row = &baseline.shared_constants[0];
    assert_eq!(row.value, 0x1_0000_0000_0000_0000_0000);
    assert_eq!(row.packages, 2);
    let text = baseline.to_turtle().expect("renders");
    assert_eq!(Baseline::from_turtle(&text).expect("reads"), baseline);
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
    assert!(first.contains("census:normalForm 1"));
    assert_eq!(first, run());
}

#[test]
fn an_absolute_directory_reports_the_same_as_the_relative_spelling() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = root.canonicalize().expect("the repository root exists");
    let absolute = builtin::of_dir(root.join("crates").to_str().expect("utf-8 path"))
        .expect("the absolute spelling reports");
    let tree = DiskTree::new(&root).expect("the repository root opens");
    assert_eq!(absolute, builtin::of(&tree, "crates").expect("relative"));
}
