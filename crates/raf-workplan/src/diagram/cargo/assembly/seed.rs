// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Complete caller-model includes before Cargo compiles library dependencies.

use super::{AssemblyPlan, DiagramError, MAX_BYTES, prepare, refused};
use crate::diagram::rustdoc;
use crate::{Workplan, exec::RustdocObservationOptions};
use purrdf_core::ContentDigest;
use purrdf_rdf::projections::escape_xml_text;
use std::fmt::Write as _;
use std::path::Path;

fn locators(
    plan: &AssemblyPlan,
    path: &Path,
    controls: &RustdocObservationOptions,
) -> Result<(), DiagramError> {
    let receipts = controls.receipts();
    let parent = receipts
        .parent()
        .ok_or_else(|| refused("receipt directory has no parent"))?;
    if parent.canonicalize().map_err(DiagramError::data)? != parent {
        return Err(refused(
            "receipt parent must be an explicit canonical directory",
        ));
    }
    match std::fs::symlink_metadata(receipts) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(DiagramError::data(error)),
        Ok(_) => return Err(refused("compiler receipts require a new owned directory")),
    }
    if path.starts_with(receipts) || receipts.starts_with(path) {
        return Err(refused("assembly plan and receipt locators collide"));
    }
    for request in &plan.requests {
        let include = plan.root.join(request.include.as_str());
        if include == path || include.starts_with(receipts) || receipts.starts_with(&include) {
            return Err(refused(
                "include, assembly plan and receipt locators must remain distinct",
            ));
        }
    }
    Ok(())
}

fn fragment(
    model: &Workplan<'_>,
    library: &crate::diagram::cargo::Library,
    request: &super::IncludeRequest,
) -> Result<String, DiagramError> {
    let bundle = model.named_diagram(&request.view)?;
    let mut body = format!(
        "Declared API references for <code>{}</code>, Cargo-observed package <code>{}</code>, version <code>{}</code>.\n\n",
        escape_xml_text(&request.module).map_err(DiagramError::data)?,
        escape_xml_text(&library.package).map_err(DiagramError::data)?,
        escape_xml_text(&library.version).map_err(DiagramError::data)?,
    );
    for required in &request.required {
        let (canonical, written) = rustdoc::spelling(required)?;
        let (head, _) = canonical.split_once("::").ok_or_else(|| {
            refused("required API references must name an item within the selected library")
        })?;
        if head != library.name {
            return Err(refused("required API reference names another library"));
        }
        let (_, suffix) = canonical
            .split_once("::")
            .ok_or_else(|| refused("required API reference has no selected-library suffix"))?;
        writeln!(body, "- [`{written}`](crate::{suffix})").expect("writing to String cannot fail");
    }
    rustdoc::compose(&bundle, &body)
}

/// Observe every destination and compose every complete seed before a write.
pub(super) fn prepare(
    plan: &AssemblyPlan,
    model: &Workplan<'_>,
    path: &Path,
    controls: &RustdocObservationOptions,
) -> Result<(AssemblyPlan, Vec<String>), DiagramError> {
    locators(plan, path, controls)?;
    let mut concrete = plan.clone();
    concrete.model_roots = model
        .source()
        .packages()
        .iter()
        .map(|package| package.dir.to_string())
        .collect();
    if concrete.model_roots.is_empty() {
        return Err(refused(
            "document model requires explicit source package roots",
        ));
    }
    let mut fragments = Vec::new();
    let mut bytes = 0usize;
    for (request, concrete) in plan.requests.iter().zip(&mut concrete.requests) {
        let library = plan
            .libraries
            .iter()
            .find(|library| library.package_id == request.package_id)
            .ok_or_else(|| refused("seed include has no selected Cargo library"))?;
        let text = fragment(model, library, request)?;
        bytes = bytes
            .checked_add(text.len())
            .filter(|bytes| *bytes <= MAX_BYTES)
            .ok_or_else(|| refused("seed includes exceed their shared byte bound"))?;
        prepare::admit_include(&plan.root, request, &text)?;
        concrete.previous = Some(ContentDigest::of(text.as_bytes()));
        fragments.push(text);
    }
    Ok((concrete, fragments))
}

pub(super) fn write(plan: &AssemblyPlan, fragments: &[String]) -> Result<(), DiagramError> {
    if fragments.len() != plan.requests.len() {
        return Err(refused(
            "seed count differs from the admitted include requests",
        ));
    }
    for (request, text) in plan.requests.iter().zip(fragments) {
        prepare::write_include(&plan.root, request, text)?;
    }
    Ok(())
}
