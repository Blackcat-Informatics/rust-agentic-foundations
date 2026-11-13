// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Following `mod` declarations from a target root, as rustc does.
//!
//! The rules, per module declaration:
//!
//! * `mod name;` without a `path` attribute is `name.rs` or `name/mod.rs` in
//!   the module directory; both existing is an error. The module directory of
//!   a root file or a `mod.rs` file is its own directory, and the module
//!   directory of any other file `a.rs` is `a/` beside it.
//! * `#[path = "p"]` on `mod name;` names a file relative to the directory of
//!   the file that holds the declaration, or, inside an inline module, relative
//!   to the module directory including the inline components. A file named
//!   this way is treated like a `mod.rs` for its own children.
//! * An inline module `mod m { .. }` adds `m` to the module directory; a
//!   `path` attribute on it replaces that component.
//! * `#[cfg_attr(predicate, path = "p")]` is another site. Every site is
//!   followed, carrying its predicate, in structural loads. Configured loads
//!   select active declarations and path attributes before this walk.
//!
//! The walk runs on the caller's thread with an explicit stack, so its own
//! depth costs no stack of the caller, and detects a cycle on the stack of
//! files being followed. Files are told apart by their identity in the
//! [`Tree`], not by their spelling: a file reached by several module paths,
//! or under several spellings (through a symlink to a directory), is read and
//! parsed once, recorded under each path, and keeps the spelling that sorts
//! first.

use std::collections::{BTreeMap, HashMap};

use purrdf_hash::fixed::FixedState;
use purrdf_lex::allocation::{Admission, Memory, OwnedAdmission, StorageError, growth_capacity};
use std::alloc::Layout;
use std::mem::size_of;
use std::rc::Rc;

use crate::budget::Budget;
use crate::cost::{btree_entry, hash_entry, shared_text, vec_first, vec_slot};
use crate::error::{LoadError, LoadErrorKind, Origin, TreeError};
use crate::facts::{ExtractError, FileFacts, ModBody, ModFact, TestRole, extract};
use crate::intern::Interner;
use crate::item::{ItemId, SharedStr};
use crate::meter::{Meter, over_budget};
use crate::parse::{Candidate, Parser, parse_counted, parse_counted_mut, storage};
use crate::path::RelPath;
use crate::tree::{EntryKind, FileId, Opened, Tree};
use crate::workspace::SourceFile;
use crate::{ActiveConfiguration, PackageId};

/// Index of a target in [`Workspace::targets`](crate::Workspace::targets).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TargetId(pub(crate) usize);

impl TargetId {
    /// The position of the target.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// One way a file was reached.
///
/// The struct is `#[non_exhaustive]`: adding a field is not a breaking change,
/// so a pattern names the fields it reads and ends in `..`.
///
/// ```compile_fail,E0638
/// # fn names(reach: &raf_source::Reach) {
/// let raf_source::Reach { target, module_path, test, predicate } = reach;
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Reach {
    /// The target whose module tree holds the file.
    pub target: TargetId,
    /// The module path of the file's own module, such as `demo::inner`.
    pub module_path: SharedStr,
    /// True when the file was reached through an item guarded by a `test`
    /// predicate, or through a `path` that a `test` predicate sets.
    pub test: bool,
    /// The `cfg_attr` predicate of the `path` that led here, if any.
    pub predicate: Option<String>,
    /// The `path` attribute that led the walk to this file, if one did.
    pub path_attribute: Option<PathSite>,
}

/// The `path` attribute through which the walk reached a file.
///
/// A `path` on `mod name;`, plain or inside `cfg_attr`, names the file
/// itself. A `path` on an inline module names a directory, and a file module
/// declared inside it, at any inline depth, is reached through that
/// attribute. A file's own children are reached through the file, not
/// through the attribute that led to it.
///
/// ```compile_fail,E0638
/// # fn names(site: &raf_source::PathSite) {
/// let raf_source::PathSite { file, line, resolved } = site;
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PathSite {
    /// The file that holds the module declaration carrying the attribute.
    pub file: RelPath,
    /// The line of that module's name.
    pub line: usize,
    /// The spelling the walk followed to the reached file: relative to the
    /// tree root, with `dir/..` collapsed through real directories.
    pub resolved: RelPath,
}

/// A parsed file.
#[derive(Debug, Clone)]
pub(crate) struct Parsed {
    pub(crate) text: String,
    pub(crate) facts: FileFacts,
}

/// Where the children of a module live.
struct DirCtx<'a> {
    /// The directory of the file, extended by inline modules.
    dir: RelPath,
    /// The stem of a non-`mod.rs` file, whose children live in `dir/stem`;
    /// folded into `dir` as soon as an inline module is entered.
    relative: Option<String>,
    /// The declaring file and line of the inline module `path` attribute
    /// that set `dir`, if one did.
    via: Option<(RelPath, usize)>,
    // The original subgrant is refunded only after the owned string dies.
    _admission: Option<OwnedAdmission<FrameAdmission<'a>>>,
}

/// How the walk arrives at a file.
#[derive(Default)]
struct Arrival<'p> {
    /// The `cfg_attr` predicate of the `path` that leads to the file.
    predicate: Option<&'p str>,
    /// The `path` attribute that leads to the file.
    path_attribute: Option<PathSite>,
}

impl<'a> DirCtx<'a> {
    fn new(
        dir: RelPath,
        relative: Option<&str>,
        via: Option<(RelPath, usize)>,
        parser: Option<&'a dyn Parser>,
    ) -> Result<Self, LoadError> {
        let mut admission =
            parser.map(|parser| OwnedAdmission::new(FrameAdmission { parser, bytes: 0 }));
        let relative = match (relative, admission.as_mut()) {
            (Some(text), Some(admission)) => Some(
                Memory::new(admission)
                    .string(text)
                    .map_err(frontier_storage)?,
            ),
            (Some(text), None) => Some(text.to_owned()),
            (None, _) => None,
        };
        Ok(Self {
            dir,
            relative,
            via,
            _admission: admission,
        })
    }

    fn copy(&self, parser: Option<&'a dyn Parser>) -> Result<Self, LoadError> {
        Self::new(
            self.dir.clone(),
            self.relative.as_deref(),
            self.via.clone(),
            parser,
        )
    }

    fn module_dir(
        &self,
        origin: &Origin,
        file: &RelPath,
        line: usize,
    ) -> Result<RelPath, LoadError> {
        match &self.relative {
            Some(stem) => join(&self.dir, stem, file, line, origin),
            None => Ok(self.dir.clone()),
        }
    }
}

fn join(
    base: &RelPath,
    relative: &str,
    file: &RelPath,
    line: usize,
    origin: &Origin,
) -> Result<RelPath, LoadError> {
    base.join(relative)
        .map_err(|error| LoadError::tree(file, Some(line), origin, error.into()))
}

/// Where a module declaration sits.
#[derive(Clone, Copy)]
struct Site<'a> {
    file: &'a RelPath,
    line: usize,
    module: &'a str,
}

/// State carried down the walk.
#[derive(Debug, Clone)]
struct Scope {
    target: TargetId,
    module_path: SharedStr,
    test: bool,
    depth: usize,
}

/// A parsed file with every way it was reached.
pub(crate) struct Visited {
    pub(crate) parsed: Rc<Parsed>,
    pub(crate) reached: Vec<Reach>,
}

/// What a walk found.
pub(crate) struct Walked {
    /// The parsed files, by the spelling each keeps.
    pub(crate) files: BTreeMap<RelPath, Visited>,
    /// Every arrival, in the order of the walk.
    pub(crate) visits: Vec<Visit>,
    /// One copy of each module path the walk recorded.
    pub(crate) paths: Interner,
}

/// One arrival of the walk at a file: the file and the position of the
/// [`Reach`] it recorded in the file's list.
///
/// The walk records a file's arrival before it follows the file's own module
/// declarations, so in the order of arrivals the file that declares a module
/// always comes before the module's own file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Visit {
    pub(crate) file: RelPath,
    pub(crate) reach: usize,
}

/// A module body of a parsed file: the file's own body, or an inline module
/// inside it, found by the positions of the inline modules on the way.
struct BodyAt<'a> {
    parsed: Rc<Parsed>,
    inline: Vec<usize>,
    // This owner follows the vector even while a popped body remains active.
    _admission: Option<OwnedAdmission<FrameAdmission<'a>>>,
}

impl<'a> BodyAt<'a> {
    fn body(&self, file: &RelPath) -> Result<&ModBody, LoadError> {
        let mut body = &self.parsed.facts.body;
        for &position in &self.inline {
            body = body
                .mods
                .get(position)
                .and_then(|module| module.inline.as_ref())
                .ok_or_else(|| {
                    LoadError::new(
                        file,
                        None,
                        LoadErrorKind::Internal {
                            what: "an inline module of the walk is not in its file".to_owned(),
                        },
                    )
                })?;
        }
        Ok(body)
    }

    fn copy(
        &self,
        position: Option<usize>,
        parser: Option<&'a dyn Parser>,
    ) -> Result<Self, LoadError> {
        let mut admission =
            parser.map(|parser| OwnedAdmission::new(FrameAdmission { parser, bytes: 0 }));
        let mut inline = if let Some(admission) = admission.as_mut() {
            let mut inline = Vec::new();
            let required = self
                .inline
                .len()
                .checked_add(usize::from(position.is_some()))
                .ok_or_else(|| frontier_storage(StorageError::SizeOverflow))?;
            Memory::new(admission)
                .reserve(&mut inline, required)
                .map_err(frontier_storage)?;
            inline.extend_from_slice(&self.inline);
            inline
        } else {
            self.inline.clone()
        };
        if let Some(position) = position {
            inline.push(position);
        }
        Ok(Self {
            parsed: Rc::clone(&self.parsed),
            inline,
            _admission: admission,
        })
    }
}

/// The module declaration at `position` of a body, with the place the walk
/// reached it from.
struct Declared<'a> {
    at: BodyAt<'a>,
    position: usize,
    file: RelPath,
    ctx: DirCtx<'a>,
}

impl Declared<'_> {
    fn module(&self) -> Result<&ModFact, LoadError> {
        self.at
            .body(&self.file)?
            .mods
            .get(self.position)
            .ok_or_else(|| {
                LoadError::new(
                    &self.file,
                    None,
                    LoadErrorKind::Internal {
                        what: "a module declaration of the walk is not in its file".to_owned(),
                    },
                )
            })
    }
}

/// What is left to do, on the walk's own stack.
enum Frame<'a> {
    /// Follow the module declarations of a body, from position `next` on.
    Decls {
        at: BodyAt<'a>,
        file: RelPath,
        ctx: DirCtx<'a>,
        scope: Scope,
        next: usize,
        frontier: (usize, usize),
    },
    /// Follow the `cfg_attr` path alternatives of a declaration, from
    /// alternative `next` on. `scope` is the scope inside the module.
    Alternatives {
        declared: Declared<'a>,
        scope: Scope,
        next: usize,
        frontier: usize,
    },
    /// The declarations of a file are done; it leaves the stack of files
    /// being followed.
    Leave,
}

/// A native subview that forwards every resize to the original scheduler grant.
struct FrameAdmission<'a> {
    parser: &'a dyn Parser,
    bytes: usize,
}

fn frontier_storage(cause: StorageError) -> LoadError {
    storage(&RelPath::root(), cause)
}

impl Admission for FrameAdmission<'_> {
    fn resize(&mut self, bytes: usize) -> Result<(), StorageError> {
        self.parser.resize_frontier(self.bytes, bytes)?;
        self.bytes = bytes;
        Ok(())
    }
}

struct Frames<'a> {
    values: Vec<Frame<'a>>,
    parser: Option<&'a dyn Parser>,
}

impl<'a> Frames<'a> {
    fn reserve(&mut self) -> Result<(), LoadError> {
        let Some(parser) = self.parser else {
            return Ok(());
        };
        let operation = (|| {
            let bytes = Layout::array::<Frame<'a>>(self.values.capacity())
                .map_err(|_| StorageError::SizeOverflow)?
                .size();
            let required = self
                .values
                .len()
                .checked_add(1)
                .ok_or(StorageError::SizeOverflow)?;
            let capacity = growth_capacity::<Frame<'a>>(self.values.capacity(), required)?;
            let mut admission = FrameAdmission { parser, bytes };
            Memory::resume(&mut admission, bytes).reserve(&mut self.values, capacity)
        })();
        operation.map_err(frontier_storage)
    }

    fn push(&mut self, frame: Frame<'a>) -> Result<(), LoadError> {
        self.reserve()?;
        self.values.push(frame);
        Ok(())
    }

    fn insert(&mut self, index: usize, frame: Frame<'a>) -> Result<(), LoadError> {
        self.reserve()?;
        self.values.insert(index, frame);
        Ok(())
    }

    fn pop(&mut self) -> Option<Frame<'a>> {
        self.values.pop()
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

impl Drop for Frames<'_> {
    fn drop(&mut self) {
        if let Some(parser) = self.parser {
            let values = std::mem::take(&mut self.values);
            let bytes = Layout::array::<Frame<'_>>(values.capacity())
                .expect("retained frontier capacity has a representable layout")
                .size();
            let mut admission = FrameAdmission { parser, bytes };
            Memory::resume(&mut admission, bytes)
                .release_vec(values)
                .expect("destroyed frontier retains its original scheduler grant");
        }
    }
}

/// One file the walk read, with every way it was reached.
struct Known<'a> {
    configuration: Option<(PackageId, &'a ActiveConfiguration)>,
    path: RelPath,
    parsed: Rc<Parsed>,
    reached: Vec<Reach>,
}

/// What remembering the identity of one spelling allocates at most: its entry
/// in the table of spellings, the spelling's text and what the identity holds.
fn spelling_bytes(path: &RelPath, id: &FileId) -> usize {
    hash_entry::<(RelPath, FileId)>() + shared_text(path.as_str().len()) + id.heap_bytes()
}

/// What a file the walk reads allocates at most beyond its text and facts: its
/// place among the files read and in the table of identities, its entry in the
/// workspace's files, its shared parse, the first room of its list of reaches,
/// and its entry and first room among the items of each file in the index.
fn file_bytes(id: &FileId) -> usize {
    vec_slot::<Known>()
        + hash_entry::<(FileId, usize)>()
        + id.heap_bytes()
        + btree_entry::<RelPath, SourceFile>()
        + size_of::<Parsed>()
        + 2 * size_of::<usize>()
        + vec_first::<Reach>()
        + btree_entry::<RelPath, Vec<ItemId>>()
        + vec_first::<ItemId>()
}

/// What an arrival allocates at most: its reach, predicate and the spellings
/// of its path attribute in its file's list, its place in both orders of
/// arrivals, the entry that marks its file, target and module path as
/// indexed, and its entry and first room among the files of each module path
/// while the index is built.
fn reach_bytes(reach: &Reach) -> usize {
    vec_slot::<Reach>()
        + reach.predicate.as_ref().map_or(0, String::capacity)
        + reach.path_attribute.as_ref().map_or(0, |site| {
            shared_text(site.file.as_str().len()) + shared_text(site.resolved.as_str().len())
        })
        + vec_slot::<(usize, usize)>()
        + vec_slot::<Visit>()
        + btree_entry::<(RelPath, TargetId, SharedStr), ()>()
        + btree_entry::<(TargetId, &str), Vec<&SourceFile>>()
        + vec_first::<&SourceFile>()
}

pub(crate) struct Walker<'a> {
    configuration: Option<(PackageId, &'a ActiveConfiguration)>,
    tree: &'a dyn Tree,
    meter: &'a Meter,
    /// Every file read, in the order it was first reached.
    files: Vec<Known<'a>>,
    /// The position in `files` of each file identity.
    by_id: HashMap<FileId, usize, FixedState>,
    /// The identity of each spelling the walk asked about.
    spellings: HashMap<RelPath, FileId, FixedState>,
    /// Every arrival, in the order of the walk: a position in `files` and in
    /// that file's reaches.
    visits: Vec<(usize, usize)>,
    /// The files being followed, outermost first.
    active: Vec<FileId>,
    /// Module paths recorded so far, over all targets.
    arrivals: usize,
    /// One copy of each module path.
    paths: Interner,
    stack: Frames<'a>,
    parser: Option<&'a dyn Parser>,
}

impl<'a> Walker<'a> {
    pub(crate) fn new(tree: &'a dyn Tree, meter: &'a Meter) -> Self {
        Self {
            configuration: None,
            tree,
            meter,
            files: Vec::new(),
            by_id: HashMap::with_hasher(FixedState::new()),
            spellings: HashMap::with_hasher(FixedState::new()),
            visits: Vec::new(),
            active: Vec::new(),
            arrivals: 0,
            paths: Interner::default(),
            stack: Frames {
                values: Vec::new(),
                parser: None,
            },
            parser: None,
        }
    }
    pub(crate) fn with_parser(
        tree: &'a dyn Tree,
        meter: &'a Meter,
        parser: Option<&'a dyn Parser>,
    ) -> Self {
        let mut walker = Self::new(tree, meter);
        walker.parser = parser;
        walker.stack.parser = parser;
        walker
    }
    pub(crate) fn configure(
        &mut self,
        configuration: Option<(PackageId, &'a ActiveConfiguration)>,
    ) {
        self.configuration = configuration;
    }

    /// Advance a target frontier once; committed physical files are not reread.
    pub(crate) fn speculate(&self, path: &RelPath) -> bool {
        self.speculate_candidate(Candidate::Existing(path))
    }

    fn speculate_candidate(&self, candidate: Candidate<'_>) -> bool {
        self.parser.is_none_or(|parser| {
            parser.speculate(self.tree, candidate, self.meter.options(), &|id| {
                self.by_id.contains_key(id)
            })
        })
    }

    fn speculative_capacity(&self) -> bool {
        self.parser.is_some_and(Parser::has_capacity)
    }

    /// Each frame keeps its own monotonically advancing candidate cursor.
    fn prefetch_body(
        &self,
        at: &BodyAt<'a>,
        file: &RelPath,
        ctx: &DirCtx<'a>,
        next: usize,
        mut frontier: (usize, usize),
    ) -> Result<(usize, usize), LoadError> {
        if frontier.0 <= next {
            frontier = (next + 1, 0);
        }
        let modules = &at.body(file)?.mods;
        while self.speculative_capacity() && frontier.0 < modules.len() {
            #[cfg(all(test, target_os = "linux"))]
            let _preparation = crate::parse::parallel::tests::preparing();
            let module = &modules[frontier.0];
            if module.inline.is_some() {
                frontier = (frontier.0 + 1, 0);
                continue;
            }
            let defaults = if module.paths.unconditional.is_empty() {
                2
            } else {
                1
            };
            let candidate = if frontier.1 < defaults {
                if let Some(path) = module.paths.unconditional.first() {
                    Some(Candidate::Joined {
                        base: &ctx.dir,
                        stem: None,
                        input: path,
                        suffix: "",
                    })
                } else {
                    Some(Candidate::Joined {
                        base: &ctx.dir,
                        stem: ctx.relative.as_deref(),
                        input: &module.name,
                        suffix: if frontier.1 == 0 { ".rs" } else { "/mod.rs" },
                    })
                }
            } else {
                module
                    .paths
                    .conditional
                    .get(frontier.1 - defaults)
                    .map(|alternative| Candidate::Joined {
                        base: &ctx.dir,
                        stem: None,
                        input: &alternative.path,
                        suffix: "",
                    })
            };
            if let Some(candidate) = candidate
                && !self.speculate_candidate(candidate)
            {
                break;
            }
            frontier.1 += 1;
            if frontier.1 >= defaults + module.paths.conditional.len() {
                frontier = (frontier.0 + 1, 0);
            }
        }
        Ok(frontier)
    }

    fn prefetch_alternatives(
        &self,
        declared: &Declared<'a>,
        mut frontier: usize,
    ) -> Result<usize, LoadError> {
        let module = declared.module()?;
        // Inline alternatives name directories. Their own Decls frames provide
        // file candidates once the corresponding directory context is reached.
        if module.inline.is_some() {
            return Ok(module.paths.conditional.len());
        }
        while self.speculative_capacity() && frontier < module.paths.conditional.len() {
            #[cfg(all(test, target_os = "linux"))]
            let _preparation = crate::parse::parallel::tests::preparing();
            if !self.speculate_candidate(Candidate::Joined {
                base: &declared.ctx.dir,
                stem: None,
                input: &module.paths.conditional[frontier].path,
                suffix: "",
            }) {
                break;
            }
            frontier += 1;
        }
        Ok(frontier)
    }

    /// The parsed files, keyed by the spelling each keeps (the one that sorts
    /// first of those it was reached by), and every arrival
    /// in the order of the walk, with the table of module paths.
    pub(crate) fn finish(self) -> Result<Walked, LoadError> {
        let visits = self
            .visits
            .iter()
            .map(|&(file, reach)| Visit {
                file: self.files[file].path.clone(),
                reach,
            })
            .collect();
        let mut files = BTreeMap::new();
        for known in self.files {
            let path = known.path;
            let visited = Visited {
                parsed: known.parsed,
                reached: known.reached,
            };
            if files.insert(path.clone(), visited).is_some() {
                return Err(LoadError::new(
                    &path,
                    None,
                    LoadErrorKind::Internal {
                        what: "two files of the walk keep one spelling".to_owned(),
                    },
                ));
            }
        }
        Ok(Walked {
            files,
            visits,
            paths: self.paths,
        })
    }

    /// Follow a target from its root file.
    pub(crate) fn root(
        &mut self,
        target: TargetId,
        root: &RelPath,
        module_path: &str,
        origin: &Origin,
    ) -> Result<(), LoadError> {
        let ctx = DirCtx::new(parent_dir(root)?, None, None, self.parser)?;
        let (module_path, cost) = self.paths.intern(module_path);
        self.meter.items(cost, root, None)?;
        let scope = Scope {
            target,
            module_path,
            test: match self.configuration {
                Some((_, cfg)) => crate::configuration::evaluate(
                    &crate::cfg::CfgExpr::Test,
                    cfg,
                    false,
                    self.meter,
                    root,
                )?,
                None => false,
            },
            depth: 0,
        };
        self.enter(root, None, origin, scope, Arrival::default(), ctx)?;
        self.run()
    }

    /// Work the stack until it is empty.
    fn run(&mut self) -> Result<(), LoadError> {
        while let Some(frame) = self.stack.pop() {
            match frame {
                Frame::Leave => {
                    self.active.pop();
                }
                Frame::Decls {
                    at,
                    file,
                    ctx,
                    scope,
                    next,
                    frontier,
                } => {
                    if next < at.body(&file)?.mods.len() {
                        let frontier = self.prefetch_body(&at, &file, &ctx, next, frontier)?;
                        self.stack.push(Frame::Decls {
                            at: at.copy(None, self.parser)?,
                            file: file.clone(),
                            ctx: ctx.copy(self.parser)?,
                            scope: scope.clone(),
                            next: next + 1,
                            frontier,
                        })?;
                        let declared = Declared {
                            at,
                            position: next,
                            file,
                            ctx,
                        };
                        self.module(declared, &scope)?;
                    }
                }
                Frame::Alternatives {
                    declared,
                    scope,
                    next,
                    frontier,
                } => self.alternative(declared, scope, next, frontier)?,
            }
        }
        Ok(())
    }

    /// The identity of the file at `path`, asked once per spelling, and its
    /// text when the walk has not read the file yet: one open gives both.
    fn identify(
        &mut self,
        path: &RelPath,
        site: Option<Site<'_>>,
        origin: &Origin,
    ) -> Result<(FileId, Option<String>), LoadError> {
        if let Some(known) = self.spellings.get(path) {
            return Ok((known.clone(), None));
        }
        let by_id = &self.by_id;
        let max_bytes = self.meter.options().max_file_bytes;
        // Speculation observed another seen predicate. Only this canonical
        // observation can decide whether an alias still needs to be read.
        let opened = self
            .tree
            .read_unseen(path, max_bytes, &|id| by_id.contains_key(id));
        let (id, text) = match opened {
            Ok(Opened::Seen(id)) => (id, None),
            Ok(Opened::Read { id, text }) => (id, Some(text)),
            Err(TreeError::NotFound { .. }) if site.is_some() => {
                return Err(unresolved(site, path, origin));
            }
            Err(error) => {
                let (file, line) = site.map_or((path, None), |s| (s.file, Some(s.line)));
                return Err(LoadError::tree(file, line, origin, error));
            }
        };
        let (charged, line) = site.map_or((path, None), |s| (s.file, Some(s.line)));
        self.meter
            .items(spelling_bytes(path, &id), charged, line)
            .map_err(|error| error.reached_by(origin))?;
        self.spellings.insert(path.clone(), id.clone());
        Ok((id, text))
    }

    /// The file `id` at `path` with `reach` recorded: parsed from `text` when
    /// it is new, which the open that gave `id` read.
    fn load(
        &mut self,
        path: &RelPath,
        (id, text): (FileId, Option<String>),
        origin: &Origin,
        reach: Reach,
    ) -> Result<Rc<Parsed>, LoadError> {
        let Some(text) = text else {
            let position = *self.by_id.get(&id).ok_or_else(|| {
                LoadError::new(
                    path,
                    None,
                    LoadErrorKind::Internal {
                        what: "a file seen by the walk was never read".to_owned(),
                    },
                )
            })?;
            let known = &mut self.files[position];
            if !match (known.configuration, self.configuration) {
                (None, None) => true,
                (Some((left, a)), Some((right, b))) => {
                    left == right && a.same_syntax_under(b, self.meter, path)?
                }
                _ => false,
            } {
                return Err(crate::configuration::failure(
                    path,
                    "one physical source file is shared by different configured package contexts; load those contexts separately",
                ));
            }
            self.visits.push((position, known.reached.len()));
            known.reached.push(reach);
            if *path < known.path {
                known.path = path.clone();
            }
            return Ok(Rc::clone(&known.parsed));
        };
        self.meter
            .read(path, &text)
            .map_err(|error| error.reached_by(origin))?;
        let allowance = self.meter.fact_allowance();
        let (text, (facts, fact_bytes), scan) = if let Some((_, configuration)) = self.configuration
        {
            let before = self.meter.usage();
            let options = self.meter.options().clone();
            let ((result, after), scan) =
                parse_counted_mut(path, &text, options.max_file_tokens, |tree| {
                    let local = Meter::resume(&options, before);
                    let result = crate::configuration::filter(tree, configuration, &local, path)
                        .and_then(|()| {
                            extract(tree, &text, allowance)
                                .map_err(|problem| extract_error(path, origin, problem))
                        });
                    (result, local.usage())
                })
                .map_err(|error| error.reached_by(origin))?;
            self.meter.configuration_spent(before, after, path)?;
            (text, result?, scan)
        } else if let Some(parser) = self.parser {
            let extracted = parser
                .parse(path, &id, text, self.meter.options(), allowance)
                .map_err(|error| error.reached_by(origin))?;
            let facts = extracted
                .facts
                .map_err(|problem| extract_error(path, origin, problem))?;
            (extracted.text, facts, extracted.scan)
        } else {
            let (extracted, scan) =
                parse_counted(path, &text, self.meter.options().max_file_tokens, |tree| {
                    extract(tree, &text, allowance)
                })
                .map_err(|error| error.reached_by(origin))?;
            (
                text,
                extracted.map_err(|problem| extract_error(path, origin, problem))?,
                scan,
            )
        };
        self.meter.facts(fact_bytes, path)?;
        self.meter.scanned(scan);
        self.meter
            .items(file_bytes(&id), path, None)
            .map_err(|error| error.reached_by(origin))?;
        let parsed = Rc::new(Parsed { text, facts });
        let position = self.files.len();
        self.visits.push((position, 0));
        self.files.push(Known {
            configuration: self.configuration,
            path: path.clone(),
            parsed: Rc::clone(&parsed),
            reached: vec![reach],
        });
        self.by_id.insert(id, position);
        Ok(parsed)
    }

    /// Count one module walk (a file arrival or an inline body) against the
    /// module-path budget. `site` is the declaring file and line, if any.
    fn count(
        &mut self,
        site: Option<(&RelPath, usize)>,
        path: &RelPath,
        origin: &Origin,
    ) -> Result<(), LoadError> {
        self.arrivals += 1;
        if self.arrivals > Budget::ModulePaths.limit() {
            let (file, line) = site.map_or((path, None), |(file, line)| (file, Some(line)));
            return Err(
                over_budget(Budget::ModulePaths, Budget::ModulePaths.limit(), file, line)
                    .reached_by(origin),
            );
        }
        Ok(())
    }

    /// Record an arrival at `path` and put its module declarations on the
    /// stack.
    fn enter(
        &mut self,
        path: &RelPath,
        site: Option<Site<'_>>,
        origin: &Origin,
        scope: Scope,
        arrival: Arrival<'_>,
        ctx: DirCtx<'a>,
    ) -> Result<(), LoadError> {
        let opened = self.identify(path, site, origin)?;
        let id = opened.0.clone();
        if self.active.contains(&id) {
            let (file, line, module) = site.map_or((path, None, String::new()), |s| {
                (s.file, Some(s.line), s.module.to_owned())
            });
            return Err(LoadError::new(
                file,
                line,
                LoadErrorKind::ModuleCycle {
                    module,
                    again: Box::new(path.clone()),
                },
            )
            .reached_by(origin));
        }
        self.count(site.map(|s| (s.file, s.line)), path, origin)?;
        let reach = Reach {
            target: scope.target,
            module_path: scope.module_path.clone(),
            test: scope.test,
            predicate: arrival.predicate.map(str::to_owned),
            path_attribute: arrival.path_attribute,
        };
        let (charged, line) = site.map_or((path, None), |s| (s.file, Some(s.line)));
        self.meter
            .items(reach_bytes(&reach), charged, line)
            .map_err(|error| error.reached_by(origin))?;
        let parsed = self.load(path, opened, origin, reach)?;
        self.active.push(id);
        self.stack.push(Frame::Leave)?;
        self.stack.push(Frame::Decls {
            at: BodyAt {
                parsed,
                inline: Vec::new(),
                _admission: None,
            },
            file: path.clone(),
            ctx,
            scope,
            next: 0,
            frontier: (0, 0),
        })?;
        Ok(())
    }

    fn deeper(module: &ModFact, file: &RelPath, scope: &Scope) -> Result<(), LoadError> {
        if scope.depth + 1 > Budget::ModuleDepth.limit() {
            return Err(over_budget(
                Budget::ModuleDepth,
                Budget::ModuleDepth.limit(),
                file,
                Some(module.name_line),
            )
            .reached_by(&Origin::ModDecl {
                module: module.name.clone(),
            }));
        }
        Ok(())
    }

    /// Follow one module declaration: its default part now, its alternatives
    /// once that is done.
    fn module(&mut self, declared: Declared<'a>, scope: &Scope) -> Result<(), LoadError> {
        let module = declared.module()?;
        let file = &declared.file;
        Self::deeper(module, file, scope)?;
        if module.paths.unconditional.len() > 1 {
            return Err(LoadError::new(
                file,
                Some(module.name_line),
                LoadErrorKind::DuplicatePathAttr {
                    module: module.name.clone(),
                },
            )
            .reached_by(&Origin::PathAttr {
                module: module.name.clone(),
                predicate: None,
            }));
        }
        let (module_path, cost) = self
            .paths
            .intern(&format!("{}::{}", scope.module_path, module.name));
        self.meter
            .items(cost, file, Some(module.name_line))
            .map_err(|error| {
                error.reached_by(&Origin::ModDecl {
                    module: module.name.clone(),
                })
            })?;
        let inner = Scope {
            target: scope.target,
            module_path,
            test: scope.test || module.test != TestRole::None,
            depth: scope.depth + 1,
        };
        let site = Site {
            file,
            line: module.name_line,
            module: &module.name,
        };
        let has_alternatives = !module.paths.conditional.is_empty();
        let frontier = if has_alternatives {
            self.prefetch_alternatives(&declared, 0)?
        } else {
            0
        };
        let before = self.stack.len();
        if module.inline.is_some() {
            self.inline_default(module, site, &declared, &inner)?;
        } else {
            self.declared_default(module, site, &declared.ctx, &inner, has_alternatives)?;
        }
        if has_alternatives {
            // Below the frames of the default part, so it runs once they are
            // done.
            self.stack.insert(
                before,
                Frame::Alternatives {
                    declared,
                    scope: inner,
                    next: 0,
                    frontier,
                },
            )?;
        }
        Ok(())
    }

    /// The default part of an inline module: its body, under its `path` or its
    /// name.
    fn inline_default(
        &mut self,
        module: &ModFact,
        site: Site<'_>,
        declared: &Declared<'a>,
        scope: &Scope,
    ) -> Result<(), LoadError> {
        let ctx = &declared.ctx;
        let (dir, via) = if let Some(path) = module.paths.unconditional.first() {
            let origin = Origin::PathAttr {
                module: module.name.clone(),
                predicate: None,
            };
            (
                join(&ctx.dir, path, site.file, site.line, &origin)?,
                Some((site.file.clone(), site.line)),
            )
        } else {
            let by_name = Origin::ModDecl {
                module: module.name.clone(),
            };
            let base = ctx.module_dir(&by_name, site.file, site.line)?;
            (
                join(&base, &module.name, site.file, site.line, &by_name)?,
                ctx.via.clone(),
            )
        };
        self.stack.push(Frame::Decls {
            at: declared.at.copy(Some(declared.position), self.parser)?,
            file: site.file.clone(),
            ctx: DirCtx::new(dir, None, via, self.parser)?,
            scope: scope.clone(),
            next: 0,
            frontier: (0, 0),
        })?;
        Ok(())
    }

    /// The default part of `mod name;`: its unconditional `path`, or
    /// `name.rs` or `name/mod.rs`.
    fn declared_default(
        &mut self,
        module: &ModFact,
        site: Site<'_>,
        ctx: &DirCtx<'a>,
        scope: &Scope,
        has_alternatives: bool,
    ) -> Result<(), LoadError> {
        if let Some(path) = module.paths.unconditional.first() {
            let origin = Origin::PathAttr {
                module: module.name.clone(),
                predicate: None,
            };
            let file = join(&ctx.dir, path, site.file, site.line, &origin)?;
            let file = self.tidy(&file, site, &origin)?;
            self.follow_path_file(&file, site, &origin, scope.clone(), None)?;
        } else {
            self.follow_default(module, site, ctx, scope, has_alternatives)?;
        }
        Ok(())
    }

    /// Follow alternative `next` of a declaration, and leave the rest on the
    /// stack.
    fn alternative(
        &mut self,
        declared: Declared<'a>,
        scope: Scope,
        next: usize,
        frontier: usize,
    ) -> Result<(), LoadError> {
        let frontier = self.prefetch_alternatives(&declared, frontier.max(next + 1))?;
        let module = declared.module()?;
        let Some(alternative) = module.paths.conditional.get(next) else {
            return Ok(());
        };
        let name = module.name.clone();
        let line = module.name_line;
        let inline = module.inline.is_some();
        let alternative = alternative.clone();
        let origin = Origin::PathAttr {
            module: name.clone(),
            predicate: Some(alternative.predicate.clone()),
        };
        let alternative_scope = Scope {
            test: scope.test || alternative.implies_test,
            ..scope.clone()
        };
        let file = declared.file.clone();
        let site = Site {
            file: &file,
            line,
            module: &name,
        };
        let dir = join(&declared.ctx.dir, &alternative.path, &file, line, &origin)?;
        if inline {
            self.count(Some((&file, line)), &file, &origin)?;
            let at = declared.at.copy(Some(declared.position), self.parser)?;
            self.stack.push(Frame::Alternatives {
                declared,
                scope,
                next: next + 1,
                frontier,
            })?;
            self.stack.push(Frame::Decls {
                at,
                file: file.clone(),
                ctx: DirCtx::new(dir, None, Some((file.clone(), line)), self.parser)?,
                scope: alternative_scope,
                next: 0,
                frontier: (0, 0),
            })?;
        } else {
            let target = self.tidy(&dir, site, &origin)?;
            let after = self.stack.len();
            self.follow_path_file(
                &target,
                site,
                &origin,
                alternative_scope,
                Some(&alternative.predicate),
            )?;
            self.stack.insert(
                after,
                Frame::Alternatives {
                    declared,
                    scope,
                    next: next + 1,
                    frontier,
                },
            )?;
        }
        Ok(())
    }

    fn tidy(&self, path: &RelPath, site: Site<'_>, origin: &Origin) -> Result<RelPath, LoadError> {
        tidy(self.tree, path)
            .map_err(|error| LoadError::tree(site.file, Some(site.line), origin, error))
    }

    /// True when a file, or a symlink that may lead to one, is at `path`.
    fn exists(&self, path: &RelPath, site: Site<'_>, origin: &Origin) -> Result<bool, LoadError> {
        match self
            .tree
            .entry(path)
            .map_err(|error| LoadError::tree(site.file, Some(site.line), origin, error))?
        {
            // A special file counts as present so that reading it is refused,
            // not mistaken for a missing module.
            Some(EntryKind::File | EntryKind::Symlink | EntryKind::Other) => Ok(true),
            Some(EntryKind::Dir) | None => Ok(false),
        }
    }

    /// A file named by a `path` attribute: a `mod.rs` for its own children.
    fn follow_path_file(
        &mut self,
        file: &RelPath,
        site: Site<'_>,
        origin: &Origin,
        scope: Scope,
        predicate: Option<&str>,
    ) -> Result<(), LoadError> {
        let ctx = DirCtx::new(parent_dir(file)?, None, None, self.parser)?;
        let arrival = Arrival {
            predicate,
            path_attribute: Some(PathSite {
                file: site.file.clone(),
                line: site.line,
                resolved: file.clone(),
            }),
        };
        self.enter(file, Some(site), origin, scope, arrival, ctx)
    }

    /// `mod name;` without a `path`: `name.rs` or `name/mod.rs`.
    fn follow_default(
        &mut self,
        module: &ModFact,
        site: Site<'_>,
        ctx: &DirCtx<'a>,
        scope: &Scope,
        has_alternatives: bool,
    ) -> Result<(), LoadError> {
        let origin = Origin::ModDecl {
            module: module.name.clone(),
        };
        let base = ctx.module_dir(&origin, site.file, site.line)?;
        let flat = join(
            &base,
            &format!("{}.rs", module.name),
            site.file,
            site.line,
            &origin,
        )?;
        let nested = join(
            &base,
            &format!("{}/mod.rs", module.name),
            site.file,
            site.line,
            &origin,
        )?;
        // A file found by name in a directory that an inline module's `path`
        // set is reached through that attribute.
        let arrival = |found: &RelPath| -> Result<Arrival<'_>, LoadError> {
            let path_attribute = match &ctx.via {
                Some((file, line)) => Some(PathSite {
                    file: file.clone(),
                    line: *line,
                    resolved: self.tidy(found, site, &origin)?,
                }),
                None => None,
            };
            Ok(Arrival {
                predicate: None,
                path_attribute,
            })
        };
        match (
            self.exists(&flat, site, &origin)?,
            self.exists(&nested, site, &origin)?,
        ) {
            (true, true) => Err(LoadError::new(
                site.file,
                Some(site.line),
                LoadErrorKind::BothModFiles {
                    module: module.name.clone(),
                    first: Box::new(flat),
                    second: Box::new(nested),
                },
            )
            .reached_by(&origin)),
            (true, false) => {
                let children = DirCtx::new(base, Some(&module.name), None, self.parser)?;
                let arrival = arrival(&flat)?;
                self.enter(&flat, Some(site), &origin, scope.clone(), arrival, children)
            }
            (false, true) => {
                let children = DirCtx::new(parent_dir(&nested)?, None, None, self.parser)?;
                let arrival = arrival(&nested)?;
                self.enter(
                    &nested,
                    Some(site),
                    &origin,
                    scope.clone(),
                    arrival,
                    children,
                )
            }
            // Every `cfg_attr` alternative names a file; none of them need the
            // default one.
            (false, false) if has_alternatives => Ok(()),
            (false, false) => Err(unresolved(Some(site), &flat, &origin)),
        }
    }
}

/// Collapse `dir/..` pairs where `dir` is a real directory, so one file has
/// one spelling. A `..` after a symlink, a file or a missing entry stays, and
/// the disk decides where it leads.
pub(crate) fn tidy(tree: &dyn Tree, path: &RelPath) -> Result<RelPath, TreeError> {
    let mut kept: Vec<&str> = Vec::new();
    for segment in path.segments() {
        if segment == ".." && kept.last().is_some_and(|last| *last != "..") {
            let prefix = path.source_root().join(&kept.join("/"))?;
            if tree.entry(&prefix)? == Some(EntryKind::Dir) {
                kept.pop();
                continue;
            }
        }
        kept.push(segment);
    }
    Ok(path.source_root().join(&kept.join("/"))?)
}

fn unresolved(site: Option<Site<'_>>, path: &RelPath, origin: &Origin) -> LoadError {
    let (file, line, module) = site.map_or((path, None, String::new()), |s| {
        (s.file, Some(s.line), s.module.to_owned())
    });
    LoadError::new(file, line, LoadErrorKind::UnresolvedMod { module }).reached_by(origin)
}

/// The directory that holds a file.
fn parent_dir(file: &RelPath) -> Result<RelPath, LoadError> {
    file.parent().ok_or_else(|| {
        LoadError::tree(
            file,
            None,
            &Origin::Discovery,
            TreeError::NotRegular {
                path: file.to_string(),
            },
        )
    })
}

fn extract_error(file: &RelPath, origin: &Origin, problem: ExtractError) -> LoadError {
    // A `path` attribute on a module is what the error comes from, not what
    // reached the file that holds it.
    if let ExtractError::PathAttr {
        line,
        module: Some(module),
        predicate,
        message,
    } = problem
    {
        return LoadError::new(file, Some(line), LoadErrorKind::CfgParse { message })
            .reached_by(&Origin::PathAttr { module, predicate });
    }
    let (line, kind) = match problem {
        ExtractError::Budget { line, limit } => (
            line,
            LoadErrorKind::Budget {
                budget: Budget::FactBytes,
                limit,
            },
        ),
        ExtractError::Cfg { line, message } | ExtractError::PathAttr { line, message, .. } => {
            (line, LoadErrorKind::CfgParse { message })
        }
        ExtractError::Unsupported { line, what } => (line, LoadErrorKind::UnsupportedItem { what }),
        ExtractError::Position { line } => (
            line,
            LoadErrorKind::UnsupportedItem {
                what: "a position outside the text".to_owned(),
            },
        ),
    };
    LoadError::new(file, Some(line), kind).reached_by(origin)
}
