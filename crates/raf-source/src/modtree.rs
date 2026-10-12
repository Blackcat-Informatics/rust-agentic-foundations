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
//! The walk runs on the caller's thread and detects a cycle on the stack of
//! files being followed. A file reached by several module paths is parsed
//! once and recorded under each path.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::rc::Rc;

use crate::budget::Budget;
use crate::error::{LoadError, LoadErrorKind, Origin, TreeError};
use crate::facts::{ExtractError, FileFacts, ModBody, ModFact, TestRole, extract};
use crate::options::LoadOptions;
use crate::parse::parse_source;
use crate::path::RelPath;
use crate::tree::{EntryKind, Tree};

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
    pub module_path: String,
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
#[derive(Clone)]
struct Scope<'a> {
    target: TargetId,
    module_path: &'a str,
    test: bool,
    depth: usize,
}

/// A parsed file with every way it was reached.
pub(crate) struct Visited {
    pub(crate) parsed: Rc<Parsed>,
    pub(crate) reached: Vec<Reach>,
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

pub(crate) struct Walker<'a> {
    tree: &'a dyn Tree,
    options: &'a LoadOptions,
    files: BTreeMap<RelPath, Visited>,
    /// Every arrival, in the order of the walk.
    visits: Vec<Visit>,
    active: Vec<RelPath>,
    /// Module paths recorded so far, over all targets.
    arrivals: usize,
}

impl<'a> Walker<'a> {
    pub(crate) fn new(tree: &'a dyn Tree, options: &'a LoadOptions) -> Self {
        Self {
            tree,
            options,
            files: BTreeMap::new(),
            visits: Vec::new(),
            active: Vec::new(),
            arrivals: 0,
        }
    }

    /// The parsed files and how each was reached, and every arrival in the
    /// order of the walk.
    pub(crate) fn finish(self) -> (BTreeMap<RelPath, Visited>, Vec<Visit>) {
        (self.files, self.visits)
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
        let scope = Scope {
            target,
            module_path,
            test: false,
            depth: 0,
        };
        self.enter(root, None, origin, &scope, None, &ctx)
    }

    /// The file at `path`, parsed on first use, with `reach` recorded.
    fn load(
        &mut self,
        path: &RelPath,
        site: Option<Site<'_>>,
        origin: &Origin,
        reach: Reach,
    ) -> Result<Rc<Parsed>, LoadError> {
        match self.files.entry(path.clone()) {
            Entry::Occupied(mut known) => {
                self.visits.push(Visit {
                    file: path.clone(),
                    reach: known.get().reached.len(),
                });
                known.get_mut().reached.push(reach);
                Ok(Rc::clone(&known.get().parsed))
            }
            Entry::Vacant(slot) => {
                let text = match self.tree.read(path, self.options.max_file_bytes) {
                    Ok(text) => text,
                    Err(TreeError::NotFound { .. }) if site.is_some() => {
                        return Err(unresolved(site, path, origin));
                    }
                    Err(error) => {
                        let (file, line) = site.map_or((path, None), |s| (s.file, Some(s.line)));
                        return Err(LoadError::tree(file, line, origin, error));
                    }
                };
                let facts = parse_source(path, &text, |tree| extract(tree, &text))
                    .map_err(|error| error.reached_by(origin))?
                    .map_err(|problem| extract_error(path, origin, problem))?;
                let parsed = Rc::new(Parsed { text, facts });
                self.visits.push(Visit {
                    file: path.clone(),
                    reach: 0,
                });
                slot.insert(Visited {
                    parsed: Rc::clone(&parsed),
                    reached: vec![reach],
                });
                Ok(parsed)
            }
        }
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
            return Err(LoadError::new(
                file,
                line,
                LoadErrorKind::Budget {
                    budget: Budget::ModulePaths,
                    limit: Budget::ModulePaths.limit(),
                },
            )
            .reached_by(origin));
        }
        Ok(())
    }

    /// Record a visit to `path` and follow its module declarations.
    fn enter(
        &mut self,
        path: &RelPath,
        site: Option<Site<'_>>,
        origin: &Origin,
        scope: &Scope<'_>,
        predicate: Option<&str>,
        ctx: &DirCtx,
    ) -> Result<(), LoadError> {
        if self.active.contains(path) {
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
            module_path: scope.module_path.to_owned(),
            test: scope.test,
            predicate: predicate.map(str::to_owned),
        };
        let parsed = self.load(path, site, origin, reach)?;
        self.active.push(path.clone());
        let result = self.body(&parsed.facts.body, path, ctx, scope);
        self.active.pop();
        result
    }

    fn body(
        &mut self,
        body: &ModBody,
        file: &RelPath,
        ctx: &DirCtx,
        scope: &Scope<'_>,
    ) -> Result<(), LoadError> {
        for module in &body.mods {
            self.module(module, file, ctx, scope)?;
        }
        Ok(())
    }

    fn deeper(module: &ModFact, file: &RelPath, scope: &Scope<'_>) -> Result<(), LoadError> {
        if scope.depth + 1 > Budget::ModuleDepth.limit() {
            return Err(LoadError::new(
                file,
                Some(module.name_line),
                LoadErrorKind::Budget {
                    budget: Budget::ModuleDepth,
                    limit: Budget::ModuleDepth.limit(),
                },
            )
            .reached_by(&Origin::ModDecl {
                module: module.name.clone(),
            }));
        }
        Ok(())
    }

    fn module(
        &mut self,
        module: &ModFact,
        file: &RelPath,
        ctx: &DirCtx,
        scope: &Scope<'_>,
    ) -> Result<(), LoadError> {
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
        let module_path = format!("{}::{}", scope.module_path, module.name);
        let inner = Scope {
            target: scope.target,
            module_path: &module_path,
            test: scope.test || module.test != TestRole::None,
            depth: scope.depth + 1,
        };
        let site = Site {
            file,
            line: module.name_line,
            module: &module.name,
        };
        match &module.inline {
            Some(body) => self.inline(module, body, site, ctx, &inner),
            None => self.declared(module, site, ctx, &inner),
        }
    }

    fn inline(
        &mut self,
        module: &ModFact,
        body: &ModBody,
        site: Site<'_>,
        ctx: &DirCtx,
        scope: &Scope<'_>,
    ) -> Result<(), LoadError> {
        let by_name = Origin::ModDecl {
            module: module.name.clone(),
        };
        if let Some(path) = module.paths.unconditional.first() {
            let origin = Origin::PathAttr {
                module: module.name.clone(),
                predicate: None,
            };
            let dir = join(&ctx.dir, path, site.file, site.line, &origin)?;
            let inside = DirCtx {
                dir,
                relative: None,
            };
            self.body(body, site.file, &inside, scope)?;
        } else {
            let base = ctx.module_dir(&by_name, site.file, site.line)?;
            let dir = join(&base, &module.name, site.file, site.line, &by_name)?;
            let inside = DirCtx {
                dir,
                relative: None,
            };
            self.body(body, site.file, &inside, scope)?;
        }
        for alternative in &module.paths.conditional {
            let origin = Origin::PathAttr {
                module: module.name.clone(),
                predicate: Some(alternative.predicate.clone()),
            };
            let dir = join(&ctx.dir, &alternative.path, site.file, site.line, &origin)?;
            let inside = DirCtx {
                dir,
                relative: None,
            };
            let alternative_scope = Scope {
                test: scope.test || alternative.implies_test,
                ..scope.clone()
            };
            self.count(Some((site.file, site.line)), site.file, &origin)?;
            self.body(body, site.file, &inside, &alternative_scope)?;
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

    fn declared(
        &mut self,
        module: &ModFact,
        site: Site<'_>,
        ctx: &DirCtx,
        scope: &Scope<'_>,
    ) -> Result<(), LoadError> {
        let has_alternatives = !module.paths.conditional.is_empty();
        if let Some(path) = module.paths.unconditional.first() {
            let origin = Origin::PathAttr {
                module: module.name.clone(),
                predicate: None,
            };
            let file = join(&ctx.dir, path, site.file, site.line, &origin)?;
            let file = self.tidy(&file, site, &origin)?;
            self.follow_path_file(&file, site, &origin, scope, None)?;
        } else {
            self.follow_default(module, site, ctx, scope, has_alternatives)?;
        }
        for alternative in &module.paths.conditional {
            let origin = Origin::PathAttr {
                module: module.name.clone(),
                predicate: Some(alternative.predicate.clone()),
            };
            let file = join(&ctx.dir, &alternative.path, site.file, site.line, &origin)?;
            let file = self.tidy(&file, site, &origin)?;
            let alternative_scope = Scope {
                test: scope.test || alternative.implies_test,
                ..scope.clone()
            };
            self.follow_path_file(
                &file,
                site,
                &origin,
                &alternative_scope,
                Some(&alternative.predicate),
            )?;
        }
        Ok(())
    }

    /// A file named by a `path` attribute: a `mod.rs` for its own children.
    fn follow_path_file(
        &mut self,
        file: &RelPath,
        site: Site<'_>,
        origin: &Origin,
        scope: &Scope<'_>,
        predicate: Option<&str>,
    ) -> Result<(), LoadError> {
        let ctx = DirCtx {
            dir: parent_dir(file)?,
            relative: None,
        };
        self.enter(file, Some(site), origin, scope, predicate, &ctx)
    }

    /// `mod name;` without a `path`: `name.rs` or `name/mod.rs`.
    fn follow_default(
        &mut self,
        module: &ModFact,
        site: Site<'_>,
        ctx: &DirCtx,
        scope: &Scope<'_>,
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
                self.enter(&flat, Some(site), &origin, scope, None, &children)
            }
            (false, true) => {
                let children = DirCtx {
                    dir: parent_dir(&nested)?,
                    relative: None,
                };
                self.enter(&nested, Some(site), &origin, scope, None, &children)
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
