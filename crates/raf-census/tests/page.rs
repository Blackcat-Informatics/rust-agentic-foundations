// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The architecture page states the normal form the code implements: every
//! pass id, level, threshold with its value, value adapter, weak keyword,
//! report member and the encoding id. A constant changed in code without the
//! page fails here.

use std::fs;
use std::path::Path;

use raf_census::{
    Baseline, DEFAULT_MIN_TOKENS, ENCODING, Index, LITERAL_TIER, Level, MAX_NESTING,
    MIN_SHIM_TOKENS, NORMAL_FORM, PASSES, SHIM_MAX_TOKENS, VALUE_ADAPTERS, WEAK_KEYWORDS, manifest,
};

fn page() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/architecture/normal-form.md");
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The lines of the section `heading` (a `## ` title) of `page`.
fn section<'a>(page: &'a str, heading: &str) -> &'a str {
    let start = page
        .find(&format!("\n## {heading}\n"))
        .unwrap_or_else(|| panic!("the page has no section {heading}"));
    let rest = &page[start + 1..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |at| at + 3);
    &rest[..end]
}

fn quoted(name: &str) -> String {
    format!("`{name}`")
}

#[test]
fn the_page_names_every_pass_level_and_the_encoding() {
    let page = page();
    let passes = section(&page, "The passes");
    let mut last = 0;
    for pass in PASSES {
        let at = passes
            .find(&quoted(pass))
            .unwrap_or_else(|| panic!("the passes section does not name {pass}"));
        assert!(at >= last || last == 0, "{pass} is out of order");
        last = at;
    }
    let ordered = format!(
        "`{}`, `{}`, `{}` and `{}`",
        PASSES[0], PASSES[1], PASSES[2], PASSES[3]
    );
    assert!(passes.contains(&ordered), "the pass order is not stated");
    for level in Level::ALL {
        assert!(
            passes.contains(&format!("| `{}` |", level.id())),
            "no row for the level {level}"
        );
    }
    assert!(
        section(&page, "Fingerprint").contains(&quoted(ENCODING)),
        "the encoding id is not on the page"
    );
    assert!(page.contains(&format!("`normal_form = {NORMAL_FORM}`")));
}

#[test]
fn the_page_names_every_threshold_with_its_value() {
    let page = page();
    let thresholds = section(&page, "Thresholds");
    for (name, value) in [
        ("DEFAULT_MIN_TOKENS", DEFAULT_MIN_TOKENS),
        ("LITERAL_TIER", LITERAL_TIER),
        ("MIN_SHIM_TOKENS", MIN_SHIM_TOKENS),
        ("SHIM_MAX_TOKENS", SHIM_MAX_TOKENS),
        ("MAX_NESTING", MAX_NESTING),
    ] {
        let row = format!("| `{name}` | {value} |");
        assert!(thresholds.contains(&row), "the table lacks {row}");
    }
    // The manifest the goldens pin carries the same values.
    let manifest = manifest();
    for (key, value) in [
        ("literal_tier", LITERAL_TIER),
        ("min_shim_tokens", MIN_SHIM_TOKENS),
        ("shim_max_tokens", SHIM_MAX_TOKENS),
        ("max_nesting", MAX_NESTING),
    ] {
        assert!(manifest.contains(&format!("{key} {value}\n")), "{key}");
    }
}

#[test]
fn the_page_names_every_value_adapter_and_weak_keyword() {
    let page = page();
    let value = section(&page, "Value form");
    for adapter in VALUE_ADAPTERS {
        assert!(value.contains(&quoted(adapter)), "adapter {adapter}");
    }
    let structural = section(&page, "Structural form");
    for word in WEAK_KEYWORDS {
        assert!(structural.contains(&quoted(word)), "keyword {word}");
    }
}

/// The members the page must name for the baseline report, by object.
const BASELINE: &[&str] = &[
    "normal_form",
    "min_tokens",
    "units",
    "body_groups",
    "shim_groups",
    "hex_tables",
    "shared_constants",
    "fingerprint",
    "tokens",
    "members",
    "symbol",
    "file",
    "line",
    "table",
    "value",
    "packages",
];

/// The members the page must name for the index report, by object.
const INDEX: &[&str] = &[
    "normal_form",
    "units",
    "symbol",
    "package",
    "file",
    "line",
    "shipping",
    "tokens",
    "body",
    "shim",
    "groups",
    "kind",
    "fingerprint",
    "members",
    "jobs",
    "id",
    "home",
    "matches",
    "class",
    "copies",
];

#[test]
fn the_page_names_every_report_member_of_both_reports() {
    let page = page();
    let reports = section(&page, "Reports");
    let (baseline, index) = reports
        .split_once("The index report")
        .expect("the section describes the baseline report, then the index report");
    for member in BASELINE {
        assert!(baseline.contains(&quoted(member)), "baseline: {member}");
    }
    for member in INDEX {
        assert!(index.contains(&quoted(member)), "index: {member}");
    }
}

/// A report member list is the closed schema: a document with exactly these
/// members reads, and one with another member does not.
#[test]
fn the_member_lists_are_the_closed_schemas() {
    let baseline = r#"{"normal_form":1,"min_tokens":20,"units":0,"body_groups":[
        {"fingerprint":"body:0000000000000000","tokens":1,"members":
        [{"symbol":"s","file":"f","line":1}]}],"shim_groups":[],
        "hex_tables":[{"symbol":"s","file":"f","line":1,"table":"hex-lower"}],
        "shared_constants":[{"value":"0x10000","packages":2,"members":
        [{"symbol":"s","file":"f","line":1}]}]}"#;
    Baseline::from_json(baseline).expect("every named baseline member reads");
    let extra = baseline.replacen("\"units\":0", "\"units\":0,\"extra\":1", 1);
    assert!(Baseline::from_json(&extra).is_err());
    let index = r#"{"normal_form":1,"units":[{"symbol":"s","package":"p","file":"f",
        "line":1,"shipping":true,"tokens":1,"body":null,"shim":null}],
        "groups":[{"kind":"body","fingerprint":"body:0000000000000000","members":
        [{"symbol":"s","file":"f","line":1}]}],
        "jobs":[{"id":"j","home":"s","matches":[{"symbol":"s","class":"home"}],
        "copies":0}]}"#;
    Index::from_json(index).expect("every named index member reads");
    let extra = index.replacen("\"copies\":0", "\"copies\":0,\"extra\":1", 1);
    assert!(Index::from_json(&extra).is_err());
}
