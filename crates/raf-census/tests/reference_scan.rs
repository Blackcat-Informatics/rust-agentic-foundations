// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reference packages supply evidence without entering first-party structural debt.

use raf_census::{Gate, Index, MatchClass, Query, QueryReport, ReferenceRoot};
use raf_rdf::{Builder, Format, Graph, TermValue, Triple, vocab};
use raf_source::{RelPath, Tree as _};
#[path = "support/reference.rs"]
mod reference;
use reference::fixture;

#[test]
fn path_references_preserve_all_witnesses_with_one_physical_consumer_count() {
    let tree = fixture(
        "pub fn copy(value:u32)->u32 { let doubled=value*2; doubled+11 }\npub fn neighbor(value:u32)->u32 { let doubled=value*3; doubled+11 }",
        "pub mod api { pub fn first(value:u32)->u32 { let doubled=value*2; doubled+11 } pub fn second(input:u32)->u32 { let result=input*2; result+11 } }",
    );
    let report = Gate::workspace(".").check_tree(&tree).unwrap();
    assert_eq!(report.census().units().len(), 2);
    assert_eq!(report.census().groups(), []);
    assert!(report.debt().groups().is_empty());
    let job = &report.jobs()[0];
    assert_eq!(job.copies, 1);
    assert_eq!(
        job.matches
            .iter()
            .filter(|row| row.class == MatchClass::Home)
            .count(),
        2
    );
    assert_eq!(
        job.matches
            .iter()
            .filter(|row| row.class == MatchClass::Copy)
            .count(),
        1
    );
    assert!(
        !job.matches
            .iter()
            .any(|row| row.symbol.ends_with("::neighbor"))
    );
    let index = report.census().index(report.jobs()).unwrap();
    assert_eq!(index.reference_sources.len(), 1);
    assert_eq!(index.reference_witnesses.len(), 2);
    let encoded = index.to_ntriples().unwrap();
    let decoded = Index::from_ntriples(&encoded).unwrap();
    assert_eq!(decoded.to_ntriples().unwrap(), encoded);
    witness_limits_require_complete_evidence(&tree, &decoded);
    unitless_references_keep_a_complete_index(&tree);
    reference_names_must_belong_to_the_owning_graph(&tree);
    direct_parent_path_keeps_the_same_reference_evidence(&tree, &decoded);
    reject_unrelated_report_evidence(&decoded);
    for query in [
        Query::Job("arithmetic"),
        Query::Count("arithmetic"),
        Query::Fingerprint("app::copy"),
    ] {
        let selected = decoded.query(query).unwrap().to_ntriples().unwrap();
        assert_eq!(
            QueryReport::from_ntriples(&selected)
                .unwrap()
                .to_ntriples()
                .unwrap(),
            selected
        );
    }
}

fn witness_limits_require_complete_evidence(tree: &raf_source::MemoryTree, expected: &Index) {
    use purrdf_jsonschema::ecma::MatchLimits;
    for (limits, cause) in [
        (
            MatchLimits {
                steps: 0,
                ..MatchLimits::default()
            },
            "matching step limit",
        ),
        (
            MatchLimits {
                steps: 1024,
                ..MatchLimits::default()
            },
            "reference-witness-storage",
        ),
        (
            MatchLimits {
                states: 1,
                ..MatchLimits::default()
            },
            "witness-state limit",
        ),
    ] {
        let error = Gate::workspace(".")
            .matching_limits(limits)
            .check_tree(tree)
            .unwrap_err();
        assert!(error.to_string().contains(cause), "{error}");
    }
    let complete = Gate::workspace(".")
        .matching_limits(MatchLimits {
            states: 2,
            ..MatchLimits::default()
        })
        .check_tree(tree)
        .unwrap();
    assert_eq!(complete.census().index(complete.jobs()).unwrap(), *expected);
}

fn unitless_references_keep_a_complete_index(tree: &raf_source::MemoryTree) {
    let mut unitless = tree.clone();
    unitless
        .replace(&RelPath::new("helper/src/lib.rs").unwrap(), String::new())
        .unwrap();
    let ledger = RelPath::new("helpers-ledger.ttl").unwrap();
    let original =
        Graph::from_text(&unitless.read(&ledger, 1_048_576).unwrap(), Format::Turtle).unwrap();
    let home = original
        .assertions()
        .find(|triple| triple.predicate == TermValue::iri(format!("{}homeSymbol", vocab::REPORT)))
        .unwrap();
    let mut policy = Builder::from_graph(&original);
    assert!(policy.remove(home.clone()).unwrap());
    policy
        .insert(Triple {
            object: TermValue::simple_literal("app::copy"),
            ..home
        })
        .unwrap();
    unitless
        .replace(
            &ledger,
            policy.finish().unwrap().to_text(Format::Turtle).unwrap(),
        )
        .unwrap();
    let report = Gate::workspace(".").check_tree(&unitless).unwrap();
    let index = report.census().index(report.jobs()).unwrap();
    assert_eq!(index.units.len(), 2);
    assert_eq!(index.jobs[0].home, "app::copy");
    assert_eq!(index.reference_sources, []);
    assert_eq!(index.reference_witnesses, []);
    let text = index.to_ntriples().unwrap();
    assert_eq!(Index::from_ntriples(&text).unwrap(), index);
}

fn direct_parent_path_keeps_the_same_reference_evidence(
    tree: &raf_source::MemoryTree,
    expected: &Index,
) {
    let mut direct = tree.clone();
    direct.replace(&RelPath::new("app/Cargo.toml").unwrap(), "[package]\nname='app'\nversion.workspace=true\nedition='2024'\n[dependencies]\nrenamed={package='helper',path='../helper',version='1'}\n".into()).unwrap();
    let report = Gate::workspace(".").check_tree(&direct).unwrap();
    assert_eq!(report.census().index(report.jobs()).unwrap(), *expected);
}

fn reference_names_must_belong_to_the_owning_graph(tree: &raf_source::MemoryTree) {
    let mut missing = tree.clone();
    let path = RelPath::new("helpers-ledger.ttl").unwrap();
    let text = tree.read(&path, 1_048_576).unwrap();
    let original = Graph::from_text(&text, Format::Turtle).unwrap();
    let reference = original
        .assertions()
        .find(|triple| triple.predicate == TermValue::iri(format!("{}reference", vocab::REPORT)))
        .unwrap();
    let mut altered = Builder::from_graph(&original);
    assert!(altered.remove(reference.clone()).unwrap());
    altered
        .insert(Triple {
            object: TermValue::simple_literal("missing"),
            ..reference
        })
        .unwrap();
    missing
        .replace(
            &path,
            altered.finish().unwrap().to_text(Format::Turtle).unwrap(),
        )
        .unwrap();
    let error = Gate::workspace(".").check_tree(&missing).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("reference \"missing\" is absent from the owning dependency graph"),
        "{error}"
    );
}

fn hardlinked_path_packages_keep_their_own_directory(root: &std::path::Path) {
    let ledger = std::fs::read_to_string(root.join("helpers-ledger.ttl")).unwrap();
    let original = Graph::from_text(&ledger, Format::Turtle).unwrap();
    let old_root = original
        .assertions()
        .find(|triple| triple.predicate == TermValue::iri(format!("{}root", vocab::REPORT)))
        .unwrap();
    let mut policy = Builder::from_graph(&original);
    assert!(policy.remove(old_root.clone()).unwrap());
    for directory in ["app", "helper"] {
        policy
            .insert(Triple {
                object: TermValue::simple_literal(directory),
                ..old_root.clone()
            })
            .unwrap();
    }
    std::fs::write(
        root.join("helpers-ledger.ttl"),
        policy.finish().unwrap().to_text(Format::Turtle).unwrap(),
    )
    .unwrap();
    std::fs::write(root.join("Cargo.toml"), "[workspace]\nmembers=['app','helper','alternative']\n[workspace.package]\nversion='1.0.0'\n[workspace.dependencies]\nrenamed={package='helper',path='alternative',version='1'}\n").unwrap();
    std::fs::create_dir_all(root.join("alternative/src")).unwrap();
    std::fs::hard_link(
        root.join("helper/Cargo.toml"),
        root.join("alternative/Cargo.toml"),
    )
    .unwrap();
    std::fs::write(
        root.join("alternative/src/lib.rs"),
        "pub mod api {pub fn first(value:u32)->u32{value+99}}\n",
    )
    .unwrap();
    let error = Gate::workspace(root).check().unwrap_err();
    assert!(
        error.to_string().contains("selects a different manifest"),
        "{error}"
    );
}

#[test]
fn reference_shim_matches_keep_every_witness_without_duplicate_copy_counts() {
    let tree = fixture(
        "pub fn shim(value:u32)->u32 { std::convert::identity(value) }",
        "pub mod api { pub fn first(value:u32)->u32 { std::convert::identity(value) } pub fn second(input:u32)->u32 { std::convert::identity(input) } }",
    );
    let report = Gate::workspace(".").check_tree(&tree).unwrap();
    assert_eq!(report.jobs()[0].copies, 1);
    assert_eq!(report.census().groups(), []);
    assert!(report.debt().groups().is_empty());
    let index = report.census().index(report.jobs()).unwrap();
    assert_eq!(
        index
            .reference_witnesses
            .iter()
            .filter(|row| matches!(row.fingerprint, raf_census::Fingerprint::Shim(_)))
            .count(),
        2
    );
    let text = index.to_ntriples().unwrap();
    assert_eq!(Index::from_ntriples(&text).unwrap(), index);
}

#[test]
fn explicit_path_roots_require_the_actual_owning_manifest_locator() {
    let tree = fixture(
        "pub fn copy(value:u32)->u32 { let doubled=value*2; doubled+11 }",
        "pub mod api { pub fn first(value:u32)->u32 { let doubled=value*2; doubled+11 } }",
    );
    let parent =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.stage/reference-sources");
    let owned = purrdf_testkit::TempDir::new_in(&parent).unwrap();
    let root = owned.path();
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "app/Cargo.toml",
        "app/src/lib.rs",
        "helper/Cargo.toml",
        "helper/src/lib.rs",
        "helpers-ledger.ttl",
    ] {
        let target = root.join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(
            target,
            tree.read(&RelPath::new(path).unwrap(), 1_048_576).unwrap(),
        )
        .unwrap();
    }
    let declared = ReferenceRoot::new("helper", root, RelPath::new("helper/Cargo.toml").unwrap());
    let report = Gate::workspace(root)
        .reference_roots([declared.clone()])
        .check()
        .unwrap();
    let plain = Gate::workspace(root).check().unwrap();
    assert_eq!(
        report.census().index(report.jobs()).unwrap(),
        plain.census().index(plain.jobs()).unwrap()
    );
    let opaque = Gate::workspace(".")
        .reference_roots([declared])
        .check_tree(&tree)
        .unwrap_err();
    assert!(opaque.to_string().contains("opaque Tree"), "{opaque}");
    let other = purrdf_testkit::TempDir::new_in(&parent).unwrap();
    std::fs::write(
        other.path().join("Cargo.toml"),
        "[package]\nname='helper'\nversion='1.0.0'\nedition='2024'\n",
    )
    .unwrap();
    let mismatched =
        ReferenceRoot::new("helper", other.path(), RelPath::new("Cargo.toml").unwrap());
    let error = Gate::workspace(root)
        .reference_roots([mismatched])
        .check()
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("differs from the actual owning declaration"),
        "{error}"
    );
    hardlinked_path_packages_keep_their_own_directory(root);
}

fn reject_unrelated_report_evidence(index: &Index) {
    let valid = index.to_ntriples().unwrap();
    let original = Graph::from_text(&valid, Format::NTriples).unwrap();
    reject_historical_reference_provenance(&original);
    let reference_unit = original
        .nodes()
        .find(|(_, node)| {
            node.values(&format!("{}referenceSource", vocab::REPORT))
                .next()
                .is_some()
                && node
                    .values(&format!("{}package", vocab::REPORT))
                    .next()
                    .is_some()
        })
        .unwrap()
        .0;
    let old = original
        .assertions()
        .find(|triple| {
            triple.subject == TermValue::iri(&reference_unit)
                && triple.predicate == TermValue::iri(format!("{}package", vocab::REPORT))
        })
        .unwrap();
    let mut altered = Builder::from_graph(&original);
    assert!(altered.remove(old.clone()).unwrap());
    altered
        .insert(Triple {
            object: TermValue::simple_literal("other"),
            ..old
        })
        .unwrap();
    let text = altered.finish().unwrap().to_text(Format::NTriples).unwrap();
    assert!(
        Index::from_ntriples(&text)
            .unwrap_err()
            .to_string()
            .contains("package differs")
    );
    for (manifest, cause) in [
        ("/Cargo.toml", "absolute"),
        ("helper/../Cargo.toml", "without parent traversal"),
    ] {
        let old = original
            .assertions()
            .find(|triple| triple.predicate == TermValue::iri(format!("{}manifest", vocab::REPORT)))
            .unwrap();
        let mut altered = Builder::from_graph(&original);
        assert!(altered.remove(old.clone()).unwrap());
        altered
            .insert(Triple {
                object: TermValue::simple_literal(manifest),
                ..old
            })
            .unwrap();
        let text = altered.finish().unwrap().to_text(Format::NTriples).unwrap();
        let error = Index::from_ntriples(&text).unwrap_err();
        assert!(error.to_string().contains(cause), "{manifest}: {error}");
    }
    let neighbor = index.query(Query::Fingerprint("app::neighbor")).unwrap();
    let selected = index.query(Query::Fingerprint("app::copy")).unwrap();
    let mut unrelated = Builder::from_graph(neighbor.graph());
    for triple in selected
        .graph()
        .assertions()
        .filter(|triple| triple.subject != TermValue::iri(""))
    {
        if !unrelated.contains(triple.clone()).unwrap() {
            unrelated.insert(triple).unwrap();
        }
    }
    let text = unrelated
        .finish()
        .unwrap()
        .to_text(Format::NTriples)
        .unwrap();
    assert!(
        QueryReport::from_ntriples(&text)
            .unwrap_err()
            .to_string()
            .contains("unrelated")
    );
}

fn reject_historical_reference_provenance(current: &Graph) {
    let header = current
        .assertions()
        .find(|triple| {
            triple.subject == TermValue::iri("")
                && triple.predicate == TermValue::iri(format!("{}reportForm", vocab::REPORT))
        })
        .unwrap();
    for form in [3, 4] {
        let mut older = Builder::from_graph(current);
        assert!(older.remove(header.clone()).unwrap());
        older
            .insert(Triple {
                object: TermValue::integer(form),
                ..header.clone()
            })
            .unwrap();
        let older = older.finish().unwrap();
        for format in [Format::Turtle, Format::NTriples] {
            let text = older.to_text(format).unwrap();
            let error = raf_census::upgrade_report_form(
                raf_census::ReportKind::Index,
                raf_census::MigrationInput {
                    path: "historical-reference-provenance",
                    text: &text,
                },
                format,
            )
            .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("older report forms cannot carry reference"),
                "{form} {format:?}: {error}"
            );
        }
    }
}
