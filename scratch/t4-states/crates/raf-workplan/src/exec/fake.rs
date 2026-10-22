// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The in-process runner for tests: scripted answers, recorded calls.

use std::sync::{Mutex, MutexGuard, PoisonError};

use super::{ExecError, Invocation, Limits, Output, Program, Runner, Stream};

/// One invocation a [`FakeRunner`] was asked to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    program: Program,
    args: Vec<String>,
    limits: Limits,
}

impl Call {
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
            .retain(|rule| !(rule.program == program && rule.args == args));
        self.rules.push(Rule {
            program,
            args,
            answer,
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
    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        let call = Call {
            program: invocation.program(),
            args: invocation.argv(),
            limits: *limits,
        };
        let command = call.command();
        let answer = self
            .rules
            .iter()
            .find(|rule| rule.program == call.program && rule.args == call.args)
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
