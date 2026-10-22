// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reachability: the items and impl blocks of every target that a set of root
//! items reach over three edges.
//!
//! * **contains**: a struct, enum or union to its methods and associated
//!   items, a trait to its members, and a module reached from a root by
//!   contains alone to its items and the impl blocks it holds. A crate root
//!   contains nothing, and a module reached any other way reaches nothing
//!   below it.
//! * **impl-of**: a struct, enum or union to the impl blocks whose self type
//!   it is, and a block to its members.
//! * **references**: an item or a block to each item that a path in its own
//!   tokens names, read as code of its module reads it. The paths are the
//!   `Ident (:: Ident)*` runs that no `.` precedes: each proper prefix is read
//!   in the type name space and the whole run in each name space.
//!
//! The tokens of an item are those of its span outside the spans of the items
//! and blocks below it: a type's members and a block's members are their own
//! units. A module's own tokens are its declaration before its body; what a
//! module body holds outside its items and blocks (its `use` declarations and
//! the macro invocations at module level) is no item's, and a crate root has
//! none. Each file is lexed once, the first time an item of it is reached, and
//! each run is resolved at most once for each module and test flag it is read
//! with.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::panic;
use std::rc::Rc;
use std::str::FromStr;
use std::thread;

use proc_macro2::{Spacing, TokenStream, TokenTree};

use crate::error::{LoadError, LoadErrorKind};
use crate::facts::TestRole;
use crate::index::Index;
use crate::item::{ImplId, ItemId, SymbolKind};
use crate::parse::PARSE_STACK_BYTES;
use crate::path::RelPath;
use crate::work::{self, Work};
use crate::workspace::SourceFile;

/// A node of the reachability graph: an item or an impl block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Node {
    /// An item of the index.
    Item(ItemId),
    /// An impl block of [`Workspace::impls`](crate::Workspace::impls).
    Impl(ImplId),
}

impl From<ItemId> for Node {
    fn from(id: ItemId) -> Self {
        Self::Item(id)
    }
}

impl From<ImplId> for Node {
    fn from(id: ImplId) -> Self {
        Self::Impl(id)
    }
}

/// The edge a node was first reached by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Edge {
    /// The node is one of the roots.
    Root,
    /// The node is below the node it was reached from: a member of a type or
    /// a trait, or an item or impl block of a module.
    Contains,
    /// The node is an impl block of the type it was reached from, or a
    /// member of the impl block it was reached from.
    ImplOf,
    /// A path in the tokens of the node it was reached from names it.
    References,
}

/// One reached node, with the edge it was first reached by and the node that
/// edge comes from (`None` for a root).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Reached {
    /// The node.
    pub node: Node,
    /// The edge it was first reached by.
    pub edge: Edge,
    /// The node that edge comes from; `None` for a root.
    pub from: Option<Node>,
}

/// What [`Workspace::reachable`](crate::Workspace::reachable) found: every
/// reached node once, in the order the walk reached them, breadth first from
/// the roots in the order of their nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Reachability {
    /// The reached nodes, roots first.
    pub reached: Vec<Reached>,
    /// The place of each node in `reached`.
    at: BTreeMap<Node, usize>,
}

impl Reachability {
    /// How `node` was reached, or `None` when it was not.
    #[must_use]
    pub fn get(&self, node: Node) -> Option<&Reached> {
        self.at.get(&node).map(|at| &self.reached[*at])
    }

    /// True when `node` was reached.
    #[must_use]
    pub fn contains(&self, node: Node) -> bool {
        self.at.contains_key(&node)
    }

    /// The reached items, in the order of `reached`.
    pub fn items(&self) -> impl Iterator<Item = ItemId> + '_ {
        self.reached
            .iter()
            .filter_map(|reached| match reached.node {
                Node::Item(id) => Some(id),
                Node::Impl(_) => None,
            })
    }

    /// The reached impl blocks, in the order of `reached`.
    pub fn impls(&self) -> impl Iterator<Item = ImplId> + '_ {
        self.reached
            .iter()
            .filter_map(|reached| match reached.node {
                Node::Impl(id) => Some(id),
                Node::Item(_) => None,
            })
    }
}

/// Why the reachability of a set of roots could not be given.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReachError {
    /// A root names no item of this workspace.
    UnknownItem(ItemId),
    /// A root names no impl block of this workspace.
    UnknownImpl(ImplId),
    /// The text of a file of a reached item could not be read for its paths:
    /// the thread that lexes it could not start
    /// ([`LoadErrorKind::Spawn`]), or the text does not lex
    /// ([`LoadErrorKind::Syntax`]), which a file the load parsed does not do.
    Source(LoadError),
}

impl fmt::Display for ReachError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownItem(id) => {
                write!(f, "item {} is not an item of the workspace", id.index())
            }
            Self::UnknownImpl(id) => {
                write!(
                    f,
                    "impl block {} is not an impl block of the workspace",
                    id.index()
                )
            }
            Self::Source(error) => write!(f, "{error}"),
        }
    }
}

impl Error for ReachError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            Self::UnknownItem(_) | Self::UnknownImpl(_) => None,
        }
    }
}

/// A path run of a file: the byte offset of its first identifier, whether a
/// `::` opens it, its segments without a raw-identifier prefix, and its text,
/// which keys what it resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Run {
    start: usize,
    leading_colon: bool,
    segs: Vec<String>,
    text: String,
}

/// What lexing one file gave: its runs in text order and the tokens read.
#[derive(Debug, Default)]
struct Lexed {
    runs: Vec<Run>,
    tokens: usize,
}

/// A lexing failure as plain data, read on the thread that lexed.
struct LexFailure {
    message: String,
    line: usize,
    column: usize,
}

/// The words that are no segment of a path, beside `crate`, `self` and
/// `super`, which start one, and `Self`, which the index does not resolve:
/// the strict and reserved keywords of every edition.
const KEYWORDS: [&str; 49] = [
    "Self", "abstract", "as", "async", "await", "become", "box", "break", "const", "continue",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl",
    "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe", "unsized",
    "use", "virtual", "where", "while", "yield",
];

/// True when `word` cannot be a segment of a path at all.
fn blocked(word: &str) -> bool {
    KEYWORDS.contains(&word)
}

/// The words after which an identifier is the name an item declares, not a
/// path: `fn helper` declares `helper`.
const DECLARES: [&str; 9] = [
    "fn", "struct", "enum", "union", "trait", "type", "mod", "const", "static",
];

/// What an identifier is to the scanner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Word {
    /// Any other identifier.
    Plain,
    /// A keyword after which an identifier is a declared name.
    Declares,
    /// `macro_rules`, after whose `!` an identifier is a declared name.
    MacroRules,
}

/// A token as the run scanner remembers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tok {
    /// An identifier, whether it is the last segment of the open run, and
    /// what it is to the scanner.
    Ident { in_run: bool, word: Word },
    /// A punctuation character and whether the next character joins it.
    Punct { ch: char, joint: bool },
    /// A literal or a delimited group.
    Other,
}

/// The run scanner of one delimiter level: the last three tokens.
#[derive(Debug, Default)]
struct Level {
    last: [Option<Tok>; 3],
}

impl Level {
    fn push(&mut self, tok: Tok) {
        self.last = [self.last[1], self.last[2], Some(tok)];
    }

    /// True when the last two tokens are `::`.
    fn colons(&self) -> bool {
        matches!(
            (self.last[1], self.last[2]),
            (
                Some(Tok::Punct {
                    ch: ':',
                    joint: true
                }),
                Some(Tok::Punct { ch: ':', .. })
            )
        )
    }

    /// True when the last token is a `.` that is not part of `..`.
    fn single_dot(&self) -> bool {
        matches!(
            self.last[2],
            Some(Tok::Punct {
                ch: '.',
                joint: false
            })
        ) && !matches!(
            self.last[1],
            Some(Tok::Punct {
                ch: '.',
                joint: true
            })
        )
    }

    /// True when the next identifier is a name an item declares: it follows
    /// a declaring keyword, or `macro_rules!`.
    fn declaring(&self) -> bool {
        matches!(
            (self.last[1], self.last[2]),
            (
                _,
                Some(Tok::Ident {
                    word: Word::Declares,
                    ..
                })
            ) | (
                Some(Tok::Ident {
                    word: Word::MacroRules,
                    ..
                }),
                Some(Tok::Punct { ch: '!', .. })
            )
        )
    }

    /// What `word`, the next identifier, is to the scanner: a keyword after
    /// a `'` is a lifetime, as `'static`.
    fn word(&self, word: &str) -> Word {
        if matches!(self.last[2], Some(Tok::Punct { ch: '\'', .. })) {
            Word::Plain
        } else if DECLARES.contains(&word) {
            Word::Declares
        } else if word == "macro_rules" {
            Word::MacroRules
        } else {
            Word::Plain
        }
    }
}

/// Where a scanned identifier goes.
enum Place {
    /// It goes on the open run.
    Extend,
    /// It starts a run, with whether a `::` opens it.
    Start(bool),
    /// It is no segment of a run.
    Skip,
}

/// Decide where the identifier `word` goes after the tokens of `level`, the
/// open run being `open` (its segments) when there is one. A raw identifier
/// (`raw`) is never a keyword.
fn place(level: &Level, word: &str, raw: bool, open: Option<&[String]>) -> Place {
    let blocked = |word: &str| !raw && blocked(word);
    let path_word = !raw && matches!(word, "crate" | "self" | "super");
    if level.colons() {
        return match level.last[0] {
            Some(Tok::Ident { in_run: true, .. }) => {
                let supers = open.is_some_and(|segs| {
                    segs.iter()
                        .all(|seg| matches!(seg.as_str(), "self" | "super"))
                });
                if blocked(word) || (path_word && !(word == "super" && supers)) {
                    Place::Skip
                } else {
                    Place::Extend
                }
            }
            Some(Tok::Ident { in_run: false, .. } | Tok::Punct { ch: '>', .. }) => Place::Skip,
            _ if blocked(word) || path_word => Place::Skip,
            _ => Place::Start(true),
        };
    }
    if blocked(word) || level.single_dot() || level.declaring() {
        return Place::Skip;
    }
    match level.last[2] {
        Some(Tok::Punct { ch: '\'', .. }) => Place::Skip,
        Some(Tok::Punct { ch: '$', .. }) if word != "crate" => Place::Skip,
        _ => Place::Start(false),
    }
}

/// The runs of `stream`, whose byte offsets are those of its text plus
/// `base`, in text order, and the tokens read. The levels of delimiters are
/// walked on an explicit stack.
///
/// A run of one segment directly followed by a single `:` is a name being
/// bound or a field, not a path (`helper: u8`, `S { helper: 1 }`), and is
/// dropped; a `::` after it goes on with it.
fn scan(stream: TokenStream, base: usize) -> Lexed {
    let mut lexed = Lexed::default();
    let mut stack: Vec<(proc_macro2::token_stream::IntoIter, Level)> =
        vec![(stream.into_iter(), Level::default())];
    let mut open: Option<usize> = None;
    // The run of one segment that the `:` just read follows, if any, and
    // whether that `:` joins the next character.
    let mut colon: Option<(usize, bool)> = None;
    while let Some((tokens, level)) = stack.last_mut() {
        let tree = tokens.next();
        let next_colon = matches!(&tree, Some(TokenTree::Punct(punct)) if punct.as_char() == ':');
        if let Some((at, joint)) = colon.take()
            && !(joint && next_colon)
            && lexed.runs.len() == at + 1
        {
            lexed.runs.pop();
            open = None;
        }
        let Some(tree) = tree else {
            stack.pop();
            open = None;
            if let Some((_, outer)) = stack.last_mut() {
                outer.push(Tok::Other);
            }
            continue;
        };
        lexed.tokens += 1;
        match tree {
            TokenTree::Group(group) => {
                open = None;
                stack.push((group.stream().into_iter(), Level::default()));
            }
            TokenTree::Ident(ident) => open = scan_ident(&mut lexed, level, open, &ident, base),
            TokenTree::Punct(punct) => {
                let ch = punct.as_char();
                let joint = punct.spacing() == Spacing::Joint;
                if ch == ':' {
                    // The first `:` of a pair, or a single one: a run of one
                    // segment before it is dropped unless a second follows.
                    let first = !matches!(
                        level.last[2],
                        Some(Tok::Punct {
                            ch: ':',
                            joint: true
                        })
                    );
                    if first
                        && let (Some(at), Some(Tok::Ident { in_run: true, .. })) =
                            (open, level.last[2])
                        && lexed.runs[at].segs.len() == 1
                    {
                        colon = Some((at, joint));
                    }
                } else {
                    // A run stays open only across the `::` that may go on
                    // with it.
                    open = None;
                }
                level.push(Tok::Punct { ch, joint });
            }
            TokenTree::Literal(_) => {
                open = None;
                level.push(Tok::Other);
            }
        }
    }
    lexed
}

/// Scan the identifier `ident` after the tokens of `level`, with the open
/// run `open`, and return the open run after it.
fn scan_ident(
    lexed: &mut Lexed,
    level: &mut Level,
    open: Option<usize>,
    ident: &proc_macro2::Ident,
    base: usize,
) -> Option<usize> {
    let raw = ident.to_string();
    // A raw identifier is never a keyword.
    let (word, plain) = match raw.strip_prefix("r#") {
        Some(word) => (word.to_owned(), true),
        None => (raw.clone(), false),
    };
    let kind = if plain {
        Word::Plain
    } else {
        level.word(&word)
    };
    let segs = open.map(|at| lexed.runs[at].segs.as_slice());
    let (in_run, open) = match place(level, &word, plain, segs) {
        Place::Extend => {
            if let Some(run) = open.map(|at| &mut lexed.runs[at]) {
                run.text.push_str("::");
                run.text.push_str(&word);
                run.segs.push(word);
            }
            (true, open)
        }
        Place::Start(leading_colon) => {
            let text = if leading_colon {
                format!("::{word}")
            } else {
                word.clone()
            };
            lexed.runs.push(Run {
                start: base + ident.span().byte_range().start,
                leading_colon,
                segs: vec![word],
                text,
            });
            (true, Some(lexed.runs.len() - 1))
        }
        Place::Skip => (false, None),
    };
    level.push(Tok::Ident { in_run, word: kind });
    open
}

/// Lex `text` and scan its runs. A byte-order mark and a first line that is a
/// shebang, as `syn` reads them, are left out, and offsets stay those of
/// `text`.
fn lex_text(text: &str) -> Result<Lexed, LexFailure> {
    let mut base = 0;
    let mut body = text;
    if let Some(rest) = body.strip_prefix('\u{feff}') {
        base += '\u{feff}'.len_utf8();
        body = rest;
    }
    if let Some(after) = body.strip_prefix("#!")
        && !after.trim_start().starts_with('[')
    {
        let cut = body.find('\n').unwrap_or(body.len());
        base += cut;
        body = &body[cut..];
    }
    let stream = TokenStream::from_str(body).map_err(|error| {
        let start = error.span().start();
        LexFailure {
            message: error.to_string(),
            line: start.line + text[..base].matches('\n').count(),
            column: start.column + 1,
        }
    })?;
    Ok(scan(stream, base))
}

/// Lex the text of `file` on a thread of its own, whose span data is dropped
/// with it, with the stack of the parse thread.
fn lex(file: &RelPath, text: &str) -> Result<Lexed, ReachError> {
    thread::scope(|scope| {
        let worker = thread::Builder::new()
            .name(file.to_string())
            .stack_size(PARSE_STACK_BYTES)
            .spawn_scoped(scope, move || lex_text(text))
            .map_err(|source| {
                ReachError::Source(LoadError::new(file, None, LoadErrorKind::Spawn(source)))
            })?;
        match worker.join() {
            Ok(Ok(lexed)) => Ok(lexed),
            Ok(Err(failure)) => Err(ReachError::Source(LoadError::new(
                file,
                Some(failure.line),
                LoadErrorKind::Syntax {
                    message: failure.message,
                    column: failure.column,
                },
            ))),
            Err(payload) => panic::resume_unwind(payload),
        }
    })
}

/// The runs of one file, each given to the innermost unit whose span holds
/// its start: an item or an impl block, by its span.
#[derive(Debug, Default)]
struct FileRuns {
    runs: Vec<Run>,
    by_unit: BTreeMap<(usize, usize), Vec<usize>>,
}

/// Give each run, in text order, to the innermost of `units` (spans as
/// `(start, end)`, nested or apart) that holds its start.
fn place_runs(
    runs: &[Run],
    mut units: Vec<(usize, usize)>,
) -> BTreeMap<(usize, usize), Vec<usize>> {
    // Outer units first where two start at one byte.
    units.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
    units.dedup();
    let mut by_unit: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    let mut open: Vec<(usize, usize)> = Vec::new();
    let mut next = 0;
    for (position, run) in runs.iter().enumerate() {
        while let Some(&unit) = units.get(next).filter(|unit| unit.0 <= run.start) {
            work::add(Work::ReachUnits, 1);
            while open.last().is_some_and(|top| top.1 <= unit.0) {
                work::add(Work::ReachUnits, 1);
                open.pop();
            }
            open.push(unit);
            next += 1;
        }
        while open.last().is_some_and(|top| top.1 <= run.start) {
            work::add(Work::ReachUnits, 1);
            open.pop();
        }
        if let Some(unit) = open.last() {
            work::add(Work::ReachUnits, 1);
            by_unit.entry(*unit).or_default().push(position);
        }
    }
    by_unit
}

/// What the runs read from one module resolved to, by text.
type Resolved = BTreeMap<String, Rc<[ItemId]>>;

/// The walk: the index, the files, the tables built for it and what it has
/// found.
struct Walk<'a> {
    index: &'a Index,
    files: &'a BTreeMap<RelPath, SourceFile>,
    /// The module each item is read from: its own module, the module of its
    /// impl block for a member of one, and the module of its trait for a
    /// member of a trait.
    homes: Vec<Option<usize>>,
    /// The impl blocks of each struct, enum and union.
    blocks_of_type: BTreeMap<ItemId, Vec<ImplId>>,
    /// The impl blocks each module holds.
    blocks_of_module: BTreeMap<usize, Vec<ImplId>>,
    /// The crate root items.
    roots: BTreeSet<ItemId>,
    /// The runs of each file lexed so far.
    files_runs: BTreeMap<RelPath, FileRuns>,
    /// What each run resolved to, by module and test flag, then by text.
    resolved: BTreeMap<(usize, bool), Resolved>,
}

impl<'a> Walk<'a> {
    fn new(index: &'a Index, files: &'a BTreeMap<RelPath, SourceFile>) -> Self {
        let mut homes = vec![None; index.items.len()];
        for (module, entry) in index.modules.iter().enumerate() {
            for id in &entry.defs {
                work::add(Work::ReachSteps, 1);
                homes[id.0] = Some(module);
            }
        }
        for (module, id) in &index.root_items {
            work::add(Work::ReachSteps, 1);
            homes[id.0] = Some(*module);
        }
        let mut blocks_of_type: BTreeMap<ItemId, Vec<ImplId>> = BTreeMap::new();
        let mut blocks_of_module: BTreeMap<usize, Vec<ImplId>> = BTreeMap::new();
        for block in &index.impl_blocks {
            work::add(Work::ReachSteps, 1);
            blocks_of_module
                .entry(block.module_index)
                .or_default()
                .push(block.id);
            for ty in &block.self_items {
                work::add(Work::ReachSteps, 1);
                blocks_of_type.entry(*ty).or_default().push(block.id);
            }
            for member in &block.members {
                work::add(Work::ReachSteps, 1);
                homes[member.0] = Some(block.module_index);
            }
        }
        for (owner, members) in &index.assoc {
            if index.items[owner.0].kind != SymbolKind::Trait {
                continue;
            }
            for id in members.values().flatten() {
                work::add(Work::ReachSteps, 1);
                homes[id.0] = homes[owner.0];
            }
        }
        Self {
            index,
            files,
            homes,
            blocks_of_type,
            blocks_of_module,
            roots: index.root_items.values().copied().collect(),
            files_runs: BTreeMap::new(),
            resolved: BTreeMap::new(),
        }
    }

    /// Lex and place the runs of `file` the first time they are asked.
    fn load(&mut self, file: &RelPath) -> Result<(), ReachError> {
        if self.files_runs.contains_key(file) {
            return Ok(());
        }
        let text = self
            .files
            .get(file)
            .map_or("", |source| source.text.as_str());
        let lexed = lex(file, text)?;
        work::add(Work::ReachTokens, lexed.tokens);
        let mut units: Vec<(usize, usize)> = self
            .index
            .by_file
            .get(file)
            .into_iter()
            .flatten()
            .filter(|id| !self.roots.contains(id))
            .map(|id| {
                let span = &self.index.items[id.0].span;
                (span.start_byte, span.end_byte)
            })
            .collect();
        let first = self
            .index
            .impl_blocks
            .partition_point(|block| block.file < *file);
        units.extend(
            self.index.impl_blocks[first..]
                .iter()
                .take_while(|block| block.file == *file)
                .map(|block| (block.span.start_byte, block.span.end_byte)),
        );
        let by_unit = place_runs(&lexed.runs, units);
        self.files_runs.insert(
            file.clone(),
            FileRuns {
                runs: lexed.runs,
                by_unit,
            },
        );
        Ok(())
    }

    /// The items the own tokens of a unit name: the runs of `file` given to
    /// the unit at `span` that start before `before`, read from `module` as
    /// code, test code when `test`.
    fn references(
        &mut self,
        file: &RelPath,
        span: (usize, usize),
        before: usize,
        module: usize,
        test: bool,
    ) -> Result<Vec<ItemId>, ReachError> {
        self.load(file)?;
        let index = self.index;
        let runs = &self.files_runs[file];
        let resolved = self.resolved.entry((module, test)).or_default();
        let mut out = Vec::new();
        for position in runs.by_unit.get(&span).into_iter().flatten() {
            let run = &runs.runs[*position];
            if run.start >= before {
                break;
            }
            work::add(Work::ReachSteps, 1);
            let sites = if let Some(sites) = resolved.get(run.text.as_str()) {
                Rc::clone(sites)
            } else {
                work::add(Work::ReachPaths, 1);
                let sites: Rc<[ItemId]> = index
                    .reach_run(module, &run.segs, run.leading_colon, test)
                    .into_iter()
                    .collect();
                resolved.insert(run.text.clone(), Rc::clone(&sites));
                sites
            };
            out.extend(sites.iter().copied());
        }
        Ok(out)
    }

    /// The nodes `node`, reached by `edge`, leads to, with their edges, and
    /// whether it opens its module (a module reached from a root by contains
    /// alone). `first` is true the first time the node is reached; a module
    /// reached again by contains is opened then.
    fn next(
        &mut self,
        node: Node,
        edge: Edge,
        first: bool,
        opened: &mut BTreeSet<ItemId>,
    ) -> Result<Vec<(Node, Edge)>, ReachError> {
        let index = self.index;
        let mut out = Vec::new();
        match node {
            Node::Item(id) => {
                let item = &index.items[id.0];
                if self.roots.contains(&id) {
                    return Ok(out);
                }
                match item.kind {
                    SymbolKind::Mod => {
                        if matches!(edge, Edge::Root | Edge::Contains)
                            && opened.insert(id)
                            && let Some(module) = index.module_of.get(&id)
                        {
                            for child in &index.modules[*module].defs {
                                work::add(Work::ReachSteps, 1);
                                out.push((Node::Item(*child), Edge::Contains));
                            }
                            for block in self.blocks_of_module.get(module).into_iter().flatten() {
                                work::add(Work::ReachSteps, 1);
                                out.push((Node::Impl(*block), Edge::Contains));
                            }
                        }
                    }
                    SymbolKind::Struct
                    | SymbolKind::Enum
                    | SymbolKind::Union
                    | SymbolKind::Trait
                        if first =>
                    {
                        for member in index.assoc.get(&id).into_iter().flat_map(|m| m.values()) {
                            for child in member {
                                work::add(Work::ReachSteps, 1);
                                out.push((Node::Item(*child), Edge::Contains));
                            }
                        }
                        for block in self.blocks_of_type.get(&id).into_iter().flatten() {
                            work::add(Work::ReachSteps, 1);
                            out.push((Node::Impl(*block), Edge::ImplOf));
                        }
                    }
                    _ => {}
                }
                if first && let Some(module) = self.homes[id.0] {
                    // A module's own tokens end where its body starts.
                    let before = match (item.kind, &item.body) {
                        (SymbolKind::Mod, Some(body)) => body.start_byte,
                        _ => usize::MAX,
                    };
                    let span = (item.span.start_byte, item.span.end_byte);
                    let test = item.test != TestRole::None;
                    for found in self.references(&item.file, span, before, module, test)? {
                        work::add(Work::ReachSteps, 1);
                        out.push((Node::Item(found), Edge::References));
                    }
                }
            }
            Node::Impl(id) => {
                if first {
                    let block = &index.impl_blocks[id.0];
                    for member in &block.members {
                        work::add(Work::ReachSteps, 1);
                        out.push((Node::Item(*member), Edge::ImplOf));
                    }
                    let span = (block.span.start_byte, block.span.end_byte);
                    let test = block.test != TestRole::None;
                    for found in
                        self.references(&block.file, span, usize::MAX, block.module_index, test)?
                    {
                        work::add(Work::ReachSteps, 1);
                        out.push((Node::Item(found), Edge::References));
                    }
                }
            }
        }
        Ok(out)
    }
}

/// The nodes that `roots` reach in `index`, whose files are `files`.
pub(crate) fn reachable(
    index: &Index,
    files: &BTreeMap<RelPath, SourceFile>,
    roots: &[Node],
) -> Result<Reachability, ReachError> {
    let mut sorted: Vec<Node> = Vec::with_capacity(roots.len());
    for root in roots {
        work::add(Work::ReachSteps, 1);
        match *root {
            Node::Item(id) if id.0 >= index.items.len() => return Err(ReachError::UnknownItem(id)),
            Node::Impl(id) if id.0 >= index.impl_blocks.len() => {
                return Err(ReachError::UnknownImpl(id));
            }
            node => sorted.push(node),
        }
    }
    sorted.sort_unstable();
    sorted.dedup();
    let mut walk = Walk::new(index, files);
    let mut queue: VecDeque<(Node, Edge, Option<Node>)> = sorted
        .into_iter()
        .map(|node| (node, Edge::Root, None))
        .collect();
    let mut reached: Vec<Reached> = Vec::new();
    let mut at: BTreeMap<Node, usize> = BTreeMap::new();
    let mut opened: BTreeSet<ItemId> = BTreeSet::new();
    while let Some((node, edge, from)) = queue.pop_front() {
        work::add(Work::ReachSteps, 1);
        let first = !at.contains_key(&node);
        if first {
            at.insert(node, reached.len());
            reached.push(Reached { node, edge, from });
        }
        for (next, by) in walk.next(node, edge, first, &mut opened)? {
            let opens = matches!(next, Node::Item(id) if index.items[id.0].kind == SymbolKind::Mod && by == Edge::Contains && !opened.contains(&id));
            if opens || !at.contains_key(&next) {
                queue.push_back((next, by, Some(node)));
            }
        }
    }
    Ok(Reachability { reached, at })
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::{Run, lex_text, place_runs};
    use crate::item::Namespace;
    use crate::reach::Node;
    use crate::work::{self, Work};
    use crate::{MemoryTree, Workspace};

    /// The runs of `text` as their text and the byte where each starts.
    fn runs(text: &str) -> Vec<(String, usize)> {
        let lexed = lex_text(text).unwrap_or_else(|failure| panic!("{}", failure.message));
        lexed
            .runs
            .into_iter()
            .map(|run| (run.text, run.start))
            .collect()
    }

    /// The texts of the runs of `text`.
    fn texts(text: &str) -> Vec<String> {
        runs(text).into_iter().map(|(text, _)| text).collect()
    }

    #[test]
    fn a_run_is_an_identifier_path_that_no_dot_precedes_with_its_first_byte() {
        assert_eq!(runs("a::b::c"), [("a::b::c".to_owned(), 0)]);
        assert_eq!(texts("x.helper(); x.0.y"), ["x", "x"]);
        assert_eq!(texts("0..LIMIT; ..=END"), ["LIMIT", "END"]);
        assert_eq!(texts("::std::fmt::Debug"), ["::std::fmt::Debug"]);
        assert_eq!(texts("Vec::<u8>::new()"), ["Vec", "u8"]);
        assert_eq!(texts("<T as Tr>::f"), ["T", "Tr"]);
        assert_eq!(texts("r#match::x"), ["match::x"]);
        assert_eq!(texts("a :: b"), ["a::b"]);
        assert_eq!(texts("f(a::b, c)"), ["f", "a::b", "c"]);
    }

    #[test]
    fn path_keywords_start_a_run_and_other_keywords_are_no_segment() {
        assert_eq!(
            texts("crate::a; self::b; super::super::c; self::super::d"),
            ["crate::a", "self::b", "super::super::c", "self::super::d"]
        );
        // A keyword ends the run, and what goes on after it is no run.
        assert_eq!(texts("a::crate; a::self::b"), ["a", "a"]);
        assert_eq!(texts("r#fn::r#match"), ["fn::match"]);
        assert_eq!(texts("Self::new(); impl Tr for S"), ["Tr", "S"]);
        assert_eq!(texts("pub use a as b;"), ["a", "b"]);
    }

    #[test]
    fn a_lifetime_a_metavariable_a_declared_name_and_a_bound_name_are_no_run() {
        assert_eq!(texts("&'a T; 'label: loop {}"), ["T"]);
        assert_eq!(texts("&'static Foo"), ["Foo"]);
        assert_eq!(texts("$x; $crate::helper()"), ["crate::helper"]);
        assert_eq!(texts("fn helper(x: u8) -> Foo {}"), ["u8", "Foo"]);
        assert_eq!(
            texts(
                "struct S { helper: u8 } enum E {} trait T {} type A = B; mod m {} const C: u8 = 1; static D: X = Y;"
            ),
            ["u8", "B", "u8", "X", "Y"]
        );
        assert_eq!(
            texts("macro_rules! m { () => { g() } }"),
            ["macro_rules", "g"]
        );
        assert_eq!(texts("S { helper: 1, other }"), ["S", "other"]);
        assert_eq!(texts("where T::Assoc: Bound"), ["T::Assoc", "Bound"]);
        assert_eq!(texts("x: ::std::Y"), ["::std::Y"]);
        assert_eq!(texts("x:&Y"), ["Y"]);
    }

    #[test]
    fn a_byte_order_mark_and_a_shebang_are_left_out_and_offsets_are_those_of_the_text() {
        let text = "\u{feff}fn f() { g() }";
        assert_eq!(runs(text), [("g".to_owned(), text.find('g').expect("g"))]);
        let text = "#!/usr/bin/env run\nfn f() { g() }";
        assert_eq!(runs(text), [("g".to_owned(), text.find("g(").expect("g"))]);
        // An inner attribute is no shebang.
        assert_eq!(texts("#![allow(unused)]\nfn f() {}"), ["allow", "unused"]);
        // A text that does not lex says where.
        let failure = lex_text("fn f() {\n\"").expect_err("an open string");
        assert_eq!(failure.line, 2);
    }

    fn run_at(start: usize) -> Run {
        Run {
            start,
            leading_colon: false,
            segs: vec!["x".to_owned()],
            text: "x".to_owned(),
        }
    }

    #[test]
    fn each_run_goes_to_the_innermost_unit_that_holds_its_start() {
        // An impl block (0..100) holding two members, beside an item after
        // it, and a run outside every unit.
        let units = vec![(120, 150), (10, 20), (0, 100), (30, 40), (0, 100)];
        let runs: Vec<Run> = [5, 10, 25, 35, 99, 100, 130, 160]
            .into_iter()
            .map(run_at)
            .collect();
        let placed = place_runs(&runs, units);
        let by: Vec<((usize, usize), Vec<usize>)> = placed.into_iter().collect();
        assert_eq!(
            by,
            [
                ((0, 100), vec![0, 2, 4]),
                ((10, 20), vec![1]),
                ((30, 40), vec![3]),
                ((120, 150), vec![6]),
            ]
        );
    }

    /// A library of `items` public functions, each calling its own private
    /// helper `references` times.
    fn fan(items: usize, references: usize) -> Workspace {
        let mut text = String::new();
        for i in 0..items {
            let calls = format!("h{i}(); ").repeat(references);
            writeln!(text, "pub fn f{i}() {{ {calls}}}\nfn h{i}() {{}}").expect("write");
        }
        let manifest = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
        let tree = MemoryTree::from([("Cargo.toml", manifest), ("src/lib.rs", text.as_str())]);
        Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"))
    }

    /// The kinds of work the walk counts.
    const KINDS: [Work; 6] = [
        Work::ReachTokens,
        Work::ReachUnits,
        Work::ReachPaths,
        Work::ReachSteps,
        Work::SegmentReads,
        Work::Lookups,
    ];

    /// The work of reaching every public function of `fan(items,
    /// references)`, by kind, and how many nodes were reached.
    fn work_of(items: usize, references: usize) -> ([usize; 6], usize) {
        let ws = fan(items, references);
        let roots: Vec<Node> = (0..items)
            .map(|i| {
                let path = format!("demo::f{i}");
                Node::Item(ws.resolve_in(&path, Namespace::Value).expect("f").sites[0])
            })
            .collect();
        let (reached, tally) = work::measure(usize::MAX, || ws.reachable(&roots));
        let reached = reached.unwrap_or_else(|e| panic!("{e}")).reached.len();
        (KINDS.map(|kind| tally.get(kind)), reached)
    }

    #[test]
    fn the_work_of_a_walk_is_counted_exactly() {
        for (n, r) in [(1, 1), (3, 2), (5, 4)] {
            let ([tokens, units, paths, steps, ..], reached) = work_of(n, r);
            assert_eq!(reached, 2 * n, "{n} {r}");
            // Each function is `pub fn f() { h(); ... }`, five tokens and
            // three for each call, and each helper `fn h() {}`, four.
            assert_eq!(tokens, n * (9 + 3 * r), "{n} {r}");
            // One push of the first unit, then for each further function
            // the helper before it and the function opened and closed, and
            // each run placed once.
            assert_eq!(units, n * r + 4 * n - 3, "{n} {r}");
            // Each helper's name is resolved once in its module, however
            // often it is called; a declared name is no path.
            assert_eq!(paths, n, "{n} {r}");
            // The tables (each item of the module and the crate root), the
            // roots, each node taken from the queue (a root, and a helper
            // once for each call), and each run and site of a function.
            assert_eq!(steps, 4 * n + 1 + 3 * n * r, "{n} {r}");
        }
    }

    #[test]
    fn the_work_grows_with_the_items_and_the_references_and_not_their_product() {
        // Three sizes in items at a fixed number of references, then in
        // references at a fixed number of items: each step adds the same
        // work, so no kind grows with a product of the two.
        let by_items: Vec<[usize; 6]> = [10, 20, 30].map(|n| work_of(n, 3).0).to_vec();
        let by_references: Vec<[usize; 6]> = [2, 4, 6].map(|r| work_of(10, r).0).to_vec();
        for sizes in [&by_items, &by_references] {
            for (kind, name) in KINDS.iter().enumerate() {
                let first = sizes[1][kind] - sizes[0][kind];
                let second = sizes[2][kind] - sizes[1][kind];
                assert_eq!(first, second, "{name:?}: {sizes:?}");
            }
        }
        // The references add no resolution: a run is resolved once in its
        // module.
        assert_eq!(by_references[0][2], by_references[2][2]);
    }

    #[test]
    fn a_path_of_fifty_thousand_segments_is_walked_once_and_not_once_for_each_prefix() {
        const SEGMENTS: usize = 50_000;
        let text = format!(
            "pub mod m {{ pub use crate::m; pub struct End; }}\npub type T = {}End;\n",
            "m::".repeat(SEGMENTS)
        );
        let manifest = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
        let tree = MemoryTree::from([("Cargo.toml", manifest), ("src/lib.rs", text.as_str())]);
        let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"));
        let alias = ws.resolve("demo::T").expect("the alias").sites[0];
        let end = ws.resolve("demo::m::End").expect("the struct").sites[0];
        // Each proper prefix is one step of one walk and the last segment is
        // read in each name space, so a segment is read a few times; a walk
        // from the start for each prefix would read it once for each prefix
        // after it, and passes the ceiling at once.
        let segments = SEGMENTS + 1;
        let (reached, tally) = work::measure_kind(Work::SegmentReads, 4 * segments, || {
            ws.reachable(&[Node::Item(alias)])
        });
        let reached = reached.unwrap_or_else(|e| panic!("{e}"));
        assert!(reached.contains(Node::Item(end)));
        assert!(tally.get(Work::SegmentReads) >= segments);
        assert!(tally.get(Work::Lookups) <= 4 * segments);
        assert_eq!(tally.get(Work::ReachPaths), 1);
    }
}
