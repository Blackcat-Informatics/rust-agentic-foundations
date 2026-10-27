// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `rustc` as the oracle for name resolution: compile a [`Program`] and say,
//! for each probe, whether the compiler accepts the `use` and which item it
//! names.
//!
//! The first compile holds every probe and reads each verdict from the line
//! of its error. Declarations that rustc itself refuses are dropped and the
//! crate is compiled again until it is valid, so a probe is judged in a crate
//! the compiler accepts apart from the probes. The second compile holds the
//! accepted probes and, under each, one assertion for every item of the
//! program: the assertion `const _: () = assert!(probe::TAG == k);` fails
//! to compile unless the probe names the item tagged `k`.
//!
//! A probe is also asked one name space at a time, for the order check: its
//! path as a type alias and as the expression of a function body, with no
//! evaluation of either.

use std::collections::{BTreeMap, BTreeSet};
use std::process::{Command, Stdio};

use super::Scratch;
use super::program::{Decl, Owner, Program, Space};
use purrdf_lex::json::{self, Value};

/// The edition both sides read the crate under.
pub(crate) const EDITION: &str = "2021";

/// The lints about glob ambiguity, which rustc raises as warnings: a program
/// they fire on is accepted, and the warning is kept to classify.
const GLOB_LINTS: [&str; 2] = ["ambiguous_glob_reexports", "ambiguous_glob_imports"];

/// The lint groups allowed so that the diagnostics hold errors and the
/// glob-ambiguity warnings only.
const QUIET: [&str; 3] = ["unused", "dead_code", "nonstandard_style"];

/// The most rounds of dropping declarations rustc refuses.
const MOST_ROUNDS: usize = 16;

/// One diagnostic of a compile.
#[derive(Debug, Clone)]
pub(crate) struct Diagnostic {
    pub(crate) level: String,
    /// The error code, or the lint name of a warning.
    pub(crate) code: Option<String>,
    /// The message and every label and note, for matching names in them.
    pub(crate) text: String,
    /// The line of the primary span.
    pub(crate) line: Option<usize>,
}

/// The message of the diagnostic `rustc` gives for an import it cannot settle
/// because the imports and globs it depends on lead back to it. It has no
/// error code.
const UNDETERMINABLE_MESSAGE: &str = "cannot determine resolution for the import";

/// The code a [`Verdict::Rejected`] holds for the diagnostic of
/// [`UNDETERMINABLE_MESSAGE`].
pub(crate) const UNDETERMINABLE: &str = "undeterminable-import";

/// The program `rustc` is run as; `RUSTC` when set, as cargo sets it.
fn tool() -> std::ffi::OsString {
    std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into())
}

/// The version text of the compiler, for the report.
pub(crate) fn version() -> String {
    let output = Command::new(tool())
        .arg("-vV")
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("the tool `rustc` cannot start: {e}"));
    assert!(
        output.status.success(),
        "the tool `rustc` failed to report its version"
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.starts_with("rustc ") || line.starts_with("release:"))
        .collect::<Vec<_>>()
        .join("; ")
}

fn texts(diagnostic: &Value, into: &mut String) {
    if let Some(message) = diagnostic.get("message").and_then(Value::as_str) {
        into.push_str(message);
        into.push('\n');
    }
    for span in diagnostic
        .get("spans")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
    {
        if let Some(label) = span.get("label").and_then(Value::as_str) {
            into.push_str(label);
            into.push('\n');
        }
    }
    for child in diagnostic
        .get("children")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
    {
        texts(child, into);
    }
}

pub(crate) fn diagnostic_level(value: &Value) -> &str {
    value
        .get("level")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("rustc diagnostic requires a string level: {value}"))
}

/// Compile `source` as a library root and return its diagnostics.
pub(crate) fn compile(source: &str) -> Vec<Diagnostic> {
    let scratch = Scratch::new();
    let file = scratch.file("lib.rs", source);
    let mut command = Command::new(tool());
    command
        .current_dir(scratch.base())
        .args(["--edition", EDITION, "--crate-type", "lib"])
        .args(["--crate-name", "demo", "--emit=metadata"])
        .arg("-o")
        .arg(scratch.path("out.rmeta"))
        .args(["--error-format=json"]);
    // `-A warnings` would silence the lints raised below as well, so only the
    // lint groups that the generated names and unused items trigger are
    // allowed.
    for group in QUIET {
        command.args(["-A", group]);
    }
    for lint in GLOB_LINTS {
        command.args(["-W", lint]);
    }
    let output = command
        .arg(&file)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("the tool `rustc` cannot start: {e}"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut found = Vec::new();
    for line in stderr.lines().filter(|line| !line.trim().is_empty()) {
        let value = json::read(line)
            .unwrap_or_else(|e| panic!("rustc printed a line that is not JSON ({e}): {line}"));
        let mut text = String::new();
        texts(&value, &mut text);
        let line_of = value
            .get("spans")
            .and_then(Value::as_array)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .find(|span| matches!(span.get("is_primary"), Some(Value::Bool(true))))
            .and_then(|span| span.get("line_start"))
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok());
        found.push(Diagnostic {
            level: diagnostic_level(&value).to_owned(),
            code: value
                .get("code")
                .and_then(|code| code.get("code"))
                .and_then(Value::as_str)
                .map(str::to_owned),
            text,
            line: line_of,
        });
    }
    assert!(
        output.status.success() || errors(&found).next().is_some(),
        "rustc failed without a diagnostic: {}: {stderr}",
        output.status
    );
    found
}

/// What `rustc` says of one probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// The `use` is refused, with the codes of its errors.
    Rejected(Vec<String>),
    /// The `use` is accepted, naming the item of each name space it binds, by
    /// tag.
    Accepted { ty: Option<u32>, value: Option<u32> },
}

/// A warning about glob ambiguity.
#[derive(Debug, Clone)]
pub(crate) struct Warning {
    pub(crate) lint: String,
    pub(crate) text: String,
}

/// What `rustc` said of a program.
#[derive(Debug)]
pub(crate) struct Judged {
    /// The program that was judged: the input without the declarations rustc
    /// refuses.
    pub(crate) program: Program,
    /// The verdict on each probe, in order.
    pub(crate) verdicts: Vec<Verdict>,
    pub(crate) warnings: Vec<Warning>,
    /// How many declarations were dropped.
    pub(crate) dropped: usize,
}

fn errors(diagnostics: &[Diagnostic]) -> impl Iterator<Item = &Diagnostic> {
    diagnostics.iter().filter(|d| d.level == "error")
}

/// An error of a compile that no line of the program owns, which is a fault
/// of the harness and not a verdict.
fn unowned(diagnostic: &Diagnostic) -> bool {
    diagnostic.line.is_none() && !diagnostic.text.starts_with("aborting due to")
}

/// The code of an error: its error code, [`UNDETERMINABLE`] for the
/// diagnostic of an import `rustc` cannot settle, and otherwise empty.
fn code_of(error: &Diagnostic) -> String {
    error.code.clone().unwrap_or_else(|| {
        if error.text.starts_with(UNDETERMINABLE_MESSAGE) {
            UNDETERMINABLE.to_owned()
        } else {
            String::new()
        }
    })
}

/// Judge every probe of `program` with `rustc`. `None` when the crate holds
/// an error on a line that cannot be dropped.
pub(crate) fn judge(program: &Program) -> Option<Judged> {
    let mut program = program.clone();
    let mut dropped = 0;
    let all: Vec<usize> = (0..program.probes.len()).collect();
    for _ in 0..MOST_ROUNDS {
        let rendered = program.render(&all, &BTreeMap::new());
        if !all
            .iter()
            .all(|at| rendered.owners.contains(&Owner::Probe(*at)))
        {
            // A probe whose module is gone is not a question.
            return None;
        }
        let diagnostics = compile(&rendered.source);
        let mut refused: Vec<usize> = Vec::new();
        let mut rejected: BTreeMap<usize, Vec<String>> = BTreeMap::new();
        for error in errors(&diagnostics) {
            assert!(
                !unowned(error),
                "rustc reported an error with no line: {}\n{}",
                error.text,
                rendered.source
            );
            let Some(line) = error.line else { continue };
            match rendered.owners.get(line - 1) {
                Some(Owner::Decl(at)) => refused.push(*at),
                Some(Owner::Probe(probe)) => {
                    rejected.entry(*probe).or_default().push(code_of(error));
                }
                Some(Owner::Frame | Owner::Assert(..)) | None => return None,
            }
        }
        if !refused.is_empty() {
            refused.sort_unstable();
            refused.dedup();
            for at in refused.iter().rev() {
                program.lines.remove(*at);
                dropped += 1;
            }
            continue;
        }
        let warnings = diagnostics
            .iter()
            .filter(|d| d.level == "warning")
            .filter_map(|d| {
                let lint = d.code.as_deref()?;
                GLOB_LINTS.contains(&lint).then(|| Warning {
                    lint: lint.to_owned(),
                    text: d.text.clone(),
                })
            })
            .collect();
        let verdicts = match identify(&program, &rendered_accepts(&all, &rejected), rejected) {
            Ok(verdicts) => verdicts,
            Err(mut refused) => {
                // Errors that the probes' own errors held back.
                refused.sort_unstable();
                refused.dedup();
                for at in refused.iter().rev() {
                    program.lines.remove(*at);
                    dropped += 1;
                }
                continue;
            }
        };
        return Some(Judged {
            program,
            verdicts,
            warnings,
            dropped,
        });
    }
    None
}

/// What `rustc` says of a path in one name space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpaceVerdict {
    /// The path names an item.
    Resolves,
    /// The path is refused as ambiguous (E0659).
    Ambiguous,
    /// The original named import of the probed local name is ambiguous.
    ImportAmbiguous,
    /// The path names nothing, for any other reason.
    Unresolved,
}

/// What `rustc` says of the path of the probe of `program`, which holds one,
/// in the type name space and in the value name space, by one compile and
/// without naming what it resolves to. The path is written as a type alias and
/// as the expression of a function body, so that each name space is asked
/// alone, and the verdict is read from the errors on the line of each. Errors
/// elsewhere in the crate are not the probe's. Used on a program `rustc` has
/// judged, declared in another order, to see whether the order decides what it
/// answers.
pub(crate) fn space_verdicts(program: &Program) -> Option<[(Space, SpaceVerdict); 2]> {
    let split = program.render_by_space(0);
    let diagnostics = compile(&split.rendered.source);
    let mut import_ambiguous = false;
    for error in errors(&diagnostics) {
        assert!(!unowned(error), "an unowned compiler error: {error:?}");
        let Some(line) = error.line else {
            continue;
        };
        if line == split.type_line || line == split.value_line {
            continue;
        }
        if error.code.as_deref() == Some("E0659")
            && let Some(Owner::Decl(at)) = split.rendered.owners.get(line - 1)
            && program.probe_import_at(*at)
        {
            import_ambiguous = true;
        } else {
            return None;
        }
    }
    let verdict = |line: usize| {
        let codes: Vec<Option<&str>> = errors(&diagnostics)
            .filter(|error| error.line == Some(line))
            .map(|error| error.code.as_deref())
            .collect();
        if import_ambiguous {
            SpaceVerdict::ImportAmbiguous
        } else if codes.is_empty() {
            SpaceVerdict::Resolves
        } else if codes.contains(&Some("E0659")) {
            SpaceVerdict::Ambiguous
        } else {
            SpaceVerdict::Unresolved
        }
    };
    Some([
        (Space::Type, verdict(split.type_line)),
        (Space::Value, verdict(split.value_line)),
    ])
}

fn rendered_accepts(all: &[usize], rejected: &BTreeMap<usize, Vec<String>>) -> Vec<usize> {
    all.iter()
        .copied()
        .filter(|at| !rejected.contains_key(at))
        .collect()
}

/// The assertion that the probe `at` names the item `decl`.
fn assertion(at: usize, decl: &Decl) -> String {
    match decl.space {
        Space::Type => format!("const _: () = assert!(probe{at}::TAG == {});", decl.tag),
        Space::Value => format!("const _: () = assert!(probe{at}() == {});", decl.tag),
    }
}

/// The verdicts, naming the item of each accepted probe by a second compile.
/// `Err` holds the lines of declarations that the compile refuses and the
/// first one did not report.
fn identify(
    program: &Program,
    accepted: &[usize],
    mut rejected: BTreeMap<usize, Vec<String>>,
) -> Result<Vec<Verdict>, Vec<usize>> {
    let decls = program.decls();
    // A rename binds another name to an item, so every item is a candidate.
    let candidates: Vec<&Decl> = decls.iter().collect();
    let mut accepted = accepted.to_vec();
    loop {
        let asserts: BTreeMap<usize, Vec<String>> = accepted
            .iter()
            .map(|at| {
                (
                    *at,
                    candidates.iter().map(|decl| assertion(*at, decl)).collect(),
                )
            })
            .collect();
        let rendered = program.render(&accepted, &asserts);
        let diagnostics = compile(&rendered.source);
        let mut failed: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut refused: Vec<usize> = Vec::new();
        let mut now_rejected: Vec<(usize, String)> = Vec::new();
        for error in errors(&diagnostics) {
            if error.line.is_none() && !unowned(error) {
                continue;
            }
            let owner = error
                .line
                .and_then(|line| rendered.owners.get(line - 1).copied());
            match owner {
                Some(Owner::Assert(probe, index)) => {
                    failed.insert((probe, index));
                }
                Some(Owner::Decl(at)) => refused.push(at),
                Some(Owner::Probe(probe)) => {
                    now_rejected.push((probe, code_of(error)));
                }
                other => panic!(
                    "rustc refused the accepted probes at {other:?}: {}\n{}",
                    error.text, rendered.source
                ),
            }
        }
        if !refused.is_empty() {
            return Err(refused);
        }
        if !now_rejected.is_empty() {
            for (probe, code) in now_rejected {
                rejected.entry(probe).or_default().push(code);
                accepted.retain(|at| *at != probe);
            }
            continue;
        }
        return Ok((0..program.probes.len())
            .map(|at| {
                if let Some(codes) = rejected.get(&at) {
                    return Verdict::Rejected(codes.clone());
                }
                let mut ty = None;
                let mut value = None;
                for (index, decl) in candidates.iter().enumerate() {
                    if failed.contains(&(at, index)) {
                        continue;
                    }
                    match decl.space {
                        Space::Type => ty = Some(decl.tag),
                        Space::Value => value = Some(decl.tag),
                    }
                }
                assert!(
                    ty.is_some() || value.is_some(),
                    "rustc accepts probe {at} but it names no item of the program:\n{}",
                    rendered.source
                );
                Verdict::Accepted { ty, value }
            })
            .collect());
    }
}
