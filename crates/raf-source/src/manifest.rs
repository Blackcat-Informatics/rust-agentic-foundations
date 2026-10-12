// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The parts of a `Cargo.toml` that package discovery reads.
//!
//! The text is read with the `toml` crate, so comments, single-quoted strings,
//! dotted keys and inline tables are all honoured. The crate's own recursion
//! limit turns deep nesting into an error and not a crash. Every wrong value
//! is an error that names its key.

use toml::{Table, Value};

/// A manifest problem: what is wrong, and the key when one is at fault.
#[derive(Debug)]
pub(crate) struct ManifestError {
    pub(crate) message: String,
    pub(crate) key: Option<String>,
}

fn at_key(key: &str, message: impl Into<String>) -> ManifestError {
    ManifestError {
        message: message.into(),
        key: Some(key.to_owned()),
    }
}

/// How `package.build` names the build script.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Build {
    /// `build.rs` when it exists.
    Default,
    /// `build = false`.
    Disabled,
    /// `build = "path"`.
    Path(String),
}

/// A `[lib]` table.
#[derive(Debug, Default)]
pub(crate) struct LibDecl {
    pub(crate) name: Option<String>,
    pub(crate) path: Option<String>,
}

/// One `[[bin]]`, `[[test]]`, `[[bench]]` or `[[example]]` table.
#[derive(Debug)]
pub(crate) struct TargetDecl {
    pub(crate) name: Option<String>,
    pub(crate) path: Option<String>,
}

/// The `[package]` table.
#[derive(Debug)]
pub(crate) struct PackageDecl {
    pub(crate) name: String,
    /// The tables (`bin`, `test`, `bench`, `example`) whose automatic
    /// discovery `autobins = false` and its siblings turned off.
    pub(crate) auto_off: Vec<&'static str>,
    pub(crate) build: Build,
}

impl PackageDecl {
    /// True unless `auto<table>s = false` turned discovery off.
    pub(crate) fn auto(&self, table: &str) -> bool {
        !self.auto_off.contains(&table)
    }
}

/// A parsed manifest.
#[derive(Debug)]
pub(crate) struct Manifest {
    pub(crate) package: Option<PackageDecl>,
    pub(crate) has_workspace: bool,
    pub(crate) lib: Option<LibDecl>,
    pub(crate) bins: Vec<TargetDecl>,
    pub(crate) tests: Vec<TargetDecl>,
    pub(crate) benches: Vec<TargetDecl>,
    pub(crate) examples: Vec<TargetDecl>,
}

fn string(table: &Table, section: &str, key: &str) -> Result<Option<String>, ManifestError> {
    match table.get(key) {
        None => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(at_key(&format!("{section}.{key}"), "must be a string")),
    }
}

fn flag(table: &Table, section: &str, key: &str) -> Result<bool, ManifestError> {
    match table.get(key) {
        None => Ok(true),
        Some(Value::Boolean(value)) => Ok(*value),
        Some(_) => Err(at_key(&format!("{section}.{key}"), "must be a boolean")),
    }
}

fn table_of<'a>(value: &'a Value, key: &str) -> Result<&'a Table, ManifestError> {
    value
        .as_table()
        .ok_or_else(|| at_key(key, "must be a table"))
}

fn declarations(root: &Table, section: &str) -> Result<Vec<TargetDecl>, ManifestError> {
    let Some(value) = root.get(section) else {
        return Ok(Vec::new());
    };
    let rows = value
        .as_array()
        .ok_or_else(|| at_key(section, "must be an array of tables"))?;
    rows.iter()
        .map(|row| {
            let table = table_of(row, section)?;
            Ok(TargetDecl {
                name: string(table, section, "name")?,
                path: string(table, section, "path")?,
            })
        })
        .collect()
}

fn package(root: &Table) -> Result<Option<PackageDecl>, ManifestError> {
    let Some(value) = root.get("package") else {
        return Ok(None);
    };
    let table = table_of(value, "package")?;
    let name =
        string(table, "package", "name")?.ok_or_else(|| at_key("package.name", "is missing"))?;
    let build = match table.get("build") {
        None => Build::Default,
        Some(Value::Boolean(false)) => Build::Disabled,
        Some(Value::String(path)) => Build::Path(path.clone()),
        Some(_) => {
            return Err(at_key("package.build", "must be a path or false"));
        }
    };
    let mut auto_off = Vec::new();
    for (key, group) in [
        ("autobins", "bin"),
        ("autotests", "test"),
        ("autobenches", "bench"),
        ("autoexamples", "example"),
    ] {
        if !flag(table, "package", key)? {
            auto_off.push(group);
        }
    }
    Ok(Some(PackageDecl {
        name,
        auto_off,
        build,
    }))
}

/// Read a manifest from its text.
///
/// # Errors
///
/// Returns a [`ManifestError`] when the text is not TOML, is nested deeper than
/// the parser allows, has neither a `[package]` nor a `[workspace]` table, or
/// holds a value of the wrong type.
pub(crate) fn parse(text: &str) -> Result<Manifest, ManifestError> {
    let root: Table = text
        .parse()
        .map_err(|error: toml::de::Error| ManifestError {
            message: error.to_string(),
            key: None,
        })?;
    let package = package(&root)?;
    let has_workspace = root.contains_key("workspace");
    if package.is_none() && !has_workspace {
        return Err(ManifestError {
            message: "has neither a [package] nor a [workspace] table".to_owned(),
            key: None,
        });
    }
    let lib = match root.get("lib") {
        None => None,
        Some(value) => {
            let table = table_of(value, "lib")?;
            Some(LibDecl {
                name: string(table, "lib", "name")?,
                path: string(table, "lib", "path")?,
            })
        }
    };
    Ok(Manifest {
        package,
        has_workspace,
        lib,
        bins: declarations(&root, "bin")?,
        tests: declarations(&root, "test")?,
        benches: declarations(&root, "bench")?,
        examples: declarations(&root, "example")?,
    })
}
