// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reference packages supply evidence without entering first-party structural debt.

use raf_census::{ExternalReference, Gate, Index, MatchClass, Query, QueryReport, ReferenceRoot};
use raf_fixtures::files;
use raf_fixtures::git_repo;
use raf_rdf::{Builder, Format, Graph, TermValue, Triple, vocab};
use raf_source::{RelPath, Tree as _};
use std::fmt::Write as _;
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
    // The job is not enforced: its classified copy is counted job debt.
    assert_eq!(report.debt().job_copies().get("arithmetic"), Some(&1));
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

/// A ledger that declares the given `reference` selectors and no job, so no
/// reference unit has a home.
fn unhomed_ledger(selectors: &[&str]) -> String {
    use raf_rdf::TermValue as Term;
    let mut ledger = Builder::new();
    let property = |name: &str| Term::iri(format!("{}{name}", vocab::REPORT));
    let integer = |value: u32| Term::integer(i128::from(value));
    for (predicate, object) in [
        (
            Term::iri(vocab::TYPE),
            Term::iri(format!("{}Ledger", vocab::REPORT)),
        ),
        (property("normalForm"), integer(raf_census::NORMAL_FORM)),
        (property("reportForm"), integer(raf_census::REPORT_FORM)),
        (property("minTokens"), Term::integer(1)),
        (property("root"), Term::simple_literal(".")),
    ]
    .into_iter()
    .chain(
        selectors
            .iter()
            .map(|selector| (property("reference"), Term::simple_literal(*selector))),
    ) {
        ledger
            .insert(Triple {
                subject: Term::iri(""),
                predicate,
                object,
            })
            .unwrap();
    }
    ledger.finish().unwrap().to_text(Format::Turtle).unwrap()
}

/// A Cargo workspace `ws` whose `app` depends on the member `acme-member`,
/// which every `acme-*` family also names, and on each named package under
/// `vendor/`, beside the workspace and outside its first-party packages. The
/// ledger declares the given reference selectors and no job.
fn family_fixture(selectors: &[&str], vendors: &[&str], consumer: &str) -> purrdf_testkit::TempDir {
    workspace_fixture(selectors, "acme-member", vendors, consumer)
}

/// As [`family_fixture`] with a chosen member. The member and each vendor are
/// `name` or `name@version`; the version defaults to 1.0.0, and a name may
/// occur at two versions.
fn workspace_fixture(
    selectors: &[&str],
    member: &str,
    vendors: &[&str],
    consumer: &str,
) -> purrdf_testkit::TempDir {
    let spec = |text: &str| {
        let (name, version) = text.split_once('@').unwrap_or((text, "1.0.0"));
        (name.to_owned(), version.to_owned())
    };
    let (member, member_version) = spec(member);
    let parent =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.stage/reference-sources");
    std::fs::create_dir_all(&parent).unwrap();
    let owned = purrdf_testkit::TempDir::new_in(&parent).unwrap();
    let mut files = vec![
        (
            "ws/Cargo.toml".to_owned(),
            format!("[workspace]\nmembers=['app','{member}']\n"),
        ),
        (
            format!("ws/{member}/Cargo.toml"),
            format!("[package]\nname='{member}'\nversion='{member_version}'\nedition='2024'\n"),
        ),
        (
            format!("ws/{member}/src/lib.rs"),
            "pub fn member(value:u32)->u32 { let tripled=value*3; tripled+5 }".into(),
        ),
        ("ws/app/src/lib.rs".into(), consumer.into()),
        ("ws/helpers-ledger.ttl".into(), unhomed_ledger(selectors)),
    ];
    let mut manifest = format!(
        "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nmember={{package='{member}',path='../{member}',version='{member_version}'}}\n"
    );
    let mut lock = format!("version=4\n[[package]]\nname='{member}'\nversion='{member_version}'\n");
    let mut app = format!(
        "[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['{member} {member_version}'"
    );
    for (index, vendor) in vendors.iter().enumerate() {
        let (name, version) = spec(vendor);
        let factor = index + 1;
        writeln!(
            manifest,
            "dep{index}={{package='{name}',path='../../vendor/{name}-{version}',version='{version}'}}"
        )
        .unwrap();
        write!(lock, "[[package]]\nname='{name}'\nversion='{version}'\n").unwrap();
        write!(app, ",'{name} {version}'").unwrap();
        files.push((
            format!("vendor/{name}-{version}/Cargo.toml"),
            format!("[package]\nname='{name}'\nversion='{version}'\nedition='2024'\n"),
        ));
        files.push((
            format!("vendor/{name}-{version}/src/lib.rs"),
            format!("pub fn provided(value:u32)->u32 {{ let scaled=value*{factor}; scaled+7 }}"),
        ));
    }
    app.push_str("]\n");
    files.push(("ws/app/Cargo.toml".into(), manifest));
    files.push(("ws/Cargo.lock".into(), lock + &app));
    for (path, text) in files {
        let target = owned.path().join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, text).unwrap();
    }
    owned
}

/// The packages whose source the reference scan read, as sorted `(name, version)`.
fn scanned_versions(fixture: &purrdf_testkit::TempDir) -> Vec<(String, String)> {
    let report = Gate::workspace(fixture.path().join("ws")).check().unwrap();
    let index = report.census().index(report.jobs()).unwrap();
    let mut packages: Vec<_> = index
        .reference_sources
        .iter()
        .map(|source| (source.package.clone(), source.version.clone()))
        .collect();
    packages.sort();
    packages
}

/// The names of the packages whose source the reference scan read.
fn scanned(fixture: &purrdf_testkit::TempDir) -> Vec<String> {
    scanned_versions(fixture)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

/// A family selector names every locked dependency package whose name
/// continues its prefix after the hyphen, so each one's source is read for
/// comparison. The boundary holds both ways: the bare prefix `acme`, the
/// hyphen-less `acmeish` and the other family `other-x` are reachable and
/// stay unscanned, a longer name `acme-a-extra` is scanned, and a first-party
/// member that matches stays a consumer rather than a reference.
#[test]
fn family_selector_scans_every_matching_dependency_but_no_member() {
    let fixture = family_fixture(
        &["acme-*"],
        &[
            "acme-a",
            "acme-b",
            "acme-c",
            "acme-a-extra",
            "acme",
            "acmeish",
            "other-x",
        ],
        "pub fn local(value:u32)->u32 { value }",
    );
    let scanned = scanned(&fixture);
    for absent in ["acme", "acmeish", "other-x"] {
        assert!(
            !scanned.iter().any(|name| name == absent),
            "{absent}: {scanned:?}"
        );
    }
    assert_eq!(scanned, ["acme-a", "acme-a-extra", "acme-b", "acme-c"]);
    let report = Gate::workspace(fixture.path().join("ws")).check().unwrap();
    assert!(
        report
            .census()
            .units()
            .iter()
            .any(|unit| unit.package == "acme-member" && unit.symbol == "acme_member::member"),
        "{:?}",
        report.census().units()
    );
}

/// A package newly added to the lock graph under the family's prefix joins
/// the scan from the same ledger bytes: the family cannot silently narrow.
#[test]
fn family_selector_follows_lock_growth_without_a_ledger_edit() {
    let three = family_fixture(&["acme-*"], &["acme-a", "acme-b", "acme-c"], "");
    let four = family_fixture(&["acme-*"], &["acme-a", "acme-b", "acme-c", "acme-d"], "");
    let ledger = |fixture: &purrdf_testkit::TempDir| {
        std::fs::read(fixture.path().join("ws/helpers-ledger.ttl")).unwrap()
    };
    assert_eq!(ledger(&three), ledger(&four));
    assert_eq!(scanned(&four), ["acme-a", "acme-b", "acme-c", "acme-d"]);
}

/// A family that names no locked dependency refuses and names itself; a
/// first-party member alone cannot satisfy a family, and a value with `*`
/// anywhere but after a prefix and its hyphen is not a selector.
#[test]
fn family_selector_matching_nothing_refuses_naming_the_selector() {
    const EMPTY: &str = "matches no locked dependency package outside the workspace members";
    const MALFORMED: &str = "a family selector is a package-name prefix followed by `-*`";
    // `acme-member` continues the prefix `acme-` but is first-party, so it
    // cannot satisfy the family by itself.
    for (selector, vendors, cause) in [
        ("other-*", &["acme-a"][..], EMPTY),
        ("acme-*", &["other-x"][..], EMPTY),
        ("acme*", &["acme-a"][..], MALFORMED),
        ("zq-*-x", &["acme-a"][..], MALFORMED),
    ] {
        let fixture = family_fixture(&[selector], vendors, "");
        let error = Gate::workspace(fixture.path().join("ws"))
            .check()
            .unwrap_err()
            .to_string();
        assert!(
            error.contains(cause) && error.contains(selector),
            "{selector}: {error}"
        );
    }
}

/// An exact name and a family that both select a package scan it once, with
/// one source receipt and one witness per matching reference unit.
#[test]
fn exact_and_family_selectors_scan_a_package_once() {
    let copy = "pub fn copy(value:u32)->u32 { let scaled=value*2; scaled+7 }";
    let family = family_fixture(&["acme-*"], &["acme-a", "acme-b"], copy);
    let both = family_fixture(&["acme-*", "acme-b"], &["acme-a", "acme-b"], copy);
    assert_eq!(scanned(&both), ["acme-a", "acme-b"]);
    let report = Gate::workspace(both.path().join("ws")).check().unwrap();
    let index = report.census().index(report.jobs()).unwrap();
    assert_eq!(index.reference_witnesses.len(), 1);
    let alone = Gate::workspace(family.path().join("ws")).check().unwrap();
    assert_eq!(report.findings(), alone.findings());
}

/// A reference that no job names as a home bans copies: a first-party body
/// equal to a body in any package of the family is a non-debt
/// `REFERENCE-COPY` finding, while a changed body stays clean.
#[test]
fn unhomed_reference_copy_is_a_hard_finding() {
    let vendors = ["acme-a", "acme-b", "acme-c"];
    let banned = family_fixture(
        &["acme-*"],
        &vendors,
        "pub fn copy(input:u32)->u32 { let result=input*2; result+7 }",
    );
    let report = Gate::workspace(banned.path().join("ws")).check().unwrap();
    assert!(!report.is_clean());
    assert_eq!(report.findings().len(), 1, "{:?}", report.findings());
    let finding = &report.findings()[0];
    assert!(
        finding.starts_with("REFERENCE-COPY app/src/lib.rs:1 app::copy matches acme_b::provided "),
        "{finding}"
    );
    assert!(report.debt().groups().is_empty());
    assert!(report.debt().job_copies().is_empty());
    // Debt writing refuses the finding and records nothing.
    let refused = Gate::workspace(banned.path().join("ws"))
        .write_debt()
        .unwrap_err();
    assert!(
        matches!(&refused, raf_census::Error::Findings(rows) if rows == report.findings()),
        "{refused}"
    );
    assert!(
        !banned
            .path()
            .join("ws")
            .join(raf_census::DEBT_FILE)
            .exists()
    );
    let neighbor = family_fixture(
        &["acme-*"],
        &vendors,
        "pub fn copy(input:u32)->u32 { let result=input*2; result+8 }",
    );
    assert!(
        Gate::workspace(neighbor.path().join("ws"))
            .check()
            .unwrap()
            .is_clean()
    );
    Gate::workspace(neighbor.path().join("ws"))
        .write_debt()
        .unwrap();
}

/// An adopted debt record does not hold the ban: a workspace that already has a
/// record, written while it was clean, still fails when a copy is introduced,
/// and the refused write leaves the record byte for byte as it was.
#[test]
fn an_existing_debt_record_does_not_hold_a_reference_copy() {
    let vendors = ["acme-a", "acme-b", "acme-c"];
    let fixture = family_fixture(
        &["acme-*"],
        &vendors,
        "pub fn copy(input:u32)->u32 { let result=input*2; result+8 }",
    );
    let root = fixture.path().join("ws");
    Gate::workspace(&root).write_debt().unwrap();
    let record = std::fs::read(root.join(raf_census::DEBT_FILE)).unwrap();
    std::fs::write(
        root.join("app/src/lib.rs"),
        "pub fn copy(input:u32)->u32 { let result=input*2; result+7 }",
    )
    .unwrap();
    let report = Gate::workspace(&root).check().unwrap();
    assert!(!report.is_clean());
    assert!(
        report
            .findings()
            .iter()
            .any(|finding| finding.starts_with("REFERENCE-COPY app/src/lib.rs:1 app::copy ")),
        "{:?}",
        report.findings()
    );
    Gate::workspace(&root).write_debt().unwrap_err();
    assert_eq!(
        std::fs::read(root.join(raf_census::DEBT_FILE)).unwrap(),
        record
    );
}

/// The ban is the declaration `reference` with no job homing the dependency:
/// a copy of a body, of a shim, and of a package that is also a workspace
/// member is a non-debt finding, and only the copy is flagged (the member's
/// own units are not). A job-less ledger evaluates; it does not refuse.
#[test]
fn unhomed_reference_bans_shim_and_member_copies_beside_changed_neighbours() {
    let library = "pub mod api { pub fn first(value:u32)->u32 { let doubled=value*2; doubled+11 } pub fn wrap(value:u32)->u32 { std::convert::identity(value) } }";
    let banned = |consumer: &str| {
        let mut tree = fixture(consumer, library);
        tree.replace(
            &RelPath::new("helpers-ledger.ttl").unwrap(),
            unhomed_ledger(&["helper"]),
        )
        .unwrap();
        Gate::workspace(".").check_tree(&tree).unwrap()
    };
    let body = banned("pub fn copy(input:u32)->u32 { let result=input*2; result+11 }");
    let shim = banned("pub fn shim(input:u32)->u32 { std::convert::identity(input) }");
    for (report, symbol, kind, count) in [
        (&body, "app::copy", "body", 1),
        (&shim, "app::shim", "shim", 2),
    ] {
        assert!(!report.is_clean(), "{kind}");
        // A shim body is both a body and a shim fingerprint, so it matches twice.
        assert_eq!(
            report.findings().len(),
            count,
            "{kind}: {:?}",
            report.findings()
        );
        for finding in report.findings() {
            assert!(
                finding.starts_with(&format!(
                    "REFERENCE-COPY app/src/lib.rs:1 {symbol} matches helper::api::"
                )),
                "{kind}: {finding}"
            );
        }
        assert!(report.debt().groups().is_empty() && report.debt().job_copies().is_empty());
    }
    let neighbour = banned("pub fn copy(input:u32)->u32 { let result=input*2; result+12 }");
    assert!(neighbour.is_clean(), "{:?}", neighbour.findings());
}

/// A sealed Git dependency selected by the lock file is scanned under
/// `check_locked` exactly as a path dependency is: its committed body copied
/// into the consumer is a hard finding, a changed body is clean, and the
/// source tree is read from its committed objects, never from the working tree,
/// although the fixture repository keeps its working tree on disk.
#[test]
fn locked_git_reference_copy_is_a_hard_finding() {
    use raf_source::{
        CommittedGitTree, LoadOptions, LockedCatalog, LockedEdge, LockedManifest,
        LockedPackageRoots, LockedTree, MemoryTree,
    };
    let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let repository = scratch.path().join("kit");
    git_repo::init(&repository);
    files::write(
        &repository,
        &[
            (
                "Cargo.toml",
                "[package]\nname='kit'\nversion='1.0.0'\nedition='2024'\n",
            ),
            (
                "src/lib.rs",
                "pub fn provided(value:u32)->u32 { let scaled=value*4; scaled+7 }",
            ),
        ],
    );
    let commit = git_repo::commit_all(&repository);
    let source = format!("git+https://example.test/kit?rev={commit}#{commit}");
    let lock = format!(
        "version=4\n[[package]]\nname='kit'\nversion='1.0.0'\nsource={source:?}\n[[package]]\nname='demo'\nversion='1.0.0'\ndependencies=[{:?}]\n",
        format!("kit 1.0.0 ({source})")
    );
    let options = LoadOptions::default();
    let catalog = LockedCatalog::load(
        &MemoryTree::from([("Cargo.lock", lock.as_str())]),
        &RelPath::new("Cargo.lock").unwrap(),
        &options,
    )
    .unwrap();
    let row = |name: &str| {
        catalog
            .packages()
            .find(|row| row.name() == name)
            .unwrap()
            .id()
    };
    let git = CommittedGitTree::open(&catalog, row("kit"), &repository, &options).unwrap();
    let manifest = format!(
        "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nkit={{git='https://example.test/kit',rev='{commit}'}}\n"
    );
    let ledger = unhomed_ledger(&["kit"]);
    let check = |consumer: &str| {
        let tree = MemoryTree::from([
            ("Cargo.toml", manifest.as_str()),
            ("src/lib.rs", consumer),
            ("Cargo.lock", lock.as_str()),
            ("helpers-ledger.ttl", ledger.as_str()),
        ]);
        let edges = [LockedEdge::new("dependencies.kit", None)];
        let local = [LockedManifest::new(
            row("demo"),
            RelPath::new("Cargo.toml").unwrap(),
            &edges,
        )];
        let kit = [LockedManifest::new(
            row("kit"),
            RelPath::new("Cargo.toml").unwrap(),
            &[],
        )];
        let sources = [LockedPackageRoots::new(LockedTree::Git(&git), &kit)];
        Gate::workspace(".")
            .check_locked(&tree, &catalog, &local, &sources)
            .unwrap()
    };
    let copied = check("pub fn copy(input:u32)->u32 { let result=input*4; result+7 }");
    assert_eq!(copied.findings().len(), 1, "{:?}", copied.findings());
    assert!(
        copied.findings()[0]
            .starts_with("REFERENCE-COPY src/lib.rs:1 demo::copy matches kit::provided "),
        "{:?}",
        copied.findings()
    );
    assert!(copied.debt().groups().is_empty() && copied.debt().job_copies().is_empty());
    let changed = check("pub fn copy(input:u32)->u32 { let result=input*4; result+8 }");
    assert!(changed.is_clean(), "{:?}", changed.findings());
}

/// A family excludes the member lock rows only. A member and an external
/// package may share a name at different versions, and the family still scans
/// the external one; with no external package of that name the member alone
/// selects nothing.
#[test]
fn family_selector_excludes_member_rows_not_every_row_with_a_member_name() {
    let shared = workspace_fixture(
        &["acme-*"],
        "acme-core@2.0.0",
        &["acme-core@1.0.0", "acme-util"],
        "",
    );
    assert_eq!(
        scanned_versions(&shared),
        [
            ("acme-core".to_owned(), "1.0.0".to_owned()),
            ("acme-util".to_owned(), "1.0.0".to_owned())
        ]
    );
    let only_util = workspace_fixture(&["acme-*"], "acme-core@2.0.0", &["acme-util"], "");
    assert_eq!(scanned(&only_util), ["acme-util"]);
}

/// A family whose name has two reachable non-member versions is ambiguous and
/// refuses rather than scanning one of them.
#[test]
fn family_selector_refuses_a_name_with_two_reachable_versions() {
    let ambiguous = workspace_fixture(
        &["acme-*"],
        "acme-member",
        &["acme-core@1.0.0", "acme-core@2.0.0"],
        "",
    );
    let error = Gate::workspace(ambiguous.path().join("ws"))
        .check()
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("\"acme-core\" is ambiguous in the lock graph"),
        "{error}"
    );
    let single = workspace_fixture(&["acme-*"], "acme-member", &["acme-core@2.0.0"], "");
    assert_eq!(
        scanned_versions(&single),
        [("acme-core".to_owned(), "2.0.0".to_owned())]
    );
}

/// A large constant held on both sides of the reference boundary is a
/// cross-boundary shared constant by value, whatever its spelling; a value at
/// or below the threshold, or held only by reference units, is not, and
/// first-party grouping is untouched.
#[test]
fn large_constants_shared_with_a_reference_cross_the_boundary_by_value() {
    let tree = fixture(
        "pub fn mix(seed:u64)->u64 { seed ^ 0x1_0000u32 as u64 ^ 0xffff ^ 0xABCD_1234_5678 }\npub fn other()->u64 { 0x1_0000 }",
        "pub mod api { pub fn hash()->u64 { 65_536 ^ 65535 ^ 0xabcd12345678 } pub fn salt()->u64 { 0xdead_beef_u64 } pub fn pepper()->u64 { 3735928559 } }",
    );
    let report = Gate::workspace(".").check_tree(&tree).unwrap();
    let index = report.census().index(report.jobs()).unwrap();
    let rows: Vec<_> = index
        .reference_constants
        .iter()
        .map(|row| {
            (
                row.value,
                row.first_party
                    .iter()
                    .map(|m| m.symbol.as_str())
                    .collect::<Vec<_>>(),
                row.reference
                    .iter()
                    .map(|m| m.symbol.as_str())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            (
                65_536,
                vec!["app::mix", "app::other"],
                vec!["helper::api::hash"]
            ),
            (
                0xabcd_1234_5678,
                vec!["app::mix"],
                vec!["helper::api::hash"]
            ),
        ]
    );
    // Reference-only values and the threshold itself never appear.
    let encoded = index.to_ntriples().unwrap();
    assert_eq!(Index::from_ntriples(&encoded).unwrap(), index);
    reject_misplaced_constant_members(&encoded);
    reject_writing_misplaced_constant_members(&index);
    reject_historical_constant_rows_alone(&encoded);
}

/// Two first-party packages that share a large constant keep their row in the
/// baseline's shared constants, which holds first-party members only; a value
/// one of them also shares with a reference additionally gets a separate
/// cross-boundary row, and a value only the first-party packages hold gets none.
#[test]
fn first_party_shared_constants_stay_in_the_baseline_beside_the_cross_boundary_rows() {
    let ledger = unhomed_ledger(&["helper"]);
    let tree = raf_source::MemoryTree::from([
        (
            "Cargo.toml",
            "[workspace]\nmembers=['app','other','helper']\n",
        ),
        (
            "app/Cargo.toml",
            "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nhelper={path='../helper'}\n",
        ),
        ("app/src/lib.rs", "pub fn a()->u64 { 0x2_0000 ^ 0x1_0000 }"),
        (
            "other/Cargo.toml",
            "[package]\nname='other'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("other/src/lib.rs", "pub fn b()->u64 { 131072 ^ 65_536 }"),
        (
            "helper/Cargo.toml",
            "[package]\nname='helper'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("helper/src/lib.rs", "pub fn h()->u64 { 65_536 }"),
        (
            "Cargo.lock",
            "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['helper']\n[[package]]\nname='other'\nversion='1.0.0'\n[[package]]\nname='helper'\nversion='1.0.0'\n",
        ),
        ("helpers-ledger.ttl", ledger.as_str()),
    ]);
    let report = Gate::workspace(".").check_tree(&tree).unwrap();
    let baseline = report.census().baseline();
    let shared: Vec<_> = baseline
        .shared_constants
        .iter()
        .map(|row| {
            (
                row.value,
                row.members
                    .iter()
                    .map(|m| m.symbol.as_str())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(
        shared,
        [
            (65_536, vec!["app::a", "other::b"]),
            (131_072, vec!["app::a", "other::b"])
        ]
    );
    assert_eq!(
        constant_rows(&report),
        [(
            65_536,
            vec!["app::a".to_owned(), "other::b".to_owned()],
            vec!["helper::h".to_owned()]
        )]
    );
}

fn reject_misplaced_constant_members(encoded: &str) {
    let original = Graph::from_text(encoded, Format::NTriples).unwrap();
    let first_party = format!("{}firstPartyMember", vocab::REPORT);
    let reference = format!("{}referenceMember", vocab::REPORT);
    let member = |predicate: &str| {
        original
            .assertions()
            .find(|triple| triple.predicate == TermValue::iri(predicate))
            .unwrap()
    };
    // Each side's member is pointed at a unit of the other side, in both
    // directions.
    for (own, other) in [(&first_party, &reference), (&reference, &first_party)] {
        let row = member(own);
        let wrong = member(other);
        let mut swapped = Builder::from_graph(&original);
        assert!(swapped.remove(row.clone()).unwrap());
        swapped
            .insert(Triple {
                object: wrong.object,
                ..row
            })
            .unwrap();
        let text = swapped.finish().unwrap().to_text(Format::NTriples).unwrap();
        let error = Index::from_ntriples(&text).unwrap_err().to_string();
        assert!(
            error.contains("own side of the reference boundary"),
            "{own}: {error}"
        );
    }
}

/// The writer refuses a constant row whose members stand on the wrong side of
/// the reference boundary, as the reader does; the same row on its own sides
/// writes.
fn reject_writing_misplaced_constant_members(index: &Index) {
    let mut misplaced = index.clone();
    let row = &mut misplaced.reference_constants[0];
    std::mem::swap(&mut row.first_party, &mut row.reference);
    let error = misplaced.to_ntriples().unwrap_err().to_string();
    assert!(
        error.contains("own side of the reference boundary"),
        "{error}"
    );
}

/// Reference constant rows alone make an index too new for the older-form
/// conversion: with every reference source and unit receipt stripped, a form 3
/// document that keeps its constant rows refuses, and the same document
/// without them is refused for its dangling unit receipts instead.
fn reject_historical_constant_rows_alone(encoded: &str) {
    let property = |name: &str| TermValue::iri(format!("{}{name}", vocab::REPORT));
    let current = Graph::from_text(encoded, Format::NTriples).unwrap();
    let typed = |name: &str| -> Vec<TermValue> {
        current
            .assertions()
            .filter(|triple| {
                triple.predicate == TermValue::iri(vocab::TYPE) && triple.object == property(name)
            })
            .map(|triple| triple.subject)
            .collect()
    };
    let sources = typed("ReferenceSource");
    let rows = typed("ReferenceConstant");
    assert!(!sources.is_empty() && !rows.is_empty());
    let header = current
        .assertions()
        .find(|triple| {
            triple.subject == TermValue::iri("") && triple.predicate == property("reportForm")
        })
        .unwrap();
    let older = |keep_rows: bool| {
        let mut graph = Builder::from_graph(&current);
        for triple in current.assertions() {
            if sources.contains(&triple.subject)
                || triple.predicate == property("referenceSource")
                || (!keep_rows && rows.contains(&triple.subject))
            {
                graph.remove(triple).unwrap();
            }
        }
        assert!(graph.remove(header.clone()).unwrap());
        graph
            .insert(Triple {
                object: TermValue::integer(3),
                ..header.clone()
            })
            .unwrap();
        graph.finish().unwrap().to_text(Format::NTriples).unwrap()
    };
    let convert = |text: &str| {
        raf_census::upgrade_report_form(
            raf_census::ReportKind::Index,
            raf_census::MigrationInput {
                path: "historical-constant-rows",
                text,
            },
            Format::NTriples,
        )
    };
    let refused = convert(&older(true)).unwrap_err().to_string();
    assert!(
        refused.contains("older report forms cannot carry reference"),
        "{refused}"
    );
    // Without the rows the conversion is no longer refused for carrying
    // reference evidence; the units still name their stripped receipts.
    let neighbour = convert(&older(false)).unwrap_err().to_string();
    assert!(
        !neighbour.contains("older report forms cannot carry reference"),
        "{neighbour}"
    );
}

/// An index with no reference package carries no constant row and no new
/// property, so its bytes are those of the previous layout.
#[test]
fn an_index_without_a_reference_has_no_reference_constant_rows() {
    let tree = raf_source::MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['app']\n"),
        (
            "app/Cargo.toml",
            "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("app/src/lib.rs", "pub fn mix()->u64 { 0x1_0000 }"),
    ]);
    let census =
        raf_census::Census::load(&tree, &["."], &raf_census::CensusOptions::default()).unwrap();
    let text = census.index(&[]).unwrap().to_ntriples().unwrap();
    assert!(!text.contains("ReferenceConstant") && !text.contains("Member"));
}

/// The consumer of the external-reference cases: `copy` repeats the
/// `upstream` body committed first, and `extra` repeats only the body that a
/// later dirty checkout of that repository holds.
const EXTERNAL_CONSUMER: &str = "pub fn copy(value:u32)->u32 { let scaled=value*1; scaled+7 }\npub fn extra(value:u32)->u32 { let scaled=value*9; scaled+7 }";

/// An `upstream` repository at `dir` whose first commit holds the body that
/// `copy` repeats. Its later head holds another body, and its checkout is then
/// edited to the body that `extra` repeats without committing. Returns the
/// first commit and the head.
fn upstream_repository(dir: &std::path::Path) -> (String, String) {
    let library = |factor: u32| {
        format!("pub fn provided(value:u32)->u32 {{ let scaled=value*{factor}; scaled+7 }}")
    };
    git_repo::init(dir);
    files::write(
        dir,
        &[
            (
                "Cargo.toml",
                "[package]\nname='upstream'\nversion='1.0.0'\nedition='2024'\n",
            ),
            ("src/lib.rs", &library(1)),
        ],
    );
    let first = git_repo::commit_all(dir);
    files::write(dir, &[("src/lib.rs", library(2).as_str())]);
    let head = git_repo::commit_all(dir);
    files::write(dir, &[("src/lib.rs", library(9).as_str())]);
    (first, head)
}

fn external_report(ws: &std::path::Path, reference: ExternalReference) -> raf_census::GateReport {
    Gate::workspace(ws)
        .external_references([reference])
        .check()
        .unwrap()
}

/// A pinned commit of a repository outside the lock graph yields the same
/// `REFERENCE-COPY` finding as the same package selected through the lock
/// graph. The declared commit is read, not the head, and never the edited
/// checkout: pinning the head finds nothing although the checkout repeats
/// `extra`.
#[test]
fn pinned_commit_reports_reference_copies_exactly_as_a_lock_selected_reference() {
    let locked = workspace_fixture(
        &["upstream"],
        "acme-member",
        &["upstream"],
        EXTERNAL_CONSUMER,
    );
    let expected = Gate::workspace(locked.path().join("ws"))
        .check()
        .unwrap()
        .findings()
        .to_vec();
    assert_eq!(expected.len(), 1, "{expected:?}");
    assert!(
        expected[0]
            .starts_with("REFERENCE-COPY app/src/lib.rs:1 app::copy matches upstream::provided "),
        "{expected:?}"
    );

    let fixture = workspace_fixture(&[], "acme-member", &[], EXTERNAL_CONSUMER);
    let repository = fixture.path().join("upstream");
    let (first, head) = upstream_repository(&repository);
    let ws = fixture.path().join("ws");
    let pinned = external_report(&ws, ExternalReference::commit(&repository, &first));
    assert_eq!(pinned.findings(), expected);
    let index = pinned.census().index(pinned.jobs()).unwrap();
    let [source] = index.reference_sources.as_slice() else {
        panic!("one receipt: {:?}", index.reference_sources);
    };
    assert_eq!(
        (
            source.package.as_str(),
            source.version.as_str(),
            source.source_kind.as_str(),
            source.repository.as_str(),
            source.revision.as_str(),
            source.observation.as_str(),
        ),
        (
            "upstream",
            "1.0.0",
            "git",
            repository.to_str().unwrap(),
            first.as_str(),
            "committed-git",
        )
    );
    assert!(external_report(&ws, ExternalReference::commit(&repository, &head)).is_clean());
}

/// Each unusable pinned input refuses with a message naming what to fix,
/// beside the exact commit and pin key that succeed: a malformed commit ID, a
/// commit absent from the repository, a missing repository, a location inside
/// the repository rather than its root, a tree with no packages, a pin key
/// that names no commit, one tree declared twice, and a relative location on
/// an opaque Tree, which has no main checkout to resolve it from.
#[test]
fn pinned_reference_refusals_name_the_failed_input_beside_the_exact_commit() {
    let fixture = workspace_fixture(&[], "acme-member", &[], EXTERNAL_CONSUMER);
    let repository = fixture.path().join("upstream");
    let (first, _) = upstream_repository(&repository);
    let empty = fixture.path().join("empty");
    git_repo::init(&empty);
    files::write(&empty, &[("README", "no packages\n")]);
    let packageless = git_repo::commit_all(&empty);
    let ws = fixture.path().join("ws");
    files::write(
        &ws,
        &[(
            "pins.toml",
            format!("[upstream]\nrev = \"{first}\"\nsize = 3\n").as_str(),
        )],
    );
    let pins = |key: &str| {
        ExternalReference::pin_file(&repository, RelPath::new("pins.toml").unwrap(), key)
    };
    assert!(!external_report(&ws, ExternalReference::commit(&repository, &first)).is_clean());
    assert!(!external_report(&ws, pins("upstream.rev")).is_clean());
    for (reference, expected) in [
        (
            ExternalReference::commit(&repository, &first[..12]),
            "full 40-character lowercase hexadecimal SHA-1",
        ),
        (
            ExternalReference::commit(&repository, first.to_uppercase()),
            "full 40-character lowercase hexadecimal SHA-1",
        ),
        (
            ExternalReference::commit(&repository, "0".repeat(40)),
            "Git commit failed",
        ),
        (
            ExternalReference::commit(fixture.path().join("absent"), &first),
            "Git repository failed",
        ),
        (
            ExternalReference::commit(repository.join("src"), &first),
            "name the repository's checkout root or Git directory",
        ),
        (
            ExternalReference::commit(&empty, &packageless),
            "yields no packages; an empty comparison is never reported as clean",
        ),
        (
            pins("upstream.tag"),
            "key \"upstream.tag\": the key is absent",
        ),
        (
            pins("upstream.size"),
            "the value must be a commit ID string",
        ),
    ] {
        let error = Gate::workspace(&ws)
            .external_references([reference])
            .check()
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{expected:?} in {error}");
    }
    let twice = Gate::workspace(&ws)
        .external_references([
            ExternalReference::commit(&repository, &first),
            ExternalReference::commit(&repository, &first),
        ])
        .check()
        .unwrap_err()
        .to_string();
    assert!(twice.contains("is declared twice"), "{twice}");
    let opaque = Gate::workspace(".")
        .external_references([ExternalReference::commit("../upstream", &first)])
        .check_tree(&reference::fixture("", "pub fn provided() {}"))
        .unwrap_err()
        .to_string();
    assert!(opaque.contains("which an opaque Tree lacks"), "{opaque}");
}

/// A relative location is resolved from the main checkout of the consumer's
/// repository, so a linked worktree and the main checkout read the same pinned
/// tree and produce byte-identical indexes. Resolved from the linked
/// worktree's own directory, `../upstream` would name a directory that does
/// not exist.
#[test]
fn linked_worktree_and_main_checkout_compare_against_the_same_pinned_tree() {
    let fixture = workspace_fixture(&[], "acme-member", &[], EXTERNAL_CONSUMER);
    let (first, _) = upstream_repository(&fixture.path().join("upstream"));
    let main = fixture.path().join("ws");
    files::write(
        &main,
        &[(
            "pins.toml",
            format!("[upstream]\nrev = \"{first}\"\n").as_str(),
        )],
    );
    git_repo::init(&main);
    git_repo::commit_all(&main);
    git_repo::git(&main, &["worktree", "add", "--quiet", ".worktrees/linked"]);
    let linked = main.join(".worktrees/linked");
    assert!(!linked.join("../upstream").exists());
    let index = |ws: &std::path::Path, reference: ExternalReference| {
        let report = external_report(ws, reference);
        assert_eq!(report.findings().len(), 1, "{:?}", report.findings());
        report
            .census()
            .index(report.jobs())
            .unwrap()
            .to_ntriples()
            .unwrap()
    };
    let pinned = || {
        ExternalReference::pin_file(
            "../upstream",
            RelPath::new("pins.toml").unwrap(),
            "upstream.rev",
        )
    };
    let expected = index(&main, pinned());
    assert_eq!(index(&linked, pinned()), expected);
    assert_eq!(
        index(&linked, ExternalReference::commit("../upstream", &first)),
        expected
    );
}

/// The unpinned mode reads a directory's current files and finds exactly what
/// the pinned commit of the same tree finds, while every receipt of its native
/// index carries the `unpinned-tree` observation. Its manifest sets no version,
/// so the receipt records Cargo's `0.0.0`. Because a debt record cannot
/// carry that label, an unpinned comparison refuses to record debt, where a
/// pinned one reaches the ordinary findings refusal instead.
#[test]
fn unpinned_directory_finds_the_same_copies_and_labels_every_receipt() {
    let fixture = workspace_fixture(&[], "acme-member", &[], EXTERNAL_CONSUMER);
    let repository = fixture.path().join("upstream");
    let (first, _) = upstream_repository(&repository);
    let plain = fixture.path().join("plain");
    files::write(
        &plain,
        &[
            ("Cargo.toml", "[package]\nname='upstream'\nedition='2024'\n"),
            (
                "src/lib.rs",
                "pub fn provided(value:u32)->u32 { let scaled=value*1; scaled+7 }",
            ),
        ],
    );
    let ws = fixture.path().join("ws");
    let pinned = external_report(&ws, ExternalReference::commit(&repository, &first));
    let unpinned = external_report(&ws, ExternalReference::unpinned(&plain));
    assert!(!pinned.is_clean());
    assert_eq!(unpinned.findings(), pinned.findings());
    let encoded = unpinned
        .census()
        .index(unpinned.jobs())
        .unwrap()
        .to_ntriples()
        .unwrap();
    let index = Index::from_ntriples(&encoded).unwrap();
    assert_eq!(index.reference_sources.len(), 1);
    for source in &index.reference_sources {
        assert_eq!(
            (
                source.observation.as_str(),
                source.source_kind.as_str(),
                source.revision.as_str(),
                source.version.as_str()
            ),
            ("unpinned-tree", "path", "", "0.0.0")
        );
    }
    let refused = Gate::workspace(&ws)
        .external_references([ExternalReference::unpinned(&plain)])
        .write_debt()
        .unwrap_err()
        .to_string();
    assert!(
        refused.contains("an unpinned reference comparison cannot record debt"),
        "{refused}"
    );
    let findings = Gate::workspace(&ws)
        .external_references([ExternalReference::commit(&repository, &first)])
        .write_debt()
        .unwrap_err();
    assert!(
        matches!(findings, raf_census::Error::Findings(_)),
        "{findings}"
    );
    assert!(!ws.join(raf_census::DEBT_FILE).exists());
}

/// The cross-boundary constants of a report as `(value, first-party symbols,
/// reference symbols)`.
fn constant_rows(report: &raf_census::GateReport) -> Vec<(u128, Vec<String>, Vec<String>)> {
    let index = report.census().index(report.jobs()).unwrap();
    let symbols = |rows: &[raf_census::MemberRow]| {
        rows.iter()
            .map(|row| row.symbol.clone())
            .collect::<Vec<_>>()
    };
    index
        .reference_constants
        .iter()
        .map(|row| {
            (
                row.value,
                symbols(&row.first_party),
                symbols(&row.reference),
            )
        })
        .collect()
}

/// A value that only reference units hold never appears, however many
/// reference packages hold it, while a value a first-party unit shares with
/// one of them does.
#[test]
fn a_constant_held_only_by_two_reference_packages_has_no_row() {
    let fixture = family_fixture(
        &["acme-*"],
        &["acme-a", "acme-b"],
        "pub fn local()->u64 { 0x1_0000 }",
    );
    let vendor = |name: &str| fixture.path().join(format!("vendor/{name}-1.0.0"));
    files::write(
        &vendor("acme-a"),
        &[("src/lib.rs", "pub fn a()->u64 { 0xdead_beef ^ 65_536 }")],
    );
    files::write(
        &vendor("acme-b"),
        &[("src/lib.rs", "pub fn b()->u64 { 3735928559 }")],
    );
    let report = Gate::workspace(fixture.path().join("ws")).check().unwrap();
    assert_eq!(
        report
            .census()
            .index(report.jobs())
            .unwrap()
            .reference_sources
            .len(),
        2
    );
    assert_eq!(
        constant_rows(&report),
        [(
            65_536,
            vec!["app::local".to_owned()],
            vec!["acme_a::a".to_owned()]
        )]
    );
}

/// An external reference, pinned or unpinned, yields the same cross-boundary
/// constant rows as the same package selected through the lock graph.
#[test]
fn external_references_report_cross_boundary_constants_like_a_lock_selected_one() {
    const CONSUMER: &str = "pub fn local()->u64 { 0x1_0000u32 as u64 ^ 0xffff }";
    const LIBRARY: &str = "pub fn provided()->u64 { 65_536 ^ 65535 }";
    let locked = workspace_fixture(&["upstream"], "acme-member", &["upstream"], CONSUMER);
    files::write(
        &locked.path().join("vendor/upstream-1.0.0"),
        &[("src/lib.rs", LIBRARY)],
    );
    let expected = constant_rows(&Gate::workspace(locked.path().join("ws")).check().unwrap());
    assert_eq!(
        expected,
        [(
            65_536,
            vec!["app::local".to_owned()],
            vec!["upstream::provided".to_owned()]
        )]
    );

    let fixture = workspace_fixture(&[], "acme-member", &[], CONSUMER);
    let repository = fixture.path().join("upstream");
    git_repo::init(&repository);
    files::write(
        &repository,
        &[
            (
                "Cargo.toml",
                "[package]\nname='upstream'\nversion='1.0.0'\nedition='2024'\n",
            ),
            ("src/lib.rs", LIBRARY),
        ],
    );
    let commit = git_repo::commit_all(&repository);
    let ws = fixture.path().join("ws");
    let pinned = external_report(&ws, ExternalReference::commit(&repository, &commit));
    assert_eq!(constant_rows(&pinned), expected);
    let unpinned = external_report(&ws, ExternalReference::unpinned(&repository));
    assert_eq!(constant_rows(&unpinned), expected);
    let encoded = unpinned
        .census()
        .index(unpinned.jobs())
        .unwrap()
        .to_ntriples()
        .unwrap();
    assert_eq!(
        Index::from_ntriples(&encoded)
            .unwrap()
            .reference_constants
            .len(),
        1
    );
}

const PROVIDED: &str = "pub fn provided(input:u32)->u32 { let result=input*2; result+7 }";

/// A consumer `app` whose registry dependency `acme-core` was stored in a cache
/// directory below the test target area. Returns the consumer tree and the
/// cache; `archive` is what the cache holds, and the lock names the checksum of
/// `locked`, the archive that Cargo would have stored.
fn registry_consumer(
    selector: &str,
    source: &str,
    locked: &[u8],
    archive: Option<&[u8]>,
) -> (raf_source::MemoryTree, purrdf_testkit::TempDir) {
    let cache = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    if let Some(archive) = archive {
        raf_fixtures::registry::place(
            cache.path(),
            "index.crates.io-0000000000000000",
            "acme-core",
            "1.0.0",
            archive,
        );
    }
    let lock = format!(
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['acme-core']\n[[package]]\nname='acme-core'\nversion='1.0.0'\nsource='registry+https://github.com/rust-lang/crates.io-index'\nchecksum='{}'\n",
        raf_fixtures::registry::checksum(locked)
    );
    let ledger = unhomed_ledger(&[selector]);
    let tree = raf_source::MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nacme-core='1'\n",
        ),
        ("src/lib.rs", source),
        ("Cargo.lock", lock.as_str()),
        ("helpers-ledger.ttl", ledger.as_str()),
    ]);
    (tree, cache)
}

fn acme_core(body: &str) -> Vec<u8> {
    raf_fixtures::registry::crate_archive(
        "acme-core",
        "1.0.0",
        &[
            (
                "Cargo.toml",
                "[package]\nname='acme-core'\nversion='1.0.0'\nedition='2024'\n[lib]\nname='acme_core'\npath='src/lib.rs'\n",
            ),
            ("src/lib.rs", body),
        ],
    )
}

fn registry_check(
    tree: &raf_source::MemoryTree,
    cache: &purrdf_testkit::TempDir,
) -> Result<raf_census::GateReport, raf_census::Error> {
    let registry = raf_census::LockedRegistry::new(raf_census::ArchiveCache::at(cache.path()));
    Gate::workspace(".").check_locked_registry(tree, &registry)
}

/// A copied body of a registry dependency, sealed from the lock file's checksum
/// and the cache, is a hard non-debt finding for an exact and a family selector,
/// and a changed body stays clean.
#[test]
fn registry_reference_copy_is_a_hard_finding_beside_a_changed_neighbour() {
    let archive = acme_core(PROVIDED);
    for selector in ["acme-core", "acme-*"] {
        let (tree, cache) = registry_consumer(
            selector,
            "pub fn copy(input:u32)->u32 { let result=input*2; result+7 }",
            &archive,
            Some(&archive),
        );
        let report = registry_check(&tree, &cache).unwrap();
        assert_eq!(report.findings().len(), 1, "{:?}", report.findings());
        assert!(
            report.findings()[0]
                .starts_with("REFERENCE-COPY src/lib.rs:1 app::copy matches acme_core::provided "),
            "{:?}",
            report.findings()
        );
        assert!(report.debt().groups().is_empty() && report.debt().job_copies().is_empty());
        let (neighbor, cache) = registry_consumer(
            selector,
            "pub fn copy(input:u32)->u32 { let result=input*2; result+8 }",
            &archive,
            Some(&archive),
        );
        let clean = registry_check(&neighbor, &cache).unwrap();
        assert!(clean.is_clean(), "{:?}", clean.findings());
    }
}

/// An archive absent from the cache, a cached archive that is not the locked
/// bytes and an oversized one each refuse by name and never pass silently.
#[test]
fn absent_mismatching_and_oversized_registry_archives_refuse() {
    let archive = acme_core(PROVIDED);
    let consumer = "pub fn copy(input:u32)->u32 { let result=input*2; result+8 }";
    let (tree, cache) = registry_consumer("acme-core", consumer, &archive, None);
    let message = registry_check(&tree, &cache).unwrap_err().to_string();
    for expected in [
        "acme-core 1.0.0",
        "acme-core-1.0.0.crate",
        &cache.path().display().to_string(),
        "cargo fetch --locked",
    ] {
        assert!(message.contains(expected), "{expected} missing: {message}");
    }
    let other = acme_core("pub fn provided() {}");
    let (tree, cache) = registry_consumer("acme-core", consumer, &archive, Some(&other));
    let message = registry_check(&tree, &cache).unwrap_err().to_string();
    assert!(
        message.contains("acme-core 1.0.0") && message.contains("checksum"),
        "{message}"
    );
    let (tree, cache) = registry_consumer("acme-core", consumer, &archive, None);
    let index = cache.path().join("index.crates.io-0000000000000000");
    std::fs::create_dir(&index).unwrap();
    // A sparse file one byte over the 256 MiB budget: no large allocation is made.
    std::fs::File::create(index.join("acme-core-1.0.0.crate"))
        .unwrap()
        .set_len((256 << 20) + 1)
        .unwrap();
    let message = registry_check(&tree, &cache).unwrap_err().to_string();
    assert!(message.contains("byte archive budget"), "{message}");
    let (tree, cache) = registry_consumer("acme-core", consumer, &archive, Some(&archive));
    assert!(registry_check(&tree, &cache).unwrap().is_clean());
}

const CRATES_IO: &str = "registry+https://github.com/rust-lang/crates.io-index";
const COPIED: &str = "pub fn copy(input:u32)->u32 { let result=input*2; result+7 }";

/// The check of a consumer made of `files`, whose lock lists `acme-core` from
/// `source` with the checksum of `archive`, with `archive` stored in a cache
/// directory below the test target area. `extra` is more lock rows.
fn registry_scenario(
    files: &[(&str, &str)],
    source: &str,
    archive: &[u8],
    extra: &str,
) -> Result<raf_census::GateReport, raf_census::Error> {
    let cache = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    raf_fixtures::registry::place(cache.path(), "index-0", "acme-core", "1.0.0", archive);
    let lock = format!(
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['acme-core']\n[[package]]\nname='acme-core'\nversion='1.0.0'\nsource='{source}'\nchecksum='{}'\n{extra}",
        raf_fixtures::registry::checksum(archive)
    );
    let ledger = unhomed_ledger(&["acme-core"]);
    let tree: raf_source::MemoryTree = files
        .iter()
        .chain(&[
            ("Cargo.lock", lock.as_str()),
            ("helpers-ledger.ttl", ledger.as_str()),
        ])
        .map(|(path, text)| ((*path).to_owned(), (*text).to_owned()))
        .collect();
    registry_check(&tree, &cache)
}

/// Each way a workspace declares a registry dependency selects its own lock row
/// and is sealed: an inherited renamed entry, a named custom registry, an
/// explicit registry index, beside a package with a build script and a library
/// whose name the archive's manifest states.
#[test]
fn registry_declarations_are_matched_to_their_own_lock_rows() {
    const APP: &str = "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n";
    let custom = "registry+https://custom.example/index";
    let archive = acme_core(PROVIDED);
    let renamed = raf_fixtures::registry::crate_archive(
        "acme-core",
        "1.0.0",
        &[
            (
                "Cargo.toml",
                "[package]\nname='acme-core'\nversion='1.0.0'\nedition='2024'\n[lib]\nname='api'\npath='src/lib.rs'\n",
            ),
            ("src/lib.rs", PROVIDED),
        ],
    );
    let manifest = |dependency: &str| format!("{APP}[dependencies]\n{dependency}\n");
    let cases = [
        (
            "inherited renamed entry",
            vec![
                (
                    "Cargo.toml",
                    "[workspace]\nmembers=['app']\n[workspace.dependencies]\nalias={package='acme-core',version='1'}\n",
                ),
                (
                    "app/Cargo.toml",
                    "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nalias.workspace=true\n",
                ),
                ("app/src/lib.rs", COPIED),
            ],
            CRATES_IO,
            &archive,
            "app/src/lib.rs:1 app::copy matches acme_core::provided ",
        ),
        (
            "named custom registry",
            vec![
                (
                    "Cargo.toml",
                    "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nacme-core={version='1',registry='custom'}\n",
                ),
                ("src/lib.rs", COPIED),
            ],
            custom,
            &archive,
            "src/lib.rs:1 app::copy matches acme_core::provided ",
        ),
        (
            "explicit registry index",
            vec![
                (
                    "Cargo.toml",
                    "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nacme-core={version='1',registry-index='https://custom.example/index'}\n",
                ),
                ("src/lib.rs", COPIED),
            ],
            custom,
            &archive,
            "src/lib.rs:1 app::copy matches acme_core::provided ",
        ),
        (
            "a build script beside the dependency",
            vec![
                ("Cargo.toml", ""),
                ("build.rs", "fn main() {}"),
                ("src/lib.rs", COPIED),
            ],
            CRATES_IO,
            &archive,
            "src/lib.rs:1 app::copy matches acme_core::provided ",
        ),
        (
            "a library named by the archive's manifest",
            vec![("Cargo.toml", ""), ("src/lib.rs", COPIED)],
            CRATES_IO,
            &renamed,
            "src/lib.rs:1 app::copy matches api::provided ",
        ),
    ];
    for (label, mut files, source, archive, expected) in cases {
        let plain = manifest("acme-core='1'");
        if files[0].1.is_empty() {
            files[0].1 = plain.as_str();
        }
        let report = registry_scenario(&files, source, archive, "").unwrap();
        assert_eq!(
            report.findings().len(),
            1,
            "{label}: {:?}",
            report.findings()
        );
        assert!(
            report.findings()[0].contains(expected),
            "{label}: {:?}",
            report.findings()
        );
    }
}

/// Only the row a workspace package depends on directly is sealed: a transitive
/// package of the same name at another version needs no archive, and the
/// directly depended-on version still refuses by name when its archive is
/// missing.
#[test]
fn a_transitive_row_of_the_same_name_is_not_sealed() {
    let archive = acme_core(PROVIDED);
    let lock = format!(
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['acme-core 1.0.0','other']\n[[package]]\nname='acme-core'\nversion='1.0.0'\nsource='{CRATES_IO}'\nchecksum='{}'\n[[package]]\nname='acme-core'\nversion='2.0.0'\nsource='{CRATES_IO}'\nchecksum='{}'\n[[package]]\nname='other'\nversion='1.0.0'\nsource='{CRATES_IO}'\nchecksum='{}'\ndependencies=['acme-core 2.0.0']\n",
        raf_fixtures::registry::checksum(&archive),
        raf_fixtures::registry::checksum(b"never cached"),
        raf_fixtures::registry::checksum(b"unused"),
    );
    let ledger = unhomed_ledger(&[]);
    let tree = raf_source::MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nacme-core='=1.0.0'\n",
        ),
        (
            "src/lib.rs",
            "pub fn copy(input:u32)->u32 { let result=input*2; result+8 }",
        ),
        ("Cargo.lock", lock.as_str()),
        ("helpers-ledger.ttl", ledger.as_str()),
    ]);
    let check = |cache: &purrdf_testkit::TempDir| {
        let registry = raf_census::LockedRegistry::new(raf_census::ArchiveCache::at(cache.path()))
            .seal("acme-core");
        Gate::workspace(".").check_locked_registry(&tree, &registry)
    };
    let cache = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    raf_fixtures::registry::place(cache.path(), "index-0", "acme-core", "1.0.0", &archive);
    let report = check(&cache).unwrap();
    assert!(report.is_clean(), "{:?}", report.findings());
    let empty = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let message = check(&empty).unwrap_err().to_string();
    assert!(message.contains("acme-core 1.0.0"), "{message}");
    assert!(!message.contains("2.0.0"), "{message}");
}

/// A cache holding the archive in two index directories uses the one whose
/// digest is the locked checksum, however the directories sort; with no match
/// it refuses naming each candidate. A lock file over the file-size bound is
/// refused before it is read.
#[test]
fn the_cached_archive_is_chosen_by_its_checksum_and_the_lock_read_is_bounded() {
    let archive = acme_core(PROVIDED);
    let stale = acme_core("pub fn provided() {}");
    let cache = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    raf_fixtures::registry::place(cache.path(), "a-first", "acme-core", "1.0.0", &stale);
    raf_fixtures::registry::place(cache.path(), "b-second", "acme-core", "1.0.0", &archive);
    let (tree, _) = registry_consumer("acme-core", COPIED, &archive, None);
    let found = registry_check(&tree, &cache).unwrap();
    assert_eq!(found.findings().len(), 1, "{:?}", found.findings());
    let other = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    raf_fixtures::registry::place(other.path(), "a-first", "acme-core", "1.0.0", &stale);
    let message = registry_check(&tree, &other).unwrap_err().to_string();
    assert!(
        message.contains("a-first") && message.contains("does not match the lock checksum"),
        "{message}"
    );
    let root = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    files::write(
        root.path(),
        &[
            (
                "Cargo.toml",
                "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[workspace]\n",
            ),
            ("src/lib.rs", "pub fn a() {}"),
            ("helpers-ledger.ttl", unhomed_ledger(&[]).as_str()),
        ],
    );
    // A sparse lock file one byte over the 8 MiB bound: nothing is allocated.
    std::fs::File::create(root.path().join("Cargo.lock"))
        .unwrap()
        .set_len((8 << 20) + 1)
        .unwrap();
    let registry = raf_census::LockedRegistry::new(raf_census::ArchiveCache::at(cache.path()));
    let refused = Gate::workspace(root.path())
        .check_registry(&registry)
        .unwrap_err()
        .to_string();
    assert!(refused.contains("Cargo.lock"), "{refused}");
}

/// A selector added with `seal` whose package the manifest and the lock file
/// disagree about is refused by package, selector and declaration, never left
/// unsealed; the agreeing declaration seals. An oversized cached candidate is
/// skipped while a later one matches, and when it is the only candidate the
/// refusal names it.
#[test]
fn a_sealed_selector_that_manifest_and_lock_disagree_about_is_refused_beside_an_agreeing_one() {
    let archive = acme_core(PROVIDED);
    let lock = format!(
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['acme-core']\n[[package]]\nname='acme-core'\nversion='1.0.0'\nsource='{CRATES_IO}'\nchecksum='{}'\n",
        raf_fixtures::registry::checksum(&archive)
    );
    let ledger = unhomed_ledger(&[]);
    let tree = |requirement: &str| {
        let manifest = format!(
            "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nacme-core='{requirement}'\n"
        );
        raf_source::MemoryTree::from([
            ("Cargo.toml", manifest.as_str()),
            (
                "src/lib.rs",
                "pub fn copy(input:u32)->u32 { let result=input*2; result+8 }",
            ),
            ("Cargo.lock", lock.as_str()),
            ("helpers-ledger.ttl", ledger.as_str()),
        ])
    };
    let cache = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    raf_fixtures::registry::place(cache.path(), "index-0", "acme-core", "1.0.0", &archive);
    let registry = raf_census::LockedRegistry::new(raf_census::ArchiveCache::at(cache.path()))
        .seal("acme-core");
    let agreeing = Gate::workspace(".")
        .check_locked_registry(&tree("1"), &registry)
        .unwrap();
    assert!(agreeing.is_clean(), "{:?}", agreeing.findings());
    let message = Gate::workspace(".")
        .check_locked_registry(&tree("2"), &registry)
        .unwrap_err()
        .to_string();
    for expected in [
        "acme-core",
        "\"acme-core\"",
        "disagree",
        "dependencies.acme-core",
    ] {
        assert!(message.contains(expected), "{expected} missing: {message}");
    }

    let oversize = |directory: &std::path::Path| {
        let index = directory.join("a-first");
        std::fs::create_dir_all(&index).unwrap();
        // A sparse file one byte over the 256 MiB budget: nothing is allocated.
        std::fs::File::create(index.join("acme-core-1.0.0.crate"))
            .unwrap()
            .set_len((256 << 20) + 1)
            .unwrap();
    };
    let skipped = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    oversize(skipped.path());
    raf_fixtures::registry::place(skipped.path(), "b-second", "acme-core", "1.0.0", &archive);
    let later = raf_census::LockedRegistry::new(raf_census::ArchiveCache::at(skipped.path()))
        .seal("acme-core");
    assert!(
        Gate::workspace(".")
            .check_locked_registry(&tree("1"), &later)
            .unwrap()
            .is_clean()
    );
    let only = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    oversize(only.path());
    let alone = raf_census::LockedRegistry::new(raf_census::ArchiveCache::at(only.path()))
        .seal("acme-core");
    let message = Gate::workspace(".")
        .check_locked_registry(&tree("1"), &alone)
        .unwrap_err()
        .to_string();
    assert!(
        message.contains("a-first") && message.contains("byte archive budget"),
        "{message}"
    );
}

/// A declaration naming no registry depends on the default registry: a
/// same-name row of a custom registry that also satisfies its requirement is
/// neither sealed nor needs an archive.
#[test]
fn an_unnamed_declaration_is_bound_to_the_default_registry_only() {
    let archive = acme_core(PROVIDED);
    let lock = format!(
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['acme-core 1.5.0 (registry+https://custom.example/index)','acme-core 1.0.0 ({CRATES_IO})']\n[[package]]\nname='acme-core'\nversion='1.0.0'\nsource='{CRATES_IO}'\nchecksum='{}'\n[[package]]\nname='acme-core'\nversion='1.5.0'\nsource='registry+https://custom.example/index'\nchecksum='{}'\n",
        raf_fixtures::registry::checksum(&archive),
        raf_fixtures::registry::checksum(b"custom never cached"),
    );
    let ledger = unhomed_ledger(&[]);
    let tree = raf_source::MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nacme-core='1'\n",
        ),
        (
            "src/lib.rs",
            "pub fn copy(input:u32)->u32 { let result=input*2; result+8 }",
        ),
        ("Cargo.lock", lock.as_str()),
        ("helpers-ledger.ttl", ledger.as_str()),
    ]);
    let cache = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    raf_fixtures::registry::place(cache.path(), "index-0", "acme-core", "1.0.0", &archive);
    let registry = raf_census::LockedRegistry::new(raf_census::ArchiveCache::at(cache.path()))
        .seal("acme-core");
    let report = Gate::workspace(".")
        .check_locked_registry(&tree, &registry)
        .unwrap();
    assert!(report.is_clean(), "{:?}", report.findings());
}

/// An unnamed declaration whose only same-name lock row comes from a custom
/// registry is refused by package, selector, declaration and registry, never
/// dropped; the same lock with a default-registry row seals clean.
#[test]
fn an_unnamed_declaration_with_only_a_custom_registry_row_is_refused() {
    let archive = acme_core(PROVIDED);
    let ledger = unhomed_ledger(&[]);
    let check = |source: &str| {
        let lock = format!(
            "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['acme-core']\n[[package]]\nname='acme-core'\nversion='1.0.0'\nsource='{source}'\nchecksum='{}'\n",
            raf_fixtures::registry::checksum(&archive)
        );
        let tree = raf_source::MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nacme-core='1'\n",
            ),
            (
                "src/lib.rs",
                "pub fn copy(input:u32)->u32 { let result=input*2; result+8 }",
            ),
            ("Cargo.lock", lock.as_str()),
            ("helpers-ledger.ttl", ledger.as_str()),
        ]);
        let cache = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
        raf_fixtures::registry::place(cache.path(), "index-0", "acme-core", "1.0.0", &archive);
        let registry = raf_census::LockedRegistry::new(raf_census::ArchiveCache::at(cache.path()))
            .seal("acme-core");
        Gate::workspace(".").check_locked_registry(&tree, &registry)
    };
    assert!(check(CRATES_IO).unwrap().is_clean());
    let message = check("registry+https://custom.example/index")
        .unwrap_err()
        .to_string();
    for expected in [
        "acme-core",
        "\"acme-core\"",
        "dependencies.acme-core",
        "non-default registry registry+https://custom.example/index",
    ] {
        assert!(message.contains(expected), "{expected} missing: {message}");
    }
}
