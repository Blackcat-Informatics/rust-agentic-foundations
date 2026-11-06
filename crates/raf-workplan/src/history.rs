// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native read-only history for authored documents and bound source.
//!
//! Document tracing follows renames and meaningful merge resolutions. Source
//! spans use native line projection; module bindings additionally select their
//! own source files. Removed bindings retain the substring-count heuristic,
//! including its first-parent relationship rather than semantic removal proof.
//!
//! Missing local objects, shallow repositories, invalid spans and exhausted
//! bounds refuse. No Git executable, helper, transport, hook or filter runs,
//! and no working file, index, reference or repository configuration is changed.

pub mod render;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use raf_okf::DocType;
use raf_source::{ResolveError, SymbolKind, Workspace};

use crate::digest;
use crate::git::{self, AuthorDate, GitError, LogEntry};
use crate::graph::BindingKind;
use crate::identifiers::{Commit, Date, Identifier, LineRange, PatternError, TreePath};
use crate::lookup::write_findings;
use crate::{Finding, Workplan};

/// The kinds of id the history answers for, in the order they are named.
pub const HISTORY_KINDS: [DocType; 6] = [
    DocType::Capability,
    DocType::Change,
    DocType::Architecture,
    DocType::Evidence,
    DocType::Assessment,
    DocType::Review,
];

/// What a history is limited to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct HistoryOptions {
    since: Option<Date>,
}

impl HistoryOptions {
    /// The whole history.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Only the commits whose author date is at or after 00:00:00 UTC of
    /// `since`.
    #[must_use]
    pub fn with_since(mut self, since: Date) -> Self {
        self.since = Some(since);
        self
    }

    /// The `--since` date, when one is given.
    #[must_use]
    pub fn since(&self) -> Option<&Date> {
        self.since.as_ref()
    }
}

/// Where a commit of a history comes from.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Source {
    /// The log of the document.
    Document,
    /// The log of the lines of a bound symbol, or of a file of a bound
    /// module: the binding's path as the document writes it.
    Symbol(String),
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Document => f.write_str("document"),
            Self::Symbol(path) => f.write_str(path),
        }
    }
}

/// One commit of a history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    commit: Commit,
    date: AuthorDate,
    subject: String,
    sources: Vec<Source>,
    paths: Vec<String>,
}

impl Entry {
    /// The commit.
    #[must_use]
    pub fn commit(&self) -> &Commit {
        &self.commit
    }

    /// The author date, as `git` printed it.
    #[must_use]
    pub fn date(&self) -> &AuthorDate {
        &self.date
    }

    /// The subject: data, never interpreted.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// The sources that list the commit: the document first, then the bound
    /// symbols by path.
    #[must_use]
    pub fn sources(&self) -> &[Source] {
        &self.sources
    }

    /// The paths the commit touched, sorted: for the document and for a file
    /// of a bound module, as the document log names them (the name the file
    /// had then, from the top of the work tree); for a symbol, the file of
    /// its span.
    #[must_use]
    pub fn paths(&self) -> &[String] {
        &self.paths
    }
}

/// What the lookup of a binding that no longer resolves found.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Removal {
    /// The newest commit that changed how often the identifier occurs in the
    /// crate, reported as the commit that removed the symbol, and its parent,
    /// reported as the last commit in which it resolved.
    Found {
        /// The commit reported as removing it.
        commit: Commit,
        /// Its author date.
        date: AuthorDate,
        /// Its subject: data, never interpreted.
        subject: String,
        /// The last commit in which it resolved; `None` when the commit is the
        /// first of the history and has no parent.
        last_resolved: Option<Commit>,
    },
    /// No commit changed how often the identifier occurs in the crate.
    NoCommit,
    /// No package of the workspace provides the crate the path begins with,
    /// so there is no directory to ask about and no process ran.
    NoCrate,
}

/// A `code` binding of the capability that no longer resolves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovedBinding {
    binding: String,
    identifier: String,
    dirs: Vec<String>,
    removal: Removal,
}

impl RemovedBinding {
    /// The binding's path as the document writes it.
    #[must_use]
    pub fn binding(&self) -> &str {
        &self.binding
    }

    /// The identifier that was looked for: the last segment of the path.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// The directories of the crate that were searched, relative to the
    /// workspace root (`.` for the root itself); empty for
    /// [`Removal::NoCrate`].
    #[must_use]
    pub fn dirs(&self) -> &[String] {
        &self.dirs
    }

    /// What the lookup found.
    #[must_use]
    pub fn removal(&self) -> &Removal {
        &self.removal
    }
}

/// The history of one document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History {
    id: String,
    path: String,
    entries: Vec<Entry>,
    removed: Vec<RemovedBinding>,
}

impl History {
    /// The id asked about.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The path of its document, relative to the workspace root.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The commits, newest first, each once.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The `code` bindings that no longer resolve, by path.
    #[must_use]
    pub fn removed(&self) -> &[RemovedBinding] {
        &self.removed
    }
}

/// Why a history has no answer.
#[derive(Debug)]
#[non_exhaustive]
pub enum HistoryError {
    /// The id is not of a kind the history answers for.
    UnknownKind {
        /// The id, as given.
        id: String,
    },
    /// No document or record has the id.
    Unknown {
        /// The id.
        id: String,
        /// The ids of that kind that loaded, sorted.
        known: Vec<String>,
    },
    /// A file has the id and did not load.
    NotLoaded {
        /// The id.
        id: String,
        /// The findings about the file.
        findings: Vec<Finding>,
    },
    /// A `code` binding of the capability resolves to no single symbol, for a
    /// reason other than that nothing has its path.
    Binding {
        /// The binding's path.
        path: String,
        /// Why it names no single symbol.
        error: ResolveError,
    },
    /// A value the history would give `git` does not match its pattern: a
    /// path of the source index or of the loaded documents, or the last
    /// segment of a binding's path.
    Pattern {
        /// What the value is.
        what: String,
        /// The refusal.
        error: PatternError,
    },
    /// The source index gives a definition of a bound symbol no lines.
    Span {
        /// The binding's path.
        path: String,
        /// The file of the definition.
        file: String,
    },
    /// The repository is a shallow clone.
    Shallow,
    /// `git log -L` failed for the span of a bound symbol, as it does for a
    /// file that is not committed or whose copy in `HEAD` ends before the line
    /// the symbol starts at in the working copy. The message says that
    /// `git log -L` could not read the lines from `HEAD`, quoting what `git`
    /// said, and names committing the file as one way to read them, not as the
    /// cause.
    SymbolLog {
        /// The binding's path.
        binding: String,
        /// The file of the span.
        file: String,
        /// What `git` said.
        error: Box<GitError>,
    },
    /// `git log --follow` lists no commit for a file of a bound module: the
    /// file is in the working tree, and no commit holds it.
    Uncommitted {
        /// The binding's path.
        binding: String,
        /// The file, relative to the workspace root.
        file: String,
    },
    /// `git` could not answer.
    Git(GitError),
}

impl fmt::Display for HistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownKind { id } => {
                let kinds: Vec<&str> = HISTORY_KINDS.iter().map(|kind| kind.id_prefix()).collect();
                write!(
                    f,
                    "`{}` is not an id the history answers for; give a {} id",
                    id.escape_debug(),
                    kinds.join(", ")
                )
            }
            Self::Unknown { id, known } => {
                write!(f, "no loaded document has the id `{}`; ", id.escape_debug())?;
                if known.is_empty() {
                    write!(f, "none of its kind loaded")
                } else {
                    write!(f, "the ids of its kind are {}", known.join(", "))
                }
            }
            Self::NotLoaded { id, findings } => {
                write!(
                    f,
                    "the document of `{id}` did not load, so its history is not read; fix its findings first"
                )?;
                write_findings(f, findings)
            }
            Self::Binding { path, error } => write!(
                f,
                "the code binding `{path}` names no single symbol ({error}), so it has no history to read; fix the binding"
            ),
            Self::Pattern { what, error } => write!(f, "{what} cannot be given to `git`: {error}"),
            Self::Span { path, file } => write!(
                f,
                "the source index gives the definition of `{path}` in `{file}` no lines, so there is no range to read the history of"
            ),
            Self::Shallow => write!(
                f,
                "the repository is a shallow clone: the commits before its shallow boundary are missing, so a history would be partial; fetch them with `git fetch --unshallow`"
            ),
            Self::SymbolLog {
                binding,
                file,
                error,
            } => write!(
                f,
                "`git log -L` could not read the lines of the bound symbol `{binding}` in `{file}` from `HEAD`: {error}; when `{file}` is not committed, or its copy in `HEAD` ends before the line the symbol starts at in the working copy, one way to read them is to commit `{file}` with the symbol at the lines it has now and ask again"
            ),
            Self::Uncommitted { binding, file } => write!(
                f,
                "the file `{file}` of the bound module `{binding}` has no commit: `git log --follow` lists none for it, so its history would be reported as empty; commit `{file}` and ask again"
            ),
            Self::Git(error) => write!(f, "{error}"),
        }
    }
}

impl Error for HistoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Pattern { error, .. } => Some(error),
            Self::SymbolLog { error, .. } => Some(error.as_ref()),
            Self::Git(error) => Some(error),
            _ => None,
        }
    }
}

impl From<GitError> for HistoryError {
    fn from(error: GitError) -> Self {
        Self::Git(error)
    }
}

/// A native line-history selection for one bound symbol definition.
struct SymbolQuery {
    binding: String,
    lines: LineRange,
    file: TreePath,
}

/// A native rename-following selection for one bound module file.
struct FileQuery {
    binding: String,
    file: TreePath,
}

/// A pickaxe lookup to run for a binding that no longer resolves.
struct RemovalQuery {
    binding: String,
    identifier: Identifier,
    dirs: Vec<TreePath>,
}

/// Every query of a history, built before the first one runs.
struct Plan {
    id: String,
    document: TreePath,
    symbols: Vec<SymbolQuery>,
    files: Vec<FileQuery>,
    removals: Vec<RemovalQuery>,
}

/// The value, or the pattern refusal with what it is.
fn checked<T>(
    what: impl FnOnce() -> String,
    built: Result<T, PatternError>,
) -> Result<T, HistoryError> {
    built.map_err(|error| HistoryError::Pattern {
        what: what(),
        error,
    })
}

/// The identifier a pickaxe looks for: the last segment of the path, without
/// a generic argument list or a raw-identifier prefix.
fn last_segment(path: &str) -> &str {
    let last = path.rsplit("::").next().unwrap_or(path);
    let last = last.split('<').next().unwrap_or(last);
    last.strip_prefix("r#").unwrap_or(last)
}

/// The directories of the packages whose targets put their items under the
/// first segment of `path`, sorted and each once.
fn crate_dirs(source: &Workspace, path: &str) -> Result<Vec<TreePath>, HistoryError> {
    let first = path.split("::").next().unwrap_or(path);
    let packages: BTreeSet<&str> = source
        .targets()
        .filter(|target| target.crate_path.split("::").next() == Some(first))
        .map(|target| target.package.as_str())
        .collect();
    let mut dirs = Vec::new();
    for package in source.packages() {
        if packages.contains(package.name.as_str()) {
            let dir = if package.dir.is_root() {
                TreePath::top()
            } else {
                checked(
                    || format!("the directory of the package `{}`", package.name),
                    TreePath::new(package.dir.as_str()),
                )?
            };
            dirs.push(dir);
        }
    }
    dirs.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    dirs.dedup();
    Ok(dirs)
}

/// Merge the log of one source into the entries by commit.
fn merge(
    into: &mut BTreeMap<String, Entry>,
    log: Vec<LogEntry>,
    source: &Source,
    file: Option<&TreePath>,
) {
    for found in log {
        let entry = into
            .entry(found.commit().as_str().to_string())
            .or_insert_with(|| Entry {
                commit: found.commit().clone(),
                date: found.date().clone(),
                subject: found.subject().to_string(),
                sources: Vec::new(),
                paths: Vec::new(),
            });
        if !entry.sources.contains(source) {
            entry.sources.push(source.clone());
        }
        entry.paths.extend(found.paths().iter().cloned());
        entry
            .paths
            .extend(file.map(|file| file.as_str().to_string()));
    }
}

impl Workplan<'_> {
    /// Read native document and source history without launching a program.
    ///
    /// The repository is supplied explicitly. Source spans refer to their line
    /// positions at HEAD; an unavailable committed span refuses with its binding
    /// and filename. The optional day selects author dates after midnight UTC.
    ///
    /// ```no_run
    /// use raf_workplan::{Gate, git::Repository, history::HistoryOptions};
    /// let opened = Gate::workspace(".").open()?;
    /// let repository = Repository::open(".")?;
    /// let history = opened.workplan().history("cap:api", &HistoryOptions::new(), &repository)?;
    /// # let _ = history;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Refuses unknown or unloaded documents, ambiguous source bindings,
    /// missing objects, shallow history, invalid spans and native size limits.
    pub fn history(
        &self,
        id: &str,
        options: &HistoryOptions,
        repository: &git::Repository,
    ) -> Result<History, HistoryError> {
        let plan = self.history_plan(id)?;
        if git::is_shallow(repository)? {
            return Err(HistoryError::Shallow);
        }
        let mut entries = BTreeMap::new();
        let log = git::document_log(repository, &plan.document)?;
        merge(&mut entries, log, &Source::Document, None);
        for query in &plan.symbols {
            let log =
                git::symbol_log(repository, query.lines, &query.file).map_err(
                    |error| match error {
                        GitError::Native(_) | GitError::Answer { .. } => HistoryError::SymbolLog {
                            binding: query.binding.clone(),
                            file: query.file.as_str().to_string(),
                            error: Box::new(error),
                        },
                        other => HistoryError::Git(other),
                    },
                )?;
            merge(
                &mut entries,
                log,
                &Source::Symbol(query.binding.clone()),
                Some(&query.file),
            );
        }
        for query in &plan.files {
            let log = git::document_log(repository, &query.file)?;
            if log.is_empty() {
                return Err(HistoryError::Uncommitted {
                    binding: query.binding.clone(),
                    file: query.file.as_str().to_string(),
                });
            }
            merge(
                &mut entries,
                log,
                &Source::Symbol(query.binding.clone()),
                None,
            );
        }
        let mut removed = Vec::new();
        for query in &plan.removals {
            let removal = if query.dirs.is_empty() {
                Removal::NoCrate
            } else {
                match git::last_change(repository, &query.identifier, &query.dirs)? {
                    None => Removal::NoCommit,
                    Some(found) => Removal::Found {
                        last_resolved: found.parent().cloned(),
                        commit: found.entry().commit().clone(),
                        date: found.entry().date().clone(),
                        subject: found.entry().subject().to_string(),
                    },
                }
            };
            removed.push(RemovedBinding {
                binding: query.binding.clone(),
                identifier: query.identifier.as_str().to_string(),
                dirs: query
                    .dirs
                    .iter()
                    .map(|dir| dir.as_str().to_string())
                    .collect(),
                removal,
            });
        }
        let mut entries: Vec<Entry> = entries.into_values().collect();
        if let Some(since) = options.since() {
            // Git's `--since` reads the committer date; the filter is by the
            // author date each entry shows, so it is applied here.
            let start = git::start_of_day(since);
            entries.retain(|entry| entry.date.seconds() >= start);
        }
        for entry in &mut entries {
            entry.sources.sort();
            entry.paths.sort();
            entry.paths.dedup();
        }
        entries.sort_by(|a, b| {
            b.date
                .seconds()
                .cmp(&a.date.seconds())
                .then_with(|| a.commit.as_str().cmp(b.commit.as_str()))
        });
        Ok(History {
            id: plan.id,
            path: plan.document.as_str().to_string(),
            entries,
            removed,
        })
    }

    /// The document `id` names and every query of its history, or why there
    /// is none. No process starts here.
    fn history_plan(&self, id: &str) -> Result<Plan, HistoryError> {
        let Some(kind) = HISTORY_KINDS
            .into_iter()
            .find(|kind| id.starts_with(kind.id_prefix()))
        else {
            return Err(HistoryError::UnknownKind { id: id.to_string() });
        };
        let path = self.document_path(kind, id)?;
        let document = checked(|| format!("the path of `{id}`"), TreePath::new(&path))?;
        let mut plan = Plan {
            id: id.to_string(),
            document,
            symbols: Vec::new(),
            files: Vec::new(),
            removals: Vec::new(),
        };
        if kind != DocType::Capability {
            return Ok(plan);
        }
        let source = self.source();
        let mut bindings: Vec<_> = self
            .graph()
            .bindings_of(id)
            .filter(|binding| binding.kind() == BindingKind::Code)
            .collect();
        bindings.sort_by(|a, b| a.path().cmp(b.path()));
        for binding in bindings {
            match binding.resolution() {
                Ok(sites) => {
                    let mut seen = BTreeSet::new();
                    // The files of a bound module, sorted and each once, as
                    // its digest orders them.
                    let mut files = BTreeSet::new();
                    for item in sites.iter().filter_map(|site| source.item(*site)) {
                        let (file, start) = item.source_key();
                        if !seen.insert((file.as_str().to_string(), start)) {
                            continue;
                        }
                        let file = checked(
                            || format!("the file of the binding `{}`", binding.path()),
                            TreePath::new(item.file.as_str()),
                        )?;
                        let last = item.span.lines.end.saturating_sub(1);
                        let lines =
                            LineRange::new(item.span.lines.start, last).ok_or_else(|| {
                                HistoryError::Span {
                                    path: binding.path().to_string(),
                                    file: item.file.as_str().to_string(),
                                }
                            })?;
                        plan.symbols.push(SymbolQuery {
                            binding: binding.path().to_string(),
                            lines,
                            file,
                        });
                        if item.kind == SymbolKind::Mod {
                            files.extend(
                                digest::module_files(source, item)
                                    .into_iter()
                                    .map(|(file, _)| file),
                            );
                        }
                    }
                    for file in files {
                        plan.files.push(FileQuery {
                            binding: binding.path().to_string(),
                            file: checked(
                                || format!("the file of the binding `{}`", binding.path()),
                                TreePath::new(file),
                            )?,
                        });
                    }
                }
                Err(ResolveError::NotFound(_)) => {
                    let identifier = checked(
                        || format!("the last segment of the binding `{}`", binding.path()),
                        Identifier::new(last_segment(binding.path())),
                    )?;
                    plan.removals.push(RemovalQuery {
                        binding: binding.path().to_string(),
                        identifier,
                        dirs: crate_dirs(source, binding.path())?,
                    });
                }
                Err(error) => {
                    return Err(HistoryError::Binding {
                        path: binding.path().to_string(),
                        error: error.clone(),
                    });
                }
            }
        }
        Ok(plan)
    }

    /// The path of the loaded document or record `id` of `kind`.
    fn document_path(&self, kind: DocType, id: &str) -> Result<String, HistoryError> {
        let loaded: Vec<(&str, &str)> = match kind {
            DocType::Capability => self
                .capabilities()
                .iter()
                .map(|doc| (doc.value().header.id.as_str(), doc.path()))
                .collect(),
            DocType::Change => self
                .changes()
                .iter()
                .map(|doc| (doc.value().header.id.as_str(), doc.path()))
                .collect(),
            DocType::Architecture => self
                .architecture()
                .iter()
                .map(|doc| (doc.value().header.id.as_str(), doc.path()))
                .collect(),
            DocType::Evidence => self
                .evidence()
                .iter()
                .map(|doc| (doc.value().header.id.as_str(), doc.path()))
                .collect(),
            DocType::Assessment => self
                .assessments()
                .iter()
                .map(|doc| (doc.value().header.id.as_str(), doc.path()))
                .collect(),
            DocType::Review => self
                .reviews()
                .iter()
                .map(|doc| (doc.value().header.id.as_str(), doc.path()))
                .collect(),
            DocType::Coverage | DocType::Debt => {
                return Err(HistoryError::UnknownKind { id: id.to_string() });
            }
        };
        if let Some((_, path)) = loaded.iter().find(|(loaded, _)| *loaded == id) {
            return Ok((*path).to_string());
        }
        if self.is_present(id) {
            let slug = &id[kind.id_prefix().len()..];
            return Err(HistoryError::NotLoaded {
                id: id.to_string(),
                findings: self.findings_of(&format!("{}/{slug}.md", kind.directory())),
            });
        }
        let mut known: Vec<String> = loaded.iter().map(|(id, _)| (*id).to_string()).collect();
        known.sort();
        Err(HistoryError::Unknown {
            id: id.to_string(),
            known,
        })
    }
}
