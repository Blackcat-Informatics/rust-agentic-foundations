// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Cargo field selection and scalar policy over the supplied native TOML arena.

use purrdf_toml::{
    Bounds, Document, Float, Kind, Node, Profile, Scalar, SourceDocument, TomlError, Vocabulary,
};

pub(crate) fn read(text: &str) -> Result<Document<'_>, TomlError> {
    let profile = Profile::new(
        "raf-cargo-toml",
        1,
        Vocabulary::standard()?,
        Bounds {
            max_source_bytes: text.len().max(1) as u64,
            max_depth: 128,
            ..Bounds::standard()
        },
    )?;
    let document = purrdf_toml::analyze(
        SourceDocument {
            id: "urn:raf:cargo-toml",
            bytes: text.as_bytes(),
        },
        &profile,
    )?;
    for node in document.nodes() {
        if matches!(
            node.kind,
            Kind::Scalar(Scalar::Float(Float::Infinity { .. }))
        ) && !["inf", "+inf", "-inf"].contains(&&text[node.span.clone()])
        {
            return Err(TomlError::Syntax {
                location: purrdf_toml::Location {
                    byte: node.span.start,
                    line: text[..node.span.start]
                        .bytes()
                        .filter(|byte| *byte == b'\n')
                        .count()
                        + 1,
                    column: text[..node.span.start]
                        .rsplit('\n')
                        .next()
                        .unwrap_or_default()
                        .chars()
                        .count()
                        + 1,
                },
                expected: "Cargo floating-point number must not overflow; explicit inf is required",
            });
        }
    }
    Ok(document)
}

pub(crate) fn field<'a>(document: &'a Document<'_>, table: &Node, key: &str) -> Option<&'a Node> {
    table
        .children
        .iter()
        .map(|id| &document.nodes()[*id])
        .find(|node| node.key.as_ref().is_some_and(|name| name.decoded == key))
}

pub(crate) fn string(node: &Node) -> Option<&str> {
    match &node.kind {
        Kind::Scalar(Scalar::String(text)) => Some(text),
        _ => None,
    }
}

pub(crate) fn boolean(node: &Node) -> Option<bool> {
    match node.kind {
        Kind::Scalar(Scalar::Boolean(value)) => Some(value),
        _ => None,
    }
}

pub(crate) fn table(node: &Node) -> Option<&Node> {
    matches!(node.kind, Kind::Table(_)).then_some(node)
}

pub(crate) fn array(node: &Node) -> Option<&[usize]> {
    matches!(node.kind, Kind::Array | Kind::ArrayOfTables).then_some(node.children.as_slice())
}

pub(crate) fn entries<'a>(document: &'a Document<'_>, table: &Node) -> Vec<(&'a str, &'a Node)> {
    let mut entries: Vec<_> = table
        .children
        .iter()
        .map(|id| {
            let node = &document.nodes()[*id];
            (
                node.key
                    .as_ref()
                    .expect("native table member key")
                    .decoded
                    .as_str(),
                node,
            )
        })
        .collect();
    // Cargo declarations previously used lexical key order. This is a schema
    // selection index, not another TOML value or assertion representation.
    purrdf_lex::walk::try_sort_unstable_by(&mut entries, |left, right| {
        Ok::<_, std::convert::Infallible>(left.0.cmp(right.0))
    })
    .expect("infallible key comparison");
    entries
}

/// Cargo manifest lint settings, including the workspace inheritance switch.
/// The native TOML arena preserves dotted and inline tables alike.
#[derive(Debug)]
pub struct CargoLintManifest<'a>(Document<'a>);

impl<'a> CargoLintManifest<'a> {
    /// Parse one Cargo manifest with the source index's native Cargo profile.
    ///
    /// # Errors
    /// Returns the native syntax or resource refusal for this manifest.
    pub fn parse(text: &'a str) -> Result<Self, TomlError> {
        read(text).map(Self)
    }

    fn lookup(&self, keys: &[&str]) -> Option<&Node> {
        keys.iter()
            .try_fold(&self.0.nodes()[0], |at, key| field(&self.0, at, key))
    }

    /// Whether this package inherits the workspace's lint tables.
    #[must_use]
    pub fn inherits(&self) -> bool {
        self.lookup(&["lints", "workspace"]).and_then(boolean) == Some(true)
    }

    /// A local Cargo configuration directive that can change which compiler,
    /// arguments or additional configuration define the production lint run.
    /// Ordinary cache, networking and registry settings do not change that selection.
    #[must_use]
    pub fn production_config_override(&self) -> Option<&str> {
        for (name, keys) in [
            ("include", &["include"][..]),
            ("alias.clippy", &["alias", "clippy"]),
            ("build.rustflags", &["build", "rustflags"]),
            ("build.target", &["build", "target"]),
            ("build.rustc", &["build", "rustc"]),
            ("build.rustc-wrapper", &["build", "rustc-wrapper"]),
            (
                "build.rustc-workspace-wrapper",
                &["build", "rustc-workspace-wrapper"],
            ),
        ] {
            if self.lookup(keys).is_some() {
                return Some(name);
            }
        }
        if let Some(targets) = self.lookup(&["target"]) {
            for (_, target) in entries(&self.0, targets) {
                if field(&self.0, target, "rustflags").is_some() {
                    return Some("target rustflags");
                }
            }
        }
        self.lookup(&["env"]).and_then(|env| {
            entries(&self.0, env)
                .into_iter()
                .map(|(key, _)| key)
                .find(|key| {
                    matches!(
                        *key,
                        "RUSTFLAGS"
                            | "RUSTC"
                            | "RUSTC_WRAPPER"
                            | "RUSTC_WORKSPACE_WRAPPER"
                            | "CARGO_ENCODED_RUSTFLAGS"
                            | "CARGO_BUILD_RUSTFLAGS"
                            | "CARGO_BUILD_TARGET"
                            | "CLIPPY_ARGS"
                            | "CLIPPY_CONF_DIR"
                    ) || (key.starts_with("CARGO_TARGET_") && key.ends_with("_RUSTFLAGS"))
                })
        })
    }

    /// Look up the effective level and numeric Cargo priority as declared in
    /// the workspace or package table. A scalar level has priority zero.
    /// Missing settings and invalid level/priority shapes have no effective value.
    #[must_use]
    pub fn setting(&self, workspace: bool, tool: &str, name: &str) -> Option<(&str, i64)> {
        let entry = if workspace {
            self.lookup(&["workspace", "lints", tool, name])
        } else {
            self.lookup(&["lints", tool, name])
        }?;
        if let Some(level) = string(entry) {
            return Some((level, 0));
        }
        let level = string(field(&self.0, entry, "level")?)?;
        let priority = match field(&self.0, entry, "priority").map(|node| &node.kind) {
            None => 0,
            Some(Kind::Scalar(Scalar::Integer(value))) => *value,
            _ => return None,
        };
        Some((level, priority))
    }
}
