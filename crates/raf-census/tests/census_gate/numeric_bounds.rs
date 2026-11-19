// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Numeric limit recognition and classification through the production gate.

use raf_census::{BoundClassification, BoundRole, GateReport};

use super::{Builder, Format, Gate, Ledger, MemoryTree, Term, base, job, put, vocab};

pub(super) fn registry(mut graph: Builder, mode: &str) -> Builder {
    put(&mut graph, "", "numericBounds", Term::iri("#bounds"));
    kind(&mut graph, "#bounds", "NumericBounds");
    put(&mut graph, "#bounds", "mode", Term::simple_literal(mode));
    graph
}

fn kind(graph: &mut Builder, node: &str, class: &str) {
    put(
        graph,
        node,
        "type",
        Term::iri(format!("{}{class}", vocab::REPORT)),
    );
}

fn row(graph: &mut Builder, id: &str, key: &str, class: &str) {
    put(graph, "#bounds", "bound", Term::iri(id));
    kind(graph, id, "NumericBound");
    for (predicate, value) in [
        ("file", "src/lib.rs"),
        ("siteKey", key),
        ("classification", class),
    ] {
        put(graph, id, predicate, Term::simple_literal(value));
    }
}

fn sanction(graph: &mut Builder, id: &str, anchor: &str) {
    put(
        graph,
        id,
        "reason",
        Term::simple_literal("The intake contract sets the quantity."),
    );
    put(graph, id, "anchor", Term::iri(anchor));
}

pub(super) fn vocabulary() -> Builder {
    let mut graph = registry(base(), "strict");
    row(&mut graph, "#one", &"a".repeat(64), "sanctioned");
    sanction(&mut graph, "#one", "urn:raf:code:demo%3A%3Aintake");
    graph
}

fn check(source: &str, graph: Builder, record: Option<&str>) -> GateReport {
    let ledger = graph.finish().unwrap().to_text(Format::Turtle).unwrap();
    let mut entries = vec![
        (
            "Cargo.toml",
            "[package]\nname='demo'\nversion='0.1.0'\nedition='2024'\n",
        ),
        ("src/lib.rs", source),
        ("helpers-ledger.ttl", &ledger),
    ];
    if let Some(record) = record {
        entries.push(("census-debt.ttl", record));
    }
    Gate::workspace(".")
        .check_tree(&MemoryTree::from_iter(entries))
        .unwrap()
}

const SOURCE: &str = r#"
    /// The intake count is fixed by the wire format.
    pub fn intake(mut items: Vec<u8>) {
        if items.len() > MAX_ITEMS { return; }
        let count = items.len();
        let _ = 13 > count;
        items.r#truncate(11); items.truncate(12);
        Vec::r#truncate(&mut items, 26);
        let _ = (0..10).take(14);
        let _ = count.clamp(15, 16);
        let _ = core::cmp::min(count, 17);
        let _ = count + 99;
        let _ = items.len() + 98;
        let _ = count.max(97);
        let _ = 96 > 95;
        assert!(items.len() <= 24);
        dbg!(items.len() + 25);
        Throttle.truncate(65, 66);
        use crate::sink::configure as apply;
        {
            use crate::sink::configure as binding;
            let binding = |_: (usize, usize), _: usize, _: usize| {};
            binding((61, 62), 63, 64);
        }
        apply((91, 92), 18, 93);
        {
            use crate::sink::*;
            configure((81, 82), 19, 83);
        }
        let apply = |_: (usize, usize), _: usize, _: usize| {};
        apply((71, 72), 73, 74);
        Throttle.apply_limit(75);
        Throttle::apply_limit(&Throttle, 27);
        let _ = items.len() > Throttle::MAX_ITEMS;
        {
            struct Throttle;
            impl Throttle { fn apply_limit(&self, index: usize) {} }
            Throttle::apply_limit(&Throttle, 28);
        }
    }
    pub const MAX_ITEMS: usize = 8 + 2;
    pub static CAPACITY: u64 = 20;
    pub const CAPTION: usize = 100;
    pub const binding: usize = 101;
    pub const LIMIT_LABEL: &str = "long";
    mod sink {
        pub fn configure((_, _): (usize, usize), max_items: usize, _: usize) {}
    }
    trait Route { fn adjust(self, input: usize); }
    impl Route for &Throttle { fn adjust(self, input: usize) {} }
    struct Throttle;
    impl Throttle {
        const MAX_ITEMS: usize = 31;
        fn apply_limit(&self, size: usize) {}
        fn truncate(&self, left: usize, right: usize) {}
        fn adjust(self, size: usize) {}
        fn local(&self) { self.r#apply_limit(21); Self::r#apply_limit(self, 22); self.adjust(23); }
    }
"#;

/// Every written occurrence has its own classification, even on the same line.
/// A spelling look-alike, arithmetic, wrong argument slot or shadow does not qualify.
#[test]
fn numeric_inventory_classifies_physical_sites_before_jobs() {
    let observed = check(SOURCE, registry(base(), "strict"), None);
    let sites = observed.numeric_bounds();
    let expected = [
        ("13", BoundRole::Comparison),
        ("11", BoundRole::Limiter),
        ("12", BoundRole::Limiter),
        ("26", BoundRole::Limiter),
        ("14", BoundRole::Limiter),
        ("15", BoundRole::Limiter),
        ("16", BoundRole::Limiter),
        ("17", BoundRole::Limiter),
        ("24", BoundRole::Comparison),
        ("18", BoundRole::SizeArgument),
        ("19", BoundRole::SizeArgument),
        ("27", BoundRole::SizeArgument),
        ("MAX_ITEMS", BoundRole::Named),
        ("CAPACITY", BoundRole::Named),
        ("MAX_ITEMS", BoundRole::Named),
        ("21", BoundRole::SizeArgument),
        ("22", BoundRole::SizeArgument),
    ];
    let starts: Vec<_> = sites.iter().map(|bound| bound.start_byte).collect();
    assert_eq!(
        sites.len(),
        expected.len(),
        "{starts:?}\n{:?}",
        observed.findings()
    );
    for (site, (text, role)) in sites.iter().zip(expected) {
        assert!(SOURCE[site.start_byte..].starts_with(text), "{site:?}");
        let roles = if text == "MAX_ITEMS" {
            vec![role, BoundRole::Comparison]
        } else {
            vec![role]
        };
        assert_eq!(site.roles.iter().copied().collect::<Vec<_>>(), roles);
        assert_eq!(site.classification, None);
    }
    assert_eq!(observed.findings().len(), sites.len());
    assert_ne!(sites[1].key, sites[2].key);
    assert_eq!(sites[1].line, sites[2].line);
    assert_eq!(check(SOURCE, base(), None).numeric_bounds(), []);
    let with_home = check(
        &format!("{SOURCE}\npub use intake as alias;"),
        registry(job(true), "strict"),
        None,
    );
    assert_eq!(with_home.numeric_bounds(), sites);
    assert_eq!(with_home.findings(), observed.findings());

    let classified = |anchor| {
        let mut graph = registry(base(), "strict");
        for (index, site) in sites.iter().enumerate() {
            let id = format!("#bound-{index}");
            let class = match index {
                0 => "sanctioned",
                1 => "experimental",
                _ => "structural",
            };
            row(&mut graph, &id, &site.key.to_hex(), class);
            if class == "sanctioned" {
                sanction(&mut graph, &id, anchor);
            }
        }
        graph
    };
    let anchor = "urn:raf:code:demo%3A%3Aintake";
    let clean = check(SOURCE, classified(anchor), None);
    assert!(clean.is_clean(), "{:?}", clean.findings());
    assert!(matches!(
        clean.numeric_bounds()[0].classification,
        Some(BoundClassification::Sanctioned { .. })
    ));
    let wrong_anchor = check(SOURCE, classified("urn:raf:code:demo%3A%3ACAPACITY"), None);
    assert_eq!(wrong_anchor.findings().len(), 1);
    assert!(wrong_anchor.findings()[0].contains("documented source item"));
    let removed = SOURCE.replace("items.truncate(12)", "items.capacity()  ");
    let stale = check(&removed, classified(anchor), None);
    assert_eq!(stale.findings().len(), 1, "{:?}", stale.findings());
    assert!(stale.findings()[0].contains(&format!(
        "stale numeric bound siteKey {}",
        sites[2].key.to_hex()
    )));
}

/// Counted roles live independently of their exact ledger keys and job exemptions.
#[test]
fn numeric_ratchet_tracks_roles_and_unclassified_multiplicity_across_line_movement() {
    let graph = || registry(base(), "ratcheted");
    let source =
        "pub fn cap(items: &mut Vec<u8>) { if items.len() > 5 { return; } items.truncate(7); }";
    let adopted = check(source, graph(), None);
    assert_eq!(adopted.debt().ratcheted().len(), 3);
    assert!(
        adopted
            .findings()
            .iter()
            .any(|finding| finding.contains("adopt the newly ratcheted rule"))
    );
    let record = adopted.debt().to_turtle().unwrap();
    let moved = check(&format!("\n{source}"), graph(), Some(&record));
    assert!(moved.is_clean(), "{:?}", moved.findings());
    assert_eq!(moved.debt().to_turtle().unwrap(), record);
    assert_ne!(
        moved.numeric_bounds()[0].key,
        adopted.numeric_bounds()[0].key
    );
    assert_eq!(adopted.debt().sites().values().copied().sum::<usize>(), 4);
    let growth = source.replace("truncate(7);", "truncate(7); items.truncate(7);");
    let growth = check(&growth, graph(), Some(&record));
    assert_eq!(growth.findings().len(), 2, "{:?}", growth.findings());
    assert!(
        growth
            .findings()
            .iter()
            .all(|row| row.contains("growth cannot be recorded"))
    );
    let mut classified = graph();
    row(
        &mut classified,
        "#one",
        &adopted.numeric_bounds()[0].key.to_hex(),
        "structural",
    );
    let shrink = check(source, classified, Some(&record));
    assert_eq!(shrink.findings().len(), 1, "{:?}", shrink.findings());
    assert!(shrink.findings()[0].contains("numeric-bound-unclassified"));
    assert_eq!(shrink.debt().sites().values().copied().sum::<usize>(), 3);

    let comparison_only = source.replace("items.truncate(7);", "");
    let no_truncation = check(&comparison_only, graph(), None);
    assert!(
        no_truncation
            .debt()
            .ratcheted()
            .contains("numeric-bound-truncating")
    );
    assert!(
        !no_truncation
            .debt()
            .sites()
            .keys()
            .any(|site| site.rule().as_str() == "numeric-bound-truncating")
    );
    let dormant_record = no_truncation.debt().to_turtle().unwrap();
    let new_truncation = check(source, graph(), Some(&dormant_record));
    assert!(new_truncation.findings().iter().any(|finding| {
        finding.contains("numeric-bound-truncating")
            && finding.contains("growth cannot be recorded")
    }));
}

/// A registry may authorize only one well-formed classification at each physical key.
#[test]
fn numeric_registry_refuses_duplicate_sites_and_incomplete_justification() {
    let parse = |graph: Builder| {
        Ledger::from_turtle(&graph.finish().unwrap().to_text(Format::Turtle).unwrap())
    };
    assert!(parse(vocabulary()).is_ok());
    let mut duplicate = vocabulary();
    row(&mut duplicate, "#again", &"a".repeat(64), "structural");
    assert!(
        parse(duplicate)
            .unwrap_err()
            .to_string()
            .contains("numeric bound")
    );
    let mut ungrounded = registry(base(), "strict");
    row(&mut ungrounded, "#one", &"a".repeat(64), "sanctioned");
    put(
        &mut ungrounded,
        "#one",
        "reason",
        Term::simple_literal("Policy limit"),
    );
    assert!(
        parse(ungrounded)
            .unwrap_err()
            .to_string()
            .contains("anchor")
    );
}

/// A shared source literal counts once, while named operands retain every proved declaration.
#[test]
fn numeric_size_arguments_require_agreement_at_shared_physical_sites() {
    let ledger = registry(base(), "strict")
        .finish()
        .unwrap()
        .to_text(Format::Turtle)
        .unwrap();
    let check = |right| {
        let tree = MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='demo'\nversion='0.1.0'\nedition='2024'\n",
            ),
            ("src/lib.rs", "mod left; mod right;"),
            (
                "src/left.rs",
                "#[path=\"shared.rs\"] mod shared; fn configure(size: usize) {} const MAX_ITEMS: usize = 3;",
            ),
            ("src/right.rs", right),
            (
                "src/shared.rs",
                "pub fn intake(count: usize) { let _ = count > super::MAX_ITEMS; super::configure(67); }",
            ),
            ("helpers-ledger.ttl", ledger.as_str()),
        ]);
        Gate::workspace(".").check_tree(&tree)
    };
    let agreed = check("#[path=\"shared.rs\"] mod shared; fn configure(limit: usize) {} const MAX_ITEMS: usize = 7;").unwrap();
    assert_eq!(agreed.numeric_bounds().len(), 3, "{:?}", agreed.findings());
    assert_eq!(
        agreed
            .numeric_bounds()
            .iter()
            .filter(|site| site.roles.contains(&BoundRole::Comparison))
            .count(),
        2
    );
    let conflict =
        check("#[path=\"shared.rs\"] mod shared; fn configure(index: usize) {} const MAX_ITEMS: usize = 7;").unwrap_err();
    assert!(
        conflict
            .to_string()
            .contains("changes across source definitions"),
        "{conflict}"
    );
}

/// Retained argument slots spend the matching grant even without call hits.
#[test]
fn numeric_inventory_refuses_a_signature_beyond_the_state_grant() {
    let source = "pub fn configure(a: usize, b: usize, c: usize, d: usize, size: usize) {}";
    let check = |source, states| {
        Gate::workspace(".")
            .matching_limits(purrdf_jsonschema::ecma::MatchLimits {
                states,
                ..Default::default()
            })
            .check_tree(&super::tree(source, registry(base(), "strict"), 0))
    };
    let refused = check(source, 5).unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("numeric bounds inventory state limit exhausted"),
        "{refused}"
    );
    assert!(check(source, 6).unwrap().is_clean());
    let ordinary = source.replace("size:", "label:");
    assert!(check(&ordinary, 5).unwrap().is_clean());
}
