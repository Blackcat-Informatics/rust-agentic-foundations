// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A [`Tree`] that counts what it is asked, so a test can assert how much work
//! a load did instead of how long it took.

use std::cell::RefCell;
use std::collections::BTreeMap;

use raf_source::{Entry, EntryKind, FileId, Opened, RelPath, Tree, TreeError};

/// One kind of question put to a tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Op {
    /// A call to `Tree::read`.
    Read,
    /// A call to `Tree::read_unseen`.
    Open,
    /// A `read_unseen` call that answered `Opened::Read`.
    OpenedRead,
    /// A call to `Tree::identity`.
    Identity,
    /// A call to `Tree::list`.
    List,
    /// A call to `Tree::entry`.
    Entry,
}

/// What a [`Counting`] tree has seen so far.
#[derive(Debug, Default)]
struct Tally {
    /// Calls per operation.
    ops: BTreeMap<Op, usize>,
    /// Calls per operation and path.
    paths: BTreeMap<(Op, String), usize>,
    /// The paths whose text was handed out, in order.
    read_log: Vec<String>,
    /// How many times the text of each file was handed out, by identity.
    reads_by_id: BTreeMap<FileId, usize>,
    /// Every call, whatever its kind, other than `OpenedRead`.
    calls: usize,
}

/// A tree that forwards to `T` and counts each operation and each path.
///
/// With a ceiling, the call that passes it panics naming the operation and
/// the path, so a load that does too much work fails at once on the caller's
/// thread instead of running to the end.
pub(crate) struct Counting<T: Tree> {
    inner: T,
    ceiling: Option<usize>,
    tally: RefCell<Tally>,
}

impl<T: Tree> Counting<T> {
    /// Count the calls made to `inner`, with no ceiling.
    pub(crate) fn new(inner: T) -> Self {
        Self {
            inner,
            ceiling: None,
            tally: RefCell::new(Tally::default()),
        }
    }

    /// Panic on the call that makes the total passes `ceiling`.
    pub(crate) fn with_ceiling(mut self, ceiling: usize) -> Self {
        self.ceiling = Some(ceiling);
        self
    }

    /// The calls of one kind so far.
    pub(crate) fn count(&self, op: Op) -> usize {
        self.tally.borrow().ops.get(&op).copied().unwrap_or(0)
    }

    /// The calls of one kind made with `path`.
    pub(crate) fn count_at(&self, op: Op, path: &str) -> usize {
        self.tally
            .borrow()
            .paths
            .get(&(op, path.to_owned()))
            .copied()
            .unwrap_or(0)
    }

    /// Every call so far, whatever its kind.
    pub(crate) fn total(&self) -> usize {
        self.tally.borrow().calls
    }

    /// The paths whose text was handed out, in order.
    pub(crate) fn read_log(&self) -> Vec<String> {
        self.tally.borrow().read_log.clone()
    }

    /// How many files were read, after checking that none was read twice
    /// under any spelling.
    pub(crate) fn files_read_once(&self) -> usize {
        let tally = self.tally.borrow();
        assert!(
            tally.reads_by_id.values().all(|count| *count == 1),
            "{:?}",
            tally.reads_by_id
        );
        tally.reads_by_id.len()
    }

    /// Record one call and panic if it passes the ceiling.
    fn note(&self, op: Op, path: &RelPath) {
        let mut tally = self.tally.borrow_mut();
        *tally.ops.entry(op).or_default() += 1;
        *tally.paths.entry((op, path.to_string())).or_default() += 1;
        if op != Op::OpenedRead {
            tally.calls += 1;
            if let Some(ceiling) = self.ceiling {
                assert!(
                    tally.calls <= ceiling,
                    "tree operation ceiling {ceiling} passed by {op:?} of {path}"
                );
            }
        }
    }

    /// Record that the text of the file at `path` with identity `id` was
    /// handed out.
    fn handed_out(&self, path: &RelPath, id: FileId) {
        let mut tally = self.tally.borrow_mut();
        tally.read_log.push(path.to_string());
        *tally.reads_by_id.entry(id).or_default() += 1;
    }
}

impl<T: Tree> Tree for Counting<T> {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        self.note(Op::Read, path);
        let id = self.inner.identity(path)?;
        self.handed_out(path, id);
        self.inner.read(path, max_bytes)
    }

    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        self.note(Op::Open, path);
        let opened = self.inner.read_unseen(path, max_bytes, seen)?;
        if let Opened::Read { id, .. } = &opened {
            self.note(Op::OpenedRead, path);
            self.handed_out(path, id.clone());
        }
        Ok(opened)
    }

    fn identity(&self, path: &RelPath) -> Result<FileId, TreeError> {
        self.note(Op::Identity, path);
        self.inner.identity(path)
    }

    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        self.note(Op::List, path);
        self.inner.list(path)
    }

    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        self.note(Op::Entry, path);
        self.inner.entry(path)
    }
}
