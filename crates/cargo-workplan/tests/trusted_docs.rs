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

fn prepare(base: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let root = base.join("workspace");
    let target = base.join("artifacts");
    let tools = base.join("tools");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(target.join("doc/demo")).unwrap();
    fs::create_dir(&tools).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n",
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"), "/// API.\npub fn api() {}\n").unwrap();
    let html = target.join("doc/demo/index.html");
    fs::write(&html, "<!doctype html><title>controlled output</title>").unwrap();
    let manifest = json(root.join("Cargo.toml").to_str().unwrap());
    let source = json(root.join("src/lib.rs").to_str().unwrap());
    let target_shape = format!(
        "{{\"name\":\"demo\",\"kind\":[\"lib\"],\"crate_types\":[\"lib\"],\"src_path\":{source},\"doc\":true}}"
    );
    let metadata = format!(
        "{{\"version\":1,\"workspace_root\":{},\"target_directory\":{},\"workspace_members\":[\"opaque-demo\"],\"packages\":[{{\"id\":\"opaque-demo\",\"name\":\"demo\",\"version\":\"1.0.0\",\"manifest_path\":{manifest},\"source\":null,\"targets\":[{target_shape}]}}],\"resolve\":{{\"nodes\":[{{\"id\":\"opaque-demo\",\"features\":[]}}]}}}}",
        json(root.to_str().unwrap()),
        json(target.to_str().unwrap())
    );
    let messages = format!(
        "{{\"reason\":\"compiler-artifact\",\"package_id\":\"opaque-demo\",\"manifest_path\":{manifest},\"target\":{target_shape},\"features\":[],\"filenames\":[{}],\"fresh\":false}}\n{{\"reason\":\"build-finished\",\"success\":true}}\n",
        json(html.to_str().unwrap())
    );
    let calls = base.join("calls");
    let log = format!(
        "printf '%s\\n' \"$*\" >> {}",
        quoted(calls.to_str().unwrap())
    );
    script(
        &tools.join("cargo"),
        &format!(
            "{log}\ncase \"$*\" in\n  --version) printf 'controlled cargo\\n' ;;\n  *' metadata '*) printf '%s\\n' {} ;;\n  *' doc '*) printf '%s' {} ;;\n  *) exit 77 ;;\nesac",
            quoted(&metadata),
            quoted(&messages)
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

#[test]
fn documentation_trust_is_explicit_and_configuration_cannot_select_a_test_driver() {
    let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let base = scratch.path().canonicalize().unwrap();
    let (root, target, tools, calls) = prepare(&base);
    let run = |trust: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-workplan"));
        command.current_dir(&root).args([
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
        ]);
        if trust {
            command.arg("--trust-docs");
        }
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
        command.env("CARGO_HOME", base.join("cargo-home"));
        command.output().unwrap()
    };
    let refused = run(false);
    assert_eq!(refused.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("--trust-docs"));
    assert!(!calls.exists());
    let built = run(true);
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
    let competing = run(true);
    assert_eq!(competing.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&competing.stderr).contains("env.RUSTDOCFLAGS"));
    assert_eq!(fs::read_to_string(&calls).unwrap(), called);
}
