// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The parts of a `Cargo.toml` that package discovery reads.
//!
//! The text is read with the `toml` crate, so comments, single-quoted strings,
//! dotted keys and inline tables are all honoured. The crate's own recursion
//! limit turns deep nesting into an error and not a crash. Every wrong value
//! is an error that names its key.

use std::collections::BTreeMap;

use toml::{Table, Value};

/// A manifest problem: what is wrong, and the key when one is at fault.
#[derive(Debug)]
pub(crate) struct ManifestError {
    pub(crate) message: String,
    pub(crate) key: Option<String>,
}

/// The most entries `workspace.members` may have.
pub(crate) const MAX_MEMBERS: usize = 4096;

/// The most entries `workspace.exclude` may have.
pub(crate) const MAX_EXCLUDES: usize = 4096;

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

/// The table a dependency is declared in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DependencyKind {
    /// `[dependencies]`.
    Normal,
    /// `[dev-dependencies]`.
    Dev,
    /// `[build-dependencies]`.
    Build,
}

/// One dependency of a package, from any of its tables, the `[target.*]`
/// ones included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Dependency {
    /// The table and name, such as `dependencies.helper`, for messages.
    pub(crate) key: String,
    /// The table it is declared in.
    pub(crate) kind: DependencyKind,
    /// The name it is declared under.
    pub(crate) name: String,
    /// `package = ".."`: the package the entry renames, when it renames one.
    pub(crate) package: Option<String>,
    /// Where the dependency gets its directory, when it names one.
    pub(crate) path: Option<PathSource>,
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

/// An entry of `[workspace.dependencies]`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorkspaceDependency {
    /// Its `path`, relative to the workspace manifest.
    pub(crate) path: Option<String>,
    /// `package = ".."`, when it renames a package.
    pub(crate) package: Option<String>,
}

/// The test settings of a target table, each `None` (or empty) when the
/// table leaves it to cargo's default.
#[derive(Debug, Default)]
pub(crate) struct TargetSettings {
    /// `harness`.
    pub(crate) harness: Option<bool>,
    /// `required-features`.
    pub(crate) required_features: Vec<String>,
    /// `test`.
    pub(crate) test: Option<bool>,
    /// `doctest`.
    pub(crate) doctest: Option<bool>,
}

/// A `[lib]` table.
#[derive(Debug, Default)]
pub(crate) struct LibDecl {
    pub(crate) name: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) settings: TargetSettings,
}

/// One `[[bin]]`, `[[test]]`, `[[bench]]` or `[[example]]` table.
#[derive(Debug)]
pub(crate) struct TargetDecl {
    pub(crate) name: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) settings: TargetSettings,
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
    /// The dependencies of the package, in all dependency tables.
    pub(crate) dependencies: Vec<Dependency>,
    /// The `[workspace.dependencies]` entries, by name.
    pub(crate) workspace_dependencies: BTreeMap<String, WorkspaceDependency>,
    /// `workspace.exclude`.
    pub(crate) exclude: Vec<String>,
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

fn optional_flag(table: &Table, section: &str, key: &str) -> Result<Option<bool>, ManifestError> {
    match table.get(key) {
        None => Ok(None),
        Some(Value::Boolean(value)) => Ok(Some(*value)),
        Some(_) => Err(at_key(&format!("{section}.{key}"), "must be a boolean")),
    }
}

/// The test settings of the target table `table` of `section`.
fn settings(table: &Table, section: &str) -> Result<TargetSettings, ManifestError> {
    if section == "lib" && table.contains_key("required-features") {
        // Cargo refuses the key for a library.
        return Err(at_key(
            "lib.required-features",
            "is not allowed for a library target",
        ));
    }
    let required_features = match table.get("required-features") {
        None => Vec::new(),
        Some(Value::Array(features)) => features
            .iter()
            .map(|feature| match feature {
                Value::String(name) => Ok(name.clone()),
                _ => Err(at_key(
                    &format!("{section}.required-features"),
                    "must be an array of strings",
                )),
            })
            .collect::<Result<_, _>>()?,
        Some(_) => {
            return Err(at_key(
                &format!("{section}.required-features"),
                "must be an array of strings",
            ));
        }
    };
    Ok(TargetSettings {
        harness: optional_flag(table, section, "harness")?,
        required_features,
        test: optional_flag(table, section, "test")?,
        doctest: optional_flag(table, section, "doctest")?,
    })
}

fn table_of<'a>(value: &'a Value, key: &str) -> Result<&'a Table, ManifestError> {
    value
        .as_table()
        .ok_or_else(|| at_key(key, "must be a table"))
}

/// The value of `package` in a dependency table at `key`, if it has one.
fn renamed(details: &Table, key: &str) -> Result<Option<String>, ManifestError> {
    match details.get("package") {
        None => Ok(None),
        Some(Value::String(package)) => Ok(Some(package.clone())),
        Some(_) => Err(at_key(key, "`package` must be a string")),
    }
}

/// The dependencies of the three tables of `table`, whose keys start with
/// `prefix` in messages.
fn dependency_entries(
    table: &Table,
    prefix: &str,
    out: &mut Vec<Dependency>,
) -> Result<(), ManifestError> {
    for (section, kind) in [
        ("dependencies", DependencyKind::Normal),
        ("dev-dependencies", DependencyKind::Dev),
        ("build-dependencies", DependencyKind::Build),
    ] {
        let Some(value) = table.get(section) else {
            continue;
        };
        for (name, entry) in table_of(value, &format!("{prefix}{section}"))? {
            let key = format!("{prefix}{section}.{name}");
            let (package, path) = match entry {
                Value::Table(details) => {
                    let path = match (details.get("path"), details.get("workspace")) {
                        (Some(Value::String(path)), _) => Some(PathSource::Own(path.clone())),
                        (Some(_), _) => return Err(at_key(&key, "`path` must be a string")),
                        (None, Some(Value::Boolean(true))) => {
                            Some(PathSource::Workspace(name.clone()))
                        }
                        (None, _) => None,
                    };
                    (renamed(details, &key)?, path)
                }
                _ => (None, None),
            };
            out.push(Dependency {
                key,
                kind,
                name: name.clone(),
                package,
                path,
            });
        }
    }
    Ok(())
}

/// The dependencies of a manifest: the three tables at the top and the same
/// three under each `[target.<platform>]`.
fn dependencies(root: &Table) -> Result<Vec<Dependency>, ManifestError> {
    let mut found = Vec::new();
    dependency_entries(root, "", &mut found)?;
    if let Some(targets) = root.get("target") {
        for (platform, value) in table_of(targets, "target")? {
            let table = table_of(value, &format!("target.{platform}"))?;
            dependency_entries(table, &format!("target.{platform}."), &mut found)?;
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
                settings: settings(table, section)?,
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
    let strings = |key: &str, cap: usize| -> Result<Vec<String>, ManifestError> {
        let Some(value) = workspace.and_then(|table| table.get(key)) else {
            return Ok(Vec::new());
        };
        let full = format!("workspace.{key}");
        let array = value
            .as_array()
            .ok_or_else(|| at_key(&full, "must be an array of strings"))?;
        if array.len() > cap {
            return Err(at_key(
                &full,
                format!("has {} entries; at most {cap} are allowed", array.len()),
            ));
        }
        array
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| at_key(&full, "must be an array of strings"))
            })
            .collect()
    };
    let mut workspace_dependencies = BTreeMap::new();
    if let Some(value) = workspace.and_then(|table| table.get("dependencies")) {
        for (name, entry) in table_of(value, "workspace.dependencies")? {
            // A version alone names no path and renames nothing.
            let Value::Table(details) = entry else {
                workspace_dependencies.insert(name.clone(), WorkspaceDependency::default());
                continue;
            };
            let key = format!("workspace.dependencies.{name}");
            let path = match details.get("path") {
                None => None,
                Some(Value::String(path)) => Some(path.clone()),
                Some(_) => return Err(at_key(&key, "`path` must be a string")),
            };
            workspace_dependencies.insert(
                name.clone(),
                WorkspaceDependency {
                    path,
                    package: renamed(details, &key)?,
                },
            );
        }
    }
    let members = strings("members", MAX_MEMBERS)?;
    let exclude = strings("exclude", MAX_EXCLUDES)?;
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
                settings: settings(table, "lib")?,
            })
        }
    };
    Ok(Manifest {
        package,
        has_workspace,
        workspace_edition,
        members,
        exclude,
        dependencies: dependencies(&root)?,
        workspace_dependencies,
        lib,
        bins: declarations(&root, "bin")?,
        tests: declarations(&root, "test")?,
        benches: declarations(&root, "bench")?,
        examples: declarations(&root, "example")?,
    })
}
