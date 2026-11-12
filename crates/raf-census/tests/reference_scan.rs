// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reference packages supply evidence without entering first-party structural debt.

use raf_census::{ExternalReference, Gate, Index, MatchClass, Query, QueryReport, ReferenceRoot};
use raf_rdf::{Builder, Format, Graph, TermValue, Triple, vocab};
use raf_source::{RelPath, Tree as _};
use std::fmt::Write as _;
#[path = "support/files.rs"]
mod files;
#[path = "support/git_repo.rs"]
mod git_repo;
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
    use raf_rdf::TermValue as Term;
    let spec = |text: &str| {
        let (name, version) = text.split_once('@').unwrap_or((text, "1.0.0"));
        (name.to_owned(), version.to_owned())
    };
    let (member, member_version) = spec(member);
    let parent =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.stage/reference-sources");
    std::fs::create_dir_all(&parent).unwrap();
    let owned = purrdf_testkit::TempDir::new_in(&parent).unwrap();
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
        (
            "ws/helpers-ledger.ttl".into(),
            ledger.finish().unwrap().to_text(Format::Turtle).unwrap(),
        ),
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
