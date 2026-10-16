// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Evidence: recording one execution as a record, and the claims that rest on
//! each record.
//!
//! [`Workplan::record_evidence`] runs one execution and writes
//! `work/evidence/<id>.md` with its verdict. The verdict has one source, the
//! [`VerdictSource`] of the [`EvidenceRequest`]:
//!
//! * [`VerdictSource::Command`]: the caller's own command, a [`TrustedArgv`]
//!   that only the command line of the running process gives (the words after
//!   `--`). It runs with no shell, standard input null, 1800 seconds and 16 MiB
//!   of output per stream ([`Limits::trusted`]), in the runner's working
//!   directory (the workspace root) and with the caller's environment unchanged.
//!   Its exit status decides the verdict: 0 is `pass`, anything else (a signal
//!   included) is `fail`, and a run killed at its time limit is `fail`. The
//!   measured values are the lines of its standard output of the form
//!   `name=number` ([`MeasureError`] lists what is refused: a value that is not
//!   a number, an exponent included, a name printed twice, and more than
//!   [`MAX_VALUES`] values); for a run that exited, a limit whose name the
//!   command never printed is an error, and a value over its limit (compared on
//!   its decimal text, exactly, by [`Number::compare`](raf_okf::Number::compare))
//!   makes the verdict `fail`. A run killed at its time limit is `fail` with no
//!   value, whatever its limits. No other output is kept. Output past the bound
//!   is an error, and a command that cannot be started writes no record. A
//!   background process that keeps the command's standard output open after it
//!   exits holds the run until the time limit.
//! * [`VerdictSource::Binding`]: a bound test or example of a capability of
//!   `for`, run through the [`Executor`] with the argv
//!   forms of the completion gate: `pass` when it passes by the proof its kind
//!   requires, otherwise `fail`. A path that is no test or example binding of a
//!   listed capability, or one the executor refuses before it starts, is an
//!   error and nothing starts. A binding run prints no measured value the tool
//!   reads, so it takes no limit.
//!
//! The record states the source (`subject: command` with the command text for
//! readers, or `subject: binding:<path>`), the verdict, `by` (`human` or
//! `agent`, as the caller says), the capabilities of `for`, the revision when the
//! caller gives one, the values and the limits, and its `inputs`: the
//! [`Digest::symbol`] of each `code` binding of each capability of `for`, at
//! the load. Each capability of `for` must be loaded and bind code that
//! resolves, or nothing runs. The stored command text and the revision are
//! never used for a decision and never reach a process.
//!
//! [`Workplan::evidence_claims`] lists the statements that rest on a record,
//! and [`Workplan::evidence_for`] the records that support a capability.

mod measure;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

use raf_okf::{
    Actor, CapabilitySection, DocType, EvidenceRecord, EvidenceSubject, EvidenceVerdict, Input,
    InputKind, Measure, is_kebab,
};
use raf_source::{LoadError, RelPath};

pub use measure::{MAX_VALUES, MeasureError, parse_limit};

use crate::bound::{BoundOutcome, Executor, Refusal};
use crate::digest::{Digest, SymbolDigestError};
use crate::exec::{Commit, ExecError, Invocation, Limits, Runner, TrustedArgv};
use crate::lookup::{capability_path, write_findings};
use crate::record::{self, Record, RecordError, RecordOptions};
use crate::{Binding, BindingKind, Finding, Workplan};

/// Where the verdict of a new evidence record comes from.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum VerdictSource {
    /// The caller's own command, from the command line of the process.
    Command(TrustedArgv),
    /// A `tests` or `examples` binding of a capability of `for`, by its path.
    Binding(String),
}

/// Whether a record is new or replaces the one with the same id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum WriteMode {
    /// A new record: a file already at the path is refused.
    #[default]
    Create,
    /// The record that exists at the path is replaced. Nothing of it is kept:
    /// the new record holds exactly what this request gives.
    Replace,
}

/// What to record.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct EvidenceRequest {
    /// The slug of the record, `work/evidence/<id>.md`.
    pub id: String,
    /// The capability ids the evidence covers (the `for` key), at least one.
    pub covers: Vec<String>,
    /// The limits, each naming a value the command prints.
    pub limits: Vec<Measure>,
    /// The commit the execution ran at, when the caller states one: 40 or 64
    /// lowercase hexadecimal digits. It is stored for readers and not used.
    pub revision: Option<String>,
    /// Who records it.
    pub by: Actor,
    /// Where the verdict comes from.
    pub source: VerdictSource,
    /// A new record, or a replacement.
    pub mode: WriteMode,
    /// The SPDX comment lines of the record.
    pub options: RecordOptions,
}

impl EvidenceRequest {
    /// A request to record a new `id` for the capabilities `covers` from
    /// `source`, by `by`, with no limit, no revision and no SPDX lines.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        covers: Vec<String>,
        by: Actor,
        source: VerdictSource,
    ) -> Self {
        Self {
            id: id.into(),
            covers,
            limits: Vec::new(),
            revision: None,
            by,
            source,
            mode: WriteMode::Create,
            options: RecordOptions::new(),
        }
    }

    /// The same request with these limits.
    #[must_use]
    pub fn limits(mut self, limits: Vec<Measure>) -> Self {
        self.limits = limits;
        self
    }

    /// The same request with a revision.
    #[must_use]
    pub fn revision(mut self, revision: impl Into<String>) -> Self {
        self.revision = Some(revision.into());
        self
    }

    /// The same request in another write mode.
    #[must_use]
    pub fn mode(mut self, mode: WriteMode) -> Self {
        self.mode = mode;
        self
    }

    /// The same request with these SPDX options.
    #[must_use]
    pub fn options(mut self, options: RecordOptions) -> Self {
        self.options = options;
        self
    }
}

/// Why a recorded verdict is `fail`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Failed {
    /// The command exited with a failing status: the code, or `None` for a
    /// signal.
    Exited(Option<i32>),
    /// The command ran past its time limit and was killed.
    TimedOut,
    /// The command printed values over their limits, by name.
    OverLimit(Vec<String>),
    /// The bound test or example did not pass.
    Binding(BoundOutcome),
}

/// A record that was written.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Recorded {
    /// The repository-relative path of the record.
    pub path: RelPath,
    /// The record as written.
    pub record: EvidenceRecord,
    /// Every reason the verdict is `fail`, in the order exit status (or
    /// binding run), then limits; empty for `pass`.
    pub failed: Vec<Failed>,
}

impl fmt::Display for Recorded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "recorded: {} ({})",
            self.path,
            self.record.verdict.name()
        )?;
        for (index, failed) in self.failed.iter().enumerate() {
            f.write_str(if index == 0 { ": " } else { "; " })?;
            match failed {
                Failed::Exited(Some(code)) => write!(f, "the command exited with {code}")?,
                Failed::Exited(None) => f.write_str("the command was ended by a signal")?,
                Failed::TimedOut => {
                    f.write_str("the command ran past its time limit and was killed")?;
                }
                Failed::OverLimit(names) => write!(f, "over the limit: {}", names.join(", "))?,
                Failed::Binding(outcome) => match outcome.reason() {
                    Some(reason) => f.write_str(&reason)?,
                    None => f.write_str("the binding did not pass")?,
                },
            }
        }
        Ok(())
    }
}

/// Why no evidence was recorded. In every case nothing was written, and in
/// every case but [`EvidenceError::Measure`], [`EvidenceError::Record`] after
/// a run and the run errors themselves, nothing was started.
#[derive(Debug)]
#[non_exhaustive]
pub enum EvidenceError {
    /// The request names no capability.
    NoCapability,
    /// A capability of `for` is not a capability id or no document has it.
    UnknownCapability {
        /// The id as given.
        id: String,
    },
    /// A capability of `for` has a document that did not load.
    CapabilityNotLoaded {
        /// The id.
        id: String,
        /// The findings about its document.
        findings: Vec<Finding>,
    },
    /// A capability of `for` binds no code, so the record would have no input
    /// whose change makes it stale.
    NoCodeBinding {
        /// The id.
        id: String,
    },
    /// A `code` binding of a capability of `for` does not resolve.
    BindingUnresolved {
        /// The capability id.
        id: String,
        /// The binding path.
        path: String,
        /// What the source index says.
        reason: String,
    },
    /// The digest of a bound symbol cannot be taken.
    Digest(SymbolDigestError),
    /// The revision is not a commit id.
    BadRevision {
        /// The text as given.
        found: String,
    },
    /// A limit, or what the command printed, cannot be read or does not fit.
    Measure(MeasureError),
    /// A limit was given with a binding as the source of the verdict.
    LimitsWithBinding,
    /// The path is no `tests` or `examples` binding of a capability of `for`.
    NoSuchBinding {
        /// The path as given.
        path: String,
        /// The test and example bindings of the capabilities, sorted.
        bound: Vec<String>,
    },
    /// The executor refuses the binding before it starts anything.
    BindingRefused(Refusal),
    /// The source index cannot say which functions are tests.
    Index(LoadError),
    /// The command cannot be started; no record is written.
    CannotStart(ExecError),
    /// The command wrote more than the bound to a stream and was killed.
    OutputTooLarge(ExecError),
    /// A process could not be read to its end.
    Exec(ExecError),
    /// The record cannot be written: its id, its SPDX lines or a value is
    /// refused, a file is already there (or, to replace, none is), or the
    /// write failed.
    Record(RecordError),
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoCapability => f.write_str(
                "the evidence covers no capability; name each with `--for cap:<slug>`",
            ),
            Self::UnknownCapability { id } => {
                write!(f, "`{id}` names no capability document; nothing was run")
            }
            Self::CapabilityNotLoaded { id, findings } => {
                write!(f, "{id} did not load, so what it binds is unknown; nothing was run")?;
                write_findings(f, findings)
            }
            Self::NoCodeBinding { id } => write!(
                f,
                "{id} binds no code, so the evidence would rest on no input whose change makes it stale; bind the code it covers in `bindings.code` first; nothing was run"
            ),
            Self::BindingUnresolved { id, path, reason } => write!(
                f,
                "bindings.code `{path}` of {id} does not resolve: {reason}; fix the binding first; nothing was run"
            ),
            Self::Digest(error) => write!(f, "{error}; nothing was run"),
            Self::BadRevision { found } => write!(
                f,
                "`{found}` is not a commit id: give 40 or 64 lowercase hexadecimal digits, as `git rev-parse HEAD` prints; nothing was run"
            ),
            Self::Measure(error) => write!(f, "{error}; nothing was recorded"),
            Self::LimitsWithBinding => f.write_str(
                "a limit needs a value the command prints, and a binding run prints none the tool reads; record a command to give limits; nothing was run",
            ),
            Self::NoSuchBinding { path, bound } => {
                write!(
                    f,
                    "`{path}` is no test or example binding of the capabilities the evidence is for; nothing was run"
                )?;
                if bound.is_empty() {
                    f.write_str("; they bind no test and no example")
                } else {
                    write!(f, "; they bind {}", bound.join(", "))
                }
            }
            Self::BindingRefused(refusal) => write!(f, "{refusal}; nothing was run"),
            Self::Index(error) => write!(f, "the source index cannot list the tests: {error}"),
            Self::CannotStart(ExecError::Spawn {
                command,
                program,
                message,
                ..
            }) => write!(
                f,
                "cannot start `{command}`: `{program}` {message}; check the program named after `--`; no record was written"
            ),
            Self::CannotStart(error) => write!(f, "{error}; no record was written"),
            Self::OutputTooLarge(error) | Self::Exec(error) => {
                write!(f, "{error}; no record was written")
            }
            Self::Record(error) => error.fmt(f),
        }
    }
}

impl Error for EvidenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Digest(error) => Some(error),
            Self::Measure(error) => Some(error),
            Self::Index(error) => Some(error),
            Self::CannotStart(error) | Self::OutputTooLarge(error) | Self::Exec(error) => {
                Some(error)
            }
            Self::Record(error) => Some(error),
            _ => None,
        }
    }
}

impl From<MeasureError> for EvidenceError {
    fn from(error: MeasureError) -> Self {
        Self::Measure(error)
    }
}

impl From<RecordError> for EvidenceError {
    fn from(error: RecordError) -> Self {
        Self::Record(error)
    }
}

/// What the run gave: the verdict, why it failed, the values.
struct Ran {
    verdict: EvidenceVerdict,
    failed: Vec<Failed>,
    values: Vec<Measure>,
}

impl Workplan<'_> {
    /// Run one execution and write it as the evidence record
    /// `work/evidence/<id>.md` below `root`, the directory that holds `work/`
    /// (see the [module](self) for the rules).
    ///
    /// The documents and the source are those of this load, and `runner`
    /// starts the process in its working directory, which the caller sets to
    /// the workspace root (`SystemRunner::new(Programs::new(root))`). Every
    /// check of the request is made before anything starts: the capabilities,
    /// their code bindings and the digests of the bound symbols, the limits, the
    /// revision, the binding a binding source names, the id, the SPDX lines and
    /// whether a file is at the path. The one file written is the record; no
    /// document is touched.
    ///
    /// ```rust
    /// use raf_okf::Actor;
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::Workplan;
    /// use raf_workplan::evidence::{EvidenceError, EvidenceRequest, VerdictSource};
    /// use raf_workplan::exec::FakeRunner;
    ///
    /// let source = Workspace::load(
    ///     &MemoryTree::from([
    ///         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///         ("src/lib.rs", ""),
    ///     ]),
    ///     &["."],
    /// )?;
    /// let plan = Workplan::load(&MemoryTree::from([("docs/architecture/.keep", "")]), &source);
    /// let request = EvidenceRequest::new(
    ///     "parse-bound",
    ///     vec!["cap:parser".to_string()],
    ///     Actor::Agent,
    ///     VerdictSource::Binding("demo::tests::bound".to_string()),
    /// );
    /// // No such capability: refused before anything starts or is written.
    /// let runner = FakeRunner::new();
    /// let refused = plan.record_evidence(std::path::Path::new("."), &runner, &request);
    /// assert!(matches!(refused, Err(EvidenceError::UnknownCapability { .. })));
    /// assert_eq!(runner.calls().len(), 0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An [`EvidenceError`]; nothing is written in any case.
    pub fn record_evidence(
        &self,
        root: &Path,
        runner: &dyn Runner,
        request: &EvidenceRequest,
    ) -> Result<Recorded, EvidenceError> {
        let inputs = self.evidence_inputs(&request.covers)?;
        measure::check_limits(&request.limits)?;
        if let Some(revision) = &request.revision
            && Commit::new(revision).is_err()
        {
            return Err(EvidenceError::BadRevision {
                found: revision.clone(),
            });
        }
        let (subject, command, binding) = match &request.source {
            VerdictSource::Command(argv) => (EvidenceSubject::Command, Some(argv.text()), None),
            VerdictSource::Binding(path) => {
                if !request.limits.is_empty() {
                    return Err(EvidenceError::LimitsWithBinding);
                }
                let binding = self.run_binding(&request.covers, path)?;
                (EvidenceSubject::Binding(path.clone()), None, Some(binding))
            }
        };
        let build = |verdict, values: Vec<Measure>| {
            let mut record = EvidenceRecord::new(
                request.id.clone(),
                subject.clone(),
                inputs.clone(),
                verdict,
                request.by,
                request.covers.clone(),
            );
            record.command.clone_from(&command);
            record.revision.clone_from(&request.revision);
            record.values = values;
            record.limits.clone_from(&request.limits);
            Record::Evidence(record)
        };
        // The record as it will be, with each limit met by a value equal to
        // it: every value the request gives is judged before a process starts.
        let provisional = build(EvidenceVerdict::Pass, request.limits.clone());
        provisional.text(&request.options)?;
        let path = provisional.path()?;
        target_fits(root, &path, request.mode)?;

        let ran = match (&request.source, binding) {
            (VerdictSource::Command(argv), _) => run_command(runner, argv, &request.limits)?,
            (VerdictSource::Binding(_), Some(binding)) => {
                let executor =
                    Executor::new(self.source(), runner).map_err(EvidenceError::Index)?;
                if let Err(refusal) = executor.plan(binding) {
                    return Err(EvidenceError::BindingRefused(refusal));
                }
                let evidence = executor.run(binding).map_err(run_error)?;
                if evidence.passed() {
                    Ran {
                        verdict: EvidenceVerdict::Pass,
                        failed: Vec::new(),
                        values: Vec::new(),
                    }
                } else {
                    Ran {
                        verdict: EvidenceVerdict::Fail,
                        failed: vec![Failed::Binding(evidence.outcome().clone())],
                        values: Vec::new(),
                    }
                }
            }
            (VerdictSource::Binding(_), None) => unreachable!("a binding source found its binding"),
        };
        let record = build(ran.verdict, ran.values);
        let path = match request.mode {
            WriteMode::Create => record::create(root, &record, &request.options)?,
            WriteMode::Replace => record::replace(root, &record, &request.options)?,
        };
        let Record::Evidence(record) = record else {
            unreachable!("an evidence record was built")
        };
        Ok(Recorded {
            path,
            record,
            failed: ran.failed,
        })
    }

    /// The `symbol` inputs of evidence for `covers`: the digest of each code
    /// binding of each capability, once each, sorted by path.
    fn evidence_inputs(&self, covers: &[String]) -> Result<Vec<Input>, EvidenceError> {
        if covers.is_empty() {
            return Err(EvidenceError::NoCapability);
        }
        let mut inputs: BTreeMap<String, Input> = BTreeMap::new();
        for id in covers {
            self.loaded_capability(id)?;
            let code: Vec<&Binding> = self
                .graph()
                .bindings_of(id)
                .filter(|binding| binding.kind() == BindingKind::Code)
                .collect();
            if code.is_empty() {
                return Err(EvidenceError::NoCodeBinding { id: id.clone() });
            }
            for binding in code {
                if let Err(error) = binding.resolution() {
                    return Err(EvidenceError::BindingUnresolved {
                        id: id.clone(),
                        path: binding.path().to_string(),
                        reason: error.to_string(),
                    });
                }
                if !inputs.contains_key(binding.path()) {
                    let digest = Digest::symbol(self.source(), binding.path())
                        .map_err(EvidenceError::Digest)?;
                    inputs.insert(
                        binding.path().to_string(),
                        Input::new(InputKind::Symbol, binding.path(), digest.to_string()),
                    );
                }
            }
        }
        Ok(inputs.into_values().collect())
    }

    /// The capability `id`, which must be loaded.
    fn loaded_capability(&self, id: &str) -> Result<(), EvidenceError> {
        if self
            .capabilities()
            .iter()
            .any(|cap| cap.value().header.id == id)
        {
            return Ok(());
        }
        match capability_path(id) {
            Some(path) if self.is_present(id) => Err(EvidenceError::CapabilityNotLoaded {
                id: id.to_string(),
                findings: self.findings_of(&path),
            }),
            _ => Err(EvidenceError::UnknownCapability { id: id.to_string() }),
        }
    }

    /// The `tests` or `examples` binding `path` of a capability of `covers`.
    fn run_binding<'a>(
        &'a self,
        covers: &'a [String],
        path: &str,
    ) -> Result<&'a Binding, EvidenceError> {
        let runnable = || {
            covers
                .iter()
                .flat_map(|id| self.graph().bindings_of(id))
                .filter(|binding| {
                    matches!(binding.kind(), BindingKind::Test | BindingKind::Example)
                })
        };
        if let Some(binding) = runnable().find(|binding| binding.path() == path) {
            return Ok(binding);
        }
        let mut bound: Vec<String> = runnable().map(|b| b.path().to_string()).collect();
        bound.sort();
        bound.dedup();
        Err(EvidenceError::NoSuchBinding {
            path: path.to_string(),
            bound,
        })
    }
}

/// Whether the record can be written at `path` below `root` in `mode`, before
/// anything runs: nothing may be there for a new record, and a regular file
/// must be there to replace. The write itself judges it again.
fn target_fits(root: &Path, path: &RelPath, mode: WriteMode) -> Result<(), EvidenceError> {
    let found = std::fs::symlink_metadata(root.join(path.as_str()));
    match (mode, found) {
        (WriteMode::Create, Ok(_)) => Err(RecordError::Exists { path: path.clone() }.into()),
        (WriteMode::Replace, Ok(meta)) if meta.file_type().is_file() => Ok(()),
        (WriteMode::Replace, _) => Err(RecordError::Missing { path: path.clone() }.into()),
        (WriteMode::Create, Err(_)) => Ok(()),
    }
}

/// The error of a process of a run that has no verdict.
fn run_error(error: ExecError) -> EvidenceError {
    match error {
        ExecError::Spawn { .. } => EvidenceError::CannotStart(error),
        ExecError::OutputTooLarge { .. } => EvidenceError::OutputTooLarge(error),
        _ => EvidenceError::Exec(error),
    }
}

/// Run the caller's command and judge it.
fn run_command(
    runner: &dyn Runner,
    argv: &TrustedArgv,
    limits: &[Measure],
) -> Result<Ran, EvidenceError> {
    let output = match runner.run(&Invocation::trusted(argv), &Limits::trusted()) {
        Ok(output) => output,
        // A run killed at its time limit is `fail` whatever its limits ask:
        // the runner keeps no output of a killed process, so it states no
        // value, and its limits are kept as the request gives them.
        Err(ExecError::Timeout { .. }) => {
            return Ok(Ran {
                verdict: EvidenceVerdict::Fail,
                failed: vec![Failed::TimedOut],
                values: Vec::new(),
            });
        }
        Err(error) => return Err(run_error(error)),
    };
    let values = measure::printed(&output.stdout)?;
    let over = measure::over_limits(&values, limits)?;
    let mut failed = Vec::new();
    if !output.success() {
        failed.push(Failed::Exited(output.code));
    }
    if !over.is_empty() {
        failed.push(Failed::OverLimit(over));
    }
    Ok(Ran {
        verdict: if failed.is_empty() {
            EvidenceVerdict::Pass
        } else {
            EvidenceVerdict::Fail
        },
        failed,
        values,
    })
}

/// A statement of a capability document that rests on an evidence record:
/// the paragraph or list item a claim marker follows.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ClaimStatement {
    /// The capability id.
    pub capability: String,
    /// The repository-relative path of its document.
    pub path: String,
    /// The section that holds the marker, or `None` before the first section.
    pub section: Option<CapabilitySection>,
    /// The 1-based line of the marker.
    pub line: usize,
    /// The text of the statement, as written.
    pub statement: String,
}

/// An evidence record that supports a capability.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Support {
    /// The id of the record, `evidence:<slug>`.
    pub evidence: String,
    /// The repository-relative path of the record.
    pub path: String,
    /// The verdict of the record, or `None` when no record loaded under the
    /// id (it was removed, or does not validate).
    pub verdict: Option<EvidenceVerdict>,
    /// Whether the record's `for` lists the capability.
    pub listed: bool,
    /// The lines of the claim markers of the capability that name the record.
    pub claims: Vec<usize>,
}

/// Why a dependency view has no answer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EvidenceLookupError {
    /// The id is not `evidence:<slug>` or a slug.
    BadId {
        /// The id as given.
        id: String,
    },
    /// No evidence record has the id and no claim names it.
    UnknownEvidence {
        /// The id.
        id: String,
    },
    /// The capability is not loaded.
    Capability {
        /// The id as given.
        id: String,
        /// Whether a document of the id exists and did not load.
        not_loaded: bool,
    },
}

impl fmt::Display for EvidenceLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadId { id } => write!(
                f,
                "`{id}` is not an evidence id; write `evidence:<slug>` or the slug"
            ),
            Self::UnknownEvidence { id } => write!(
                f,
                "no evidence record has the id `{id}` and no claim names it"
            ),
            Self::Capability {
                id,
                not_loaded: true,
            } => write!(f, "{id} did not load; fix its document first"),
            Self::Capability { id, .. } => write!(f, "`{id}` names no capability document"),
        }
    }
}

impl Error for EvidenceLookupError {}

/// The id `evidence:<slug>` of `id`, given with or without the prefix.
fn evidence_id(id: &str) -> Result<String, EvidenceLookupError> {
    let prefix = DocType::Evidence.id_prefix();
    let slug = id.strip_prefix(prefix).unwrap_or(id);
    if is_kebab(slug) {
        Ok(format!("{prefix}{slug}"))
    } else {
        Err(EvidenceLookupError::BadId { id: id.to_string() })
    }
}

/// The path of the record `evidence:<slug>`.
pub(crate) fn record_path(id: &str) -> String {
    let slug = id.strip_prefix(DocType::Evidence.id_prefix()).unwrap_or(id);
    format!("{}/{slug}.md", DocType::Evidence.directory())
}

impl Workplan<'_> {
    /// The statements that rest on the evidence `evidence` (`evidence:<slug>`
    /// or the slug): every claim marker of a capability document that names
    /// it, outside PROPOSED blocks, with its file, line, section and the text
    /// of its statement, by capability and line. A claim on a record that did
    /// not load is listed too, because the statement still rests on it.
    ///
    /// # Errors
    ///
    /// [`EvidenceLookupError::BadId`], and
    /// [`EvidenceLookupError::UnknownEvidence`] when no record file has the
    /// id and no claim names it.
    pub fn evidence_claims(
        &self,
        evidence: &str,
    ) -> Result<Vec<ClaimStatement>, EvidenceLookupError> {
        let id = evidence_id(evidence)?;
        let slug = id
            .strip_prefix(DocType::Evidence.id_prefix())
            .unwrap_or(&id);
        let mut found = Vec::new();
        for cap in self.capabilities() {
            let body = cap.body();
            let base = body.span().bytes.start;
            for claim in body.claims() {
                if !claim.evidence().iter().any(|named| named == slug) {
                    continue;
                }
                let span = &claim.statement().bytes;
                let statement = body
                    .text()
                    .get(span.start.saturating_sub(base)..span.end.saturating_sub(base))
                    .unwrap_or_default()
                    .to_string();
                found.push(ClaimStatement {
                    capability: cap.value().header.id.clone(),
                    path: cap.path().to_string(),
                    section: claim.section().and_then(CapabilitySection::from_slug),
                    line: claim.line(),
                    statement,
                });
            }
        }
        if found.is_empty() && !self.is_present(&id) {
            return Err(EvidenceLookupError::UnknownEvidence { id });
        }
        found.sort_by(|a, b| (&a.capability, a.line).cmp(&(&b.capability, b.line)));
        Ok(found)
    }

    /// The entry of `id` in `found`, made from the loaded record when there is
    /// none yet.
    fn support_entry<'m>(
        &self,
        found: &'m mut BTreeMap<String, Support>,
        id: &str,
        capability: &str,
    ) -> &'m mut Support {
        found.entry(id.to_string()).or_insert_with(|| {
            let record = self
                .evidence()
                .iter()
                .find(|record| record.value().header.id == id);
            Support {
                evidence: id.to_string(),
                path: record_path(id),
                verdict: record.map(|record| record.value().verdict),
                listed: record
                    .is_some_and(|record| record.value().covers.iter().any(|c| c == capability)),
                claims: Vec::new(),
            }
        })
    }

    /// The evidence records that support the capability `capability`: those
    /// whose `for` lists it and those its claims name, by id, each with its
    /// verdict and the lines of the claims that name it.
    ///
    /// # Errors
    ///
    /// [`EvidenceLookupError::Capability`] when the capability is not loaded.
    pub fn evidence_for(&self, capability: &str) -> Result<Vec<Support>, EvidenceLookupError> {
        let Some(cap) = self
            .capabilities()
            .iter()
            .find(|cap| cap.value().header.id == capability)
        else {
            return Err(EvidenceLookupError::Capability {
                id: capability.to_string(),
                not_loaded: self.is_present(capability),
            });
        };
        let mut found: BTreeMap<String, Support> = BTreeMap::new();
        for id in self.graph().evidence_of(capability) {
            self.support_entry(&mut found, id, capability);
        }
        let prefix = DocType::Evidence.id_prefix();
        for claim in cap.body().claims() {
            for slug in claim.evidence() {
                let support =
                    self.support_entry(&mut found, &format!("{prefix}{slug}"), capability);
                if !support.claims.contains(&claim.line()) {
                    support.claims.push(claim.line());
                }
            }
        }
        Ok(found.into_values().collect())
    }
}
