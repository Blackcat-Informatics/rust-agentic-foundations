// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The typed invocation: a program and arguments that can only be built from
//! fixed text and validated values.

use std::fmt;

use super::pattern::{
    Commit, DocFilter, ExampleName, Filter, Package, RefName, SourcePath, Target,
};

/// Which program an invocation starts. The path of each is held by
/// [`Programs`](super::Programs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Program {
    /// `cargo`, from `$CARGO` or `PATH`.
    Cargo,
    /// `git`, from `PATH`.
    Git,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cargo => "cargo",
            Self::Git => "git",
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
            Self::Literal(text) => write!(f, "{text}"),
            Self::Package(package) => write!(f, "--package={package}"),
            Self::Target(target) => write!(f, "{target}"),
            Self::Example(example) => write!(f, "--example={example}"),
            Self::Filter(filter) => write!(f, "{filter}"),
            Self::DocFilter(filter) => write!(f, "{filter}"),
            Self::RefCommit(name) => write!(f, "{name}^{{commit}}"),
            Self::Commit(commit) => write!(f, "{commit}"),
            Self::RevPath { rev, path } => write!(f, "{rev}:{path}"),
        }
    }
}

/// A program and its arguments, the only thing a [`Runner`](super::Runner)
/// accepts.
///
/// The constructors below are the argv tables of the capability documents, one
/// function for each row. There is no public way to build any other invocation:
/// the general constructor is private to the crate, and so is the constructor of
/// [`Literal`], so a caller cannot assemble an argv of its own.
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
}

impl Invocation {
    /// An invocation of `program` with these arguments.
    #[must_use]
    pub(crate) fn new(program: Program, args: Vec<Arg>) -> Self {
        Self { program, args }
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
    /// element of the argv vector and without the program.
    #[must_use]
    pub fn argv(&self) -> Vec<String> {
        self.args.iter().map(ToString::to_string).collect()
    }

    /// A cargo invocation of these literals, a package and the tail.
    fn cargo(args: Vec<Arg>) -> Self {
        Self::new(Program::Cargo, args)
    }

    fn git(args: Vec<Arg>) -> Self {
        Self::new(Program::Git, args)
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

    /// Root discovery: `cargo metadata --format-version 1 --no-deps --frozen`.
    #[must_use]
    pub fn metadata() -> Self {
        Self::cargo(vec![
            Arg::lit("metadata"),
            Arg::lit("--format-version"),
            Arg::lit("1"),
            Arg::lit("--no-deps"),
            Arg::lit("--frozen"),
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

    /// The merge base with a commit: `git merge-base HEAD <commit>`.
    #[must_use]
    pub fn git_merge_base(default: &Commit) -> Self {
        Self::git(vec![
            Arg::lit("merge-base"),
            Arg::lit("HEAD"),
            Arg::Commit(default.clone()),
        ])
    }

    /// A file at a commit: `git cat-file blob <merge-base>:<path>`.
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
}

impl fmt::Display for Invocation {
    /// `program arg arg ...`, for messages. The text is not a shell command
    /// line and is never run.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.program)?;
        for arg in &self.args {
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}
