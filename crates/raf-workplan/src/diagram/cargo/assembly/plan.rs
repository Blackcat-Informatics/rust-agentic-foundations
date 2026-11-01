// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One exact Cargo selection and explicitly owned receiving include locations.

use super::{DiagramError, MAX_BYTES, MAX_ITEMS, Selection, record, refused};
use crate::diagram::cargo::Library;
use crate::diagram::rustdoc;
use purrdf_core::ContentDigest;
use raf_rdf::{Builder, Format, Graph};
use raf_source::RelPath;
use std::path::{Path, PathBuf};

/// An include owned by an explicit assembly operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncludeRequest {
    pub(super) package_id: String,
    pub(super) module: String,
    pub(super) include: RelPath,
    pub(super) view: String,
    pub(super) capability: String,
    pub(super) required: Vec<String>,
    pub(super) previous: Option<ContentDigest>,
}

impl IncludeRequest {
    /// Select the actual receiving public module, named view and public bindings.
    /// The include is relative to the captured workspace, with real parent
    /// directories. Byte-identical existing fragments are reused without writing.
    /// Replacing different contents requires exact prior-content ownership.
    #[must_use]
    pub fn new(
        package_id: impl Into<String>,
        module: impl Into<String>,
        include: RelPath,
        view: impl Into<String>,
        capability: impl Into<String>,
        required: Vec<String>,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            module: module.into(),
            include,
            view: view.into(),
            capability: capability.into(),
            required,
            previous: None,
        }
    }
    /// Authorize replacement only of these exact previously owned fragment bytes.
    #[must_use]
    pub const fn replace_owned(mut self, digest: ContentDigest) -> Self {
        self.previous = Some(digest);
        self
    }
}

/// Closed, bounded pre-HTML assembly instructions for one captured Cargo selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssemblyPlan {
    pub(super) root: PathBuf,
    target: PathBuf,
    pub(super) metadata: String,
    pub(super) libraries: Vec<Library>,
    pub(super) requests: Vec<IncludeRequest>,
    pub(super) model_roots: Vec<String>,
}

impl AssemblyPlan {
    /// Bind include ownership to an actual captured Cargo package/library selection.
    ///
    /// # Errors
    /// Refuses missing/unknown selected packages, duplicate or traversing include
    /// locators, unbounded instructions and unsupported public-path spellings.
    pub fn new(selection: &Selection, requests: Vec<IncludeRequest>) -> Result<Self, DiagramError> {
        let plan = Self {
            root: selection.workspace_root().into(),
            target: selection.target_directory().into(),
            metadata: selection.metadata_sha256().to_string(),
            libraries: selection.libraries().to_vec(),
            requests,
            model_roots: Vec::new(),
        };
        plan.validate()?;
        Ok(plan)
    }

    pub(super) fn matches_selection(&self, selection: &Selection) -> Result<(), DiagramError> {
        if self.root != selection.workspace_root()
            || self.target != selection.target_directory()
            || self.metadata != selection.metadata_sha256().to_string()
            || self.libraries != selection.libraries()
        {
            return Err(refused(
                "assembly plan differs from the actual captured Cargo selection",
            ));
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), DiagramError> {
        if !self.root.is_absolute()
            || !self.target.is_absolute()
            || self.libraries.is_empty()
            || self.libraries.len() > MAX_ITEMS
            || self.requests.len() > MAX_ITEMS
        {
            return Err(refused(
                "assembly selection is absent, unbounded or nonabsolute",
            ));
        }
        super::digest(&self.metadata)?;
        let mut bytes = 0usize;
        let mut items = 0usize;
        let mut charge = |value: &str| -> Result<(), DiagramError> {
            items = items
                .checked_add(1)
                .filter(|items| *items <= MAX_ITEMS)
                .ok_or_else(|| refused("assembly instructions exceed their shared item bound"))?;
            bytes = bytes
                .checked_add(value.len())
                .filter(|bytes| *bytes <= MAX_BYTES)
                .ok_or_else(|| refused("assembly instructions exceed their shared byte bound"))?;
            Ok(())
        };
        for root in &self.model_roots {
            charge(root)?;
            let path = RelPath::new(root).map_err(DiagramError::data)?;
            if path.segments().any(|part| part == "..") {
                return Err(refused(
                    "document-model roots must stay within the workspace",
                ));
            }
        }
        for library in &self.libraries {
            for value in [
                &library.package_id,
                &library.package,
                &library.version,
                &library.name,
            ]
            .into_iter()
            .chain(&library.kind)
            .chain(&library.crate_types)
            .chain(&library.features)
            {
                charge(value)?;
            }
            charge(path(&library.manifest)?)?;
            charge(path(&library.source)?)?;
            if !library.manifest.starts_with(&self.root)
                || !library.source.starts_with(&self.root)
                || !self
                    .requests
                    .iter()
                    .any(|request| request.package_id == library.package_id)
            {
                return Err(refused(
                    "each selected workspace library requires actual source identity and an include request",
                ));
            }
        }
        for (index, request) in self.requests.iter().enumerate() {
            for value in [
                &request.package_id,
                &request.module,
                &request.view,
                &request.capability,
            ]
            .into_iter()
            .chain(&request.required)
            {
                charge(value)?;
            }
            charge(request.include.as_str())?;
            if request.include.as_str().is_empty()
                || request.include.segments().any(|part| part == "..")
                || self.requests[..index]
                    .iter()
                    .any(|known| known.include == request.include)
                || !self
                    .libraries
                    .iter()
                    .any(|library| library.package_id == request.package_id)
                || request.required.is_empty()
                || request.capability.is_empty()
                || request.required.len() > MAX_ITEMS
            {
                return Err(refused(
                    "assembly includes need unique contained ownership and required selected-library bindings",
                ));
            }
            rustdoc::spelling(&request.module)?;
            rustdoc::api_names(request.required.iter().map(String::as_str))?;
        }
        Ok(())
    }

    /// Complete canonical native plan carrier, before exclusive on-disk creation.
    ///
    /// # Errors
    /// Preserves native RDF failures and the closed plan byte/item limits.
    pub fn to_ntriples(&self) -> Result<String, DiagramError> {
        self.validate()?;
        let text = self
            .graph()?
            .to_text(Format::NTriples)
            .map_err(DiagramError::data)?;
        if text.len() > MAX_BYTES {
            return Err(refused("assembly plan exceeds its byte bound"));
        }
        Ok(text)
    }

    fn graph(&self) -> Result<Graph, DiagramError> {
        let mut builder = Builder::new();
        for (key, value) in [
            ("form", "1"),
            ("root", path(&self.root)?),
            ("target", path(&self.target)?),
            ("metadata", self.metadata.as_str()),
        ] {
            record::put(&mut builder, "", key, value)?;
        }
        record::words(&mut builder, "", "modelRoot", &self.model_roots)?;
        record::put(
            &mut builder,
            "",
            "libraryCount",
            &self.libraries.len().to_string(),
        )?;
        for (index, library) in self.libraries.iter().enumerate() {
            let subject = format!("#library/{index}");
            for (key, value) in [
                ("id", library.package_id.as_str()),
                ("package", &library.package),
                ("version", &library.version),
                ("manifest", path(&library.manifest)?),
                ("source", path(&library.source)?),
                ("name", &library.name),
            ] {
                record::put(&mut builder, &subject, key, value)?;
            }
            for (key, values) in [
                ("kind", &library.kind),
                ("crateType", &library.crate_types),
                ("feature", &library.features),
            ] {
                record::words(&mut builder, &subject, key, values)?;
            }
        }
        record::put(
            &mut builder,
            "",
            "includeCount",
            &self.requests.len().to_string(),
        )?;
        for (index, request) in self.requests.iter().enumerate() {
            let subject = format!("#include/{index}");
            let previous = request
                .previous
                .map_or_else(String::new, |digest| digest.to_string());
            for (key, value) in [
                ("package", request.package_id.as_str()),
                ("module", &request.module),
                ("path", request.include.as_str()),
                ("view", &request.view),
                ("capability", &request.capability),
                ("previous", &previous),
            ] {
                record::put(&mut builder, &subject, key, value)?;
            }
            record::words(&mut builder, &subject, "required", &request.required)?;
        }
        builder.finish().map_err(DiagramError::data)
    }

    pub(super) fn read(graph: &Graph) -> Result<Self, DiagramError> {
        if record::text(graph, "", "form")? != "1" {
            return Err(refused("unsupported assembly plan form"));
        }
        let mut libraries = Vec::new();
        for index in 0..record::count(graph, "", "libraryCount")? {
            let subject = format!("#library/{index}");
            libraries.push(Library {
                package_id: record::text(graph, &subject, "id")?,
                package: record::text(graph, &subject, "package")?,
                version: record::text(graph, &subject, "version")?,
                manifest: record::text(graph, &subject, "manifest")?.into(),
                source: record::text(graph, &subject, "source")?.into(),
                name: record::text(graph, &subject, "name")?,
                kind: record::read_words(graph, &subject, "kind")?,
                crate_types: record::read_words(graph, &subject, "crateType")?,
                features: record::read_words(graph, &subject, "feature")?,
            });
        }
        let mut requests = Vec::new();
        for index in 0..record::count(graph, "", "includeCount")? {
            let subject = format!("#include/{index}");
            let previous = record::text(graph, &subject, "previous")?;
            requests.push(IncludeRequest {
                package_id: record::text(graph, &subject, "package")?,
                module: record::text(graph, &subject, "module")?,
                include: RelPath::new(&record::text(graph, &subject, "path")?)
                    .map_err(DiagramError::data)?,
                view: record::text(graph, &subject, "view")?,
                capability: record::text(graph, &subject, "capability")?,
                required: record::read_words(graph, &subject, "required")?,
                previous: if previous.is_empty() {
                    None
                } else {
                    Some(super::digest(&previous)?)
                },
            });
        }
        let plan = Self {
            root: record::text(graph, "", "root")?.into(),
            target: record::text(graph, "", "target")?.into(),
            metadata: record::text(graph, "", "metadata")?,
            libraries,
            requests,
            model_roots: record::read_words(graph, "", "modelRoot")?,
        };
        plan.validate()?;
        record::closed(&plan.graph()?, graph)?;
        Ok(plan)
    }
}

fn path(path: &Path) -> Result<&str, DiagramError> {
    path.to_str()
        .ok_or_else(|| refused("assembly locators must be UTF-8"))
}
