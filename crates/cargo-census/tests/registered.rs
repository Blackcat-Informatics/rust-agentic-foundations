// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Real frozen Cargo targets retain repository registrations and requested cwd.

#[path = "../../raf-census/examples/support/registration.rs"]
mod registration;

#[path = "../../raf-census/examples/support/ledger.rs"]
mod policy;
#[path = "../../raf-census/tests/support/reference_ledger.rs"]
mod reference_ledger;

use policy::ledger;
use std::{fmt::Write as _, fs, path::Path, process::Command};

const WRITE_TARGET: &str = r#"#[path="support/registration.rs"]
mod registration;
use raf_census::{RegisteredOperation, RegisteredCheck, RegisteredDebtWrite};
use raf_source::{DiskTree, RelPath, LoadOptions, LockedCatalog, LockedManifest, LockedEdge, DeclaredCfg, DeclaredConfiguration, TargetKind};
use std::{error::Error, io::Write as _, process::ExitCode};
fn main() -> Result<ExitCode, Box<dyn Error>> {
    let operation = RegisteredOperation::from_args(std::env::args_os().skip(1))?;
    let tree = DiskTree::new(".")?;
    let catalog = LockedCatalog::load(&tree, &RelPath::new("Cargo.lock")?, &LoadOptions::default())?;
    let package = |name| catalog.packages().find(|row| row.name() == name).expect("fixture package").id();
    let config = |name| DeclaredConfiguration::new(DeclaredCfg::complete("fixture-target", &[]), &[], TargetKind::Lib, name);
    let edges = [LockedEdge::new("dependencies.renamed", None)];
    let manifests = [
        LockedManifest::new(package("demo"), RelPath::new("Cargo.toml")?, &edges).with_configuration(config("demo")),
        LockedManifest::new(package("provider"), RelPath::new("provider/Cargo.toml")?, &[]).with_configuration(config("provider")),
    ];
    let gate = registration::registered_gate(".");
    let (text, code) = match operation {
        RegisteredOperation::Check => {
            let response = RegisteredCheck::from_check(&gate.check_locked(&tree, &catalog, &manifests, &[]))?;
            (response.to_ntriples()?, response.exit_code())
        }
        RegisteredOperation::WriteDebt => {
            let response = RegisteredDebtWrite::from_write(&gate.write_debt_locked(&catalog, &manifests, &[]))?;
            (response.to_ntriples()?, response.exit_code())
        }
        RegisteredOperation::Index => {
            let report = gate.check_locked(&tree, &catalog, &manifests, &[])?;
            (report.census().index(report.jobs())?.to_ntriples()?, 0)
        }
    };
    std::io::stdout().lock().write_all(text.as_bytes())?;
    Ok(ExitCode::from(code))
}
"#;

const TARGET: &str = r#"#[path="support/registration.rs"]
mod registration;
use std::{error::Error, io::Write as _, process::ExitCode};
fn main() -> Result<ExitCode, Box<dyn Error>> {
    let result = registration::registered_gate(".").check();
    let response = raf_census::RegisteredCheck::from_check(&result)?;
    std::io::stdout().lock().write_all(response.to_ntriples()?.as_bytes())?;
    Ok(ExitCode::from(response.exit_code()))
}
"#;

fn fixture() -> purrdf_testkit::TempDir {
    let scratch = scratch();
    let root = scratch.path();
    fs::create_dir_all(root.join("examples/support")).unwrap();
    fs::create_dir(root.join("src")).unwrap();
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let manifest = format!(
        "[package]\nname='demo'\nversion='0.1.0'\nedition='2024'\n[workspace]\n\
         [dev-dependencies]\nraf-census={{path={}}}\nraf-rdf={{path={}}}\nraf-source={{path={}}}\n\
         syn={{version='3',default-features=false,features=['full','parsing']}}\n{}\n",
        quoted_path(&crates.join("raf-census")),
        quoted_path(&crates.join("raf-rdf")),
        quoted_path(&crates.join("raf-source")),
        include_str!("../../../Cargo.toml")
            .lines()
            .find(|line| line.starts_with("purrdf-lex = "))
            .expect("the shared consumer uses the workspace's exact native storage dependency"),
    );
    fs::write(root.join("Cargo.toml"), manifest).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "/// Home.\npub fn home()->u32{1}\n",
    )
    .unwrap();
    fs::write(root.join("helpers-ledger.ttl"), ledger("demo")).unwrap();
    fs::write(root.join("examples/registered.rs"), TARGET).unwrap();
    fs::write(
        root.join("examples/support/registration.rs"),
        include_bytes!("../../raf-census/examples/support/registration.rs"),
    )
    .unwrap();
    let lock = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(lock.status.success(), "{lock:?}");
    scratch
}

fn quoted_path(path: &Path) -> String {
    let mut quoted = String::new();
    // Escaping every control also satisfies TOML's basic-string restriction.
    purrdf_lex::json_escape::push_string(
        &mut quoted,
        path.to_str().unwrap(),
        purrdf_lex::json_escape::JsonEscapes::Controls,
    );
    quoted
}

fn scratch() -> purrdf_testkit::TempDir {
    let parent = Path::new(env!("CARGO_TARGET_TMPDIR")).join("registered-index-fixtures");
    fs::create_dir_all(&parent).unwrap();
    purrdf_testkit::TempDir::new_in(parent).unwrap()
}

fn run(root: &Path, caller: &Path, example: &str) -> std::process::Output {
    let before = snapshot(root);
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-census"))
        .current_dir(caller)
        .args(["census", "--workspace"])
        .arg(root)
        .args(["check", "--package", "demo", "--example", example])
        .output()
        .unwrap();
    assert_eq!(snapshot(root), before);
    assert!(!root.join("census-debt.ttl").exists());
    result
}

fn snapshot(root: &Path) -> Vec<Vec<u8>> {
    [
        "Cargo.toml",
        "Cargo.lock",
        "src/lib.rs",
        "helpers-ledger.ttl",
        "examples/registered.rs",
        "examples/support/registration.rs",
    ]
    .map(|path| fs::read(root.join(path)).unwrap())
    .to_vec()
}

#[test]
fn actual_registered_target_preserves_rule_extension_root_and_read_only_parity() {
    let fixture = fixture();
    let root = fixture.path();
    let caller = scratch();
    let before = snapshot(root);
    assert!(
        registration::registered_gate(root)
            .check()
            .unwrap()
            .is_clean()
    );
    assert!(raf_census::Gate::workspace(root).check().is_err());
    assert!(
        registration::rule_gate(root)
            .check()
            .unwrap_err()
            .to_string()
            .contains("unclaimed extension")
    );
    let clean = run(root, caller.path(), "registered");
    assert!(clean.status.success(), "{clean:?}");
    assert_eq!(clean.stdout, b"census: clean\n");
    assert_eq!(snapshot(root), before);
    assert!(!root.join("census-debt.ttl").exists());

    imported_policy_parity(root, caller.path());

    fs::write(
        root.join("src/lib.rs"),
        "/// Home.\npub fn home()->u32{1}\npub fn local_copy()->u32{2}\n",
    )
    .unwrap();
    let planted = snapshot(root);
    let expected = registration::registered_gate(root).check().unwrap();
    assert!(!expected.is_clean());
    assert!(
        expected
            .findings()
            .iter()
            .any(|message| message.contains("rule no-local-copy: use home"))
    );
    assert!(
        expected
            .findings()
            .iter()
            .any(|message| message.contains("rule sole-name:"))
    );
    let result = run(root, caller.path(), "registered");
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        expected.findings().join("\n") + "\n"
    );
    assert_eq!(snapshot(root), planted);
    assert!(!root.join("census-debt.ttl").exists());

    fs::write(root.join("helpers-ledger.ttl"), ledger("other")).unwrap();
    let direct = registration::registered_gate(root).check().unwrap_err();
    assert!(direct.to_string().contains("owner extension requires"));
    let denied = run(root, caller.path(), "registered");
    assert_eq!(denied.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&denied.stderr).contains("owner extension requires"));
    let missing = run(root, caller.path(), "absent");
    assert_eq!(missing.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("no example target"));
    fs::write(
        root.join("examples/registered.rs"),
        "compile_error!(\"registered target failed\"); fn main() {}\n",
    )
    .unwrap();
    let broken = run(root, caller.path(), "registered");
    assert_eq!(broken.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&broken.stderr).contains("registered target failed"));
    registered_writes(caller.path());
}

struct ProofImport;

impl raf_census::ForeignLedgerAdapter for ProofImport {
    fn claim(
        &self,
        _: &str,
        field: raf_census::ForeignField<'_, '_>,
    ) -> Option<Result<String, String>> {
        (field.node().key.as_ref()?.decoded == "capacity_proof")
            .then(|| Ok("urn:example:capacity-proof".into()))
    }
}

fn imported_policy_parity(root: &Path, caller: &Path) {
    let original = fs::read(root.join("helpers-ledger.ttl")).unwrap();
    let proof = "{file='src/lib.rs',source='source:original-proof-pin',authorities=['A','B','A'],constructors=['Make'],reason='Declared capacity argument'}";
    let foreign = format!(
        "[[job]]\nid='proof'\nsummary='The registered proof'\nhome='demo::home'\nenforced=true\nforbidden_fingerprints=['rule:no-local-copy']\ncapacity_proof=[{proof},{proof}]\n"
    );
    let baseline = r#"{"min_tokens":30,"units":1,"body_groups":[],"shim_groups":[],"hex_tables":[],"shared_constants":[]}"#;
    fs::write(root.join("foreign-baseline.json"), baseline).unwrap();
    for (text, cause) in [
        (foreign.clone(), None),
        (
            foreign.replace("file='src/lib.rs',", ""),
            Some("missing required fields"),
        ),
        (
            foreign.replace("src/lib.rs", "src/other.rs"),
            Some("file changed"),
        ),
        (
            foreign.replace("source:original-proof-pin", "source:stale-proof-pin"),
            Some("source pin is stale"),
        ),
    ] {
        fs::write(root.join("foreign-ledger.toml"), &text).unwrap();
        let tree = raf_source::DiskTree::new(root).unwrap();
        let migrated = raf_census::migrate_legacy_with_extensions(
            &tree,
            raf_census::MigrationInput {
                path: "foreign-ledger.toml",
                text: &text,
            },
            raf_census::MigrationInput {
                path: "foreign-baseline.json",
                text: baseline,
            },
            &[&ProofImport],
        )
        .unwrap();
        assert_eq!(migrated.unmapped, []);
        fs::write(root.join("helpers-ledger.ttl"), migrated.ledger).unwrap();
        let direct = registration::registered_gate(root).check();
        let registered = run(root, caller, "registered");
        if let Some(cause) = cause {
            let error = direct.unwrap_err().to_string();
            assert!(error.contains(cause), "{error}");
            assert_eq!(registered.status.code(), Some(1), "{registered:?}");
            assert!(
                String::from_utf8_lossy(&registered.stderr)
                    .contains(&error.escape_debug().to_string()),
                "{registered:?}"
            );
        } else {
            assert!(direct.unwrap().is_clean());
            assert!(registered.status.success(), "{registered:?}");
            assert_eq!(registered.stdout, b"census: clean\n");
        }
        assert_eq!(
            fs::read_to_string(root.join("foreign-ledger.toml")).unwrap(),
            text
        );
        assert_eq!(
            fs::read_to_string(root.join("foreign-baseline.json")).unwrap(),
            baseline
        );
    }
    fs::write(root.join("helpers-ledger.ttl"), original).unwrap();
}

fn writer_fixture() -> purrdf_testkit::TempDir {
    let fixture = fixture();
    let root = fixture.path();
    fs::create_dir_all(root.join("provider/src")).unwrap();
    let manifest = fs::read_to_string(root.join("Cargo.toml"))
        .unwrap()
        .replace("[workspace]\n", "[workspace]\nmembers=['provider']\n");
    fs::write(
        root.join("Cargo.toml"),
        manifest + "\n[dependencies]\nrenamed={package='provider',path='provider'}\n",
    )
    .unwrap();
    fs::write(
        root.join("provider/Cargo.toml"),
        "[package]\nname='provider'\nversion='1.0.0'\nedition='2024'\n",
    )
    .unwrap();
    fs::write(
        root.join("provider/src/lib.rs"),
        "pub mod api { pub fn encode(value:u32)->u32 {value} }\n",
    )
    .unwrap();
    fs::write(root.join("examples/registered.rs"), WRITE_TARGET).unwrap();
    fs::write(root.join("helpers-ledger.ttl"), writer_ledger("demo")).unwrap();
    writer_source(root, &["first", "second"], "");
    let lock = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(lock.status.success(), "{lock:?}");
    fixture
}

fn writer_source(root: &Path, copies: &[&str], extra: &str) {
    let mut source = "/// Home.\npub fn home()->u32{1}\npub fn run(value:u32)->u32{renamed::api::encode(value)}\npub fn number()->u32{11}\n".to_owned();
    for name in copies {
        writeln!(source, "pub fn {name}()->u32{{11}}").unwrap();
    }
    source.push_str(extra);
    fs::write(root.join("src/lib.rs"), source).unwrap();
}

fn writer_ledger(owner: &str) -> String {
    use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};
    let graph = raf_rdf::Graph::from_text(&ledger(owner), Format::Turtle).unwrap();
    let mut builder = Builder::from_graph(&graph);
    let iri = |name: &str| Term::iri(format!("{}{name}", vocab::REPORT));
    let mut put = |subject: &str, key: &str, object| {
        builder
            .insert(Triple {
                subject: Term::iri(subject),
                predicate: if key == "type" {
                    Term::iri(vocab::TYPE)
                } else {
                    iri(key)
                },
                object,
            })
            .unwrap();
    };
    for (id, home, enforced) in [
        ("native", "demo::run", true),
        ("copies", "demo::number", false),
    ] {
        let node = format!("#{id}");
        put("", "job", Term::iri(&node));
        put(&node, "type", iri("LedgerJob"));
        put(&node, "jobId", Term::simple_literal(id));
        put(&node, "summary", Term::simple_literal("Use home"));
        put(&node, "homeSymbol", Term::simple_literal(home));
        put(&node, "enforced", Term::boolean(enforced));
        put(
            &node,
            "forbiddenConstant",
            Term::integer(if enforced { 7 } else { 11 }),
        );
    }
    put("#native", "preferredProvider", Term::iri("#provider"));
    put("#provider", "type", iri("PreferredProvider"));
    for (key, value) in [
        ("package", "provider"),
        ("consumer", "demo::run"),
        ("entryPoint", "provider::api::encode"),
    ] {
        put("#provider", key, Term::simple_literal(value));
    }
    builder.finish().unwrap().to_text(Format::Turtle).unwrap()
}

fn writer_snapshot(root: &Path) -> Vec<Vec<u8>> {
    let mut inputs = snapshot(root);
    for path in ["provider/Cargo.toml", "provider/src/lib.rs"] {
        inputs.push(fs::read(root.join(path)).unwrap());
    }
    inputs
}

fn write_cli(root: &Path, caller: &Path) -> std::process::Output {
    let before = writer_snapshot(root);
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-census"))
        .current_dir(caller)
        .args(["census", "--workspace"])
        .arg(root)
        .args([
            "debt",
            "--write",
            "--package",
            "demo",
            "--example",
            "registered",
        ])
        .output()
        .unwrap();
    assert_eq!(writer_snapshot(root), before);
    assert!(!root.join(".census-debt.ttl.write").exists());
    result
}

fn write_authorization(root: &Path, caller: &Path) {
    for args in [
        vec!["debt", "--package", "demo", "--example", "registered"],
        vec!["debt", "--write", "--package", "demo"],
        vec!["debt", "--write", "--example", "registered"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_cargo-census"))
            .current_dir(caller)
            .args(["--workspace"])
            .arg(root)
            .args(args)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2), "{result:?}");
        assert!(!root.join("census-debt.ttl").exists());
    }
    for args in [
        vec!["--write"],
        vec!["--census-debt-write-v1", "extra"],
        vec!["--census-debt-write-v1", "--census-debt-write-v1"],
    ] {
        let result = Command::new("cargo")
            .current_dir(root)
            .args([
                "run",
                "--frozen",
                "--package=demo",
                "--example=registered",
                "--",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(!result.status.success(), "{result:?}");
        assert!(!root.join("census-debt.ttl").exists());
    }
}

fn locked_report(root: &Path) -> raf_census::GateReport {
    use raf_source::{
        DeclaredCfg, DeclaredConfiguration, DiskTree, LoadOptions, LockedCatalog, LockedEdge,
        LockedManifest, RelPath, TargetKind,
    };
    let tree = DiskTree::new(root).unwrap();
    let catalog = LockedCatalog::load(
        &tree,
        &RelPath::new("Cargo.lock").unwrap(),
        &LoadOptions::default(),
    )
    .unwrap();
    let package = |name| {
        catalog
            .packages()
            .find(|row| row.name() == name)
            .unwrap()
            .id()
    };
    let config = |name| {
        DeclaredConfiguration::new(
            DeclaredCfg::complete("fixture-target", &[]),
            &[],
            TargetKind::Lib,
            name,
        )
    };
    let edges = [LockedEdge::new("dependencies.renamed", None)];
    let manifests = [
        LockedManifest::new(package("demo"), RelPath::new("Cargo.toml").unwrap(), &edges)
            .with_configuration(config("demo")),
        LockedManifest::new(
            package("provider"),
            RelPath::new("provider/Cargo.toml").unwrap(),
            &[],
        )
        .with_configuration(config("provider")),
    ];
    registration::registered_gate(root)
        .check_locked(&tree, &catalog, &manifests, &[])
        .unwrap()
}

fn index_cli(root: &Path, caller: &Path, extra: &[&str]) -> std::process::Output {
    index_cli_for(root, caller, "registered", extra)
}

fn index_cli_for(
    root: &Path,
    caller: &Path,
    example: &str,
    extra: &[&str],
) -> std::process::Output {
    query_cli_for(root, caller, example, &["index"], extra)
}

fn query_cli_for(
    root: &Path,
    caller: &Path,
    example: &str,
    command: &[&str],
    extra: &[&str],
) -> std::process::Output {
    let before = writer_snapshot(root);
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-census"))
        .current_dir(caller)
        .args(["--workspace"])
        .arg(root)
        .args(command)
        .args(["--package", "demo", "--example", example])
        .args(extra)
        .output()
        .unwrap();
    assert_eq!(writer_snapshot(root), before);
    assert!(!root.join("census-debt.ttl").exists());
    result
}

fn registered_index(root: &Path, caller: &Path) {
    let original = fs::read(root.join("src/lib.rs")).unwrap();
    writer_source(
        root,
        &["first", "second"],
        "pub fn local_copy()->u32{2}\npub fn fingerprinted(a:u32)->u32{let b=a+1;let c=b+2;let d=c+3;let e=d+4;e+a}\n",
    );
    let report = locked_report(root);
    assert!(!report.is_clean());
    let expected = report.census().index(report.jobs()).unwrap();
    assert!(expected.units.iter().any(|unit| unit.body.is_some()));
    assert!(expected.units.iter().any(|unit| unit.shim.is_some()));
    let copies = expected.jobs.iter().find(|job| job.id == "copies").unwrap();
    assert_eq!(copies.copies, 2);
    assert!(
        copies
            .matches
            .iter()
            .any(|row| row.class == raf_census::MatchClass::Home)
    );
    assert!(
        copies
            .matches
            .iter()
            .any(|row| row.class == raf_census::MatchClass::Copy)
    );
    let first = index_cli(root, caller, &[]);
    assert!(first.status.success(), "{first:?}");
    assert_eq!(
        String::from_utf8(first.stdout.clone()).unwrap(),
        expected.to_ntriples().unwrap()
    );
    assert_eq!(index_cli(root, caller, &[]).stdout, first.stdout);
    let turtle = index_cli(root, caller, &["--format", "turtle"]);
    assert!(turtle.status.success(), "{turtle:?}");
    assert_eq!(
        raf_census::Index::from_turtle(std::str::from_utf8(&turtle.stdout).unwrap()).unwrap(),
        expected
    );
    let written = index_cli(root, caller, &["--out", "index.ttl", "--format", "turtle"]);
    assert!(written.status.success(), "{written:?}");
    assert_eq!(written.stdout, b"");
    assert_eq!(fs::read(caller.join("index.ttl")).unwrap(), turtle.stdout);
    assert!(!root.join("index.ttl").exists());
    index_refusals(root, caller, &first.stdout);
    registered_queries(root, caller, &expected);
    registered_references(root, caller);
    fs::write(root.join("src/lib.rs"), original).unwrap();
}

fn registered_references(root: &Path, caller: &Path) {
    let original_ledger = fs::read(root.join("helpers-ledger.ttl")).unwrap();
    let original_source = fs::read(root.join("provider/src/lib.rs")).unwrap();
    let policy = reference_ledger::ledger("provider", "provider::api", "references");
    fs::write(root.join("helpers-ledger.ttl"), policy).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn body(value:u32)->u32{let doubled=value*2;doubled+11}\npub fn shim(value:u32)->u32{std::convert::identity(value)}\n").unwrap();
    fs::write(root.join("provider/src/lib.rs"), "pub mod api {pub fn first(value:u32)->u32{let doubled=value*2;doubled+11} pub fn second(input:u32)->u32{let result=input*2;result+11} pub fn shim(value:u32)->u32{std::convert::identity(value)}}\npub fn local_copy()->u32{77}\n").unwrap();
    let report = locked_report(root);
    assert_eq!(report.census().groups(), []);
    assert!(report.debt().groups().is_empty());
    let expected = report.census().index(report.jobs()).unwrap();
    assert_eq!(expected.reference_sources.len(), 1);
    assert!(expected.reference_witnesses.len() >= 3);
    assert_eq!(expected.jobs[0].copies, 2);
    let index = index_cli(root, caller, &[]);
    assert!(index.status.success(), "{index:?}");
    assert_eq!(index.stdout, expected.to_ntriples().unwrap().as_bytes());
    for (command, selector, query) in [
        ("job", "references", raf_census::Query::Job("references")),
        (
            "count",
            "references",
            raf_census::Query::Count("references"),
        ),
        (
            "fingerprint",
            "demo::body",
            raf_census::Query::Fingerprint("demo::body"),
        ),
    ] {
        let selected = expected.query(query).unwrap();
        for (format, text) in [
            ("turtle", selected.to_turtle().unwrap()),
            ("ntriples", selected.to_ntriples().unwrap()),
        ] {
            let result = query_cli_for(
                root,
                caller,
                "registered",
                &[command, selector],
                &["--format", format],
            );
            assert!(result.status.success(), "{result:?}");
            assert_eq!(result.stdout, text.as_bytes());
        }
    }
    fs::write(root.join("helpers-ledger.ttl"), original_ledger).unwrap();
    fs::write(root.join("provider/src/lib.rs"), original_source).unwrap();
}

fn registered_queries(root: &Path, caller: &Path, expected: &raf_census::Index) {
    use raf_census::Query;
    for (command, selector, query) in [
        ("job", "copies", Query::Job("copies")),
        ("count", "copies", Query::Count("copies")),
        (
            "fingerprint",
            "demo::fingerprinted",
            Query::Fingerprint("demo::fingerprinted"),
        ),
    ] {
        let expected = expected.query(query).unwrap();
        for (format, text) in [
            ("ntriples", expected.to_ntriples().unwrap()),
            ("turtle", expected.to_turtle().unwrap()),
        ] {
            let result = query_cli_for(
                root,
                caller,
                "registered",
                &[command, selector],
                &["--format", format],
            );
            assert!(result.status.success(), "{result:?}");
            assert_eq!(result.stdout, text.as_bytes());
        }
    }
    let selected = expected.query(Query::Count("copies")).unwrap();
    let written = query_cli_for(
        root,
        caller,
        "registered",
        &["count", "copies"],
        &["--out", "count.ttl", "--format", "turtle"],
    );
    assert!(written.status.success(), "{written:?}");
    assert_eq!(written.stdout, b"");
    assert_eq!(
        fs::read(caller.join("count.ttl")).unwrap(),
        selected.to_turtle().unwrap().as_bytes()
    );
    assert!(!root.join("count.ttl").exists());
    let destination = caller.join("query-preserved.nt");
    fs::write(&destination, b"preserve destination").unwrap();
    for (command, selector, cause) in [
        ("job", "missing", "unknown job"),
        ("count", "missing", "unknown job"),
        ("fingerprint", "missing", "unknown symbol"),
    ] {
        let result = query_cli_for(
            root,
            caller,
            "registered",
            &[command, selector],
            &["--out", "query-preserved.nt"],
        );
        assert_eq!(result.status.code(), Some(1), "{result:?}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(cause),
            "{result:?}"
        );
        assert_eq!(result.stdout, b"");
        assert_eq!(fs::read(&destination).unwrap(), b"preserve destination");
    }
    let failed = query_cli_for(
        root,
        caller,
        "absent",
        &["count", "copies"],
        &["--out", "query-preserved.nt"],
    );
    assert_eq!(failed.status.code(), Some(1));
    assert_eq!(fs::read(&destination).unwrap(), b"preserve destination");
    let original = fs::read(root.join("src/lib.rs")).unwrap();
    writer_source(root, &[], "");
    let report = locked_report(root);
    let zero = report.census().index(report.jobs()).unwrap();
    assert_eq!(
        zero.jobs
            .iter()
            .find(|job| job.id == "copies")
            .unwrap()
            .copies,
        0
    );
    let result = query_cli_for(root, caller, "registered", &["count", "copies"], &[]);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(
        result.stdout,
        zero.query(Query::Count("copies"))
            .unwrap()
            .to_ntriples()
            .unwrap()
            .as_bytes()
    );
    fs::write(root.join("src/lib.rs"), original).unwrap();
}

fn index_refusals(root: &Path, caller: &Path, valid: &[u8]) {
    let destination = caller.join("preserved.nt");
    fs::write(&destination, valid).unwrap();
    let original = fs::read(root.join("helpers-ledger.ttl")).unwrap();
    fs::write(root.join("helpers-ledger.ttl"), writer_ledger("other")).unwrap();
    let failed = index_cli(root, caller, &["--out", "preserved.nt"]);
    assert_eq!(failed.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&failed.stderr).contains("owner extension requires"));
    assert_eq!(fs::read(&destination).unwrap(), valid);
    fs::write(root.join("helpers-ledger.ttl"), original).unwrap();
    let missing = index_cli_for(root, caller, "absent", &["--out", "preserved.nt"]);
    assert_eq!(missing.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("no example target"));
    assert_eq!(fs::read(&destination).unwrap(), valid);
    let failed = index_cli(root, caller, &["--out", "."]);
    assert_eq!(failed.status.code(), Some(1));
    assert_eq!(failed.stdout, b"");
    let invalid = index_cli(root, caller, &["--format", "invalid"]);
    assert_eq!(invalid.status.code(), Some(2));
}

fn registered_writes(caller: &Path) {
    let fixture = writer_fixture();
    let root = fixture.path();
    registered_index(root, caller);
    write_authorization(root, caller);
    let result = write_cli(root, caller);
    assert!(result.status.success(), "{result:?}");
    let adopted = fs::read(root.join("census-debt.ttl")).unwrap();
    assert!(String::from_utf8_lossy(&result.stdout).contains("1 jobs with copies"));
    let direct = writer_fixture();
    let before = writer_snapshot(direct.path());
    let result = Command::new("cargo")
        .current_dir(direct.path())
        .args([
            "run",
            "--frozen",
            "--package=demo",
            "--example=registered",
            "--",
            "--census-debt-write-v1",
        ])
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    assert_eq!(writer_snapshot(direct.path()), before);
    assert_eq!(
        fs::read(direct.path().join("census-debt.ttl")).unwrap(),
        adopted
    );
    assert!(write_cli(root, caller).status.success());
    assert_eq!(fs::read(root.join("census-debt.ttl")).unwrap(), adopted);
    // Empty target arguments remain a read-only check after adoption.
    let checked = Command::new("cargo")
        .current_dir(root)
        .args(["run", "--frozen", "--package=demo", "--example=registered"])
        .output()
        .unwrap();
    assert!(checked.status.success(), "{checked:?}");
    assert!(
        raf_census::RegisteredCheck::from_ntriples(std::str::from_utf8(&checked.stdout).unwrap())
            .is_ok()
    );
    assert_eq!(fs::read(root.join("census-debt.ttl")).unwrap(), adopted);
    writer_source(root, &["first"], "");
    assert!(write_cli(root, caller).status.success());
    let shrunk = fs::read(root.join("census-debt.ttl")).unwrap();
    assert_ne!(shrunk, adopted);
    writer_source(root, &["first", "second"], "");
    write_refuses(root, caller, &shrunk, "growth");
    writer_source(root, &["first"], "pub fn local_copy()->u32{2}\n");
    write_refuses(root, caller, &shrunk, "rule no-local-copy");
    writer_source(root, &["first"], "pub fn competitor()->u32{7}\n");
    write_refuses(root, caller, &shrunk, "competitor");
    writer_source(root, &["first"], "");
    fs::write(root.join("helpers-ledger.ttl"), writer_ledger("other")).unwrap();
    write_refuses(root, caller, &shrunk, "owner extension requires");
    fs::write(root.join("helpers-ledger.ttl"), writer_ledger("demo")).unwrap();
    fs::write(root.join("census-debt.ttl"), b"broken prior\n").unwrap();
    write_refuses(root, caller, b"broken prior\n", "census-debt.ttl");
    fs::write(root.join("census-debt.ttl"), &shrunk).unwrap();
    assert!(write_cli(root, caller).status.success());
    let foreign = raf_census::DebtRecord::new(
        raf_census::NORMAL_FORM + 1,
        [],
        std::collections::BTreeMap::new(),
    )
    .unwrap()
    .to_turtle()
    .unwrap();
    fs::write(root.join("census-debt.ttl"), &foreign).unwrap();
    assert!(write_cli(root, caller).status.success());
    assert_eq!(fs::read(root.join("census-debt.ttl")).unwrap(), shrunk);
    // A committed write precedes its receipt: protocol failure promises no rollback.
    let target = WRITE_TARGET
        .replace(
            "write_all(text.as_bytes())?",
            "write_all(b\"invalid receipt\")?",
        )
        .replace("let (text, code)", "let (_text, code)");
    fs::write(root.join("examples/registered.rs"), target).unwrap();
    // Keep a positive unenforced copy count: zero requires an explicit policy edit.
    fs::write(root.join("census-debt.ttl"), foreign).unwrap();
    let prior = fs::read(root.join("census-debt.ttl")).unwrap();
    let failed = write_cli(root, caller);
    assert_eq!(failed.status.code(), Some(1), "{failed:?}");
    assert_eq!(failed.stdout, [] as [u8; 0]);
    assert!(
        !String::from_utf8_lossy(&failed.stderr).contains("target failed with exit"),
        "{}",
        String::from_utf8_lossy(&failed.stderr)
    );
    assert_ne!(fs::read(root.join("census-debt.ttl")).unwrap(), prior);
}

fn write_refuses(root: &Path, caller: &Path, prior: &[u8], cause: &str) {
    let result = write_cli(root, caller);
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(
        String::from_utf8_lossy(&result.stderr).contains(cause),
        "{result:?}"
    );
    assert_eq!(fs::read(root.join("census-debt.ttl")).unwrap(), prior);
}
