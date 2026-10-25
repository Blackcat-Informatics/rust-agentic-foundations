// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Real frozen Cargo targets retain repository registrations and requested cwd.

#[path = "../../raf-census/examples/support/registration.rs"]
mod registration;

#[path = "../../raf-census/examples/support/ledger.rs"]
mod policy;

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
    let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let root = scratch.path();
    fs::create_dir_all(root.join("examples/support")).unwrap();
    fs::create_dir(root.join("src")).unwrap();
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let manifest = format!(
        "[package]\nname='demo'\nversion='0.1.0'\nedition='2024'\n[workspace]\n\
         [dev-dependencies]\nraf-census={{path={}}}\nraf-rdf={{path={}}}\nraf-source={{path={}}}\n\
         syn={{version='3',default-features=false,features=['full','parsing']}}\n",
        toml::Value::String(crates.join("raf-census").to_str().unwrap().into()),
        toml::Value::String(crates.join("raf-rdf").to_str().unwrap().into()),
        toml::Value::String(crates.join("raf-source").to_str().unwrap().into()),
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
    let caller = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
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
    let result = run(root, caller.path(), "registered");
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        expected.findings().join("\n") + "\n"
    );
    assert_eq!(snapshot(root), planted);
    assert!(!root.join("census-debt.ttl").exists());

    fs::write(root.join("helpers-ledger.ttl"), ledger("other")).unwrap();
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

fn registered_writes(caller: &Path) {
    let fixture = writer_fixture();
    let root = fixture.path();
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
