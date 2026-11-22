// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Refuse competing compiler controls; this does not make build-time code safe.

use super::STREAM_BYTES;
use crate::exit::Failure;
use purrdf_toml::{Document, SourceDocument};
use std::collections::BTreeSet;
use std::io::Read as _;
use std::path::{Path, PathBuf};

fn compiler_environment(name: &str) -> bool {
    matches!(
        name,
        "RUSTC"
            | "RUSTDOC"
            | "RUSTC_WRAPPER"
            | "RUSTC_WORKSPACE_WRAPPER"
            | "RUSTFLAGS"
            | "CARGO_ENCODED_RUSTFLAGS"
            | "RUSTDOCFLAGS"
            | "CARGO_ENCODED_RUSTDOCFLAGS"
            | "CARGO_BUILD_RUSTC"
            | "CARGO_BUILD_RUSTDOC"
            | "CARGO_BUILD_RUSTC_WRAPPER"
            | "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"
            | "CARGO_BUILD_RUSTFLAGS"
            | "CARGO_BUILD_RUSTDOCFLAGS"
            | "CARGO_BUILD_TARGET"
            | "RUST_TARGET_PATH"
            | "CARGO_BUILD_BUILD_DIR"
            | "CARGO_BUILD_OUT_DIR"
            | "CARGO_BUILD_ARTIFACT_DIR"
    ) || (name.starts_with("CARGO_TARGET_")
        && (name.ends_with("_RUSTFLAGS") || name.ends_with("_RUSTDOCFLAGS")))
        || name.starts_with("RAF_RUSTDOC_")
}

pub(super) fn inspect(root: &Path) -> Result<(), Failure> {
    for (name, value) in std::env::vars_os() {
        let Some(name) = name.to_str() else {
            continue;
        };
        if compiler_environment(name) && !value.is_empty() {
            return Err(Failure::new(format!(
                "trusted documentation refuses competing environment control {name}; select tools and library scope through this invocation"
            )));
        }
    }
    let home = std::env::var_os("CARGO_HOME").map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
        .ok_or_else(|| Failure::new("set CARGO_HOME or HOME so trusted documentation can inspect Cargo's configuration scope"))?;
    if !home.is_absolute() {
        return Err(Failure::new("Cargo configuration home must be absolute"));
    }
    let mut directories = root
        .ancestors()
        .map(|path| path.join(".cargo"))
        .collect::<Vec<_>>();
    directories.push(home);
    let mut seen = BTreeSet::new();
    let mut remaining = STREAM_BYTES;
    for directory in directories {
        for name in ["config", "config.toml"] {
            let path = directory.join(name);
            if !seen.insert(path.clone()) {
                continue;
            }
            let file = match std::fs::File::open(&path) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(Failure::new(format!(
                        "cannot inspect Cargo configuration {}: {error}",
                        path.display()
                    )));
                }
            };
            if !file.metadata()?.is_file() {
                return Err(Failure::new("Cargo configuration must be a regular file"));
            }
            let mut bytes = Vec::new();
            file.take(u64::try_from(remaining)? + 1)
                .read_to_end(&mut bytes)?;
            remaining = remaining.checked_sub(bytes.len()).ok_or_else(|| {
                Failure::new("Cargo configurations exceed their cumulative 16 MiB inspection bound")
            })?;
            let profile = purrdf_toml::Profile::standard()?;
            let document = purrdf_toml::analyze(
                SourceDocument {
                    id: "urn:raf:trusted-docs:configuration",
                    bytes: &bytes,
                },
                &profile,
            )?;
            inspect_document(&document).map_err(|error| {
                Failure::new(format!("Cargo configuration {}: {error}", path.display()))
            })?;
        }
    }
    Ok(())
}

fn inspect_document(document: &Document<'_>) -> Result<(), Failure> {
    for node in document.nodes() {
        let Some(key) = node.key.as_ref() else {
            continue;
        };
        let mut path = vec![key.decoded.as_str()];
        let mut parent = node.parent;
        while let Some(index) = parent {
            let ancestor = &document.nodes()[index];
            if let Some(key) = &ancestor.key {
                path.push(key.decoded.as_str());
            }
            parent = ancestor.parent;
        }
        path.reverse();
        let controlled = match path.as_slice() {
            ["build", key] => matches!(
                *key,
                "rustc"
                    | "rustdoc"
                    | "rustc-wrapper"
                    | "rustc-workspace-wrapper"
                    | "rustflags"
                    | "rustdocflags"
                    | "target"
                    | "build-dir"
                    | "out-dir"
                    | "artifact-dir"
            ),
            ["target", _, key] => matches!(*key, "rustflags" | "rustdocflags"),
            ["env", key, ..] => compiler_environment(key),
            ["include" | "unstable", ..] | ["alias", "doc" | "metadata", ..] => true,
            _ => false,
        };
        if controlled {
            return Err(Failure::new(format!(
                "unsupported compiler, target or indirection control {}; trusted documentation has a closed tool/argument selection",
                path.join(".")
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admitted(text: &str) -> bool {
        let profile = purrdf_toml::Profile::standard().unwrap();
        let document = purrdf_toml::analyze(
            SourceDocument {
                id: "urn:raf:configuration-fixture",
                bytes: text.as_bytes(),
            },
            &profile,
        )
        .unwrap();
        inspect_document(&document).is_ok()
    }

    #[test]
    fn compiler_flags_wrappers_and_includes_cannot_change_the_closed_doc_operation() {
        for text in [
            "[build]\nrustdocflags=['--test']\n",
            "[build]\nrustc-wrapper='repository-wrapper'\n",
            "[target.'cfg(unix)']\nrustdocflags=['--test']\n",
            "[env]\nRUSTDOCFLAGS={value='--test',force=true}\n",
            "[env]\nCARGO_BUILD_RUSTDOC={value='repository-driver',force=true}\n",
            "include=['outside.toml']\n",
            "[unstable]\nconfig-include=true\n",
        ] {
            assert!(!admitted(text), "{text}");
        }
        assert!(admitted("[net]\noffline=true\n[build]\njobs=2\n"));
    }
}
