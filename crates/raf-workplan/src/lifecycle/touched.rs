// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `contract-effect-understated`: under the complete policy, a change that says
//! `contract_effect: none` and touches a bound public item.
//!
//! The public surface of a side is read from a source index: the working
//! tree's, and one loaded at the merge base from its files. On each side the
//! units are the items of the public surface of every library and binary
//! target of a package that holds a bound item ([`Workspace::exported_items`]:
//! the items another crate can name, the `pub` members of the inherent impls
//! of an exported type and the members of an exported trait), and the
//! members of the trait impls of an exported struct, enum or union. A unit is
//! the same unit on both sides when its path, its kind and, for a member of an
//! impl block, the tokens of the impl's header (its attributes, generics,
//! trait, self type and where clause) are the same. It is bound when a `code`
//! binding of any loaded capability covers it, itself or as an item below it,
//! as two-way coverage reads it ([`covering`], [`covering_path`]).
//!
//! "Touch" is read literally. A unit is compared with its own counterpart as
//! the `proc_macro2` token stream of its span: a doc comment arrives as a
//! `#[doc]` attribute and counts, while whitespace and comments that are not
//! doc comments are not tokens and do not; CRLF line endings read as LF, as
//! `rustc` reads them. Then:
//!
//! * a bound unit whose counterpart has another stream is [`Touch::Tokens`];
//! * a bound unit with no counterpart is [`Touch::New`]: new, made public,
//!   given a new path, or under a changed impl header;
//! * a bound unit of the merge base with no counterpart is
//!   [`Touch::Hidden`] when the working tree still has an item of its path and
//!   kind, off the surface, and [`Touch::Removed`] otherwise;
//! * a re-export path of the surface that names another unit than at the
//!   merge base is [`Touch::Retargeted`], one that is new is [`Touch::New`]
//!   and one that is gone is [`Touch::Removed`].
//!
//! A path is reported once, by the first of these that holds.
//!
//! The source index at the merge base is loaded from the directories of the
//! packages that hold a bound item, through a [`Tree`] of the files of the
//! merge base: `git ls-tree -r -z --full-tree <merge-base>` lists them once,
//! and each file the load reads is read once with the row
//! `git cat-file blob <merge-base>:<prefix><path>` that `section-untouched`
//! uses, within the 1 MiB bound of a blob. [`BaseIndex`] keeps that index, so
//! the two judgements of `complete` at one merge base read it once.
//!
//! A merge base whose index does not load (a file that is not UTF-8 or does
//! not parse, a module that is missing) is an error and not a pass, and so is
//! a surface that a source index cannot list.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::ops::Range;

use proc_macro2::{Delimiter, LineColumn, Spacing, TokenStream, TokenTree};
use raf_source::syn::Item;
use raf_source::syn::spanned::Spanned;
use raf_source::{
    Entry, EntryKind, FileId, LoadError, Opened as TreeRead, RelPath, SelfType, SymbolKind,
    TargetKind, TestRole, Tree, TreeError, Workspace, parse_source,
};

use crate::Workplan;
use crate::coverage::{covering, covering_path};
use crate::exec::Runner;
use crate::git::{self, MergeBase, Prefix, TreeEntry};
use crate::graph::{Binding, BindingKind};
use crate::obligation::{Needed, Obligation, ObligationKind, Part, Place, Subject, Touch, Touched};

use super::session::{LifecycleError, Side};
use super::untouched::{blob, merge_base};

/// The source index loaded at one merge base from the directories of the
/// packages that hold a bound item, kept between two judgements of one
/// change so it is read once: the files at a commit do not change.
#[derive(Debug)]
pub(super) struct BaseIndex {
    /// The commit id of the merge base.
    commit: String,
    /// The directories it was loaded from, sorted.
    roots: Vec<String>,
    /// The index, or `None` when none of the directories is at the merge
    /// base.
    workspace: Option<Workspace>,
}

/// The `contract-effect-understated` obligations of the change `change`, one
/// for each touched bound public item, on the change record.
///
/// The caller asks only under the complete policy and for a change with
/// `contract_effect: none`. Nothing is asked of `git` when no `code` binding
/// resolves to an item; otherwise the merge base is found once, kept in `base`
/// for the other kinds, and the source index at it is loaded once, kept in
/// `kept` for the next judgement at the same merge base.
pub(super) fn raise(
    plan: &Workplan<'_>,
    runner: &dyn Runner,
    prefix: &Prefix,
    change: &str,
    base: &mut Option<MergeBase>,
    kept: &mut Option<BaseIndex>,
) -> Result<Vec<Obligation>, LifecycleError> {
    let record = plan.change(change)?;
    let source = plan.source();
    let packages = bound_packages(plan);
    if packages.is_empty() {
        return Ok(Vec::new());
    }
    let now = Surface::of(source, &packages, Side::WorkingTree)?;
    let base = merge_base(base, runner)?;
    let roots = package_roots(source, &packages);
    let current = kept
        .as_ref()
        .is_some_and(|index| index.commit == base.as_str() && index.roots == roots);
    if !current {
        let workspace = load_at(runner, base, prefix, &roots)?;
        *kept = Some(BaseIndex {
            commit: base.as_str().to_string(),
            roots,
            workspace,
        });
    }
    let then_source = kept.as_ref().and_then(|index| index.workspace.as_ref());
    let then = match then_source {
        Some(workspace) => Surface::of(workspace, &packages, Side::MergeBase)?,
        None => Surface::default(),
    };
    let touched = compare(plan, &now, then_source, &then);
    Ok(touched
        .into_iter()
        .map(|touched| Obligation {
            kind: ObligationKind::ContractEffectUnderstated,
            subject: Subject::Change(record.value().header.id.clone()),
            section: Part::Document,
            location: Place {
                path: record.path().to_string(),
                line: None,
            },
            evidence: Needed::StatedEffect(touched),
        })
        .collect())
}

/// The packages that hold an item a `code` binding resolves to.
fn bound_packages<'p>(plan: &'p Workplan<'_>) -> BTreeSet<&'p str> {
    let source = plan.source();
    plan.graph()
        .bindings()
        .iter()
        .filter(|binding| binding.kind() == BindingKind::Code)
        .flat_map(|binding| binding.items().iter())
        .filter_map(|id| source.item(*id).and_then(|item| source.target(item.target)))
        .map(|target| target.package.as_str())
        .collect()
}

/// The directories of `packages` in the working tree, sorted: the roots the
/// index at the merge base is loaded from.
fn package_roots(source: &Workspace, packages: &BTreeSet<&str>) -> Vec<String> {
    let mut roots: Vec<String> = source
        .packages()
        .iter()
        .filter(|package| packages.contains(package.name.as_str()))
        .map(|package| match package.dir.as_str() {
            "" => ".".to_string(),
            dir => dir.to_string(),
        })
        .collect();
    roots.sort();
    roots.dedup();
    roots
}

/// The source index of the merge base below `roots`, or `None` when none of
/// them is a directory there.
fn load_at(
    runner: &dyn Runner,
    base: &MergeBase,
    prefix: &Prefix,
    roots: &[String],
) -> Result<Option<Workspace>, LifecycleError> {
    let tree = BaseTree::new(runner, base, prefix, git::tree_at(runner, base, prefix)?);
    let present: Vec<&str> = roots
        .iter()
        .map(String::as_str)
        .filter(|root| tree.is_dir(root))
        .collect();
    if present.is_empty() {
        return Ok(None);
    }
    match Workspace::load(&tree, &present) {
        Ok(workspace) => Ok(Some(workspace)),
        Err(error) => {
            Err(tree
                .failure
                .borrow_mut()
                .take()
                .unwrap_or_else(|| LifecycleError::Document {
                    path: error.file.to_string(),
                    side: Side::MergeBase,
                    reason: error.to_string(),
                }))
        }
    }
}

/// The identity of a unit on one side: its path, its kind and, for a member
/// of an impl block, the rendered tokens of the impl's header.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    path: String,
    kind: SymbolKind,
    header: String,
}

/// A unit of a side: its item and the rendered token stream of its span.
#[derive(Debug)]
struct Unit<'w> {
    item: &'w raf_source::Item,
    stream: String,
}

/// The public surface of one side, as the rule compares it.
#[derive(Debug, Default)]
struct Surface<'w> {
    units: BTreeMap<Key, Unit<'w>>,
    /// Each path of the surface, with the unit it names.
    exports: BTreeMap<String, Key>,
}

impl<'w> Surface<'w> {
    /// The units of the library and binary targets of `packages` in
    /// `source`.
    fn of(
        source: &'w Workspace,
        packages: &BTreeSet<&str>,
        side: Side,
    ) -> Result<Self, LifecycleError> {
        let mut surface = Self::default();
        let mut headers = Headers::default();
        for id in source.target_ids() {
            let Some(target) = source.target(id) else {
                continue;
            };
            if !matches!(target.kind, TargetKind::Lib | TargetKind::Bin)
                || !packages.contains(target.package.as_str())
            {
                continue;
            }
            let exports = source
                .exported_items(id)
                .map_err(|error| LifecycleError::Surface {
                    package: target.package.clone(),
                    target: target.crate_path.clone(),
                    side,
                    reason: error.to_string(),
                })?;
            let mut types = BTreeSet::new();
            for export in &exports.items {
                let Some(item) = source.item(export.item) else {
                    continue;
                };
                let key = surface.add(source, item, &mut headers, side)?;
                surface.exports.insert(export.path.clone(), key);
                if matches!(
                    item.kind,
                    SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Union
                ) {
                    types.insert(item.path.as_str());
                }
            }
            // The members of the trait impls of an exported type: the
            // surface leaves them out, and they are what the type does.
            let members: Vec<&raf_source::Item> = source
                .items()
                .filter(|item| {
                    item.target == id
                        && item.test == TestRole::None
                        && item.trait_path.is_some()
                        && item.self_key == Some(SelfType::Local)
                        && item
                            .path
                            .rsplit_once("::")
                            .is_some_and(|(owner, _)| types.contains(owner))
                })
                .collect();
            for item in members {
                surface.add(source, item, &mut headers, side)?;
            }
        }
        Ok(surface)
    }

    /// Add `item` as a unit, once, and return its key.
    fn add(
        &mut self,
        source: &'w Workspace,
        item: &'w raf_source::Item,
        headers: &mut Headers,
        side: Side,
    ) -> Result<Key, LifecycleError> {
        let document = |reason: String| LifecycleError::Document {
            path: item.file.to_string(),
            side,
            reason,
        };
        let text = source
            .source(&item.file)
            .ok_or_else(|| document("is not in the source index".to_string()))?;
        let header = if item.self_type.is_some() {
            headers
                .of(&item.file, text)
                .map_err(|error| document(error.to_string()))?
                .iter()
                .filter(|impl_block| impl_block.body.contains(&item.span.start_byte))
                .min_by_key(|impl_block| impl_block.body.len())
                .map_or_else(
                    || "\u{0}no impl block".to_string(),
                    |impl_block| impl_block.header.clone(),
                )
        } else {
            String::new()
        };
        let key = Key {
            path: item.path.to_string(),
            kind: item.kind,
            header,
        };
        self.units.entry(key.clone()).or_insert_with(|| Unit {
            item,
            stream: Reader::new(text).stream_of(item.span.start_byte..item.span.end_byte),
        });
        Ok(key)
    }
}

/// The touched units of `now` against `then`, the surface of `then_source`
/// at the merge base, sorted by file, line and path.
fn compare(
    plan: &Workplan<'_>,
    now: &Surface<'_>,
    then_source: Option<&Workspace>,
    then: &Surface<'_>,
) -> Vec<Touched> {
    let graph = plan.graph();
    let source = plan.source();
    let at_base = |item: &raf_source::Item| -> Option<&Binding> {
        let same = then_source.map_or(&[][..], |workspace| workspace.same_source(item));
        std::iter::once(item.path.as_str())
            .chain(
                same.iter()
                    .filter_map(|id| then_source.and_then(|w| w.item(*id)))
                    .map(|item| item.path.as_str()),
            )
            .find_map(|path| covering_path(graph, source, path))
    };
    let mut found: BTreeMap<String, Touched> = BTreeMap::new();
    let mut report = |path: &str, item: &raf_source::Item, binding: &Binding, how: Touch| {
        found.entry(path.to_string()).or_insert_with(|| Touched {
            item: path.to_string(),
            file: item.file.to_string(),
            line: item.name_line,
            capability: binding.capability().to_string(),
            how,
        });
    };

    for (key, unit) in &now.units {
        let how = match then.units.get(key) {
            Some(old) if old.stream == unit.stream => continue,
            Some(_) => Touch::Tokens,
            None => Touch::New,
        };
        if let Some(binding) = covering(graph, source, unit.item.id) {
            report(&key.path, unit.item, binding, how);
        }
    }
    for (key, unit) in &then.units {
        if now.units.contains_key(key) {
            continue;
        }
        let Some(binding) = at_base(unit.item) else {
            continue;
        };
        let kept = source
            .items_under(&key.path)
            .find(|item| *item.path == *key.path && item.kind == key.kind);
        match kept {
            Some(item) => report(&key.path, item, binding, Touch::Hidden),
            None => report(&key.path, unit.item, binding, Touch::Removed),
        }
    }
    for (path, key) in &now.exports {
        if *path == key.path {
            continue;
        }
        let how = match then.exports.get(path) {
            Some(old) if old == key => continue,
            Some(_) => Touch::Retargeted,
            None => Touch::New,
        };
        if let Some(unit) = now.units.get(key)
            && let Some(binding) = covering(graph, source, unit.item.id)
        {
            report(path, unit.item, binding, how);
        }
    }
    for (path, key) in &then.exports {
        if *path == key.path || now.exports.contains_key(path) {
            continue;
        }
        if let Some(unit) = then.units.get(key)
            && let Some(binding) = at_base(unit.item)
        {
            report(path, unit.item, binding, Touch::Removed);
        }
    }

    let mut touched: Vec<Touched> = found.into_values().collect();
    touched.sort_by(|a, b| (&a.file, a.line, &a.item).cmp(&(&b.file, b.line, &b.item)));
    touched
}

// --- the headers of impl blocks ----------------------------------------------

/// An impl block of a file: the bytes of its braced body and the rendered
/// tokens of everything before the brace.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ImplHeader {
    body: Range<usize>,
    header: String,
}

/// The impl blocks of each file read so far.
#[derive(Default)]
struct Headers {
    files: BTreeMap<RelPath, Vec<ImplHeader>>,
}

impl Headers {
    fn of(&mut self, path: &RelPath, text: &str) -> Result<&[ImplHeader], LoadError> {
        if !self.files.contains_key(path) {
            let read = impl_headers(path, text)?;
            self.files.insert(path.clone(), read);
        }
        Ok(self.files.get(path).map_or(&[][..], Vec::as_slice))
    }
}

/// Parse `text` through [`parse_source`] and read every impl block of it, at
/// every depth of inline modules.
///
/// The spans are read on the thread that parsed, which holds the location
/// data of the parse, and leave it as plain values.
fn impl_headers(path: &RelPath, text: &str) -> Result<Vec<ImplHeader>, LoadError> {
    parse_source(path, text, |file| {
        let reader = Reader::new(text);
        let mut out = Vec::new();
        reader.impls(&file.items, &mut out);
        out
    })
}

/// The text of a file with the byte offset where each line starts.
struct Reader<'t> {
    text: &'t str,
    starts: Vec<usize>,
}

impl<'t> Reader<'t> {
    fn new(text: &'t str) -> Self {
        let starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(at, _)| at + 1))
            .collect();
        Self { text, starts }
    }

    /// The byte offset of a line and column of the parse: the line is 1-based
    /// and the column counts characters from 0.
    fn byte(&self, at: LineColumn) -> Option<usize> {
        let start = *self.starts.get(at.line.checked_sub(1)?)?;
        let line = self.text.get(start..)?;
        let line = line.split('\n').next().unwrap_or(line);
        let offset = line
            .char_indices()
            .map(|(offset, _)| offset)
            .chain(std::iter::once(line.len()))
            .nth(at.column)?;
        Some(start + offset)
    }

    /// The rendered stream of the bytes `range`.
    fn stream_of(&self, range: Range<usize>) -> String {
        let Some(slice) = self.text.get(range.clone()) else {
            // A span that is not a range of the text names no item of it.
            return format!("\u{0}no text at {}..{}", range.start, range.end);
        };
        let slice = slice.replace("\r\n", "\n");
        match slice.parse::<TokenStream>() {
            Ok(stream) => {
                let mut out = String::new();
                render(stream, &mut out);
                out
            }
            // The parse accepted the file, so its items lex; a slice that does
            // not is compared by its text, which no stream equals.
            Err(_) => format!("\u{0}{slice}"),
        }
    }

    /// Read the impl blocks of `items` and of the inline modules among them.
    fn impls(&self, items: &[Item], out: &mut Vec<ImplHeader>) {
        for item in items {
            match item {
                Item::Impl(block) => {
                    let start = self.byte(block.span().start());
                    let open = self.byte(block.brace_token.span.open().start());
                    let close = self.byte(block.brace_token.span.close().end());
                    if let (Some(start), Some(open), Some(close)) = (start, open, close)
                        && start <= open
                        && open <= close
                    {
                        out.push(ImplHeader {
                            body: open..close,
                            header: self.stream_of(start..open),
                        });
                    }
                }
                Item::Mod(module) => {
                    if let Some((_, content)) = &module.content {
                        self.impls(content, out);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Write `stream` as a comparison key: each identifier and literal as its
/// text between spaces, each punctuation character followed by a space unless
/// it is joined to the next, and each group between its delimiters. Spans,
/// whitespace and comments that are not doc comments leave no trace.
fn render(stream: TokenStream, out: &mut String) {
    for tree in stream {
        match tree {
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    Delimiter::Parenthesis => ('(', ')'),
                    Delimiter::Brace => ('{', '}'),
                    Delimiter::Bracket => ('[', ']'),
                    Delimiter::None => ('\u{2}', '\u{3}'),
                };
                out.push(open);
                render(group.stream(), out);
                out.push(close);
            }
            TokenTree::Ident(ident) => {
                out.push(' ');
                out.push_str(&ident.to_string());
                out.push(' ');
            }
            TokenTree::Punct(punct) => {
                out.push(punct.as_char());
                if punct.spacing() == Spacing::Alone {
                    out.push(' ');
                }
            }
            TokenTree::Literal(literal) => {
                out.push(' ');
                out.push_str(&literal.to_string());
                out.push(' ');
            }
        }
    }
}

// --- the files of the merge base ---------------------------------------------

/// The most symbolic links one read follows before it is a loop.
const MAX_LINKS: usize = 40;

/// The files of the merge base below the workspace root, as a [`Tree`]: the
/// listing answers every question of entries and directories, and a file is
/// read with the blob row the first time the load reads it.
struct BaseTree<'r> {
    runner: &'r dyn Runner,
    base: &'r MergeBase,
    prefix: &'r Prefix,
    files: BTreeMap<String, TreeEntry>,
    dirs: BTreeSet<String>,
    texts: RefCell<BTreeMap<String, String>>,
    /// The first failure of `git`, which the load can only report as an
    /// error of the tree.
    failure: RefCell<Option<LifecycleError>>,
}

impl<'r> BaseTree<'r> {
    fn new(
        runner: &'r dyn Runner,
        base: &'r MergeBase,
        prefix: &'r Prefix,
        listing: Vec<(RelPath, TreeEntry)>,
    ) -> Self {
        let mut files = BTreeMap::new();
        let mut dirs = BTreeSet::from([String::new()]);
        for (path, kind) in listing {
            let key = path.as_str().to_string();
            let mut at = key.as_str();
            while let Some((parent, _)) = at.rsplit_once('/') {
                dirs.insert(parent.to_string());
                at = parent;
            }
            files.insert(key, kind);
        }
        Self {
            runner,
            base,
            prefix,
            files,
            dirs,
            texts: RefCell::new(BTreeMap::new()),
            failure: RefCell::new(None),
        }
    }

    fn is_dir(&self, root: &str) -> bool {
        let key = if root == "." { "" } else { root };
        self.dirs.contains(key)
    }

    /// The key of the file `path` names, after its symbolic links.
    fn file_key(&self, path: &RelPath) -> Result<String, TreeError> {
        let mut key = collapse(path.as_str()).ok_or_else(|| TreeError::OutsideRoot {
            path: path.to_string(),
        })?;
        for _ in 0..MAX_LINKS {
            match self.files.get(&key) {
                Some(TreeEntry::File) => return Ok(key),
                Some(TreeEntry::Symlink) => {
                    let target = self.text(&key, path)?;
                    let joined = match key.rsplit_once('/') {
                        _ if target.starts_with('/') => None,
                        Some((dir, _)) => collapse(&format!("{dir}/{target}")),
                        None => collapse(&target),
                    };
                    key = joined.ok_or_else(|| TreeError::OutsideRoot {
                        path: path.to_string(),
                    })?;
                }
                Some(TreeEntry::Submodule) => {
                    return Err(TreeError::NotRegular {
                        path: path.to_string(),
                    });
                }
                None if self.dirs.contains(&key) => {
                    return Err(TreeError::NotRegular {
                        path: path.to_string(),
                    });
                }
                None => {
                    return Err(TreeError::NotFound {
                        path: path.to_string(),
                    });
                }
            }
        }
        Err(TreeError::SymlinkLoop {
            path: path.to_string(),
        })
    }

    /// The text of the listed file `key`, read once.
    fn text(&self, key: &str, path: &RelPath) -> Result<String, TreeError> {
        if let Some(text) = self.texts.borrow().get(key) {
            return Ok(text.clone());
        }
        let bytes = match blob(self.runner, self.base, self.prefix, key) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                return Err(TreeError::NotFound {
                    path: path.to_string(),
                });
            }
            Err(error) => {
                let reason = error.to_string();
                self.failure.borrow_mut().get_or_insert(error);
                return Err(TreeError::Io {
                    path: path.to_string(),
                    source: io::Error::other(reason),
                });
            }
        };
        let text = String::from_utf8(bytes).map_err(|_| TreeError::NotUtf8 {
            path: path.to_string(),
        })?;
        self.texts
            .borrow_mut()
            .insert(key.to_string(), text.clone());
        Ok(text)
    }
}

impl Tree for BaseTree<'_> {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        let key = self.file_key(path)?;
        let text = self.text(&key, path)?;
        if u64::try_from(text.len()).map_or(true, |len| len > max_bytes) {
            return Err(TreeError::TooLarge {
                path: path.to_string(),
                limit: max_bytes,
            });
        }
        Ok(text)
    }

    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<TreeRead, TreeError> {
        let key = self.file_key(path)?;
        let id = FileId::from_key(&key);
        if seen(&id) {
            return Ok(TreeRead::Seen(id));
        }
        Ok(TreeRead::Read {
            id,
            text: self.read(path, max_bytes)?,
        })
    }

    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        let key = collapse(path.as_str()).ok_or_else(|| TreeError::OutsideRoot {
            path: path.to_string(),
        })?;
        if self.files.contains_key(&key) {
            return Err(TreeError::NotRegular {
                path: path.to_string(),
            });
        }
        if !self.dirs.contains(&key) {
            return Err(TreeError::NotFound {
                path: path.to_string(),
            });
        }
        let below = if key.is_empty() {
            String::new()
        } else {
            format!("{key}/")
        };
        let mut entries: BTreeMap<&str, EntryKind> = BTreeMap::new();
        for (file, kind) in self.files.range(below.clone()..) {
            let Some(rest) = file.strip_prefix(&below) else {
                break;
            };
            match rest.split_once('/') {
                Some((dir, _)) => {
                    entries.insert(dir, EntryKind::Dir);
                }
                None => {
                    entries.insert(
                        rest,
                        match kind {
                            TreeEntry::File => EntryKind::File,
                            TreeEntry::Symlink => EntryKind::Symlink,
                            TreeEntry::Submodule => EntryKind::Other,
                        },
                    );
                }
            }
        }
        Ok(entries
            .into_iter()
            .map(|(name, kind)| Entry::new(name, kind))
            .collect())
    }

    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        let Some(key) = collapse(path.as_str()) else {
            return Err(TreeError::OutsideRoot {
                path: path.to_string(),
            });
        };
        Ok(match self.files.get(&key) {
            Some(TreeEntry::File) => Some(EntryKind::File),
            Some(TreeEntry::Symlink) => Some(EntryKind::Symlink),
            Some(TreeEntry::Submodule) => Some(EntryKind::Other),
            None if self.dirs.contains(&key) => Some(EntryKind::Dir),
            None => None,
        })
    }
}

/// The lexical form of a relative path, `.` dropped and `..` applied against
/// its own segments; `None` when it climbs above the root.
fn collapse(path: &str) -> Option<String> {
    let mut kept: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                kept.pop()?;
            }
            other => kept.push(other),
        }
    }
    Some(kept.join("/"))
}

#[cfg(test)]
mod tests;
