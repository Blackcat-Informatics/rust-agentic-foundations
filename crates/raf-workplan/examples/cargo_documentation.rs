// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Capture actual selected Cargo documentation and preserve cumulative output.

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use purrdf_core::ContentDigest;
use raf_source::{DiskTree, RelPath, Workspace};
use raf_workplan::{
    Workplan,
    diagram::cargo::{self, assembly, observation},
    exec::{CargoDocOptions, Programs, RustdocObservationOptions, SystemRunner},
};

const FILES: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        "[workspace]\nmembers=['alpha','beta','unselected']\nresolver='3'\n",
    ),
    (
        "Cargo.lock",
        "version=4\n[[package]]\nname='doc_capture_alpha'\nversion='1.0.0'\ndependencies=['doc_capture_beta']\n[[package]]\nname='doc_capture_beta'\nversion='2.0.0'\n[[package]]\nname='doc_capture_unselected'\nversion='3.0.0'\ndependencies=['doc_capture_alpha']\n",
    ),
    (
        "alpha/Cargo.toml",
        "[package]\nname='doc_capture_alpha'\nversion='1.0.0'\nedition='2024'\n[features]\nextended=[]\nextra=[]\n[dependencies]\ndoc_capture_beta={path='../beta'}\n",
    ),
    (
        "alpha/src/lib.rs",
        "#[cfg(doc)] pub struct DocumentationOnly;\n#[cfg(feature=\"extended\")] pub struct FeatureOnly;\ninclude!(concat!(env!(\"OUT_DIR\"),\"/generated.rs\"));\nmacro_rules! expose { () => { pub struct Expanded; }; }\nexpose!();\nmod internal { pub struct Definition; }\npub use internal::Definition as PublicAlias;\n",
    ),
    (
        "alpha/build.rs",
        "fn main() { let output=std::path::PathBuf::from(std::env::var_os(\"OUT_DIR\").expect(\"Cargo OUT_DIR\")); std::fs::write(output.join(\"generated.rs\"),\"pub struct Generated;\").expect(\"generated source\"); }\n",
    ),
    (
        "beta/Cargo.toml",
        "[package]\nname='doc_capture_beta'\nversion='2.0.0'\nedition='2024'\n",
    ),
    ("beta/src/lib.rs", "pub struct Companion;\n"),
    (
        "unselected/Cargo.toml",
        "[package]\nname='doc_capture_unselected'\nversion='3.0.0'\nedition='2024'\n[dependencies]\ndoc_capture_alpha={path='../alpha',features=['extra']}\n",
    ),
    ("unselected/src/lib.rs", "pub struct Unselected;\n"),
];

fn main() -> Result<(), Box<dyn Error>> {
    if let Some(result) = observation::dispatch_driver() {
        std::process::exit(result?);
    }
    if std::env::args_os().len() != 1 {
        return Err("usage: cargo_documentation (no arguments)".into());
    }
    let owned = purrdf_testkit::TempDir::new_in(std::env::temp_dir())?;
    let canonical_root = owned.path().canonicalize()?;
    let root = canonical_root.as_path();
    for (name, contents) in FILES {
        let path = root.join(name);
        // Every constant fixture name is a nonempty child of the owned root.
        fs::create_dir_all(path.parent().expect("fixture file has a parent"))?;
        fs::write(path, contents)?;
    }
    let runner = SystemRunner::new(Programs::new(root));
    let options = CargoDocOptions::new(
        &["doc_capture_alpha", "doc_capture_beta"],
        &["doc_capture_alpha/extended"],
        false,
        None,
    )?;
    let selection = cargo::capture(&runner, root, &options)?;
    assert!(
        selection.libraries()[0]
            .features
            .iter()
            .any(|feature| feature == "extra")
    );
    // Deliberately select Rustup's native documentation compiler. This does not
    // discover or preserve an ambient Cargo compiler override.
    let located = Command::new("rustup").args(["which", "rustdoc"]).output()?;
    if !located.status.success() {
        return Err(format!(
            "Rustdoc selection failed: {}",
            String::from_utf8_lossy(&located.stderr)
        )
        .into());
    }
    let compiler = PathBuf::from(std::str::from_utf8(&located.stdout)?.trim());
    let controls = RustdocObservationOptions::new(
        &std::env::current_exe()?,
        &compiler,
        &root.join("compiler-receipts"),
    )?;
    let observed = observation::document(&runner, &selection, &controls)?;
    assert_eq!(observed.invocations().len(), 2);
    for invocation in observed.invocations() {
        assert!(
            std::str::from_utf8(&invocation.configuration_output().stdout)?
                .lines()
                .any(|predicate| predicate == "doc")
        );
    }
    let first = observed.documentation();
    assert_eq!(first.artifacts().len(), 2);
    assert!(
        first.artifacts()[0]
            .features()
            .iter()
            .any(|feature| feature == "extended")
    );
    assert!(
        !first.artifacts()[0]
            .features()
            .iter()
            .any(|feature| feature == "extra")
    );
    let disk = DiskTree::new(first.selection().target_directory())?;
    let alpha = disk.read_bytes(first.artifacts()[0].relative_path(), 16 * 1024 * 1024)?;
    let html = std::str::from_utf8(&alpha)?;
    for name in [
        "DocumentationOnly",
        "FeatureOnly",
        "Generated",
        "Expanded",
        "PublicAlias",
    ] {
        assert!(html.contains(name), "selected HTML omitted {name}");
    }
    let beta = &first.artifacts()[1];
    let options = CargoDocOptions::new(
        &["doc_capture_alpha"],
        &["doc_capture_alpha/extended"],
        false,
        Some(first.selection().target_directory()),
    )?;
    let selected = cargo::capture(&runner, root, &options)?;
    let second = cargo::document(&runner, &selected)?;
    assert_eq!(second.artifacts().len(), 1);
    assert_eq!(
        ContentDigest::of(&disk.read_bytes(beta.relative_path(), 16 * 1024 * 1024)?),
        beta.sha256(),
        "documenting a selected library removed or changed unrelated HTML"
    );
    literal_includes(root, &runner, &options, &compiler)?;
    println!("captured two selected libraries and preserved cumulative documentation");
    Ok(())
}

// REUSE-IgnoreStart
const LITERAL_CAPABILITY: &str = r"---
# SPDX-FileCopyrightText: 2026 Example
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:alpha
title: Documented library
description: A selected library with a shared documentation include.
state: implementing
diagrams:
  - name: dependencies
    kind: capability-dependencies
    scope: workspace
bindings:
  code: [doc_capture_alpha::Documented]
  tests: []
  examples: []
depends: []
---

## Purpose

Show the selected library's shared diagram.

## Contract

The public Documented struct appears in documentation.

## Usage

Read the library's root documentation.

## Examples

The dependencies view includes this capability.

## Constraints

Only the selected compiler context admits API links.

## Explanation

A literal include supplies the root documentation.

## Verification

Actual Rustdoc consumes the required struct link.
";
// REUSE-IgnoreEnd

fn literal_includes(
    root: &Path,
    runner: &SystemRunner,
    options: &CargoDocOptions,
    compiler: &Path,
) -> Result<(), Box<dyn Error>> {
    fs::write(
        root.join("alpha/src/lib.rs"),
        "#![doc=include_str!(\"../../includes/alpha.md\")]\n#[cfg(doc)]\n/// The documented API.\npub struct Documented;\n",
    )?;
    fs::create_dir_all(root.join("includes"))?;
    fs::create_dir_all(root.join("docs/capabilities"))?;
    fs::write(root.join("docs/capabilities/alpha.md"), LITERAL_CAPABILITY)?;
    let tree = DiskTree::new(root)?;
    let source = Workspace::load(&tree, &["alpha", "beta"])?;
    let seed_model = Workplan::load(&tree, &source);
    let selection = cargo::capture(runner, root, options)?;
    let library = &selection.libraries()[0];
    let request = assembly::IncludeRequest::new(
        &library.package_id,
        &library.name,
        RelPath::new("includes/alpha.md")?,
        "cap:alpha#dependencies",
        "cap:alpha",
        vec!["doc_capture_alpha::Documented".into()],
    );
    let plan = assembly::AssemblyPlan::new(&selection, vec![request])?;
    let controls = RustdocObservationOptions::new(
        &std::env::current_exe()?,
        compiler,
        &root.join("assembly-receipts"),
    )?;
    let assembled = assembly::document(
        runner,
        &selection,
        &controls,
        &plan,
        &root.join("assembly.nt"),
        &seed_model,
    )?;
    assert_eq!(assembled.publications().len(), 1);
    assert!(assembled.publications()[0].join("manifest.ttl").is_file());
    let include = fs::read_to_string(root.join("includes/alpha.md"))?;
    assert!(include.contains("(struct@crate::Documented)"));
    Ok(())
}
