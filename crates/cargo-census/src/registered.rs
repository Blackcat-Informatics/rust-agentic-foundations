// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded repository-owned check execution; no built-in-only fallback.

use std::{error::Error as StdError, io::Write, path::Path};

use crate::error::Error;

#[cfg(test)]
mod tests;
use raf_census::{MAX_CHECK_RESPONSE_BYTES, RegisteredCheck};
use raf_workplan::exec::{
    ExampleName, Invocation, Limits, Package, Programs, Runner, SystemRunner,
};

pub(crate) fn check(
    root: &Path,
    package: &Package,
    example: &ExampleName,
    output: &mut dyn Write,
) -> Result<u8, Error> {
    let runner = SystemRunner::new(Programs::new(root));
    run(&runner, package, example, output)
}

fn run(
    runner: &dyn Runner,
    package: &Package,
    example: &ExampleName,
    output: &mut dyn Write,
) -> Result<u8, Error> {
    let context = format!("registered check {package}/{example}");
    let captured = runner
        .run(
            &Invocation::example_run(package, example),
            &Limits::build().with_max_output(MAX_CHECK_RESPONSE_BYTES),
        )
        .map_err(|source| input(&context, source))?;
    if !matches!(captured.code, Some(0 | 1)) {
        return Err(Error::Transport(format!(
            "{context}: target failed with exit {:?}; stderr: {}",
            captured.code,
            captured.stderr.escape_ascii()
        )));
    }
    let text = std::str::from_utf8(&captured.stdout)
        .map_err(|source| input(&diagnostics(&context, &captured.stderr), source))?;
    let response = RegisteredCheck::from_ntriples(text)
        .map_err(|source| input(&diagnostics(&context, &captured.stderr), source))?;
    if captured.code != Some(i32::from(response.exit_code())) {
        return Err(Error::Transport(format!(
            "{}: target exit {:?} disagrees with protocol outcome {}",
            diagnostics(&context, &captured.stderr),
            captured.code,
            response.exit_code()
        )));
    }
    if let Some(error) = response.error_message() {
        return Err(Error::Transport(format!(
            "{context}: repository evaluation failed: {}",
            error.escape_debug()
        )));
    }
    if response.exit_code() == 0 {
        writeln!(output, "census: clean").map_err(|source| input("<stdout>", source))?;
    } else {
        for finding in response.findings() {
            writeln!(output, "{finding}").map_err(|source| input("<stdout>", source))?;
        }
    }
    Ok(response.exit_code())
}

fn diagnostics(context: &str, stderr: &[u8]) -> String {
    if stderr.is_empty() {
        context.into()
    } else {
        format!("{context}; stderr: {}", stderr.escape_ascii())
    }
}

fn input(context: &str, source: impl StdError + Send + Sync + 'static) -> Error {
    Error::Input {
        context: context.into(),
        source: Box::new(source),
    }
}
