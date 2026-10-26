// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The architecture page states the normal form the code implements: every
//! pass id, level, threshold with its value, value adapter, weak keyword,
//! report member and the encoding id. A constant changed in code without the
//! page fails here.

use std::fs;
use std::path::Path;

use raf_census::{
    DEFAULT_MIN_TOKENS, ENCODING, LITERAL_TIER, Level, MAX_NESTING, MIN_SHIM_TOKENS, NORMAL_FORM,
    PASSES, SHIM_MAX_TOKENS, Slice, Tower, VALUE_ADAPTERS, WEAK_KEYWORDS, manifest,
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

/// Every threshold counts the value form, and the code counts the same.
#[test]
fn the_page_states_that_every_threshold_counts_the_value_form() {
    let page = page();
    let thresholds = section(&page, "Thresholds");
    assert!(
        thresholds.contains("tokens of the value form"),
        "the counting rule is not stated"
    );
    assert!(thresholds.contains("before `kind_literals`"));
    assert!(
        !thresholds.contains("structural tokens"),
        "a threshold is described by the structural count"
    );
    for name in ["DEFAULT_MIN_TOKENS", "LITERAL_TIER"] {
        let row = thresholds
            .lines()
            .find(|line| line.starts_with(&format!("| `{name}` |")))
            .unwrap_or_else(|| panic!("no row for {name}"));
        assert!(row.contains("value tokens"), "{row}");
    }
    // The code: the count is the length of the value level, which is shorter
    // than the structural level when the value form erases something.
    let slice = Slice::new("f.rs", 1, "g(&a.clone())").params(["a"]);
    let tower = Tower::new(&slice).expect("lexes");
    assert_eq!(tower.value_tokens(), tower.form(Level::Value).len());
    assert!(tower.value_tokens() < tower.form(Level::Structural).len());
}

/// The numbering rule the Decision states and the structural form details.
#[test]
fn the_page_states_the_renaming_and_numbering_rules() {
    let page = page();
    assert!(
        section(&page, "Decision").contains("by order of first appearance"),
        "the Decision does not state the numbering"
    );
    let structural = section(&page, "Structural form")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for phrase in [
        "only the names that the body or its signature binds",
        "by order of first appearance",
        "signature order",
        "Every other identifier is the name of an item",
        "never a struct literal",
        "`S { x }`",
    ] {
        assert!(structural.contains(phrase), "the page lacks: {phrase}");
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

#[test]
fn the_page_states_the_closed_shape_of_every_report() {
    let page = page();
    let reports = section(&page, "Reports");
    let expected = raf_census::report::vocab::SHAPES
        .iter()
        .map(|row| {
            let max = if row.max == usize::MAX {
                "unbounded".to_owned()
            } else {
                row.max.to_string()
            };
            let prefix = if row.property == "reifies" {
                "rdf"
            } else {
                "census"
            };
            format!(
                "| {} | `census:{}` | `{}:{}` | {}..{} | {} |",
                row.report, row.class, prefix, row.property, row.min, max, row.object
            )
        })
        .collect::<Vec<_>>();
    let actual = reports
        .lines()
        .filter(|line| {
            line.starts_with("| baseline |")
                || line.starts_with("| index |")
                || line.starts_with("| both |")
                || line.starts_with("| debt |")
                || line.starts_with("| job-query |")
                || line.starts_with("| count-query |")
                || line.starts_with("| fingerprint-query |")
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn the_shape_table_is_the_reader_shape() {
    let rows = raf_census::report::vocab::SHAPES;
    for row in rows {
        assert!(row.min <= row.max);
        assert_eq!(
            rows.iter()
                .filter(|other| other.report == row.report
                    && other.class == row.class
                    && other.property == row.property)
                .count(),
            1
        );
    }
    assert!(
        rows.iter()
            .any(|r| r.class == "Match" && r.property == "reifies" && r.min == 1 && r.max == 1)
    );
}
