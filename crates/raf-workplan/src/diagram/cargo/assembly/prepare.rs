// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Source admission and include generation at the cfg-to-HTML handoff.

use super::{
    AssemblyPlan, DiagramError, MAX_BYTES, configuration::Configuration, read_plan, receiver,
    record, refused, source::Observed,
};
use crate::diagram::cargo::observation::{driver, receipt};
use crate::diagram::{DiagramBundle, rustdoc};
use crate::exec::{Output, RustdocObservationOptions};
use purrdf_core::ContentDigest;
use raf_rdf::{Builder, Format, Graph};
use raf_source::{EntryKind, FileId, LoadOptions, RelPath, Tree, Workspace};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Real pre-HTML preparation evidence, separate from emitted-link admission.
#[derive(Debug, Clone)]
pub struct Preparation {
    pub(in crate::diagram::cargo) plan: String,
    pub(super) package: String,
    host: Output,
    pub(crate) html_arguments: Vec<String>,
    pub(super) sources: Vec<(String, String)>,
    pub(super) fragments: Vec<(String, String, String)>,
}

impl Preparation {
    /// Successful verbose-version information query from the selected compiler.
    #[must_use]
    pub const fn compiler_version(&self) -> &Output {
        &self.host
    }
    /// Actual assembly HTML arguments, including required-link lint enforcement.
    #[must_use]
    pub fn html_arguments(&self) -> &[String] {
        &self.html_arguments
    }
    /// Workspace-relative read paths and native content digests; no atomicity claim.
    #[must_use]
    pub fn source_observations(&self) -> &[(String, String)] {
        &self.sources
    }
    /// Workspace-relative includes with exact fragment and shared bundle digests.
    #[must_use]
    pub fn fragments(&self) -> &[(String, String, String)] {
        &self.fragments
    }

    fn graph(&self) -> Result<Graph, DiagramError> {
        let mut builder = Builder::new();
        for (key, value) in [
            ("form", "1"),
            ("plan", self.plan.as_str()),
            ("package", self.package.as_str()),
        ] {
            record::put(&mut builder, "", key, value)?;
        }
        record::words(&mut builder, "", "htmlArgument", &self.html_arguments)?;
        record::put(
            &mut builder,
            "",
            "sourceCount",
            &self.sources.len().to_string(),
        )?;
        for (index, (path, digest)) in self.sources.iter().enumerate() {
            let subject = format!("#source/{index}");
            record::put(&mut builder, &subject, "path", path)?;
            record::put(&mut builder, &subject, "digest", digest)?;
        }
        record::put(
            &mut builder,
            "",
            "fragmentCount",
            &self.fragments.len().to_string(),
        )?;
        for (index, (path, digest, bundle)) in self.fragments.iter().enumerate() {
            let subject = format!("#fragment/{index}");
            for (key, value) in [("path", path), ("digest", digest), ("bundle", bundle)] {
                record::put(&mut builder, &subject, key, value)?;
            }
        }
        receipt::output(&mut builder, "assemblyHost", &self.host)?;
        builder.finish().map_err(DiagramError::data)
    }

    pub(in crate::diagram::cargo) fn encode(&self) -> Result<String, DiagramError> {
        let text = self
            .graph()?
            .to_text(Format::NTriples)
            .map_err(DiagramError::data)?;
        if text.len() > MAX_BYTES {
            return Err(refused(
                "assembly preparation receipt exceeds its byte bound",
            ));
        }
        Ok(text)
    }

    pub(in crate::diagram::cargo) fn read(text: &str) -> Result<Self, DiagramError> {
        let graph = Graph::from_text(text, Format::NTriples).map_err(DiagramError::data)?;
        if record::text(&graph, "", "form")? != "1" {
            return Err(refused("unsupported assembly preparation form"));
        }
        let mut sources = Vec::new();
        for index in 0..record::count(&graph, "", "sourceCount")? {
            let subject = format!("#source/{index}");
            sources.push((
                record::text(&graph, &subject, "path")?,
                record::text(&graph, &subject, "digest")?,
            ));
        }
        let mut fragments = Vec::new();
        for index in 0..record::count(&graph, "", "fragmentCount")? {
            let subject = format!("#fragment/{index}");
            fragments.push((
                record::text(&graph, &subject, "path")?,
                record::text(&graph, &subject, "digest")?,
                record::text(&graph, &subject, "bundle")?,
            ));
        }
        let result = Self {
            plan: record::text(&graph, "", "plan")?,
            package: record::text(&graph, "", "package")?,
            host: receipt::read_output(&graph, "assemblyHost")?,
            html_arguments: record::read_words(&graph, "", "htmlArgument")?,
            sources,
            fragments,
        };
        super::digest(&result.plan)?;
        for (path, digest) in &result.sources {
            safe_path(path)?;
            super::digest(digest)?;
        }
        for (path, digest, bundle) in &result.fragments {
            safe_path(path)?;
            super::digest(digest)?;
            super::digest(bundle)?;
        }
        record::closed(&result.graph()?, &graph)?;
        Ok(result)
    }
}

fn safe_path(text: &str) -> Result<RelPath, DiagramError> {
    let path = RelPath::new(text).map_err(DiagramError::data)?;
    if path.segments().any(|part| part == "..") {
        return Err(refused("assembly receipt paths cannot traverse parents"));
    }
    Ok(path)
}

pub(crate) struct Pending {
    plan: AssemblyPlan,
    locator: PathBuf,
    identity: FileId,
    observed: Observed,
    pub(crate) preparation: Preparation,
    bundles: Vec<Arc<DiagramBundle>>,
}

impl Pending {
    pub(crate) fn before_html(
        controls: &RustdocObservationOptions,
        directory: &Path,
        arguments: &[String],
        environment: &[(String, String)],
        cfg: &Output,
        host: Output,
    ) -> Result<Self, DiagramError> {
        let (locator, digest) = controls
            .assembly()
            .ok_or_else(|| refused("assembly control is absent"))?;
        let (plan, identity) = read_plan(locator, digest)?;
        let mut selected =
            plan.libraries.iter().filter_map(|library| {
                match driver::matches_inputs(arguments, environment, directory, library) {
                    Ok(true) => Some(Ok(library)),
                    Ok(false) => None,
                    Err(error) => Some(Err(error)),
                }
            });
        let library = selected.next().ok_or_else(|| {
            refused("actual compiler inputs match no selected assembly library")
        })??;
        if selected.next().is_some() {
            return Err(refused(
                "actual compiler inputs match multiple assembly libraries",
            ));
        }
        let effective = html_arguments(arguments)?;
        let input = driver::parse_input(arguments)?;
        if input.target.is_some()
            && environment
                .iter()
                .any(|(key, value)| key == "RUST_TARGET_PATH" && !value.is_empty())
        {
            return Err(refused(
                "custom target search paths require compiler target-specification input observations",
            ));
        }
        let configuration = Configuration::read(cfg, &host, input.target)?;
        let observed = Observed::new(&plan.root)?;
        let (source, target) = configured_source(&plan, library, &configuration, &observed)?;
        receiver::audit(&source, target)?;
        let declared = model_source(&plan, &observed)?;
        let workplan = crate::Workplan::load(&observed, &declared);
        let mut prepared = Vec::new();
        let mut bytes = 0usize;
        for request in plan
            .requests
            .iter()
            .filter(|request| request.package_id == library.package_id)
        {
            receiver::admit(&plan.root, &source, target, request)?;
            let bundle = workplan.named_diagram(&request.view)?;
            let fragment = rustdoc::fragment(
                &workplan,
                &source,
                &bundle,
                rustdoc::Context {
                    target,
                    module: &request.module,
                    capability: &request.capability,
                    required: &request.required,
                    library,
                    root: &plan.root,
                    metadata: &plan.metadata,
                },
            )?;
            bytes = bytes
                .checked_add(fragment.len())
                .filter(|bytes| *bytes <= MAX_BYTES)
                .ok_or_else(|| {
                    refused("generated assembly includes exceed their shared byte bound")
                })?;
            admit_include(&plan.root, request, &fragment)?;
            prepared.push((request, fragment, bundle));
        }
        let mut fragments = Vec::new();
        let mut bundles = Vec::new();
        for (request, fragment, bundle) in prepared {
            write_include(&plan.root, request, &fragment)?;
            fragments.push((
                request.include.to_string(),
                ContentDigest::of(fragment.as_bytes()).to_string(),
                ContentDigest::of(bundle.json().as_bytes()).to_string(),
            ));
            bundles.push(bundle);
        }
        let preparation = Preparation {
            plan: digest.into(),
            package: library.package_id.clone(),
            host,
            html_arguments: effective,
            sources: observed.verify()?,
            fragments,
        };
        Ok(Self {
            plan,
            locator: locator.into(),
            identity,
            observed,
            preparation,
            bundles,
        })
    }

    fn verify(&self) -> Result<(), DiagramError> {
        let (_, identity) = read_plan(&self.locator, &self.preparation.plan)?;
        if identity != self.identity || self.observed.verify()? != self.preparation.sources {
            return Err(refused(
                "assembly plan/source identities changed during HTML compilation",
            ));
        }
        verify_fragments(&self.plan.root, &self.preparation)
    }

    pub(crate) fn finish(self) -> Result<Preparation, DiagramError> {
        self.verify()?;
        Ok(self.preparation)
    }
}

fn model_source(plan: &AssemblyPlan, observed: &Observed) -> Result<Workspace, DiagramError> {
    if plan.model_roots.is_empty() {
        return Err(refused(
            "assembly plan has no complete document-model source roots",
        ));
    }
    let roots: Vec<_> = plan.model_roots.iter().map(String::as_str).collect();
    Workspace::load(observed, &roots).map_err(DiagramError::data)
}

fn configured_source(
    plan: &AssemblyPlan,
    library: &crate::diagram::cargo::Library,
    configuration: &Configuration,
    observed: &Observed,
) -> Result<(Workspace, raf_source::TargetId), DiagramError> {
    let manifest = relative(&plan.root, &library.manifest)?;
    let source = configuration
        .with(&library.name, |configuration| {
            Workspace::load_configured_target(
                observed,
                &manifest,
                configuration,
                &LoadOptions::default(),
            )
        })
        .map_err(DiagramError::data)?;
    let target = source
        .target_ids()
        .next()
        .ok_or_else(|| refused("configured assembly source has no target"))?;
    let actual = source
        .target(target)
        .ok_or_else(|| refused("configured assembly target disappeared"))?;
    if plan
        .root
        .join(actual.root.as_str())
        .canonicalize()
        .map_err(DiagramError::data)?
        != library.source.canonicalize().map_err(DiagramError::data)?
    {
        return Err(refused(
            "configured source target differs from the actual compiler input operand",
        ));
    }
    Ok((source, target))
}

fn verify_fragments(root: &Path, preparation: &Preparation) -> Result<(), DiagramError> {
    let tree = raf_source::DiskTree::new(root).map_err(DiagramError::data)?;
    for (path, digest, _) in &preparation.fragments {
        let text = tree
            .read(&safe_path(path)?, MAX_BYTES as u64)
            .map_err(DiagramError::data)?;
        if ContentDigest::of(text.as_bytes()).to_string() != *digest {
            return Err(refused(
                "owned include differs from the fragment consumed by the compiler",
            ));
        }
    }
    Ok(())
}

fn verify_reads(root: &Path, preparation: &Preparation) -> Result<(), DiagramError> {
    let tree = raf_source::DiskTree::new(root).map_err(DiagramError::data)?;
    for (path, digest) in &preparation.sources {
        let text = tree
            .read(&safe_path(path)?, MAX_BYTES as u64)
            .map_err(DiagramError::data)?;
        if ContentDigest::of(text.as_bytes()).to_string() != *digest {
            return Err(refused(
                "prepared source bytes changed before assembly publication",
            ));
        }
    }
    Ok(())
}

fn verify_artifact(
    target: &Path,
    artifact: &crate::diagram::cargo::LibraryArtifact,
) -> Result<PathBuf, DiagramError> {
    let tree = raf_source::DiskTree::new(target).map_err(DiagramError::data)?;
    let raf_source::OpenedBytes::Read { id, bytes } = tree
        .read_unseen_bytes(artifact.relative_path(), MAX_BYTES as u64, &|_| false)
        .map_err(DiagramError::data)?
    else {
        return Err(refused("actual Cargo HTML artifact was not read"));
    };
    if id != *artifact.file_id() || ContentDigest::of(&bytes) != artifact.sha256() {
        return Err(refused(
            "actual Cargo HTML artifact changed before assembly publication",
        ));
    }
    // Cargo clears a selected crate's directory before invoking Rustdoc, even
    // when our driver refuses. Keep immutable bundles beside those directories.
    let documentation = artifact
        .entry_directory()
        .parent()
        .ok_or_else(|| refused("actual Cargo HTML entry has no shared documentation directory"))?;
    Ok(target.join(documentation.as_str()))
}

fn navigation(bundle: &DiagramBundle) -> Result<(), DiagramError> {
    if bundle.presented_export().is_some_and(|export| {
        export
            .presentation
            .displays
            .iter()
            .any(|display| display.navigation.is_some())
    }) {
        return Err(refused(
            "this native diagram has explicit navigation requiring Cargo-host href admission; no API/source URL is guessed",
        ));
    }
    for detail in bundle.details() {
        navigation(detail)?;
    }
    Ok(())
}

pub(super) fn publish(
    controls: &RustdocObservationOptions,
    plan: &AssemblyPlan,
    observed: &crate::diagram::cargo::observation::ObservedDocumentation,
) -> Result<Vec<PathBuf>, DiagramError> {
    let mut admitted = Vec::new();
    for invocation in observed.invocations() {
        let preparation = invocation
            .assembly_preparation()
            .ok_or_else(|| refused("selected artifact has no actual pre-HTML assembly evidence"))?;
        if preparation.package != invocation.artifact().library().package_id {
            return Err(refused(
                "assembly preparation package differs from its actual Cargo artifact",
            ));
        }
        verify_reads(&plan.root, preparation)?;
        verify_fragments(&plan.root, preparation)?;
        let pending = Pending::before_html(
            controls,
            Path::new(invocation.directory()),
            invocation.arguments(),
            invocation.cargo_environment(),
            invocation.configuration_output(),
            preparation.host.clone(),
        )?;
        pending.verify()?;
        if pending.preparation.fragments != preparation.fragments
            || pending.preparation.html_arguments != preparation.html_arguments
        {
            return Err(refused(
                "regenerated shared model/includes differ from those actually consumed by Rustdoc",
            ));
        }
        let root = verify_artifact(
            observed.documentation().selection().target_directory(),
            invocation.artifact(),
        )?;
        for (index, bundle) in pending.bundles.iter().enumerate() {
            navigation(bundle)?;
            admitted.push((
                invocation,
                pending.plan.clone(),
                root.clone(),
                bundle.clone(),
                index,
            ));
        }
    }
    // Every model, include, link verdict and artifact is admitted before any
    // shared publisher runs. Its existing by-name publication limits still apply.
    let mut publications = Vec::new();
    for (invocation, plan, root, bundle, index) in admitted {
        let preparation = invocation
            .assembly_preparation()
            .ok_or_else(|| refused("assembly preparation disappeared"))?;
        verify_reads(&plan.root, preparation)?;
        verify_fragments(&plan.root, preparation)?;
        verify_artifact(
            observed.documentation().selection().target_directory(),
            invocation.artifact(),
        )?;
        let context = public_record(&plan, invocation, preparation, index)?;
        publications.push(crate::diagram::publication::publish_assembly(
            &bundle, &root, &context,
        )?);
        verify_reads(&plan.root, preparation)?;
        verify_fragments(&plan.root, preparation)?;
        verify_artifact(
            observed.documentation().selection().target_directory(),
            invocation.artifact(),
        )?;
    }
    Ok(publications)
}

fn public_record(
    plan: &AssemblyPlan,
    invocation: &crate::diagram::cargo::observation::InvocationObservation,
    preparation: &Preparation,
    index: usize,
) -> Result<Graph, DiagramError> {
    let library = invocation.artifact().library();
    let request = plan
        .requests
        .iter()
        .filter(|request| request.package_id == library.package_id)
        .nth(index)
        .ok_or_else(|| refused("publication has no exact include owner"))?;
    let fragment = preparation
        .fragments
        .get(index)
        .ok_or_else(|| refused("publication has no consumed include receipt"))?;
    let mut builder = Builder::new();
    // Only explicit public identities/digests enter this graph. Complete compiler
    // and Cargo environments remain solely in the private invocation receipt.
    for (key, value) in [
        ("form", "1"),
        ("package", library.package.as_str()),
        ("version", &library.version),
        ("crate", &library.name),
        ("module", &request.module),
        ("include", request.include.as_str()),
        ("view", &request.view),
        ("capability", &request.capability),
        ("fragmentDigest", &fragment.1),
        ("bundleDigest", &fragment.2),
        (
            "linkAdmission",
            "actual-rustdoc-forbid-broken-and-private-intra-doc-links",
        ),
    ] {
        record::put(&mut builder, "", key, value)?;
    }
    for (key, value) in [
        (
            "cargoPackageDigest",
            ContentDigest::of(library.package_id.as_bytes()).to_string(),
        ),
        ("source", relative(&plan.root, &library.source)?.to_string()),
        (
            "manifest",
            relative(&plan.root, &library.manifest)?.to_string(),
        ),
        (
            "artifact",
            invocation.artifact().relative_path().to_string(),
        ),
        ("artifactDigest", invocation.artifact().sha256().to_string()),
    ] {
        record::put(&mut builder, "", key, &value)?;
    }
    record::words(&mut builder, "", "requiredPublicBinding", &request.required)?;
    builder.finish().map_err(DiagramError::data)
}

fn relative(root: &Path, path: &Path) -> Result<RelPath, DiagramError> {
    safe_path(
        path.strip_prefix(root)
            .map_err(DiagramError::data)?
            .to_str()
            .ok_or_else(|| refused("assembly source locator must be UTF-8"))?,
    )
}

pub(in crate::diagram::cargo) fn html_arguments(
    arguments: &[String],
) -> Result<Vec<String>, DiagramError> {
    let input = driver::parse_input(arguments)?;
    if input.cap_lints || input.force_warn {
        return Err(refused(
            "checked assembly cannot accept lint caps or force-warn overrides of required link errors",
        ));
    }
    if input.include_namespace_override {
        return Err(refused(
            "assembly cannot admit overridden builtin include namespaces from --extern",
        ));
    }
    let boundary = input.terminator.unwrap_or(arguments.len());
    let mut effective = arguments[..boundary].to_vec();
    effective.extend([
        "--forbid=rustdoc::broken_intra_doc_links".into(),
        "--forbid=rustdoc::private_intra_doc_links".into(),
    ]);
    effective.extend_from_slice(&arguments[boundary..]);
    if effective.len() > super::MAX_ITEMS {
        return Err(refused(
            "assembly lint arguments exceed the original argument limit",
        ));
    }
    Ok(effective)
}

pub(super) fn admit_include(
    root: &Path,
    request: &super::IncludeRequest,
    text: &str,
) -> Result<bool, DiagramError> {
    let tree = raf_source::DiskTree::new(root).map_err(DiagramError::data)?;
    let mut at = RelPath::root();
    let parent = request
        .include
        .parent()
        .ok_or_else(|| refused("include has no parent"))?;
    for segment in parent.segments() {
        at = at.join(segment).map_err(DiagramError::data)?;
        if tree.entry(&at).map_err(DiagramError::data)? != Some(EntryKind::Dir) {
            return Err(refused(
                "include parent must consist of existing real directories",
            ));
        }
    }
    match tree.entry(&request.include).map_err(DiagramError::data)? {
        None if request.previous.is_none() => {}
        Some(EntryKind::File) => {
            let previous = tree
                .read(&request.include, MAX_BYTES as u64)
                .map_err(DiagramError::data)?;
            if previous == text {
                return Ok(true);
            }
            if request.previous != Some(ContentDigest::of(previous.as_bytes())) {
                return Err(refused(
                    "existing include is not the exact previously owned fragment",
                ));
            }
        }
        _ => {
            return Err(refused(
                "include ownership or regular-file admission failed",
            ));
        }
    }
    Ok(false)
}

pub(super) fn write_include(
    root: &Path,
    request: &super::IncludeRequest,
    text: &str,
) -> Result<(), DiagramError> {
    if admit_include(root, request, text)? {
        return Ok(());
    }
    let destination = root.join(request.include.as_str());
    let staged = crate::sync::stage(&destination, text).map_err(DiagramError::data)?;
    std::fs::rename(staged, destination).map_err(DiagramError::data)
}
