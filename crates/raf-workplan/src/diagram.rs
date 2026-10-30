// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Named semantic views with one native model for every presentation artifact.

use std::{error::Error, fmt, fmt::Write as _, num::NonZeroUsize, sync::Arc};

use purrdf_core::RdfDataset;
use purrdf_lex::json::{self, Object, Value, record::ToJson as _};
use purrdf_rdf::viz::{
    self, VizError, VizExport, VizLayoutOptions, VizPresentedExport, VizSpec, VizSvgOptions,
};

use crate::{Workplan, evidence::ClaimsDatasetError};
pub use raf_okf::{DiagramDeclaration, DiagramGrouping, DiagramKind};

pub(crate) mod assets;
pub(crate) mod authored;
pub mod book;
pub mod cargo;
pub mod documentation;
pub mod publication;
mod semantic;
pub use assets::publication::Publication as DiagramPublication;

/// An explicit selection in the existing semantic model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagramScope {
    /// Every authored claim reference to the named evidence record.
    Evidence {
        /// An evidence id, with or without its `evidence:` prefix.
        id: String,
    },
    /// A native automatic family over a workspace or document selection.
    Automatic {
        /// The requested relationship family.
        kind: DiagramKind,
        /// `workspace`, a package name, capability id or change id.
        scope: String,
    },
}

/// A checked name, typed family and normalized explicit scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedDiagramView {
    name: String,
    scope: DiagramScope,
    spec: VizSpec,
    focus: Vec<String>,
    grouping: DiagramGrouping,
    relations: Vec<String>,
    owner: Option<String>,
}

impl NamedDiagramView {
    /// Check a view name and normalize its evidence scope.
    ///
    /// # Errors
    /// Refuses a name outside the native kebab-slug grammar and preserves the
    /// existing evidence query's malformed-id error.
    pub fn new(name: &str, scope: DiagramScope) -> Result<Self, DiagramError> {
        if !raf_okf::is_kebab(name) {
            return Err(DiagramError::Name {
                name: name.to_owned(),
            });
        }
        let scope = match scope {
            DiagramScope::Evidence { id } => DiagramScope::Evidence {
                id: crate::evidence::evidence_id(&id).map_err(ClaimsDatasetError::Lookup)?,
            },
            DiagramScope::Automatic { kind, scope }
                if kind != DiagramKind::EvidenceClaims && !scope.trim().is_empty() =>
            {
                DiagramScope::Automatic { kind, scope }
            }
            DiagramScope::Automatic { .. } => {
                return Err(DiagramError::refused(
                    "automatic scope must name a non-evidence family and a nonempty selection",
                ));
            }
        };
        Ok(Self {
            name: name.to_owned(),
            scope,
            spec: VizSpec::default(),
            focus: Vec::new(),
            grouping: DiagramGrouping::None,
            relations: Vec::new(),
            owner: None,
        })
    }

    /// Normalize one closed document declaration into the common view API.
    ///
    /// # Errors
    /// Refuses malformed names or family-specific scope spellings.
    pub fn from_declaration(value: &DiagramDeclaration) -> Result<Self, DiagramError> {
        let scope = if value.kind == DiagramKind::EvidenceClaims {
            DiagramScope::Evidence {
                id: value.scope.clone(),
            }
        } else {
            DiagramScope::Automatic {
                kind: value.kind,
                scope: value.scope.clone(),
            }
        };
        let mut view = Self::new(&value.name, scope)?;
        view.focus.clone_from(&value.focus);
        view.grouping = value.grouping;
        view.relations.clone_from(&value.relations);
        view.spec.max_statements = value.max_statements;
        view.spec.max_terms = value.max_terms;
        Ok(view)
    }

    /// Select focus identifiers inside the declared scope, retaining touching relationships.
    #[must_use]
    pub fn with_focus(mut self, ids: Vec<String>) -> Self {
        self.focus = ids;
        self
    }

    /// Request an overview with explicit group membership and linked detail views.
    #[must_use]
    pub fn with_grouping(mut self, grouping: DiagramGrouping) -> Self {
        self.grouping = grouping;
        self
    }

    /// Restrict the closed relationship vocabulary of the selected family.
    #[must_use]
    pub fn with_relations(mut self, relations: Vec<String>) -> Self {
        self.relations = relations;
        self
    }

    /// Set positive native projection limits; exceeding either refuses.
    #[must_use]
    pub fn with_limits(mut self, statements: NonZeroUsize, terms: NonZeroUsize) -> Self {
        self.spec.max_statements = statements.get();
        self.spec.max_terms = terms.get();
        self
    }

    /// The checked view name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The view's typed semantic family.
    #[must_use]
    pub fn kind(&self) -> DiagramKind {
        match self.scope {
            DiagramScope::Evidence { .. } => DiagramKind::EvidenceClaims,
            DiagramScope::Automatic { kind, .. } => kind,
        }
    }

    /// The normalized selection, retained independently of rendering.
    #[must_use]
    pub fn scope(&self) -> &DiagramScope {
        &self.scope
    }
}

/// One semantic selection and its immutable, mutually consistent artifacts.
#[derive(Debug)]
pub struct DiagramBundle {
    view: NamedDiagramView,
    dataset: Arc<RdfDataset>,
    export: NativeExport,
    svg: String,
    text: String,
    json: String,
    caption: String,
    details: Vec<DiagramBundle>,
    full_model_hash: Option<String>,
}

#[derive(Debug)]
enum NativeExport {
    Ordinary(Box<VizExport>),
    Presented(Box<VizPresentedExport>),
}

impl NativeExport {
    fn base(&self) -> &VizExport {
        match self {
            Self::Ordinary(export) => export,
            Self::Presented(export) => &export.export,
        }
    }
    fn to_json(&self) -> Value {
        match self {
            Self::Ordinary(export) => export.to_json(),
            Self::Presented(export) => export.to_json(),
        }
    }

    fn text(&self, title: &str) -> String {
        let mut text = format!("{title}\n");
        for accessible in self
            .base()
            .scene
            .nodes
            .iter()
            .map(|node| &node.accessibility)
            .chain(
                self.base()
                    .scene
                    .edges
                    .iter()
                    .map(|edge| &edge.accessibility),
            )
        {
            writeln!(text, "{}: {}", accessible.title, accessible.description)
                .expect("writing to String cannot fail");
        }
        if let Self::Presented(export) = self {
            if let Some(scenario) = &export.presentation.scenario {
                for region in &scenario.regions {
                    writeln!(
                        text,
                        "DECLARED region {}: kind={} parent={} order={}; {}",
                        region.id,
                        json::write_compact(&region.kind.to_json()),
                        region.parent.as_deref().unwrap_or("root"),
                        region.order,
                        region.label.full_text
                    )
                    .expect("writing to String cannot fail");
                }
            }
            for interaction in &export.geometry.interactions {
                writeln!(text, "{}", interaction.label.full_text)
                    .expect("writing to String cannot fail");
            }
        }
        crate::escape::shown_lines(&text)
    }
}

impl DiagramBundle {
    /// Caption describing the semantic family and its explicit scope.
    #[must_use]
    pub fn caption(&self) -> &str {
        &self.caption
    }

    /// Scoped detail bundles linked by an explicitly grouped overview.
    #[must_use]
    pub fn details(&self) -> &[DiagramBundle] {
        &self.details
    }

    /// Native hash of the complete model, independently of a selected domain projection.
    #[must_use]
    pub fn full_model_hash(&self) -> Option<&str> {
        self.full_model_hash.as_deref()
    }
    /// The normalized named view which produced this bundle.
    #[must_use]
    pub fn view(&self) -> &NamedDiagramView {
        &self.view
    }

    /// The exact existing semantic dataset selected by the view.
    #[must_use]
    pub fn dataset(&self) -> &RdfDataset {
        &self.dataset
    }

    /// The single native export shared by all presentation artifacts.
    #[must_use]
    pub fn export(&self) -> &VizExport {
        self.export.base()
    }

    /// Complete native presentation, geometry and element index for authored models.
    #[must_use]
    pub fn presented_export(&self) -> Option<&VizPresentedExport> {
        match &self.export {
            NativeExport::Ordinary(_) => None,
            NativeExport::Presented(export) => Some(export),
        }
    }

    /// Accessible SVG with the complete native export in its metadata.
    #[must_use]
    pub fn svg(&self) -> &str {
        &self.svg
    }

    /// Terminal-safe native accessibility descriptions in deterministic scene order.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Structured view metadata and the same complete native export.
    #[must_use]
    pub fn json(&self) -> &str {
        &self.json
    }
}

/// Why a complete named diagram bundle could not be produced.
#[derive(Debug)]
#[non_exhaustive]
pub enum DiagramError {
    /// The requested view name is not a native kebab slug.
    Name {
        /// The invalid name supplied by the caller.
        name: String,
    },
    /// The existing semantic query or its checked identity refused.
    Claims(ClaimsDatasetError),
    /// Native projection, layout or SVG emission refused.
    Native(VizError),
    /// A scope, relation, grouping or source binding cannot be represented completely.
    Refused {
        /// Actionable policy reason.
        reason: String,
    },
    /// Native dataset storage, identity or RDF validation error.
    Data(Box<dyn Error + Send + Sync>),
}

impl fmt::Display for DiagramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name { name } => write!(
                f,
                "diagram name `{name}` is not a kebab slug; use lowercase words separated by hyphens"
            ),
            Self::Claims(error) => error.fmt(f),
            Self::Native(error) => error.fmt(f),
            Self::Refused { reason } => write!(f, "diagram refuses: {reason}"),
            Self::Data(error) => write!(f, "native diagram data: {error}"),
        }
    }
}

impl Error for DiagramError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Name { .. } | Self::Refused { .. } => None,
            Self::Claims(error) => Some(error),
            Self::Native(error) => Some(error),
            Self::Data(error) => Some(error.as_ref()),
        }
    }
}

impl DiagramError {
    fn refused(reason: impl Into<String>) -> Self {
        Self::Refused {
            reason: reason.into(),
        }
    }
    fn data(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Data(Box::new(error))
    }
}

impl From<ClaimsDatasetError> for DiagramError {
    fn from(error: ClaimsDatasetError) -> Self {
        Self::Claims(error)
    }
}

impl From<VizError> for DiagramError {
    fn from(error: VizError) -> Self {
        Self::Native(error)
    }
}

impl Workplan<'_> {
    /// Resolve one named view and generate every artifact from one native export.
    ///
    /// Evidence scopes retain the existing dataset's document identities,
    /// occurrence reifiers and unasserted statement terms. This interprets no
    /// statement's truth and adds no execution or currency assertion.
    ///
    /// # Errors
    /// Preserves semantic query errors and native limit, layout and rendering
    /// failures. No bundle is returned on a partial result and no file is written.
    pub fn diagram(&self, view: &NamedDiagramView) -> Result<DiagramBundle, DiagramError> {
        if view.spec.max_statements > 10000 || view.spec.max_terms > 30000 {
            return Err(DiagramError::refused(
                "native view budgets may not exceed 10000 statements or 30000 terms",
            ));
        }
        let mut normalized = view.clone();
        for values in [&mut normalized.focus, &mut normalized.relations] {
            if values.len() > normalized.spec.max_terms {
                return Err(DiagramError::refused("selection list exceeds max_terms"));
            }
            purrdf_lex::walk::try_sort_unstable_by(values, |a, b| Ok::<_, DiagramError>(a.cmp(b)))
                .map_err(DiagramError::data)?;
            values.dedup();
        }
        let view = &normalized;
        match &view.scope {
            DiagramScope::Evidence { id } => {
                if !view.focus.is_empty()
                    || !view.relations.is_empty()
                    || view.grouping != DiagramGrouping::None
                {
                    return Err(DiagramError::refused(
                        "evidence-claims already selects one exact evidence record; use coverage for grouped or focused evidence relationships",
                    ));
                }
                DiagramBundle::build(view, self.evidence_claims_dataset(id)?, Vec::new())
            }
            DiagramScope::Automatic { kind, scope } => {
                if matches!(
                    kind,
                    DiagramKind::Components
                        | DiagramKind::Scenario
                        | DiagramKind::Artifacts
                        | DiagramKind::ModelProvenance
                ) {
                    return authored::bundle(self, view, scope);
                }
                let facts = semantic::collect(self, view, *kind, scope)?;
                if view.grouping == DiagramGrouping::None {
                    return DiagramBundle::build(
                        view,
                        semantic::dataset(&facts, view, scope)?,
                        Vec::new(),
                    );
                }
                let (overview, parts) = semantic::grouped(&facts, view)?;
                let mut details = Vec::new();
                for (name, part) in parts {
                    let mut detail = view.clone();
                    detail.name = name;
                    detail.grouping = DiagramGrouping::None;
                    detail.focus = part
                        .entities
                        .ordered()
                        .map(|(iri, _)| iri.clone())
                        .collect();
                    details.push(DiagramBundle::build(
                        &detail,
                        semantic::dataset(&part, &detail, scope)?,
                        Vec::new(),
                    )?);
                }
                DiagramBundle::build(view, semantic::dataset(&overview, view, scope)?, details)
            }
        }
    }
}

impl DiagramBundle {
    fn build(
        view: &NamedDiagramView,
        dataset: Arc<RdfDataset>,
        details: Vec<Self>,
    ) -> Result<Self, DiagramError> {
        let scope = match &view.scope {
            DiagramScope::Evidence { id } => id,
            DiagramScope::Automatic { scope, .. } => scope,
        };
        let projection = viz::project_dataset(&dataset, &view.spec)?;
        Self::finish(view, dataset, &projection, None, details, scope, None)
    }

    fn finish(
        view: &NamedDiagramView,
        dataset: Arc<RdfDataset>,
        projection: &viz::VizProjection,
        presentation: Option<&viz::VizPresentation>,
        details: Vec<Self>,
        scope: &str,
        full_model_hash: Option<&str>,
    ) -> Result<Self, DiagramError> {
        let export = if let Some(presentation) = presentation {
            NativeExport::Presented(Box::new(viz::build_presented_export(
                projection,
                &view.spec,
                presentation,
                &VizLayoutOptions::default(),
            )?))
        } else {
            NativeExport::Ordinary(Box::new(viz::build_export(
                projection,
                &view.spec,
                &VizLayoutOptions::default(),
            )?))
        };
        let title = format!("{}: {} for {scope}", view.name, view.kind().as_str());
        let options = VizSvgOptions {
            title: title.clone(),
            ..VizSvgOptions::default()
        };
        let svg = match &export {
            NativeExport::Ordinary(export) => viz::render_export_svg(export, &options)?,
            NativeExport::Presented(export) => viz::render_presented_export_svg(export, &options)?,
        };
        let mut text = export.text(&title);
        if let Some(hash) = full_model_hash {
            writeln!(text, "Full RDF provenance model hash: {hash}")
                .expect("writing to String cannot fail");
        }
        let metadata = Object::new()
            .with("schema", "raf-diagram-bundle-1")
            .with("name", view.name.as_str())
            .with("kind", view.kind().as_str())
            .with("owner", view.owner.clone())
            .with(
                "scope",
                Object::new()
                    .with("selection", scope)
                    .with("focus", view.focus.clone())
                    .with("grouping", view.grouping.as_str())
                    .with("relations", view.relations.clone()),
            )
            .with(
                "details",
                details
                    .iter()
                    .map(|detail| {
                        Value::from(
                            Object::new()
                                .with("name", detail.view.name.as_str())
                                .with("kind", detail.view.kind().as_str())
                                .with("caption", detail.caption.as_str())
                                .with("export", detail.export.to_json()),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .with("export", export.to_json());
        let metadata = if let Some(hash) = full_model_hash {
            metadata.with("full_model_hash", hash)
        } else {
            metadata
        };
        let json = json::write_compact(&Value::from(metadata));
        Ok(DiagramBundle {
            view: view.clone(),
            dataset,
            export,
            svg,
            text,
            json,
            caption: title,
            details,
            full_model_hash: full_model_hash.map(str::to_owned),
        })
    }
}
