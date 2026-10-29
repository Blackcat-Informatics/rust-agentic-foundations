// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One cached diagram computation and checked ownership of its publication assets.

mod manifest;
pub(crate) mod publication;

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use purrdf_lex::json::{self, Limits as JsonLimits, Reader};
use raf_okf::{Body, DiagramDeclaration, DocType, Location};
use raf_source::{EntryKind, RelPath, Tree, TreeError};

use super::{DiagramBundle, DiagramError, NamedDiagramView, semantic::hash};
use crate::{Finding, SyncOptions, Workplan, finding::kind};
use manifest::Manifest;
use std::fmt::Write as _;

pub(crate) const MANIFEST: &str = "docs/diagrams/manifest.ttl";
const MAX_ASSETS: usize = 4096;
const MAX_BYTES: u64 = 4 * 1024 * 1024;
const MAX_TOTAL: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Row {
    owner: String,
    view: String,
    path: RelPath,
    digest: String,
}

pub(super) fn stem(owner: &str, view: &str) -> String {
    format!("docs/diagrams/{}/{}", hash(owner), hash(view))
}

#[derive(Debug)]
struct Asset {
    row: Row,
    body: String,
    format: &'static str,
}

#[derive(Debug)]
struct Block {
    path: String,
    name: String,
    markdown: String,
}

/// The renderer runs once after model loading; check, sync and preview reuse it.
#[derive(Debug, Default)]
pub(crate) struct Stored {
    manifest: Option<Manifest>,
    manifest_text: Option<String>,
    assets: Vec<Asset>,
    blocks: Vec<Block>,
    current: OrderedTable<RelPath, Option<String>>,
    faults: Vec<Finding>,
    views: Vec<(String, std::sync::Arc<DiagramBundle>)>,
}

fn finding(path: &str, error: impl std::fmt::Display) -> Finding {
    Finding::new(
        kind::DIAGRAM_INVALID,
        path,
        Location::Document,
        error.to_string(),
        "repair the named diagram declaration, owned asset or its native input; no partial diagram is published",
    )
}

pub(crate) fn load(tree: &dyn Tree, plan: &Workplan<'_>) -> Stored {
    match loaded(tree, plan) {
        Ok(stored) => stored,
        Err(error) => Stored {
            faults: vec![finding(MANIFEST, error)],
            ..Stored::default()
        },
    }
}

fn loaded(tree: &dyn Tree, plan: &Workplan<'_>) -> Result<Stored, DiagramError> {
    let mut stored = Stored::default();
    let manifest_path = RelPath::new(MANIFEST).map_err(DiagramError::data)?;
    stored.manifest_text = read(tree, &manifest_path)?;
    stored.manifest = stored
        .manifest_text
        .as_deref()
        .map(Manifest::parse)
        .transpose()?;
    let mut views = 0;
    for cap in plan.capabilities() {
        let owner = DocType::Capability
            .document_iri(&cap.value().header.id)
            .map_err(DiagramError::data)?;
        add_views(
            plan,
            &owner,
            cap.path(),
            &cap.value().diagrams,
            &mut views,
            &mut stored,
        )?;
    }
    for page in plan.architecture() {
        let owner = DocType::Architecture
            .document_iri(&page.value().header.id)
            .map_err(DiagramError::data)?;
        add_views(
            plan,
            &owner,
            page.path(),
            &page.value().diagrams,
            &mut views,
            &mut stored,
        )?;
    }
    let mut paths = OrderedTable::default();
    let mut admission = Resident;
    let mut memory = Memory::new(&mut admission);
    for path in stored.assets.iter().map(|asset| &asset.row.path) {
        if paths
            .insert(path.clone(), (), &mut memory)
            .map_err(DiagramError::data)?
            .is_some()
        {
            return Err(DiagramError::refused(format!(
                "{path} is produced by multiple views; rename a declared view to avoid its generated detail names"
            )));
        }
    }
    for path in stored
        .manifest
        .iter()
        .flat_map(|manifest| manifest.rows.iter().map(|row| &row.path))
    {
        paths
            .insert(path.clone(), (), &mut memory)
            .map_err(DiagramError::data)?;
    }
    let mut current_bytes = 0usize;
    for (path, ()) in paths.ordered() {
        let text = read(tree, path)?;
        current_bytes = current_bytes
            .checked_add(text.as_ref().map_or(0, String::len))
            .ok_or_else(|| DiagramError::refused("owned asset byte accounting overflow"))?;
        if current_bytes > MAX_TOTAL {
            return Err(DiagramError::refused(format!(
                "owned diagram text exceeds 32 MiB at {path}"
            )));
        }
        stored
            .current
            .insert(path.clone(), text, &mut memory)
            .map_err(DiagramError::data)?;
    }
    Ok(stored)
}

fn read(tree: &dyn Tree, path: &RelPath) -> Result<Option<String>, DiagramError> {
    for ancestor in path
        .as_str()
        .split('/')
        .scan(String::new(), |prefix, part| {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(part);
            Some(prefix.clone())
        })
    {
        let item = RelPath::new(&ancestor).map_err(DiagramError::data)?;
        match tree.entry(&item) {
            Ok(None) => return Ok(None),
            Ok(Some(EntryKind::Dir)) if item != *path => {}
            Ok(Some(EntryKind::File)) if item == *path => {}
            Ok(_) => {
                return Err(DiagramError::refused(format!(
                    "owned asset {path} crosses a link, non-directory parent or non-regular destination"
                )));
            }
            Err(error) => return Err(DiagramError::data(error)),
        }
    }
    match tree.read(path, MAX_BYTES) {
        Ok(text) => Ok(Some(text)),
        Err(TreeError::NotFound { .. }) => Ok(None),
        Err(error) => Err(DiagramError::data(error)),
    }
}

fn add_views(
    plan: &Workplan<'_>,
    owner: &str,
    path: &str,
    declarations: &[DiagramDeclaration],
    count: &mut usize,
    stored: &mut Stored,
) -> Result<(), DiagramError> {
    for declaration in declarations {
        *count += 1;
        if *count > 256 {
            return Err(DiagramError::refused(
                "more than 256 declared views; split the document set into explicit scopes",
            ));
        }
        let mut view = NamedDiagramView::from_declaration(declaration)?;
        view.owner = Some(owner.to_owned());
        let bundle = plan.diagram(&view)?;
        let mut markdown = format!(
            "![{}]({}/diagram.svg)\n\n{}\n\n",
            crate::generated::cell(bundle.caption()),
            relative(path, &stem(owner, view.name())),
            crate::generated::cell(bundle.caption())
        );
        markdown.push_str(&links(path, owner, &bundle));
        for detail in bundle.details() {
            write!(
                markdown,
                "\nDetail: [{}]({}/diagram.svg)\n",
                crate::generated::cell(detail.caption()),
                relative(path, &stem(owner, detail.view().name()))
            )
            .expect("writing to String cannot fail");
        }
        stored.blocks.push(Block {
            path: path.into(),
            name: format!("diagram-{}", declaration.name),
            markdown,
        });
        add_bundle(owner, &bundle, stored)?;
        stored.views.push((
            format!("{owner}#{}", view.name()),
            std::sync::Arc::new(bundle),
        ));
    }
    Ok(())
}

impl Workplan<'_> {
    /// The cached native bundle for `cap:<slug>#<view>` or `arch:<slug>#<view>`.
    ///
    /// This is the same computation that check, preview and sync judge.
    ///
    /// # Errors
    /// Refuses unknown declarations, malformed references or incomplete native assets.
    pub fn named_diagram(
        &self,
        reference: &str,
    ) -> Result<std::sync::Arc<DiagramBundle>, DiagramError> {
        let (owner, name) = reference.split_once('#').ok_or_else(|| {
            DiagramError::refused("named view must be cap:<slug>#<name> or arch:<slug>#<name>")
        })?;
        if !raf_okf::is_kebab(name) {
            return Err(DiagramError::refused(
                "named view fragment is not a kebab slug",
            ));
        }
        let key = format!("{}#{name}", super::semantic::doc(owner)?);
        if let Some((_, view)) = self
            .diagram_assets
            .views
            .iter()
            .find(|(known, _)| *known == key)
        {
            return Ok(view.clone());
        }
        if !self.diagram_assets.faults.is_empty() {
            return Err(DiagramError::refused(
                self.diagram_assets
                    .faults
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n"),
            ));
        }
        Err(DiagramError::refused(format!(
            "no declared diagram is named {reference}"
        )))
    }
}

fn relative(document: &str, target: &str) -> String {
    let levels = document.split('/').count().saturating_sub(1);
    format!("{}{target}", "../".repeat(levels))
}

fn links(path: &str, owner: &str, bundle: &DiagramBundle) -> String {
    let base = relative(path, &stem(owner, bundle.view().name()));
    format!(
        "[Text alternative]({base}/diagram.txt) · [Semantic RDF]({base}/diagram.ttl) · [Structured view]({base}/diagram.json)\n"
    )
}

fn add_bundle(
    owner: &str,
    bundle: &DiagramBundle,
    stored: &mut Stored,
) -> Result<(), DiagramError> {
    let rdf = manifest::serialize(bundle.dataset())?;
    let body_bytes = bundle.svg().len() + bundle.text().len() + bundle.json().len() + rdf.len();
    if body_bytes > MAX_TOTAL
        || stored
            .assets
            .iter()
            .map(|asset| asset.body.len())
            .sum::<usize>()
            > MAX_TOTAL - body_bytes
    {
        return Err(DiagramError::refused(
            "diagram collection exceeds 32 MiB of artifact text",
        ));
    }
    for (file, body, format) in [
        ("diagram.svg", bundle.svg(), "svg"),
        ("diagram.json", bundle.json(), "json"),
        ("diagram.txt", bundle.text(), "text"),
        ("diagram.ttl", rdf.as_str(), "rdf"),
    ] {
        if stored.assets.len() >= MAX_ASSETS
            || u64::try_from(body.len()).map_or(true, |len| len > MAX_BYTES)
        {
            return Err(DiagramError::refused(
                "diagram asset count or per-file byte limit exceeded",
            ));
        }
        let path = RelPath::new(&format!("{}/{file}", stem(owner, bundle.view().name())))
            .map_err(DiagramError::data)?;
        stored.assets.push(Asset {
            row: Row {
                owner: owner.into(),
                view: bundle.view().name().into(),
                path,
                digest: String::new(),
            },
            body: body.into(),
            format,
        });
    }
    for detail in bundle.details() {
        add_bundle(owner, detail, stored)?;
    }
    Ok(())
}

fn header(asset: &Asset, manifest: &Manifest) -> Result<String, DiagramError> {
    match asset.format {
        "svg" => Ok(format!(
            "<!-- SPDX-FileCopyrightText: {} -->\n<!-- SPDX-License-Identifier: {} -->\n{}",
            manifest.copyright, manifest.licence, asset.body
        )),
        "json" => {
            let mut reader = Reader::new(&asset.body, JsonLimits::default());
            let value = reader.read_value().map_err(DiagramError::data)?;
            let mut object = value
                .as_object()
                .ok_or_else(|| DiagramError::refused("structured diagram export is not an object"))?
                .clone();
            object.insert("SPDX-FileCopyrightText", manifest.copyright.clone());
            object.insert("SPDX-License-Identifier", manifest.licence.clone());
            Ok(json::write_compact(&object.into()))
        }
        _ => Ok(format!(
            "# SPDX-FileCopyrightText: {}\n# SPDX-License-Identifier: {}\n\n{}",
            manifest.copyright, manifest.licence, asset.body
        )),
    }
}

#[derive(Debug, Default)]
pub(crate) struct Analysis {
    pub findings: Vec<Finding>,
    pub refusals: Vec<Finding>,
    pub edits: Vec<(String, String)>,
    pub publication: Option<publication::Publication>,
}

impl Stored {
    pub(crate) fn rewrite(
        &self,
        path: &str,
        head: &str,
        body: &Body,
    ) -> Result<Option<String>, Finding> {
        let splices = self.blocks(path, body)?;
        if splices.is_empty() {
            return Ok(None);
        }
        let splices: Vec<_> = splices
            .iter()
            .map(|(span, text)| (span.clone(), text.as_str()))
            .collect();
        let mut body = body.clone();
        body.replace_spans(&splices)
            .map_err(|error| finding(path, error))?;
        Ok(Some(format!("{head}{}", body.text())))
    }
    pub(crate) fn inputs(&self) -> impl Iterator<Item = &RelPath> {
        self.current
            .ordered()
            .filter_map(|(path, text)| text.as_ref().map(|_| path))
    }

    pub(crate) fn has_manifest(&self) -> bool {
        self.manifest_text.is_some()
    }
    pub(crate) fn analyze(&self, options: Option<&SyncOptions>) -> Analysis {
        match self.expected(options) {
            Ok(analysis) => analysis,
            Err(error) => Analysis {
                refusals: vec![finding(MANIFEST, error)],
                ..Analysis::default()
            },
        }
    }

    fn expected(&self, options: Option<&SyncOptions>) -> Result<Analysis, DiagramError> {
        if !self.faults.is_empty() {
            return Ok(Analysis {
                refusals: self.faults.clone(),
                ..Analysis::default()
            });
        }
        if self.assets.is_empty() && self.manifest.is_none() {
            return Ok(Analysis::default());
        }
        let Some(mut wanted) = self.ownership(options)? else {
            return Ok(Analysis {
                findings: vec![Finding::new(
                    kind::DIAGRAM_STALE,
                    MANIFEST,
                    Location::Document,
                    "diagram ownership manifest is missing",
                    "run docs sync with the SPDX options used for generated assets",
                )],
                ..Analysis::default()
            });
        };
        let mut edits: Vec<(String, String)> = Vec::new();
        let mut guards = Vec::new();
        let mut removals = Vec::new();
        let mut bytes = 0usize;
        for asset in &self.assets {
            let current = self.current.get(&asset.row.path).and_then(Option::as_deref);
            let previous = self
                .manifest
                .iter()
                .flat_map(|manifest| &manifest.rows)
                .find(|row| row.path == asset.row.path);
            if current.is_some() && previous.is_none() {
                return Err(DiagramError::refused(format!(
                    "{} exists without diagram ownership; choose another declared view name or move the authored file",
                    asset.row.path
                )));
            }
            let text = header(asset, &wanted)?;
            bytes = bytes.checked_add(text.len()).ok_or_else(|| {
                DiagramError::refused("diagram publication byte accounting overflow")
            })?;
            if u64::try_from(text.len()).map_or(true, |len| len > MAX_BYTES) || bytes > MAX_TOTAL {
                return Err(DiagramError::refused(
                    "SPDX-bearing artifacts exceed per-file or aggregate text limits",
                ));
            }
            let row = Row {
                digest: hash(&text),
                ..asset.row.clone()
            };
            if current != Some(text.as_str()) {
                edits.push((row.path.as_str().into(), text));
                guards.push(publication::Guard {
                    path: row.path.clone(),
                    before: current.map(hash),
                    after: Some(row.digest.clone()),
                });
            }
            wanted.rows.push(row);
        }
        for row in self.manifest.iter().flat_map(|manifest| &manifest.rows) {
            if !wanted.rows.iter().any(|new| new.path == row.path) {
                let current = self.current.get(&row.path).and_then(Option::as_deref);
                if let Some(current) = current {
                    removals.push(publication::Guard {
                        path: row.path.clone(),
                        before: Some(hash(current)),
                        after: None,
                    });
                }
            }
        }
        purrdf_lex::walk::try_sort_unstable_by(&mut wanted.rows, |a, b| {
            Ok::<_, DiagramError>(a.path.cmp(&b.path))
        })
        .map_err(DiagramError::data)?;
        self.publication(&wanted, edits, guards, removals)
    }

    fn ownership(&self, options: Option<&SyncOptions>) -> Result<Option<Manifest>, DiagramError> {
        if let Some(manifest) = &self.manifest {
            return Ok(Some(Manifest {
                rows: Vec::new(),
                ..manifest.clone()
            }));
        }
        let Some(options) = options else {
            return Ok(None);
        };
        let (copyright, licence) = options.spdx().map_err(DiagramError::data)?;
        Ok(Some(Manifest {
            copyright: copyright.into(),
            licence: licence.into(),
            rows: Vec::new(),
        }))
    }

    fn publication(
        &self,
        wanted: &Manifest,
        mut edits: Vec<(String, String)>,
        guards: Vec<publication::Guard>,
        removals: Vec<publication::Guard>,
    ) -> Result<Analysis, DiagramError> {
        let manifest = wanted.text()?;
        if self.manifest_text.as_deref() != Some(manifest.as_str())
            || !edits.is_empty()
            || !removals.is_empty()
        {
            edits.push((MANIFEST.into(), manifest.clone()));
            let mut intent = wanted.clone();
            for row in self.manifest.iter().flat_map(|old| &old.rows) {
                if !intent.rows.iter().any(|new| new.path == row.path) {
                    intent.rows.push(row.clone());
                }
            }
            purrdf_lex::walk::try_sort_unstable_by(&mut intent.rows, |a, b| {
                Ok::<_, DiagramError>(a.path.cmp(&b.path))
            })
            .map_err(DiagramError::data)?;
            let publication = publication::Publication {
                original_manifest: self.manifest_text.as_deref().map(hash),
                intent: intent.text()?,
                manifest,
                guards,
                removals,
            };
            let findings = edits
                .iter()
                .map(|(path, _)| {
                    Finding::new(
                        kind::DIAGRAM_STALE,
                        path,
                        Location::Document,
                        "owned diagram artifact differs from its named semantic view",
                        "run docs sync to regenerate owned assets and preserve unrelated files",
                    )
                })
                .collect();
            return Ok(Analysis {
                findings,
                edits,
                publication: Some(publication),
                ..Analysis::default()
            });
        }
        Ok(Analysis::default())
    }

    pub(crate) fn blocks(
        &self,
        path: &str,
        body: &Body,
    ) -> Result<Vec<(raf_okf::Span, String)>, Finding> {
        if (self.blocks.iter().any(|block| block.path == path)
            || body
                .generated_blocks()
                .iter()
                .any(|block| block.kind().starts_with("diagram-")))
            && crate::generated::mixed_lines(body.text()).is_some()
        {
            return Err(finding(
                path,
                "a diagram-bearing body mixes line endings; use one line ending before preview or synchronization",
            ));
        }
        let mut splices = Vec::new();
        for block in body
            .generated_blocks()
            .iter()
            .filter(|block| block.kind().starts_with("diagram-"))
        {
            if path.starts_with("docs/capabilities/")
                && !body.sections().iter().any(|section| {
                    section.content().bytes.start <= block.outer().bytes.start
                        && block.outer().bytes.end <= section.content().bytes.end
                })
            {
                return Err(finding(
                    path,
                    "a capability diagram reference belongs inside one of its sections",
                ));
            }
            let Some(expected) = self
                .blocks
                .iter()
                .find(|expected| expected.path == path && expected.name == block.kind())
            else {
                return Err(finding(
                    path,
                    format!(
                        "{} has no named diagram declaration on this document",
                        block.kind()
                    ),
                ));
            };
            let content = &body.text()[block.content().bytes.start - body.span().bytes.start
                ..block.content().bytes.end - body.span().bytes.start];
            let text = if body.text().contains("\r\n") {
                expected.markdown.replace('\n', "\r\n")
            } else {
                expected.markdown.clone()
            };
            if content != text {
                splices.push((block.content().clone(), text));
            }
        }
        for declared in self.blocks.iter().filter(|block| block.path == path) {
            if !body
                .generated_blocks()
                .iter()
                .any(|block| block.kind() == declared.name)
            {
                return Err(finding(
                    path,
                    format!(
                        "declared view {} requires <!-- raf:generated:{} --> and the ordinary end marker",
                        declared.name, declared.name
                    ),
                ));
            }
        }
        Ok(splices)
    }
}
