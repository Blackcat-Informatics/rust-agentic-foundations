// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The executor: runs one binding through a [`Runner`] and judges it by the
//! proof its kind requires.

use std::cell::RefCell;
use std::collections::HashMap;

use raf_source::{CompilerPlatform, ItemId, LoadError, TestFn, Workspace};

use crate::Binding;
use crate::exec::{ExecError, Invocation, Limits, Output, Runner};

use super::evidence::{BoundOutcome, Ended, Evidence, Proof, Run};
use super::parse::{self, DocStatus, EntryKind};
use super::plan::{self, DocPlan, ExamplePlan, Plan, Refusal, TestPlan};

/// Runs the tests, documentation tests and examples that capabilities bind.
///
/// Every process goes through the [`Runner`]; the executor starts nothing
/// itself, and it builds an [`Invocation`] only from a [`Plan`], whose values
/// are validated (see [`Refusal`]).
///
/// The limits are those of the completion gate: [`Limits::build`] (1800
/// seconds, `LC_ALL=C`) for each pass of a test or documentation test, which
/// may compile first, and [`Limits::example`] (300 seconds) for the one
/// `cargo run` of an example. That 300 seconds covers the build `cargo run`
/// does first, so an example that needs a longer build is killed and fails.
/// [`Executor::with_limits`] replaces them.
pub struct Executor<'a, R> {
    source: &'a Workspace,
    runner: R,
    tests: HashMap<ItemId, TestFn>,
    build: Limits,
    example: Limits,
    platforms: RefCell<HashMap<Invocation, CompilerPlatform>>,
    view: Option<crate::exec::ViewContext>,
}

impl<'a, R: Runner> Executor<'a, R> {
    /// An executor over the source index and a runner.
    ///
    /// # Errors
    ///
    /// The [`LoadError`] of the index when it cannot say which functions are
    /// tests, which its own invariants rule out.
    pub fn new(source: &'a Workspace, runner: R) -> Result<Self, LoadError> {
        let tests = source
            .test_fns()?
            .into_iter()
            .map(|test| (test.id, test))
            .collect();
        Ok(Self {
            source,
            runner,
            tests,
            build: Limits::build(),
            example: Limits::example(),
            platforms: RefCell::new(HashMap::new()),
            view: None,
        })
    }

    /// The same executor with other limits: `build` for each pass of a test
    /// or documentation test, `example` for the run of an example.
    #[must_use]
    pub fn with_limits(mut self, build: Limits, example: Limits) -> Self {
        self.build = build;
        self.example = example;
        self
    }

    /// Run every compiler observation, listing and execution in this prepared view.
    #[must_use]
    pub fn with_view(mut self, view: &crate::exec::PreparedView) -> Self {
        self.view = Some(view.context().clone());
        self
    }

    /// What the binding runs as, decided from the facts of the source index
    /// alone. No process is started.
    ///
    /// # Errors
    ///
    /// A [`Refusal`] when the path does not resolve, resolves to something
    /// `cargo` cannot run as the argv table runs it (an item that is not a
    /// `#[test]`, a target with `harness = false` or `required-features`, a
    /// benchmark), or yields a value that fails its pattern.
    pub fn plan(&self, binding: &Binding) -> Result<Plan, Refusal> {
        plan::plan(self.source, &self.tests, binding)
    }

    /// Run the binding and judge it.
    ///
    /// A binding that cannot be planned is [`BoundOutcome::Unresolved`] with
    /// no process started. A process killed at its limit, or for its output,
    /// is [`BoundOutcome::Failed`], with the elapsed time in the reason.
    ///
    /// # Errors
    ///
    /// An [`ExecError`] when a process cannot be started or read to its end:
    /// the run has no verdict, which is not the same as a failing binding.
    pub fn run(&self, binding: &Binding) -> Result<Evidence, ExecError> {
        let mut run = Judged::new(binding);
        let plan = match self.plan(binding) {
            Err(refusal) => {
                run.unresolved(refusal);
                return Ok(run.finish());
            }
            Ok(plan) => plan,
        };
        if self.platform_excludes(binding, &plan, &mut run)? {
            return Ok(run.finish());
        }
        match plan {
            Plan::Test(plan) => self.test(&plan, &mut run)?,
            Plan::DocTest(plan) => self.doc(&plan, &mut run)?,
            Plan::Example(plan) => self.example(&plan, &mut run)?,
        }
        Ok(run.finish())
    }

    /// Only a compiler-observed platform condition may authorize nonexecution.
    /// Alternative definitions are alternatives: every site must be excluded.
    fn platform_excludes(
        &self,
        binding: &Binding,
        plan: &Plan,
        run: &mut Judged<'_>,
    ) -> Result<bool, ExecError> {
        // rustdoc has its own flags, so a rustc observation cannot certify its
        // configuration. Its ordinary listing/execution proof remains required.
        if matches!(plan, Plan::DocTest(_)) {
            return Ok(false);
        }
        let Ok(sites) = binding.resolution() else {
            return Ok(false);
        };
        let guards = sites
            .iter()
            .map(|id| self.source.test_platform_guard(*id))
            .collect::<Result<Vec<_>, _>>();
        let guards = match guards {
            Ok(guards) => guards,
            Err(error) => {
                run.fail(format!("cannot inspect binding platform guards: {error}"));
                return Ok(true);
            }
        };
        if !guards
            .iter()
            .any(raf_source::TestPlatformGuard::has_platform_condition)
        {
            return Ok(false);
        }
        let invocation = match plan {
            Plan::Test(plan) => Invocation::platform_cfg(plan.package(), plan.target()),
            Plan::DocTest(_) => return Ok(false),
            Plan::Example(plan) => Invocation::example_platform_cfg(plan.package(), plan.example()),
        };
        if !self.platforms.borrow().contains_key(&invocation) {
            let Some(output) = self.start(run, invocation.clone(), &self.build)? else {
                return Ok(true);
            };
            if !output.success() {
                run.fail(not_success(&invocation, &output));
                return Ok(true);
            }
            let configuration = std::str::from_utf8(&output.stdout)
                .map_err(|error| error.to_string())
                .and_then(|text| CompilerPlatform::parse(text).map_err(|error| error.to_string()));
            match configuration {
                Ok(platform) => {
                    self.platforms
                        .borrow_mut()
                        .insert(invocation.clone(), platform);
                }
                Err(error) => {
                    run.fail(format!(
                        "invalid compiler configuration from `{invocation}`: {error}"
                    ));
                    return Ok(true);
                }
            }
        }
        let platforms = self.platforms.borrow();
        let Some(platform) = platforms.get(&invocation) else {
            run.fail("compiler platform observation was not retained".to_string());
            return Ok(true);
        };
        if guards.is_empty()
            || guards
                .iter()
                .any(|guard| guard.evaluate(platform) != Some(false))
        {
            return Ok(false);
        }
        let conditions = guards
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        run.outcome = Some(BoundOutcome::NotApplicable {
            reason: format!(
                "all definition guards are false ({conditions}) for {platform}, observed by `{invocation}`"
            ),
        });
        Ok(true)
    }

    /// Start one process. `None` when it was killed at a limit, which the run
    /// has recorded as its failure.
    fn start(
        &self,
        run: &mut Judged<'_>,
        invocation: Invocation,
        limits: &Limits,
    ) -> Result<Option<Output>, ExecError> {
        let invocation = if let Some(view) = &self.view {
            invocation.in_view(view)
        } else {
            invocation
        };
        match self.runner.run(&invocation, limits) {
            Ok(output) => {
                run.runs.push(Run::new(
                    invocation,
                    Ended::Exited(output.code),
                    output.elapsed,
                ));
                Ok(Some(output))
            }
            Err(error @ ExecError::Timeout { elapsed, .. }) => {
                run.runs
                    .push(Run::new(invocation, Ended::TimedOut, elapsed));
                run.fail(error.to_string());
                Ok(None)
            }
            Err(error @ ExecError::OutputTooLarge { elapsed, .. }) => {
                run.runs
                    .push(Run::new(invocation, Ended::Overflowed, elapsed));
                run.fail(error.to_string());
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    /// The list pass and the run pass of a test.
    fn test(&self, plan: &TestPlan, run: &mut Judged<'_>) -> Result<(), ExecError> {
        let name = plan.name().as_str();
        let target = plan.target().to_string();
        let list = plan.list();
        let Some(output) = self.start(run, list.clone(), &self.build)? else {
            return Ok(());
        };
        if !output.success() {
            run.fail(not_success(&list, &output));
            return Ok(());
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let listed: Vec<_> = parse::test_list(&text)
            .into_iter()
            .filter(|entry| entry.kind == EntryKind::Test)
            .collect();
        run.proof.listed = Some(listed.len());
        match listed.as_slice() {
            [] => {
                run.unresolved(Refusal::NoTestListed {
                    name: name.to_string(),
                    target,
                });
                return Ok(());
            }
            [only] if only.name == name => {}
            _ => {
                run.unresolved(Refusal::TestListedWrong {
                    name: name.to_string(),
                    target,
                    listed: listed.iter().map(|entry| entry.name.clone()).collect(),
                });
                return Ok(());
            }
        }

        let invocation = plan.run();
        let Some(output) = self.start(run, invocation.clone(), &self.build)? else {
            return Ok(());
        };
        let text = String::from_utf8_lossy(&output.stdout);
        let counts = match parse::results(&text) {
            Ok(counts) => counts,
            Err(parse::BadResult(line)) => {
                run.unresolved(Refusal::Unparsable {
                    what: "result line of the run pass",
                    text: line,
                });
                return Ok(());
            }
        };
        let [counts] = counts.as_slice() else {
            if output.success() {
                run.unresolved(Refusal::Unparsable {
                    what: "result of the run pass (it must hold exactly one `test result:` line)",
                    text: format!("{} result lines", counts.len()),
                });
            } else {
                run.fail(not_success(&invocation, &output));
            }
            return Ok(());
        };
        run.proof.passed = Some(counts.passed);
        run.proof.failed = Some(counts.failed);
        run.proof.ignored = Some(counts.ignored);
        if !output.success() || counts.failed > 0 {
            run.fail(format!(
                "{}; {} passed, {} failed, {} ignored",
                not_success(&invocation, &output),
                counts.passed,
                counts.failed,
                counts.ignored
            ));
        } else if counts.ignored > 0 {
            run.unresolved(Refusal::TestIgnored {
                name: name.to_string(),
            });
        } else if counts.passed == 1 {
            run.outcome = Some(BoundOutcome::Passed);
        } else {
            run.unresolved(Refusal::TestCounts {
                name: name.to_string(),
                passed: counts.passed,
                failed: counts.failed,
                ignored: counts.ignored,
            });
        }
        Ok(())
    }

    /// The list pass over the documentation tests of the package and the run
    /// pass with the item path as the filter.
    fn doc(&self, plan: &DocPlan, run: &mut Judged<'_>) -> Result<(), ExecError> {
        let item = plan.item().as_str();
        let list = plan.list();
        let Some(output) = self.start(run, list.clone(), &self.build)? else {
            return Ok(());
        };
        if !output.success() {
            run.fail(not_success(&list, &output));
            return Ok(());
        }
        let entries = parse::doc_list(&String::from_utf8_lossy(&output.stdout));
        let selected: Vec<_> = entries
            .iter()
            .filter(|entry| entry.item.as_deref() == Some(item))
            .collect();
        run.proof.listed = Some(entries.len());
        run.proof.selected = Some(selected.len());
        if selected.is_empty() {
            run.unresolved(Refusal::NoDocTest {
                item: item.to_string(),
                package: plan.package().to_string(),
            });
            return Ok(());
        }

        let invocation = plan.run();
        let Some(output) = self.start(run, invocation.clone(), &self.build)? else {
            return Ok(());
        };
        let text = String::from_utf8_lossy(&output.stdout);
        let by_status = |wanted: &[DocStatus]| -> Vec<String> {
            selected
                .iter()
                .filter(|entry| wanted.contains(&parse::doc_status(&text, &entry.name)))
                .map(|entry| entry.name.clone())
                .collect()
        };
        let ok = by_status(&[DocStatus::Ok]).len();
        run.proof.ok_lines = Some(ok);
        let not_runnable = by_status(&[DocStatus::CompileOnly, DocStatus::CompileFail]);
        let failed = by_status(&[DocStatus::Failed]);
        let not_run = by_status(&[DocStatus::Ignored, DocStatus::Missing]);
        if !not_runnable.is_empty() {
            run.unresolved(Refusal::DocNotRunnable {
                item: item.to_string(),
                names: not_runnable,
            });
        } else if !output.success() || !failed.is_empty() {
            // The exit status covers the documentation tests the substring
            // also matched (`foo_more` for `foo`), which the gate does not
            // count and which still refuse the binding.
            let named = if failed.is_empty() {
                String::new()
            } else {
                format!("; failed: {}", failed.join(", "))
            };
            run.fail(format!("{}{named}", not_success(&invocation, &output)));
        } else if !not_run.is_empty() {
            run.unresolved(Refusal::DocNotRun {
                item: item.to_string(),
                names: not_run,
            });
        } else if ok == selected.len() {
            run.outcome = Some(BoundOutcome::Passed);
        }
        Ok(())
    }

    /// The run of an example: its exit status decides.
    fn example(&self, plan: &ExamplePlan, run: &mut Judged<'_>) -> Result<(), ExecError> {
        let invocation = plan.run();
        let Some(output) = self.start(run, invocation.clone(), &self.example)? else {
            return Ok(());
        };
        if output.success() {
            run.outcome = Some(BoundOutcome::Passed);
        } else {
            run.fail(not_success(&invocation, &output));
        }
        Ok(())
    }
}

/// The unsuccessful process's invocation, exit and bounded labelled streams.
fn not_success(invocation: &Invocation, output: &Output) -> String {
    let how = output.code.map_or_else(
        || "was ended by a signal".to_string(),
        |code| format!("exited with code {code}"),
    );
    let mut reason = format!("`{invocation}` {how}");
    for (label, text) in [
        ("stdout", output.stdout_text()),
        ("stderr", output.stderr_text()),
    ] {
        if !text.is_empty() {
            reason.push('\n');
            reason.push_str(label);
            reason.push_str(": ");
            reason.push_str(&text);
        }
    }
    reason
}

/// A run in progress.
struct Judged<'b> {
    binding: &'b Binding,
    runs: Vec<Run>,
    proof: Proof,
    outcome: Option<BoundOutcome>,
}

impl<'b> Judged<'b> {
    fn new(binding: &'b Binding) -> Self {
        Self {
            binding,
            runs: Vec::new(),
            proof: Proof::default(),
            outcome: None,
        }
    }

    fn fail(&mut self, reason: String) {
        self.outcome = Some(BoundOutcome::Failed { reason });
    }

    fn unresolved(&mut self, reason: Refusal) {
        self.outcome = Some(BoundOutcome::Unresolved { reason });
    }

    /// The evidence. A run that reached no verdict is a failure: a pass is
    /// only ever set by the proof of its kind.
    fn finish(self) -> Evidence {
        let outcome = self.outcome.unwrap_or_else(|| BoundOutcome::Failed {
            reason: "the run ended without the proof its kind requires".to_string(),
        });
        Evidence::new(
            self.binding.capability(),
            self.binding.kind(),
            self.binding.path(),
            self.runs,
            self.proof,
            outcome,
        )
    }
}
