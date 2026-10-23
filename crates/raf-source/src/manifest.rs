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

/// Package version evidence, without synthesizing a missing identity.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum VersionDecl {
    Explicit(String),
    Inherited,
    Unset,
}

/// The table a dependency is declared in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyKind {
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
    pub(crate) declaration: DependencyDeclaration,
}

/// Written dependency constraints, not evidence of activation or source provenance.
///
/// Optional flags retain the distinction between omission and an explicit value.
/// Version requirements are validated by the maintained `SemVer` parser but retain
/// their written spelling. Git selectors remain declarations, not resolved commits.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct DependencySpec {
    /// Renamed package, when explicitly declared.
    pub package: Option<String>,
    /// Written version requirement.
    pub version: Option<String>,
    /// Path relative to the declaring manifest.
    pub path: Option<String>,
    /// Named registry; resolving Cargo configuration is a separate operation.
    pub registry: Option<String>,
    /// Explicit registry index, when supplied.
    pub registry_index: Option<String>,
    /// Git repository spelling.
    pub git: Option<String>,
    /// Git revision selector, which need not be a complete commit hash.
    pub rev: Option<String>,
    /// Git branch selector.
    pub branch: Option<String>,
    /// Git tag selector.
    pub tag: Option<String>,
    /// Explicit optional-dependency setting.
    pub optional: Option<bool>,
    /// Explicit default-feature setting.
    pub default_features: Option<bool>,
    /// Requested features, without computing feature unification.
    pub features: Vec<String>,
    /// Whether this entry inherits from `workspace.dependencies`.
    pub workspace: bool,
}

impl DependencySpec {
    pub(crate) fn heap_bytes(&self) -> usize {
        [
            &self.package,
            &self.version,
            &self.path,
            &self.registry,
            &self.registry_index,
            &self.git,
            &self.rev,
            &self.branch,
            &self.tag,
        ]
        .into_iter()
        .flatten()
        .map(String::len)
        .fold(
            self.features.len().saturating_mul(size_of::<String>()),
            usize::saturating_add,
        )
        .saturating_add(self.features.iter().map(String::len).sum::<usize>())
    }
}

/// The workspace entry inherited by a dependency, retaining its declaring base.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct InheritedDependency {
    /// Manifest whose `workspace.dependencies` supplies the constraints and path base.
    pub manifest: crate::RelPath,
    /// The workspace entry, before combining local features or flags.
    pub specification: DependencySpec,
}

/// One package dependency declaration, including its unevaluated target condition.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DependencyDeclaration {
    /// Original manifest key, useful for diagnostics.
    pub key: String,
    /// Dependency table kind.
    pub kind: DependencyKind,
    /// Written dependency alias, before Rust's hyphen normalization.
    pub name: String,
    /// Written target triple or `cfg(...)` table key; not an activation decision.
    pub condition: Option<String>,
    /// Constraints written in the package manifest.
    pub specification: DependencySpec,
    /// Resolved workspace declaration, when inheritance was requested.
    pub inherited: Option<InheritedDependency>,
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
    pub(crate) specification: DependencySpec,
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
    pub(crate) version: VersionDecl,
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
    /// Complete local Cargo feature declarations, before activation.
    pub(crate) features: BTreeMap<String, Vec<String>>,
    pub(crate) package: Option<PackageDecl>,
    pub(crate) has_workspace: bool,
    pub(crate) package_workspace: Option<String>,
    /// `[workspace.package] edition`, for the packages that inherit it.
    pub(crate) workspace_edition: Option<u16>,
    pub(crate) workspace_version: Option<String>,
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

fn dependency_spec(value: &Value, key: &str) -> Result<DependencySpec, ManifestError> {
    let spec = match value {
        Value::String(version) => DependencySpec {
            version: Some(version.clone()),
            ..DependencySpec::default()
        },
        Value::Table(table) => {
            let features = match table.get("features") {
                None => Vec::new(),
                Some(Value::Array(values)) => values
                    .iter()
                    .map(|value| {
                        value.as_str().map(str::to_owned).ok_or_else(|| {
                            at_key(&format!("{key}.features"), "must contain only strings")
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                Some(_) => {
                    return Err(at_key(
                        &format!("{key}.features"),
                        "must be an array of strings",
                    ));
                }
            };
            let workspace = optional_flag(table, key, "workspace")?;
            if workspace == Some(false) {
                return Err(at_key(
                    &format!("{key}.workspace"),
                    "must be true when present",
                ));
            }
            DependencySpec {
                package: renamed(table, key)?,
                version: string(table, key, "version")?,
                path: string(table, key, "path").map_err(|mut error| {
                    // Path declarations retain their established owning-entry origin.
                    error.key = Some(key.to_owned());
                    error.message = format!("`path` {}", error.message);
                    error
                })?,
                registry: string(table, key, "registry")?,
                registry_index: string(table, key, "registry-index")?,
                git: string(table, key, "git")?,
                rev: string(table, key, "rev")?,
                branch: string(table, key, "branch")?,
                tag: string(table, key, "tag")?,
                optional: optional_flag(table, key, "optional")?,
                default_features: optional_flag(table, key, "default-features")?,
                features,
                workspace: workspace == Some(true),
            }
        }
        _ => return Err(at_key(key, "dependency must be a version string or table")),
    };
    validate_dependency_spec(&spec, key)?;
    Ok(spec)
}

pub(crate) fn validate_dependency_spec(
    spec: &DependencySpec,
    key: &str,
) -> Result<(), ManifestError> {
    if let Some(version) = &spec.version {
        semver::VersionReq::parse(version).map_err(|error| {
            at_key(
                &format!("{key}.version"),
                format!("invalid version requirement: {error}"),
            )
        })?;
    }
    let selectors = [&spec.rev, &spec.branch, &spec.tag]
        .into_iter()
        .flatten()
        .count();
    if selectors > 1 || (selectors != 0 && spec.git.is_none()) {
        return Err(at_key(
            key,
            "a Git dependency requires `git` and at most one of `rev`, `branch`, or `tag`",
        ));
    }
    if spec.git.is_some()
        && (spec.path.is_some() || spec.registry.is_some() || spec.registry_index.is_some())
    {
        return Err(at_key(
            key,
            "`git` cannot be combined with a path or registry",
        ));
    }
    if spec.workspace
        && [
            &spec.package,
            &spec.version,
            &spec.path,
            &spec.registry,
            &spec.registry_index,
            &spec.git,
            &spec.rev,
            &spec.branch,
            &spec.tag,
        ]
        .into_iter()
        .any(Option::is_some)
    {
        return Err(at_key(
            key,
            "an inherited dependency cannot override its package or source constraints",
        ));
    }
    Ok(())
}

/// The dependencies of the three tables of `table`, whose keys start with
/// `prefix` in messages.
fn dependency_entries(
    table: &Table,
    prefix: &str,
    condition: Option<&str>,
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
            let specification = dependency_spec(entry, &key)?;
            let path = if specification.workspace {
                Some(PathSource::Workspace(name.clone()))
            } else {
                specification.path.clone().map(PathSource::Own)
            };
            out.push(Dependency {
                declaration: DependencyDeclaration {
                    key: key.clone(),
                    kind,
                    name: name.clone(),
                    condition: condition.map(str::to_owned),
                    specification: specification.clone(),
                    inherited: None,
                },
                key,
                kind,
                name: name.clone(),
                package: specification.package,
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
    dependency_entries(root, "", None, &mut found)?;
    if let Some(targets) = root.get("target") {
        for (platform, value) in table_of(targets, "target")? {
            let table = table_of(value, &format!("target.{platform}"))?;
            dependency_entries(
                table,
                &format!("target.{platform}."),
                Some(platform),
                &mut found,
            )?;
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

fn workspace_version(workspace: Option<&Table>) -> Result<Option<String>, ManifestError> {
    let version = workspace
        .and_then(|table| table.get("package"))
        .map(|value| table_of(value, "workspace.package"))
        .transpose()?
        .map(|table| string(table, "workspace.package", "version"))
        .transpose()?
        .flatten();
    if let Some(version) = &version {
        validate_version(version, "workspace.package.version")?;
    }
    Ok(version)
}

fn validate_version(version: &str, key: &str) -> Result<(), ManifestError> {
    semver::Version::parse(version)
        .map(|_| ())
        .map_err(|error| at_key(key, format!("invalid package version: {error}")))
}

fn package(root: &Table) -> Result<Option<PackageDecl>, ManifestError> {
    let Some(value) = root.get("package") else {
        return Ok(None);
    };
    let table = table_of(value, "package")?;
    let name =
        string(table, "package", "name")?.ok_or_else(|| at_key("package.name", "is missing"))?;
    let version = match table.get("version") {
        None => VersionDecl::Unset,
        Some(Value::String(version)) => {
            validate_version(version, "package.version")?;
            VersionDecl::Explicit(version.clone())
        }
        Some(Value::Table(inherited))
            if inherited.len() == 1
                && inherited.get("workspace") == Some(&Value::Boolean(true)) =>
        {
            VersionDecl::Inherited
        }
        Some(_) => {
            return Err(at_key(
                "package.version",
                "must be a version string or `{ workspace = true }`",
            ));
        }
    };
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
        version,
        auto_off,
        build,
        edition,
    }))
}

fn workspace_edition(root: &Table) -> Result<Option<u16>, ManifestError> {
    match root
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(Value::as_table)
        .and_then(|package| package.get("edition"))
    {
        None => Ok(None),
        Some(Value::String(edition)) => Ok(Some(year(edition, "workspace.package.edition")?)),
        Some(_) => Err(at_key("workspace.package.edition", "must be a string")),
    }
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
    if let Some(value) = root.get("workspace") {
        table_of(value, "workspace")?;
    }
    let package_workspace = root
        .get("package")
        .and_then(Value::as_table)
        .map(|table| string(table, "package", "workspace"))
        .transpose()?
        .flatten();
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
            let key = format!("workspace.dependencies.{name}");
            let specification = dependency_spec(entry, &key)?;
            if specification.workspace || specification.optional.is_some() {
                return Err(at_key(
                    &key,
                    "workspace dependencies cannot inherit or set `optional`",
                ));
            }
            workspace_dependencies.insert(
                name.clone(),
                WorkspaceDependency {
                    path: specification.path.clone(),
                    package: specification.package.clone(),
                    specification,
                },
            );
        }
    }
    let members = strings("members", MAX_MEMBERS)?;
    let exclude = strings("exclude", MAX_EXCLUDES)?;
    let workspace_edition = workspace_edition(&root)?;
    let workspace_version = workspace_version(workspace)?;
    if package.is_none() && !has_workspace {
        return Err(ManifestError {
            message: "has neither a [package] nor a [workspace] table".to_owned(),
            key: None,
        });
    }
    Ok(Manifest {
        features: feature_declarations(&root)?,
        package,
        has_workspace,
        package_workspace,
        workspace_edition,
        workspace_version,
        members,
        exclude,
        dependencies: dependencies(&root)?,
        workspace_dependencies,
        lib: library_declaration(&root)?,
        bins: declarations(&root, "bin")?,
        tests: declarations(&root, "test")?,
        benches: declarations(&root, "bench")?,
        examples: declarations(&root, "example")?,
    })
}

fn feature_declarations(root: &Table) -> Result<BTreeMap<String, Vec<String>>, ManifestError> {
    let mut features = BTreeMap::new();
    if let Some(value) = root.get("features") {
        for (name, value) in table_of(value, "features")? {
            let key = format!("features.{name}");
            let values = value
                .as_array()
                .ok_or_else(|| at_key(&key, "must be an array of feature references"))?;
            let mut references = Vec::new();
            for value in values {
                references.push(
                    value
                        .as_str()
                        .ok_or_else(|| at_key(&key, "must contain only strings"))?
                        .to_owned(),
                );
            }
            features.insert(name.clone(), references);
        }
    }
    Ok(features)
}

fn library_declaration(root: &Table) -> Result<Option<LibDecl>, ManifestError> {
    let Some(value) = root.get("lib") else {
        return Ok(None);
    };
    let table = table_of(value, "lib")?;
    Ok(Some(LibDecl {
        name: string(table, "lib", "name")?,
        path: string(table, "lib", "path")?,
        settings: settings(table, "lib")?,
    }))
}
