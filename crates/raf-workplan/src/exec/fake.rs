// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The in-process runner for tests: scripted answers, recorded calls.

use std::sync::{Mutex, MutexGuard, PoisonError};

use super::{ExecError, Invocation, Limits, Output, Program, Runner, Stream};

type ViewCheck = fn(&super::CapturedView) -> Result<(), String>;

/// One invocation a [`FakeRunner`] was asked to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    program: Program,
    args: Vec<String>,
    limits: Limits,
    view: Option<super::ViewContext>,
    rustdoc: Option<super::RustdocObservationOptions>,
    rustdoc_delegate: bool,
}

impl Call {
    /// Scoped documentation compiler controls retained by the actual invocation.
    #[must_use]
    pub fn rustdoc_observation(&self) -> Option<&super::RustdocObservationOptions> {
        self.rustdoc.as_ref()
    }

    /// Whether the selected compiler runs directly with observer controls removed.
    #[must_use]
    pub const fn is_rustdoc_delegate(&self) -> bool {
        self.rustdoc_delegate
    }

    /// Explicit prepared context, absent for original-root calls.
    #[must_use]
    pub fn view(&self) -> Option<&super::ViewContext> {
        self.view.as_ref()
    }

    /// The program.
    #[must_use]
    pub fn program(&self) -> Program {
        self.program
    }

    /// The argv vector, without the program.
    #[must_use]
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// The limits the caller asked for.
    #[must_use]
    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    /// `program arg arg ...`, as the invocation reads in a message; for the
    /// caller's own command, its words.
    #[must_use]
    pub fn command(&self) -> String {
        if self.program == Program::Trusted {
            return self.args.join(" ");
        }
        let mut text = self.program.to_string();
        for arg in &self.args {
            text.push(' ');
            text.push_str(arg);
        }
        text
    }
}

/// A scripted answer for one invocation.
#[derive(Debug)]
struct Rule {
    program: Program,
    args: Vec<String>,
    answer: Result<Output, ExecError>,
    prepared: bool,
}

/// A runner that starts nothing. It answers an invocation from the script it
/// was built with and records every call, on every platform.
///
/// Answers repeat: an invocation gets the same answer each time it is asked.
/// An invocation with no answer is [`ExecError::Unscripted`], so a test that
/// runs a command it did not expect fails. The bounds of the [`Limits`] are
/// applied to a scripted [`Output`] as the real runner applies them to a
/// process: more bytes than the output bound on a stream is
/// [`ExecError::OutputTooLarge`], and an elapsed time over the time limit is
/// [`ExecError::Timeout`].
#[derive(Debug, Default)]
pub struct FakeRunner {
    rules: Vec<Rule>,
    calls: Mutex<Vec<Call>>,
    view_check: Option<ViewCheck>,
    views: super::VirtualViews,
}

impl FakeRunner {
    /// A runner with no answers.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Answer the invocation with exactly this program and argv vector. A
    /// later answer for the same invocation replaces the earlier one.
    #[must_use]
    pub fn on(
        mut self,
        program: Program,
        args: &[&str],
        answer: Result<Output, ExecError>,
    ) -> Self {
        let args: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();
        self.rules
            .retain(|rule| !(rule.program == program && rule.args == args && !rule.prepared));
        self.rules.push(Rule {
            program,
            args,
            answer,
            prepared: false,
        });
        self
    }

    /// Explicitly accept virtual captures only after this model expectation passes.
    #[must_use]
    pub fn with_view_check(
        mut self,
        check: fn(&super::CapturedView) -> Result<(), String>,
    ) -> Self {
        self.view_check = Some(check);
        self
    }

    /// Script a command specifically in an accepted prepared view.
    /// Ordinary `on` rules never answer prepared invocations.
    #[must_use]
    pub fn on_view(
        mut self,
        program: Program,
        args: &[&str],
        answer: Result<Output, ExecError>,
    ) -> Self {
        let args: Vec<String> = args.iter().map(|value| (*value).to_string()).collect();
        self.rules
            .retain(|rule| !(rule.program == program && rule.args == args && rule.prepared));
        self.rules.push(Rule {
            program,
            args,
            answer,
            prepared: true,
        });
        self
    }

    /// Every call so far, in order.
    #[must_use]
    pub fn calls(&self) -> Vec<Call> {
        self.recorded().clone()
    }

    fn recorded(&self) -> MutexGuard<'_, Vec<Call>> {
        self.calls.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Runner for FakeRunner {
    fn prepare_view(&self, request: &super::ViewRequest) -> Result<super::PreparedView, ExecError> {
        let check = self
            .view_check
            .ok_or_else(|| super::view::unsupported("prepare scripted view"))?;
        let view = self.views.prepare(request)?;
        if let Err(reason) = check(view.captured()) {
            let primary = super::ViewError::refusal("scripted completed model", reason).into();
            return match self.views.finish(&view) {
                Ok(()) => Err(primary),
                Err(cleanup) => Err(ExecError::ViewCleanup {
                    primary: Box::new(primary),
                    cleanup: Box::new(cleanup),
                }),
            };
        }
        Ok(view)
    }
    fn revalidate_view(&self, view: &super::PreparedView) -> Result<(), ExecError> {
        self.views.revalidate(view)
    }
    fn finish_view(&self, view: &super::PreparedView) -> Result<(), ExecError> {
        self.views.finish(view)
    }
    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        invocation.validate_rustdoc_context()?;
        if let Some(context) = invocation.view() {
            self.views.validate(context)?;
        }
        let call = Call {
            program: invocation.program(),
            args: invocation.argv(),
            limits: *limits,
            view: invocation.view().cloned(),
            rustdoc: invocation.rustdoc_observation().cloned(),
            rustdoc_delegate: invocation.is_rustdoc_delegate(),
        };
        let command = call.command();
        let answer = self
            .rules
            .iter()
            .find(|rule| {
                rule.program == call.program
                    && rule.args == call.args
                    && rule.prepared == call.view.is_some()
            })
            .map(|rule| rule.answer.clone());
        self.recorded().push(call);
        let output = answer.ok_or(ExecError::Unscripted {
            command: command.clone(),
        })??;
        if output.elapsed > limits.timeout() {
            return Err(ExecError::Timeout {
                command,
                timeout: limits.timeout(),
                elapsed: output.elapsed,
            });
        }
        for (stream, bytes) in [
            (Stream::Stdout, &output.stdout),
            (Stream::Stderr, &output.stderr),
        ] {
            if bytes.len() > limits.max_output() {
                return Err(ExecError::OutputTooLarge {
                    command,
                    stream,
                    limit: limits.max_output(),
                    elapsed: output.elapsed,
                });
            }
        }
        Ok(output)
    }
}
