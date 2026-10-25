// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded repository-owned checking and explicitly authorized debt publication.

use std::{error::Error as StdError, io::Write, path::Path};

use crate::cli::RdfFormat;
use crate::error::Error;

#[cfg(test)]
mod tests;
use raf_census::{
    Index, MAX_CHECK_RESPONSE_BYTES, NORMAL_FORM, RegisteredCheck, RegisteredDebtWrite,
    RegisteredOperation,
};
use raf_workplan::exec::{
    ExampleName, Invocation, Limits, Output, Package, Programs, Runner, SystemRunner,
};

pub(crate) fn check(
    root: &Path,
    package: &Package,
    example: &ExampleName,
    output: &mut dyn Write,
) -> Result<u8, Error> {
    let runner = SystemRunner::new(Programs::new(root));
    run(
        &runner,
        RegisteredOperation::Check,
        package,
        example,
        output,
    )
}

pub(crate) fn write_debt(
    root: &Path,
    package: &Package,
    example: &ExampleName,
    output: &mut dyn Write,
) -> Result<u8, Error> {
    let runner = SystemRunner::new(Programs::new(root));
    run(
        &runner,
        RegisteredOperation::WriteDebt,
        package,
        example,
        output,
    )
}

pub(crate) fn index(
    root: &Path,
    package: &Package,
    example: &ExampleName,
    format: RdfFormat,
) -> Result<String, Error> {
    index_with(
        &SystemRunner::new(Programs::new(root)),
        package,
        example,
        format,
    )
}

fn index_with(
    runner: &dyn Runner,
    package: &Package,
    example: &ExampleName,
    format: RdfFormat,
) -> Result<String, Error> {
    let (captured, context) = capture(runner, RegisteredOperation::Index, package, example)?;
    let context = diagnostics(&context, &captured.stderr);
    if captured.code != Some(0) {
        return Err(Error::Transport(format!(
            "{context}: index target failed with exit {:?}",
            captured.code
        )));
    }
    let text = std::str::from_utf8(&captured.stdout).map_err(|source| input(&context, source))?;
    let report = Index::from_ntriples(text).map_err(|source| input(&context, source))?;
    if report.normal_form != NORMAL_FORM {
        return Err(Error::Transport(format!(
            "{context}: index normal form {} is unsupported; expected {NORMAL_FORM}",
            report.normal_form
        )));
    }
    let canonical = report
        .to_ntriples()
        .map_err(|source| input(&context, source))?;
    if canonical != text {
        return Err(Error::Transport(format!(
            "{context}: index response must be canonical N-Triples 1.2"
        )));
    }
    match format {
        RdfFormat::Ntriples => Ok(canonical),
        RdfFormat::Turtle => report.to_turtle().map_err(|source| input(&context, source)),
    }
}

fn capture(
    runner: &dyn Runner,
    operation: RegisteredOperation,
    package: &Package,
    example: &ExampleName,
) -> Result<(Output, String), Error> {
    let (name, invocation) = match operation {
        RegisteredOperation::Check => ("check", Invocation::example_run(package, example)),
        RegisteredOperation::Index => ("index", Invocation::example_index(package, example)),
        RegisteredOperation::WriteDebt => (
            "debt write",
            Invocation::example_debt_write(package, example),
        ),
    };
    let context = format!("registered {name} {package}/{example}");
    let captured = runner
        .run(
            &invocation,
            &Limits::build().with_max_output(MAX_CHECK_RESPONSE_BYTES),
        )
        .map_err(|source| input(&context, source))?;
    Ok((captured, context))
}

fn run(
    runner: &dyn Runner,
    operation: RegisteredOperation,
    package: &Package,
    example: &ExampleName,
    output: &mut dyn Write,
) -> Result<u8, Error> {
    let (captured, context) = capture(runner, operation, package, example)?;
    if !matches!(captured.code, Some(0 | 1)) {
        return Err(Error::Transport(format!(
            "{context}: target failed with exit {:?}; stderr: {}",
            captured.code,
            captured.stderr.escape_ascii()
        )));
    }
    let text = std::str::from_utf8(&captured.stdout)
        .map_err(|source| input(&diagnostics(&context, &captured.stderr), source))?;
    if operation == RegisteredOperation::WriteDebt {
        let response = RegisteredDebtWrite::from_ntriples(text)
            .map_err(|source| input(&diagnostics(&context, &captured.stderr), source))?;
        agreement(
            &context,
            &captured.stderr,
            captured.code,
            response.exit_code(),
        )?;
        if let Some(error) = response.error_message() {
            return Err(Error::Transport(format!(
                "{context}: repository publication failed: {}",
                error.escape_debug()
            )));
        }
        let (groups, jobs) = response
            .counts()
            .expect("successful validated write receipt");
        writeln!(
            output,
            "census: debt written: {groups} groups, {jobs} jobs with copies"
        )
        .map_err(|source| input("<stdout>", source))?;
        return Ok(0);
    }
    let response = RegisteredCheck::from_ntriples(text)
        .map_err(|source| input(&diagnostics(&context, &captured.stderr), source))?;
    agreement(
        &context,
        &captured.stderr,
        captured.code,
        response.exit_code(),
    )?;
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

fn agreement(context: &str, stderr: &[u8], code: Option<i32>, expected: u8) -> Result<(), Error> {
    if code != Some(i32::from(expected)) {
        return Err(Error::Transport(format!(
            "{}: target exit {code:?} disagrees with protocol outcome {expected}",
            diagnostics(context, stderr),
        )));
    }
    Ok(())
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
