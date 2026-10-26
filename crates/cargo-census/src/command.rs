// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Host input adapter; Census policy lives in the repository target and library.

use crate::cli::{Args, Command, MigrationArgs, QueryTarget, RdfFormat};
use raf_census::{
    Census, CensusOptions, Error, Gate, LEDGER_FILE, Ledger, MAX_INDEX_BYTES, MAX_LEDGER_BYTES,
    MigrationInput, ReportError, migrate_index, migrate_legacy,
};
use raf_source::{DiskTree, RelPath, Tree as _};
use std::{io::Write, path::Path};

pub(crate) fn run(args: &Args, output: &mut dyn Write) -> Result<u8, crate::error::Error> {
    let gate = Gate::workspace(&args.workspace);
    match &args.command {
        Command::DumpLedger { format, out } => {
            dump_ledger(&gate.root()?, *format, out.as_deref(), output)?;
            Ok(0)
        }
        Command::Baseline { format, out } => {
            baseline(&gate.root()?, *format, out.as_deref(), output)?;
            Ok(0)
        }
        Command::Check { package, example } => {
            crate::registered::check(&gate.root()?, package, example, output)
        }
        Command::Index {
            package,
            example,
            format,
            out,
        } => {
            let text = crate::registered::index(&gate.root()?, package, example, *format)?;
            write_report(&text, out.as_deref(), output)?;
            Ok(0)
        }
        Command::Job { id, target } => {
            query(&gate.root()?, target, raf_census::Query::Job(id), output)
        }
        Command::Count { id, target } => {
            query(&gate.root()?, target, raf_census::Query::Count(id), output)
        }
        Command::Fingerprint { symbol, target } => query(
            &gate.root()?,
            target,
            raf_census::Query::Fingerprint(symbol),
            output,
        ),
        Command::Debt {
            write: _,
            package,
            example,
        } => match (package, example) {
            (Some(package), Some(example)) => {
                crate::registered::write_debt(&gate.root()?, package, example, output)
            }
            (None, None) => {
                let receipt = gate.write_debt()?;
                writeln!(output, "{receipt}").map_err(stdout)?;
                Ok(0)
            }
            _ => Err(Error::Options(
                "registered debt writing requires both --package and --example".into(),
            )
            .into()),
        },
        Command::Migrate(args) => migrate(&gate, args, output),
    }
}

fn query(
    root: &Path,
    target: &QueryTarget,
    selector: raf_census::Query<'_>,
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let index = crate::registered::index_report(root, &target.package, &target.example)?;
    let result = index.query(selector)?;
    let text = match target.format {
        RdfFormat::Ntriples => result.to_ntriples()?,
        RdfFormat::Turtle => result.to_turtle()?,
    };
    write_report(&text, target.out.as_deref(), output)?;
    Ok(0)
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
    let previous = read(
        companion,
        if args.legacy {
            MAX_LEDGER_BYTES
        } else {
            MAX_INDEX_BYTES
        },
    )?;
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
    write_report(&text, out, output)
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
    write_report(&text, out, output)
}

fn write_report(text: &str, out: Option<&Path>, output: &mut dyn Write) -> Result<(), Error> {
    if let Some(path) = out {
        std::fs::write(path, text).map_err(|source| Error::Io {
            file: path.display().to_string(),
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
