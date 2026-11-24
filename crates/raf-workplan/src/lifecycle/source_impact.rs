// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Changed definitions and their paths through the authored document graph.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use raf_okf::CapabilitySection;
use raf_source::{Item, ItemId, TargetKind};

use super::{Lifecycle, LifecycleError, Side};
use crate::Workplan;
use crate::assessment::{Naming, symbol_inputs};
use crate::digest::Digest;
use crate::git;
use crate::graph::{Binding, BindingKind, SectionId, code_span_paths, prose_of};
use crate::review::ReviewChangeKind;
use crate::strict::ClaimsReport;

/// The selected commit and its differences from the working tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceImpact {
    /// The full object id used for historical reads.
    pub base: String,
    /// Changed bound or public items, sorted by source path.
    pub symbols: Vec<SymbolImpact>,
}

/// A source difference and the bindings that explain its documented effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolImpact {
    /// A definition or public re-export path.
    pub path: String,
    /// The difference from the selected base.
    pub change: ReviewChangeKind,
    /// Whether the path is part of the public surface on the reported side.
    pub public: bool,
    /// Binding paths in their capability and role order.
    pub owners: Vec<BindingImpact>,
}

impl SymbolImpact {
    /// Whether this public path lacks a code binding on its reported side.
    #[must_use]
    pub fn is_unowned(&self) -> bool {
        self.public
            && !self.owners.iter().any(|binding| {
                binding.kind == BindingKind::Code
                    && binding.at_base == (self.change == ReviewChangeKind::Removed)
            })
    }
}

/// The authored binding through which an item reaches a capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingImpact {
    /// True for a binding read from the selected commit rather than the checkout.
    pub at_base: bool,
    /// Capability that declares this binding.
    pub capability: String,
    /// The binding list in the capability header.
    pub kind: BindingKind,
    /// The path written in that list.
    pub path: String,
    /// Sections supported by this binding and the evidence attached to each.
    pub sections: Vec<SectionImpact>,
}

/// Guarantees and evidence to inspect in a section reached through a binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionImpact {
    /// The affected capability section.
    pub section: SectionId,
    /// How the binding selects this section.
    pub reason: ImpactReason,
    /// Authored claim keys with the evidence for each association.
    pub guarantees: Vec<GuaranteeImpact>,
    /// Test paths named by its markers or its test bindings.
    pub tests: Vec<String>,
    /// Example paths that substantiate the section.
    pub examples: Vec<String>,
    /// Stored assessment identifiers with this section as subject.
    pub assessments: Vec<String>,
}

/// An authored claim and the reason its binding selects it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuaranteeImpact {
    /// The stable claim key within its capability and section.
    pub key: String,
    /// The association between the binding and the claim.
    pub reason: ImpactReason,
}

/// Evidence for associating a source binding with a section or guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactReason {
    /// The prose names the full binding or a multi-segment suffix.
    Named,
    /// The prose names the final segment alone.
    WeakName,
    /// With no strong name the assessment includes all capability code bindings.
    AllBindings,
    /// The graph associates this declared test or example with the section.
    SectionBinding,
    /// A claim marker explicitly names this test.
    TestedBy,
}

impl ImpactReason {
    /// Stable text and RDF spelling of the association.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Named => "named",
            Self::WeakName => "weak-name",
            Self::AllBindings => "all-code-bindings",
            Self::SectionBinding => "section-binding",
            Self::TestedBy => "tested-by",
        }
    }
}

impl From<Naming> for ImpactReason {
    fn from(naming: Naming) -> Self {
        match naming {
            Naming::Strong => Self::Named,
            Naming::Weak => Self::WeakName,
            Naming::Unnamed => Self::AllBindings,
        }
    }
}

impl fmt::Display for SourceImpact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "base {}", self.base)?;
        for symbol in &self.symbols {
            writeln!(f, "{} {}", symbol.change.name(), symbol.path)?;
            if symbol.is_unowned() {
                writeln!(f, "  public item without a documented owner")?;
            } else if symbol.public {
                writeln!(f, "  public item with a documented owner")?;
            } else {
                writeln!(f, "  private item")?;
            }
            for owner in &symbol.owners {
                writeln!(
                    f,
                    "  {} via {} {} ({})",
                    owner.capability,
                    owner.kind.key(),
                    owner.path,
                    if owner.at_base { "base" } else { "checkout" }
                )?;
                for section in &owner.sections {
                    writeln!(f, "    {} ({})", section.section, section.reason.name())?;
                    for guarantee in &section.guarantees {
                        writeln!(
                            f,
                            "      guarantee {} ({})",
                            guarantee.key,
                            guarantee.reason.name()
                        )?;
                    }
                    for (kind, names) in [
                        ("test", &section.tests),
                        ("example", &section.examples),
                        ("assessment", &section.assessments),
                    ] {
                        for name in names {
                            writeln!(f, "      {kind} {name}")?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Lifecycle {
    /// Compare the working tree with HEAD, a full reference or a full commit id.
    /// Removed items use the bindings and declarations at the selected commit.
    /// Changed items also retain bindings withdrawn from the working tree.
    ///
    /// # Errors
    /// Refuses unavailable Git data, unreadable source, invalid capability
    /// documents and public surfaces that exceed the source index's bounds.
    pub fn source_impact(&self, base: &str) -> Result<SourceImpact, LifecycleError> {
        let base = git::resolve_revision(&self.repository, base)?;
        let (tree, source) = self.open_revision(&base)?;
        let then = tree.workplan(&source)?;
        let now = self.opened.workplan();
        let old = Snapshot::read(&then, Side::Revision)?;
        let new = Snapshot::read(&now, Side::WorkingTree)?;
        Ok(SourceImpact {
            base: base.to_string(),
            symbols: new.compare(&old)?,
        })
    }
}

/// All written definitions let a documentation-only edit add or remove a
/// binding without inventing a source change. Only bound and public paths
/// enter the comparison.
#[derive(Default)]
struct Entry {
    definitions: BTreeSet<String>,
    external: BTreeSet<String>,
    public: bool,
    owners: BTreeSet<usize>,
}

impl Entry {
    fn selected(&self) -> bool {
        self.public || !self.owners.is_empty()
    }
}

struct Snapshot<'p, 's> {
    plan: &'p Workplan<'s>,
    side: Side,
    paths: BTreeMap<String, Entry>,
}

impl<'p, 's> Snapshot<'p, 's> {
    fn read(plan: &'p Workplan<'s>, side: Side) -> Result<Self, LifecycleError> {
        if let Some(finding) = &plan.authoring_fault {
            return Err(LifecycleError::Document {
                path: finding.file.clone(),
                side,
                reason: finding.to_string(),
            });
        }
        let capabilities: Vec<_> = plan
            .unloaded()
            .into_iter()
            .filter(|id| id.starts_with("cap:"))
            .collect();
        if !capabilities.is_empty() {
            return Err(LifecycleError::Unloaded {
                capabilities,
                findings: plan.findings().to_vec(),
            });
        }
        let source = plan.source();
        let owners = owners_by_item(plan, side)?;
        let mut paths = BTreeMap::<String, Entry>::new();
        let mut add = |path: &str, item: &Item, public: bool| {
            let entry = paths.entry(path.to_owned()).or_default();
            entry.definitions.insert(item.path.to_string());
            entry.public |= public;
            if let Some(indices) = owners.get(&item.id) {
                entry.owners.extend(indices);
            }
        };
        for item in source.items() {
            add(&item.path, item, false);
        }
        let mut options = raf_source::ExportOptions::default();
        options.include_hidden = true;
        let mut external = Vec::new();
        for id in source.target_ids() {
            let Some(target) = source.target(id) else {
                continue;
            };
            if !matches!(target.kind, TargetKind::Lib | TargetKind::Bin) {
                continue;
            }
            let exports = super::touched::exports_at(source, (id, target), side, &options)?;
            for export in exports.items {
                if let Some(item) = source.item(export.item) {
                    add(&export.path, item, true);
                }
            }
            for export in exports.reexports {
                if let Some(item) = export.item.and_then(|id| source.item(id)) {
                    add(&export.path, item, true);
                }
                external.push((export.path, export.written));
            }
        }
        for (path, written) in external {
            let entry = paths.entry(path).or_default();
            entry.public = true;
            entry.external.insert(written);
        }
        Ok(Self { plan, side, paths })
    }

    fn digests(&self, paths: &BTreeSet<&String>) -> Result<BTreeMap<&str, Digest>, LifecycleError> {
        let mut digests = BTreeMap::new();
        for definition in paths
            .iter()
            .filter_map(|path| self.paths.get(*path))
            .flat_map(|e| &e.definitions)
        {
            if let std::collections::btree_map::Entry::Vacant(slot) =
                digests.entry(definition.as_str())
            {
                let digest =
                    Digest::review_symbol(self.plan.source(), definition).map_err(|error| {
                        LifecycleError::Document {
                            path: definition.clone(),
                            side: self.side,
                            reason: error.to_string(),
                        }
                    })?;
                slot.insert(digest);
            }
        }
        Ok(digests)
    }

    fn compare(&self, old: &Snapshot<'_, '_>) -> Result<Vec<SymbolImpact>, LifecycleError> {
        let paths: BTreeSet<_> = self
            .paths
            .iter()
            .chain(&old.paths)
            .filter(|(_, entry)| entry.selected())
            .map(|(path, _)| path)
            .collect();
        let before = old.digests(&paths)?;
        let after = self.digests(&paths)?;
        let old_claims = old.plan.claims_in_states(|_| true);
        let new_claims = self.plan.claims_in_states(|_| true);
        let mut result = Vec::new();
        for path in paths {
            let (entry, plan, claims, change) = match (old.paths.get(path), self.paths.get(path)) {
                (Some(a), Some(b))
                    if a.public == b.public
                        && a.external == b.external
                        && a.definitions == b.definitions
                        && b.definitions
                            .iter()
                            .all(|p| before[p.as_str()] == after[p.as_str()]) =>
                {
                    continue;
                }
                (Some(a), None) => (a, old.plan, &old_claims, ReviewChangeKind::Removed),
                (None, Some(b)) => (b, self.plan, &new_claims, ReviewChangeKind::Added),
                (Some(a), Some(b)) if a.public && !b.public && b.owners.is_empty() => {
                    (a, old.plan, &old_claims, ReviewChangeKind::Removed)
                }
                (Some(_), Some(b)) => (b, self.plan, &new_claims, ReviewChangeKind::Changed),
                (None, None) => unreachable!("INVARIANT: path came from either snapshot"),
            };
            let bindings = plan.graph().bindings();
            let at_base = change == ReviewChangeKind::Removed;
            let mut owners: Vec<_> = entry
                .owners
                .iter()
                .map(|&i| binding_impact(plan, &bindings[i], claims, at_base))
                .collect();
            if change == ReviewChangeKind::Changed
                && let Some(previous) = old.paths.get(path)
            {
                for &i in &previous.owners {
                    let old_binding = &old.plan.graph().bindings()[i];
                    if !owners.iter().any(|owner| {
                        owner.capability == old_binding.capability()
                            && owner.path == old_binding.path()
                            && owner.kind == old_binding.kind()
                    }) {
                        owners.push(binding_impact(old.plan, old_binding, &old_claims, true));
                    }
                }
            }
            result.push(SymbolImpact {
                path: path.clone(),
                change,
                public: entry.public,
                owners,
            });
        }
        Ok(result)
    }
}

/// Resolve each binding once. Code ownership extends below a bound item
/// except at a crate root, as in the public coverage report.
fn owners_by_item(
    plan: &Workplan<'_>,
    side: Side,
) -> Result<BTreeMap<ItemId, BTreeSet<usize>>, LifecycleError> {
    let source = plan.source();
    let mut owners = BTreeMap::<ItemId, BTreeSet<usize>>::new();
    for (index, binding) in plan.graph().bindings().iter().enumerate() {
        if let Err(
            error @ (raf_source::ResolveError::Budget { .. }
            | raf_source::ResolveError::Storage { .. }),
        ) = binding.resolution()
        {
            return Err(LifecycleError::Document {
                path: binding.path().to_owned(),
                side,
                reason: error.to_string(),
            });
        }
        for &id in binding.items() {
            let Some(item) = source.item(id) else {
                continue;
            };
            let mut record = |item: &Item| {
                for id in std::iter::once(&item.id).chain(source.same_source(item)) {
                    owners.entry(*id).or_default().insert(index);
                }
            };
            record(item);
            if binding.kind() == BindingKind::Code
                && crate::coverage::below_bound(source, id).is_some()
            {
                for descendant in source.items_under(&item.path) {
                    record(descendant);
                }
            }
        }
    }
    Ok(owners)
}

#[expect(
    clippy::expect_used,
    reason = "the graph's bindings are built from the loaded capabilities of this same Workplan"
)]
fn binding_impact(
    plan: &Workplan<'_>,
    binding: &Binding,
    claims: &ClaimsReport,
    at_base: bool,
) -> BindingImpact {
    let graph = plan.graph();
    let id = binding.capability();
    let cap = plan
        .capabilities()
        .iter()
        .find(|c| c.value().header.id == id)
        .expect("INVARIANT: bindings originate in loaded capability documents");
    let mut sections = Vec::new();
    for section in CapabilitySection::ALL {
        let Some(authored) = cap
            .body()
            .sections()
            .iter()
            .find(|s| s.slug() == section.slug())
        else {
            continue;
        };
        let prose = prose_of(cap.body(), authored).unwrap_or_default();
        let cites = |text: &str| {
            let named = code_span_paths(text);
            let paths = cap.value().bindings.code.iter().map(String::as_str);
            symbol_inputs(&named, paths)
                .into_iter()
                .find_map(|(p, naming)| (p == binding.path()).then(|| ImpactReason::from(naming)))
        };
        let reason = match binding.kind() {
            BindingKind::Code => cites(&prose),
            _ => binding
                .sections()
                .contains(&section)
                .then_some(ImpactReason::SectionBinding),
        };
        let Some(reason) = reason else { continue };
        let subject = SectionId::new(id, section);
        let mut tests = Vec::new();
        let mut examples = Vec::new();
        let mut guarantees = Vec::new();
        for claim in claims
            .claims()
            .iter()
            .filter(|c| c.capability() == id && c.section() == section)
        {
            let claim_reason = match binding.kind() {
                BindingKind::Code => {
                    let lines = claim.lines();
                    let text = prose
                        .lines()
                        .skip(lines.start.saturating_sub(authored.content().lines.start))
                        .take(lines.len())
                        .collect::<Vec<_>>()
                        .join("\n");
                    cites(&text)
                }
                BindingKind::Test | BindingKind::Example
                    if claim.tests().iter().any(|p| p == binding.path()) =>
                {
                    Some(ImpactReason::TestedBy)
                }
                BindingKind::Test => None,
                BindingKind::Example => Some(ImpactReason::SectionBinding),
            };
            let Some(claim_reason) = claim_reason else {
                continue;
            };
            guarantees.push(GuaranteeImpact {
                key: claim.key(),
                reason: claim_reason,
            });
        }
        for bound in graph
            .bindings_of(id)
            .filter(|b| b.sections().contains(&section))
        {
            match bound.kind() {
                BindingKind::Test => tests.push(bound.path().to_owned()),
                BindingKind::Example => examples.push(bound.path().to_owned()),
                BindingKind::Code => {}
            }
        }
        sections.push(SectionImpact {
            reason,
            assessments: graph
                .assessments_of(&subject)
                .into_iter()
                .map(str::to_owned)
                .collect(),
            section: subject,
            tests,
            examples,
            guarantees,
        });
    }
    BindingImpact {
        at_base,
        capability: id.to_owned(),
        kind: binding.kind(),
        path: binding.path().to_owned(),
        sections,
    }
}
