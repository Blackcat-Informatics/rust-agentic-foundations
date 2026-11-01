// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One production gate over the ledger, source workspace, rules and exact debt.

mod context;
mod evaluate;
mod migrate;
mod owners;
mod patterns;
mod providers;
mod reference;
mod rules;
mod subjects;

pub use migrate::{
    ForeignField, ForeignLedgerAdapter, MAX_INDEX_BYTES, Migration, MigrationInput, UnmappedRow,
    migrate_index, migrate_legacy, migrate_legacy_with_extensions,
};
pub use reference::ReferenceRoot;

use std::io::Write as _;
use std::path::{Component, Path, PathBuf};

use purrdf_jsonschema::ecma::MatchLimits;

use raf_source::{
    DiskTree, GitPackageRoots, LoadOptions, LockedCatalog, LockedManifest, ReferenceObservation,
    RelPath, SourceAuthorityId, Tree, Workspace, discover_root,
};

use crate::{
    Census, DebtRecord, Error, JobReport, Ledger, MAX_DEBT_BYTES, MAX_LEDGER_BYTES, NORMAL_FORM,
    Rule,
};

/// Source-item justification anchors carry an encoded canonical or alias path.
pub const SYMBOL_ANCHOR_PREFIX: &str = "urn:raf:code:";
/// The unified RDF ledger filename.
pub const LEDGER_FILE: &str = "helpers-ledger.ttl";
/// The exact recorded debt filename.
pub const DEBT_FILE: &str = "census-debt.ttl";

/// The exact record produced by the gate's one debt-writing path.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DebtWrite {
    /// The record path.
    pub path: PathBuf,
    /// The number of present unsanctioned groups.
    pub groups: usize,
    /// The number of unenforced jobs with positive copy counts.
    pub jobs: usize,
}

impl std::fmt::Display for DebtWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} groups, {} jobs with copies",
            self.path.display(),
            self.groups,
            self.jobs
        )
    }
}

/// A repository's validator for its uniquely owned native extension closure.
pub trait LedgerExtension: Sync {
    /// `None` declines ownership; `Some` claims the closure and validates it.
    /// Exactly one registered extension must claim a nonempty extension.
    /// The consumer validates every field and evidence meaning it accepts.
    fn validate(
        &self,
        job: &str,
        graph: crate::LedgerExtensionGraph<'_>,
    ) -> Option<Result<(), String>>;
}

/// The source census, jobs and non-debt findings from one complete evaluation.
#[derive(Debug)]
pub struct GateReport {
    census: Census,
    jobs: Vec<JobReport>,
    findings: Vec<String>,
    debt: DebtRecord,
}

impl GateReport {
    /// The collected workspace units and groups.
    #[must_use]
    pub const fn census(&self) -> &Census {
        &self.census
    }
    /// Extended job matches, with each matching unit counted once.
    #[must_use]
    pub fn jobs(&self) -> &[JobReport] {
        &self.jobs
    }
    /// Every finding, including debt changes when the gate checked a record.
    #[must_use]
    pub fn findings(&self) -> &[String] {
        &self.findings
    }
    /// Exact present debt; non-debt findings must be absent before recording it.
    #[must_use]
    pub const fn debt(&self) -> &DebtRecord {
        &self.debt
    }
    /// Whether all findings are absent.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}

/// The census builder used by repositories as a dev-dependency.
pub struct Gate {
    reference_roots: Vec<ReferenceRoot>,
    dir: PathBuf,
    rules: Vec<Box<dyn Rule>>,
    required_rules: Vec<String>,
    extensions: Vec<Box<dyn LedgerExtension>>,
    matching_limits: MatchLimits,
}

impl Gate {
    /// Discover the actual workspace root used by this gate's disk operations.
    ///
    /// # Errors
    /// Refuses unavailable paths, invalid manifests and bounded root discovery failures.
    pub fn root(&self) -> Result<PathBuf, Error> {
        disk_root(&self.dir)
    }
    /// Find the workspace containing this directory when the gate is run.
    #[must_use]
    pub fn workspace(dir: impl AsRef<Path>) -> Self {
        Self {
            reference_roots: Vec::new(),
            dir: dir.as_ref().to_path_buf(),
            rules: Vec::new(),
            required_rules: Vec::new(),
            extensions: Vec::new(),
            matching_limits: MatchLimits::default(),
        }
    }

    /// Supply explicit locally available reference repositories/directories.
    /// Path declarations inside the selected Tree need no extra locator.
    /// Nothing searches a shared package cache or acquires data from the network.
    #[must_use]
    pub fn reference_roots(mut self, roots: impl IntoIterator<Item = ReferenceRoot>) -> Self {
        self.reference_roots.extend(roots);
        self
    }

    /// Set the shared native step/state grant for witnesses, names and ledger patterns.
    ///
    /// Witness pairs, retained states, storage growth, token comparisons and pending
    /// pattern hits use this same grant. Source
    /// loading keeps its separate original limits; exhausted matching is an error.
    #[must_use]
    pub const fn matching_limits(mut self, limits: MatchLimits) -> Self {
        self.matching_limits = limits;
        self
    }

    /// Register executable repository rules and their admission fixtures.
    #[must_use]
    pub fn rules(mut self, rules: impl IntoIterator<Item = Box<dyn Rule>>) -> Self {
        self.rules.extend(rules);
        self
    }

    /// Require rules independently of ledger jobs.
    ///
    /// Their findings cannot be exempted by a home, variant or debt record.
    /// Each detector retains its documented source eligibility.
    /// Execution refuses unknown rules and detectors that need a job's home.
    #[must_use]
    pub fn require_rules(mut self, rules: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.required_rules
            .extend(rules.into_iter().map(Into::into));
        self
    }

    /// Register validators for repository extension nodes.
    #[must_use]
    pub fn extensions(
        mut self,
        extensions: impl IntoIterator<Item = Box<dyn LedgerExtension>>,
    ) -> Self {
        self.extensions.extend(extensions);
        self
    }

    /// Check the real workspace through bounded source-tree reads.
    ///
    /// # Errors
    /// Refuses root discovery, ledger/debt/schema failures, missing executable
    /// dependencies and rule admission or ownership failures.
    pub fn check(&self) -> Result<GateReport, Error> {
        let root = self.root()?;
        let tree = DiskTree::new(&root).map_err(|source| Error::Tree {
            file: root.display().to_string(),
            source,
        })?;
        check_debt(
            &tree,
            self.evaluate_reference_tree(&tree, Some(&root), &[])?,
        )
    }

    /// Check a caller-owned workspace-root tree through the production path.
    /// This is the same evaluation used by the disk gate, without filesystem
    /// discovery. It never writes the tree.
    ///
    /// # Errors
    /// The ledger, source, rules and debt failures of [`Self::check`].
    pub fn check_tree(&self, tree: &impl Tree) -> Result<GateReport, Error> {
        check_debt(tree, self.evaluate_tree(tree)?)
    }

    /// Evaluate caller-owned reference observations through the ordinary Gate.
    /// Each observation must match a reachable exact row in this Tree's lock file.
    ///
    /// # Errors
    /// Refuses incomplete acquisition and every source, policy, report or debt
    /// failure of [`Self::check_tree`]. Observations cannot authorize provider proof.
    pub fn check_tree_with_references(
        &self,
        tree: &impl Tree,
        references: &[ReferenceObservation],
    ) -> Result<GateReport, Error> {
        check_debt(tree, self.evaluate_reference_tree(tree, None, references)?)
    }

    /// Check the ledger's complete first-party roots with supplied locked sources.
    ///
    /// Loads one source workspace, reusing its exact package and definition IDs
    /// for Census and policy execution. Every first-party package discovered under
    /// the ledger roots must have a checked local manifest association. Sealed Git
    /// dependencies remain available for definition lookup and are excluded from
    /// first-party unit collection and registered-rule execution.
    ///
    /// # Errors
    /// Refuses missing or extra manifest associations, invalid locked evidence,
    /// source limits and every ledger, rule and debt failure of [`Self::check_tree`].
    pub fn check_locked(
        &self,
        tree: &impl Tree,
        catalog: &LockedCatalog,
        local: &[LockedManifest<'_>],
        sources: &[GitPackageRoots<'_>],
    ) -> Result<GateReport, Error> {
        check_debt(tree, self.evaluate_locked(tree, catalog, local, sources)?)
    }

    /// Assert that the actual workspace has no findings.
    ///
    /// # Panics
    /// Panics with all findings, or the actionable failure that prevented a
    /// verdict. Missing tools, files and executable rules never count as clean.
    pub fn assert_clean(&self) {
        match self.check() {
            Ok(report) if report.is_clean() => {}
            Ok(report) => panic!("census gate failed:\n{}", report.findings.join("\n")),
            Err(error) => panic!("census gate could not run: {error}"),
        }
    }

    /// Write exactly the present debt after every non-debt check succeeds.
    /// An existing record may only shrink. Creation and replacement of a record
    /// from a different normal form are the explicit adoption exceptions.
    /// This operation belongs in a repository binary, never a test.
    /// A nonblocking kernel lock on persistent `.census-debt.ttl.lock` covers
    /// evaluation through publication. The coordination file is not removed.
    ///
    /// # Errors
    /// Refuses all non-debt findings, growth into an existing current-version
    /// record, malformed existing data, a busy publication lock and any
    /// filesystem failure. The target is replaced only after a complete
    /// canonical record has been written.
    pub fn write_debt(&self) -> Result<DebtWrite, Error> {
        let root = self.root()?;
        self.write_debt_with(|tree| self.evaluate_reference_tree(tree, Some(&root), &[]))
    }

    /// Write present debt after complete configured locked-source evaluation.
    ///
    /// First-party inputs are read from this Gate's discovered disk root. The
    /// supplied catalog, manifest associations and sealed sources retain the
    /// same authority and configuration contract as [`Self::check_locked`].
    /// Rules, extensions and provider proofs run before the shared debt ratchet.
    /// The same persistent publication lock as [`Self::write_debt`] covers
    /// evaluation, prior-record comparison and publication.
    /// This explicit write belongs in a repository binary or xtask.
    ///
    /// # Errors
    /// Refuses invalid locked evidence, incomplete or failing policy, malformed
    /// prior debt, same-form growth, a busy publication lock and filesystem or
    /// storage failures. A refusal preserves prior debt; publication uses
    /// atomic replacement.
    pub fn write_debt_locked(
        &self,
        catalog: &LockedCatalog,
        local: &[LockedManifest<'_>],
        sources: &[GitPackageRoots<'_>],
    ) -> Result<DebtWrite, Error> {
        self.write_debt_with(|tree| self.evaluate_locked(tree, catalog, local, sources))
    }

    fn write_debt_with(
        &self,
        evaluate: impl FnOnce(&DiskTree) -> Result<GateReport, Error>,
    ) -> Result<DebtWrite, Error> {
        let root = self.root()?;
        let _publication = lock_publication(&root)?;
        let tree = DiskTree::new(&root).map_err(|source| Error::Tree {
            file: root.display().to_string(),
            source,
        })?;
        let report = evaluate(&tree)?;
        if !report.findings.is_empty() {
            return Err(Error::Findings(report.findings));
        }
        let previous = read_debt(&tree)?;
        if let Some(previous) = &previous
            && previous.normal_form() == NORMAL_FORM
        {
            let growth: Vec<_> = previous
                .compare(&report.debt)?
                .into_iter()
                .filter(crate::DebtChange::is_growth)
                .map(|change| format!("{DEBT_FILE}: growth cannot be recorded: {change:?}"))
                .collect();
            if !growth.is_empty() {
                return Err(Error::Findings(growth));
            }
        }
        let path = root.join(DEBT_FILE);
        if previous.as_ref() != Some(&report.debt) {
            replace_record(&root, &report.debt.to_turtle()?)?;
        }
        Ok(DebtWrite {
            path,
            groups: report.debt.groups().len(),
            jobs: report.debt.job_copies().len(),
        })
    }

    fn evaluate_locked(
        &self,
        tree: &impl Tree,
        catalog: &LockedCatalog,
        local: &[LockedManifest<'_>],
        sources: &[GitPackageRoots<'_>],
    ) -> Result<GateReport, Error> {
        let ledger = read_ledger(tree)?;
        let mut options = LoadOptions::default();
        options.max_file_bytes = ledger.options().max_file_bytes;
        let roots = ledger
            .roots()
            .iter()
            .map(|root| RelPath::new(root).map_err(|error| Error::Options(error.to_string())))
            .collect::<Result<Vec<_>, _>>()?;
        let workspace =
            Workspace::load_locked_sources_under(catalog, tree, &roots, local, sources, &options)?;
        let consumers: std::collections::BTreeSet<_> = workspace
            .package_ids()
            .filter(|id| {
                workspace.package(*id).is_some_and(|package| {
                    package.manifest.authority() == SourceAuthorityId::FIRST_PARTY
                })
            })
            .collect();
        let references = if ledger.reference().is_empty() {
            reference::References::empty(&workspace)
        } else {
            reference::References::prepare(&ledger, &workspace, catalog, &[], &[])?
        };
        let consumers = consumers
            .intersection(&references.consumers)
            .copied()
            .collect();
        let mut census = Census::from_packages(&workspace, ledger.options(), &consumers)?;
        let mut limits = self.matching_limits;
        references.attach(&mut census, &mut limits)?;
        evaluate::evaluate(
            &ledger,
            &workspace,
            census,
            rules::Selected {
                registered: &self.rules,
                required: &self.required_rules,
            },
            &self.extensions,
            limits,
            &references,
        )
    }

    fn evaluate_tree(&self, tree: &impl Tree) -> Result<GateReport, Error> {
        self.evaluate_reference_tree(tree, None, &[])
    }

    fn evaluate_reference_tree(
        &self,
        tree: &impl Tree,
        host: Option<&Path>,
        supplied: &[ReferenceObservation],
    ) -> Result<GateReport, Error> {
        let ledger = read_ledger(tree)?;
        let mut options = LoadOptions::default();
        options.max_file_bytes = ledger.options().max_file_bytes;
        let roots: Vec<_> = ledger.roots().iter().map(String::as_str).collect();
        let workspace = Workspace::load_with(tree, &roots, &options)?;
        let acquired = if ledger.reference().is_empty() {
            None
        } else {
            Some(reference::acquire::acquire(
                &ledger,
                &workspace,
                tree,
                host,
                &self.reference_roots,
                supplied,
            )?)
        };
        let mut observed = Vec::new();
        if let Some(acquired) = &acquired {
            observed.extend(acquired.references.iter());
        }
        observed.extend(supplied.iter());
        // The owned acquired observations and caller observations share one
        // borrowed view; neither can contribute configured provider bindings.
        let references = if let Some(acquired) = &acquired {
            reference::References::prepare_borrowed(
                &ledger,
                &workspace,
                &acquired.catalog,
                &observed,
                &acquired.owners,
            )?
        } else {
            reference::References::empty(&workspace)
        };
        let mut census =
            Census::from_packages(&workspace, ledger.options(), &references.consumers)?;
        let mut limits = self.matching_limits;
        references.attach(&mut census, &mut limits)?;
        evaluate::evaluate(
            &ledger,
            &workspace,
            census,
            rules::Selected {
                registered: &self.rules,
                required: &self.required_rules,
            },
            &self.extensions,
            limits,
            &references,
        )
    }
}

/// Keep one cooperating publisher's evaluation and record update indivisible.
fn lock_publication(root: &Path) -> Result<std::fs::File, Error> {
    let path = root.join(".census-debt.ttl.lock");
    let io = |source| Error::Io {
        file: path.display().to_string(),
        source,
    };
    let unexpected = || {
        Error::Findings(vec![format!(
            "{}: expected a regular coordination file; resolve the unexpected path before publishing debt",
            path.display()
        )])
    };
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) if !metadata.is_file() => return Err(unexpected()),
        Ok(_) => {}
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => return Err(io(source)),
    }
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .map_err(io)?;
    if !file.metadata().map_err(io)?.is_file()
        || !std::fs::symlink_metadata(&path).map_err(io)?.is_file()
    {
        return Err(unexpected());
    }
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(std::fs::TryLockError::WouldBlock) => Err(io(std::io::Error::new(
            std::io::ErrorKind::WouldBlock,
            "debt publication is busy; retry after the other writer ends",
        ))),
        Err(std::fs::TryLockError::Error(source)) => Err(io(source)),
    }
}

fn read_ledger(tree: &impl Tree) -> Result<Ledger, Error> {
    let path = RelPath::new(LEDGER_FILE).expect("literal ledger path");
    let text = tree
        .read(&path, MAX_LEDGER_BYTES as u64)
        .map_err(|source| Error::Tree {
            file: path.to_string(),
            source,
        })?;
    let ledger = Ledger::from_turtle(&text).map_err(|source| Error::Input {
        file: LEDGER_FILE.into(),
        source: Box::new(source),
    })?;
    Ok(ledger)
}

fn check_debt(tree: &impl Tree, mut report: GateReport) -> Result<GateReport, Error> {
    let previous = read_debt(tree)?.unwrap_or(DebtRecord::new(
        NORMAL_FORM,
        [],
        std::collections::BTreeMap::default(),
    )?);
    for change in previous.compare(&report.debt)? {
        let direction = if change.is_growth() {
            "growth cannot be recorded"
        } else {
            "run cargo census debt --write or the matching Gate::write_debt or Gate::write_debt_locked to lower the record"
        };
        report
            .findings
            .push(format!("{DEBT_FILE}: {change:?}; {direction}"));
    }
    Ok(report)
}

fn replace_record(root: &Path, text: &str) -> Result<(), Error> {
    let temporary = root.join(".census-debt.ttl.write");
    let target = root.join(DEBT_FILE);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|source| Error::Io {
            file: temporary.display().to_string(),
            source,
        })?;
    let written = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all());
    drop(file);
    let result = written.and_then(|()| std::fs::rename(&temporary, &target));
    if let Err(source) = result {
        if let Err(cleanup) = std::fs::remove_file(&temporary) {
            return Err(Error::Findings(vec![
                format!("{}: {source}", target.display()),
                format!(
                    "could not remove owned temporary record {}: {cleanup}",
                    temporary.display()
                ),
            ]));
        }
        return Err(Error::Io {
            file: target.display().to_string(),
            source,
        });
    }
    Ok(())
}

fn read_debt(tree: &impl Tree) -> Result<Option<DebtRecord>, Error> {
    let path = RelPath::new(DEBT_FILE).expect("literal debt path");
    if tree
        .entry(&path)
        .map_err(|source| Error::Tree {
            file: path.to_string(),
            source,
        })?
        .is_none()
    {
        return Ok(None);
    }
    let text = tree
        .read(&path, MAX_DEBT_BYTES as u64)
        .map_err(|source| Error::Tree {
            file: path.to_string(),
            source,
        })?;
    DebtRecord::from_turtle(&text)
        .map(Some)
        .map_err(|source| Error::Input {
            file: DEBT_FILE.into(),
            source: Box::new(source),
        })
}

fn disk_root(dir: &Path) -> Result<PathBuf, Error> {
    let canonical = std::fs::canonicalize(dir).map_err(|source| Error::Io {
        file: dir.display().to_string(),
        source,
    })?;
    let boundary = canonical
        .ancestors()
        .last()
        .expect("absolute path has a root");
    let tree = DiskTree::new(boundary).map_err(|source| Error::Tree {
        file: boundary.display().to_string(),
        source,
    })?;
    let relative = canonical.strip_prefix(boundary).expect("ancestor prefix");
    let components = relative
        .components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| evaluate::policy("workspace path is not UTF-8")),
            _ => Err(evaluate::policy(
                "workspace discovery requires normal path components",
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let relative = if components.is_empty() {
        ".".into()
    } else {
        components.join("/")
    };
    let start = RelPath::new(&relative).map_err(|error| evaluate::policy(&error.to_string()))?;
    Ok(boundary.join(discover_root(&tree, &start)?.to_string()))
}
