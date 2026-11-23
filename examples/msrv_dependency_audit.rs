// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Preserve Cargo's target inventories and check unused dependencies by package.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, BufRead as _, Write as _};
use std::path::Path;
use std::process::{Command, ExitCode, ExitStatus};

use purrdf_core::ContentDigest;
use purrdf_lex::json::{self, Value};

const INVENTORY: &str = "RAF compiler externs:";
const UNUSED: &str = "unused_crate_dependencies";
type Counts = BTreeMap<String, usize>;
type TargetKey = (String, String, Vec<String>);
type Targets = BTreeMap<TargetKey, usize>;

#[derive(Default)]
struct Package {
    provided: Counts,
    unused: Counts,
    libraries: BTreeSet<String>,
    declared: Option<BTreeSet<String>>,
    units: BTreeSet<String>,
    inventoried: Targets,
    artifacts: Targets,
}

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    match arguments.as_slice() {
        [command, root, launcher @ ..] if command == "--check" => {
            let script = Path::new(env!("CARGO_MANIFEST_DIR")).join(".githooks/check-msrv");
            let status = Command::new(script)
                .arg(root)
                .arg(std::env::current_exe()?)
                .args(launcher)
                .status()?;
            Ok(exit_code(status))
        }
        [command, metadata] if command == "--report" => {
            report(metadata)?;
            Ok(ExitCode::SUCCESS)
        }
        [command] if command == "--identity" => {
            let binary = fs::read(std::env::current_exe()?)?;
            writeln!(io::stdout().lock(), "{}", ContentDigest::of(&binary))?;
            Ok(ExitCode::SUCCESS)
        }
        [command, rest @ ..] => compile(command, rest),
        [] => Err("supply --check, --report, --identity or a compiler invocation".into()),
    }
}

fn compile(command: &OsStr, arguments: &[OsString]) -> Result<ExitCode, Box<dyn Error>> {
    if arguments
        .iter()
        .any(|arg| arg.to_string_lossy().starts_with("--emit="))
    {
        let unit = arguments
            .windows(2)
            .filter(|pair| pair[0] == "-C")
            .filter_map(|pair| pair[1].to_str())
            .find_map(|option| option.strip_prefix("metadata="))
            .filter(|unit| !unit.is_empty() && unit.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .ok_or("compiler invocation lacks a Cargo unit identity")?;
        let mut names = BTreeSet::new();
        for pair in arguments.windows(2).filter(|pair| pair[0] == "--extern") {
            let input = pair[1].to_string_lossy();
            let (name, _) = input
                .split_once('=')
                .ok_or("compiler --extern lacks an artifact path")?;
            let name = name
                .rsplit(':')
                .next()
                .ok_or("compiler --extern lacks a name")?;
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                return Err(format!("unsupported compiler extern name {name:?}").into());
            }
            names.insert(name.to_owned());
        }
        let names = names.into_iter().collect::<Vec<_>>().join(" ");
        // Cargo preserves the note with the matching target diagnostics, including on cache hits.
        writeln!(
            io::stderr().lock(),
            r#"{{"$message_type":"diagnostic","message":"{INVENTORY} {unit} {names}","code":null,"level":"note","spans":[],"children":[],"rendered":null}}"#
        )?;
    }
    let status = Command::new(command).args(arguments).status()?;
    Ok(exit_code(status))
}

fn exit_code(status: ExitStatus) -> ExitCode {
    ExitCode::from(
        status
            .code()
            .and_then(|code| u8::try_from(code).ok())
            .unwrap_or(1),
    )
}

fn report(metadata: &OsStr) -> Result<(), Box<dyn Error>> {
    let metadata = json::read(&fs::read_to_string(metadata)?)?;
    let members = metadata
        .get("workspace_members")
        .and_then(Value::as_array)
        .filter(|members| !members.is_empty())
        .ok_or("Cargo supplied no workspace members")?;
    let mut packages = BTreeMap::new();
    for member in members {
        let id = member
            .as_str()
            .ok_or("invalid Cargo workspace member identity")?;
        if packages.insert(id.to_owned(), Package::default()).is_some() {
            return Err(format!("duplicate Cargo workspace member {id}").into());
        }
    }
    let declarations = metadata
        .get("packages")
        .and_then(Value::as_array)
        .ok_or("Cargo supplied no package declarations")?;
    for declaration in declarations {
        let id = field(declaration, "id")?;
        if let Some(package) = packages.get_mut(id) {
            let dependencies = declaration
                .get("dependencies")
                .and_then(Value::as_array)
                .ok_or("missing Cargo dependency declarations")?;
            let mut aliases = BTreeSet::new();
            for dependency in dependencies {
                let name = dependency
                    .get("rename")
                    .and_then(Value::as_str)
                    .map_or_else(|| field(dependency, "name"), Ok)?;
                aliases.insert(name.replace('-', "_"));
            }
            package.declared = Some(aliases);
        }
    }
    let mut finished = false;
    for line in io::stdin().lock().lines() {
        let message = json::read(&line?)?;
        match field(&message, "reason")? {
            "compiler-message" | "compiler-artifact" => record(&mut packages, &message)?,
            "build-finished" => {
                finished = matches!(message.get("success"), Some(Value::Bool(true)));
            }
            _ => {}
        }
    }
    if !finished {
        return Err("Cargo supplied no successful build-finished result".into());
    }
    verify(packages)
}

fn record(packages: &mut BTreeMap<String, Package>, message: &Value) -> Result<(), Box<dyn Error>> {
    let package = packages.get_mut(field(message, "package_id")?);
    if let Some(value) = message.get("message") {
        return diagnostic(package, message, value);
    }
    let Some(package) = package else {
        return Ok(());
    };
    let target = message
        .get("target")
        .ok_or("missing compiler artifact target")?;
    let key = target_key(message)?;
    if target
        .get("kind")
        .and_then(Value::as_array)
        .is_some_and(|kinds| {
            kinds
                .iter()
                .any(|kind| matches!(kind.as_str(), Some("lib" | "rlib" | "dylib" | "proc-macro")))
        })
    {
        package.libraries.insert(key.1.replace('-', "_"));
    }
    *package.artifacts.entry(key).or_default() += 1;
    Ok(())
}

fn target_key(message: &Value) -> Result<TargetKey, Box<dyn Error>> {
    let target = message.get("target").ok_or("missing compiler target")?;
    let kinds = target
        .get("kind")
        .and_then(Value::as_array)
        .ok_or("missing Cargo target kinds")?
        .iter()
        .map(|kind| {
            kind.as_str()
                .map(str::to_owned)
                .ok_or("invalid Cargo target kind")
        })
        .collect::<Result<_, _>>()?;
    Ok((
        field(target, "src_path")?.to_owned(),
        field(target, "name")?.to_owned(),
        kinds,
    ))
}

fn diagnostic(
    package: Option<&mut Package>,
    message: &Value,
    value: &Value,
) -> Result<(), Box<dyn Error>> {
    let text = field(value, "message")?;
    let code = value
        .get("code")
        .and_then(|code| code.get("code"))
        .and_then(Value::as_str);
    if let Some(names) = text.strip_prefix(INVENTORY) {
        let package = package.ok_or("extern inventory from outside the workspace")?;
        inventory(package, message, value, names)?;
    } else if code == Some(UNUSED) {
        let Some(package) = package else {
            return Ok(());
        };
        let name = text
            .strip_prefix("extern crate `")
            .and_then(|tail| tail.split_once("` is unused in crate `"))
            .map(|(name, _)| name)
            .ok_or("unrecognized compiler unused-dependency diagnostic")?;
        *package.unused.entry(name.to_owned()).or_default() += 1;
    } else if let Some(rendered) = value.get("rendered").and_then(Value::as_str) {
        io::stderr().lock().write_all(rendered.as_bytes())?;
    }
    Ok(())
}

fn inventory(
    package: &mut Package,
    message: &Value,
    value: &Value,
    names: &str,
) -> Result<(), Box<dyn Error>> {
    if field(value, "level")? != "note"
        || !matches!(value.get("code"), Some(Value::Null))
        || !matches!(value.get("rendered"), Some(Value::Null))
        || !["spans", "children"].iter().all(|key| {
            value
                .get(key)
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty)
        })
    {
        return Err("malformed Cargo compiler inventory".into());
    }
    let mut words = names.split_ascii_whitespace();
    let unit = words.next().ok_or("missing Cargo compiler unit identity")?;
    if !unit.bytes().all(|byte| byte.is_ascii_hexdigit()) || !package.units.insert(unit.to_owned())
    {
        return Err(format!("invalid or repeated Cargo compiler unit {unit}").into());
    }
    *package.inventoried.entry(target_key(message)?).or_default() += 1;
    let mut seen = BTreeSet::new();
    for name in words {
        if !seen.insert(name)
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(format!("invalid or repeated compiler extern {name}").into());
        }
        *package.provided.entry(name.to_owned()).or_default() += 1;
    }
    Ok(())
}

fn verify(packages: BTreeMap<String, Package>) -> Result<(), Box<dyn Error>> {
    let mut findings = Vec::new();
    for (id, package) in packages {
        let declared = package
            .declared
            .ok_or("missing Cargo workspace member declaration")?;
        if let Some(ambiguous) = package.libraries.intersection(&declared).next() {
            return Err(format!(
                "{id}: dependency {ambiguous} shares the package library's import name"
            )
            .into());
        }
        if package.artifacts.is_empty() || package.artifacts != package.inventoried {
            return Err(format!(
                "{id}: compiler extern inventory does not cover its Cargo artifacts"
            )
            .into());
        }
        for (name, unused) in package.unused {
            let provided = package.provided.get(&name).copied().unwrap_or_default();
            if unused > provided {
                return Err(format!("{id}: inconsistent compiler inventory for {name}").into());
            }
            if unused == provided && !package.libraries.contains(&name) {
                findings.push(format!(
                    "{id}: dependency {name} is unused in every target that receives it"
                ));
            }
        }
    }
    if findings.is_empty() {
        Ok(())
    } else {
        Err(findings.join("\n").into())
    }
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing Cargo JSON field {key}").into())
}
