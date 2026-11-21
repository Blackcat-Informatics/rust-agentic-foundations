// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

use raf_census::{Gate, PRODUCTION_CLIPPY_ARGS, ProductionInputs, ProductionLintRun};
use raf_source::DiskTree;
use std::{fs, path::Path, process::Command};

const MANIFEST: &str = r#"
[workspace]
[package]
name = "demo"
version = "0.1.0"
edition = "2024"
[lints.rust]
unused = { level = "deny", priority = -1 }
dead_code = "deny"
unreachable_pub = "deny"
unused_crate_dependencies = "warn"
[lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
allow_attributes = "deny"
allow_attributes_without_reason = "deny"
used_underscore_binding = "deny"
dbg_macro = "deny"
todo = "deny"
unimplemented = "deny"
"#;

fn capture(root: &Path, status: u8, message: &str) -> ProductionLintRun {
    let inputs = ProductionInputs::read(&DiskTree::new(root).unwrap()).unwrap();
    ProductionLintRun::capture(
        &inputs.digest().to_hex(),
        &inputs,
        PRODUCTION_CLIPPY_ARGS,
        "controlled compiler".to_owned(),
        status,
        message.to_owned(),
    )
    .unwrap()
}

fn check(root: &Path, receipt: Option<&Path>, options: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-census"));
    command
        .current_dir(root)
        .env_remove("RAF_CENSUS_LINT_EVIDENCE")
        .arg("check")
        .args(options);
    if let Some(path) = receipt {
        command.env("RAF_CENSUS_LINT_EVIDENCE", path);
    }
    super::alone(&mut command)
}

fn assert_verdict(root: &Path, destination: &Path, run: &ProductionLintRun, finding: Option<&str>) {
    fs::write(destination, run.to_turtle().unwrap()).unwrap();
    let cli = check(root, Some(destination), &[]);
    let report = Gate::workspace(root)
        .with_production_lint_run(run.clone())
        .check()
        .unwrap();
    let cli_findings = String::from_utf8(cli.stdout).unwrap();
    assert_eq!(cli_findings.lines().collect::<Vec<_>>(), report.findings());
    assert_eq!(cli.status.success(), finding.is_none(), "{cli_findings}");
    assert_eq!(report.is_clean(), finding.is_none(), "{cli_findings}");
    if let Some(required) = finding {
        assert!(cli_findings.contains(required), "{cli_findings}");
    }
}

#[test]
fn default_checks_require_passing_production_evidence_for_the_current_sources_and_policy() {
    // Both entry points must require a clean verdict for exactly the inspected inputs.
    let scratch = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let root = scratch.path().join("demo");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("Cargo.toml"), MANIFEST).unwrap();
    fs::write(root.join("Cargo.lock"), "version = 4\n").unwrap();
    fs::write(root.join("src/implementation.rs"), "pub fn helper() {}").unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn home() -> u32 { 7 } include!(\"implementation.rs\");",
    )
    .unwrap();
    super::write_ledger(&root, &[]);
    let receipt = scratch.path().join("run.ttl");
    let missing = check(&root, None, &[]);
    assert_eq!(missing.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&missing.stdout).contains("missing production lint evidence"));
    let structural = check(&root, None, &["--no-production-lints"]);
    assert!(structural.status.success(), "{structural:?}");
    let clean = capture(&root, 0, "");
    assert_verdict(&root, &receipt, &clean, None);
    let inputs = ProductionInputs::read(&DiskTree::new(&root).unwrap()).unwrap();
    let mut other_targets = PRODUCTION_CLIPPY_ARGS.to_vec();
    other_targets.splice(2..4, ["--all-targets"]);
    assert!(
        ProductionLintRun::capture(
            &inputs.digest().to_hex(),
            &inputs,
            &other_targets,
            "controlled compiler".to_owned(),
            0,
            String::new()
        )
        .is_err()
    );
    fs::write(root.join("src/implementation.rs"), "fn helper() {}").unwrap();
    assert_verdict(
        &root,
        &receipt,
        &clean,
        Some("stale production lint evidence"),
    );
    let failed = capture(&root, 101, "dead_code: function helper is never used");
    assert_verdict(&root, &receipt, &failed, Some("dead_code"));
    fs::write(
        root.join("Cargo.toml"),
        MANIFEST.replace("dbg_macro = \"deny\"", "dbg_macro = \"allow\""),
    )
    .unwrap();
    let weak = capture(&root, 0, "");
    assert_verdict(&root, &receipt, &weak, Some("lints.clippy.dbg_macro"));
    fs::write(
        root.join("Cargo.toml"),
        format!("{MANIFEST}\nrestriction = {{ level = \"allow\", priority = 1 }}\n"),
    )
    .unwrap();
    let overridden = capture(&root, 0, "");
    assert_verdict(&root, &receipt, &overridden, Some("lints.clippy.dbg_macro"));
    member_configuration(&root, &receipt);
    unverified_inputs(&root, &receipt);
}

fn member_configuration(root: &Path, receipt: &Path) {
    // A member can inherit its Clippy thresholds from an intermediate directory.
    let member = root.join("crates/member");
    fs::create_dir_all(member.join("src")).unwrap();
    fs::write(member.join("src/lib.rs"), "pub fn member() {}").unwrap();
    fs::write(
        member.join("Cargo.toml"),
        MANIFEST
            .replace("[workspace]", "")
            .replace("\"demo\"", "\"member\""),
    )
    .unwrap();
    fs::write(
        root.join("Cargo.toml"),
        MANIFEST.replace("[workspace]", "[workspace]\nmembers = ['crates/member']"),
    )
    .unwrap();
    let previous = capture(root, 0, "");
    assert_verdict(root, receipt, &previous, None);
    fs::write(
        root.join("crates/.clippy.toml"),
        "trivial-copy-size-limit = 12\n",
    )
    .unwrap();
    assert_verdict(
        root,
        receipt,
        &previous,
        Some("stale production lint evidence"),
    );
}

fn unverified_inputs(root: &Path, receipt: &Path) {
    fs::write(root.join("Cargo.toml"), MANIFEST).unwrap();
    fs::write(root.join("build.rs"), "fn main() {}").unwrap();
    assert_verdict(
        root,
        receipt,
        &capture(root, 0, ""),
        Some("first-party build script"),
    );
    fs::remove_file(root.join("build.rs")).unwrap();
    fs::create_dir(root.join(".cargo")).unwrap();
    fs::write(
        root.join(".cargo/config.toml"),
        "[build]\nrustflags=['--cfg', 'test']\n",
    )
    .unwrap();
    let tree = DiskTree::new(root).unwrap();
    let error = ProductionInputs::read(&tree).unwrap_err().to_string();
    assert!(error.contains("build.rustflags"), "{error}");
    fs::remove_file(root.join(".cargo/config.toml")).unwrap();
    fs::write(
        root.join("src/implementation.rs"),
        "use std::include as fragment; fragment!(\"nested.rs\");",
    )
    .unwrap();
    let error = ProductionInputs::read(&tree).unwrap_err().to_string();
    assert!(error.contains("aliased include!"), "{error}");
    fs::write(root.join("src/implementation.rs"), "mod generated;").unwrap();
    let error = ProductionInputs::read(&tree).unwrap_err().to_string();
    assert!(error.contains("file-backed module"), "{error}");
}
