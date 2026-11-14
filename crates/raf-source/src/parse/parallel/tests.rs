// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Per-load synchronization around the actual native parser and explicit joins.

use purrdf_alloc_probe::{CountingAllocator, CurrentThreadWindow};
use purrdf_lex::allocation::{Memory, Resident, Shared};
use std::cell::RefCell;
use std::fmt;
use std::io;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Condvar, Mutex};

use super::{ParallelOptions, ParallelTree};
use crate::{
    Entry, EntryKind, FileId, LoadOptions, MemoryTree, Opened, RelPath, Tree, TreeError, Workspace,
};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

thread_local! {
    static CURRENT: RefCell<Option<Shared<Probe>>> = const { RefCell::new(None) };
}

#[derive(Default)]
struct Observed {
    active_parsers: usize,
    peak_parsers: usize,
    pair_arrivals: u8,
    sibling_parsed: bool,
    extraction: Vec<String>,
    started: usize,
    joined: usize,
    disposed: usize,
    disposed_after_join: usize,
    observe_preparation: bool,
    preparing: bool,
    preparation_attempts: usize,
    preparation_peak: i64,
    read_ahead: usize,
}

pub(super) struct Probe {
    pair: bool,
    worker_panic: bool,
    observed: Mutex<Observed>,
    changed: Condvar,
}

impl Probe {
    fn new(pair: bool, worker_panic: bool) -> Shared<Self> {
        let mut resident = Resident;
        Memory::new(&mut resident)
            .add_bytes(Shared::<Self>::allocation_layout().size())
            .expect("admit the per-load fixture owner layout");
        Shared::try_new(Self {
            pair,
            worker_panic,
            observed: Mutex::default(),
            changed: Condvar::new(),
        })
        .expect("allocate the admitted per-load fixture owner")
    }

    pub(super) fn started(&self) {
        self.observed.lock().expect("probe lock").started += 1;
    }

    pub(super) fn joined(&self) {
        self.observed.lock().expect("probe lock").joined += 1;
    }
}

pub(super) struct Installed(Option<Shared<Probe>>);

pub(super) fn install(probe: Option<Shared<Probe>>) -> Installed {
    Installed(CURRENT.with(|current| current.replace(probe)))
}

impl Drop for Installed {
    fn drop(&mut self) {
        CURRENT.with(|current| {
            current.replace(self.0.take());
        });
    }
}

pub(super) fn current() -> Option<Shared<Probe>> {
    CURRENT.with(|current| current.borrow().clone())
}

/// Observe only the coordinator's preparation; workers are not the subject of
/// this bound. The whole-process performance driver retains its separate scope.
pub(crate) struct Preparation {
    window: Option<CurrentThreadWindow>,
    probe: Shared<Probe>,
}

pub(crate) fn preparing() -> Option<Preparation> {
    let probe = current()?;
    {
        let mut state = probe.observed.lock().expect("probe lock");
        if !state.observe_preparation {
            return None;
        }
        state.preparing = true;
    }
    Some(Preparation {
        window: Some(CurrentThreadWindow::open()),
        probe,
    })
}

impl Drop for Preparation {
    fn drop(&mut self) {
        let measured = self.window.take().expect("one preparation window").close();
        let mut state = self.probe.observed.lock().expect("probe lock");
        state.preparing = false;
        state.preparation_attempts += 1;
        state.preparation_peak = state.preparation_peak.max(measured.peak_working_bytes);
    }
}

pub(in crate::parse) struct Parsing(Shared<Probe>);

impl Drop for Parsing {
    fn drop(&mut self) {
        self.0.observed.lock().expect("probe lock").active_parsers -= 1;
    }
}

pub(in crate::parse) fn parsing(file: &RelPath) -> Option<Parsing> {
    let probe = current()?;
    if probe.worker_panic && file.as_str() == "src/a.rs" {
        panic::panic_any(SelectedPanic);
    }
    {
        let mut state = probe.observed.lock().expect("probe lock");
        state.active_parsers += 1;
        state.peak_parsers = state.peak_parsers.max(state.active_parsers);
        if probe.pair {
            let bit = match file.as_str() {
                "src/a.rs" => 1,
                "src/b.rs" => 2,
                _ => 0,
            };
            if bit != 0 {
                state.pair_arrivals |= bit;
                probe.changed.notify_all();
                while state.pair_arrivals != 3 {
                    state = probe.changed.wait(state).expect("probe lock");
                }
            }
            if file.as_str() == "src/a/child.rs" {
                while !state.sibling_parsed {
                    state = probe.changed.wait(state).expect("probe lock");
                }
            }
        }
    }
    Some(Parsing(probe))
}

pub(super) fn parsed(file: &RelPath) {
    if let Some(probe) = current()
        && file.as_str() == "src/b.rs"
    {
        probe.observed.lock().expect("probe lock").sibling_parsed = true;
        probe.changed.notify_all();
    }
}

pub(super) fn extracting(file: &RelPath) {
    if let Some(probe) = current() {
        probe
            .observed
            .lock()
            .expect("probe lock")
            .extraction
            .push(file.to_string());
    }
}

const MANIFEST: &str = "[package]\nname=\"demo\"\nversion=\"0.1.0\"\nedition=\"2021\"\n";

#[test]
fn native_parser_activations_overlap_and_the_head_advances_with_a_parked_sibling() {
    let tree = MemoryTree::from([
        ("Cargo.toml", MANIFEST),
        ("src/lib.rs", "mod a; mod b;"),
        ("src/a.rs", "mod child;"),
        ("src/a/child.rs", "pub fn child() {}"),
        ("src/b.rs", "pub fn b() {}"),
    ]);
    let serial = Workspace::load(&tree, &["."]).expect("serial loads");
    let probe = Probe::new(true, false);
    let _installed = install(Some(probe.clone()));
    let parallel = Workspace::par_load(
        &tree,
        &["."],
        &LoadOptions::default(),
        &ParallelOptions::default(),
    )
    .expect("head reaches descendant while sibling waits");
    assert_eq!(serial, parallel);
    assert_eq!(serial.usage(), parallel.usage());
    let state = probe.observed.lock().expect("probe lock");
    assert!(state.peak_parsers >= 2);
    assert_eq!(
        state.extraction,
        ["src/lib.rs", "src/a.rs", "src/a/child.rs", "src/b.rs"]
    );
    assert_eq!(state.started, state.joined);
}

/// Run one parallel worker body on a fresh thread. The released control makes a
/// parsed file end its body without extraction, so `Ok(None)` is a parse.
fn run_worker(file: &RelPath) -> Result<bool, crate::LoadError> {
    let text = "pub fn f() {}\n".to_owned();
    let scan = crate::prescan::prescan_file(file, &text, usize::MAX).expect("prescan");
    let control = super::Control::default();
    control.set(super::Grant::Cancel);
    let input = super::WorkerInput {
        file: file.clone(),
        text,
        scan,
        control: &control,
        probe: None,
    };
    std::thread::scope(|scope| {
        scope
            .spawn(move || input.run().map(|parsed| parsed.is_none()))
            .join()
            .expect("joined")
    })
}

#[test]
fn a_parallel_worker_whose_registry_a_host_callback_populated_is_refused_before_it_parses() {
    let file = RelPath::new("src/parallel_occupied.rs").expect("path");
    let error = crate::parse::entry::tests::with_host(
        &file,
        crate::parse::entry::tests::lex_foreign,
        || run_worker(&file),
    )
    .expect_err("an occupied registry refuses the parallel worker");
    assert!(
        matches!(&error.kind, crate::LoadErrorKind::Registry { first } if first != "<parsed string 1>"),
        "{error}"
    );
}

#[test]
fn a_parallel_worker_with_a_fresh_registry_parses() {
    let file = RelPath::new("src/parallel_fresh.rs").expect("path");
    let parsed = crate::parse::entry::tests::with_host(&file, || {}, || run_worker(&file))
        .expect("a fresh registry parses");
    assert!(parsed, "the released control ends before extraction");
}

#[derive(Debug)]
struct SelectedPanic;

struct DisposalWitness(Shared<Probe>);

impl fmt::Debug for DisposalWitness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DisposalWitness")
    }
}

impl fmt::Display for DisposalWitness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("original opaque source")
    }
}

impl std::error::Error for DisposalWitness {}

impl Drop for DisposalWitness {
    fn drop(&mut self) {
        let mut state = self.0.observed.lock().expect("probe lock");
        state.disposed += 1;
        if state.started == state.joined {
            state.disposed_after_join += 1;
        }
    }
}

struct FailureTree {
    tree: MemoryTree,
    probe: Shared<Probe>,
    coordinator_panic: bool,
    discarded_panic: bool,
}

impl Tree for FailureTree {
    fn read(&self, file: &RelPath, maximum: u64) -> Result<String, TreeError> {
        self.tree.read(file, maximum)
    }

    fn read_unseen(
        &self,
        file: &RelPath,
        maximum: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        if file.as_str() == "src/c.rs" {
            let witness = DisposalWitness(self.probe.clone());
            if self.discarded_panic {
                panic::panic_any(witness);
            }
            return Err(TreeError::Io {
                path: file.to_string(),
                source: io::Error::other(witness),
            });
        }
        self.tree.read_unseen(file, maximum, seen)
    }

    fn list(&self, file: &RelPath) -> Result<Vec<Entry>, TreeError> {
        self.tree.list(file)
    }

    fn entry(&self, file: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        if self.coordinator_panic && file.as_str() == "src/a.rs" {
            panic::panic_any(SelectedPanic);
        }
        self.tree.entry(file)
    }
}

impl ParallelTree for FailureTree {}

#[test]
fn every_worker_is_joined_before_unselected_opaque_errors_or_panics_are_destroyed() {
    // Each case selects a different owning exit: typed failure, worker panic,
    // then coordinator unwinding. Both opaque disposal carriers are exercised.
    for (worker_panic, coordinator_panic, discarded_panic) in [
        (false, false, false),
        (true, false, true),
        (false, true, false),
    ] {
        let probe = Probe::new(false, worker_panic);
        let tree = FailureTree {
            tree: MemoryTree::from([
                ("Cargo.toml", MANIFEST),
                ("src/lib.rs", "mod a; mod b; mod c;"),
                ("src/a.rs", "fn invalid()"),
                ("src/b.rs", "pub fn b() {}"),
                ("src/c.rs", "pub fn c() {}"),
            ]),
            probe: probe.clone(),
            coordinator_panic,
            discarded_panic,
        };
        let _installed = install(Some(probe.clone()));
        let options = ParallelOptions::new(
            std::num::NonZeroUsize::new(4).expect("nonzero"),
            16 * 1024 * 1024,
        )
        .expect("scheduler fits");
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            Workspace::par_load(&tree, &["."], &LoadOptions::default(), &options)
        }));
        if worker_panic || coordinator_panic {
            assert!(
                result
                    .expect_err("selected panic resumes")
                    .is::<SelectedPanic>()
            );
        } else {
            assert!(matches!(
                result
                    .expect("typed failure returns")
                    .expect_err("invalid syntax")
                    .kind,
                crate::LoadErrorKind::Syntax { .. }
            ));
        }
        let state = probe.observed.lock().expect("probe lock");
        assert!(state.started >= 2);
        assert_eq!(state.started, state.joined);
        assert_eq!(state.disposed, 1);
        assert_eq!(state.disposed_after_join, 1);
    }
}

struct PreparationTree<'a> {
    tree: &'a MemoryTree,
    path: &'a str,
    probe: Shared<Probe>,
}

impl Tree for PreparationTree<'_> {
    fn read(&self, path: &RelPath, maximum: u64) -> Result<String, TreeError> {
        self.tree.read(path, maximum)
    }

    fn read_unseen(
        &self,
        path: &RelPath,
        maximum: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        if path.as_str() == self.path {
            let mut state = self.probe.observed.lock().expect("probe lock");
            if state.preparing {
                state.read_ahead += 1;
            }
        }
        self.tree.read_unseen(path, maximum, seen)
    }

    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        self.tree.list(path)
    }
    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        self.tree.entry(path)
    }
}

impl ParallelTree for PreparationTree<'_> {}

#[test]
fn candidate_preparation_is_bounded_before_materializing_a_long_borrowed_path() {
    let name = "p".repeat(64 * 1024);
    let path = format!("src/{name}.rs");
    let source = format!("mod a; #[path=\"{name}.rs\"] mod b;");
    let tree = MemoryTree::from([
        ("Cargo.toml", MANIFEST),
        ("src/lib.rs", source.as_str()),
        ("src/a.rs", "pub fn a() {}"),
        (path.as_str(), "pub fn b() {}"),
    ]);
    let serial = Workspace::load(&tree, &["."]).expect("canonical memory paths load");
    for grant in [32 * 1024, 1024 * 1024] {
        let probe = Probe::new(false, false);
        probe
            .observed
            .lock()
            .expect("probe lock")
            .observe_preparation = true;
        let observed = PreparationTree {
            tree: &tree,
            path: &path,
            probe: probe.clone(),
        };
        let _installed = install(Some(probe.clone()));
        let options = ParallelOptions::new(std::num::NonZeroUsize::new(2).expect("nonzero"), grant)
            .expect("fixed scheduler fits");
        let parallel = Workspace::par_load(&observed, &["."], &LoadOptions::default(), &options)
            .expect("preparation refusal preserves canonical progress");
        assert_eq!(serial, parallel);
        assert_eq!(serial.usage(), parallel.usage());
        let state = probe.observed.lock().expect("probe lock");
        assert!(state.preparation_attempts > 0);
        if grant == 32 * 1024 {
            assert!(state.preparation_peak <= 32 * 1024);
            assert_eq!(state.read_ahead, 0);
        } else {
            assert!(state.preparation_peak > 32 * 1024);
            assert!(state.read_ahead > 0);
        }
    }
}
