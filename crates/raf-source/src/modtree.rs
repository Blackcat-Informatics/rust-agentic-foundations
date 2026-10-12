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
//!   followed, carrying its predicate. No `cfg` is evaluated and nothing is
//!   dropped.
//!
//! The walk runs on the caller's thread with an explicit stack, so its own
//! depth costs no stack of the caller, and detects a cycle on the stack of
//! files being followed. Files are told apart by their identity in the
//! [`Tree`], not by their spelling: a file reached by several module paths,
//! or under several spellings (through a symlink to a directory), is read and
//! parsed once, recorded under each path, and keeps the spelling that sorts
//! first.

use std::collections::{BTreeMap, HashMap};
use std::mem::size_of;
use std::rc::Rc;

use crate::budget::Budget;
use crate::cost::{btree_entry, hash_entry, shared_text, vec_first, vec_slot};
use crate::error::{LoadError, LoadErrorKind, Origin, TreeError};
use crate::facts::{ExtractError, FileFacts, ModBody, ModFact, TestRole, extract};
use crate::intern::Interner;
use crate::item::{ItemId, SharedStr};
use crate::meter::{Meter, over_budget};
use crate::parse::parse_counted;
use crate::path::RelPath;
use crate::tree::{EntryKind, FileId, Opened, Tree};
use crate::workspace::SourceFile;

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
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

/// A parsed file.
#[derive(Debug, Clone)]
pub(crate) struct Parsed {
    pub(crate) text: String,
    pub(crate) facts: FileFacts,
}

/// Where the children of a module live.
#[derive(Debug, Clone)]
struct DirCtx {
    /// The directory of the file, extended by inline modules.
    dir: RelPath,
    /// The stem of a non-`mod.rs` file, whose children live in `dir/stem`;
    /// folded into `dir` as soon as an inline module is entered.
    relative: Option<String>,
}

impl DirCtx {
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
#[derive(Clone)]
struct BodyAt {
    parsed: Rc<Parsed>,
    inline: Vec<usize>,
}

impl BodyAt {
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

    fn child(&self, position: usize) -> Self {
        let mut inline = self.inline.clone();
        inline.push(position);
        Self {
            parsed: Rc::clone(&self.parsed),
            inline,
        }
    }
}

/// The module declaration at `position` of a body, with the place the walk
/// reached it from.
struct Declared {
    at: BodyAt,
    position: usize,
    file: RelPath,
    ctx: DirCtx,
}

impl Declared {
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
enum Frame {
    /// Follow the module declarations of a body, from position `next` on.
    Decls {
        at: BodyAt,
        file: RelPath,
        ctx: DirCtx,
        scope: Scope,
        next: usize,
    },
    /// Follow the `cfg_attr` path alternatives of a declaration, from
    /// alternative `next` on. `scope` is the scope inside the module.
    Alternatives {
        declared: Declared,
        scope: Scope,
        next: usize,
    },
    /// The declarations of a file are done; it leaves the stack of files
    /// being followed.
    Leave,
}

/// One file the walk read, with every way it was reached.
struct Known {
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

/// What an arrival allocates at most: its reach and predicate in its file's
/// list, its place in both orders of arrivals, the entry that marks its file,
/// target and module path as indexed, and its entry and first room among the
/// files of each module path while the index is built.
fn reach_bytes(reach: &Reach) -> usize {
    vec_slot::<Reach>()
        + reach.predicate.as_ref().map_or(0, String::capacity)
        + vec_slot::<(usize, usize)>()
        + vec_slot::<Visit>()
        + btree_entry::<(RelPath, TargetId, SharedStr), ()>()
        + btree_entry::<&str, Vec<&SourceFile>>()
        + vec_first::<&SourceFile>()
}

pub(crate) struct Walker<'a> {
    tree: &'a dyn Tree,
    meter: &'a Meter,
    /// Every file read, in the order it was first reached.
    files: Vec<Known>,
    /// The position in `files` of each file identity.
    by_id: HashMap<FileId, usize>,
    /// The identity of each spelling the walk asked about.
    spellings: HashMap<RelPath, FileId>,
    /// Every arrival, in the order of the walk: a position in `files` and in
    /// that file's reaches.
    visits: Vec<(usize, usize)>,
    /// The files being followed, outermost first.
    active: Vec<FileId>,
    /// Module paths recorded so far, over all targets.
    arrivals: usize,
    /// One copy of each module path.
    paths: Interner,
    stack: Vec<Frame>,
}

impl<'a> Walker<'a> {
    pub(crate) fn new(tree: &'a dyn Tree, meter: &'a Meter) -> Self {
        Self {
            tree,
            meter,
            files: Vec::new(),
            by_id: HashMap::new(),
            spellings: HashMap::new(),
            visits: Vec::new(),
            active: Vec::new(),
            arrivals: 0,
            paths: Interner::default(),
            stack: Vec::new(),
        }
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
        let ctx = DirCtx {
            dir: parent_dir(root)?,
            relative: None,
        };
        let (module_path, cost) = self.paths.intern(module_path);
        self.meter.items(cost, root, None)?;
        let scope = Scope {
            target,
            module_path,
            test: false,
            depth: 0,
        };
        self.enter(root, None, origin, scope, None, ctx)?;
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
                } => {
                    if next < at.body(&file)?.mods.len() {
                        self.stack.push(Frame::Decls {
                            at: at.clone(),
                            file: file.clone(),
                            ctx: ctx.clone(),
                            scope: scope.clone(),
                            next: next + 1,
                        });
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
                } => self.alternative(declared, scope, next)?,
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
        let (extracted, tokens) =
            parse_counted(path, &text, self.meter.options().max_file_tokens, |tree| {
                extract(tree, &text, allowance)
            })
            .map_err(|error| error.reached_by(origin))?;
        let (facts, fact_bytes) =
            extracted.map_err(|problem| extract_error(path, origin, problem))?;
        self.meter.facts(fact_bytes);
        self.meter.tokens(tokens);
        self.meter
            .items(file_bytes(&id), path, None)
            .map_err(|error| error.reached_by(origin))?;
        let parsed = Rc::new(Parsed { text, facts });
        let position = self.files.len();
        self.visits.push((position, 0));
        self.files.push(Known {
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
        predicate: Option<&str>,
        ctx: DirCtx,
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
            predicate: predicate.map(str::to_owned),
        };
        let (charged, line) = site.map_or((path, None), |s| (s.file, Some(s.line)));
        self.meter
            .items(reach_bytes(&reach), charged, line)
            .map_err(|error| error.reached_by(origin))?;
        let parsed = self.load(path, opened, origin, reach)?;
        self.active.push(id);
        self.stack.push(Frame::Leave);
        self.stack.push(Frame::Decls {
            at: BodyAt {
                parsed,
                inline: Vec::new(),
            },
            file: path.clone(),
            ctx,
            scope,
            next: 0,
        });
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
    fn module(&mut self, declared: Declared, scope: &Scope) -> Result<(), LoadError> {
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
                },
            );
        }
        Ok(())
    }

    /// The default part of an inline module: its body, under its `path` or its
    /// name.
    fn inline_default(
        &mut self,
        module: &ModFact,
        site: Site<'_>,
        declared: &Declared,
        scope: &Scope,
    ) -> Result<(), LoadError> {
        let ctx = &declared.ctx;
        let dir = if let Some(path) = module.paths.unconditional.first() {
            let origin = Origin::PathAttr {
                module: module.name.clone(),
                predicate: None,
            };
            join(&ctx.dir, path, site.file, site.line, &origin)?
        } else {
            let by_name = Origin::ModDecl {
                module: module.name.clone(),
            };
            let base = ctx.module_dir(&by_name, site.file, site.line)?;
            join(&base, &module.name, site.file, site.line, &by_name)?
        };
        self.stack.push(Frame::Decls {
            at: declared.at.child(declared.position),
            file: site.file.clone(),
            ctx: DirCtx {
                dir,
                relative: None,
            },
            scope: scope.clone(),
            next: 0,
        });
        Ok(())
    }

    /// The default part of `mod name;`: its unconditional `path`, or
    /// `name.rs` or `name/mod.rs`.
    fn declared_default(
        &mut self,
        module: &ModFact,
        site: Site<'_>,
        ctx: &DirCtx,
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
        declared: Declared,
        scope: Scope,
        next: usize,
    ) -> Result<(), LoadError> {
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
            let at = declared.at.child(declared.position);
            self.stack.push(Frame::Alternatives {
                declared,
                scope,
                next: next + 1,
            });
            self.stack.push(Frame::Decls {
                at,
                file: file.clone(),
                ctx: DirCtx {
                    dir,
                    relative: None,
                },
                scope: alternative_scope,
                next: 0,
            });
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
                },
            );
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
        let ctx = DirCtx {
            dir: parent_dir(file)?,
            relative: None,
        };
        self.enter(file, Some(site), origin, scope, predicate, ctx)
    }

    /// `mod name;` without a `path`: `name.rs` or `name/mod.rs`.
    fn follow_default(
        &mut self,
        module: &ModFact,
        site: Site<'_>,
        ctx: &DirCtx,
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
                let children = DirCtx {
                    dir: base,
                    relative: Some(module.name.clone()),
                };
                self.enter(&flat, Some(site), &origin, scope.clone(), None, children)
            }
            (false, true) => {
                let children = DirCtx {
                    dir: parent_dir(&nested)?,
                    relative: None,
                };
                self.enter(&nested, Some(site), &origin, scope.clone(), None, children)
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
            let prefix = RelPath::new(&kept.join("/"))?;
            if tree.entry(&prefix)? == Some(EntryKind::Dir) {
                kept.pop();
                continue;
            }
        }
        kept.push(segment);
    }
    Ok(RelPath::new(&kept.join("/"))?)
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
    let (line, kind) = match problem {
        ExtractError::Budget { line, limit } => (
            line,
            LoadErrorKind::Budget {
                budget: Budget::FactBytes,
                limit,
            },
        ),
        ExtractError::Cfg { line, message } => (line, LoadErrorKind::CfgParse { message }),
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
