// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The baseline and index reports: exact JSON over the built-in workspace of
//! the `baseline` example, the round trip, the closed schema, the job
//! classification, and byte-identical output from two fresh loads, on fixtures
//! and on this repository's own `crates`.

use std::path::Path;

use raf_census::{
    Baseline, Census, CensusOptions, Error, Fingerprint, Index, Job, JobForbidden, MatchClass,
};
use raf_source::{DiskTree, MemoryTree};
use serde_json::Value;

#[path = "support/builtin.rs"]
mod builtin;

/// The output of `cargo run -p raf-census --example baseline`. A change to a
/// fingerprint here is a change of the normal form.
const BUILTIN: &str = r#"baseline
{
  "body_groups": [
    {
      "fingerprint": "body:6bb022e241a79a28",
      "members": [
        {
          "file": "src/lib.rs",
          "line": 1,
          "symbol": "demo::alpha"
        },
        {
          "file": "src/lib.rs",
          "line": 2,
          "symbol": "demo::beta"
        }
      ],
      "tokens": 21
    }
  ],
  "hex_tables": [
    {
      "file": "src/lib.rs",
      "line": 6,
      "symbol": "demo::digits",
      "table": "hex-lower"
    }
  ],
  "min_tokens": 20,
  "normal_form": 1,
  "shared_constants": [],
  "shim_groups": [
    {
      "fingerprint": "shim:203616cb54a3e53d",
      "members": [
        {
          "file": "src/lib.rs",
          "line": 4,
          "symbol": "demo::one"
        },
        {
          "file": "src/lib.rs",
          "line": 5,
          "symbol": "demo::two"
        }
      ],
      "tokens": 6
    }
  ],
  "units": 7
}
index
{
  "groups": [
    {
      "fingerprint": "body:6bb022e241a79a28",
      "kind": "body",
      "members": [
        {
          "file": "src/lib.rs",
          "line": 1,
          "symbol": "demo::alpha"
        },
        {
          "file": "src/lib.rs",
          "line": 2,
          "symbol": "demo::beta"
        }
      ]
    },
    {
      "fingerprint": "shim:203616cb54a3e53d",
      "kind": "shim",
      "members": [
        {
          "file": "src/lib.rs",
          "line": 4,
          "symbol": "demo::one"
        },
        {
          "file": "src/lib.rs",
          "line": 5,
          "symbol": "demo::two"
        }
      ]
    }
  ],
  "jobs": [
    {
      "copies": 2,
      "home": "demo::alpha",
      "id": "sum",
      "matches": [
        {
          "class": "copy",
          "symbol": "demo::BIG"
        },
        {
          "class": "home",
          "symbol": "demo::alpha"
        },
        {
          "class": "copy",
          "symbol": "demo::beta"
        }
      ]
    }
  ],
  "normal_form": 1,
  "units": [
    {
      "body": null,
      "file": "src/lib.rs",
      "line": 7,
      "package": "demo",
      "shim": null,
      "shipping": true,
      "symbol": "demo::BIG",
      "tokens": 0
    },
    {
      "body": "body:6bb022e241a79a28",
      "file": "src/lib.rs",
      "line": 1,
      "package": "demo",
      "shim": null,
      "shipping": true,
      "symbol": "demo::alpha",
      "tokens": 21
    },
    {
      "body": "body:6bb022e241a79a28",
      "file": "src/lib.rs",
      "line": 2,
      "package": "demo",
      "shim": null,
      "shipping": true,
      "symbol": "demo::beta",
      "tokens": 21
    },
    {
      "body": null,
      "file": "src/lib.rs",
      "line": 6,
      "package": "demo",
      "shim": null,
      "shipping": true,
      "symbol": "demo::digits",
      "tokens": 1
    },
    {
      "body": null,
      "file": "src/lib.rs",
      "line": 3,
      "package": "demo",
      "shim": null,
      "shipping": true,
      "symbol": "demo::helper",
      "tokens": 3
    },
    {
      "body": null,
      "file": "src/lib.rs",
      "line": 4,
      "package": "demo",
      "shim": "shim:203616cb54a3e53d",
      "shipping": true,
      "symbol": "demo::one",
      "tokens": 6
    },
    {
      "body": null,
      "file": "src/lib.rs",
      "line": 5,
      "package": "demo",
      "shim": "shim:203616cb54a3e53d",
      "shipping": true,
      "symbol": "demo::two",
      "tokens": 6
    }
  ]
}
"#;

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
    assert!(output.contains("body:6bb022e241a79a28"), "{output}");
    assert!(output.contains("shim:203616cb54a3e53d"), "{output}");
    assert!(output.contains("\"id\": \"sum\""), "{output}");
}

#[test]
fn both_reports_read_back_what_they_write() {
    let (baseline, index) = halves();
    let b = Baseline::from_json(&baseline).expect("a baseline");
    assert_eq!(b.to_json().expect("renders"), baseline);
    assert_eq!(b, builtin_census().baseline());
    let i = Index::from_json(&index).expect("an index");
    assert_eq!(i.to_json().expect("renders"), index);
    assert_eq!((b.units, i.units.len()), (7, 7));
    assert_eq!((b.body_groups.len(), b.shim_groups.len()), (1, 1));
    assert_eq!(b.hex_tables.len(), 1);
    assert_eq!(b.normal_form, 1);
    assert_eq!(b.min_tokens, 20);
}

#[test]
fn output_has_sorted_keys_and_one_trailing_newline() {
    let (baseline, index) = halves();
    for text in [baseline, index] {
        assert!(text.ends_with("}\n") && !text.ends_with("\n\n"));
        let value: Value = serde_json::from_str(&text).expect("json");
        assert_sorted(&value);
    }
}

fn assert_sorted(value: &Value) {
    match value {
        Value::Object(map) => {
            let keys: Vec<&String> = map.keys().collect();
            assert!(keys.windows(2).all(|w| w[0] < w[1]), "{keys:?}");
            map.values().for_each(assert_sorted);
        }
        Value::Array(items) => items.iter().for_each(assert_sorted),
        _ => {}
    }
}

fn refused(result: Result<impl std::fmt::Debug, Error>, needle: &str) {
    match result {
        Err(Error::Report(reason)) => assert!(reason.contains(needle), "{reason}"),
        other => panic!("expected a report error naming {needle}, found {other:?}"),
    }
}

#[test]
fn an_unknown_key_missing_key_or_wrong_type_is_not_a_report() {
    let (baseline, index) = halves();
    // An unknown key at the top and inside nested rows.
    refused(
        Baseline::from_json(&baseline.replacen("{\n", "{\n  \"extra\": 1,\n", 1)),
        "extra",
    );
    refused(
        Baseline::from_json(&baseline.replacen(
            "\"line\"",
            "\"column\": 1,\n          \"line\"",
            1,
        )),
        "column",
    );
    refused(
        Index::from_json(&index.replacen("\"home\"", "\"owner\": \"x\", \"home\"", 1)),
        "owner",
    );
    // A missing key and a wrong type.
    refused(
        Baseline::from_json(&baseline.replacen("\"min_tokens\": 20,", "", 1)),
        "min_tokens",
    );
    refused(
        Baseline::from_json(&baseline.replacen(
            "\"normal_form\": 1",
            "\"normal_form\": \"one\"",
            1,
        )),
        "expected u32",
    );
    refused(
        Index::from_json(&index.replacen("\"copy\"", "\"duplicate\"", 1)),
        "duplicate",
    );
    // `body` and `shim` may be null but must be present.
    assert!(
        Index::from_json(&index).is_ok(),
        "the null-carrying form reads"
    );
    assert!(index.contains("\"body\": null,") && index.contains("\"shim\": null,"));
    refused(
        Index::from_json(&index.replacen("\"body\": null,", "", 1)),
        "body",
    );
    refused(
        Index::from_json(&index.replacen("\"shim\": null,", "", 1)),
        "shim",
    );
    // An index is not a baseline.
    assert!(Baseline::from_json(&index).is_err());
    assert!(Index::from_json(&baseline).is_err());
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
    let index = census.index(&census.match_jobs(&[job, idle]));
    assert_eq!(index.jobs.len(), 2);
    let sum = &index.jobs[0];
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
    assert!(index.jobs[1].matches.is_empty() && index.jobs[1].copies == 0);
    let text = index.to_json().expect("renders");
    assert_eq!(Index::from_json(&text).expect("reads"), index);
    assert!(text.contains("\"class\": \"variant\""));
}

#[test]
fn a_shared_constant_is_hexadecimal_text() {
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
    assert_eq!(row.value, "0x100000000000000000000");
    assert_eq!(row.packages, 2);
    let text = baseline.to_json().expect("renders");
    assert_eq!(Baseline::from_json(&text).expect("reads"), baseline);
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
    assert!(first.contains("\"normal_form\": 1"));
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
