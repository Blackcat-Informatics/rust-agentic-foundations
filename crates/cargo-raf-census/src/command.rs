// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Host input adapter; Census policy lives in the repository target and library.

use crate::cli::{
    AdmissionArgs, Args, Command, HistoryArgs, MigrationAdmissionArgs, MigrationArgs,
    PersistenceArgs, QueryTarget, RdfFormat, ReaderAdmissionArgs, ReaderEvidenceArgs,
    ReferenceArgs, RegistryArgs, ResealArgs, SchemaArgs,
};
use raf_census::capnp_history::persistence::{admission, migration};
use raf_census::capnp_history::{
    self, CaptureFiles, CaptureVerification, HistoryOptions, Snapshot, persistence, reader_evidence,
};
use raf_census::{
    ArchiveCache, Census, CensusOptions, Error, ExternalReference, Gate, GateReport, LEDGER_FILE,
    Ledger, LockedRegistry, MAX_INDEX_BYTES, MAX_LEDGER_BYTES, MigrationInput, ReportError,
    migrate_index, migrate_legacy,
};
use raf_source::{DiskTree, RelPath, Tree as _};
use std::{fmt::Display, io::Write, path::Path};

pub(crate) fn run(
    args: &Args,
    configure: Option<impl FnOnce(Gate) -> Gate>,
    output: &mut dyn Write,
    diagnostics: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let external = external(&args.reference)?;
    let registry = registry(&args.registry)?;
    if (external.is_some() || registry.is_some()) && !accepts_external_sources(&args.command) {
        return Err(Error::Options(
            "reference options apply only to check, index, reference-matches, job, count, fingerprint and debt, and so do the registry options".into(),
        )
        .into());
    }
    if let Some(result) = pinned_command(args, output) {
        return result;
    }
    if let Command::LintCapture { out, .. } = &args.command {
        return Ok(crate::lints::capture(&args.workspace, out, diagnostics)?);
    }
    if let Some(directory) = &args.reference.unpinned {
        warn_unpinned_reference(directory, diagnostics)?;
    }
    let gate = Gate::workspace(&args.workspace).external_references(external);
    let (gate, registered) = match configure {
        Some(configure) => (configure(gate), true),
        None => (gate, false),
    };
    run_gate(
        args,
        gate,
        registered,
        registry.as_ref(),
        output,
        diagnostics,
    )
}

fn run_gate(
    args: &Args,
    gate: Gate,
    registered: bool,
    registry: Option<&LockedRegistry>,
    output: &mut dyn Write,
    diagnostics: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let gate = if matches!(args.command, Command::Check { .. }) {
        gate
    } else {
        gate.without_production_lints()
    };
    match &args.command {
        Command::Init => init(&gate, output),
        Command::CapnpReaderEvidence(_)
        | Command::CapnpReaderAdmission(_)
        | Command::CapnpMigrationAdmission(_)
        | Command::CapnpPersistence(_) => {
            unreachable!(
                "INVARIANT: Cap'n Proto policy and evidence dispatch before Gate registration"
            )
        }
        Command::CapnpHistory(history) => compare_schema_history(&args.workspace, history, output),
        Command::Embedded { command } => {
            crate::embedded::run(&gate.root()?, command, output, diagnostics)
        }
        Command::SelfTest => self_test(&gate, output),
        Command::DumpLedger { format, out } => {
            dump_ledger(&gate.root()?, *format, out.as_deref(), output)?;
            Ok(0)
        }
        Command::Reseal(args) => reseal(&gate.root()?, args, output, diagnostics),
        Command::Baseline { format, out } => {
            baseline(
                &gate.root()?,
                registered.then_some(&gate),
                *format,
                out.as_deref(),
                output,
            )?;
            Ok(0)
        }
        Command::Visibility {
            demand,
            format,
            out,
        } => visibility(&gate.root()?, *format, out.as_deref(), *demand, output),
        Command::LintInputs => {
            writeln!(output, "{}", crate::lints::inputs(&gate)?.digest()).map_err(stdout)?;
            Ok(0)
        }
        Command::LintCapture { .. } => {
            unreachable!("INVARIANT: trusted lint capture dispatches before Gate registration")
        }
        Command::LintReceipt {
            before,
            compiler,
            status,
            diagnostics,
            arguments,
        } => {
            let text =
                crate::lints::receipt(&gate, before, compiler, *status, diagnostics, arguments)?;
            output.write_all(text.as_bytes()).map_err(stdout)?;
            Ok(0)
        }
        Command::Check { policy, lints } => {
            let gate = crate::lints::configure(policy.apply(gate), lints)?;
            let report = evaluate(&gate, registry)?;
            for finding in report.findings() {
                writeln!(output, "{finding}").map_err(stdout)?;
            }
            Ok(u8::from(!report.is_clean()))
        }

        Command::Index(target) => index(&gate, registry, target, output),
        Command::ReferenceMatches(target) => {
            let report = evaluate(&gate, registry)?.reference_matches()?;
            write_query(&gate, target, report.graph(), output)
        }
        Command::Job { id, target } => {
            query(&gate, registry, target, raf_census::Query::Job(id), output)
        }
        Command::Count { id, target } => query(
            &gate,
            registry,
            target,
            raf_census::Query::Count(id),
            output,
        ),
        Command::Fingerprint { symbol, target } => query(
            &gate,
            registry,
            target,
            raf_census::Query::Fingerprint(symbol),
            output,
        ),
        Command::Debt { write: _, policy } => {
            let gate = policy.apply(gate);
            let receipt = match registry {
                Some(registry) => gate.write_debt_registry(registry)?,
                None => gate.write_debt()?,
            };
            writeln!(output, "{receipt}").map_err(stdout)?;
            Ok(0)
        }
        Command::Migrate(args) => migrate(&gate, args, output),
    }
}

fn init(gate: &Gate, output: &mut dyn Write) -> Result<u8, crate::error::Error> {
    let path = gate.init_ledger()?;
    writeln!(output, "created {}", path.display()).map_err(stdout)?;
    Ok(0)
}

fn warn_unpinned_reference(directory: &Path, diagnostics: &mut dyn Write) -> Result<(), Error> {
    writeln!(
        diagnostics,
        "cargo-raf-census: unpinned reference: the comparison reads the current files of {}, not a pinned commit",
        directory.display()
    )
    .map_err(|source| Error::Io {
        file: "<stderr>".into(),
        source,
    })
}

fn pinned_command(args: &Args, output: &mut dyn Write) -> Option<Result<u8, crate::error::Error>> {
    match &args.command {
        Command::CapnpReaderEvidence(evidence) => {
            Some(verify_reader_evidence(&args.workspace, evidence, output))
        }
        Command::CapnpPersistence(persistence) => {
            Some(compare_persistence(&args.workspace, persistence, output))
        }
        Command::CapnpReaderAdmission(admission) => {
            Some(compare_reader_admission(&args.workspace, admission, output))
        }
        Command::CapnpMigrationAdmission(admission) => Some(compare_migration_admission(
            &args.workspace,
            admission,
            output,
        )),
        _ => None,
    }
}

fn verify_reader_evidence(
    root: &Path,
    args: &ReaderEvidenceArgs,
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let paths = reader_evidence::verify(
        root,
        reader_evidence::Expected {
            base: &args.base,
            current: &args.current,
            root: args.root,
            entry_point: &args.reader_entry,
            artifact: reader_evidence::artifact_digest(&args.reader_sha256)?,
            payloads: &args.payloads,
        },
        reader_evidence::Evidence {
            commit: &args.evidence_commit,
            files: CaptureFiles {
                statement: &args.statement,
                signature: &args.signature,
            },
            key: &args.key,
        },
    )?;
    writeln!(output, "verified signed reader acceptance claim").map_err(stdout)?;
    for path in paths {
        writeln!(output, "{path}").map_err(stdout)?;
    }
    Ok(0)
}

fn compare_schema_history(
    root: &Path,
    args: &HistoryArgs,
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let findings = with_schema_inputs(&args.schema, |base, current, capture| {
        Ok(capnp_history::check(
            root,
            base,
            current,
            HistoryOptions {
                roots: &args.types,
                file_ids: &args.file_ids,
                required_annotation: args.schema.required_annotation,
                paired_presence_annotation: args.schema.paired_presence_annotation,
                capture,
            },
        )?)
    })?;
    write_findings(&findings, output)
}

fn compare_persistence(
    root: &Path,
    args: &PersistenceArgs,
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let findings = with_schema_inputs(&args.schema, |base, current, capture| {
        Ok(persistence::check(
            root,
            base,
            current,
            persistence_options(args, capture),
        )?)
    })?;
    write_findings(&findings, output)
}

fn compare_reader_admission(
    root: &Path,
    args: &ReaderAdmissionArgs,
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let selection = &args.selection;
    let policy = &selection.persistence;
    let claim = selected_evidence(
        selection,
        args.reader_sha256.as_deref(),
        |digest, evidence| {
            Ok(admission::Claim {
                artifact: reader_evidence::artifact_digest(digest)?,
                evidence,
            })
        },
    )?;
    let report = with_schema_inputs(&policy.schema, |base, current, capture| {
        Ok(admission::check(
            root,
            admission::Request {
                base,
                current,
                options: persistence_options(policy, capture),
                declaration: &selection.declaration,
                claim,
            },
        )?)
    })?;
    write_findings(&report.findings, output)?;
    match report.verdict {
        admission::Verdict::Clean => Ok(0),
        admission::Verdict::Uncovered => {
            writeln!(
                output,
                "strict findings remain without admitted reader coverage"
            )
            .map_err(stdout)?;
            Ok(1)
        }
        admission::Verdict::Admitted { specimens } => {
            writeln!(
                output,
                "admitted signed old-shape reader claim for {}",
                selection.declaration
            )
            .map_err(stdout)?;
            for path in specimens {
                writeln!(output, "{path}").map_err(stdout)?;
            }
            Ok(0)
        }
    }
}

fn compare_migration_admission(
    root: &Path,
    args: &MigrationAdmissionArgs,
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let selection = &args.selection;
    let policy = &selection.persistence;
    let claim = selected_evidence(
        selection,
        args.migration_sha256.as_deref(),
        |digest, evidence| {
            Ok(migration::Claim {
                artifact: reader_evidence::artifact_digest(digest)?,
                evidence,
            })
        },
    )?;
    let report = with_schema_inputs(&policy.schema, |base, current, capture| {
        Ok(migration::check(
            root,
            migration::Request {
                base,
                current,
                options: persistence_options(policy, capture),
                declaration: &selection.declaration,
                claim,
            },
        )?)
    })?;
    write_findings(&report.findings, output)?;
    match report.verdict {
        migration::Verdict::Clean => Ok(0),
        migration::Verdict::Uncovered => {
            writeln!(
                output,
                "strict findings remain without admitted migration coverage"
            )
            .map_err(stdout)?;
            Ok(1)
        }
        migration::Verdict::Admitted { pairs } => {
            writeln!(
                output,
                "admitted signed migration claim for {}",
                selection.declaration
            )
            .map_err(stdout)?;
            for pair in pairs {
                writeln!(output, "{}\t{}", pair.input, pair.output).map_err(stdout)?;
            }
            Ok(0)
        }
    }
}

fn persistence_options<'a>(
    args: &'a PersistenceArgs,
    capture: Option<CaptureVerification<'a>>,
) -> persistence::Options<'a> {
    persistence::Options {
        policy: &args.policy,
        required_annotation: args.schema.required_annotation,
        paired_presence_annotation: args.schema.paired_presence_annotation,
        capture,
    }
}

fn selected_evidence<'a, T>(
    args: &'a AdmissionArgs,
    artifact: Option<&str>,
    claim: impl FnOnce(&str, reader_evidence::Evidence<'a>) -> Result<T, reader_evidence::Error>,
) -> Result<Option<T>, crate::error::Error> {
    match (
        &args.evidence_commit,
        &args.statement,
        &args.signature,
        &args.key,
        artifact,
    ) {
        (Some(commit), Some(statement), Some(signature), Some(key), Some(digest)) => {
            Ok(Some(claim(
                digest,
                reader_evidence::Evidence {
                    commit,
                    files: CaptureFiles {
                        statement,
                        signature,
                    },
                    key,
                },
            )?))
        }
        (None, None, None, None, None) => Ok(None),
        _ => Err(Error::Options("supply all admission evidence options together".into()).into()),
    }
}

fn with_schema_inputs<T>(
    args: &SchemaArgs,
    analyze: impl FnOnce(
        Snapshot<'_>,
        Snapshot<'_>,
        Option<CaptureVerification<'_>>,
    ) -> Result<T, crate::error::Error>,
) -> Result<T, crate::error::Error> {
    let base_descriptor =
        RelPath::new(&args.base_descriptor).map_err(|error| Error::Options(error.to_string()))?;
    let current_descriptor = RelPath::new(&args.current_descriptor)
        .map_err(|error| Error::Options(error.to_string()))?;
    let capture = match (
        &args.capture_key,
        &args.base_capture,
        &args.base_capture_signature,
        &args.current_capture,
        &args.current_capture_signature,
    ) {
        (Some(key), Some(base), Some(base_signature), Some(current), Some(current_signature)) => {
            Some(CaptureVerification {
                key,
                base: CaptureFiles {
                    statement: base,
                    signature: base_signature,
                },
                current: CaptureFiles {
                    statement: current,
                    signature: current_signature,
                },
            })
        }
        (None, None, None, None, None) => None,
        _ => return Err(Error::Options(
            "capture verification needs a public key and both committed statements and signatures"
                .to_owned(),
        )
        .into()),
    };
    analyze(
        Snapshot {
            commit: &args.base,
            descriptor: &base_descriptor,
        },
        Snapshot {
            commit: &args.current,
            descriptor: &current_descriptor,
        },
        capture,
    )
}

fn write_findings(
    findings: &[impl Display],
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    for finding in findings {
        writeln!(output, "{finding}").map_err(stdout)?;
    }
    Ok(u8::from(!findings.is_empty()))
}

fn index(
    gate: &Gate,
    registry: Option<&LockedRegistry>,
    target: &QueryTarget,
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let report = evaluate(gate, registry)?;
    let index = report.census().index(report.jobs())?;
    let serialized = index.to_ntriples()?;
    let limit = target.max_index_bytes;
    if serialized.len() > limit {
        return Err(Error::Options(format!(
            "native index is {} bytes, exceeding its {limit}-byte output bound",
            serialized.len()
        ))
        .into());
    }
    let text = match target.format {
        RdfFormat::Ntriples => serialized,
        RdfFormat::Turtle => {
            drop(serialized);
            index.to_turtle()?
        }
    };
    write_report(&gate.root()?, &text, target.out.as_deref(), output)?;
    Ok(0)
}

fn query(
    gate: &Gate,
    registry: Option<&LockedRegistry>,
    target: &QueryTarget,
    selector: raf_census::Query<'_>,
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let result = evaluate(gate, registry)?.query(selector)?;
    write_query(gate, target, result.graph(), output)
}

fn write_query(
    gate: &Gate,
    target: &QueryTarget,
    graph: &raf_rdf::Graph,
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    let text = graph
        .to_text_bounded(target.format.native(), target.max_index_bytes)
        .map_err(ReportError::Ground)
        .map_err(Error::from)?;
    write_report(&gate.root()?, &text, target.out.as_deref(), output)?;
    Ok(0)
}

/// The same evaluation supplies checks, the complete index and each selected
/// report, with sealed registry dependencies when the options name any.
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
    let path = RelPath::new(LEDGER_FILE).map_err(|source| Error::Io {
        file: LEDGER_FILE.into(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidInput, source),
    })?;
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

/// Re-seal the workspace ledger. Its rows are verified by the read-back of the
/// sealed text. The sealed text goes to stdout, to a new `--out` file, or with
/// `--in-place` over the workspace ledger by one rename. The ledger that was read
/// is named on diagnostics.
fn reseal(
    root: &Path,
    args: &ResealArgs,
    output: &mut dyn Write,
    diagnostics: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    if let Some(out) = args.out.as_deref()
        && out.extension().and_then(std::ffi::OsStr::to_str) != Some("ttl")
    {
        return Err(Error::Options(format!(
            "{}: reseal writes Turtle, so its destination needs a .ttl extension",
            out.display()
        ))
        .into());
    }
    let source = root.join(LEDGER_FILE);
    writeln!(diagnostics, "reseal: read {}", source.display()).map_err(|source| Error::Io {
        file: "<stderr>".into(),
        source,
    })?;
    let tree = DiskTree::new(root).map_err(|source| Error::Tree {
        file: root.display().to_string(),
        source,
    })?;
    let path = RelPath::new(LEDGER_FILE).map_err(|source| Error::Io {
        file: LEDGER_FILE.into(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidInput, source),
    })?;
    let text = tree
        .read(&path, MAX_LEDGER_BYTES as u64)
        .map_err(|source| Error::Tree {
            file: path.to_string(),
            source,
        })?;
    let sealed = Ledger::reseal_turtle(&text).map_err(|source| Error::Input {
        file: LEDGER_FILE.into(),
        source: Box::new(source),
    })?;
    if args.in_place {
        replace_ledger(&source, &text, &sealed)?;
    } else {
        write_report(root, &sealed, args.out.as_deref(), output)?;
    }
    Ok(0)
}

/// Replace the ledger file with the sealed text.
///
/// The text goes to a sibling file with the ledger's permissions, which is synced
/// and then renamed over the ledger; the directory is synced after the rename. A
/// ledger that is a symbolic link is refused, so the link is never replaced by a
/// file. A failed step leaves the original ledger intact, and an error names the
/// sibling file it left behind.
fn replace_ledger(ledger: &Path, read: &str, sealed: &str) -> Result<(), Error> {
    let io_error = |file: &Path, source: std::io::Error| Error::Io {
        file: file.display().to_string(),
        source,
    };
    let current = std::fs::symlink_metadata(ledger).map_err(|source| io_error(ledger, source))?;
    if current.file_type().is_symlink() {
        return Err(io_error(
            ledger,
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the ledger is a symbolic link; --in-place replaces a regular file, \
                 so point the link at a regular ledger, or write the text with --out",
            ),
        ));
    }
    let directory = ledger.parent().ok_or_else(|| {
        io_error(
            ledger,
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the ledger path has no directory to sync",
            ),
        )
    })?;
    let temp = ledger.with_extension("ttl.reseal-tmp");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|source| {
            let source = if source.kind() == std::io::ErrorKind::AlreadyExists {
                std::io::Error::new(
                    source.kind(),
                    format!(
                        "{source}; if no other reseal is running, remove this file and run \
                         the command again"
                    ),
                )
            } else {
                source
            };
            io_error(&temp, source)
        })?;
    let staged = file
        .set_permissions(current.permissions())
        .and_then(|()| file.write_all(sealed.as_bytes()))
        .and_then(|()| file.sync_all());
    staged.map_err(|source| io_error(&temp, source))?;
    // A ledger that differs from the text this reseal read was changed by another
    // writer, so it is left as it is. This comparison is not a lock: an edit made
    // between it and the rename is still replaced.
    let current = std::fs::read_to_string(ledger).map_err(|source| io_error(ledger, source))?;
    if current != read {
        std::fs::remove_file(&temp).map_err(|source| io_error(&temp, source))?;
        return Err(io_error(
            ledger,
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "the ledger changed after reseal read it, so nothing was replaced; \
                 run reseal --in-place again on the current ledger",
            ),
        ));
    }
    std::fs::rename(&temp, ledger).map_err(|source| io_error(&temp, source))?;
    // The rename has already replaced the ledger. A failed directory sync leaves
    // only the durability of that replacement unconfirmed, and the error says so.
    std::fs::File::open(directory)
        .and_then(|handle| handle.sync_all())
        .map_err(|source| {
            io_error(
                directory,
                std::io::Error::new(
                    source.kind(),
                    format!(
                        "the ledger at {} was replaced, but syncing its directory failed: \
                         {source}; the replacement may not survive a crash, so run the \
                         command again to confirm it",
                        ledger.display()
                    ),
                ),
            )
        })
}

fn baseline(
    root: &Path,
    registration: Option<&Gate>,
    format: RdfFormat,
    out: Option<&Path>,
    output: &mut dyn Write,
) -> Result<(), Error> {
    render_tree_report(root, out, output, |tree| {
        if let Some(gate) = registration {
            gate.validate_tree_registrations(tree)?;
        }
        let report = Census::load(tree, &["."], &CensusOptions::default())?.baseline();
        match format {
            RdfFormat::Ntriples => report.to_ntriples(),
            RdfFormat::Turtle => report.to_turtle(),
        }
    })
}

fn render_tree_report(
    root: &Path,
    out: Option<&Path>,
    output: &mut dyn Write,
    render: impl FnOnce(&DiskTree) -> Result<String, Error>,
) -> Result<(), Error> {
    let tree = DiskTree::new(root).map_err(|source| Error::Tree {
        file: root.display().to_string(),
        source,
    })?;
    let text = render(&tree)?;
    write_report(root, &text, out, output)
}

fn accepts_external_sources(command: &Command) -> bool {
    matches!(
        command,
        Command::Check { .. }
            | Command::Index(_)
            | Command::ReferenceMatches(_)
            | Command::Job { .. }
            | Command::Count { .. }
            | Command::Fingerprint { .. }
            | Command::Debt { .. }
    )
}

fn visibility(
    root: &Path,
    format: RdfFormat,
    out: Option<&Path>,
    demand: bool,
    output: &mut dyn Write,
) -> Result<u8, crate::error::Error> {
    render_tree_report(root, out, output, |tree| {
        let graph = if demand {
            raf_census::visibility_demand(tree, &RelPath::root())?
        } else {
            raf_census::visibility(tree, &RelPath::root())?
        };
        graph
            .to_text(format.native())
            .map_err(|error| ReportError::Ground(error).into())
    })?;
    Ok(0)
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
        if git_administration(root, &parent)? {
            return Err(refuse(
                "report destinations cannot enter Git administration",
            ));
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

pub(crate) fn git_administration(root: &Path, parent: &Path) -> Result<bool, Error> {
    if parent.components().any(|part| part.as_os_str() == ".git") {
        return Ok(true);
    }
    for location in [root, parent] {
        if let Some(repository) = raf_source::git::Repository::discover(location)
            .map_err(|source| Error::Options(source.to_string()))?
            && (parent.starts_with(repository.git_dir())
                || parent.starts_with(repository.common_dir()))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn read(path: &Path, limit: usize) -> Result<String, Error> {
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
    #[cfg_attr(
        test,
        expect(
            clippy::unwrap_used,
            reason = "a failed fixture operation is a test failure at the point of use"
        )
    )]
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

    #[test]
    #[expect(
        clippy::unwrap_used,
        reason = "a failed fixture operation is a test failure at the point of use"
    )]
    fn replace_ledger_refuses_a_ledger_changed_after_reseal_read_it_and_removes_its_staged_file() {
        // INVARIANT: a ledger whose text differs from the text reseal read is never replaced,
        // and the staged sibling file this call created is removed.
        // CARGO_TARGET_TMPDIR is set only for integration tests. A library unit test's
        // binary lives under the target directory, which holds its scratch tree.
        let target = std::env::current_exe()
            .unwrap()
            .ancestors()
            .nth(3)
            .unwrap()
            .to_path_buf();
        let dir = purrdf_testkit::TempDir::new_in(&target).unwrap();
        let ledger = dir.path().join(raf_census::LEDGER_FILE);
        std::fs::write(&ledger, "edited by another writer\n").unwrap();
        let error =
            super::replace_ledger(&ledger, "the text reseal read\n", "sealed\n").unwrap_err();
        assert!(
            error.to_string().contains("changed after reseal read it"),
            "{error}"
        );
        assert_eq!(
            std::fs::read_to_string(&ledger).unwrap(),
            "edited by another writer\n"
        );
        assert!(
            !dir.path()
                .join(format!("{}.reseal-tmp", raf_census::LEDGER_FILE))
                .exists()
        );
    }
}
