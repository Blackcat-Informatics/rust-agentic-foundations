// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The typed invocation: a program and arguments that can only be built from
//! fixed text and validated values.

use std::fmt;
use std::path::{Path, PathBuf};

use super::ViewContext;

use super::pattern::{
    Commit, DocFilter, ExampleName, Filter, Identifier, LineRange, Package, RefName, SourcePath,
    Target, TreePath,
};
use super::trusted::{TrustedArgv, TrustedWord, text_of};

/// Which program an invocation starts. The paths of `cargo` and `git` are
/// held by [`Programs`](super::Programs); the program of a
/// [`Program::Trusted`] invocation is the first word of its argv.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Program {
    /// `cargo`, from `$CARGO` or `PATH`.
    Cargo,
    /// `git`, from `PATH`.
    Git,
    /// The caller's own command, a [`TrustedArgv`] from the command line of
    /// the process (see [`Invocation::trusted`]).
    Trusted,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cargo => "cargo",
            Self::Git => "git",
            Self::Trusted => "the command of the command line",
        })
    }
}

/// Fixed text of a program's command line, such as `--frozen` or `--`.
///
/// The text is a compile-time constant of this crate. The constructor is
/// private to the crate, so no caller can put free text on a command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Literal(&'static str);

impl Literal {
    /// Fixed text.
    pub(crate) const fn new(text: &'static str) -> Self {
        Self(text)
    }

    /// The text.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// One argument of an [`Invocation`].
///
/// Text that is not a compile-time constant enters only through a variant that
/// holds a validated newtype, and each of those can be built only through its
/// pattern (see [`Pattern`](super::Pattern)). Document text therefore never
/// reaches a process as a free string, and the one place an argument is
/// rendered is [`Arg`]'s `Display`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Arg {
    /// A native host-protocol argument, constructed only inside this crate.
    Host(HostArgument),
    /// Fixed text of the program, such as `--frozen` or `--`.
    Literal(Literal),
    /// `--package=<pkg>`.
    Package(Package),
    /// `--lib`, `--bin=<bin>` or `--test=<test>`.
    Target(Target),
    /// `--example=<example>`.
    Example(ExampleName),
    /// A libtest filter, always placed after `--`.
    Filter(Filter),
    /// A rustdoc filter, always placed after `--`.
    DocFilter(DocFilter),
    /// `<ref>^{commit}`, the form `git rev-parse --verify --end-of-options`
    /// resolves to a commit id.
    RefCommit(RefName),
    /// `<commit>^{commit}`, verifying an explicit full object identifier.
    CommitExpression(Commit),
    /// A commit id.
    Commit(Commit),
    /// `<rev>:<path>` in one argument, so the path cannot be read as an
    /// option.
    RevPath {
        /// The commit the path is read at.
        rev: Commit,
        /// The repository-relative path.
        path: SourcePath,
    },
    /// A path of the workspace, always placed after `--`, so that a path that
    /// begins with `-` is a path.
    Path(TreePath),
    /// `-L<start>,<end>:<file>` in one argument, so the file cannot be read
    /// as an option.
    LineLog {
        /// The lines.
        lines: LineRange,
        /// The file.
        file: TreePath,
    },
    /// `-S<identifier>` in one argument: the pickaxe of `git log`.
    Pickaxe(Identifier),
    /// A word of the caller's own command, which only a [`TrustedArgv`] holds.
    Trusted(TrustedWord),
}

impl Arg {
    /// Fixed text.
    fn lit(text: &'static str) -> Self {
        Self::Literal(Literal::new(text))
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host(value) => f.write_str(&value.0),
            Self::Literal(text) => write!(f, "{text}"),
            Self::Package(package) => write!(f, "--package={package}"),
            Self::Target(target) => write!(f, "{target}"),
            Self::Example(example) => write!(f, "--example={example}"),
            Self::Filter(filter) => write!(f, "{filter}"),
            Self::DocFilter(filter) => write!(f, "{filter}"),
            Self::RefCommit(name) => write!(f, "{name}^{{commit}}"),
            Self::CommitExpression(commit) => write!(f, "{commit}^{{commit}}"),
            Self::Commit(commit) => write!(f, "{commit}"),
            Self::RevPath { rev, path } => write!(f, "{rev}:{path}"),
            Self::Path(path) => write!(f, "{path}"),
            Self::LineLog { lines, file } => write!(f, "-L{lines}:{file}"),
            Self::Pickaxe(identifier) => write!(f, "-S{identifier}"),
            Self::Trusted(word) => write!(f, "{word}"),
        }
    }
}

/// A program and its arguments, the only thing a [`Runner`](super::Runner)
/// accepts.
///
/// The constructors below are the argv tables of the capability documents, one
/// function for each row, and [`Invocation::trusted`], the caller's own command
/// from the command line of the process. There is no public way to build any
/// other invocation: the general constructor is private to the crate, and so
/// are the constructors of [`Literal`] and of a [`TrustedArgv`] from words, so a
/// caller cannot assemble an argv of its own.
///
/// ```compile_fail,E0624
/// use raf_workplan::exec::{Invocation, Program};
///
/// // `Invocation::new` is not public.
/// let _ = Invocation::new(Program::Git, Vec::new());
/// ```
///
/// ```compile_fail,E0624
/// use raf_workplan::exec::{Arg, Literal};
///
/// // Nor is `Literal::new`, so free text cannot become an argument.
/// let _ = Arg::Literal(Literal::new("--upload-pack=evil"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Invocation {
    program: Program,
    args: Vec<Arg>,
    view: Option<ViewContext>,
    directory: Option<PathBuf>,
    filesystem: Option<std::sync::Arc<super::view::FilesystemView>>,
    git: Option<super::view::GitContext>,
}

impl Invocation {
    /// An invocation of `program` with these arguments.
    #[must_use]
    pub(crate) fn new(program: Program, args: Vec<Arg>) -> Self {
        Self {
            program,
            args,
            view: None,
            directory: None,
            filesystem: None,
            git: None,
        }
    }

    /// Explicit prepared execution context, retained in bound evidence.
    #[must_use]
    pub fn view(&self) -> Option<&ViewContext> {
        self.view.as_ref()
    }

    pub(crate) fn in_view(mut self, view: &ViewContext) -> Self {
        self.view = Some(view.clone());
        self
    }

    pub(crate) fn in_filesystem(
        mut self,
        view: std::sync::Arc<super::view::FilesystemView>,
    ) -> Self {
        self.filesystem = Some(view);
        self
    }
    pub(crate) fn filesystem(&self) -> Option<std::sync::Arc<super::view::FilesystemView>> {
        self.filesystem.clone()
    }

    pub(crate) fn in_git(mut self, context: super::view::GitContext) -> Self {
        self.git = Some(context);
        self
    }

    pub(crate) fn git_context(&self) -> Option<&super::view::GitContext> {
        self.git.as_ref()
    }

    pub(crate) fn host(program: Program, args: &[String], directory: &Path) -> Self {
        Self {
            program,
            args: args
                .iter()
                .cloned()
                .map(|value| Arg::Host(HostArgument(value)))
                .collect(),
            view: None,
            directory: Some(directory.to_path_buf()),
            filesystem: None,
            git: None,
        }
    }

    pub(crate) fn at_directory(mut self, directory: &Path) -> Self {
        self.directory = Some(directory.to_path_buf());
        self
    }

    pub(crate) fn directory(&self) -> Option<&Path> {
        self.directory.as_deref()
    }

    /// The program.
    #[must_use]
    pub fn program(&self) -> Program {
        self.program
    }

    /// The arguments, in order.
    #[must_use]
    pub fn args(&self) -> &[Arg] {
        &self.args
    }

    /// The arguments as the process receives them, one string for each
    /// element of the argv vector and without the program. For a
    /// [`Program::Trusted`] invocation the first element is the program the
    /// caller named, and the rest are its arguments.
    #[must_use]
    pub fn argv(&self) -> Vec<String> {
        self.args.iter().map(ToString::to_string).collect()
    }

    /// The caller's own command: `<program> <arg>...`, exactly the words of
    /// `argv`. The runner starts the first word as the program, with no shell
    /// and with the environment of this process unchanged.
    #[must_use]
    pub fn trusted(argv: &TrustedArgv) -> Self {
        Self::new(
            Program::Trusted,
            argv.words().iter().cloned().map(Arg::Trusted).collect(),
        )
    }

    /// A cargo invocation of these literals, a package and the tail.
    fn cargo(args: Vec<Arg>) -> Self {
        Self::new(Program::Cargo, args)
    }

    fn git(args: Vec<Arg>) -> Self {
        Self::new(Program::Git, args)
    }

    pub(crate) fn git_head() -> Self {
        Self::git(vec![
            Arg::lit("rev-parse"),
            Arg::lit("--verify"),
            Arg::lit("--end-of-options"),
            Arg::lit("HEAD^{commit}"),
        ])
    }

    /// Observe the compiler configuration Cargo selects for a pinned target.
    #[must_use]
    pub fn platform_cfg(package: &Package, target: &Target) -> Self {
        Self::cargo(vec![
            Arg::lit("rustc"),
            Arg::lit("--frozen"),
            Arg::Package(package.clone()),
            Arg::Target(target.clone()),
            Arg::lit("--"),
            Arg::lit("--print=cfg"),
        ])
    }

    /// Observe the compiler configuration Cargo selects for an example.
    #[must_use]
    pub fn example_platform_cfg(package: &Package, example: &ExampleName) -> Self {
        Self::cargo(vec![
            Arg::lit("rustc"),
            Arg::lit("--frozen"),
            Arg::Package(package.clone()),
            Arg::Example(example.clone()),
            Arg::lit("--"),
            Arg::lit("--print=cfg"),
        ])
    }

    /// The list pass for a test:
    /// `cargo test --frozen --package=<pkg> <target> -- --list --exact -- <name>`.
    #[must_use]
    pub fn test_list(package: &Package, target: &Target, name: &Filter) -> Self {
        Self::cargo(vec![
            Arg::lit("test"),
            Arg::lit("--frozen"),
            Arg::Package(package.clone()),
            Arg::Target(target.clone()),
            Arg::lit("--"),
            Arg::lit("--list"),
            Arg::lit("--exact"),
            Arg::lit("--"),
            Arg::Filter(name.clone()),
        ])
    }

    /// The run pass for a test:
    /// `cargo test --frozen --package=<pkg> <target> -- --exact -- <name>`.
    #[must_use]
    pub fn test_run(package: &Package, target: &Target, name: &Filter) -> Self {
        Self::cargo(vec![
            Arg::lit("test"),
            Arg::lit("--frozen"),
            Arg::Package(package.clone()),
            Arg::Target(target.clone()),
            Arg::lit("--"),
            Arg::lit("--exact"),
            Arg::lit("--"),
            Arg::Filter(name.clone()),
        ])
    }

    /// The list pass for the documentation tests of a package:
    /// `cargo test --frozen --package=<pkg> --doc -- --list`.
    #[must_use]
    pub fn doc_list(package: &Package) -> Self {
        Self::cargo(vec![
            Arg::lit("test"),
            Arg::lit("--frozen"),
            Arg::Package(package.clone()),
            Arg::lit("--doc"),
            Arg::lit("--"),
            Arg::lit("--list"),
        ])
    }

    /// The run pass for a documentation test:
    /// `cargo test --frozen --package=<pkg> --doc -- <item-path>`.
    #[must_use]
    pub fn doc_run(package: &Package, item: &DocFilter) -> Self {
        Self::cargo(vec![
            Arg::lit("test"),
            Arg::lit("--frozen"),
            Arg::Package(package.clone()),
            Arg::lit("--doc"),
            Arg::lit("--"),
            Arg::DocFilter(item.clone()),
        ])
    }

    /// An example target:
    /// `cargo run --frozen --package=<pkg> --example=<example>`.
    #[must_use]
    pub fn example_run(package: &Package, example: &ExampleName) -> Self {
        Self::cargo(vec![
            Arg::lit("run"),
            Arg::lit("--frozen"),
            Arg::Package(package.clone()),
            Arg::Example(example.clone()),
        ])
    }

    /// Read the complete Census index from a repository example:
    /// `cargo run --frozen --package=<pkg> --example=<example> -- --census-index-v1`.
    /// The fixed versioned token selects this read-only operation alone.
    #[must_use]
    pub fn example_index(package: &Package, example: &ExampleName) -> Self {
        Self::cargo(vec![
            Arg::lit("run"),
            Arg::lit("--frozen"),
            Arg::Package(package.clone()),
            Arg::Example(example.clone()),
            Arg::lit("--"),
            Arg::lit("--census-index-v1"),
        ])
    }

    /// Explicit Census debt publication by a repository example:
    /// `cargo run --frozen --package=<pkg> --example=<example> -- --census-debt-write-v1`.
    /// The fixed versioned token authorizes this operation alone.
    #[must_use]
    pub fn example_debt_write(package: &Package, example: &ExampleName) -> Self {
        Self::cargo(vec![
            Arg::lit("run"),
            Arg::lit("--frozen"),
            Arg::Package(package.clone()),
            Arg::Example(example.clone()),
            Arg::lit("--"),
            Arg::lit("--census-debt-write-v1"),
        ])
    }

    /// The version gate: `git --version`.
    #[must_use]
    pub fn git_version() -> Self {
        Self::git(vec![Arg::lit("--version")])
    }

    /// Whether the working directory is inside a work tree:
    /// `git rev-parse --is-inside-work-tree`.
    #[must_use]
    pub fn git_inside_work_tree() -> Self {
        Self::git(vec![
            Arg::lit("rev-parse"),
            Arg::lit("--is-inside-work-tree"),
        ])
    }

    /// The path of the working directory below the top of its work tree:
    /// `git rev-parse --show-prefix`.
    #[must_use]
    pub fn git_show_prefix() -> Self {
        Self::git(vec![Arg::lit("rev-parse"), Arg::lit("--show-prefix")])
    }

    /// The tracked paths, NUL-separated: `git ls-files -z`.
    #[must_use]
    pub fn git_ls_files() -> Self {
        Self::git(vec![Arg::lit("ls-files"), Arg::lit("-z")])
    }

    /// The default branch of the remote: `git symbolic-ref refs/remotes/origin/HEAD`.
    #[must_use]
    pub fn git_default_ref() -> Self {
        Self::git(vec![
            Arg::lit("symbolic-ref"),
            Arg::lit("refs/remotes/origin/HEAD"),
        ])
    }

    /// The commit a reference names:
    /// `git rev-parse --verify --end-of-options <ref>^{commit}`.
    #[must_use]
    pub fn git_resolve(reference: &RefName) -> Self {
        Self::git(vec![
            Arg::lit("rev-parse"),
            Arg::lit("--verify"),
            Arg::lit("--end-of-options"),
            Arg::RefCommit(reference.clone()),
        ])
    }

    /// Verify an explicit full identifier with its commit qualifier.
    #[must_use]
    pub fn git_resolve_commit_id(commit: &Commit) -> Self {
        Self::git(vec![
            Arg::lit("rev-parse"),
            Arg::lit("--verify"),
            Arg::lit("--end-of-options"),
            Arg::CommitExpression(commit.clone()),
        ])
    }

    /// The merge base with a commit: `git merge-base HEAD <commit>`.
    #[must_use]
    pub fn git_merge_base(default: &Commit) -> Self {
        Self::git(vec![
            Arg::lit("merge-base"),
            Arg::lit("HEAD"),
            Arg::Commit(default.clone()),
        ])
    }

    /// A file at a commit: `git cat-file blob <commit>:<path>`.
    #[must_use]
    pub fn git_blob(rev: &Commit, path: &SourcePath) -> Self {
        Self::git(vec![
            Arg::lit("cat-file"),
            Arg::lit("blob"),
            Arg::RevPath {
                rev: rev.clone(),
                path: path.clone(),
            },
        ])
    }

    /// Every file of the tree of a commit, NUL-separated, each named from the
    /// top of the work tree: `git ls-tree -r -z --full-tree <merge-base>`.
    #[must_use]
    pub fn git_tree(rev: &Commit) -> Self {
        Self::git(vec![
            Arg::lit("ls-tree"),
            Arg::lit("-r"),
            Arg::lit("-z"),
            Arg::lit("--full-tree"),
            Arg::Commit(rev.clone()),
        ])
    }
}

/// The record format of the two history logs: a record separator, then the
/// commit, the author date and the subject separated by unit separators.
const HISTORY_FORMAT: &str = "--format=%x1e%H%x1f%aI%x1f%s";

impl Invocation {
    /// Whether the repository is a shallow clone:
    /// `git rev-parse --is-shallow-repository`.
    #[must_use]
    pub fn git_shallow() -> Self {
        Self::git(vec![
            Arg::lit("rev-parse"),
            Arg::lit("--is-shallow-repository"),
        ])
    }

    /// The history of a document, through its moves, with the merges whose
    /// resolution changed it and the first commit of the history:
    /// `git --no-pager log --no-show-signature --follow --cc --root
    /// --format=%x1e%H%x1f%aI%x1f%s --name-only -- <doc-path>`. The same row
    /// reads the log of each file of a bound module.
    #[must_use]
    pub fn git_document_log(path: &TreePath) -> Self {
        Self::git(vec![
            Arg::lit("--no-pager"),
            Arg::lit("log"),
            Arg::lit("--no-show-signature"),
            Arg::lit("--follow"),
            Arg::lit("--cc"),
            Arg::lit("--root"),
            Arg::lit(HISTORY_FORMAT),
            Arg::lit("--name-only"),
            Arg::lit("--"),
            Arg::Path(path.clone()),
        ])
    }

    /// The history of the lines of a symbol:
    /// `git --no-pager log --no-show-signature -L<start>,<end>:<file>
    /// --no-patch --format=%x1e%H%x1f%aI%x1f%s --`.
    #[must_use]
    pub fn git_symbol_log(lines: LineRange, file: &TreePath) -> Self {
        Self::git(vec![
            Arg::lit("--no-pager"),
            Arg::lit("log"),
            Arg::lit("--no-show-signature"),
            Arg::LineLog {
                lines,
                file: file.clone(),
            },
            Arg::lit("--no-patch"),
            Arg::lit(HISTORY_FORMAT),
            Arg::lit("--"),
        ])
    }

    /// The newest commit that changed how often `identifier` occurs in the
    /// directories, with its parents: `git --no-pager log --no-show-signature
    /// --root -S<identifier> -n 1 --format=%H%x1f%P%x1f%aI%x1f%s --
    /// <crate-dir>...`.
    #[must_use]
    pub fn git_pickaxe(identifier: &Identifier, dirs: &[TreePath]) -> Self {
        let mut args = vec![
            Arg::lit("--no-pager"),
            Arg::lit("log"),
            Arg::lit("--no-show-signature"),
            Arg::lit("--root"),
            Arg::Pickaxe(identifier.clone()),
            Arg::lit("-n"),
            Arg::lit("1"),
            Arg::lit("--format=%H%x1f%P%x1f%aI%x1f%s"),
            Arg::lit("--"),
        ];
        args.extend(dirs.iter().cloned().map(Arg::Path));
        Self::git(args)
    }
}

impl fmt::Display for Invocation {
    /// `program arg arg ...`, for messages. The caller's own command is its
    /// words as [`TrustedArgv::text`] writes them, each that a shell would
    /// split or expand in single quotes, so a message names the command
    /// unambiguously and as its record stores it. The text is never run.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.program == Program::Trusted {
            return f.write_str(&text_of(self.argv().iter().map(String::as_str)));
        }
        write!(f, "{}", self.program)?;
        for arg in &self.args {
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}

/// An argument from a checked native repository protocol, never document command text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HostArgument(String);
