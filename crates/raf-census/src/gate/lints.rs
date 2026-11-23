// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Production-only Cargo policy and the inputs a trusted build must identify.

use crate::{Error, ReportError};
use purrdf_core::{ContentDigest, ContentHasher};
use raf_source::{CargoLintManifest, LoadOptions, RelPath, TargetKind, Tree, Workspace};
use std::collections::{BTreeMap, BTreeSet};
mod includes;

/// The compiler arguments required on the externally executed production pass.
pub const PRODUCTION_CLIPPY_ARGS: &[&str] = &[
    "clippy",
    "--workspace",
    "--lib",
    "--bins",
    "--locked",
    "--target",
    "host-tuple",
    "--",
    "-D",
    "warnings",
    "-D",
    "unused",
    "-D",
    "unreachable-pub",
    "-D",
    "clippy::allow_attributes",
    "-D",
    "clippy::used_underscore_binding",
    "-D",
    "clippy::needless_pass_by_value",
    "-D",
    "clippy::trivially_copy_pass_by_ref",
    "-D",
    "clippy::redundant_clone",
];

const REQUIRED: &[(&str, &str, &str)] = &[
    ("rust", "unused", "deny"),
    ("rust", "dead_code", "deny"),
    ("rust", "unreachable_pub", "deny"),
    ("rust", "unused_crate_dependencies", "warn"),
    ("clippy", "all", "deny"),
    ("clippy", "pedantic", "warn"),
    ("clippy", "allow_attributes", "deny"),
    ("clippy", "allow_attributes_without_reason", "deny"),
    ("clippy", "used_underscore_binding", "deny"),
    ("clippy", "dbg_macro", "deny"),
    ("clippy", "todo", "deny"),
    ("clippy", "unimplemented", "deny"),
];

/// Selected workspace inputs, production target identities and manifest findings.
#[derive(Debug)]
pub struct ProductionInputs {
    digest: ContentDigest,
    findings: Vec<String>,
}

fn frame(hash: &mut ContentHasher, bytes: &[u8]) {
    hash.update(&(bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}

fn file_error(path: &RelPath, source: raf_source::TreeError) -> Error {
    Error::Tree {
        file: path.to_string(),
        source,
    }
}

fn manifest<'a>(path: &RelPath, text: &'a str) -> Result<CargoLintManifest<'a>, Error> {
    CargoLintManifest::parse(text).map_err(|source| Error::LintManifest {
        file: path.to_string(),
        source,
    })
}

fn policy(
    lints: &CargoLintManifest<'_>,
    inherited: bool,
    path: &RelPath,
    findings: &mut Vec<String>,
) {
    for &(tool, name, minimum) in REQUIRED {
        let entry = lints.setting(inherited, tool, name);
        let sufficient = entry.is_some_and(|(level, priority)| {
            let level_ok =
                matches!(level, "deny" | "forbid") || (minimum == "warn" && level == "warn");
            let group = matches!(
                (tool, name),
                ("clippy", "all" | "pedantic") | ("rust", "unused")
            );
            let competing_groups: &[&str] = match (tool, name) {
                ("rust", "dead_code") => &["unused"],
                ("clippy", "all") => &["correctness", "suspicious", "style", "complexity", "perf"],
                ("clippy", "pedantic") | ("rust", _) => &[],
                ("clippy", "used_underscore_binding") => &["pedantic"],
                ("clippy", _) => &["restriction"],
                _ => &[],
            };
            level_ok
                && (!group || priority < 0)
                && competing_groups.iter().all(|group_name| {
                    lints.setting(inherited, tool, group_name).is_none_or(
                        |(group_level, group_priority)| {
                            matches!(group_level, "deny" | "forbid") || group_priority < priority
                        },
                    )
                })
        });
        if !sufficient {
            let table = if inherited {
                "workspace.lints"
            } else {
                "lints"
            };
            findings.push(format!("{path}: production lint policy requires {table}.{tool}.{name} at {minimum} or stronger, with groups below individual lint priority; found {entry:?}"));
        }
    }
}

fn read_configuration(
    tree: &impl Tree,
    workspace: &Workspace,
    max_bytes: u64,
    files: &mut BTreeMap<RelPath, ContentDigest>,
) -> Result<(), Error> {
    let mut paths = BTreeSet::new();
    let legacy =
        RelPath::new(".cargo/config").map_err(|error| Error::Options(error.to_string()))?;
    // Cargo selects the extensionless configuration when both names exist.
    let config = if tree
        .entry(&legacy)
        .map_err(|e| file_error(&legacy, e))?
        .is_some()
    {
        ".cargo/config"
    } else {
        ".cargo/config.toml"
    };
    for name in [
        "Cargo.lock",
        config,
        "rust-toolchain",
        "rust-toolchain.toml",
    ] {
        paths.insert(RelPath::new(name).map_err(|error| Error::Options(error.to_string()))?);
    }
    // Clippy searches upwards from each member's manifest directory.
    let mut directories = BTreeSet::new();
    for package in workspace.packages() {
        for dir in std::iter::successors(package.manifest.parent(), RelPath::parent) {
            if !directories.insert(dir.clone()) {
                break;
            }
            for name in [".clippy.toml", "clippy.toml"] {
                paths.insert(
                    dir.join(name)
                        .map_err(|error| Error::Options(error.to_string()))?,
                );
            }
        }
    }
    for path in paths {
        let present = tree
            .entry(&path)
            .map_err(|e| file_error(&path, e))?
            .is_some();
        if !present && path.as_str() == "Cargo.lock" {
            return Err(Error::Options("production Clippy requires Cargo.lock; run cargo generate-lockfile in the workspace root".into()));
        }
        if present {
            let bytes = tree
                .read_bytes(&path, max_bytes)
                .map_err(|e| file_error(&path, e))?;
            if matches!(path.as_str(), ".cargo/config" | ".cargo/config.toml") {
                let text = std::str::from_utf8(&bytes).map_err(|e| {
                    Error::Options(format!("{path}: Cargo configuration must be UTF-8: {e}"))
                })?;
                let configuration = CargoLintManifest::parse_config(text).map_err(|source| {
                    Error::LintManifest {
                        file: path.to_string(),
                        source,
                    }
                })?;
                if let Some(key) = configuration.production_config_override() {
                    return Err(Error::Options(format!(
                        "{path}: {key} can change the production compiler invocation; supply a build configuration with the default production target and lint flags"
                    )));
                }
            }
            files.insert(path, ContentDigest::of(&bytes));
        }
    }
    Ok(())
}

impl ProductionInputs {
    /// Index the declared Cargo members and identify the selected production sources,
    /// target declarations, manifests, lockfile and workspace compiler configuration.
    /// No compiler or inspected program is executed.
    ///
    /// # Errors
    /// Refuses missing inputs and every source discovery, parser or resource failure.
    pub fn read(tree: &impl Tree) -> Result<Self, Error> {
        let options = LoadOptions::default();
        let workspace = Workspace::load_shipping_members(tree, &RelPath::root(), &options)?;
        let root = RelPath::new("Cargo.toml").map_err(|error| Error::Options(error.to_string()))?;
        let root_text = tree
            .read(&root, options.max_file_bytes)
            .map_err(|e| file_error(&root, e))?;
        let workspace_lints = manifest(&root, &root_text)?;
        let mut files = BTreeMap::new();
        files.insert(root.clone(), ContentDigest::of(root_text.as_bytes()));
        let mut findings = Vec::new();
        for package in workspace.packages() {
            let path = &package.manifest;
            let text = tree
                .read(path, options.max_file_bytes)
                .map_err(|e| file_error(path, e))?;
            let local = manifest(path, &text)?;
            if local.inherits() {
                policy(&workspace_lints, true, &root, &mut findings);
            } else {
                policy(&local, false, path, &mut findings);
            }
            files.insert(path.clone(), ContentDigest::of(text.as_bytes()));
        }
        for file in workspace.files().filter(|file| file.shipping) {
            files.insert(file.path.clone(), ContentDigest::of(file.text.as_bytes()));
        }
        includes::add(tree, &workspace, &options, &mut files)?;
        read_configuration(tree, &workspace, options.max_file_bytes, &mut files)?;
        let mut hash = ContentHasher::default();
        frame(&mut hash, b"raf-census-production-clippy-v1");
        for arg in PRODUCTION_CLIPPY_ARGS {
            frame(&mut hash, arg.as_bytes());
        }
        let mut targets = Vec::new();
        for script in workspace
            .targets()
            .filter(|target| target.kind == TargetKind::BuildScript)
        {
            findings.push(format!("{}: a first-party build script can read inputs outside the module tree; its compiler inputs cannot be established by this receipt", script.root));
        }
        for target in workspace
            .targets()
            .filter(|target| matches!(target.kind, TargetKind::Lib | TargetKind::Bin))
        {
            if !target.required_features.is_empty() {
                findings.push(format!("{}: cannot certify default production coverage for target {} with required features {}", target.root, target.name, target.required_features.join(", ")));
            }
            let package = workspace.package(target.package_id).ok_or_else(|| {
                Error::Options(format!("{}: target has no workspace package", target.root))
            })?;
            let kind = if target.kind == TargetKind::Lib {
                "lib"
            } else {
                "bin"
            };
            targets.push((&package.manifest, kind, &target.name, &target.root));
        }
        targets.sort_unstable();
        if targets.is_empty() {
            findings
                .push("production lint policy: workspace has no library or binary targets".into());
        }
        for (package, kind, name, root) in targets {
            frame(&mut hash, b"target");
            for field in [package.as_str(), kind, name.as_str(), root.as_str()] {
                frame(&mut hash, field.as_bytes());
            }
        }
        for (path, digest) in files {
            frame(&mut hash, b"file");
            frame(&mut hash, path.as_str().as_bytes());
            frame(&mut hash, digest.as_bytes());
        }
        findings.sort();
        findings.dedup();
        Ok(Self {
            digest: hash.finish(),
            findings,
        })
    }

    /// Digest of the production selection used before and after the compiler run.
    #[must_use]
    pub const fn digest(&self) -> &ContentDigest {
        &self.digest
    }
}

/// Result supplied by the trusted build that ran production Clippy. Reading a
/// receipt validates its contents. The caller selects the trusted channel.
#[derive(Debug, Clone)]
pub struct ProductionLintRun {
    inputs: ContentDigest,
    compiler: String,
    status: u8,
    diagnostics: String,
}

impl ProductionLintRun {
    /// Read an explicitly selected build-owned receipt outside the workspace.
    ///
    /// # Errors
    /// Refuses relative paths, receipts inside the workspace, and invalid or oversized input.
    pub fn read_external(root: &std::path::Path, path: &std::path::Path) -> Result<Self, Error> {
        if !path.is_absolute() {
            return Err(Error::Options(
                "production lint evidence requires an absolute build-owned path".into(),
            ));
        }
        let canonical = path.canonicalize().map_err(|source| Error::Io {
            file: path.display().to_string(),
            source,
        })?;
        let root = root.canonicalize().map_err(|source| Error::Io {
            file: root.display().to_string(),
            source,
        })?;
        if canonical.starts_with(&root) {
            return Err(Error::Options(
                "the inspected workspace cannot supply its own production lint receipt".into(),
            ));
        }
        let parent = canonical
            .parent()
            .ok_or_else(|| Error::Options("expected a receipt file".into()))?;
        let name = canonical
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| Error::Options("expected a UTF-8 receipt filename".into()))?;
        let name = RelPath::new(name).map_err(|e| Error::Options(e.to_string()))?;
        let tree = raf_source::DiskTree::new(parent).map_err(|source| Error::Tree {
            file: parent.display().to_string(),
            source,
        })?;
        let text = tree
            .read(&name, crate::MAX_LEDGER_BYTES as u64)
            .map_err(|e| file_error(&name, e))?;
        Self::from_turtle(&text)
    }

    /// Record a completed external command against the inputs it inspected.
    ///
    /// # Errors
    /// Refuses a different command, absent compiler identity or changed inputs.
    pub fn capture(
        before: &str,
        after: &ProductionInputs,
        args: &[&str],
        compiler: String,
        status: u8,
        diagnostics: String,
    ) -> Result<Self, Error> {
        if args != PRODUCTION_CLIPPY_ARGS {
            return Err(Error::Options("production lint evidence requires the exact production-only cargo clippy command and denied lints".into()));
        }
        if before != after.digest().to_hex() {
            return Err(Error::Options(
                "production inputs changed while Clippy ran; repeat the build on a fixed snapshot"
                    .into(),
            ));
        }
        if compiler.trim().is_empty() || (status != 0 && diagnostics.trim().is_empty()) {
            return Err(Error::Options("production lint evidence requires the compiler identity and diagnostics for a failed run".into()));
        }
        Ok(Self {
            inputs: *after.digest(),
            compiler,
            status,
            diagnostics,
        })
    }

    /// Emit canonical RDF for the caller's build-owned evidence destination.
    ///
    /// # Errors
    /// Returns a native RDF error or refuses output larger than the reader admits.
    pub fn to_turtle(&self) -> Result<String, Error> {
        use crate::report::write;
        let mut graph = raf_rdf::Builder::new();
        write::class(&mut graph, "", "ProductionLintRun")?;
        for (name, value) in [
            ("command", PRODUCTION_CLIPPY_ARGS.join(" ").as_str()),
            ("inputs", self.inputs.to_hex().as_str()),
            ("compiler", self.compiler.as_str()),
            ("exitCode", self.status.to_string().as_str()),
            ("diagnostics", self.diagnostics.as_str()),
        ] {
            write::text(&mut graph, "", name, value)?;
        }
        let text = graph
            .finish()
            .and_then(|g| g.to_text(raf_rdf::Format::Turtle))
            .map_err(ReportError::Ground)?;
        Self::check_size(&text)?;
        Ok(text)
    }

    fn check_size(text: &str) -> Result<(), Error> {
        if text.len() > crate::MAX_LEDGER_BYTES {
            return Err(Error::Options(
                "production lint evidence exceeds its byte limit".into(),
            ));
        }
        Ok(())
    }

    /// Read the canonical, closed receipt selected by the calling build.
    ///
    /// # Errors
    /// Refuses malformed, incomplete or unknown fields and other target selections.
    pub fn from_turtle(text: &str) -> Result<Self, Error> {
        use crate::report::shape;
        Self::check_size(text)?;
        let graph = raf_rdf::Graph::from_text(text, raf_rdf::Format::Turtle)
            .map_err(ReportError::Ground)?;
        let root = graph
            .get("")
            .ok_or_else(|| Error::Options("production lint evidence lacks its root".into()))?;
        if shape::class(&root)? != "ProductionLintRun"
            || shape::string(&root, "command")? != PRODUCTION_CLIPPY_ARGS.join(" ")
        {
            return Err(Error::Options(
                "production lint evidence has the wrong class, command or target selection".into(),
            ));
        }
        let digest = shape::string(&root, "inputs")?;
        let inputs = ContentDigest::from_hex(&digest)
            .ok_or_else(|| Error::Options("invalid production lint input digest".into()))?;
        let compiler = shape::string(&root, "compiler")?;
        let status = shape::string(&root, "exitCode")?
            .parse::<u8>()
            .map_err(|cause| {
                Error::Options(format!("invalid production Clippy exit status: {cause}"))
            })?;
        let diagnostics = shape::string(&root, "diagnostics")?;
        let run = Self {
            inputs,
            compiler,
            status,
            diagnostics,
        };
        if run.compiler.trim().is_empty()
            || (status != 0 && run.diagnostics.trim().is_empty())
            || run.to_turtle()? != text
        {
            return Err(Error::Options(
                "production lint evidence is incomplete or noncanonical".into(),
            ));
        }
        Ok(run)
    }
}

/// Explicit selection of external compiler evidence for a static check.
#[derive(Default)]
pub(super) enum Enforcement {
    Disabled,
    #[default]
    Required,
    Supplied(ProductionLintRun),
}

impl Enforcement {
    pub(super) fn findings(&self, tree: &impl Tree) -> Result<Vec<String>, Error> {
        if matches!(self, Self::Disabled) {
            return Ok(Vec::new());
        }
        let mut inputs = ProductionInputs::read(tree)?;
        match self {
            Self::Supplied(run) if run.inputs != inputs.digest => inputs.findings.push("stale production lint evidence: repeat Clippy on this workspace snapshot".into()),
            Self::Supplied(run) if run.status != 0 => inputs.findings.push(format!("production Clippy failed (exit {}): {}", run.status, run.diagnostics)),
            Self::Supplied(_) => (),
            Self::Required => inputs.findings.push("missing production lint evidence: supply the result of the production-only Clippy build".into()),
            Self::Disabled => unreachable!("INVARIANT: disabled enforcement returned before loading"),
        }
        Ok(inputs.findings)
    }
}
