// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One production gate over the ledger, source workspace, rules and exact debt.

mod context;
mod evaluate;
mod migrate;
mod owners;
mod patterns;
mod providers;
mod rules;
mod subjects;

pub use migrate::{
    MAX_INDEX_BYTES, Migration, MigrationInput, UnmappedRow, migrate_index, migrate_legacy,
};

use std::io::Write as _;
use std::path::{Component, Path, PathBuf};

use purrdf_jsonschema::ecma::MatchLimits;

use raf_source::{
    DiskTree, GitPackageRoots, LoadOptions, LockedCatalog, LockedManifest, RelPath,
    SourceAuthorityId, Tree, Workspace, discover_root,
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

/// A repository's validator for its one native extension node.
pub trait LedgerExtension: Sync {
    /// `None` declines ownership; `Some` claims the node and validates it.
    /// Exactly one registered extension must claim a nonempty extension.
    fn validate(&self, job: &str, node: raf_rdf::Node<'_>) -> Option<Result<(), String>>;
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
    dir: PathBuf,
    rules: Vec<Box<dyn Rule>>,
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
            dir: dir.as_ref().to_path_buf(),
            rules: Vec::new(),
            extensions: Vec::new(),
            matching_limits: MatchLimits::default(),
        }
    }

    /// Set the shared native step/state grant for names and ledger patterns.
    ///
    /// Token comparisons and pending pattern hits use this same grant. Source
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
        self.check_tree(&tree)
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
        let consumers = workspace
            .package_ids()
            .filter(|id| {
                workspace.package(*id).is_some_and(|package| {
                    package.manifest.authority() == SourceAuthorityId::FIRST_PARTY
                })
            })
            .collect();
        let census = Census::from_packages(&workspace, ledger.options(), &consumers)?;
        let report = evaluate::evaluate(
            &ledger,
            &workspace,
            census,
            &self.rules,
            &self.extensions,
            self.matching_limits,
        )?;
        check_debt(tree, report)
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
    ///
    /// # Errors
    /// Refuses all non-debt findings, growth into an existing current-version
    /// record, malformed existing data and any filesystem failure. The target
    /// is replaced only after a complete canonical record has been written.
    pub fn write_debt(&self) -> Result<DebtWrite, Error> {
        let root = self.root()?;
        let tree = DiskTree::new(&root).map_err(|source| Error::Tree {
            file: root.display().to_string(),
            source,
        })?;
        let report = self.evaluate_tree(&tree)?;
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

    fn evaluate_tree(&self, tree: &impl Tree) -> Result<GateReport, Error> {
        let ledger = read_ledger(tree)?;
        let mut options = LoadOptions::default();
        options.max_file_bytes = ledger.options().max_file_bytes;
        let roots: Vec<_> = ledger.roots().iter().map(String::as_str).collect();
        let workspace = Workspace::load_with(tree, &roots, &options)?;
        let census = Census::from_workspace(&workspace, ledger.options())?;
        evaluate::evaluate(
            &ledger,
            &workspace,
            census,
            &self.rules,
            &self.extensions,
            self.matching_limits,
        )
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
    if !ledger.reference().is_empty() {
        return Err(evaluate::policy(
            "reference packages require an executable reference scan; no first-party-only fallback is permitted",
        ));
    }
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
            "run cargo census debt --write or Gate::write_debt to lower the record"
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
