// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Recording evidence from the caller's own command, with real processes.
//!
//! A [`TrustedArgv`] can be built from words only inside the crate, so the
//! runs of a command are tested here. The commands are `/bin/sh -c <script>`:
//! the shell is the caller's program, as any program could be, and the
//! workplan itself starts none. No executable is written, so a start can never
//! meet a file still open for writing.

use purrdf_testkit::TempDir;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use raf_okf::{Actor, EvidenceSubject, EvidenceVerdict, Measure, Number};
use raf_source::{DiskTree, Workspace};

use super::{EvidenceError, EvidenceRequest, Failed, MeasureError, VerdictSource, WriteMode};
use crate::digest::Digest;
use crate::exec::{
    BUILD_TIMEOUT, ExecError, FakeRunner, Invocation, Limits, Output, Program, Programs, Runner,
    SystemRunner, TrustedArgv,
};
use crate::record::{RecordError, RecordOptions};
use crate::{LinkContext, Workplan};

/// A workspace on a disk, removed on drop: the crate `demo` with `run` and
/// `other`, and the capability `demo` that binds `demo::run`.
struct Root(TempDir);

const LIB: &str = "pub fn run() {}\npub fn other() {}\n";

fn capability(code: &str) -> String {
    let mut sections = String::new();
    for name in [
        "Purpose",
        "Contract",
        "Usage",
        "Examples",
        "Constraints",
        "Explanation",
        "Verification",
    ] {
        sections.push_str("## ");
        sections.push_str(name);
        sections.push_str("\n\nText.\n\n");
    }
    format!(
        "---\ntype: Capability\nid: cap:demo\ntitle: Demo\ndescription: Demo.\n\
         state: implementing\nbindings:\n  code: {code}\n  tests: []\n  examples: []\n\
         depends: []\n---\n\n# Demo\n\n{sections}"
    )
}

impl Root {
    fn new(tag: &str) -> Self {
        let root = Self(
            TempDir::new_in(std::env::temp_dir())
                .unwrap_or_else(|error| panic!("create {tag} evidence fixture: {error}")),
        );
        root.put(
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[workspace]\n",
        );
        root.put("src/lib.rs", LIB);
        root.put("docs/capabilities/demo.md", &capability("[demo::run]"));
        root
    }

    fn path(&self) -> &Path {
        self.0.path()
    }

    fn put(&self, path: &str, text: &str) {
        let target = self.path().join(path);
        fs::create_dir_all(target.parent().expect("a parent")).expect("a directory");
        fs::write(target, text).expect("written");
    }

    fn read(&self, path: &str) -> Option<String> {
        fs::read_to_string(self.path().join(path)).ok()
    }

    fn record(&self) -> Option<String> {
        self.read("work/evidence/bound.md")
    }

    /// Every file below the root, by path; a symbolic link is listed by its
    /// target and not followed.
    fn files(&self) -> BTreeMap<String, Vec<u8>> {
        fn walk(root: &Path, dir: &Path, into: &mut BTreeMap<String, Vec<u8>>) {
            for entry in fs::read_dir(dir).expect("a listing") {
                let entry = entry.expect("an entry");
                let path = entry.path();
                let name = path
                    .strip_prefix(root)
                    .expect("below")
                    .display()
                    .to_string();
                let kind = entry.file_type().expect("a file type");
                if kind.is_symlink() {
                    let target = fs::read_link(&path).expect("a link target");
                    into.insert(
                        format!("{name} -> link"),
                        target.display().to_string().into_bytes(),
                    );
                } else if kind.is_dir() {
                    walk(root, &path, into);
                } else {
                    into.insert(name, fs::read(&path).expect("bytes"));
                }
            }
        }
        let mut files = BTreeMap::new();
        walk(self.path(), self.path(), &mut files);
        files
    }

    /// Load the repository and record `request` through `runner`.
    fn record_with(
        &self,
        runner: &dyn Runner,
        request: &EvidenceRequest,
    ) -> Result<super::Recorded, EvidenceError> {
        let tree = DiskTree::new(self.path()).expect("a tree");
        let source = Workspace::load(&tree, &["."]).expect("a workspace");
        let plan = Workplan::load_with(&tree, &source, &LinkContext::of_tree(&tree));
        plan.record_evidence(self.path(), runner, request)
    }

    /// Load the repository and record `request` through the real runner,
    /// whose processes start in the root.
    fn run(&self, request: &EvidenceRequest) -> Result<super::Recorded, EvidenceError> {
        self.record_with(&SystemRunner::new(Programs::new(self.path())), request)
    }

    fn digest(&self) -> String {
        let tree = DiskTree::new(self.path()).expect("a tree");
        let source = Workspace::load(&tree, &["."]).expect("a workspace");
        Digest::symbol(&source, "demo::run")
            .expect("a digest")
            .to_string()
    }
}

/// The command `/bin/sh -c <script> sh <args>...`.
fn sh(script: &str, args: &[&str]) -> TrustedArgv {
    let mut words = vec!["/bin/sh".to_string(), "-c".to_string(), script.to_string()];
    words.push("sh".to_string());
    words.extend(args.iter().map(|arg| (*arg).to_string()));
    TrustedArgv::new(words).expect("a command")
}

fn request(argv: TrustedArgv) -> EvidenceRequest {
    EvidenceRequest::new(
        "bound",
        vec!["cap:demo".to_string()],
        Actor::Agent,
        VerdictSource::Command(argv),
    )
}

fn number(text: &str) -> Number {
    Number::parse(text).expect("a number")
}

fn measure(name: &str, value: &str) -> Measure {
    Measure::new(name, number(value))
}

/// A runner that runs through the real one with other limits.
struct Bounded {
    inner: SystemRunner,
    max_output: Option<usize>,
}

impl Runner for Bounded {
    fn run(&self, invocation: &Invocation, limits: &Limits) -> Result<Output, ExecError> {
        let mut limits = *limits;
        if let Some(max) = self.max_output {
            limits = limits.with_max_output(max);
        }
        self.inner.run(invocation, &limits)
    }
}

/// A script that prints a fault, its limits, the neighbour that does not, and
/// the fault expected.
type ParseCase = (
    &'static str,
    &'static [(&'static str, &'static str)],
    &'static str,
    fn(&MeasureError) -> bool,
);

/// A refused request, by name, and the error expected.
type RefusalCase = (&'static str, EvidenceRequest, fn(&EvidenceError) -> bool);

// --- the record --------------------------------------------------------------

#[cfg(unix)]
#[test]
fn a_passing_command_writes_one_record_with_its_command_values_limits_and_input_digests() {
    let root = Root::new("pass");
    let before = root.files();
    // The command prints a line its own text does not hold.
    let argv = sh(
        "printf 'n%sise line\\n' o; echo peak=41; echo ratio=0.5",
        &[],
    );
    let recorded = root
        .run(
            &request(argv.clone())
                .limits(vec![measure("peak", "64")])
                .revision("0123456789abcdef0123456789abcdef01234567")
                .options(
                    RecordOptions::new()
                        .copyright("2026 Example Org <dev@example.org>")
                        .licence("CC-BY-4.0"),
                ),
        )
        .expect("recorded");
    assert_eq!(recorded.path.as_str(), "work/evidence/bound.md");
    assert_eq!(recorded.failed, []);
    let record = &recorded.record;
    assert_eq!(record.verdict, EvidenceVerdict::Pass);
    assert_eq!(record.subject, EvidenceSubject::Command);
    assert_eq!(record.command.as_deref(), Some(argv.text().as_str()));
    assert_eq!(
        record.revision.as_deref(),
        Some("0123456789abcdef0123456789abcdef01234567")
    );
    assert_eq!(
        record.values,
        [measure("peak", "41"), measure("ratio", "0.5")]
    );
    assert_eq!(record.limits, [measure("peak", "64")]);
    assert_eq!(record.covers, ["cap:demo"]);
    assert_eq!(record.by, Actor::Agent);
    assert_eq!(record.inputs.len(), 1);
    assert_eq!(record.inputs[0].name, "demo::run");
    assert_eq!(record.inputs[0].digest, root.digest());

    // Exactly one file was written, and no document moved.
    let after = root.files();
    let mut added: Vec<&String> = after
        .keys()
        .filter(|path| !before.contains_key(*path))
        .collect();
    added.sort();
    assert_eq!(added, ["work/evidence/bound.md"]);
    for (path, bytes) in &before {
        assert_eq!(after.get(path), Some(bytes), "{path} changed");
    }
    // The text holds the result and no output of the command.
    let text = root.record().expect("the record");
    assert!(text.starts_with("---\n# SPDX-FileCopyrightText: 2026 Example Org"));
    assert!(!text.contains("noise"), "{text}");
    assert!(text.contains("verdict: pass"));
    assert!(text.contains("\nrevision: 0123456789abcdef0123456789abcdef01234567\n"));
}

#[cfg(unix)]
#[test]
fn the_exit_status_and_the_limits_decide_the_verdict() {
    for (script, limit, verdict, failed) in [
        ("echo peak=41", "64", EvidenceVerdict::Pass, vec![]),
        ("echo peak=64", "64", EvidenceVerdict::Pass, vec![]),
        (
            "echo peak=80",
            "64",
            EvidenceVerdict::Fail,
            vec![Failed::OverLimit(vec!["peak".to_string()])],
        ),
        (
            "echo peak=41; exit 3",
            "64",
            EvidenceVerdict::Fail,
            vec![Failed::Exited(Some(3))],
        ),
        // A failing status and a value over its limit are both stated.
        (
            "echo peak=80; exit 3",
            "64",
            EvidenceVerdict::Fail,
            vec![
                Failed::Exited(Some(3)),
                Failed::OverLimit(vec!["peak".to_string()]),
            ],
        ),
        (
            "echo peak=41; kill -9 $$",
            "64",
            EvidenceVerdict::Fail,
            vec![Failed::Exited(None)],
        ),
    ] {
        let root = Root::new("verdict");
        let recorded = root
            .run(&request(sh(script, &[])).limits(vec![measure("peak", limit)]))
            .unwrap_or_else(|error| panic!("{script}: {error}"));
        assert_eq!(recorded.record.verdict, verdict, "{script}");
        assert_eq!(recorded.failed, failed, "{script}");
        // A failing run is recorded with what it printed.
        assert_eq!(recorded.record.values.len(), 1, "{script}");
        let text = root.record().expect("a record");
        assert!(
            text.contains(&format!("verdict: {}", verdict.name())),
            "{script}: {text}"
        );
    }
}

#[cfg(unix)]
#[test]
fn the_argv_reaches_the_process_exactly_with_no_shell_of_the_workplan() {
    let root = Root::new("argv");
    let hostile = ["a b", "$(touch pwned)", "`touch pwned`", "*", "", "-x;y"];
    let script = "printf '%s\\0' \"$@\" > argv.bin; echo count=$#";
    let recorded = root.run(&request(sh(script, &hostile))).expect("recorded");
    assert_eq!(recorded.record.values, [measure("count", "6")]);
    let bytes = fs::read(root.path().join("argv.bin")).expect("the argv");
    let mut seen: Vec<&[u8]> = bytes.split(|byte| *byte == 0).collect();
    assert_eq!(seen.pop(), Some(&b""[..]));
    let expected: Vec<&[u8]> = hostile.iter().map(|word| word.as_bytes()).collect();
    assert_eq!(seen, expected);
    assert!(!root.path().join("pwned").exists());
}

#[cfg(unix)]
#[test]
fn the_command_runs_in_the_root_with_its_input_closed_and_the_callers_environment() {
    let root = Root::new("where");
    let canonical = fs::canonicalize(root.path()).expect("a canonical root");
    let canonical = canonical.to_str().expect("a UTF-8 path");
    let script = "test \"$(pwd -P)\" = \"$1\" && echo cwd=1; \
                  if read line; then echo input=1; else echo input=0; fi; \
                  echo lc=${#LC_ALL}; echo path=${#PATH}";
    let recorded = root
        .run(&request(sh(script, &[canonical])))
        .expect("recorded");
    let length = |name: &str| std::env::var(name).map_or(0, |value| value.len());
    assert_eq!(
        recorded.record.values,
        [
            measure("cwd", "1"),
            measure("input", "0"),
            measure("lc", &length("LC_ALL").to_string()),
            measure("path", &length("PATH").to_string()),
        ]
    );
}

#[test]
fn the_command_is_asked_with_the_bounds_of_a_build_and_the_callers_locale() {
    let root = Root::new("limits");
    let runner = FakeRunner::new().on(
        Program::Trusted,
        &["cargo", "bench", "--bench=parse"],
        Ok(Output::new(Some(0), b"peak=1\n".to_vec(), Vec::new())),
    );
    let argv = TrustedArgv::new(
        ["cargo", "bench", "--bench=parse"]
            .map(str::to_string)
            .to_vec(),
    )
    .expect("a command");
    root.record_with(&runner, &request(argv)).expect("recorded");
    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].program(), Program::Trusted);
    assert_eq!(calls[0].command(), "cargo bench --bench=parse");
    assert_eq!(*calls[0].limits(), Limits::trusted());
    assert_eq!(calls[0].limits().max_output(), 16 * 1024 * 1024);
    assert_eq!(calls[0].limits().timeout(), BUILD_TIMEOUT);
    assert!(!calls[0].limits().c_locale());
}

#[test]
fn a_run_killed_at_its_time_limit_records_fail_and_its_neighbour_passes() {
    let script = ["/bin/sh", "-c", "echo peak=1", "sh"];
    let answer = || Output::new(Some(0), b"peak=1\n".to_vec(), Vec::new());
    let past = FakeRunner::new().on(
        Program::Trusted,
        &script,
        Ok(answer().with_elapsed(BUILD_TIMEOUT + Duration::from_millis(1))),
    );
    let root = Root::new("timeout");
    let recorded = root
        .record_with(&past, &request(sh("echo peak=1", &[])))
        .expect("a killed run is recorded");
    assert_eq!(recorded.record.verdict, EvidenceVerdict::Fail);
    assert_eq!(recorded.failed, [Failed::TimedOut]);
    assert_eq!(recorded.record.values.len(), 0);
    let calls = past.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(*calls[0].limits(), Limits::trusted());
    // With a limit, the killed run is still recorded as `fail`, with the
    // limit as given and no value: its limit is not "never printed".
    let limited = Root::new("timeout-limit");
    let recorded = limited
        .record_with(
            &past,
            &request(sh("echo peak=1", &[])).limits(vec![measure("peak", "5")]),
        )
        .expect("a killed run with a limit is recorded");
    assert_eq!(recorded.record.verdict, EvidenceVerdict::Fail);
    assert_eq!(recorded.failed, [Failed::TimedOut]);
    assert_eq!(recorded.record.limits, [measure("peak", "5")]);
    assert_eq!(recorded.record.values.len(), 0);
    let text = limited.record().expect("the record");
    assert!(
        text.contains("verdict: fail") && text.contains("limits:\n  peak: 5\n"),
        "{text}"
    );
    // The neighbour that took exactly the limit passes.
    let at = FakeRunner::new().on(
        Program::Trusted,
        &script,
        Ok(answer().with_elapsed(BUILD_TIMEOUT)),
    );
    let quick = Root::new("timeout-neighbour");
    let recorded = quick
        .record_with(&at, &request(sh("echo peak=1", &[])))
        .expect("recorded");
    assert_eq!(recorded.record.verdict, EvidenceVerdict::Pass);
    assert_eq!(recorded.record.values, [measure("peak", "1")]);
}

#[cfg(unix)]
#[test]
fn output_past_the_bound_is_an_error_and_writes_nothing_and_output_at_it_records() {
    let root = Root::new("bound");
    let runner = |max| Bounded {
        inner: SystemRunner::new(Programs::new(root.path())),
        max_output: Some(max),
    };
    // 4096 bytes of a line that is not a measure.
    let script = "i=0; while [ $i -lt 512 ]; do printf 'xxxxxxx\\n'; i=$((i+1)); done";
    let error = root
        .record_with(&runner(1024), &request(sh(script, &[])))
        .expect_err("past the bound");
    assert!(matches!(error, EvidenceError::OutputTooLarge(_)), "{error}");
    assert_eq!(root.record(), None);
    let recorded = root
        .record_with(&runner(4096), &request(sh(script, &[])))
        .expect("at the bound");
    assert_eq!(recorded.record.verdict, EvidenceVerdict::Pass);
}

#[test]
fn a_command_that_cannot_start_writes_no_record() {
    let root = Root::new("spawn");
    let before = root.files();
    let argv = TrustedArgv::new(vec![
        root.path().join("no-such-program").display().to_string(),
    ])
    .expect("a command");
    let error = root.run(&request(argv)).expect_err("cannot start");
    assert!(matches!(error, EvidenceError::CannotStart(_)), "{error}");
    let text = error.to_string();
    assert!(text.contains("no record was written"), "{text}");
    assert!(
        text.contains("check the program named after `--`"),
        "{text}"
    );
    assert!(!text.contains("$CARGO"), "{text}");
    assert_eq!(root.files(), before);
}

#[cfg(unix)]
#[test]
fn each_fault_of_the_printed_values_is_an_error_beside_a_passing_neighbour() {
    let cases: [ParseCase; 5] = [
        ("echo peak=41", &[("rss", "64")], "echo rss=41", |e| {
            matches!(e, MeasureError::LimitNotPrinted { .. })
        }),
        ("echo peak=41MiB", &[], "echo peak=41", |e| {
            matches!(e, MeasureError::NotNumber { .. })
        }),
        ("echo peak=1e-3", &[], "echo peak=0.001", |e| {
            matches!(e, MeasureError::NotNumber { .. }) && e.to_string().contains("no exponent")
        }),
        (
            "echo peak=1; echo peak=1",
            &[],
            "echo peak=1; echo other=1",
            |e| matches!(e, MeasureError::Twice { .. }),
        ),
        // A failing run must still print the values its limits name.
        ("exit 1", &[("peak", "64")], "echo peak=1; exit 1", |e| {
            matches!(e, MeasureError::LimitNotPrinted { .. })
        }),
    ];
    for (bad, limits, good, expected) in cases {
        let limits: Vec<Measure> = limits.iter().map(|(n, v)| measure(n, v)).collect();
        let root = Root::new("parse");
        let error = root
            .run(&request(sh(bad, &[])).limits(limits.clone()))
            .expect_err(bad);
        match &error {
            EvidenceError::Measure(fault) => assert!(expected(fault), "{bad}: {fault}"),
            other => panic!("{bad}: {other}"),
        }
        assert!(
            error.to_string().contains("nothing was recorded"),
            "{error}"
        );
        assert_eq!(root.record(), None, "{bad}");
        root.run(&request(sh(good, &[])).limits(limits))
            .unwrap_or_else(|error| panic!("{good}: {error}"));
        assert!(root.record().is_some(), "{good}");
    }
}

// --- refusals before anything starts -----------------------------------------

#[test]
fn every_refusal_of_the_request_starts_nothing_and_writes_nothing() {
    let root = Root::new("refusals");
    let argv = || TrustedArgv::new(vec!["true".to_string()]).expect("a command");
    let cases: Vec<RefusalCase> = vec![
        (
            "no capability",
            EvidenceRequest::new(
                "bound",
                Vec::new(),
                Actor::Human,
                VerdictSource::Command(argv()),
            ),
            |e| matches!(e, EvidenceError::NoCapability),
        ),
        (
            "unknown capability",
            EvidenceRequest::new(
                "bound",
                vec!["cap:ghost".to_string()],
                Actor::Human,
                VerdictSource::Command(argv()),
            ),
            |e| matches!(e, EvidenceError::UnknownCapability { .. }),
        ),
        (
            "a limit given twice",
            request(argv()).limits(vec![measure("peak", "1"), measure("peak", "2")]),
            |e| matches!(e, EvidenceError::Measure(MeasureError::Twice { .. })),
        ),
        (
            "a revision that is not a commit",
            request(argv()).revision("HEAD"),
            |e| matches!(e, EvidenceError::BadRevision { .. }),
        ),
        (
            "a slug that is not one",
            EvidenceRequest::new(
                "Bad Slug",
                vec!["cap:demo".to_string()],
                Actor::Human,
                VerdictSource::Command(argv()),
            ),
            |e| matches!(e, EvidenceError::Record(RecordError::BadSlug { .. })),
        ),
        (
            "one SPDX line only",
            request(argv()).options(RecordOptions::new().licence("CC-BY-4.0")),
            |e| matches!(e, EvidenceError::Record(RecordError::SpdxUnset { .. })),
        ),
        (
            "a replacement of a record that is not there",
            request(argv()).mode(WriteMode::Replace),
            |e| matches!(e, EvidenceError::Record(RecordError::Missing { .. })),
        ),
        (
            "a limit with a binding",
            EvidenceRequest::new(
                "bound",
                vec!["cap:demo".to_string()],
                Actor::Human,
                VerdictSource::Binding("demo::tests::x".to_string()),
            )
            .limits(vec![measure("peak", "1")]),
            |e| matches!(e, EvidenceError::LimitsWithBinding),
        ),
        (
            "a binding that is no test or example of the capability",
            EvidenceRequest::new(
                "bound",
                vec!["cap:demo".to_string()],
                Actor::Human,
                VerdictSource::Binding("demo::run".to_string()),
            ),
            |e| matches!(e, EvidenceError::NoSuchBinding { .. }),
        ),
    ];
    let before = root.files();
    for (name, request, expected) in cases {
        let runner = FakeRunner::new();
        let error = root.record_with(&runner, &request).expect_err(name);
        assert!(expected(&error), "{name}: {error}");
        assert!(runner.calls().is_empty(), "{name}: a process started");
        assert_eq!(root.files(), before, "{name}");
    }
    // The neighbour: the plain request records.
    let runner = FakeRunner::new().on(
        Program::Trusted,
        &["true"],
        Ok(Output::new(Some(0), Vec::new(), Vec::new())),
    );
    root.record_with(&runner, &request(argv()))
        .expect("recorded");
    assert!(root.record().is_some());
}

#[test]
fn a_capability_with_no_code_binding_or_one_that_does_not_resolve_is_refused() {
    let runner = FakeRunner::new().on(
        Program::Trusted,
        &["true"],
        Ok(Output::new(Some(0), Vec::new(), Vec::new())),
    );
    let argv = || TrustedArgv::new(vec!["true".to_string()]).expect("a command");
    for (code, expected) in [
        ("[]", "binds no code"),
        ("[demo::gone]", "does not resolve"),
        ("[demo::run, demo::gone]", "does not resolve"),
    ] {
        let root = Root::new("unbound");
        root.put("docs/capabilities/demo.md", &capability(code));
        let error = root.record_with(&runner, &request(argv())).expect_err(code);
        assert!(error.to_string().contains(expected), "{code}: {error}");
        assert_eq!(root.record(), None);
    }
    assert_eq!(runner.calls().len(), 0);
    // Two code bindings that resolve give two inputs, sorted.
    let root = Root::new("bound-two");
    root.put(
        "docs/capabilities/demo.md",
        &capability("[demo::run, demo::other]"),
    );
    let recorded = root
        .record_with(&runner, &request(argv()))
        .expect("recorded");
    let names: Vec<&str> = recorded
        .record
        .inputs
        .iter()
        .map(|input| input.name.as_str())
        .collect();
    assert_eq!(names, ["demo::other", "demo::run"]);
}

#[test]
fn a_revision_is_metadata_with_exact_lowercase_commit_widths() {
    let root = Root::new("revision-width");
    let argv = || TrustedArgv::new(vec!["true".to_string()]).expect("a command");
    for revision in [
        "a".repeat(39),
        "a".repeat(41),
        "a".repeat(63),
        "a".repeat(65),
        "A".repeat(40),
        "g".repeat(64),
    ] {
        let before = root.files();
        let runner = FakeRunner::new();
        let error = root
            .record_with(&runner, &request(argv()).revision(&revision))
            .expect_err("not a canonical commit");
        assert!(
            matches!(error, EvidenceError::BadRevision { .. }),
            "{error:?}"
        );
        assert_eq!(runner.calls(), []);
        assert_eq!(root.files(), before);
    }
    for (width, mode) in [(40, WriteMode::Create), (64, WriteMode::Replace)] {
        let revision = "a".repeat(width);
        let runner = FakeRunner::new().on(
            Program::Trusted,
            &["true"],
            Ok(Output::new(Some(0), Vec::new(), Vec::new())),
        );
        let recorded = root
            .record_with(&runner, &request(argv()).revision(&revision).mode(mode))
            .expect("canonical metadata");
        assert_eq!(recorded.record.revision.as_deref(), Some(revision.as_str()));
        assert_eq!(recorded.record.verdict, EvidenceVerdict::Pass);
        assert_eq!(runner.calls().len(), 1);
        assert_eq!(runner.calls()[0].program(), Program::Trusted);
    }
}

#[test]
fn a_capability_that_does_not_load_refuses_before_any_command_runs() {
    let root = Root::new("unloaded-capability");
    let original = capability("[demo::run]");
    root.put(
        "docs/capabilities/demo.md",
        &original.replace("state: implementing", "state: unknown"),
    );
    let before = root.files();
    let argv = || TrustedArgv::new(vec!["true".to_string()]).expect("a command");
    let runner = FakeRunner::new();
    let error = root
        .record_with(&runner, &request(argv()))
        .expect_err("unloaded capability");
    assert!(
        matches!(&error, EvidenceError::CapabilityNotLoaded { id, findings } if id == "cap:demo" && !findings.is_empty()),
        "{error:?}"
    );
    assert_eq!(runner.calls(), []);
    assert_eq!(root.files(), before);
    root.put("docs/capabilities/demo.md", &original);
    let runner = FakeRunner::new().on(
        Program::Trusted,
        &["true"],
        Ok(Output::new(Some(0), Vec::new(), Vec::new())),
    );
    assert_eq!(
        root.record_with(&runner, &request(argv()))
            .expect("loaded capability")
            .record
            .verdict,
        EvidenceVerdict::Pass
    );
    assert_eq!(runner.calls().len(), 1);
}

#[cfg(unix)]
#[test]
fn relative_and_dot_path_commands_resolve_at_the_workspace_root_from_a_nested_caller() {
    use std::os::unix::fs::PermissionsExt as _;

    const CHILD: &str = "RAF_EVIDENCE_ROOT_PROGRAM_CHILD";
    const PROGRAM: &str = "RAF_EVIDENCE_ROOT_PROGRAM";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        assert_eq!(
            std::env::current_dir().expect("caller directory"),
            fs::canonicalize(root.join("nested")).expect("nested directory")
        );
        let tree = DiskTree::new(&root).expect("a tree");
        let source = Workspace::load(&tree, &["."]).expect("a workspace");
        let plan = Workplan::load_with(&tree, &source, &LinkContext::of_tree(&tree));
        let program = std::env::var(PROGRAM).expect("the selected relative command");
        let argv = TrustedArgv::new(vec![program.clone()]).expect("a command");
        let recorded = plan
            .record_evidence(
                &root,
                &SystemRunner::new(Programs::new(&root)),
                &request(argv),
            )
            .expect("the root command records");
        assert_eq!(recorded.record.values, [measure("chosen", "7")]);
        assert_eq!(recorded.record.command.as_deref(), Some(program.as_str()));
        return;
    }
    let root = Root::new("root-program");
    for (path, value) in [("fixture-tool", 7), ("nested/fixture-tool", 9)] {
        root.put(path, &format!("#!/bin/sh\necho chosen={value}\n"));
        fs::set_permissions(root.path().join(path), fs::Permissions::from_mode(0o755))
            .expect("an executable fixture");
    }
    let before = root.files();
    for program in ["./fixture-tool", "fixture-tool"] {
        let output = std::process::Command::new(std::env::current_exe().expect("the current test binary"))
            .args(["--exact", "evidence::tests::relative_and_dot_path_commands_resolve_at_the_workspace_root_from_a_nested_caller", "--nocapture"])
            .env(CHILD, root.path())
            .env(PROGRAM, program)
            .env("PATH", ".")
            .current_dir(root.path().join("nested"))
            .output()
            .expect("the isolated nested caller runs");
        assert!(output.status.success(), "{program}: {output:?}");
        let mut after = root.files();
        assert!(after.remove("work/evidence/bound.md").is_some());
        assert_eq!(after, before);
        fs::remove_file(root.path().join("work/evidence/bound.md"))
            .expect("remove the previous result");
    }
}

#[test]
fn a_new_record_never_replaces_one_and_a_replacement_keeps_nothing_of_it() {
    let root = Root::new("replace");
    let answer = |stdout: &[u8]| {
        FakeRunner::new().on(
            Program::Trusted,
            &["true"],
            Ok(Output::new(Some(0), stdout.to_vec(), Vec::new())),
        )
    };
    let argv = || TrustedArgv::new(vec!["true".to_string()]).expect("a command");
    root.record_with(
        &answer(b"peak=1\n"),
        &request(argv()).limits(vec![measure("peak", "2")]),
    )
    .expect("recorded");
    let first = root.record().expect("the record");
    // A second new record with the id is refused before the run.
    let runner = answer(b"peak=1\n");
    let error = root
        .record_with(&runner, &request(argv()))
        .expect_err("exists");
    assert!(
        matches!(error, EvidenceError::Record(RecordError::Exists { .. })),
        "{error}"
    );
    assert_eq!(runner.calls().len(), 0);
    assert_eq!(root.record().as_deref(), Some(first.as_str()));
    // A replacement holds what its request gives: no limit is carried over.
    let recorded = root
        .record_with(
            &answer(b"peak=5\n"),
            &request(argv()).mode(WriteMode::Replace),
        )
        .expect("replaced");
    assert_eq!(recorded.record.limits.len(), 0);
    assert_eq!(recorded.record.values, [measure("peak", "5")]);
    let text = root.record().expect("the record");
    assert!(!text.contains("limits:"), "{text}");
}

#[test]
fn the_recorded_line_says_why_a_verdict_is_fail() {
    let root = Root::new("shown");
    let runner = FakeRunner::new().on(
        Program::Trusted,
        &["true"],
        Ok(Output::new(Some(0), b"peak=80\n".to_vec(), Vec::new())),
    );
    let recorded = root
        .record_with(
            &runner,
            &request(TrustedArgv::new(vec!["true".to_string()]).expect("a command"))
                .limits(vec![measure("peak", "64")]),
        )
        .expect("recorded");
    assert_eq!(
        recorded.to_string(),
        "recorded: work/evidence/bound.md (fail): over the limit: peak"
    );
}

/// The values a floating-point comparison misjudges, each over its limit, and
/// a neighbour equal to the limit and one just under it.
fn exact() -> Vec<(String, String, String)> {
    let one_e400 = format!("1{}", "0".repeat(400));
    vec![
        (
            "9007199254740993".to_string(),
            "9007199254740992".to_string(),
            "9007199254740991".to_string(),
        ),
        (
            "0.30000000000000001".to_string(),
            "0.3".to_string(),
            "0.29999999999999999".to_string(),
        ),
        (
            "1.0000000000000000001".to_string(),
            "1".to_string(),
            "0.9999999999999999999".to_string(),
        ),
        (
            format!("2{}", "0".repeat(400)),
            one_e400.clone(),
            one_e400[..400].to_string(),
        ),
    ]
}

#[test]
fn a_value_over_its_limit_by_less_than_a_float_tells_apart_records_fail() {
    for (over, limit, under) in exact() {
        for (value, verdict) in [
            (&over, EvidenceVerdict::Fail),
            (&limit, EvidenceVerdict::Pass),
            (&under, EvidenceVerdict::Pass),
        ] {
            let root = Root::new("exact");
            let runner = FakeRunner::new().on(
                Program::Trusted,
                &["true"],
                Ok(Output::new(
                    Some(0),
                    format!("peak={value}\n").into_bytes(),
                    Vec::new(),
                )),
            );
            let recorded = root
                .record_with(
                    &runner,
                    &request(TrustedArgv::new(vec!["true".to_string()]).expect("a command"))
                        .limits(vec![measure("peak", &limit)]),
                )
                .unwrap_or_else(|error| panic!("{value} against {limit}: {error}"));
            assert_eq!(recorded.record.verdict, verdict, "{value} against {limit}");
        }
    }
}

#[test]
fn more_values_than_a_record_keeps_is_an_error_and_the_most_it_keeps_records() {
    let lines = |count: usize| -> Vec<u8> {
        let mut text = String::new();
        for index in 0..count {
            text.push('v');
            text.push_str(&index.to_string());
            text.push_str("=1\n");
        }
        text.into_bytes()
    };
    for (count, records) in [(super::MAX_VALUES, true), (super::MAX_VALUES + 1, false)] {
        let root = Root::new("many");
        let runner = FakeRunner::new().on(
            Program::Trusted,
            &["true"],
            Ok(Output::new(Some(0), lines(count), Vec::new())),
        );
        let result = root.record_with(
            &runner,
            &request(TrustedArgv::new(vec!["true".to_string()]).expect("a command")),
        );
        if records {
            assert_eq!(result.expect("recorded").record.values.len(), count);
        } else {
            assert!(
                matches!(
                    result,
                    Err(EvidenceError::Measure(MeasureError::TooMany { limit }))
                        if limit == super::MAX_VALUES
                ),
                "{count}"
            );
            assert_eq!(root.record(), None);
        }
    }
}

// --- the way to the record ----------------------------------------------------

#[test]
fn a_regular_file_on_the_way_to_the_record_refuses_before_any_command_runs() {
    for blocked in ["work", "work/evidence"] {
        let root = Root::new("file-preflight");
        root.put("work/evidence/bound.md", "original\n");
        let tree = DiskTree::new(root.path()).expect("a tree");
        let source = Workspace::load(&tree, &["."]).expect("a workspace");
        let plan = Workplan::load_with(&tree, &source, &LinkContext::of_tree(&tree));
        fs::rename(root.path().join(blocked), root.path().join("moved")).unwrap();
        fs::write(root.path().join(blocked), "not a directory\n").unwrap();
        let before = root.files();
        for mode in [WriteMode::Create, WriteMode::Replace] {
            let runner = FakeRunner::new();
            let argv = TrustedArgv::new(vec!["true".to_string()]).unwrap();
            let error = plan
                .record_evidence(root.path(), &runner, &request(argv).mode(mode))
                .expect_err("a regular file cannot be a record directory");
            assert!(
                matches!(&error, EvidenceError::Record(RecordError::Write { path, .. }) if path == blocked),
                "{blocked} {mode:?}: {error}"
            );
            assert!(runner.calls().is_empty(), "{blocked} {mode:?}");
            assert_eq!(root.files(), before, "{blocked} {mode:?}");
        }

        fs::remove_file(root.path().join(blocked)).unwrap();
        fs::rename(root.path().join("moved"), root.path().join(blocked)).unwrap();
        for mode in [WriteMode::Replace, WriteMode::Create] {
            if mode == WriteMode::Create {
                fs::remove_file(root.path().join("work/evidence/bound.md")).unwrap();
            }
            let runner = FakeRunner::new().on(
                Program::Trusted,
                &["true"],
                Ok(Output::new(Some(0), Vec::new(), Vec::new())),
            );
            let argv = TrustedArgv::new(vec!["true".to_string()]).unwrap();
            plan.record_evidence(root.path(), &runner, &request(argv).mode(mode))
                .expect("the restored real directory records");
            assert_eq!(runner.calls().len(), 1);
            assert_ne!(root.record().as_deref(), Some("original\n"));
        }
    }
}

/// A directory link inside the root at `work` or `work/evidence`, with a file
/// of the record's name behind it, is refused before the command runs, for a
/// new record and for a replacement, and nothing is written; once the
/// directory is a real one again, both record.
#[cfg(unix)]
#[test]
fn a_directory_link_on_the_way_to_the_record_is_refused_before_the_command_runs() {
    let argv = || TrustedArgv::new(vec!["true".to_string()]).expect("a command");
    let passing = || {
        FakeRunner::new().on(
            Program::Trusted,
            &["true"],
            Ok(Output::new(Some(0), Vec::new(), Vec::new())),
        )
    };
    for link in ["work", "work/evidence"] {
        let root = Root::new("linked");
        root.put("work/evidence/bound.md", "mine\n");
        // The workspace and the documents are read before the link is made:
        // the default package discovery refuses a linked directory, and a
        // library caller may name other source roots.
        let tree = DiskTree::new(root.path()).expect("a tree");
        let source = Workspace::load(&tree, &["."]).expect("a workspace");
        let plan = Workplan::load_with(&tree, &source, &LinkContext::of_tree(&tree));
        fs::rename(root.path().join(link), root.path().join("moved")).expect("moves");
        let up = "../".repeat(link.matches('/').count());
        std::os::unix::fs::symlink(format!("{up}moved"), root.path().join(link)).expect("links");
        let before = root.files();
        for mode in [WriteMode::Create, WriteMode::Replace] {
            let runner = FakeRunner::new();
            let error = plan
                .record_evidence(root.path(), &runner, &request(argv()).mode(mode))
                .expect_err(link);
            assert!(
                matches!(&error, EvidenceError::Record(RecordError::Write { path, .. }) if path == link),
                "{link} {mode:?}: {error}"
            );
            assert!(
                runner.calls().is_empty(),
                "{link} {mode:?}: a process started"
            );
            assert_eq!(root.files(), before, "{link} {mode:?}");
        }
        // The neighbour: the directory is real again.
        fs::remove_file(root.path().join(link)).expect("unlinks");
        fs::rename(root.path().join("moved"), root.path().join(link)).expect("moves back");
        plan.record_evidence(
            root.path(),
            &passing(),
            &request(argv()).mode(WriteMode::Replace),
        )
        .expect("replaced");
        assert_ne!(root.record().as_deref(), Some("mine\n"));
        fs::remove_file(root.path().join("work/evidence/bound.md")).expect("removed");
        plan.record_evidence(root.path(), &passing(), &request(argv()))
            .expect("recorded");
    }
}

/// The reproduction through the documented gate: a workspace member `pkg`,
/// a capability document `docs/capabilities/fixture.md`, and the link
/// `work/evidence -> ../docs/capabilities`. The gate opens, and a replacement
/// of the evidence `fixture` is refused before anything runs: the capability
/// document keeps its text. The neighbour with a real `work/evidence`
/// records.
#[cfg(unix)]
#[test]
fn evidence_through_the_gate_never_replaces_a_capability_document_through_a_link() {
    let root = Root::new("gate-link");
    root.put(
        "Cargo.toml",
        "[workspace]\nmembers = [\"pkg\"]\nresolver = \"2\"\n",
    );
    fs::remove_dir_all(root.path().join("src")).expect("removed");
    root.put(
        "pkg/Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    );
    root.put("pkg/src/lib.rs", LIB);
    fs::rename(
        root.path().join("docs/capabilities/demo.md"),
        root.path().join("docs/capabilities/fixture.md"),
    )
    .expect("renamed");
    let document = capability("[demo::run]").replace("id: cap:demo", "id: cap:fixture");
    root.put("docs/capabilities/fixture.md", &document);
    fs::create_dir_all(root.path().join("work")).expect("work");
    std::os::unix::fs::symlink("../docs/capabilities", root.path().join("work/evidence"))
        .expect("links");
    let opened = crate::Gate::workspace(root.path())
        .source_roots(&["pkg"])
        .tracked_paths(Vec::new())
        .open()
        .expect("the gate opens");
    let argv = TrustedArgv::new(vec!["true".to_string()]).expect("a command");
    let request = EvidenceRequest::new(
        "fixture",
        vec!["cap:fixture".to_string()],
        Actor::Agent,
        VerdictSource::Command(argv),
    )
    .mode(WriteMode::Replace);
    let before = root.files();
    let runner = FakeRunner::new();
    let error = opened
        .workplan()
        .record_evidence(root.path(), &runner, &request)
        .expect_err("refused");
    assert!(
        matches!(&error, EvidenceError::Record(RecordError::Write { path, .. }) if path == "work/evidence"),
        "{error}"
    );
    assert!(runner.calls().is_empty(), "a process started");
    assert_eq!(
        root.read("docs/capabilities/fixture.md").as_deref(),
        Some(document.as_str())
    );
    assert_eq!(root.files(), before);
    // The neighbour: a real `work/evidence`.
    fs::remove_file(root.path().join("work/evidence")).expect("unlinks");
    let runner = FakeRunner::new().on(
        Program::Trusted,
        &["true"],
        Ok(Output::new(Some(0), Vec::new(), Vec::new())),
    );
    let recorded = opened
        .workplan()
        .record_evidence(root.path(), &runner, &request.mode(WriteMode::Create))
        .expect("recorded");
    assert_eq!(recorded.path.as_str(), "work/evidence/fixture.md");
    assert_eq!(
        root.read("docs/capabilities/fixture.md").as_deref(),
        Some(document.as_str())
    );
}
