// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native Cargo-launched driver; all subprocesses use the existing bounded runner.

use super::super::{DiagramError, Library, refused};
use super::{
    canonical_options, executable_identity,
    receipt::{MAX_ITEMS, Observation},
};
use crate::exec::{
    DEFAULT_OUTPUT_BOUND, Invocation, Limits, Output, Programs, Runner, RustdocObservationOptions,
    SystemRunner,
};
use purrdf_core::ContentDigest;
use purrdf_lex::{
    allocation::{Memory, Resident},
    walk::try_sort_unstable_by,
};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

mod arguments;
pub(in crate::diagram::cargo) use arguments::parse as parse_input;
#[cfg(test)]
mod tests;

pub(super) fn run() -> Result<i32, DiagramError> {
    let controls = controls()?;
    let driver = executable_identity(controls.driver())?;
    let delegate = executable_identity(controls.delegate())?;
    if driver == delegate
        || executable_identity(&std::env::current_exe().map_err(DiagramError::data)?)? != driver
    {
        return Err(refused(
            "Rustdoc observer must execute the selected driver and a distinct delegate",
        ));
    }
    let arguments = arguments()?;
    let directory = std::env::current_dir().map_err(DiagramError::data)?;
    let runner = SystemRunner::new(Programs::new(&directory));
    if version_probe(&arguments) {
        let output = delegate_run(&runner, &controls, &directory, &arguments, &Limits::query())?;
        forward(&output)?;
        return Ok(output.code.unwrap_or(2));
    }
    let cargo_environment = cargo_environment()?;
    let compiler_environment = compiler_environment(&cargo_environment, &controls)?;
    let query_arguments = query_arguments(&arguments)?;
    let configuration = delegate_run(
        &runner,
        &controls,
        &directory,
        &query_arguments,
        &Limits::query(),
    )?;
    if !configuration.success() {
        forward(&configuration)?;
        return Err(refused(&format!(
            "selected Rustdoc configuration query failed with {:?}",
            configuration.code
        )));
    }
    let cfg = std::str::from_utf8(&configuration.stdout).map_err(DiagramError::data)?;
    if !cfg.lines().any(|line| line == "doc") {
        return Err(refused(
            "selected compiler did not report native Rustdoc doc configuration",
        ));
    }
    let pending = if controls.assembly().is_some() {
        let version = runner
            .run(
                &Invocation::rustdoc_compiler_version(&controls, &directory)
                    .map_err(DiagramError::data)?,
                &Limits::query(),
            )
            .map_err(DiagramError::data)?;
        Some(super::super::assembly::Pending::before_html(
            &controls,
            &directory,
            &arguments,
            &cargo_environment,
            &configuration,
            version,
        )?)
    } else {
        None
    };
    let html_arguments = pending.as_ref().map_or(arguments.as_slice(), |pending| {
        pending.preparation.html_arguments()
    });
    let html = delegate_run(
        &runner,
        &controls,
        &directory,
        html_arguments,
        &Limits::build(),
    )?;
    forward(&html)?;
    if !html.success() {
        return Ok(html.code.unwrap_or(2));
    }
    let assembly = pending
        .map(super::super::assembly::Pending::finish)
        .transpose()?;
    if executable_identity(controls.driver())? != driver
        || executable_identity(controls.delegate())? != delegate
    {
        return Err(refused(
            "Rustdoc executable identity changed during the driver operation",
        ));
    }
    let observed = Observation {
        driver: locator(controls.driver())?,
        delegate: locator(controls.delegate())?,
        directory: locator(&directory)?,
        arguments,
        query_arguments,
        cargo_environment,
        compiler_environment,
        configuration,
        html,
        assembly,
    };
    publish(&controls, &observed)?;
    Ok(0)
}

fn controls() -> Result<RustdocObservationOptions, DiagramError> {
    let path = |key| {
        std::env::var_os(key)
            .map(PathBuf::from)
            .ok_or_else(|| refused("required Rustdoc observer control is absent"))
    };
    let mut options = RustdocObservationOptions::new(
        &path("RUSTDOC")?,
        &path(RustdocObservationOptions::DELEGATE)?,
        &path(RustdocObservationOptions::RECEIPTS)?,
    )
    .map_err(DiagramError::data)?;
    match (
        std::env::var_os(RustdocObservationOptions::ASSEMBLY),
        std::env::var_os(RustdocObservationOptions::ASSEMBLY_DIGEST),
    ) {
        (None, None) => {}
        (Some(plan), Some(digest)) => {
            let digest = digest
                .to_str()
                .ok_or_else(|| refused("assembly digest must be UTF-8"))?;
            let digest = ContentDigest::from_hex(digest)
                .ok_or_else(|| refused("assembly digest must be native SHA-256"))?;
            options = options
                .with_assembly(Path::new(&plan), digest)
                .map_err(DiagramError::data)?;
        }
        _ => {
            return Err(refused(
                "assembly controls must supply both exact plan and digest",
            ));
        }
    }
    let canonical = canonical_options(&options)?;
    if canonical != options {
        return Err(refused(
            "Rustdoc driver controls differ from their admitted locators",
        ));
    }
    Ok(options)
}

fn locator(path: &Path) -> Result<String, DiagramError> {
    let text = path
        .to_str()
        .ok_or_else(|| refused("Rustdoc observation locators must be UTF-8"))?;
    let mut resident = Resident;
    Memory::new(&mut resident)
        .string(text)
        .map_err(DiagramError::data)
}

fn arguments() -> Result<Vec<String>, DiagramError> {
    let mut args = Vec::new();
    let mut remaining = DEFAULT_OUTPUT_BOUND;
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    for argument in std::env::args_os().skip(1) {
        let text = argument
            .to_str()
            .ok_or_else(|| refused("Rustdoc argument is not UTF-8"))?;
        remaining = remaining
            .checked_sub(text.len())
            .ok_or_else(|| refused("Rustdoc arguments exceed their byte bound"))?;
        if args.len() == MAX_ITEMS {
            return Err(refused("too many Rustdoc arguments"));
        }
        let text = memory.string(text).map_err(DiagramError::data)?;
        memory.push(&mut args, text).map_err(DiagramError::data)?;
    }
    Ok(args)
}

fn version_probe(arguments: &[String]) -> bool {
    !arguments.is_empty()
        && arguments.iter().all(|argument| {
            matches!(
                argument.as_str(),
                "-V" | "--version" | "-vV" | "-v" | "--verbose"
            )
        })
}

pub(super) fn query_arguments(arguments: &[String]) -> Result<Vec<String>, DiagramError> {
    if arguments.len() > MAX_ITEMS - 3 {
        return Err(refused(
            "Rustdoc observation arguments leave no room for the bounded configuration query",
        ));
    }
    let admitted = arguments::parse(arguments)?;
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut query = Vec::new();
    let separator = admitted.terminator.unwrap_or(arguments.len());
    for argument in arguments[..separator]
        .iter()
        .map(String::as_str)
        .chain(["-Z", "unstable-options", "--print=cfg"])
        .chain(arguments[separator..].iter().map(String::as_str))
    {
        let text = memory.string(argument).map_err(DiagramError::data)?;
        memory.push(&mut query, text).map_err(DiagramError::data)?;
    }
    Ok(query)
}

fn cargo_environment() -> Result<Vec<(String, String)>, DiagramError> {
    let mut environment = Vec::new();
    let mut remaining = DEFAULT_OUTPUT_BOUND;
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    for (key, value) in std::env::vars_os() {
        let key = key
            .to_str()
            .ok_or_else(|| refused("Rustdoc environment name is not UTF-8"))?;
        let value = value
            .to_str()
            .ok_or_else(|| refused("Rustdoc environment value is not UTF-8"))?;
        remaining = remaining
            .checked_sub(key.len().saturating_add(value.len()))
            .ok_or_else(|| refused("Rustdoc environment exceeds its byte bound"))?;
        if environment.len() == MAX_ITEMS {
            return Err(refused("too many Rustdoc environment entries"));
        }
        let pair = (
            memory.string(key).map_err(DiagramError::data)?,
            memory.string(value).map_err(DiagramError::data)?,
        );
        memory
            .push(&mut environment, pair)
            .map_err(DiagramError::data)?;
    }
    sort_environment(&mut environment)?;
    Ok(environment)
}

fn sort_environment(environment: &mut [(String, String)]) -> Result<(), DiagramError> {
    try_sort_unstable_by(environment, |left, right| {
        Ok::<_, std::convert::Infallible>(left.0.cmp(&right.0))
    })
    .unwrap_or_else(|never| match never {});
    if environment.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(refused("Rustdoc environment repeats a variable name"));
    }
    Ok(())
}

pub(super) fn compiler_environment(
    environment: &[(String, String)],
    controls: &RustdocObservationOptions,
) -> Result<Vec<(String, String)>, DiagramError> {
    for (key, value) in controls.environment() {
        if environment
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
            != value.to_str()
        {
            return Err(refused(
                "Cargo Rustdoc environment disagrees with its scoped observer controls",
            ));
        }
    }
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut result = Vec::new();
    for (key, value) in environment {
        if matches!(
            key.as_str(),
            RustdocObservationOptions::MARKER
                | RustdocObservationOptions::DELEGATE
                | RustdocObservationOptions::RECEIPTS
                | RustdocObservationOptions::ASSEMBLY
                | RustdocObservationOptions::ASSEMBLY_DIGEST
        ) {
            continue;
        }
        let value = if key == "RUSTDOC" {
            controls
                .delegate()
                .to_str()
                .ok_or_else(|| refused("Rustdoc delegate must be UTF-8"))?
        } else {
            value
        };
        let pair = (
            memory.string(key).map_err(DiagramError::data)?,
            memory.string(value).map_err(DiagramError::data)?,
        );
        memory.push(&mut result, pair).map_err(DiagramError::data)?;
    }
    Ok(result)
}

fn delegate_run(
    runner: &dyn Runner,
    controls: &RustdocObservationOptions,
    directory: &Path,
    arguments: &[String],
    limits: &Limits,
) -> Result<Output, DiagramError> {
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut argv = Vec::new();
    for argument in std::iter::once(
        controls
            .delegate()
            .to_str()
            .ok_or_else(|| refused("Rustdoc delegate must be UTF-8"))?,
    )
    .chain(arguments.iter().map(String::as_str))
    {
        let text = memory.string(argument).map_err(DiagramError::data)?;
        memory.push(&mut argv, text).map_err(DiagramError::data)?;
    }
    runner
        .run(
            &Invocation::rustdoc_delegate(controls, &argv, directory),
            limits,
        )
        .map_err(DiagramError::data)
}

fn forward(output: &Output) -> Result<(), DiagramError> {
    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    stdout
        .write_all(&output.stdout)
        .and_then(|()| stdout.flush())
        .map_err(DiagramError::data)?;
    stderr
        .write_all(&output.stderr)
        .and_then(|()| stderr.flush())
        .map_err(DiagramError::data)
}

fn publish(
    controls: &RustdocObservationOptions,
    observed: &Observation,
) -> Result<(), DiagramError> {
    let text = super::receipt::encode(observed)?;
    if text.len() > DEFAULT_OUTPUT_BOUND {
        return Err(refused("native Rustdoc receipt exceeds its byte bound"));
    }
    let path = controls
        .receipts()
        .join(format!("{}.nt", ContentDigest::of(text.as_bytes())));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(DiagramError::data)?;
    file.write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(DiagramError::data)
}

pub(super) fn matches_library(
    observed: &Observation,
    library: &Library,
) -> Result<bool, DiagramError> {
    matches_inputs(
        &observed.arguments,
        &observed.cargo_environment,
        Path::new(&observed.directory),
        library,
    )
}

pub(in crate::diagram::cargo) fn matches_inputs(
    arguments: &[String],
    environment: &[(String, String)],
    directory: &Path,
    library: &Library,
) -> Result<bool, DiagramError> {
    let get = |name| {
        environment
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    };
    if get("CARGO_PKG_NAME") != Some(library.package.as_str())
        || get("CARGO_PKG_VERSION") != Some(library.version.as_str())
        || arguments::parse(arguments)?.crate_name != library.name
    {
        return Ok(false);
    }
    let manifest = get("CARGO_MANIFEST_DIR")
        .ok_or_else(|| refused("Rustdoc Cargo environment has no manifest directory"))?;
    if !Path::new(manifest).is_absolute()
        || Path::new(manifest)
            .join("Cargo.toml")
            .canonicalize()
            .map_err(DiagramError::data)?
            != library
                .manifest
                .canonicalize()
                .map_err(DiagramError::data)?
    {
        return Ok(false);
    }
    let source = library.source.canonicalize().map_err(DiagramError::data)?;
    let candidate = directory.join(arguments::parse(arguments)?.source);
    Ok(candidate.canonicalize().map_err(DiagramError::data)? == source)
}
