// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fixed, controlled tool fixtures exercise invocation-scoped documentation trust.

#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
    process::Command,
};

mod support;

fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\"'\"'"))
}
fn json(text: &str) -> String {
    purrdf_lex::json::write_compact(&purrdf_lex::json::Value::from(text))
}
fn script(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\nset -eu\n{body}\n")).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn carrier(name: &str, package: &Path, html: &Path) -> (String, String, String) {
    let manifest = json(package.join("Cargo.toml").to_str().unwrap());
    let source = json(package.join("src/lib.rs").to_str().unwrap());
    let target = format!(
        r#"{{"name":"{name}","kind":["lib"],"crate_types":["lib"],"src_path":{source},"doc":true}}"#
    );
    let id = format!(r#""opaque-{name}""#);
    let package = format!(
        r#"{{"id":{id},"name":"{name}","version":"1.0.0","manifest_path":{manifest},"source":null,"targets":[{target}]}}"#
    );
    let message = format!(
        r#"{{"reason":"compiler-artifact","package_id":{id},"manifest_path":{manifest},"target":{target},"features":[],"filenames":[{}],"fresh":false}}"#,
        json(html.to_str().unwrap())
    );
    (id, package, message)
}

fn prepare(base: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let root = base.join("workspace");
    let target = base.join("artifacts");
    let tools = base.join("tools");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("other/src")).unwrap();
    let first_entry = target.join("doc/custom output/entry.html");
    let second_entry = target.join("doc/another/guide#api.html");
    for html in [&first_entry, &second_entry] {
        fs::create_dir_all(html.parent().unwrap()).unwrap();
        fs::write(html, "<!doctype html><title>controlled output</title>").unwrap();
    }
    fs::create_dir(&tools).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n\n[workspace]\nmembers=['other']\n",
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"), "/// API.\npub fn api() {}\n").unwrap();
    fs::write(
        root.join("other/Cargo.toml"),
        "[package]\nname='other'\nversion='1.0.0'\nedition='2024'\n",
    )
    .unwrap();
    fs::write(
        root.join("other/src/lib.rs"),
        "/// Other API.\npub fn api() {}\n",
    )
    .unwrap();
    let (first, demo, first_message) = carrier("demo", &root, &first_entry);
    let (second, other, second_message) = carrier("other", &root.join("other"), &second_entry);
    let metadata = format!(
        r#"{{"version":1,"workspace_root":{},"target_directory":{},"workspace_members":[{first},{second}],"packages":[{demo},{other}],"resolve":{{"nodes":[{{"id":{first},"features":[]}},{{"id":{second},"features":[]}}]}}}}"#,
        json(root.to_str().unwrap()),
        json(target.to_str().unwrap())
    );
    let messages = base.join("messages.jsonl");
    fs::write(
        &messages,
        format!("{first_message}\n{second_message}\n{{\"reason\":\"build-finished\",\"success\":true}}\n"),
    ).unwrap();
    let calls = base.join("calls");
    let log = format!(
        "printf '%s\\n' \"$*\" >> {}",
        quoted(calls.to_str().unwrap())
    );
    script(
        &tools.join("cargo"),
        &format!(
            "{log}\ncase \"$*\" in\n  --version) printf 'controlled cargo\\n' ;;\n  *' metadata '*) printf '%s\\n' {} ;;\n  *' doc '*) cat {} ;;\n  *) exit 77 ;;\nesac",
            quoted(&metadata),
            quoted(messages.to_str().unwrap())
        ),
    );
    for name in ["rustc", "rustdoc"] {
        script(
            &tools.join(name),
            &format!("{log}\nprintf 'controlled {name}\\n'"),
        );
    }
    (root, target, tools, calls)
}

fn run(root: &Path, target: &Path, tools: &Path, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-workplan"));
    command
        .current_dir(root)
        .args([
            "--root",
            root.to_str().unwrap(),
            "docs",
            "build",
            "rustdoc",
            "--package",
            "demo",
            "--target-directory",
            target.to_str().unwrap(),
            "--toolchain-directory",
            tools.to_str().unwrap(),
        ])
        .args(args);
    // This fixture selects its own complete compiler-control environment.
    for name in [
        "RUSTC",
        "RUSTDOC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTDOCFLAGS",
        "CARGO_ENCODED_RUSTDOCFLAGS",
        "CARGO_BUILD_RUSTC",
        "CARGO_BUILD_RUSTDOC",
        "CARGO_BUILD_RUSTC_WRAPPER",
        "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
        "CARGO_BUILD_RUSTFLAGS",
        "CARGO_BUILD_RUSTDOCFLAGS",
        "CARGO_BUILD_TARGET",
        "RUST_TARGET_PATH",
        "CARGO_BUILD_BUILD_DIR",
        "CARGO_BUILD_OUT_DIR",
        "CARGO_BUILD_ARTIFACT_DIR",
    ] {
        command.env_remove(name);
    }
    command.env("CARGO_HOME", tools.join("cargo-home"));
    command.output().unwrap()
}

#[test]
fn documentation_trust_is_explicit_and_configuration_cannot_select_a_test_driver() {
    let scratch = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let base = scratch.path().canonicalize().unwrap();
    let (root, target, tools, calls) = prepare(&base);
    let run = |args: &[&str]| run(&root, &target, &tools, args);
    let refused = run(&[]);
    assert_eq!(refused.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("--trust-docs"));
    assert!(!calls.exists());
    let built = run(&["--trust-docs"]);
    assert!(built.status.success(), "{built:?}");
    let called = fs::read_to_string(&calls).unwrap();
    assert!(
        called
            .lines()
            .any(|line| line.contains(" doc --frozen --offline ")
                && line.contains("--no-deps --lib --message-format=json")),
        "{called}"
    );
    assert!(
        called.lines().all(|line| !line.contains("--test")
            && !line.contains("--example")
            && !line.contains("--all-targets")),
        "{called}"
    );
    fs::create_dir(root.join(".cargo")).unwrap();
    fs::write(
        root.join(".cargo/config.toml"),
        "[env]\nRUSTDOCFLAGS={value='--test',force=true}\n",
    )
    .unwrap();
    let competing = run(&["--trust-docs"]);
    assert_eq!(competing.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&competing.stderr).contains("env.RUSTDOCFLAGS"));
    assert_eq!(fs::read_to_string(&calls).unwrap(), called);
}

/// The trusted path publishes only after admitting all actual reported entries.
#[test]
fn trusted_rustdoc_publishes_the_named_view_beside_selected_library_entries() {
    let scratch = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let base = scratch.path().canonicalize().unwrap();
    let (root, target, tools, calls) = prepare(&base);
    let doc = support::capability(
        "demo",
        "considered",
        ["[]", "[]", "[]"],
        "[]",
        &[("Verification", support::BLOCKS)],
    )
    .replace("bindings:", "diagrams:\n  - name: dependencies\n    kind: capability-dependencies\n    scope: workspace\nbindings:");
    fs::create_dir_all(root.join("docs/capabilities")).unwrap();
    fs::write(root.join("docs/capabilities/demo.md"), doc).unwrap();
    let host = target.join("doc");
    let unrelated = host.join("keep.html");
    fs::write(&unrelated, "unrelated cumulative output").unwrap();
    let run = |view| {
        run(
            &root,
            &target,
            &tools,
            &["--trust-docs", "--package", "other", "--view", view],
        )
    };

    let missing = run("cap:demo#missing");
    assert_eq!(missing.status.code(), Some(2), "{missing:?}");
    assert!(
        String::from_utf8_lossy(&missing.stderr).contains("missing"),
        "{missing:?}"
    );
    assert_eq!(missing.stdout, []);
    assert!(!calls.exists(), "invalid view launched documentation tools");

    let view = "cap:demo#dependencies";
    let built = run(view);
    assert!(built.status.success(), "{built:?}");
    let stdout = String::from_utf8(built.stdout).unwrap();
    let entry = stdout
        .lines()
        .find_map(|line| line.strip_prefix("published cap:demo#dependencies at "))
        .unwrap();
    let entry = Path::new(entry);
    assert!(entry.starts_with(host.join("raf-workplan")), "{entry:?}");
    let published = fs::read_to_string(entry).unwrap();
    assert!(
        published.contains("href=\"../../custom%20output/entry.html\""),
        "{published}"
    );
    assert!(
        published.contains("href=\"../../another/guide%23api.html\""),
        "{published}"
    );
    assert!(!published.contains("keep.html"));
    assert_eq!(
        fs::read_to_string(&unrelated).unwrap(),
        "unrelated cumulative output"
    );
    let retry = run(view);
    assert!(retry.status.success(), "{retry:?}");
    assert_eq!(retry.stdout, stdout.as_bytes());
    assert_eq!(fs::read_to_string(entry).unwrap(), published);

    let messages = base.join("messages.jsonl");
    let other = target.join("other-doc");
    fs::create_dir(&other).unwrap();
    fs::rename(host.join("another"), other.join("another")).unwrap();
    fs::write(
        &messages,
        fs::read_to_string(&messages)
            .unwrap()
            .replace("doc/another", "other-doc/another"),
    )
    .unwrap();
    let mixed = run(view);
    assert_eq!(mixed.status.code(), Some(2), "{mixed:?}");
    assert_eq!(mixed.stdout, []);
    assert!(
        String::from_utf8_lossy(&mixed.stderr).contains("shared documentation output directory"),
        "{mixed:?}"
    );
    assert!(!other.join("raf-workplan").exists());
    assert_eq!(fs::read_to_string(entry).unwrap(), published);
}
