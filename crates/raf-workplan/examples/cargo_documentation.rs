// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Capture actual selected Cargo documentation and preserve cumulative output.

use std::{error::Error, fs};

use purrdf_core::ContentDigest;
use raf_source::DiskTree;
use raf_workplan::{
    diagram::cargo,
    exec::{CargoDocOptions, Programs, SystemRunner},
};

const FILES: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        "[workspace]\nmembers=['alpha','beta']\nresolver='3'\n",
    ),
    (
        "Cargo.lock",
        "version=4\n[[package]]\nname='doc_capture_alpha'\nversion='1.0.0'\ndependencies=['doc_capture_beta']\n[[package]]\nname='doc_capture_beta'\nversion='2.0.0'\n",
    ),
    (
        "alpha/Cargo.toml",
        "[package]\nname='doc_capture_alpha'\nversion='1.0.0'\nedition='2024'\n[features]\nextended=[]\n[dependencies]\ndoc_capture_beta={path='../beta'}\n",
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
];

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::args_os().len() != 1 {
        return Err("usage: cargo_documentation (no arguments)".into());
    }
    let owned = purrdf_testkit::TempDir::new_in(std::env::temp_dir())?;
    let root = owned.path();
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
    let first = cargo::document(&runner, &selection)?;
    assert_eq!(first.artifacts().len(), 2);
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
    println!("captured two selected libraries and preserved cumulative documentation");
    Ok(())
}
