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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
    let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-raf-census"));
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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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

/// Trust and destination admission precede process launch. A completed build
/// supplies a usable receipt, while overflow and changed inputs publish none.
#[test]
#[expect(
    clippy::unwrap_used,
    reason = "fixture inputs and observations must hold at the point of use"
)]
fn installed_capture_requires_trust_and_writes_only_current_bounded_receipts() {
    use std::os::unix::fs::PermissionsExt as _;

    let scratch = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let root = scratch.path().join("demo");
    let tools = scratch.path().join("tools");
    let cache = scratch.path().join("cache");
    let observed = scratch.path().join("arguments");
    let launched = scratch.path().join("launched");
    fs::create_dir_all(root.join("src")).unwrap();
    let root = root.canonicalize().unwrap();
    fs::create_dir_all(&tools).unwrap();
    fs::write(root.join("Cargo.toml"), MANIFEST).unwrap();
    fs::write(root.join("Cargo.lock"), "version = 4\n").unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn home() -> u32 { 7 }").unwrap();
    super::write_ledger(&root, &[]);
    for (tool, script) in [
        (
            "rustc",
            "#!/bin/sh\nprintf 'started\\n' > \"$RAF_TEST_LAUNCHED\"\nprintf '%s\\n' 'fixture rustc'\n",
        ),
        ("cargo-clippy", FIXTURE_CLIPPY),
    ] {
        let path = tools.join(tool);
        fs::write(&path, script).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let capture = |target: &Path, trust: bool, outcome: &str| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-raf-census"));
        command
            .current_dir(&root)
            .env_clear()
            .env("PATH", &tools)
            .env("CARGO_TARGET_DIR", &cache)
            .env("CARGO_ALIAS_CLIPPY", "test")
            .env("CARGO", "unexpected-cargo")
            .env("RAF_TEST_ARGS", &observed)
            .env("RAF_TEST_LAUNCHED", &launched)
            .env("RAF_TEST_OUTCOME", outcome)
            .arg("lint-capture");
        if trust {
            command.arg("--trust-lints");
        }
        super::alone(command.arg("--out").arg(target))
    };
    let receipt = scratch.path().join("production.ttl");
    let untrusted = capture(&receipt, false, "clean");
    assert_eq!(untrusted.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&untrusted.stderr).contains("--trust-lints"));
    let inside = capture(&root.join("inside.ttl"), true, "clean");
    assert_eq!(inside.status.code(), Some(1));
    assert!(!launched.exists() && !receipt.exists());

    let clean = capture(&receipt, true, "clean");
    assert!(clean.status.success(), "{clean:?}");
    assert_eq!(clean.stdout, [] as [u8; 0]);
    let actual = fs::read_to_string(&observed).unwrap();
    let mut expected = vec![root.to_str().unwrap(), cache.to_str().unwrap()];
    expected.extend_from_slice(PRODUCTION_CLIPPY_ARGS);
    assert_eq!(actual.lines().collect::<Vec<_>>(), expected);
    assert!(check(&root, Some(&receipt), &[]).status.success());

    let failed_path = scratch.path().join("failed.ttl");
    let failed = capture(&failed_path, true, "failed");
    assert_eq!(failed.status.code(), Some(23));
    let failure_check = check(&root, Some(&failed_path), &[]);
    assert_eq!(failure_check.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&failure_check.stdout).contains("fixture compiler result"));

    for (mode, diagnostic) in [
        ("overflow", "production output exceeds"),
        ("changed", "production inputs changed"),
    ] {
        let path = scratch.path().join(format!("{mode}.ttl"));
        let refusal = capture(&path, true, mode);
        assert_eq!(refusal.status.code(), Some(1), "{refusal:?}");
        assert!(
            String::from_utf8_lossy(&refusal.stderr).contains(diagnostic),
            "{refusal:?}"
        );
        assert!(!path.exists());
    }
}

const FIXTURE_CLIPPY: &str = r#"#!/bin/sh
set -eu
if [ "$2" = --version ]; then
    printf '%s\n' 'fixture clippy'
    exit 0
fi
test -z "$CARGO_ENCODED_RUSTFLAGS"
test -z "${CARGO+x}"
printf '%s\n' "$PWD" "$CARGO_TARGET_DIR" "$@" > "$RAF_TEST_ARGS"
printf '%s\n' 'fixture compiler result' >&2
case "$RAF_TEST_OUTCOME" in
    failed) exit 23 ;;
    overflow) printf '%1048577s' '' ;;
    changed) printf '%s\n' 'pub fn home() -> u32 { 8 }' > src/lib.rs ;;
esac
"#;

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn default_checks_require_passing_production_evidence_for_the_current_sources_and_policy() {
    // Both entry points must require a clean verdict for exactly the inspected inputs.
    let scratch = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let root = scratch.path().join("demo");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("Cargo.toml"), MANIFEST).unwrap();
    fs::write(root.join("Cargo.lock"), "version = 4\n").unwrap();
    fs::write(
        root.join("src/implementation.rs"),
        r#"
        pub struct Widget;
        impl Widget {
            #[cfg(test)] const DATA: &'static str = include_str!(concat!("test", ".txt"));
        }
        pub fn helper() {
            #[cfg(test)] let _data = include_bytes!(concat!("test", ".bin"));
        }"#,
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        r#"pub fn home() -> u32 { 7 } include!("implementation.rs",);
           #[cfg(test)] mod tests { const DATA: &str = include_str!(concat!("fixture", ".txt")); }"#,
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
    locked_shipping_inputs(&root);
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
    fs::write(
        root.join("Cargo.toml"),
        format!("{MANIFEST}\ncorrectness = {{ level = \"allow\", priority = 1 }}\n"),
    )
    .unwrap();
    assert_verdict(
        &root,
        &receipt,
        &capture(&root, 0, ""),
        Some("lints.clippy.all"),
    );
    let oversized = capture(&root, 0, &"x".repeat(raf_census::MAX_LEDGER_BYTES));
    assert!(
        oversized
            .to_turtle()
            .unwrap_err()
            .to_string()
            .contains("byte limit")
    );
    member_configuration(&root, &receipt);
    unverified_inputs(&root, &receipt);
    macro_call_site(&root);
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn locked_shipping_inputs(root: &Path) {
    let tree = DiskTree::new(root).unwrap();
    let expected = ProductionInputs::read(&tree).unwrap();
    fs::create_dir(root.join("tests")).unwrap();
    fs::write(root.join("tests/draft.rs"), "fn invalid( {").unwrap();
    assert_eq!(
        ProductionInputs::read(&tree).unwrap().digest(),
        expected.digest()
    );
    fs::remove_file(root.join("tests/draft.rs")).unwrap();
    fs::remove_file(root.join("Cargo.lock")).unwrap();
    let error = ProductionInputs::read(&tree).unwrap_err().to_string();
    assert!(error.contains("cargo generate-lockfile"), "{error}");
    fs::write(root.join("Cargo.lock"), "version = 4\n").unwrap();
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn macro_call_site(root: &Path) {
    // Rust resolves a macro-emitted inclusion at its call site. A same-named
    // file beside the definition cannot attest the bytes the compiler reads.
    let tree = DiskTree::new(root).unwrap();
    fs::create_dir(root.join("src/helpers")).unwrap();
    fs::write(root.join("src/helpers/message.txt"), "ab").unwrap();
    fs::write(root.join("src/message.txt"), "a").unwrap();
    let defs = root.join("src/helpers/defs.rs");
    let main = root.join("src/lib.rs");
    fs::write(&main, r#"#[macro_use] #[path = "helpers/defs.rs"] mod defs; pub const DATA: &[u8; 1] = message!();"#).unwrap();
    fs::write(
        &defs,
        r#"macro_rules! message { () => {{ include_bytes!("message.txt") }} }"#,
    )
    .unwrap();
    let error = ProductionInputs::read(&tree).unwrap_err().to_string();
    assert!(
        error.contains("expansion call site of include_bytes!"),
        "{error}"
    );
    fs::write(
        &defs,
        r#"macro_rules! message { ($read:ident) => { $read!("message.txt") } }"#,
    )
    .unwrap();
    fs::write(&main, r#"#[macro_use] #[path = "helpers/defs.rs"] mod defs; pub const DATA: &[u8; 1] = message!(include_bytes);"#).unwrap();
    let error = ProductionInputs::read(&tree).unwrap_err().to_string();
    assert!(error.contains("forwarded include_bytes"), "{error}");
    fs::write(
        &main,
        r#"macro_rules! keep { ($include:expr) => { $include }; }
           pub const DATA: &[u8; 1] = keep!(include_bytes!("message.txt"));
           pub fn skip(include: bool, enabled: bool) -> bool { return !(include && enabled); }"#,
    )
    .unwrap();
    let direct = ProductionInputs::read(&tree).unwrap();
    fs::write(root.join("src/message.txt"), "b").unwrap();
    assert_ne!(
        ProductionInputs::read(&tree).unwrap().digest(),
        direct.digest()
    );
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
    for (config, expected) in [
        ("env = ['RUSTFLAGS']\n", "env must be a table"),
        (
            "[target]\ninvalid = false\n",
            "target configuration must be a table",
        ),
    ] {
        fs::write(root.join(".cargo/config.toml"), config).unwrap();
        let error = ProductionInputs::read(&tree).unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
    }
    fs::write(root.join(".cargo/config"), "").unwrap();
    let effective = capture(root, 0, "");
    assert_verdict(root, receipt, &effective, None);
    fs::write(
        root.join(".cargo/config.toml"),
        "[build]\ntarget='ignored'\n",
    )
    .unwrap();
    assert_verdict(root, receipt, &effective, None);
    fs::remove_file(root.join(".cargo/config")).unwrap();
    fs::remove_file(root.join(".cargo/config.toml")).unwrap();
    let main = root.join("src/lib.rs");
    let source = fs::read_to_string(&main).unwrap();
    fs::write(&main, source.replace("#[cfg(test)]", "#[cfg(not(test))]")).unwrap();
    let error = ProductionInputs::read(&tree).unwrap_err().to_string();
    assert!(error.contains("dynamic include_str!"), "{error}");
    let rewritten = format!(
        "#[custom] mod rewritten {{ #[cfg(test)] const PATH: &str = include_str!(concat!(\"a\", \".rs\")); }}\n{source}"
    );
    fs::write(&main, rewritten).unwrap();
    let error = ProductionInputs::read(&tree).unwrap_err().to_string();
    assert!(error.contains("dynamic include_str!"), "{error}");
    fs::write(main, source).unwrap();
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
