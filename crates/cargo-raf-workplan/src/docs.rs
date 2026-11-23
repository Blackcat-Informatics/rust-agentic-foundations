// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static Markdown fragments, artifact inspection and invocation-scoped compilation.

use crate::{
    cli::{ArtifactArgs, FragmentArgs, LibraryArgs, TrustedDocsArgs},
    exit::Failure,
    render::{Record, Report},
    result::Result as CommandResult,
    trusted_docs,
};
use raf_source::TargetKind;
use raf_workplan::diagram::cargo::{self, CargoDocOptions, DocumentationArtifacts};
use raf_workplan::{BindingKind, Gate};
use std::{
    fs::File,
    io::Read as _,
    path::{Path, PathBuf},
};

fn options(args: &LibraryArgs) -> Result<CargoDocOptions, Failure> {
    let packages: Vec<_> = args.packages.iter().map(String::as_str).collect();
    let features: Vec<_> = args.features.iter().map(String::as_str).collect();
    Ok(CargoDocOptions::new(
        &packages,
        &features,
        args.no_default_features,
        Some(&args.target_directory),
    )?)
}

fn read(path: &Path) -> Result<Vec<u8>, Failure> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(cargo::MAX_METADATA_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > cargo::MAX_METADATA_BYTES {
        return Err(Failure::new(format!(
            "{} exceeds the artifact carrier byte bound",
            path.display()
        )));
    }
    Ok(bytes)
}

/// Assemble one complete static fragment before its caller emits any Markdown.
pub(crate) fn fragment(root: &Path, args: &FragmentArgs) -> Result<String, Failure> {
    let options = CargoDocOptions::new(
        &[args.package.as_str()],
        &[],
        false,
        Some(&args.target_directory),
    )?;
    let selection = cargo::read_metadata(
        root,
        &args.target_directory,
        &options,
        &read(&args.metadata)?,
    )?;
    let [library] = selection.libraries() else {
        return Err(Failure::new(
            "docs fragment requires exactly one selected library",
        ));
    };
    // This uses the same bounded, static default Source load as diagram commands.
    // Imported metadata never selects effective compiler cfg or generated inputs.
    let opened = Gate::workspace(root).open()?;
    let source = opened.workspace();
    let mut targets = source.target_ids().filter(|id| {
        source.target(*id).is_some_and(|target| {
            target.kind == TargetKind::Lib
                && target.package == library.package
                && target.name == library.name
        })
    });
    let target = targets.next().ok_or_else(|| {
        Failure::new("selected Cargo library has no matching static source target")
    })?;
    if targets.next().is_some() {
        return Err(Failure::new(
            "selected Cargo library has ambiguous static source targets",
        ));
    }
    let plan = opened.workplan();
    let bundle = plan.named_diagram(&args.view)?;
    let mut required = Vec::new();
    for binding in plan
        .graph()
        .bindings_of(&args.capability)
        .filter(|binding| binding.kind() == BindingKind::Code)
    {
        if required.len() == 4096 {
            return Err(Failure::new(
                "the capability exceeds 4096 required API references",
            ));
        }
        required.push(binding.path().to_owned());
    }
    Ok(raf_workplan::diagram::rustdoc::fragment(
        &plan,
        source,
        &bundle,
        raf_workplan::diagram::rustdoc::Context {
            target,
            module: &args.module,
            capability: &args.capability,
            required: &required,
            library,
            root: selection.workspace_root(),
            metadata: &selection.metadata_sha256().to_string(),
        },
    )?)
}

pub(crate) fn artifacts(root: &Path, args: &ArtifactArgs) -> Result<Report, Failure> {
    let options = options(&args.libraries)?;
    let metadata = read(&args.metadata)?;
    let messages = read(&args.messages)?;
    let selection =
        cargo::read_metadata(root, &args.libraries.target_directory, &options, &metadata)?;
    let artifacts = cargo::read_artifacts(&selection, &messages)?;
    report(
        &artifacts,
        "imported artifact data; compiler execution is not verified",
        None,
    )
}

pub(crate) fn build(root: &Path, args: &TrustedDocsArgs) -> Result<Report, Failure> {
    let companion = args
        .view
        .as_deref()
        .map(|view| {
            let opened = Gate::workspace(root).open()?;
            let bundle = opened.workplan().named_diagram(view)?;
            Ok::<_, Failure>((view, bundle))
        })
        .transpose()?;
    let built = trusted_docs::build(&trusted_docs::BuildRequest {
        root: root.to_path_buf(),
        toolchain_directory: args.toolchain_directory.clone(),
        target_directory: args.libraries.target_directory.clone(),
        packages: args.libraries.packages.clone(),
        features: args.libraries.features.clone(),
        no_default_features: args.libraries.no_default_features,
        trust_docs: args.trust_docs,
    })?;
    let artifacts = built
        .target
        .read_artifacts(&built.selection, &built.messages)?;
    let published = if let Some((view, bundle)) = companion {
        let directory = built.target.publish_companion(&bundle, &artifacts)?;
        Some(publication_record(view, bundle.view().name(), directory)?)
    } else {
        None
    };
    let mut report = report(
        &artifacts,
        "trusted documentation compilation completed",
        published,
    )?;
    report.note(raf_workplan::escape::shown_lines(&format!(
        "{}\n{}\n{}",
        built.cargo_version, built.rustc_version, built.rustdoc_version
    )));
    if !built.diagnostics.is_empty() {
        report.note(raf_workplan::escape::shown_lines(&String::from_utf8_lossy(
            &built.diagnostics,
        )));
    }
    Ok(report)
}

#[expect(
    clippy::unnecessary_debug_formatting,
    reason = "these diagnostics specifically report non-UTF-8 paths; debug escaping preserves their original bytes"
)]
pub(crate) fn publication_record(
    view: &str,
    name: &str,
    directory: PathBuf,
) -> Result<Record, Failure> {
    let entry = directory.join(format!("{name}.html"));
    let directory = directory.into_os_string().into_string().map_err(|path| {
        Failure::new(format!(
            "documentation directory cannot be represented as UTF-8: {path:?}"
        ))
    })?;
    let entry = entry.into_os_string().into_string().map_err(|path| {
        Failure::new(format!(
            "documentation entry cannot be represented as UTF-8: {path:?}"
        ))
    })?;
    let message = format!("published {view} at {entry}");
    Ok(Record::outcome(
        CommandResult::DocumentationBuilt {
            view: view.to_owned(),
            directory,
            entry,
        },
        message,
    ))
}

fn report(
    artifacts: &DocumentationArtifacts,
    message: &str,
    published: Option<Record>,
) -> Result<Report, Failure> {
    let records = artifacts
        .artifacts()
        .iter()
        .map(|artifact| {
            let entry = artifact
                .reported_path()
                .to_str()
                .ok_or_else(|| Failure::new("artifact path must be UTF-8"))?;
            Ok(Record::outcome(
                CommandResult::DocumentationArtifact {
                    package: artifact.library().package.clone(),
                    entry: entry.to_owned(),
                    sha256: artifact.sha256().to_string(),
                    provenance: message.to_owned(),
                },
                raf_workplan::escape::shown_lines(&format!(
                    "{}: {entry} SHA-256 {}",
                    artifact.library().package,
                    artifact.sha256()
                )),
            ))
        })
        .collect::<Result<Vec<_>, Failure>>()?;
    let mut report = Report::of(records.into_iter().chain(published));
    report.note(message);
    Ok(report)
}
