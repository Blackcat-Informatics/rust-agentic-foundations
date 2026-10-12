// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The parts of a `Cargo.toml` that package discovery reads.
//!
//! The text is read with the `toml` crate, so comments, single-quoted strings,
//! dotted keys and inline tables are all honoured. The crate's own recursion
//! limit turns deep nesting into an error and not a crash. Every wrong value
//! is an error that names its key.

use std::collections::{BTreeMap, BTreeSet};

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

/// Where a package gets its edition.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EditionDecl {
    /// `edition = "2021"`.
    Explicit(u16),
    /// `edition.workspace = true`.
    Inherited,
    /// No `edition` key, which cargo reads as 2015.
    Unset,
}

/// The names of the crates a package declares, as they are written in code.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Dependencies {
    pub(crate) normal: BTreeSet<String>,
    pub(crate) dev: BTreeSet<String>,
    pub(crate) build: BTreeSet<String>,
}

/// A dependency that names a directory, and where its path comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PathDependency {
    /// The table and name, such as `dependencies.helper`, for messages.
    pub(crate) key: String,
    pub(crate) path: PathSource,
}

/// Where a path dependency gets its directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathSource {
    /// `path = ".."`, relative to the manifest's directory.
    Own(String),
    /// `workspace = true`: the dependency of that name in
    /// `[workspace.dependencies]`, relative to the workspace manifest.
    Workspace(String),
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
    pub(crate) edition: EditionDecl,
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
    /// `[workspace.package] edition`, for the packages that inherit it.
    pub(crate) workspace_edition: Option<u16>,
    /// `workspace.members`, when the manifest has a `[workspace]` table.
    pub(crate) members: Vec<String>,
    /// The path dependencies of the package, in all dependency tables.
    pub(crate) path_dependencies: Vec<PathDependency>,
    /// `[workspace.dependencies]` entries that have a `path`, by name.
    pub(crate) workspace_paths: BTreeMap<String, String>,
    /// `workspace.exclude`.
    pub(crate) exclude: Vec<String>,
    pub(crate) dependencies: Dependencies,
    pub(crate) lib: Option<LibDecl>,
    pub(crate) bins: Vec<TargetDecl>,
    pub(crate) tests: Vec<TargetDecl>,
    pub(crate) benches: Vec<TargetDecl>,
    pub(crate) examples: Vec<TargetDecl>,
}

/// An edition such as `2021` as its year.
fn year(text: &str, key: &str) -> Result<u16, ManifestError> {
    text.parse()
        .map_err(|_| at_key(key, "must be a year such as \"2021\""))
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

/// The keys of one dependency table, as crate names in code.
fn dependency_keys(root: &Table, key: &str) -> Result<BTreeSet<String>, ManifestError> {
    match root.get(key) {
        None => Ok(BTreeSet::new()),
        Some(value) => Ok(table_of(value, key)?
            .keys()
            .map(|name| name.replace('-', "_"))
            .collect()),
    }
}

/// The dependencies of a manifest: the three tables at the top and the same
/// three under each `[target.<platform>]`.
fn dependencies(root: &Table) -> Result<Dependencies, ManifestError> {
    let mut found = Dependencies {
        normal: dependency_keys(root, "dependencies")?,
        dev: dependency_keys(root, "dev-dependencies")?,
        build: dependency_keys(root, "build-dependencies")?,
    };
    if let Some(targets) = root.get("target") {
        for (platform, value) in table_of(targets, "target")? {
            let table = table_of(value, &format!("target.{platform}"))?;
            found.normal.extend(dependency_keys(table, "dependencies")?);
            found
                .dev
                .extend(dependency_keys(table, "dev-dependencies")?);
            found
                .build
                .extend(dependency_keys(table, "build-dependencies")?);
        }
    }
    Ok(found)
}

/// The dependencies of `table` that name a directory.
fn path_entries(
    table: &Table,
    prefix: &str,
    out: &mut Vec<PathDependency>,
) -> Result<(), ManifestError> {
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(value) = table.get(section) else {
            continue;
        };
        for (name, entry) in table_of(value, &format!("{prefix}{section}"))? {
            let Value::Table(details) = entry else {
                continue;
            };
            let key = format!("{prefix}{section}.{name}");
            match (details.get("path"), details.get("workspace")) {
                (Some(Value::String(path)), _) => out.push(PathDependency {
                    key,
                    path: PathSource::Own(path.clone()),
                }),
                (Some(_), _) => return Err(at_key(&key, "`path` must be a string")),
                (None, Some(Value::Boolean(true))) => out.push(PathDependency {
                    key,
                    path: PathSource::Workspace(name.clone()),
                }),
                (None, _) => {}
            }
        }
    }
    Ok(())
}

fn path_dependencies(root: &Table) -> Result<Vec<PathDependency>, ManifestError> {
    let mut found = Vec::new();
    path_entries(root, "", &mut found)?;
    if let Some(targets) = root.get("target") {
        for (platform, value) in table_of(targets, "target")? {
            let table = table_of(value, &format!("target.{platform}"))?;
            path_entries(table, &format!("target.{platform}."), &mut found)?;
        }
    }
    Ok(found)
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
    let edition = match table.get("edition") {
        None => EditionDecl::Unset,
        Some(Value::String(edition)) => EditionDecl::Explicit(year(edition, "package.edition")?),
        Some(Value::Table(inherit)) if inherit.get("workspace") == Some(&Value::Boolean(true)) => {
            EditionDecl::Inherited
        }
        Some(_) => {
            return Err(at_key(
                "package.edition",
                "must be a string or `workspace = true`",
            ));
        }
    };
    Ok(Some(PackageDecl {
        name,
        auto_off,
        build,
        edition,
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
    let workspace = root.get("workspace").and_then(Value::as_table);
    let strings = |key: &str| -> Result<Vec<String>, ManifestError> {
        let Some(value) = workspace.and_then(|table| table.get(key)) else {
            return Ok(Vec::new());
        };
        let full = format!("workspace.{key}");
        value
            .as_array()
            .ok_or_else(|| at_key(&full, "must be an array of strings"))?
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| at_key(&full, "must be an array of strings"))
            })
            .collect()
    };
    let mut workspace_paths = BTreeMap::new();
    if let Some(value) = workspace.and_then(|table| table.get("dependencies")) {
        for (name, entry) in table_of(value, "workspace.dependencies")? {
            let Value::Table(details) = entry else {
                continue;
            };
            match details.get("path") {
                None => {}
                Some(Value::String(path)) => {
                    workspace_paths.insert(name.clone(), path.clone());
                }
                Some(_) => {
                    return Err(at_key(
                        &format!("workspace.dependencies.{name}"),
                        "`path` must be a string",
                    ));
                }
            }
        }
    }
    let members = strings("members")?;
    let exclude = strings("exclude")?;
    let workspace_edition = match root
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(Value::as_table)
        .and_then(|package| package.get("edition"))
    {
        None => None,
        Some(Value::String(edition)) => Some(year(edition, "workspace.package.edition")?),
        Some(_) => return Err(at_key("workspace.package.edition", "must be a string")),
    };
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
        workspace_edition,
        members,
        exclude,
        path_dependencies: path_dependencies(&root)?,
        workspace_paths,
        dependencies: dependencies(&root)?,
        lib,
        bins: declarations(&root, "bin")?,
        tests: declarations(&root, "test")?,
        benches: declarations(&root, "bench")?,
        examples: declarations(&root, "example")?,
    })
}
