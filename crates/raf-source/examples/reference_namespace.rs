// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Resolve an actual renamed reference export without configured provider proof.

use std::error::Error;
use std::fs;

use raf_source::{
    DiskTree, LoadOptions, LockedCatalog, LockedEdge, LockedManifest, ReferencePackageRoots,
    ReferenceTree, RelPath, Workspace,
};

fn main() -> Result<(), Box<dyn Error>> {
    let directory = purrdf_testkit::TempDir::new_in(std::env::temp_dir())?;
    for path in ["app/src", "helper/src"] {
        fs::create_dir_all(directory.path().join(path))?;
    }
    for (path, text) in [
        (
            "app/Cargo.toml",
            "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nrenamed={package='helper',version='1',path='../helper'}\n",
        ),
        ("app/src/lib.rs", "pub use renamed::exported as selected;\n"),
        (
            "helper/Cargo.toml",
            "[package]\nname='helper'\nversion='1.0.0'\nedition='2024'\n[lib]\nname='native_helper'\n",
        ),
        (
            "helper/src/lib.rs",
            "pub mod implementation {pub fn provided()->u32{7}}\npub use implementation::provided as exported;\n",
        ),
        (
            "app/Cargo.lock",
            "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['helper']\n[[package]]\nname='helper'\nversion='1.0.0'\n",
        ),
    ] {
        fs::write(directory.path().join(path), text)?;
    }
    let app = DiskTree::new(directory.path().join("app"))?;
    let helper = DiskTree::new(directory.path().join("helper"))?;
    let options = LoadOptions::default();
    let catalog = LockedCatalog::load(&app, &RelPath::new("Cargo.lock")?, &options)?;
    let app_row = catalog
        .packages()
        .find(|row| row.name() == "app")
        .ok_or("missing app lock row")?
        .id();
    let helper_row = catalog
        .packages()
        .find(|row| row.name() == "helper")
        .ok_or("missing helper lock row")?
        .id();
    let edges = [LockedEdge::new("dependencies.renamed", None)];
    let local = [LockedManifest::new(
        app_row,
        RelPath::new("Cargo.toml")?,
        &edges,
    )];
    let packages = [LockedManifest::new(
        helper_row,
        RelPath::new("Cargo.toml")?,
        &[],
    )];
    let references = [ReferencePackageRoots::new(
        ReferenceTree::Disk(&helper),
        &packages,
    )];
    let workspace = Workspace::load_reference_sources_under(
        &catalog,
        ReferenceTree::Disk(&app),
        &[RelPath::root()],
        &local,
        &references,
        &options,
    )?;
    let selected = workspace.resolve("app::selected")?;
    let provided = workspace.resolve("native_helper::implementation::provided")?;
    assert_eq!(selected.sites, provided.sites);
    let actual = selected
        .sites
        .first()
        .and_then(|id| workspace.item(*id))
        .ok_or("renamed export has no actual source item")?;
    assert_eq!(actual.path, "native_helper::implementation::provided");
    let binding = workspace
        .package_ids()
        .filter_map(|id| workspace.reference_binding(id))
        .find(|binding| binding.name() == "helper")
        .ok_or("missing helper reference identity")?;
    assert_eq!(binding.row(), helper_row);
    assert_eq!(binding.version(), "1.0.0");
    assert!(binding.source().is_none());
    assert!(
        workspace
            .package_ids()
            .all(|id| workspace.locked_binding(id).is_none())
    );
    println!(
        "app::selected resolves to {}; reference identity grants no configured binding",
        actual.path
    );
    Ok(())
}
