// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What a binding runs as, decided from the facts of the source index before
//! any process is started.
//!
//! A plan holds only validated values (see [`crate::exec::Pattern`]); a fact
//! that fails its pattern, or that says `cargo` could not run the item in the
//! way the argv table runs it, is a [`Refusal`] and the binding is never
//! passed to `cargo`.

use std::fmt;

use raf_source::{Item, ItemId, SelfType, Target as SourceTarget, TargetKind, TestFn, Workspace};

use crate::exec::{
    DocFilter, ExampleName, Filter, Invocation, Package, PatternError, Target, TargetName,
};
use crate::{Binding, BindingKind, ExampleKind};

/// Why a binding was not run, or why a run proves nothing. Each case names the
/// binding and what to change; none is a skip.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Refusal {
    /// A `code` binding: nothing runs it.
    NotRunnable {
        /// The bound path.
        path: String,
    },
    /// The path does not resolve in the source index.
    Unresolved {
        /// The bound path.
        path: String,
        /// What the index said.
        error: String,
    },
    /// The path resolves to something that is not a function marked
    /// `#[test]`.
    NotATest {
        /// The bound path.
        path: String,
        /// What it is, with its place.
        found: String,
    },
    /// An `examples` binding that is neither an example target nor a
    /// documented item of a library whose documentation tests run.
    NotAnExample {
        /// The bound path.
        path: String,
    },
    /// The test is in a target that `cargo test` runs in a way the argv table
    /// cannot select a test in: a benchmark, an example or the build script.
    TargetKind {
        /// The bound path.
        path: String,
        /// The Cargo name of the target.
        target: String,
        /// What the target is.
        kind: TargetKind,
    },
    /// The target has `harness = false`: its output is not libtest's, so no
    /// test can be listed or counted in it.
    NoHarness {
        /// The bound path.
        path: String,
        /// The Cargo name of the target.
        target: String,
    },
    /// The target has `required-features`, which the argv table does not
    /// enable.
    RequiredFeatures {
        /// The bound path.
        path: String,
        /// The Cargo name of the target.
        target: String,
        /// The features it requires.
        features: Vec<String>,
    },
    /// The path of a documented item cannot be written as the name rustdoc
    /// prints, for instance the root of a crate, which has no item part.
    NoDocItem {
        /// The bound path.
        path: String,
    },
    /// A value taken from the index does not match its pattern.
    Pattern {
        /// The bound path.
        path: String,
        /// Which value it is.
        what: &'static str,
        /// The refusal of the pattern.
        error: PatternError,
    },
    /// The definitions the path resolves to would run as different things.
    Divergent {
        /// The bound path.
        path: String,
        /// What each definition would run as.
        plans: Vec<String>,
    },
    /// The list pass shows no `<name>: test` entry.
    NoTestListed {
        /// The resolved test name.
        name: String,
        /// The target the list pass was pinned to.
        target: String,
    },
    /// The list pass shows more than one entry, or one under another name.
    TestListedWrong {
        /// The resolved test name.
        name: String,
        /// The target the list pass was pinned to.
        target: String,
        /// The entries listed.
        listed: Vec<String>,
    },
    /// The run pass did not report exactly one test passed and none failed or
    /// ignored.
    TestCounts {
        /// The resolved test name.
        name: String,
        /// Tests passed.
        passed: usize,
        /// Tests failed.
        failed: usize,
        /// Tests ignored.
        ignored: usize,
    },
    /// The test is `#[ignore]`d: it is listed and it did not run.
    TestIgnored {
        /// The resolved test name.
        name: String,
    },
    /// The output of a pass does not have the form the proof reads.
    Unparsable {
        /// What was read.
        what: &'static str,
        /// The offending text.
        text: String,
    },
    /// The list pass selects no documentation test of the item.
    NoDocTest {
        /// The item path within the crate.
        item: String,
        /// The package.
        package: String,
    },
    /// A selected documentation test was built and not run (`no_run`), or
    /// passes when it does not build (`compile_fail`).
    DocNotRunnable {
        /// The item path within the crate.
        item: String,
        /// The names of the tests, as rustdoc prints them.
        names: Vec<String>,
    },
    /// A selected documentation test has no `... ok` line in the run output
    /// and did not fail: it was ignored or did not appear.
    DocNotRun {
        /// The item path within the crate.
        item: String,
        /// The names of the tests, as rustdoc prints them.
        names: Vec<String>,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRunnable { path } => write!(
                f,
                "`{path}` is a code binding: nothing runs it; bind a test or an example to have it run"
            ),
            Self::Unresolved { path, error } => write!(
                f,
                "`{path}` does not resolve in the source index ({error}); nothing is passed to cargo"
            ),
            Self::NotATest { path, found } => write!(
                f,
                "`{path}` resolves to {found}, not to a function marked `#[test]`, so it would run zero tests; bind a `#[test]` function"
            ),
            Self::NotAnExample { path } => write!(
                f,
                "`{path}` is neither the crate root of an example target nor a documented item of a library whose documentation tests run; bind an example or a documented item"
            ),
            Self::TargetKind { path, target, kind } => write!(
                f,
                "`{path}` is in the {} target `{target}`, and `cargo test` cannot select a test there with `--lib`, `--bin` or `--test`; move the test to the library, a binary or an integration test",
                kind_name(*kind)
            ),
            Self::NoHarness { path, target } => write!(
                f,
                "`{path}` is in the target `{target}`, which has `harness = false` in its manifest: its output is not libtest's, so no test can be listed or counted; bind a test of a target with the libtest harness"
            ),
            Self::RequiredFeatures {
                path,
                target,
                features,
            } => write!(
                f,
                "`{path}` is in the target `{target}`, which has `required-features = [{}]`; the argv table enables no feature, so the target would not build; bind a test of a target with no required features",
                features.join(", ")
            ),
            Self::NoDocItem { path } => write!(
                f,
                "`{path}` has no item part in a documentation test name (the documentation of a crate root is named `src/lib.rs - (line n)`); bind a documented item"
            ),
            Self::Pattern { path, what, error } => write!(
                f,
                "`{path}`: the {what} is refused before any process starts: {error}"
            ),
            Self::Divergent { path, plans } => write!(
                f,
                "`{path}` has several definitions that would run as different things ({}); bind the canonical path of one definition",
                plans.join("; ")
            ),
            Self::NoTestListed { name, target } => write!(
                f,
                "the list pass of `{target}` shows no test named `{name}`, so a run would prove nothing (a test behind a disabled `cfg(feature)`, or a name that differs from the libtest name, lists nothing); bind a test that the target lists"
            ),
            Self::TestListedWrong {
                name,
                target,
                listed,
            } => write!(
                f,
                "the list pass of `{target}` for `{name}` shows {} entries ({}), not exactly one test named `{name}`; bind a test that the target lists once",
                listed.len(),
                listed.join(", ")
            ),
            Self::TestCounts {
                name,
                passed,
                failed,
                ignored,
            } => write!(
                f,
                "the run of `{name}` reported {passed} passed; {failed} failed; {ignored} ignored, and the proof needs exactly 1 passed; 0 failed; 0 ignored"
            ),
            Self::TestIgnored { name } => write!(
                f,
                "`{name}` is marked `#[ignore]`: it is listed as a test and the run did not run it; remove `#[ignore]` or bind a test that runs"
            ),
            Self::Unparsable { what, text } => write!(
                f,
                "the {what} cannot be read: `{}`; the run has no proof",
                text.escape_debug()
            ),
            Self::NoDocTest { item, package } => write!(
                f,
                "no documentation test of the package `{package}` is listed for the item `{item}`; document the item with a runnable example, or bind another item"
            ),
            Self::DocNotRunnable { item, names } => write!(
                f,
                "the documentation test of `{item}` ({}) was built and not run (`no_run`) or passes when it does not build (`compile_fail`); bind a runnable doc test",
                names.join(", ")
            ),
            Self::DocNotRun { item, names } => write!(
                f,
                "the documentation test of `{item}` ({}) has no `... ok` line in the run output; it was ignored or did not run",
                names.join(", ")
            ),
        }
    }
}

impl std::error::Error for Refusal {}

fn kind_name(kind: TargetKind) -> &'static str {
    match kind {
        TargetKind::Lib => "library",
        TargetKind::Bin => "binary",
        TargetKind::Test => "integration test",
        TargetKind::Bench => "benchmark",
        TargetKind::Example => "example",
        TargetKind::BuildScript => "build script",
        _ => "unknown",
    }
}

/// A test binding, ready to run:
/// `cargo test --frozen --package=<pkg> <target> -- --list --exact -- <name>`
/// and then the run pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestPlan {
    package: Package,
    target: Target,
    name: Filter,
}

impl TestPlan {
    /// The package.
    #[must_use]
    pub fn package(&self) -> &Package {
        &self.package
    }

    /// The pinned target: `--lib`, `--bin=<bin>` or `--test=<test>`.
    #[must_use]
    pub fn target(&self) -> &Target {
        &self.target
    }

    /// The libtest name of the test within its target.
    #[must_use]
    pub fn name(&self) -> &Filter {
        &self.name
    }

    /// The list pass.
    #[must_use]
    pub fn list(&self) -> Invocation {
        Invocation::test_list(&self.package, &self.target, &self.name)
    }

    /// The run pass.
    #[must_use]
    pub fn run(&self) -> Invocation {
        Invocation::test_run(&self.package, &self.target, &self.name)
    }
}

/// A documentation-test binding, ready to run:
/// `cargo test --frozen --package=<pkg> --doc -- --list` and then
/// `cargo test --frozen --package=<pkg> --doc -- <item-path>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocPlan {
    package: Package,
    item: DocFilter,
}

impl DocPlan {
    /// The package.
    #[must_use]
    pub fn package(&self) -> &Package {
        &self.package
    }

    /// The item path within the crate, as rustdoc prints it.
    #[must_use]
    pub fn item(&self) -> &DocFilter {
        &self.item
    }

    /// The list pass over the documentation tests of the package.
    #[must_use]
    pub fn list(&self) -> Invocation {
        Invocation::doc_list(&self.package)
    }

    /// The run pass, with the item path as the substring filter.
    #[must_use]
    pub fn run(&self) -> Invocation {
        Invocation::doc_run(&self.package, &self.item)
    }
}

/// An example binding, ready to run:
/// `cargo run --frozen --package=<pkg> --example=<example>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExamplePlan {
    package: Package,
    example: ExampleName,
}

impl ExamplePlan {
    /// The package.
    #[must_use]
    pub fn package(&self) -> &Package {
        &self.package
    }

    /// The Cargo name of the example target.
    #[must_use]
    pub fn example(&self) -> &ExampleName {
        &self.example
    }

    /// The run.
    #[must_use]
    pub fn run(&self) -> Invocation {
        Invocation::example_run(&self.package, &self.example)
    }
}

/// What a binding runs as.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Plan {
    /// A `#[test]` function in a pinned target.
    Test(TestPlan),
    /// A documented item's documentation tests.
    DocTest(DocPlan),
    /// An example target.
    Example(ExamplePlan),
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Test(plan) => write!(
                f,
                "test `{}` of package `{}` ({})",
                plan.name, plan.package, plan.target
            ),
            Self::DocTest(plan) => write!(
                f,
                "documentation tests of `{}` in package `{}`",
                plan.item, plan.package
            ),
            Self::Example(plan) => write!(
                f,
                "example `{}` of package `{}`",
                plan.example, plan.package
            ),
        }
    }
}

/// The plan of a binding, decided from the source index.
///
/// `tests` is the index's `#[test]` functions by item.
pub(crate) fn plan(
    source: &Workspace,
    tests: &std::collections::HashMap<ItemId, TestFn>,
    binding: &Binding,
) -> Result<Plan, Refusal> {
    let path = binding.path();
    let sites = binding.resolution().map_err(|error| Refusal::Unresolved {
        path: path.to_string(),
        error: error.to_string(),
    })?;
    let mut plans = Vec::new();
    for id in sites {
        let plan = match binding.kind() {
            BindingKind::Code => {
                return Err(Refusal::NotRunnable {
                    path: path.to_string(),
                });
            }
            BindingKind::Test => test_site(source, tests, path, *id)?,
            BindingKind::Example => example_site(source, binding, path, *id)?,
        };
        if !plans.contains(&plan) {
            plans.push(plan);
        }
    }
    let mut plans = plans.into_iter();
    match (plans.next(), plans.next()) {
        (Some(only), None) => Ok(only),
        (Some(first), Some(second)) => Err(Refusal::Divergent {
            path: path.to_string(),
            plans: [first, second]
                .into_iter()
                .chain(plans)
                .map(|plan| plan.to_string())
                .collect(),
        }),
        // A resolution is never empty.
        (None, _) => Err(Refusal::Unresolved {
            path: path.to_string(),
            error: "the path resolves to no definition".to_string(),
        }),
    }
}

fn test_site(
    source: &Workspace,
    tests: &std::collections::HashMap<ItemId, TestFn>,
    path: &str,
    id: ItemId,
) -> Result<Plan, Refusal> {
    let (Some(item), Some(test)) = (source.item(id), tests.get(&id)) else {
        return Err(Refusal::NotATest {
            path: path.to_string(),
            found: source
                .item(id)
                .map_or_else(|| "nothing the index holds".to_string(), describe),
        });
    };
    let Some(target) = source.target(item.target) else {
        return Err(Refusal::NotATest {
            path: path.to_string(),
            found: "an item whose target the index does not hold".to_string(),
        });
    };
    test_plan(
        path,
        &TestFacts {
            package: &test.package,
            kind: test.kind,
            target: &test.target,
            name: &test.name,
            harness: target.harness,
            required_features: &target.required_features,
        },
    )
    .map(Plan::Test)
}

/// What the index says of an item that is not a test.
fn describe(item: &Item) -> String {
    format!(
        "{} defined at {}:{}",
        format!("{:?}", item.kind).to_lowercase(),
        item.file,
        item.name_line
    )
}

/// The index facts of a test function, as text. The tests of this module
/// build them by hand, with values no manifest would hold, to show that each
/// one is judged before it reaches an invocation.
pub(crate) struct TestFacts<'a> {
    pub(crate) package: &'a str,
    pub(crate) kind: TargetKind,
    pub(crate) target: &'a str,
    pub(crate) name: &'a str,
    pub(crate) harness: bool,
    pub(crate) required_features: &'a [String],
}

pub(crate) fn test_plan(path: &str, facts: &TestFacts<'_>) -> Result<TestPlan, Refusal> {
    let pinned =
        |name: &str| TargetName::new(name).map_err(|error| pattern(path, "target name", error));
    let target = match facts.kind {
        TargetKind::Lib => Target::Lib,
        TargetKind::Bin => Target::Bin(pinned(facts.target)?),
        TargetKind::Test => Target::Test(pinned(facts.target)?),
        kind => {
            return Err(Refusal::TargetKind {
                path: path.to_string(),
                target: facts.target.to_string(),
                kind,
            });
        }
    };
    if !facts.harness {
        return Err(Refusal::NoHarness {
            path: path.to_string(),
            target: facts.target.to_string(),
        });
    }
    if !facts.required_features.is_empty() {
        return Err(Refusal::RequiredFeatures {
            path: path.to_string(),
            target: facts.target.to_string(),
            features: facts.required_features.to_vec(),
        });
    }
    Ok(TestPlan {
        package: Package::new(facts.package)
            .map_err(|error| pattern(path, "package name", error))?,
        target,
        name: Filter::new(facts.name).map_err(|error| pattern(path, "test name", error))?,
    })
}

fn example_site(
    source: &Workspace,
    binding: &Binding,
    path: &str,
    id: ItemId,
) -> Result<Plan, Refusal> {
    let not_an_example = || Refusal::NotAnExample {
        path: path.to_string(),
    };
    let item = source.item(id).ok_or_else(not_an_example)?;
    let target = source.target(item.target).ok_or_else(not_an_example)?;
    match binding.example_kind(id).ok_or_else(not_an_example)? {
        ExampleKind::Target => example_plan(
            path,
            &ExampleFacts {
                package: &target.package,
                example: &target.name,
                required_features: &target.required_features,
            },
        )
        .map(Plan::Example),
        ExampleKind::Doctest => {
            let within = doc_item_path(item, target).ok_or_else(|| Refusal::NoDocItem {
                path: path.to_string(),
            })?;
            doc_plan(path, &target.package, &within).map(Plan::DocTest)
        }
    }
}

/// The index facts of an example target, as text.
pub(crate) struct ExampleFacts<'a> {
    pub(crate) package: &'a str,
    pub(crate) example: &'a str,
    pub(crate) required_features: &'a [String],
}

pub(crate) fn example_plan(path: &str, facts: &ExampleFacts<'_>) -> Result<ExamplePlan, Refusal> {
    if !facts.required_features.is_empty() {
        return Err(Refusal::RequiredFeatures {
            path: path.to_string(),
            target: facts.example.to_string(),
            features: facts.required_features.to_vec(),
        });
    }
    Ok(ExamplePlan {
        package: Package::new(facts.package)
            .map_err(|error| pattern(path, "package name", error))?,
        example: ExampleName::new(facts.example)
            .map_err(|error| pattern(path, "example name", error))?,
    })
}

pub(crate) fn doc_plan(path: &str, package: &str, item: &str) -> Result<DocPlan, Refusal> {
    Ok(DocPlan {
        package: Package::new(package).map_err(|error| pattern(path, "package name", error))?,
        item: DocFilter::new(item).map_err(|error| pattern(path, "item path", error))?,
    })
}

fn pattern(path: &str, what: &'static str, error: PatternError) -> Refusal {
    Refusal::Pattern {
        path: path.to_string(),
        what,
        error,
    }
}

/// The item part of the name rustdoc gives the documentation tests of `item`:
/// its path within the crate, with an inherent impl's method written under the
/// self type as the impl writes it (`Wrapper<T>::get`). `None` for the crate
/// root, which has no item part.
///
/// The impl block is taken to sit in the module of its self type; an impl
/// written elsewhere, or under another spelling of its self type, is named
/// differently by rustdoc and the list pass then selects nothing, which is a
/// refusal.
fn doc_item_path(item: &Item, target: &SourceTarget) -> Option<String> {
    let within = item
        .path
        .strip_prefix(&target.crate_path)?
        .strip_prefix("::")?;
    if let (Some(self_type), Some(SelfType::Local)) = (&item.self_type, &item.self_key) {
        let (owner, method) = within.rsplit_once("::")?;
        let (modules, type_name) = owner
            .rsplit_once("::")
            .map_or(("", owner), |(modules, name)| (modules, name));
        if self_type.split('<').next() == Some(type_name) {
            return Some(if modules.is_empty() {
                format!("{self_type}::{method}")
            } else {
                format!("{modules}::{self_type}::{method}")
            });
        }
    }
    Some(within.to_string())
}
