// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Automatic RAF relationships over the loaded document and Source models.

use std::sync::Arc;

use purrdf_core::{RdfDataset, RdfDatasetBuilder, RdfLiteral, TermId};
use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use purrdf_lex::walk::try_sort_unstable_by;
use raf_okf::{DocType, InputKind};
use raf_source::{DependencyKind, DependencySpec};

use super::{DiagramError, DiagramGrouping, DiagramKind, NamedDiagramView};
pub(super) use crate::digest::semantic_hash as hash;
use crate::{Workplan, digest::Digest};

pub(super) const VOCAB: &str = crate::evidence::CLAIMS_VOCAB;
const NODES: &str = "https://blackcatinformatics.ca/raf/workplan/views/";
const TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
const STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

pub(super) fn doc(id: &str) -> Result<String, DiagramError> {
    let kind = DocType::ALL
        .into_iter()
        .find(|kind| id.starts_with(kind.id_prefix()))
        .ok_or_else(|| DiagramError::refused(format!("{id} is not a document id")))?;
    kind.document_iri(id).map_err(DiagramError::data)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Entity {
    pub iri: String,
    pub label: String,
    pub group: String,
    pub fields: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Relation {
    pub from: String,
    pub kind: String,
    pub to: String,
    pub fields: Vec<(String, String)>,
}

pub(super) struct Facts {
    pub entities: OrderedTable<String, Entity>,
    pub relations: Vec<Relation>,
    max_nodes: usize,
    max_relations: usize,
}

impl Facts {
    fn new(view: &NamedDiagramView) -> Self {
        Self {
            entities: OrderedTable::default(),
            relations: Vec::new(),
            max_nodes: view.spec.max_terms,
            max_relations: view.spec.max_statements,
        }
    }

    fn entity(
        &mut self,
        iri: String,
        label: &str,
        group: &str,
        mut fields: Vec<(String, String)>,
    ) -> Result<String, DiagramError> {
        if self.entities.get(&iri).is_none() {
            if self.entities.len() >= self.max_nodes {
                return Err(DiagramError::refused(
                    "term limit exceeded; focus the view or increase max_terms",
                ));
            }
            try_sort_unstable_by(&mut fields, |a, b| Ok::<_, DiagramError>(a.cmp(b)))
                .map_err(DiagramError::data)?;
            fields.dedup();
            let value = Entity {
                iri: iri.clone(),
                label: label.to_owned(),
                group: group.to_owned(),
                fields,
            };
            let mut admission = Resident;
            self.entities
                .insert(iri.clone(), value, &mut Memory::new(&mut admission))
                .map_err(DiagramError::data)?;
        }
        Ok(iri)
    }

    fn document(&mut self, id: &str) -> Result<String, DiagramError> {
        self.entity(doc(id)?, id, id.split('#').next().unwrap_or(id), Vec::new())
    }

    fn relation(
        &mut self,
        from: String,
        kind: &str,
        to: String,
        mut fields: Vec<(String, String)>,
    ) -> Result<(), DiagramError> {
        if self.relations.len() >= self.max_relations {
            return Err(DiagramError::refused(
                "relationship limit exceeded; focus the view or increase max_statements",
            ));
        }
        try_sort_unstable_by(&mut fields, |a, b| Ok::<_, DiagramError>(a.cmp(b)))
            .map_err(DiagramError::data)?;
        self.relations.push(Relation {
            from,
            kind: kind.to_owned(),
            to,
            fields,
        });
        Ok(())
    }

    fn scoped(&mut self, view: &NamedDiagramView) -> Result<(), DiagramError> {
        let allowed: &[&str] = match view.kind() {
            DiagramKind::WorkspaceDependencies => &[
                "normalDependency",
                "devDependency",
                "buildDependency",
                "hasLockedSource",
            ],
            DiagramKind::CapabilityDependencies => &["dependsOn"],
            DiagramKind::Ownership => &["implementedBy", "testedBy", "exemplifiedBy"],
            DiagramKind::Coverage => &["hasEvidence", "hasAssessment", "referencesEvidence"],
            DiagramKind::ChangeImpact => &["changes", "affects", "dependsOnAffected"],
            DiagramKind::EvidenceClaims => &[],
        };
        for kind in &view.relations {
            if !allowed.contains(&kind.as_str()) {
                return Err(DiagramError::refused(format!(
                    "relationship `{kind}` is not available in this {} scope",
                    view.kind().as_str()
                )));
            }
        }
        let mut focused = Vec::new();
        for focus in &view.focus {
            let matches: Vec<_> = self
                .entities
                .ordered()
                .filter(|(_, entity)| entity.label == *focus || entity.iri == *focus)
                .map(|(id, _)| id.clone())
                .collect();
            if matches.is_empty() {
                return Err(DiagramError::refused(format!(
                    "focus `{focus}` names no entity in the selected scope"
                )));
            }
            focused.extend(matches);
        }
        self.relations.retain(|relation| {
            (view.relations.is_empty() || view.relations.contains(&relation.kind))
                && (focused.is_empty()
                    || focused.contains(&relation.from)
                    || focused.contains(&relation.to))
        });
        try_sort_unstable_by(&mut self.relations, |a, b| Ok::<_, DiagramError>(a.cmp(b)))
            .map_err(DiagramError::data)?;
        self.relations.dedup();
        if !view.focus.is_empty() || !view.relations.is_empty() {
            let mut admission = Resident;
            let mut memory = Memory::new(&mut admission);
            let mut selected = OrderedTable::default();
            for relation in &self.relations {
                selected
                    .insert(relation.from.clone(), (), &mut memory)
                    .map_err(DiagramError::data)?;
                selected
                    .insert(relation.to.clone(), (), &mut memory)
                    .map_err(DiagramError::data)?;
            }
            for id in focused {
                selected
                    .insert(id, (), &mut memory)
                    .map_err(DiagramError::data)?;
            }
            let mut kept = OrderedTable::default();
            for (id, entity) in self.entities.ordered() {
                if selected.get(id).is_some() {
                    kept.insert(id.clone(), entity.clone(), &mut memory)
                        .map_err(DiagramError::data)?;
                }
            }
            self.entities = kept;
        }
        Ok(())
    }
}

fn included(scope: &str, id: &str) -> bool {
    scope == "workspace" || scope == id || id.split_once(':').is_some_and(|(_, slug)| slug == scope)
}

fn capabilities(plan: &Workplan<'_>, scope: &str) -> Result<(), DiagramError> {
    if scope != "workspace"
        && !plan
            .capabilities()
            .iter()
            .any(|cap| included(scope, &cap.value().header.id))
    {
        return Err(DiagramError::refused(format!(
            "unknown capability scope `{scope}`"
        )));
    }
    Ok(())
}

pub(super) fn collect(
    plan: &Workplan<'_>,
    view: &NamedDiagramView,
    kind: DiagramKind,
    scope: &str,
) -> Result<Facts, DiagramError> {
    match (kind, view.grouping) {
        (DiagramKind::WorkspaceDependencies, DiagramGrouping::Capability)
        | (
            DiagramKind::CapabilityDependencies | DiagramKind::Coverage | DiagramKind::ChangeImpact,
            DiagramGrouping::Package,
        ) => {
            return Err(DiagramError::refused(format!(
                "{} has no {} ownership mapping; use none or the native family grouping",
                kind.as_str(),
                view.grouping.as_str()
            )));
        }
        _ => {}
    }
    if !plan.unloaded().is_empty() {
        return Err(DiagramError::refused(format!(
            "documents did not load: {}",
            plan.unloaded().join(", ")
        )));
    }
    let invalid: Vec<_> = plan
        .findings
        .iter()
        .take(plan.semantic_findings)
        .filter(|finding| !generated_link(plan, finding))
        .collect();
    if !invalid.is_empty() {
        return Err(DiagramError::refused(
            invalid
                .into_iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }
    let mut facts = Facts::new(view);
    match kind {
        DiagramKind::WorkspaceDependencies => dependencies(plan, scope, &mut facts)?,
        DiagramKind::CapabilityDependencies => {
            capabilities(plan, scope)?;
            for cap in plan
                .capabilities()
                .iter()
                .filter(|cap| included(scope, &cap.value().header.id))
            {
                facts.document(&cap.value().header.id)?;
            }
            for edge in plan
                .graph()
                .depends()
                .iter()
                .filter(|edge| included(scope, edge.from()) || included(scope, edge.to()))
            {
                let from = facts.document(edge.from())?;
                let to = facts.document(edge.to())?;
                facts.relation(
                    from,
                    "dependsOn",
                    to,
                    vec![("origin".into(), "authored-declaration".into())],
                )?;
            }
        }
        DiagramKind::Ownership => ownership(plan, scope, &mut facts)?,
        DiagramKind::Coverage => coverage(plan, scope, &mut facts)?,
        DiagramKind::ChangeImpact => impact(plan, scope, &mut facts)?,
        DiagramKind::EvidenceClaims => {
            return Err(DiagramError::refused(
                "evidence claims use their existing native claim selection",
            ));
        }
    }
    facts.scoped(view)?;
    Ok(facts)
}

// Generated Markdown is output, not an input to these semantic families. A
// removed record can stale its old relationship row before sync repairs it.
fn generated_link(plan: &Workplan<'_>, finding: &crate::Finding) -> bool {
    if finding.kind != crate::finding::kind::LINK_BROKEN {
        return false;
    }
    let raf_okf::Location::Line(line) = finding.location else {
        return false;
    };
    let body = plan
        .capabilities()
        .iter()
        .map(|doc| (doc.path(), doc.body()))
        .chain(
            plan.architecture()
                .iter()
                .map(|doc| (doc.path(), doc.body())),
        )
        .find_map(|(path, body)| (path == finding.file).then_some(body))
        .or_else(|| match &plan.index {
            crate::model::IndexSlot::Valid(index)
                if finding.file == crate::generated::INDEX_PATH =>
            {
                Some(index.body())
            }
            _ => None,
        });
    body.is_some_and(|body| {
        body.generated_blocks()
            .iter()
            .any(|block| block.outer().lines.contains(&line))
    })
}

fn spec_fields(spec: &DependencySpec, prefix: &str, fields: &mut Vec<(String, String)>) {
    for (key, value) in [
        ("package", &spec.package),
        ("versionRequirement", &spec.version),
        ("pathDeclaration", &spec.path),
        ("registry", &spec.registry),
        ("registryIndex", &spec.registry_index),
        ("gitDeclaration", &spec.git),
        ("revisionDeclaration", &spec.rev),
        ("branchDeclaration", &spec.branch),
        ("tagDeclaration", &spec.tag),
    ] {
        if let Some(value) = value {
            fields.push((format!("{prefix}{key}"), value.clone()));
        }
    }
    for (key, value) in [
        ("optional", spec.optional),
        ("defaultFeatures", spec.default_features),
    ] {
        if let Some(value) = value {
            fields.push((format!("{prefix}{key}"), value.to_string()));
        }
    }
    fields.push((
        format!("{prefix}workspaceInheritance"),
        spec.workspace.to_string(),
    ));
    for feature in &spec.features {
        fields.push((format!("{prefix}requestedFeature"), feature.clone()));
    }
}

fn package_node(
    source: &raf_source::Workspace,
    package: &raf_source::Package,
    package_id: raf_source::PackageId,
    facts: &mut Facts,
) -> Result<String, DiagramError> {
    let mut root_fields = vec![
            ("identityScope".into(), "export-scoped package selection with checked target inputs; not a complete manifest or repository content identity".into()),
            (
                "manifestLocator".into(),
                package.manifest.as_str().to_owned(),
            ),
            (
                "configuration".into(),
                "declared; target and feature conditions unevaluated".into(),
            ),
        ];
    let mut roots = Vec::new();
    for target in source
        .targets()
        .filter(|target| target.package_id == package_id)
    {
        let digest = Digest::symbol(source, &target.crate_path).map_err(DiagramError::data)?;
        roots.push(format!(
            "{}\0{}\0{}",
            target.crate_path,
            target.root.as_str(),
            digest
        ));
        root_fields.push((
            "targetInput".into(),
            format!("{} {digest}", target.crate_path),
        ));
    }
    try_sort_unstable_by(&mut roots, |a, b| Ok::<_, DiagramError>(a.cmp(b)))
        .map_err(DiagramError::data)?;
    facts.entity(
        format!(
            "{NODES}package-selection/{}",
            hash(&format!(
                "{}\0{}\0{}",
                package.name,
                package.manifest,
                roots.join("\0")
            ))
        ),
        &package.name,
        &package.name,
        root_fields,
    )
}

fn dependencies(plan: &Workplan<'_>, scope: &str, facts: &mut Facts) -> Result<(), DiagramError> {
    let source = plan.source();
    if scope != "workspace"
        && !source
            .packages()
            .iter()
            .any(|package| package.name == scope)
    {
        return Err(DiagramError::refused(format!(
            "unknown package scope `{scope}`"
        )));
    }
    for package in source
        .packages()
        .iter()
        .filter(|package| scope == "workspace" || package.name == scope)
    {
        let package_id = source
            .package_ids()
            .find(|id| {
                source
                    .package(*id)
                    .is_some_and(|known| std::ptr::eq(known, package))
            })
            .ok_or_else(|| DiagramError::refused("loaded package has no native identity"))?;
        let from = package_node(source, package, package_id, facts)?;
        let declarations = source.dependency_declarations(package_id).ok_or_else(|| {
            DiagramError::refused("native dependency declarations are unavailable")
        })?;
        for declaration in declarations {
            let mut fields = vec![
                ("origin".into(), "authored-cargo-declaration".into()),
                (
                    "manifestLocator".into(),
                    package.manifest.as_str().to_owned(),
                ),
                ("declarationKey".into(), declaration.key.clone()),
            ];
            if let Some(condition) = &declaration.condition {
                fields.push(("targetCondition".into(), condition.clone()));
            }
            spec_fields(&declaration.specification, "", &mut fields);
            if let Some(inherited) = &declaration.inherited {
                fields.push((
                    "inheritedManifestLocator".into(),
                    inherited.manifest.as_str().to_owned(),
                ));
                spec_fields(&inherited.specification, "inherited_", &mut fields);
            }
            let name = declaration
                .specification
                .package
                .as_deref()
                .or_else(|| {
                    declaration
                        .inherited
                        .as_ref()
                        .and_then(|value| value.specification.package.as_deref())
                })
                .unwrap_or(&declaration.name);
            let to = facts.entity(
                format!(
                    "{NODES}dependency-declaration/{}",
                    hash(&format!("{from}\0{}\0{name}", declaration.key))
                ),
                name,
                name,
                Vec::new(),
            )?;
            let relation = match declaration.kind {
                DependencyKind::Normal => "normalDependency",
                DependencyKind::Dev => "devDependency",
                DependencyKind::Build => "buildDependency",
            };
            facts.relation(from.clone(), relation, to, fields)?;
        }
        if let Some(binding) = source.locked_binding(package_id) {
            locked_source(binding, from, facts)?;
        }
    }
    Ok(())
}

fn locked_source(
    binding: &raf_source::LockedBinding,
    from: String,
    facts: &mut Facts,
) -> Result<(), DiagramError> {
    let mut fields = vec![
        ("origin".into(), "checked-locked-source-binding".into()),
        ("version".into(), binding.version().into()),
    ];
    if let Some(identity) = binding.source() {
        fields.push(("lockedSourceKind".into(), format!("{:?}", identity.kind())));
        fields.push(("lockedRepository".into(), identity.repository().to_owned()));
        if let Some(reference) = identity.reference() {
            fields.push(("lockedReference".into(), format!("{reference:?}")));
        }
        if let Some(revision) = identity.revision() {
            fields.push(("lockedRevision".into(), revision.to_owned()));
        }
    }
    for configuration in binding.configurations() {
        fields.push(("observedConfiguration".into(), format!("{configuration:?}")));
    }
    let to = facts.entity(
        format!(
            "{NODES}locked-package/{}",
            hash(&format!(
                "{}\0{}\0{:?}",
                binding.name(),
                binding.version(),
                binding.source()
            ))
        ),
        binding.name(),
        binding.name(),
        fields,
    )?;
    facts.relation(
        from,
        "hasLockedSource",
        to,
        vec![(
            "configuration".into(),
            "native source binding; not feature-unified Cargo execution".into(),
        )],
    )?;

    Ok(())
}

fn ownership(plan: &Workplan<'_>, scope: &str, facts: &mut Facts) -> Result<(), DiagramError> {
    capabilities(plan, scope)?;
    for binding in plan
        .graph()
        .bindings()
        .iter()
        .filter(|binding| included(scope, binding.capability()))
    {
        binding
            .resolution()
            .map_err(|error| DiagramError::refused(format!("{}: {error}", binding.path())))?;
        let digest = Digest::symbol(plan.source(), binding.path()).map_err(DiagramError::data)?;
        let from = facts.document(binding.capability())?;
        let mut fields = vec![
            ("inputDigest".into(), digest.to_string()),
            ("bindingPath".into(), binding.path().into()),
        ];
        let mut groups = Vec::new();
        for id in binding.items() {
            let item = plan
                .source()
                .item(*id)
                .ok_or_else(|| DiagramError::refused("resolved binding item is unavailable"))?;
            let target = plan
                .source()
                .target(item.target)
                .ok_or_else(|| DiagramError::refused("resolved binding target is unavailable"))?;
            let authority = if item.file.authority() == raf_source::SourceAuthorityId::FIRST_PARTY {
                "first-party source".to_owned()
            } else {
                let identity = plan
                    .source()
                    .locked_binding(target.package_id)
                    .and_then(raf_source::LockedBinding::source)
                    .ok_or_else(|| {
                        DiagramError::refused(
                            "external binding source has no native checked source identity",
                        )
                    })?;
                format!(
                    "{:?} {} {:?} {:?}",
                    identity.kind(),
                    identity.repository(),
                    identity.reference(),
                    identity.revision()
                )
            };
            fields.push((
                "definitionSite".into(),
                format!(
                    "{}:{}; target {} {:?} {}; {authority}",
                    item.file, item.name_line, target.package, target.kind, target.name
                ),
            ));
            if !groups.contains(&target.package) {
                groups.push(target.package.clone());
            }
        }
        let group = if groups.len() == 1 {
            groups[0].as_str()
        } else {
            binding.capability()
        };
        let to = facts.entity(
            format!(
                "{NODES}bound-input/{}",
                hash(&format!("{}\0{digest}", binding.path()))
            ),
            binding.path(),
            group,
            fields,
        )?;
        facts.relation(
            from,
            match binding.kind() {
                crate::BindingKind::Code => "implementedBy",
                crate::BindingKind::Test => "testedBy",
                crate::BindingKind::Example => "exemplifiedBy",
            },
            to,
            vec![(
                "origin".into(),
                "resolved-source-binding; not execution proof".into(),
            )],
        )?;
    }
    Ok(())
}

fn coverage(plan: &Workplan<'_>, scope: &str, facts: &mut Facts) -> Result<(), DiagramError> {
    capabilities(plan, scope)?;
    for edge in plan
        .graph()
        .covers()
        .iter()
        .filter(|edge| included(scope, edge.capability()))
    {
        let from = facts.document(edge.capability())?;
        let to = facts.document(edge.evidence())?;
        let record = plan
            .evidence()
            .iter()
            .find(|record| record.value().header.id == edge.evidence());
        let mut fields = vec![("origin".into(), "recorded-execution-evidence".into())];
        if let Some(record) = record {
            fields.push(("verdict".into(), record.value().verdict.name().into()));
            let mut stale = Vec::new();
            for input in &record.value().inputs {
                if input.kind != InputKind::Symbol {
                    return Err(DiagramError::refused("unsupported evidence input kind"));
                }
                match Digest::symbol(plan.source(), &input.name) {
                    Ok(now) if now.to_string() == input.digest => {}
                    Ok(_) => stale.push(format!("{} changed", input.name)),
                    Err(error) if error.is_query_refusal() => {
                        return Err(DiagramError::data(error));
                    }
                    Err(error) => stale.push(format!("{}: {error}", input.name)),
                }
                fields.push((
                    "originalInput".into(),
                    format!("{} {}", input.name, input.digest),
                ));
            }
            fields.push((
                "currency".into(),
                if stale.is_empty() { "current" } else { "stale" }.into(),
            ));
            for reason in stale {
                fields.push(("staleReason".into(), reason));
            }
            if let Some(revision) = &record.value().revision {
                fields.push(("recordedRevision".into(), revision.clone()));
            }
        } else {
            fields.push(("currency".into(), "missing".into()));
        }
        facts.relation(from, "hasEvidence", to, fields)?;
    }
    for edge in plan
        .graph()
        .assesses()
        .iter()
        .filter(|edge| included(scope, edge.section().capability()))
    {
        let from = facts.document(edge.section().capability())?;
        let to = facts.document(edge.assessment())?;
        let record = plan
            .assessments()
            .iter()
            .find(|record| record.value().header.id == edge.assessment())
            .ok_or_else(|| DiagramError::refused("assessment edge has no loaded record"))?;
        facts.relation(
            from,
            "hasAssessment",
            to,
            vec![
                ("origin".into(), "authored-prose-assessment".into()),
                ("section".into(), edge.section().to_string()),
                ("verdict".into(), record.value().verdict.name().into()),
                (
                    "currency".into(),
                    if plan
                        .currency_checked(record.value())
                        .map_err(DiagramError::data)?
                        .is_current()
                    {
                        "current"
                    } else {
                        "stale"
                    }
                    .into(),
                ),
            ],
        )?;
    }
    claim_references(plan, scope, facts)?;
    Ok(())
}

fn claim_references(
    plan: &Workplan<'_>,
    scope: &str,
    facts: &mut Facts,
) -> Result<(), DiagramError> {
    for edge in plan
        .graph()
        .claims()
        .iter()
        .filter(|edge| included(scope, edge.capability()))
    {
        let from = facts.document(edge.capability())?;
        let to = facts.document(edge.evidence())?;
        facts.relation(
            from,
            "referencesEvidence",
            to,
            vec![
                (
                    "origin".into(),
                    "authored-reference; not proof of truth".into(),
                ),
                ("line".into(), edge.line().to_string()),
            ],
        )?;
    }
    Ok(())
}

fn impact(plan: &Workplan<'_>, scope: &str, facts: &mut Facts) -> Result<(), DiagramError> {
    if scope != "workspace"
        && !plan
            .changes()
            .iter()
            .any(|change| included(scope, &change.value().header.id))
    {
        return Err(DiagramError::refused(format!(
            "unknown change scope `{scope}`"
        )));
    }
    let mut touched = Vec::new();
    for edge in plan
        .graph()
        .changes_edges()
        .iter()
        .filter(|edge| included(scope, edge.change()))
    {
        let from = facts.document(edge.change())?;
        let to = facts.document(edge.capability())?;
        touched.push(edge.capability().to_owned());
        facts.relation(
            from,
            "changes",
            to,
            vec![("origin".into(), "authored-change-declaration".into())],
        )?;
    }
    for edge in plan
        .graph()
        .affects()
        .iter()
        .filter(|edge| included(scope, edge.change()))
    {
        let from = facts.document(edge.change())?;
        let to = facts.document(edge.section().capability())?;
        touched.push(edge.section().capability().to_owned());
        facts.relation(
            from,
            "affects",
            to,
            vec![
                ("section".into(), edge.section().to_string()),
                ("origin".into(), "authored-change-declaration".into()),
            ],
        )?;
    }
    loop {
        let mut added = false;
        for edge in plan.graph().depends() {
            if touched.iter().any(|id| id == edge.to())
                && !touched.iter().any(|id| id == edge.from())
            {
                touched.push(edge.from().into());
                added = true;
            }
        }
        if !added {
            break;
        }
        if touched.len() > facts.max_nodes {
            return Err(DiagramError::refused("impact closure exceeds term limit"));
        }
    }
    for edge in plan
        .graph()
        .depends()
        .iter()
        .filter(|edge| touched.iter().any(|id| id == edge.to()))
    {
        let from = facts.document(edge.from())?;
        let to = facts.document(edge.to())?;
        facts.relation(
            from,
            "dependsOnAffected",
            to,
            vec![(
                "origin".into(),
                "declared reverse-dependency impact; not runtime interaction".into(),
            )],
        )?;
    }
    Ok(())
}

struct Writer {
    native: RdfDatasetBuilder,
    count: usize,
    limit: usize,
}
impl Writer {
    fn iri(&mut self, iri: &str) -> Result<TermId, DiagramError> {
        self.native.intern_iri(iri).map_err(DiagramError::data)
    }
    fn literal(&mut self, text: &str) -> Result<TermId, DiagramError> {
        self.native
            .intern_literal(RdfLiteral {
                lexical_form: text.to_owned(),
                datatype: Some(STRING.into()),
                language: None,
                direction: None,
            })
            .map_err(DiagramError::data)
    }
    fn put(
        &mut self,
        subject: TermId,
        predicate: &str,
        object: TermId,
    ) -> Result<(), DiagramError> {
        if self.count >= self.limit {
            return Err(DiagramError::refused(
                "semantic statement limit exceeded; focus the view or increase max_statements",
            ));
        }
        let predicate = self.iri(predicate)?;
        self.native
            .push_quad(subject, predicate, object, None)
            .map_err(DiagramError::data)?;
        self.count += 1;
        Ok(())
    }
    fn text(&mut self, subject: TermId, key: &str, value: &str) -> Result<(), DiagramError> {
        let value = self.literal(value)?;
        self.put(subject, &format!("{VOCAB}{key}"), value)
    }
}

fn selection_iri(view: &NamedDiagramView) -> String {
    let scope = match &view.scope {
        super::DiagramScope::Automatic { scope, .. } => scope.as_str(),
        super::DiagramScope::Evidence { id } => id.as_str(),
    };
    format!(
        "{NODES}selection/{}",
        hash(&format!(
            "{:?}\0{}\0{}\0{scope}\0{:?}\0{:?}\0{:?}",
            view.owner,
            view.name,
            view.kind().as_str(),
            view.grouping,
            view.focus,
            view.relations
        ))
    )
}

fn group_iri(view: &NamedDiagramView, key: &str) -> String {
    format!(
        "{NODES}group/{}",
        hash(&format!("{}\0{key}", selection_iri(view)))
    )
}

pub(super) fn dataset(
    facts: &Facts,
    view: &NamedDiagramView,
    scope: &str,
) -> Result<Arc<RdfDataset>, DiagramError> {
    let mut writer = Writer {
        native: RdfDatasetBuilder::new(),
        count: 0,
        limit: view.spec.max_statements,
    };
    let selection = writer.iri(&selection_iri(view))?;
    let class = writer.iri(&format!("{VOCAB}DiagramSelection"))?;
    writer.put(selection, TYPE, class)?;
    writer.text(selection, "name", &view.name)?;
    writer.text(selection, "family", view.kind().as_str())?;
    writer.text(selection, "scope", scope)?;
    writer.text(selection, "grouping", view.grouping.as_str())?;
    for focus in &view.focus {
        writer.text(selection, "focus", focus)?;
    }
    for relation in &view.relations {
        writer.text(selection, "selectedRelation", relation)?;
    }
    for (_, entity) in facts.entities.ordered() {
        let subject = writer.iri(&entity.iri)?;
        let label = writer.literal(&entity.label)?;
        writer.put(subject, LABEL, label)?;
        for (key, value) in &entity.fields {
            if key == "member" || key == "detailView" {
                let member = writer.iri(value)?;
                writer.put(subject, &format!("{VOCAB}{key}"), member)?;
            } else {
                writer.text(subject, key, value)?;
            }
        }
    }
    for relation in &facts.relations {
        let subject = writer.iri(&relation.from)?;
        let predicate = writer.iri(&format!("{VOCAB}{}", relation.kind))?;
        let object = writer.iri(&relation.to)?;
        writer.put(subject, &format!("{VOCAB}{}", relation.kind), object)?;
        let statement = writer
            .native
            .intern_triple(subject, predicate, object)
            .map_err(DiagramError::data)?;
        let occurrence = writer.iri(&format!(
            "{NODES}relation/{}",
            hash(&format!("{relation:?}"))
        ))?;
        writer
            .native
            .push_reifier(occurrence, statement)
            .map_err(DiagramError::data)?;
        writer.put(selection, &format!("{VOCAB}relationship"), occurrence)?;
        for (key, value) in &relation.fields {
            if key == "aggregatedMember" {
                let member = writer.iri(value)?;
                writer.put(occurrence, &format!("{VOCAB}{key}"), member)?;
            } else {
                writer.text(occurrence, key, value)?;
            }
        }
    }
    writer.native.freeze().map_err(DiagramError::data)
}

struct Group {
    members: OrderedTable<String, ()>,
    relations: Vec<usize>,
}

/// Group indexes retain every original fact and give each endpoint a detail view.
pub(super) fn grouped(
    facts: &Facts,
    view: &NamedDiagramView,
) -> Result<(Facts, Vec<(String, Facts)>), DiagramError> {
    let mut groups: OrderedTable<String, Group> = OrderedTable::default();
    let mut summary = Facts::new(view);
    let mut admission = Resident;
    let mut memory = Memory::new(&mut admission);
    for (index, relation) in facts.relations.iter().enumerate() {
        let from = facts
            .entities
            .get(&relation.from)
            .ok_or_else(|| DiagramError::refused("group endpoint is missing"))?;
        let to = facts
            .entities
            .get(&relation.to)
            .ok_or_else(|| DiagramError::refused("group endpoint is missing"))?;
        let from_group = from.group.as_str();
        let to_group =
            if view.grouping == DiagramGrouping::Capability && from_group.starts_with("cap:") {
                from_group
            } else {
                to.group.as_str()
            };
        let mut endpoints = Vec::new();
        for (key, entity) in [(from_group, from), (to_group, to)] {
            let key = key.to_owned();
            if groups.get(&key).is_none() {
                groups
                    .insert(
                        key.clone(),
                        Group {
                            members: OrderedTable::default(),
                            relations: Vec::new(),
                        },
                        &mut memory,
                    )
                    .map_err(DiagramError::data)?;
            }
            let group = groups
                .get_mut(&key)
                .ok_or_else(|| DiagramError::refused("group index was not retained"))?;
            group
                .members
                .insert(entity.iri.clone(), (), &mut memory)
                .map_err(DiagramError::data)?;
            if group.relations.last() != Some(&index) {
                memory
                    .push(&mut group.relations, index)
                    .map_err(DiagramError::data)?;
            }
            endpoints.push(group_iri(view, &key));
        }
        let mut fields: Vec<_> = relation
            .fields
            .iter()
            .map(|(key, value)| (format!("member_{key}"), value.clone()))
            .collect();
        fields.push((
            "aggregatedMember".into(),
            format!("{NODES}relation/{}", hash(&format!("{relation:?}"))),
        ));
        fields.push(("origin".into(), "explicit semantic aggregation".into()));
        summary.relation(
            endpoints[0].clone(),
            &relation.kind,
            endpoints[1].clone(),
            fields,
        )?;
    }
    let mut details = Vec::new();
    // Preserve scoped isolates as explicit groups instead of losing them merely
    // because they have no relationship in this projection.
    for (_, entity) in facts.entities.ordered() {
        if !groups
            .ordered()
            .any(|(_, group)| group.members.get(&entity.iri).is_some())
        {
            if groups.get(&entity.group).is_none() {
                groups
                    .insert(
                        entity.group.clone(),
                        Group {
                            members: OrderedTable::default(),
                            relations: Vec::new(),
                        },
                        &mut memory,
                    )
                    .map_err(DiagramError::data)?;
            }
            groups
                .get_mut(&entity.group)
                .ok_or_else(|| DiagramError::refused("isolated group was not retained"))?
                .members
                .insert(entity.iri.clone(), (), &mut memory)
                .map_err(DiagramError::data)?;
        }
    }
    for (key, group) in groups.ordered() {
        details.push(group_detail(facts, view, key, group, &mut summary)?);
    }
    Ok((summary, details))
}

fn group_detail(
    facts: &Facts,
    view: &NamedDiagramView,
    key: &str,
    group: &Group,
    summary: &mut Facts,
) -> Result<(String, Facts), DiagramError> {
    let detail_name = format!("{}-{}", view.name, hash(key));
    let mut detail = Facts::new(view);
    for (id, ()) in group.members.ordered() {
        let entity = facts
            .entities
            .get(id)
            .ok_or_else(|| DiagramError::refused("detail member is missing"))?;
        detail.entity(
            entity.iri.clone(),
            &entity.label,
            &entity.group,
            entity.fields.clone(),
        )?;
    }
    for index in &group.relations {
        let relation = &facts.relations[*index];
        for id in [&relation.from, &relation.to] {
            let entity = facts
                .entities
                .get(id)
                .ok_or_else(|| DiagramError::refused("detail endpoint is missing"))?;
            detail.entity(
                entity.iri.clone(),
                &entity.label,
                &entity.group,
                entity.fields.clone(),
            )?;
        }
        detail.relation(
            relation.from.clone(),
            &relation.kind,
            relation.to.clone(),
            relation.fields.clone(),
        )?;
    }
    let mut detail_view = view.clone();
    detail_view.name.clone_from(&detail_name);
    detail_view.grouping = DiagramGrouping::None;
    detail_view.focus = detail
        .entities
        .ordered()
        .map(|(id, _)| id.clone())
        .collect();
    let mut fields = vec![
        ("grouping".into(), view.grouping.as_str().into()),
        ("detailView".into(), selection_iri(&detail_view)),
    ];
    for (member, ()) in group.members.ordered() {
        fields.push(("member".into(), member.clone()));
    }
    summary.entity(group_iri(view, key), key, key, fields)?;
    Ok((detail_name, detail))
}
