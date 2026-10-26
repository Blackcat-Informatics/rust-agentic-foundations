// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The dev-dependency gate: [`Gate::workspace`] loads the documents and the
//! source of a checkout from disk, runs [`Workplan::check`] and, in
//! [`Gate::assert_clean`], panics with every finding. [`Gate::open`] is the load
//! itself, for a caller that reads the documents instead of judging them.

use std::error::Error;
use std::fmt::{self, Write as _};
use std::fs;
use std::path::{Path, PathBuf};

use raf_okf::{DocType, Location};
use raf_source::{DiskTree, EntryKind, LoadError, RelPath, Tree, TreeError, Workspace};

use crate::exec::{Programs, Runner, SystemRunner};
use crate::record::RecordOptions;
use crate::staged::Staged;
use crate::{Finding, LinkContext, Workplan, git};

/// The name of the navigation file of the capability directory, which is not a
/// capability document.
const INDEX_NAME: &str = "index.md";

/// Why a [`Gate`] cannot give a verdict. A gate that cannot look at the
/// repository does not pass it: every case is an error and none is a skip.
#[derive(Debug)]
#[non_exhaustive]
pub enum GateError {
    /// The directory cannot be opened as a tree.
    Root {
        /// The directory the gate was given.
        dir: PathBuf,
        /// Why it cannot be opened.
        source: TreeError,
    },
    /// `docs/capabilities` does not exist, cannot be listed, or holds no
    /// capability document. A repository with nothing to check must not pass.
    NoCapabilities {
        /// The directory the gate was given.
        dir: PathBuf,
        /// What is wrong with `docs/capabilities`.
        reason: String,
    },
    /// The source workspace does not load.
    Source {
        /// The directory the gate was given.
        dir: PathBuf,
        /// The source roots the load was given.
        roots: Vec<String>,
        /// Why the load failed.
        source: Box<LoadError>,
    },
    /// The tracked paths of a git checkout cannot be listed.
    Tracked {
        /// The directory the gate was given.
        dir: PathBuf,
        /// Why.
        reason: String,
    },
}

impl fmt::Display for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Root { dir, source } => write!(
                f,
                "cannot open {} as a repository tree: {source}; pass the directory that holds `docs/` and `work/`",
                dir.display()
            ),
            Self::NoCapabilities { dir, reason } => write!(
                f,
                "{}/{} {reason}; a repository with no capability document has nothing to check, so the gate refuses it: create the capability documents, or pass the repository root",
                dir.display(),
                DocType::Capability.directory(),
            ),
            Self::Source { dir, roots, source } => write!(
                f,
                "the source workspace under {} (roots {roots:?}) does not load: {source}; name the directories that hold the packages with `Gate::source_roots`",
                dir.display()
            ),
            Self::Tracked { dir, reason } => write!(
                f,
                "the tracked paths of {} cannot be listed: {reason}; pass them with `Gate::tracked_paths`",
                dir.display()
            ),
        }
    }
}

impl Error for GateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Root { source, .. } => Some(source),
            Self::Source { source, .. } => Some(source),
            Self::NoCapabilities { .. } | Self::Tracked { .. } => None,
        }
    }
}

/// The checks of a repository on disk, for a test of the repository itself.
///
/// A repository of one package and one delivered capability, written to a
/// scratch directory, loads with complete bindings and prose. After synchronizing
/// the capability, its one finding is the unwritten catalogue; writing that
/// catalogue supplies the passing neighbor:
///
/// ```rust
/// use std::fs;
///
/// use raf_source::RelPath;
/// use raf_workplan::{Gate, SyncOptions, Workplan};
///
/// let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir())?;
/// let dir = scratch.path();
/// let sections: String = ["Purpose", "Contract", "Usage", "Examples", "Constraints", "Explanation", "Verification"]
///     .iter()
///     .map(|name| {
///         let detail = match *name {
///             "Contract" => "<!-- raf:tested-by demo::tests::it_runs -->\n",
///             "Usage" => "Call `demo::run` to exercise its runnable documentation example.\n",
///             "Constraints" => "<!-- raf:tested-by demo::tests::it_runs -->\n\n- This demonstration records no persistent application data.\n<!-- raf:tested-by demo::tests::it_runs -->\n",
///             "Verification" => "<!-- raf:generated:verification-matrix -->\n<!-- raf:generated:end -->\n",
///             _ => "",
///         };
///         let lead = if *name == "Contract" { "**Running.** " } else { "" };
///         format!("## {name}\n\n{lead}The demo function returns successfully without arguments or persistent changes; its executable checks exercise the same behavior through a small standalone library.\n{detail}\n")
///     })
///     .collect();
/// let capability = format!(
///     "---\ntype: Capability\nid: cap:demo\ntitle: \"Demo\"\ndescription: \"One line.\"\n\
///      state: delivered\n\
///      bindings:\n  code: [demo::run]\n  tests: [demo::tests::it_runs]\n  examples: [demo::run]\n\
///      depends: []\n---\n\n# Demo\n\n{sections}"
/// );
/// let files = [
///     ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n".to_string()),
///     ("src/lib.rs", "/// Runs without persistent changes.\n/// ```\n/// demo::run();\n/// ```\npub fn run() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_runs() {\n        super::run();\n    }\n}\n".to_string()),
///     ("docs/capabilities/demo.md", capability),
///     ("work/coverage/default.md", "---\ntype: Coverage\nid: coverage:default\nscope: [demo]\nexclusions: []\n---\n".to_string()),
/// ];
/// for (path, text) in files {
///     let file = dir.join(path);
///     fs::create_dir_all(file.parent().expect("a file has a directory"))?;
///     fs::write(file, text)?;
/// }
///
/// let gate = Gate::workspace(&dir).tracked_paths([]);
/// let opened = gate.open()?;
/// let synced = opened.workplan().sync(&SyncOptions::new("2026 Example", "CC-BY-4.0"))?;
/// for (path, text) in &synced.edits {
///     if path.as_str() == "docs/capabilities/demo.md" {
///         fs::write(dir.join(path.as_str()), text)?;
///     }
/// }
/// let findings = gate.try_check()?;
/// let kinds: Vec<&str> = findings.iter().map(|finding| finding.kind).collect();
/// assert_eq!(kinds, ["index-missing"]);
/// assert_eq!(findings[0].file, "docs/capabilities/index.md");
/// Workplan::write(&synced, &dir)?;
/// let tracked = ["docs/capabilities/demo.md", "docs/capabilities/index.md"]
///     .into_iter().map(RelPath::new).collect::<Result<Vec<_>, _>>()?;
/// let clean = Gate::workspace(&dir)
///     .tracked_paths(tracked)
///     .try_check()?;
/// assert!(clean.is_empty(), "{clean:?}");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// [`Gate::workspace`] names the directory. The gate reads `docs/capabilities`,
/// `docs/architecture` and `work/changes` below it, and the records of
/// `work/evidence`, `work/assessments`, `work/coverage` and `work/debt`, loads the source
/// [`Workspace`] with the roots `["."]` (change them with
/// [`Gate::source_roots`]), and runs [`Workplan::check`].
///
/// A repository with no `docs/capabilities` directory, or with a directory
/// that holds no capability document, is an error and not a clean verdict: a
/// gate pointed at the wrong directory must not pass. A missing `work/changes`
/// is the empty set of changes.
///
/// Links are judged against the tracked paths of the checkout. They are taken
/// from `git ls-files` when the directory is inside a git work tree. If `git`
/// cannot run, fails for another reason, is told by `GIT_DIR` or
/// `GIT_WORK_TREE` about a repository it cannot use, or cannot list the index,
/// the gate is an error ([`GateError::Tracked`]).
///
/// There is one fallback, taken only when `git` itself answers that the
/// directory is not inside a git work tree, as for a copy of the files without
/// `.git` such as the snapshot of a commit hook. Then every file under the
/// directory stands for a tracked path (see [`LinkContext::of_tree`]). That
/// weakens the rule that a link to an untracked file is broken, since a file
/// that is on disk and untracked then passes. [`Gate::tracked_paths`] replaces
/// both with a list the caller owns. A path that is a symlink on disk is a
/// tracked symlink, and a link to it is `link-symlink`.
///
/// Every opened workplan judges the complete coverage, prose, claims and
/// debt policy. There is no caller-selected policy floor.
#[derive(Debug, Clone)]
pub struct Gate {
    dir: PathBuf,
    roots: Vec<String>,
    tracked: Option<Vec<RelPath>>,
    programs: Option<Programs>,
    record: RecordOptions,
}

impl Gate {
    /// A gate over the checkout in `dir`: the directory that holds `docs/`
    /// and `work/`, usually the manifest directory of the package. A
    /// repository runs it from a test of its own; the block is that test file,
    /// so it is compiled and not run:
    ///
    /// ```rust,no_run
    /// // tests/workplan.rs
    /// #[test]
    /// fn workplan() {
    ///     raf_workplan::Gate::workspace(env!("CARGO_MANIFEST_DIR")).assert_clean();
    /// }
    /// ```
    #[must_use]
    pub fn workspace(dir: impl AsRef<Path>) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
            roots: vec![".".to_string()],
            tracked: None,
            programs: None,
            record: RecordOptions::new(),
        }
    }

    /// Open the debt record that [`Gate::write_debt`] creates with these SPDX
    /// comment lines, for a repository that checks the licence of every file.
    #[must_use]
    pub fn record_options(mut self, options: RecordOptions) -> Self {
        self.record = options;
        self
    }

    /// The SPDX lines of [`Gate::record_options`].
    pub(crate) fn record_spdx(&self) -> &RecordOptions {
        &self.record
    }

    /// Load the source workspace from these roots, relative to the directory,
    /// instead of `["."]`.
    #[must_use]
    pub fn source_roots(mut self, roots: &[&str]) -> Self {
        self.roots = roots.iter().map(|root| (*root).to_string()).collect();
        self
    }

    /// Judge links against exactly these tracked paths, instead of the ones
    /// `git ls-files` lists or the files of the directory.
    #[must_use]
    pub fn tracked_paths(mut self, paths: impl IntoIterator<Item = RelPath>) -> Self {
        self.tracked = Some(paths.into_iter().collect());
        self
    }

    /// Start the `git` of the tracked-path listing from these [`Programs`]
    /// instead of the `git` on `PATH`. The working directory of the programs is
    /// always the gate's directory, whatever `cwd` the value holds, since the
    /// listing is relative to it. Nothing is started when the caller gives the
    /// tracked paths with [`Gate::tracked_paths`].
    #[must_use]
    pub fn programs(mut self, programs: Programs) -> Self {
        self.programs = Some(programs);
        self
    }

    /// Open the repository: the directory as a tree, its source
    /// [`Workspace`], and the tracked paths the links are judged against. The
    /// documents are loaded from what is opened by [`Opened::workplan`].
    ///
    /// This is the load that [`Gate::try_check`] runs, with the same errors and
    /// the same fallback, taken only when `git` itself answers that the
    /// directory is not inside a git work tree.
    ///
    /// # Errors
    ///
    /// A [`GateError`] when the gate cannot look at the repository: the
    /// directory does not open, `docs/capabilities` is missing or holds no
    /// capability document, the source workspace does not load, or the
    /// tracked paths of a git checkout cannot be listed.
    pub fn open(&self) -> Result<Opened, GateError> {
        let mut programs = self
            .programs
            .clone()
            .unwrap_or_else(|| Programs::new(&self.dir));
        programs.cwd.clone_from(&self.dir);
        self.open_with_runner(&SystemRunner::new(programs))
    }

    /// Open with the caller's runner for every tracked-path query. Explicit
    /// tracked paths still bypass those queries; the runner owns its programs
    /// and must already use the gate's working directory.
    pub(crate) fn open_with_runner(&self, runner: &dyn Runner) -> Result<Opened, GateError> {
        let tree = DiskTree::new(&self.dir).map_err(|source| GateError::Root {
            dir: self.dir.clone(),
            source,
        })?;
        self.require_capabilities(&tree)?;
        let roots: Vec<&str> = self.roots.iter().map(String::as_str).collect();
        let workspace = Workspace::load(&tree, &roots).map_err(|source| GateError::Source {
            dir: self.dir.clone(),
            roots: self.roots.clone(),
            source: Box::new(source),
        })?;
        let tracked = self.tracked_list(runner)?;
        let symlinks = tracked
            .iter()
            .flatten()
            .filter(|path| self.is_symlink(path))
            .cloned()
            .collect();
        Ok(Opened {
            dir: self.dir.clone(),
            tree,
            workspace,
            tracked,
            symlinks,
        })
    }

    /// Every finding of the checks over the repository.
    ///
    /// # Errors
    ///
    /// A [`GateError`] when the gate cannot look at the repository: the
    /// directory does not open, `docs/capabilities` is missing or holds no
    /// capability document, the source workspace does not load, or the
    /// tracked paths of a git checkout cannot be listed.
    pub fn try_check(&self) -> Result<Vec<Finding>, GateError> {
        Ok(self.open()?.workplan().check())
    }

    /// Every finding of the checks over the repository.
    ///
    /// # Panics
    ///
    /// Panics with the [`GateError`] when the gate cannot look at the
    /// repository; see [`Gate::try_check`].
    #[must_use]
    pub fn check(&self) -> Vec<Finding> {
        match self.try_check() {
            Ok(findings) => findings,
            Err(error) => panic!("workplan gate: {error}"),
        }
    }

    /// Panic unless the repository has no finding. The message lists every
    /// finding as `kind file:line message (fix)`.
    ///
    /// # Panics
    ///
    /// Panics with every finding, or with the [`GateError`] when the gate
    /// cannot look at the repository.
    pub fn assert_clean(&self) {
        let findings = self.check();
        assert!(findings.is_empty(), "{}", report(&self.dir, &findings));
    }

    /// Require a `docs/capabilities` directory that holds a capability
    /// document.
    fn require_capabilities(&self, tree: &DiskTree) -> Result<(), GateError> {
        let missing = |reason: String| GateError::NoCapabilities {
            dir: self.dir.clone(),
            reason,
        };
        let directory = RelPath::new(DocType::Capability.directory())
            .map_err(|error| missing(format!("is not a path: {error}")))?;
        let entries = match tree.list(&directory) {
            Ok(entries) => entries,
            Err(TreeError::NotFound { .. }) => return Err(missing("does not exist".to_string())),
            Err(error) => return Err(missing(format!("cannot be listed: {error}"))),
        };
        let has_document = entries
            .iter()
            .any(|entry| entry.kind != EntryKind::Dir && is_document_name(&entry.name));
        if has_document {
            Ok(())
        } else {
            Err(missing("holds no capability document".to_string()))
        }
    }

    /// The tracked paths the links are judged against, or `None` when every
    /// file of the directory stands for one.
    fn tracked_list(&self, runner: &dyn Runner) -> Result<Option<Vec<RelPath>>, GateError> {
        if let Some(paths) = &self.tracked {
            return Ok(Some(paths.clone()));
        }
        git::tracked_paths(runner).map_err(|error| GateError::Tracked {
            dir: self.dir.clone(),
            reason: error.to_string(),
        })
    }

    /// Whether the path is a symlink on disk. A path that is not there is not
    /// a symlink: the link rules report a link to it as they do for any
    /// tracked path.
    fn is_symlink(&self, path: &RelPath) -> bool {
        fs::symlink_metadata(self.dir.join(path.as_str()))
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
    }
}

/// A repository opened by [`Gate::open`]: the directory as a [`DiskTree`], its
/// source [`Workspace`] and the tracked paths the links are judged against.
///
/// A [`Workplan`] borrows the source workspace, so the documents are not kept
/// beside it: [`Opened::workplan`] loads them from the tree, as they are on
/// disk at that call. A caller that rewrites the files loads again to see the
/// result. The links are judged against the tracked paths and tracked symlinks
/// the open found ([`Opened::links`]). Each [`Workplan`] it gives is judged
/// with the complete repository policy.
///
/// ```rust,no_run
/// use raf_workplan::Gate;
///
/// let opened = Gate::workspace(".").open()?;
/// let findings = opened.workplan().check();
/// println!("{} capabilities, {} findings", opened.workplan().capabilities().len(), findings.len());
/// # Ok::<(), raf_workplan::GateError>(())
/// ```
#[derive(Debug)]
pub struct Opened {
    dir: PathBuf,
    tree: DiskTree,
    workspace: Workspace,
    tracked: Option<Vec<RelPath>>,
    symlinks: Vec<RelPath>,
}

impl Opened {
    /// The directory the gate was given.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The directory as a tree.
    #[must_use]
    pub fn tree(&self) -> &DiskTree {
        &self.tree
    }

    /// The source workspace the documents bind to.
    #[must_use]
    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    /// The tracked paths the links are judged against, or `None` when every
    /// file of the directory stands for one (a directory that is not inside a
    /// git work tree, see [`Gate`]).
    #[must_use]
    pub fn tracked(&self) -> Option<&[RelPath]> {
        self.tracked.as_deref()
    }

    /// The oracle the links of the documents are judged against, over this
    /// tree.
    #[must_use]
    pub fn links(&self) -> LinkContext<'_> {
        match &self.tracked {
            Some(paths) => LinkContext::new(&self.tree, paths.iter().cloned())
                .with_symlinks(self.symlinks.iter().cloned()),
            None => LinkContext::of_tree(&self.tree),
        }
    }

    /// The documents of the repository as they are on disk now, related to the
    /// source workspace, using the complete repository policy.
    #[must_use]
    pub fn workplan(&self) -> Workplan<'_> {
        Workplan::load_with(&self.tree, &self.workspace, &self.links())
    }

    /// The documents as they will be once the file `hidden` is removed and the
    /// files of `replaced` hold their new text: the tree shows exactly that,
    /// and the links are judged against the tracked paths without the removed
    /// file, so a link to it is broken. Nothing is written. The same complete
    /// policy as [`Opened::workplan`] judges the resulting documents.
    pub(crate) fn workplan_after(
        &self,
        hidden: &RelPath,
        replaced: &[(RelPath, String)],
    ) -> Workplan<'_> {
        let tree = Staged::new(
            &self.tree,
            hidden,
            replaced.iter().map(|(path, text)| (path, text.as_str())),
        );
        let links = match &self.tracked {
            Some(paths) => {
                LinkContext::new(&tree, paths.iter().filter(|path| *path != hidden).cloned())
                    .with_symlinks(self.symlinks.iter().cloned())
            }
            None => LinkContext::of_tree(&tree),
        };
        Workplan::load_with(&tree, &self.workspace, &links)
    }
}

/// Whether a file name in the capability directory is a capability document:
/// Markdown other than the index.
#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "the profile defines a document file name by the lowercase suffix `.md`"
)]
fn is_document_name(name: &str) -> bool {
    name.ends_with(".md") && name != INDEX_NAME
}

/// The panic message of a gate with findings: one line for each, as
/// `kind file:line message (fix)`.
fn report(dir: &Path, findings: &[Finding]) -> String {
    let mut text = format!(
        "workplan gate: {} finding(s) in {}:\n",
        findings.len(),
        dir.display()
    );
    for finding in findings {
        let _ = writeln!(
            text,
            "  {} {} {} ({})",
            finding.kind,
            place(finding),
            finding.message,
            finding.fix
        );
    }
    text
}

/// `file:line` for a finding at a line, and the file with the key or the
/// section otherwise.
fn place(finding: &Finding) -> String {
    match &finding.location {
        Location::Line(line) => format!("{}:{line}", finding.file),
        Location::Key(key) => format!("{} (key `{key}`)", finding.file),
        Location::Section(name) => format!("{} (section `{name}`)", finding.file),
        _ => finding.file.clone(),
    }
}
