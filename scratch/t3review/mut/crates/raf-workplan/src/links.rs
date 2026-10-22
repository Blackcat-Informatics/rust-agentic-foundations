// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The link rules: which paths a link may lead to, and which anchors exist.
//!
//! A [`LinkContext`] is the oracle every link is judged against: the set of
//! tracked paths, the tracked symlinks among them, and the tree the Markdown
//! files that anchors point into are read from. The oracle is the set the
//! caller supplies, not the file system, so a link to a file that exists but
//! is not tracked is still broken.

use std::cell::{OnceCell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::rc::Rc;

use raf_okf::{Body, LinkError, LinkTarget, Location, MAX_TEXT_BYTES};
use raf_source::{EntryKind, RelPath, Tree};

use crate::Finding;
use crate::finding::kind;

/// One more byte than the profile accepts, so an oversized Markdown file is
/// refused by the reader and not silently cut.
const READ_LIMIT: u64 = MAX_TEXT_BYTES as u64 + 1;

/// The paths a link can lead to.
#[derive(Debug, Default)]
struct Universe {
    /// Every tracked path that is not a directory.
    files: BTreeSet<String>,
    /// Every directory that holds a tracked path.
    directories: BTreeSet<String>,
    /// The tracked symlinks, which are also in `files`.
    symlinks: BTreeSet<String>,
    /// The directories a walk of the tree could not list, with the reason.
    unreadable: Vec<(String, String)>,
}

impl Universe {
    fn add_file(&mut self, path: &str) {
        let mut at = path;
        while let Some((parent, _)) = at.rsplit_once('/') {
            if !self.directories.insert(parent.to_string()) {
                break;
            }
            at = parent;
        }
        self.files.insert(path.to_string());
    }

    fn walk(tree: &dyn Tree) -> Self {
        let mut universe = Self::default();
        let mut pending = vec![RelPath::root()];
        while let Some(directory) = pending.pop() {
            let entries = match tree.list(&directory) {
                Ok(entries) => entries,
                Err(error) => {
                    universe
                        .unreadable
                        .push((directory.to_string(), error.to_string()));
                    continue;
                }
            };
            for entry in entries {
                let Ok(path) = directory.join(&entry.name) else {
                    universe.unreadable.push((
                        directory.to_string(),
                        format!("the entry name `{}` is not a path", entry.name),
                    ));
                    continue;
                };
                match entry.kind {
                    EntryKind::Dir => {
                        universe.directories.insert(path.as_str().to_string());
                        pending.push(path);
                    }
                    EntryKind::Symlink => {
                        universe.add_file(path.as_str());
                        universe.symlinks.insert(path.as_str().to_string());
                    }
                    _ => universe.add_file(path.as_str()),
                }
            }
        }
        universe
    }
}

/// The oracle the link rules judge links against.
///
/// A relative link must lead to a path in the set of tracked paths or to a
/// directory that holds one, and not to a tracked symlink. A link with an
/// anchor to a Markdown file must name a heading of that file, which is read
/// through the tree the context was built with; a tracked Markdown file the
/// tree cannot give is reported, not skipped.
///
/// [`LinkContext::new`] takes the tracked paths from the caller, typically
/// the list of the version-control index. [`LinkContext::of_tree`] takes every
/// file the tree lists. For a memory tree and a small fixture that is the
/// tracked set. For a disk tree it also includes untracked and ignored files,
/// build output and the `.git` directory, so a link to a file that is not
/// tracked passes: a caller that checks a working copy passes its tracked list
/// to [`LinkContext::new`].
///
/// A context caches the Markdown files it reads and walks the tree at most
/// once, with interior mutability that is not thread-safe, so it is neither
/// `Send` nor `Sync`; use one on one thread.
///
/// ```rust
/// use std::collections::BTreeSet;
/// use raf_source::{MemoryTree, RelPath};
/// use raf_workplan::{LinkContext, plain};
///
/// let tree = MemoryTree::from([("README.md", "# Readme\n"), ("NOTES.md", "[a](README.md#readme)\n")]);
/// let tracked = BTreeSet::from([RelPath::new("README.md")?, RelPath::new("NOTES.md")?]);
/// let context = LinkContext::new(&tree, tracked);
/// assert!(plain::check("NOTES.md", "[a](README.md#readme)\n", &context).is_empty());
/// assert_eq!(plain::check("NOTES.md", "[a](gone.md)\n", &context)[0].kind, "link-broken");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct LinkContext<'t> {
    tree: &'t dyn Tree,
    universe: OnceCell<Universe>,
    /// Tracked symlinks the caller named on top of the universe.
    symlinks: BTreeSet<String>,
    bodies: RefCell<BTreeMap<String, Option<Rc<Body>>>>,
}

impl fmt::Debug for LinkContext<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LinkContext")
            .field("universe", &self.universe)
            .field("symlinks", &self.symlinks)
            .finish_non_exhaustive()
    }
}

impl<'t> LinkContext<'t> {
    /// A context whose tracked paths are `tracked`, with the Markdown files
    /// that anchors point into read from `tree`.
    #[must_use]
    pub fn new(tree: &'t dyn Tree, tracked: impl IntoIterator<Item = RelPath>) -> Self {
        let mut universe = Universe::default();
        for path in tracked {
            universe.add_file(path.as_str());
        }
        let cell = OnceCell::new();
        // A new cell is empty, so the set cannot fail.
        let _ = cell.set(universe);
        Self {
            tree,
            universe: cell,
            symlinks: BTreeSet::new(),
            bodies: RefCell::new(BTreeMap::new()),
        }
    }

    /// A context whose tracked paths are every file `tree` lists, and whose
    /// tracked symlinks are the symlinks it lists. The tree is walked on the
    /// first relative link, and a directory it cannot list is reported by
    /// [`Workplan::check`](crate::Workplan::check) as `unreadable-directory`.
    ///
    /// On a disk tree the listing holds untracked and ignored files, build
    /// output and `.git`, which are not tracked paths. Use this for a memory
    /// tree or a tree that holds only tracked files, and give a working copy
    /// its tracked list with [`LinkContext::new`].
    #[must_use]
    pub fn of_tree(tree: &'t dyn Tree) -> Self {
        Self {
            tree,
            universe: OnceCell::new(),
            symlinks: BTreeSet::new(),
            bodies: RefCell::new(BTreeMap::new()),
        }
    }

    /// Name tracked symlinks. They are tracked paths too, and a link to one
    /// is `link-symlink`. The directories that hold them count as tracked
    /// directories.
    #[must_use]
    pub fn with_symlinks(mut self, symlinks: impl IntoIterator<Item = RelPath>) -> Self {
        self.symlinks
            .extend(symlinks.into_iter().map(|path| path.as_str().to_string()));
        self
    }

    fn universe(&self) -> &Universe {
        self.universe.get_or_init(|| Universe::walk(self.tree))
    }

    /// The directories the walk of the tree could not list, if it ran.
    pub(crate) fn unreadable(&self) -> &[(String, String)] {
        self.universe
            .get()
            .map_or(&[], |universe| universe.unreadable.as_slice())
    }

    /// Whether the path is tracked: a file, a symlink, or a directory that
    /// holds one. The empty path is the root.
    pub(crate) fn is_tracked(&self, path: &str) -> bool {
        let universe = self.universe();
        path.is_empty()
            || universe.files.contains(path)
            || universe.directories.contains(path)
            || self.symlinks.contains(path)
            || self.symlinks.iter().any(|link| {
                link.strip_prefix(path)
                    .is_some_and(|rest| rest.starts_with('/'))
            })
    }

    /// Whether the path is a tracked file or symlink, not a directory.
    pub(crate) fn is_tracked_file(&self, path: &str) -> bool {
        self.universe().files.contains(path) || self.symlinks.contains(path)
    }

    fn is_symlink(&self, path: &str) -> bool {
        self.universe().symlinks.contains(path) || self.symlinks.contains(path)
    }

    /// The body of the Markdown file at `path`, or `None` when the tree
    /// cannot give it or it does not read.
    fn body_of(&self, path: &str) -> Option<Rc<Body>> {
        if let Some(found) = self.bodies.borrow().get(path) {
            return found.clone();
        }
        let body = RelPath::new(path)
            .ok()
            .and_then(|rel| self.tree.read(&rel, READ_LIMIT).ok())
            .and_then(|text| Body::parse(&text).ok())
            .map(Rc::new);
        self.bodies
            .borrow_mut()
            .insert(path.to_string(), body.clone());
        body
    }
}

/// Whether a repository path names a Markdown file.
#[allow(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "the profile defines a Markdown file by the lowercase suffix `.md`"
)]
pub(crate) fn is_markdown(path: &str) -> bool {
    path.ends_with(".md")
}

/// Judge every link of `body`, the body of the document at `file`.
pub(crate) fn check_body(
    file: &str,
    body: &Body,
    context: &LinkContext<'_>,
    out: &mut Vec<Finding>,
) {
    for link in body.links() {
        let report = |kind: &'static str, message: String, fix: &str| {
            Finding::new(kind, file, Location::Line(link.line()), message, fix)
        };
        match link.target_in(file) {
            Err(error @ LinkError::Empty) => out.push(report(
                kind::LINK_BROKEN,
                error.to_string(),
                "give the link a target, or remove it",
            )),
            Err(error) => out.push(report(
                kind::LINK_OUTSIDE,
                error.to_string(),
                "link to a path inside the repository, relative to this file",
            )),
            Ok(LinkTarget::External(_)) => {}
            Ok(LinkTarget::Fragment(anchor)) => {
                if !anchor.is_empty() && !body.has_anchor(&anchor) {
                    out.push(report(
                        kind::ANCHOR_BROKEN,
                        format!("broken anchor `{}`", link.target()),
                        ANCHOR_FIX,
                    ));
                }
            }
            Ok(LinkTarget::Relative { path, anchor }) => {
                if let Some((kind, message, fix)) =
                    judge_relative(context, link.target(), &path, anchor.as_deref())
                {
                    out.push(report(kind, message, fix));
                }
            }
        }
    }
}

const ANCHOR_FIX: &str =
    "use the anchor of an existing heading of the target: lowercase, with hyphens for spaces";

/// The verdict on a link to a path inside the repository.
fn judge_relative(
    context: &LinkContext<'_>,
    target: &str,
    path: &str,
    anchor: Option<&str>,
) -> Option<(&'static str, String, &'static str)> {
    if !context.is_tracked(path) {
        return Some((
            kind::LINK_BROKEN,
            format!("broken link `{target}`"),
            "point the link at a tracked path, or remove it",
        ));
    }
    if context.is_symlink(path) {
        return Some((
            kind::LINK_SYMLINK,
            format!("link `{target}` targets a tracked symlink"),
            "link to the regular file the symlink stands for",
        ));
    }
    let anchor = anchor.filter(|anchor| !anchor.is_empty())?;
    if !context.is_tracked_file(path) || !is_markdown(path) {
        return None;
    }
    match context.body_of(path) {
        Some(body) if body.has_anchor(anchor) => None,
        Some(_) => Some((
            kind::ANCHOR_BROKEN,
            format!("broken anchor `{target}`"),
            ANCHOR_FIX,
        )),
        None => Some((
            kind::ANCHOR_BROKEN,
            format!("link `{target}` targets `{path}`, which cannot be read as Markdown"),
            "make the target a Markdown file that reads, or drop the anchor",
        )),
    }
}
