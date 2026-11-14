// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Coordinator-owned speculation; native syntax never leaves its fresh worker.

use super::{
    Candidate, Extracted as Parsed, PARSE_STACK_BYTES, Parser, PreparedSource, parse_syntax,
    storage,
};
use crate::facts::extract;
use crate::meter::FactAllowance;
use crate::{
    DiskTree, FileId, LoadError, LoadErrorKind, LoadOptions, MemoryTree, Opened, RelPath, Tree,
    TreeError,
};
#[cfg(test)]
use purrdf_lex::allocation::Shared;
use purrdf_lex::allocation::{Admission, Ceiling, Memory, OwnedAdmission, StorageError};
use std::alloc::Layout;
use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Condvar, Mutex};
use std::thread::{Scope, ScopedJoinHandle};

#[cfg(test)]
pub(crate) mod tests;

/// An explicitly selected Tree capability for native parallel parsing.
///
/// During a parallel load, observations must be stable and independent of call
/// order, including file identities, listings and operation failures. Returned
/// values, transitive opaque error sources and panic payloads exposed to read-ahead
/// must have normal nonpanicking destruction, without depending on speculative
/// disposal or panic-hook effects. These are behavioral eligibility conditions,
/// not an unsafe marker or an immutable-filesystem claim. Stateful observation
/// contracts retain the serial API.
/// The coordinator performs all Tree access; no worker receives this object.
pub trait ParallelTree: Tree + Sync {}

impl ParallelTree for MemoryTree {}
impl ParallelTree for DiskTree {}

/// Independent scheduler ownership limits; source/index options remain unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParallelOptions {
    workers: NonZeroUsize,
    max_speculative_bytes: usize,
}

/// Invalid worker-stack arithmetic or insufficient fixed scheduler ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelOptionsError {
    /// The requested stack or scheduler layout cannot be represented.
    SizeOverflow,
    /// The ownership grant cannot hold the fixed job and synchronization slots.
    SchedulerStorage {
        /// Required bytes for the fixed scheduler layouts.
        required: usize,
        /// Caller-selected ownership ceiling.
        maximum: usize,
    },
}

impl fmt::Display for ParallelOptionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SizeOverflow => f.write_str("parallel worker stack or scheduler size overflows"),
            Self::SchedulerStorage { required, maximum } => write!(
                f,
                "parallel scheduler requires {required} bytes, exceeding {maximum}; increase the speculative ownership grant or reduce workers"
            ),
        }
    }
}
impl Error for ParallelOptionsError {}

fn fixed_bytes(workers: usize) -> Result<usize, ParallelOptionsError> {
    workers
        .checked_mul(PARSE_STACK_BYTES)
        .ok_or(ParallelOptionsError::SizeOverflow)?;
    let controls =
        Layout::array::<Control>(workers).map_err(|_| ParallelOptionsError::SizeOverflow)?;
    let tasks = Layout::array::<Option<Task<'static>>>(workers)
        .map_err(|_| ParallelOptionsError::SizeOverflow)?;
    let inputs = Layout::array::<WorkerInput<'static>>(workers)
        .map_err(|_| ParallelOptionsError::SizeOverflow)?;
    controls
        .size()
        .checked_add(tasks.size())
        .and_then(|bytes| bytes.checked_add(inputs.size()))
        .ok_or(ParallelOptionsError::SizeOverflow)
}

impl ParallelOptions {
    /// Validate an explicit worker count and additional scheduler/input grant.
    ///
    /// Native AST, parser TLS, runtime internals, original backend-returned
    /// failure payloads (including path/name buffers), opaque panic payloads and
    /// worker stacks are not covered by this byte grant. Scheduler-created
    /// copies and carriers remain admitted. Stacks reserve 64 MiB per worker
    /// separately.
    ///
    /// # Errors
    /// Refuses unrepresentable layouts or a grant below the fixed slot layouts.
    pub fn new(
        workers: NonZeroUsize,
        max_speculative_bytes: usize,
    ) -> Result<Self, ParallelOptionsError> {
        let required = fixed_bytes(workers.get())?;
        if required > max_speculative_bytes {
            return Err(ParallelOptionsError::SchedulerStorage {
                required,
                maximum: max_speculative_bytes,
            });
        }
        Ok(Self {
            workers,
            max_speculative_bytes,
        })
    }
}

impl Default for ParallelOptions {
    fn default() -> Self {
        // Two fixed control/job slots fit in this constant grant on the primary
        // native target; the constructor remains the single validation path.
        Self::new(
            NonZeroUsize::new(2).expect("two is nonzero"),
            16 * 1024 * 1024,
        )
        .expect("two scheduler slots fit in the default ownership grant")
    }
}

/// Scheduler high-water counts, finalized only after every worker has joined.
///
/// These counts exclude complete native AST/TLS/runtime allocations and do not
/// change semantic [`crate::Usage`] or workspace structural equality.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParallelUsage {
    /// Most simultaneously admitted jobs, including retained completed outcomes.
    pub peak_live_jobs: usize,
    /// Most requested scheduler and speculative ownership bytes admitted.
    pub peak_scheduler_bytes: usize,
    /// Most requested stack bytes for started workers not yet joined.
    pub peak_worker_stack_bytes: usize,
}

#[derive(Default)]
enum Grant {
    #[default]
    Pending,
    Extract(FactAllowance),
    Cancel,
}

#[derive(Default)]
struct Control {
    grant: Mutex<Grant>,
    changed: Condvar,
}

/// The worker closure captures this one value by ownership. Its layout is
/// admitted once per fixed slot; no separately allocated thread name is used.
struct WorkerInput<'a> {
    file: RelPath,
    text: String,
    scan: crate::prescan::Scan,
    control: &'a Control,
    #[cfg(test)]
    probe: Option<Shared<tests::Probe>>,
}

impl WorkerInput<'_> {
    fn run(self) -> Result<Option<Parsed>, LoadError> {
        #[cfg(test)]
        let _probe = tests::install(self.probe.clone());
        super::entry::claim_registry(&self.file)?;
        let syntax = parse_syntax(&self.file, &self.text);
        #[cfg(test)]
        tests::parsed(&self.file);
        let Some(allowance) = self.control.wait() else {
            return syntax.map(|_| None);
        };
        let tree = syntax?;
        #[cfg(test)]
        tests::extracting(&self.file);
        let facts = extract(&tree, &self.text, allowance);
        drop(tree);
        Ok(Some(Parsed {
            text: self.text,
            facts,
            scan: self.scan,
        }))
    }
}

impl Control {
    fn set(&self, grant: Grant) {
        // No parser or caller code executes while this control lock is held.
        *self
            .grant
            .lock()
            .expect("fixed parser control operations do not panic") = grant;
        self.changed.notify_one();
    }

    fn wait(&self) -> Option<FactAllowance> {
        // Only fixed state assignments occur under this lock.
        let mut grant = self
            .grant
            .lock()
            .expect("fixed parser control operations do not panic");
        loop {
            match std::mem::take(&mut *grant) {
                Grant::Extract(allowance) => return Some(allowance),
                Grant::Cancel => return None,
                Grant::Pending => {
                    grant = self
                        .changed
                        .wait(grant)
                        .expect("fixed parser control operations do not panic");
                }
            }
        }
    }

    fn cancel(&self) {
        // During unwinding recover only to release a parked worker; no poisoned
        // state is accepted as a successful parsing outcome.
        let mut grant = match self.grant.lock() {
            Ok(grant) => grant,
            Err(poison) => poison.into_inner(),
        };
        *grant = Grant::Cancel;
        self.changed.notify_one();
    }
}

struct Account {
    bytes: usize,
    peak: usize,
    admission: OwnedAdmission<Ceiling>,
}

/// One candidate's subview of the original scheduler admission account.
struct PreparationAdmission<'a> {
    account: &'a RefCell<Account>,
    bytes: usize,
}

impl Admission for PreparationAdmission<'_> {
    fn resize(&mut self, bytes: usize) -> Result<(), StorageError> {
        self.account.borrow_mut().with_memory(|memory| {
            if bytes >= self.bytes {
                memory.add_bytes(bytes - self.bytes)
            } else {
                memory.release_bytes(self.bytes - bytes)
            }
        })?;
        self.bytes = bytes;
        Ok(())
    }
}

struct PreparedCandidate<'a> {
    path: RelPath,
    admission: OwnedAdmission<PreparationAdmission<'a>>,
}

impl Account {
    fn with_memory<T>(
        &mut self,
        operation: impl FnOnce(&mut Memory<'_, OwnedAdmission<Ceiling>>) -> T,
    ) -> T {
        let mut memory = Memory::resume(&mut self.admission, self.bytes);
        let result = operation(&mut memory);
        self.bytes = memory.admitted_bytes();
        // All allocations use fresh fixed buffers or a single checked add;
        // no operation grows a previously allocated buffer under this account.
        self.peak = self.peak.max(self.bytes);
        result
    }
}

pub(crate) struct Resources {
    controls: Vec<Control>,
    // Controls die before their original admission guard. Executor borrows keep
    // this same account alive until every task has been destroyed and joined.
    account: RefCell<Account>,
    #[cfg(test)]
    probe: Option<Shared<tests::Probe>>,
}
impl Resources {
    pub(crate) fn new(options: &ParallelOptions) -> Result<Self, LoadError> {
        let mut admission = OwnedAdmission::new(Ceiling(options.max_speculative_bytes));
        let mut controls = Vec::new();
        let mut memory = Memory::new(&mut admission);
        memory
            .reserve(&mut controls, options.workers.get())
            .map_err(|cause| storage(&RelPath::root(), cause))?;
        let inputs = Layout::array::<WorkerInput<'_>>(options.workers.get())
            .map_err(|_| storage(&RelPath::root(), StorageError::SizeOverflow))?;
        memory
            .add_bytes(inputs.size())
            .map_err(|cause| storage(&RelPath::root(), cause))?;
        let bytes = memory.admitted_bytes();
        controls.resize_with(options.workers.get(), Control::default);
        Ok(Self {
            controls,
            account: RefCell::new(Account {
                bytes,
                peak: bytes,
                admission,
            }),
            #[cfg(test)]
            probe: tests::current(),
        })
    }
}

enum Running<'scope> {
    Worker(ScopedJoinHandle<'scope, Result<Option<Parsed>, LoadError>>),
    Failed(LoadError),
    Joined(std::thread::Result<Result<Option<Parsed>, LoadError>>),
    TreeFailed(TreeError),
    Panicked(Box<dyn std::any::Any + Send>),
    Empty,
}

struct Task<'scope> {
    path: RelPath,
    id: Option<FileId>,
    running: Running<'scope>,
    bytes: usize,
    // Final candidate storage retains its original guard through payload drop.
    _path_admission: Option<OwnedAdmission<PreparationAdmission<'scope>>>,
}

struct State<'scope> {
    tasks: Vec<Option<Task<'scope>>>,
    live_jobs: usize,
    stack_bytes: usize,
    usage: ParallelUsage,
}

pub(crate) struct Executor<'scope, 'env> {
    scope: &'scope Scope<'scope, 'env>,
    controls: &'scope [Control],
    account: &'scope RefCell<Account>,
    state: RefCell<State<'scope>>,
    #[cfg(test)]
    probe: Option<Shared<tests::Probe>>,
}

impl<'scope, 'env> Executor<'scope, 'env> {
    pub(crate) fn new(
        scope: &'scope Scope<'scope, 'env>,
        resources: &'scope Resources,
        options: &ParallelOptions,
    ) -> Result<Self, LoadError> {
        let mut tasks = Vec::new();
        resources
            .account
            .borrow_mut()
            .with_memory(|memory| memory.reserve(&mut tasks, options.workers.get()))
            .map_err(|cause| storage(&RelPath::root(), cause))?;
        tasks.resize_with(options.workers.get(), || None);
        Ok(Self {
            scope,
            controls: &resources.controls,
            account: &resources.account,
            state: RefCell::new(State {
                tasks,
                live_jobs: 0,
                stack_bytes: 0,
                usage: ParallelUsage::default(),
            }),
            #[cfg(test)]
            probe: resources.probe.clone(),
        })
    }

    /// Destroy all remaining jobs and slots before reporting a successful load.
    pub(crate) fn finish(self) -> ParallelUsage {
        self.cleanup();
        let mut usage = self.state.borrow().usage;
        usage.peak_scheduler_bytes = self.account.borrow().peak;
        usage
    }

    fn admitted(state: &mut State<'scope>, task: &Task<'scope>) {
        state.live_jobs = state
            .live_jobs
            .checked_add(1)
            .expect("admitted jobs fit the fixed slot count");
        state.usage.peak_live_jobs = state.usage.peak_live_jobs.max(state.live_jobs);
        if matches!(task.running, Running::Worker(_)) {
            state.stack_bytes = state
                .stack_bytes
                .checked_add(PARSE_STACK_BYTES)
                .expect("worker stack total was validated before loading");
            state.usage.peak_worker_stack_bytes =
                state.usage.peak_worker_stack_bytes.max(state.stack_bytes);
        }
    }

    fn joined(state: &mut State<'scope>) {
        state.stack_bytes = state
            .stack_bytes
            .checked_sub(PARSE_STACK_BYTES)
            .expect("joined worker retains its requested stack charge");
    }

    fn cleanup(&self) {
        let mut state = self.state.borrow_mut();
        // Cancellation has no user callbacks. Join every worker into its fixed
        // slot before dropping any opaque error or panic payload.
        for (slot, task) in state.tasks.iter().enumerate() {
            if task.is_some() {
                self.controls[slot].cancel();
            }
        }
        for slot in 0..state.tasks.len() {
            let Some(task) = state.tasks[slot].as_mut() else {
                continue;
            };
            if matches!(task.running, Running::Worker(_)) {
                let Running::Worker(worker) = std::mem::replace(&mut task.running, Running::Empty)
                else {
                    unreachable!("the selected running task is a worker");
                };
                task.running = Running::Joined(worker.join());
                Self::joined(&mut state);
                #[cfg(test)]
                if let Some(probe) = &self.probe {
                    probe.joined();
                }
            }
        }
        for slot in 0..state.tasks.len() {
            let Some(task) = state.tasks[slot].take() else {
                continue;
            };
            let bytes = task.bytes;
            match task.running {
                Running::Joined(result) => drop(result),
                Running::Failed(error) => drop(error),
                Running::TreeFailed(error) => drop(error),
                Running::Panicked(payload) => drop(payload),
                Running::Worker(_) | Running::Empty => {
                    unreachable!("all workers were joined before disposal")
                }
            }
            drop(task.path);
            drop(task.id);
            state.live_jobs = state
                .live_jobs
                .checked_sub(1)
                .expect("discarded job retains its fixed slot charge");
            self.account
                .borrow_mut()
                .with_memory(|memory| memory.release_bytes(bytes))
                .expect("destroyed parser task must retain its original admitted layout");
        }
        let tasks = std::mem::take(&mut state.tasks);
        self.account
            .borrow_mut()
            .with_memory(|memory| memory.release_vec(tasks))
            .expect("destroyed parser slots must retain their original admitted layout");
    }

    fn start(
        &self,
        slot: usize,
        mut task: Task<'scope>,
        text: String,
        max_tokens: usize,
    ) -> Task<'scope> {
        task.running = panic::catch_unwind(AssertUnwindSafe(|| {
            self.start_worker(slot, &task.path, text, max_tokens)
        }))
        .unwrap_or_else(Running::Panicked);
        task
    }

    fn prepare(
        &self,
        candidate: Candidate<'_>,
    ) -> Result<Option<PreparedCandidate<'scope>>, StorageError> {
        let mut admission = OwnedAdmission::new(PreparationAdmission {
            account: self.account,
            bytes: 0,
        });
        let path =
            Memory::new(&mut admission).scope(|memory| Self::prepare_path(candidate, memory))?;
        Ok(path.map(|path| PreparedCandidate { path, admission }))
    }

    fn prepare_path(
        candidate: Candidate<'_>,
        memory: &mut Memory<'_, OwnedAdmission<PreparationAdmission<'_>>>,
    ) -> Result<Option<RelPath>, StorageError> {
        match candidate {
            Candidate::Existing(path) => {
                memory.add_bytes(path.owned_layout()?)?;
                Ok(Some(path.clone()))
            }
            Candidate::Joined {
                base,
                stem,
                input,
                suffix,
            } => {
                let intermediate = if let Some(stem) = stem {
                    let Some(path) = base.join_admitted(stem, memory)? else {
                        return Ok(None);
                    };
                    Some(path)
                } else {
                    None
                };
                let tail = if suffix.is_empty() {
                    None
                } else {
                    Some(memory.format(&format_args!("{input}{suffix}"))?)
                };
                let path = intermediate
                    .as_ref()
                    .unwrap_or(base)
                    .join_admitted(tail.as_deref().unwrap_or(input), memory)?;
                if let Some(tail) = tail {
                    memory.release_string(tail)?;
                }
                if let Some(intermediate) = intermediate {
                    let bytes = intermediate.owned_layout()?;
                    drop(intermediate);
                    memory.release_bytes(bytes)?;
                }
                Ok(path)
            }
        }
    }

    fn start_worker(
        &self,
        slot: usize,
        path: &RelPath,
        text: String,
        max_tokens: usize,
    ) -> Running<'scope> {
        let prepared = PreparedSource::new(path, &text, max_tokens);
        match prepared {
            Err(error) => Running::Failed(error),
            Ok(prepared) => {
                let scan = prepared.scan;
                let control = &self.controls[slot];
                control.set(Grant::Pending);
                let input = WorkerInput {
                    file: path.clone(),
                    text,
                    scan,
                    control,
                    #[cfg(test)]
                    probe: self.probe.clone(),
                };
                match std::thread::Builder::new()
                    .stack_size(PARSE_STACK_BYTES)
                    .spawn_scoped(self.scope, move || input.run())
                {
                    Ok(worker) => {
                        #[cfg(test)]
                        if let Some(probe) = &self.probe {
                            probe.started();
                        }
                        Running::Worker(worker)
                    }
                    Err(source) => {
                        Running::Failed(LoadError::new(path, None, LoadErrorKind::Spawn(source)))
                    }
                }
            }
        }
    }
}

impl Parser for Executor<'_, '_> {
    fn has_capacity(&self) -> bool {
        self.state
            .borrow()
            .tasks
            .iter()
            .skip(1)
            .any(Option::is_none)
    }

    fn resize_frontier(&self, previous: usize, next: usize) -> Result<(), StorageError> {
        self.account.borrow_mut().with_memory(|memory| {
            if next >= previous {
                memory.add_bytes(next - previous)
            } else {
                memory.release_bytes(previous - next)
            }
        })
    }

    fn speculate(
        &self,
        tree: &dyn Tree,
        candidate: Candidate<'_>,
        options: &LoadOptions,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> bool {
        let mut state = self.state.borrow_mut();
        // Slot zero always belongs to canonical progress, including a newly
        // discovered descendant that sorts before every parked sibling.
        let Some(slot) = (1..state.tasks.len()).find(|slot| state.tasks[*slot].is_none()) else {
            return false;
        };
        let Ok(Some(prepared)) = self.prepare(candidate) else {
            return true;
        };
        let path = &prepared.path;
        if state.tasks.iter().flatten().any(|task| task.path == *path) {
            return true;
        }
        let opened = panic::catch_unwind(AssertUnwindSafe(|| {
            tree.read_unseen(path, options.max_file_bytes, &|id| {
                seen(id)
                    || state
                        .tasks
                        .iter()
                        .flatten()
                        .any(|task| task.id.as_ref() == Some(id))
            })
        }));
        let (id, text, deferred) = match opened {
            Ok(Ok(Opened::Seen(_))) => {
                return true;
            }
            Ok(Ok(Opened::Read { id, text })) => (Some(id), Some(text), None),
            Ok(Err(error)) => (None, None, Some(Running::TreeFailed(error))),
            Err(payload) => (None, None, Some(Running::Panicked(payload))),
        };
        let extra = text
            .as_ref()
            .map_or(0, String::capacity)
            .checked_add(id.as_ref().map_or(0, FileId::heap_bytes));
        if extra.is_none()
            || self
                .account
                .borrow_mut()
                .with_memory(|memory| {
                    memory.add_bytes(extra.expect("a representable total has an extra layout"))
                })
                .is_err()
        {
            drop(text);
            drop(id);
            return true;
        }
        let bytes = extra.expect("the admitted layout is representable");
        let PreparedCandidate { path, admission } = prepared;
        let task = Task {
            path,
            id,
            running: deferred.unwrap_or(Running::Empty),
            bytes,
            _path_admission: Some(admission),
        };
        let task = if let Some(text) = text {
            self.start(slot, task, text, options.max_file_tokens)
        } else {
            task
        };
        Self::admitted(&mut state, &task);
        state.tasks[slot] = Some(task);
        true
    }

    fn parse(
        &self,
        path: &RelPath,
        id: &FileId,
        text: String,
        options: &LoadOptions,
        allowance: FactAllowance,
    ) -> Result<Parsed, LoadError> {
        let (slot, task, observed) = {
            let mut state = self.state.borrow_mut();
            if let Some(slot) = state.tasks.iter().position(|task| {
                task.as_ref()
                    .is_some_and(|task| task.id.as_ref() == Some(id))
            }) {
                let task = state.tasks[slot]
                    .take()
                    .expect("the selected task slot is occupied");
                (slot, task, Some(text))
            } else {
                let task = self.start(
                    0,
                    Task {
                        path: path.clone(),
                        id: Some(id.clone()),
                        running: Running::Empty,
                        bytes: 0,
                        _path_admission: None,
                    },
                    text,
                    options.max_file_tokens,
                );
                Self::admitted(&mut state, &task);
                (0, task, None)
            }
        };
        self.controls[slot].set(Grant::Extract(allowance));
        let result = match task.running {
            Running::Failed(error) => Err(error),
            Running::Worker(worker) => {
                let result = worker.join();
                Self::joined(&mut self.state.borrow_mut());
                #[cfg(test)]
                if let Some(probe) = &self.probe {
                    probe.joined();
                }
                match result {
                    Ok(result) => result,
                    Err(payload) => panic::resume_unwind(payload),
                }
            }
            Running::Panicked(payload) => panic::resume_unwind(payload),
            Running::TreeFailed(_) | Running::Joined(_) | Running::Empty => {
                unreachable!("only parser outcomes can be parsed")
            }
        };
        // Destroy the speculative identity owners before refunding their charge.
        // A worker panic retains the grant until Resources destruction instead.
        drop(task.path);
        drop(task.id);
        let result = (|| {
            let mut parsed = result
                .map_err(|mut error| {
                    error.file = path.clone();
                    error
                })?
                .ok_or_else(|| {
                    LoadError::new(
                        path,
                        None,
                        LoadErrorKind::Internal {
                            what: "the canonical parser task was cancelled".into(),
                        },
                    )
                })?;
            if let Some(observed) = observed {
                if observed != parsed.text {
                    return Err(LoadError::new(path, None, LoadErrorKind::Internal {
                        what: "source changed between speculative parsing and canonical observation".into(),
                    }));
                }
                // Keep the bytes from the canonical Tree observation in the index.
                parsed.text = observed;
            }
            Ok(parsed)
        })();
        // The speculative text has now died or been replaced by canonical text;
        // the returned value belongs to the canonical source accounting.
        self.account
            .borrow_mut()
            .with_memory(|memory| memory.release_bytes(task.bytes))
            .map_err(|cause| storage(path, cause))?;
        let mut state = self.state.borrow_mut();
        state.live_jobs = state
            .live_jobs
            .checked_sub(1)
            .expect("consumed job retains its fixed slot charge");
        result
    }
}

impl Drop for Executor<'_, '_> {
    fn drop(&mut self) {
        self.cleanup();
    }
}
