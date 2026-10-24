// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Real frozen Cargo targets retain repository registrations and requested cwd.

#[path = "../../raf-census/examples/support/registration.rs"]
mod registration;

#[path = "../../raf-census/examples/support/ledger.rs"]
mod policy;

use policy::ledger;
use std::{fs, path::Path, process::Command};

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
         [dev-dependencies]\nraf-census={{path={}}}\nraf-rdf={{path={}}}\n\
         syn={{version='3',default-features=false,features=['full','parsing']}}\n",
        toml::Value::String(crates.join("raf-census").to_str().unwrap().into()),
        toml::Value::String(crates.join("raf-rdf").to_str().unwrap().into()),
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
}
