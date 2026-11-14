// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Host input adapter; Census policy lives in the repository target and library.

use crate::cli::{
    Args, Command, MigrationArgs, QueryTarget, RdfFormat, ReferenceArgs, RegistryArgs,
};
use raf_census::{
    ArchiveCache, Census, CensusOptions, Error, ExternalReference, Gate, GateReport, LEDGER_FILE,
    Ledger, LockedRegistry, MAX_INDEX_BYTES, MAX_LEDGER_BYTES, MigrationInput, ReportError,
    migrate_index, migrate_legacy,
};
use raf_source::{DiskTree, RelPath, Tree as _};
use std::{io::Write, path::Path};

pub(crate) fn run(
    args: &Args,
    output: &mut dyn Write,
    diagnostics: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let external = external(&args.reference)?;
    let registry = registry(&args.registry)?;
    if (external.is_some() || registry.is_some())
        && matches!(
            args.command,
            Command::SelfTest
                | Command::DumpLedger { .. }
                | Command::Baseline { .. }
                | Command::Migrate(_)
                | Command::Embedded { .. }
        )
    {
        return Err(Error::Options(
            "reference options apply only to check, index, job, count, fingerprint and debt, and so do the registry options".into(),
        )
        .into());
    }
    if let Some(directory) = &args.reference.unpinned {
        writeln!(
            diagnostics,
            "cargo-census: unpinned reference: the comparison reads the current files of {}, not a pinned commit",
            directory.display()
        )
        .map_err(|source| Error::Io {
            file: "<stderr>".into(),
            source,
        })?;
    }
    let gate = Gate::workspace(&args.workspace).external_references(external);
    match &args.command {
        Command::Embedded { command } => {
            crate::embedded::run(&gate.root()?, command, output, diagnostics)
        }
        Command::SelfTest => self_test(&gate, output),
        Command::DumpLedger { format, out } => {
            dump_ledger(&gate.root()?, *format, out.as_deref(), output)?;
            Ok(0)
        }
        Command::Baseline { format, out } => {
            baseline(&gate.root()?, *format, out.as_deref(), output)?;
            Ok(0)
        }
        Command::Check => {
            let report = evaluate(&gate, registry.as_ref())?;
            for finding in report.findings() {
                writeln!(output, "{finding}").map_err(stdout)?;
            }
            Ok(u8::from(!report.is_clean()))
        }

        Command::Index(target) => {
            let index = native_index(&gate, registry.as_ref(), target.max_index_bytes)?;
            let text = match target.format {
                RdfFormat::Ntriples => index.to_ntriples()?,
                RdfFormat::Turtle => index.to_turtle()?,
            };
            write_report(&gate.root()?, &text, target.out.as_deref(), output)?;
            Ok(0)
        }
        Command::Job { id, target } => query(
            &gate,
            registry.as_ref(),
            target,
            raf_census::Query::Job(id),
            output,
        ),
        Command::Count { id, target } => query(
            &gate,
            registry.as_ref(),
            target,
            raf_census::Query::Count(id),
            output,
        ),
        Command::Fingerprint { symbol, target } => query(
            &gate,
            registry.as_ref(),
            target,
            raf_census::Query::Fingerprint(symbol),
            output,
        ),
        Command::Debt { write: _ } => {
            let receipt = match registry.as_ref() {
                Some(registry) => gate.write_debt_registry(registry)?,
                None => gate.write_debt()?,
            };
            writeln!(output, "{receipt}").map_err(stdout)?;
            Ok(0)
        }
        Command::Migrate(args) => migrate(&gate, args, output),
    }
}

fn native_index(
    gate: &Gate,
    registry: Option<&LockedRegistry>,
    limit: usize,
) -> Result<raf_census::Index, crate::error::Error> {
    let report = evaluate(gate, registry)?;
    let index = report.census().index(report.jobs())?;
    let serialized = index.to_ntriples()?;
    if serialized.len() > limit {
        return Err(Error::Options(format!(
            "native index is {} bytes, exceeding its {limit}-byte output bound",
            serialized.len()
        ))
        .into());
    }
    Ok(index)
}

fn query(
    gate: &Gate,
    registry: Option<&LockedRegistry>,
    target: &QueryTarget,
    selector: raf_census::Query<'_>,
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let root = gate.root()?;
    let index = native_index(gate, registry, target.max_index_bytes)?;
    let result = index.query(selector)?;
    let text = match target.format {
        RdfFormat::Ntriples => result.to_ntriples()?,
        RdfFormat::Turtle => result.to_turtle()?,
    };
    write_report(&root, &text, target.out.as_deref(), output)?;
    Ok(0)
}

/// The one evaluation that `check`, `index`, `job`, `count` and `fingerprint`
/// share: the locked check with the sealed registry dependencies when the
/// options name any, otherwise the ordinary check.
fn evaluate(
    gate: &Gate,
    registry: Option<&LockedRegistry>,
) -> Result<GateReport, crate::error::Error> {
    Ok(match registry {
        Some(registry) => gate.check_registry(registry)?,
        None => gate.check()?,
    })
}

/// The sealed registry dependencies the options declare, or `None` when they
/// declare none. Every sealed package needs its complete feature set declared.
fn registry(args: &RegistryArgs) -> Result<Option<LockedRegistry>, Error> {
    if args.cache.is_none() && args.seal.is_empty() && args.features.is_empty() {
        return Ok(None);
    }
    let cache = match &args.cache {
        Some(directory) => ArchiveCache::at(directory),
        None => ArchiveCache::cargo_home()?,
    };
    let mut registry = LockedRegistry::new(cache).require_declared_features();
    for selector in &args.seal {
        registry = registry.seal_reference(selector);
    }
    let mut declared = std::collections::BTreeSet::new();
    for declaration in &args.features {
        let (package, features) = declaration.split_once('=').ok_or_else(|| {
            Error::Options(format!(
                "--registry-features {declaration:?}: expected PACKAGE=FEATURE,FEATURE (PACKAGE= for none)"
            ))
        })?;
        if package.is_empty() || !declared.insert(package) {
            return Err(Error::Options(format!(
                "--registry-features {declaration:?}: name each package once, and name it"
            )));
        }
        registry = registry.features(
            package,
            features.split(',').filter(|feature| !feature.is_empty()),
        );
    }
    Ok(Some(registry))
}

/// The one caller-declared reference tree, when the options name one.
fn external(args: &ReferenceArgs) -> Result<Option<ExternalReference>, Error> {
    Ok(Some(
        match (
            &args.repo,
            &args.commit,
            &args.pin_file,
            &args.pin_key,
            &args.unpinned,
        ) {
            (None, None, None, None, None) => return Ok(None),
            (None, None, None, None, Some(directory)) => ExternalReference::unpinned(directory),
            (Some(repository), Some(commit), None, None, None) => {
                ExternalReference::commit(repository, commit)
            }
            (Some(repository), None, Some(file), Some(key), None) => ExternalReference::pin_file(
                repository,
                RelPath::new(file).map_err(|error| {
                    Error::Options(format!("--reference-pin-file {file:?}: {error}"))
                })?,
                key,
            ),
            _ => {
                return Err(Error::Options(
                    "reference options: give --reference-repo with --reference-commit, or with --reference-pin-file and --reference-pin-key, or give --reference-unpinned alone".into(),
                ));
            }
        },
    ))
}

fn migrate(
    gate: &Gate,
    args: &MigrationArgs,
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    if let Some(kind) = args.report_form {
        let path = args
            .input
            .as_deref()
            .ok_or_else(|| Error::Options("report-form conversion requires --input".into()))?;
        let format = args
            .format
            .ok_or_else(|| Error::Options("report-form conversion requires --format".into()))?;
        let text = read(path, kind.native().max_bytes())?;
        let name = path.display().to_string();
        let converted = raf_census::upgrade_report_form(
            kind.native(),
            MigrationInput {
                path: &name,
                text: &text,
            },
            format.native(),
        )?;
        output.write_all(converted.as_bytes()).map_err(stdout)?;
        return Ok(0);
    }
    let ledger = args
        .ledger
        .as_deref()
        .ok_or_else(|| Error::Options("migration requires --ledger".into()))?;
    let root = gate.root()?;
    let tree = DiskTree::new(&root).map_err(|source| Error::Tree {
        file: root.display().to_string(),
        source,
    })?;
    let old = read(ledger, MAX_LEDGER_BYTES)?;
    let companion = if args.legacy {
        args.baseline.as_ref()
    } else {
        args.index.as_ref()
    }
    .ok_or_else(|| {
        Error::Options("migration requires its explicitly selected baseline or index".into())
    })?;
    let previous = read(companion, MAX_INDEX_BYTES)?;
    let old_path = ledger.display().to_string();
    let previous_path = companion.display().to_string();
    let old = MigrationInput {
        path: &old_path,
        text: &old,
    };
    let previous = MigrationInput {
        path: &previous_path,
        text: &previous,
    };
    let result = if args.legacy {
        migrate_legacy(&tree, old, previous)
    } else {
        migrate_index(&tree, old, previous)
    }?;
    output.write_all(result.ledger.as_bytes()).map_err(stdout)?;
    Ok(u8::from(!result.unmapped.is_empty()))
}

fn dump_ledger(
    root: &Path,
    format: RdfFormat,
    out: Option<&Path>,
    output: &mut dyn Write,
) -> Result<(), Error> {
    let tree = DiskTree::new(root).map_err(|source| Error::Tree {
        file: root.display().to_string(),
        source,
    })?;
    let path = RelPath::new(LEDGER_FILE).expect("literal ledger path");
    let text = tree
        .read(&path, MAX_LEDGER_BYTES as u64)
        .map_err(|source| Error::Tree {
            file: path.to_string(),
            source,
        })?;
    let ledger = Ledger::from_turtle(&text).map_err(|source| Error::Input {
        file: LEDGER_FILE.into(),
        source: Box::new(source),
    })?;
    let format = match format {
        RdfFormat::Ntriples => raf_rdf::Format::NTriples,
        RdfFormat::Turtle => raf_rdf::Format::Turtle,
    };
    let text = ledger
        .graph()
        .to_text(format)
        .map_err(ReportError::Ground)?;
    write_report(root, &text, out, output)
}

fn baseline(
    root: &Path,
    format: RdfFormat,
    out: Option<&Path>,
    output: &mut dyn Write,
) -> Result<(), Error> {
    let tree = DiskTree::new(root).map_err(|source| Error::Tree {
        file: root.display().to_string(),
        source,
    })?;
    let report = Census::load(&tree, &["."], &CensusOptions::default())?.baseline();
    let text = match format {
        RdfFormat::Ntriples => report.to_ntriples()?,
        RdfFormat::Turtle => report.to_turtle()?,
    };
    write_report(root, &text, out, output)
}

pub(crate) fn write_report(
    root: &Path,
    text: &str,
    out: Option<&Path>,
    output: &mut dyn Write,
) -> Result<(), Error> {
    if let Some(path) = out {
        let refuse = |message: &str| Error::Options(format!("{}: {message}", path.display()));
        if !matches!(
            path.extension().and_then(std::ffi::OsStr::to_str),
            Some("ttl" | "nt")
        ) {
            return Err(refuse(
                "report destinations require a .ttl or .nt extension",
            ));
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent = parent.canonicalize().map_err(|source| Error::Io {
            file: parent.display().to_string(),
            source,
        })?;
        if parent
            .components()
            .any(|component| component.as_os_str() == ".git")
        {
            return Err(refuse(
                "report destinations cannot enter Git administration",
            ));
        }
        for selected in [root, parent.as_path()] {
            if let Some(repository) = raf_source::git::Repository::discover(selected)
                .map_err(|error| Error::Options(error.to_string()))?
                && (parent.starts_with(repository.git_dir())
                    || parent.starts_with(repository.common_dir()))
            {
                return Err(refuse(
                    "report destinations cannot enter Git administration",
                ));
            }
        }
        let destination = parent.join(
            path.file_name()
                .ok_or_else(|| refuse("report filename is absent"))?,
        );
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .map_err(|source| Error::Io {
                file: path.display().to_string(),
                source,
            })?;
        file.write_all(text.as_bytes())
            .map_err(|source| Error::Io {
                file: destination.display().to_string(),
                source,
            })?;
    } else {
        output.write_all(text.as_bytes()).map_err(stdout)?;
    }
    Ok(())
}

fn read(path: &Path, limit: usize) -> Result<String, Error> {
    let canonical = std::fs::canonicalize(path).map_err(|source| Error::Io {
        file: path.display().to_string(),
        source,
    })?;
    let parent = canonical
        .parent()
        .ok_or_else(|| Error::Options(format!("{}: expected an input file", path.display())))?;
    let name = canonical
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| {
            Error::Options(format!("{}: input filename is not UTF-8", path.display()))
        })?;
    let relative = RelPath::new(name).map_err(|error| Error::Options(error.to_string()))?;
    let tree = DiskTree::new(parent).map_err(|source| Error::Tree {
        file: path.display().to_string(),
        source,
    })?;
    tree.read(
        &relative,
        u64::try_from(limit).map_err(|error| Error::Options(error.to_string()))?,
    )
    .map_err(|source| Error::Tree {
        file: path.display().to_string(),
        source,
    })
}

fn stdout(source: std::io::Error) -> Error {
    Error::Io {
        file: "<stdout>".into(),
        source,
    }
}

/// Print the self-test report and map it to the exit status: 0 only when every case held.
fn self_test(gate: &Gate, output: &mut dyn Write) -> Result<u8, crate::error::Error> {
    let report = gate.self_test();
    write!(output, "{report}").map_err(stdout)?;
    Ok(u8::from(!report.passed()))
}

#[cfg(test)]
mod tests {
    use super::self_test;
    use raf_census::{FileCtx, Gate, Rule, RuleFixtures, RuleHit};

    /// A registered rule whose positive fixture is never flagged, so its fixture case fails.
    struct NeverFlags;

    impl Rule for NeverFlags {
        fn id(&self) -> &'static str {
            "never-flags"
        }
        fn check(&self, _: &FileCtx, _: &syn::File, _: &mut Vec<RuleHit>) {}
        fn fixtures(&self) -> RuleFixtures {
            RuleFixtures {
                positive: vec!["fn copy() {}"],
                negative: vec!["fn home() {}"],
            }
        }
    }

    /// The command path exits 1 and prints the failing case when a case fails, and exits 0
    /// otherwise. The command binary registers no rules, so a rule on the Gate is the only way
    /// to make a case fail through the same function the command calls.
    #[test]
    fn self_test_exits_one_with_the_failing_case_and_zero_when_every_case_holds() {
        let mut out = Vec::new();
        let failing =
            Gate::workspace("/nonexistent").rules(vec![Box::new(NeverFlags) as Box<dyn Rule>]);
        assert_eq!(self_test(&failing, &mut out).unwrap(), 1);
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("fail rule/never-flags/fixtures: "), "{text}");
        assert!(text.ends_with(" cases, 1 failed\n"), "{text}");

        let mut out = Vec::new();
        assert_eq!(
            self_test(&Gate::workspace("/nonexistent"), &mut out).unwrap(),
            0
        );
        assert!(
            String::from_utf8(out)
                .unwrap()
                .ends_with(" cases, 0 failed\n")
        );
    }
}
