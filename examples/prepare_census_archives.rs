// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Prepare the exact registry bytes used by the repository's Census checks.

use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

use purrdf_core::ContentDigest;
use raf_source::{
    DiskTree, LoadOptions, LockedCatalog, LockedPackage, LockedSourceKind, RelPath, Tree as _,
};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [root, directory] = args.as_slice() else {
        return Err("give the repository root and build-owned archive directory".into());
    };
    let tree = DiskTree::new(root)?;
    let directory = Path::new(directory);
    let index = directory.join("crates-io");
    let options = LoadOptions::default();
    let limit = u64::try_from(options.max_total_bytes)?;
    owned_directory(directory)?;
    owned_directory(&index)?;
    let archives = DiskTree::new(&index)?;
    for lock in ["Cargo.lock", "fuzz/Cargo.lock"] {
        let catalog = LockedCatalog::load(&tree, &RelPath::new(lock)?, &options)?;
        for package in catalog
            .packages()
            .filter(|row| row.name().starts_with("purrdf-"))
        {
            prepare(package, &archives, &index, limit)?;
        }
    }
    Ok(())
}

fn owned_directory(path: &Path) -> Result<(), Box<dyn Error>> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(error)
            if error.kind() == ErrorKind::AlreadyExists && fs::symlink_metadata(path)?.is_dir() =>
        {
            Ok(())
        }
        Err(error) => {
            Err(format!("create build-owned directory {}: {error}", path.display()).into())
        }
    }
}

fn prepare(
    package: &LockedPackage,
    archives: &DiskTree,
    index: &Path,
    limit: u64,
) -> Result<(), Box<dyn Error>> {
    let name = package.name();
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(format!("invalid crates.io package name {name:?} in the lock file").into());
    }
    let version = package.version();
    let archive = format!("{name}-{version}.crate");
    let source = package
        .source()
        .ok_or("a locked PurRDF package needs its registry source")?;
    if source.kind() != LockedSourceKind::Registry
        || source.repository() != "https://github.com/rust-lang/crates.io-index"
    {
        return Err(format!("{archive} must name the crates.io registry in the lock file").into());
    }
    let checksum = package
        .checksum()
        .ok_or("a locked PurRDF package needs its archive checksum")?;
    let path = index.join(&archive);
    let relative = RelPath::new(&archive)?;
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() => {
            return verify(&archives.read_bytes(&relative, limit)?, &path, checksum);
        }
        Ok(_) => return Err(format!("{} must be a regular archive file", path.display()).into()),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(format!("read {}: {error}", path.display()).into()),
    }

    let staging = purrdf_testkit::TempDir::new_in(index)?;
    let pending = staging.path().join(&archive);
    let url = format!("https://static.crates.io/crates/{name}/{archive}");
    let status = Command::new("curl")
        .args([
            "--disable",
            "--fail",
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--max-filesize",
        ])
        .arg(limit.to_string())
        .arg("--output")
        .arg(&pending)
        .arg(&url)
        .status()
        .map_err(|error| format!("download {archive} with curl: {error}"))?;
    if !status.success() {
        return Err(format!("download of {url} failed with {status}").into());
    }
    let downloaded = DiskTree::new(staging.path())?.read_bytes(&relative, limit)?;
    verify(&downloaded, &pending, checksum)?;
    fs::rename(&pending, &path)?;
    Ok(())
}

fn verify(bytes: &[u8], path: &Path, checksum: &[u8; 32]) -> Result<(), Box<dyn Error>> {
    if ContentDigest::of(bytes).as_bytes() != checksum {
        return Err(format!(
            "{} differs from the locked archive checksum; remove it and run make registry-archives",
            path.display()
        )
        .into());
    }
    Ok(())
}
