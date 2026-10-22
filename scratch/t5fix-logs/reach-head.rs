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
//!   tokens names. The paths are the `Ident (:: Ident)*` runs that no `.`
//!   precedes, read as code of its module reads it: each proper prefix in the
//!   type name space and the whole run in each name space. A run that `Self`
//!   starts, and a method called on `self`, are read in the impl block or the
//!   trait that holds the unit: they name its members of that name.
//!
//! The tokens of an item are those of its span outside the spans of the items
//! and blocks below it: a type's members and a block's members are their own
//! units. A module's own tokens are its declaration before its body; what a
//! module body holds outside its items and blocks (its `use` declarations and
//! the macro invocations at module level) is no item's, and a crate root has
//! none. Each file is lexed once, the first time an item of it is reached, and
//! each run is resolved, and what it names given to the walk, at most once
//! for each place it is read in: a module with a test flag, or an impl block
//! or a trait. What the walk holds is charged to the item-bytes budget of the
//! load.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::mem::size_of;
use std::panic;
use std::str::FromStr;
use std::thread;

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};

use crate::cost::{btree_entry, btree_map, vec_first, vec_slot};
use crate::error::{LoadError, LoadErrorKind};
use crate::facts::TestRole;
use crate::index::Index;
use crate::item::{ImplId, ItemId, SharedStr, SymbolKind};
use crate::meter::Meter;
use crate::options::LoadOptions;
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
    /// A root is past the items of this workspace. An [`ItemId`] is the
    /// place of an item in the workspace that gave it, so an id of another
    /// workspace that is not past the items of this one is read as the item
    /// of this one at that place.
    UnknownItem(ItemId),
    /// A root is past the impl blocks of this workspace; as for
    /// [`ReachError::UnknownItem`], an [`ImplId`] is a place.
    UnknownImpl(ImplId),
    /// The text of a file of a reached item could not be read for its paths:
    /// the thread that lexes it could not start
    /// ([`LoadErrorKind::Spawn`]), or the text does not lex
    /// ([`LoadErrorKind::Syntax`]), which a file the load parsed does not do.
    Source(LoadError),
    /// What the walk holds passed the item-bytes budget of the load
    /// ([`Budget::ItemBytes`](crate::Budget::ItemBytes)), counted on from
    /// what the load holds: a `Budget` load error naming the budget, its
    /// limit and the file of the runs or the node being held.
    Budget(LoadError),
}

impl fmt::Display for ReachError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownItem(id) => {
                write!(f, "item {} is past the items of the workspace", id.index())
            }
            Self::UnknownImpl(id) => {
                write!(
                    f,
                    "impl block {} is past the impl blocks of the workspace",
                    id.index()
                )
            }
            Self::Source(error) => write!(f, "{error}"),
            Self::Budget(error) => write!(f, "the walk is over budget: {error}"),
        }
    }
}

impl Error for ReachError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Source(error) | Self::Budget(error) => Some(error),
            Self::UnknownItem(_) | Self::UnknownImpl(_) => None,
        }
    }
}

/// How a run is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunKind {
    /// A path, read as code of the unit's module reads it.
    Path,
    /// A path that `Self` starts, read in the impl block or trait that holds
    /// the unit.
    SelfPath,
    /// The name of a method called on `self` (`self.name(` or
    /// `self.name::<`), read as `Self::name` is.
    Method,
}

/// A run of a file: the byte offset of its first identifier, whether a `::`
/// opens it, how it is read, and its segments: `len` byte ranges of the
/// file's text from `first` in [`Lexed::segs`], each without a
/// raw-identifier prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Run {
    start: usize,
    first: usize,
    len: usize,
    leading_colon: bool,
    kind: RunKind,
}

/// What lexing one file gave: its runs in text order, their segments and the
/// tokens read.
#[derive(Debug, Default)]
struct Lexed {
    runs: Vec<Run>,
    segs: Vec<(usize, usize)>,
    tokens: usize,
}

impl Lexed {
    /// Start a run of one segment and return its place.
    fn start(&mut self, start: usize, leading_colon: bool, kind: RunKind, seg: (usize, usize)) -> usize {
        self.runs.push(Run {
            start,
            first: self.segs.len(),
            len: 1,
            leading_colon,
            kind,
        });
        self.segs.push(seg);
        self.runs.len() - 1
    }

    /// Add a segment to the run at `at`, the last one.
    fn extend(&mut self, at: usize, seg: (usize, usize)) {
        self.runs[at].len += 1;
        self.segs.push(seg);
    }

    /// Drop the last run and its segments.
    fn drop_last(&mut self) {
        if let Some(run) = self.runs.pop() {
            self.segs.truncate(run.first);
        }
    }
}

/// A lexing failure as plain data, read on the thread that lexed.
struct LexFailure {
    message: String,
    line: usize,
    column: usize,
}

/// The words that are no segment of a path in every edition, beside `crate`,
/// `self` and `super`, which start one, and `Self`, which starts one read in
/// its impl block or trait: the strict and reserved keywords of edition 2015.
const KEYWORDS: [&str; 43] = [
    "abstract", "as", "become", "box", "break", "const", "continue", "do", "else", "enum",
    "extern", "false", "final", "fn", "for", "if", "impl", "in", "let", "loop", "macro", "match",
    "mod", "move", "mut", "override", "priv", "pub", "ref", "return", "static", "struct", "trait",
    "true", "type", "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// The keywords that edition 2018 adds.
const KEYWORDS_2018: [&str; 4] = ["async", "await", "dyn", "try"];

/// The keyword that edition 2024 adds.
const KEYWORD_2024: &str = "gen";

/// True when `word` cannot be a segment of a path at all in `edition`, or
/// goes on no run (`Self`).
fn blocked(word: &str, edition: u16) -> bool {
    word == "Self"
        || KEYWORDS.contains(&word)
        || (edition >= 2018 && KEYWORDS_2018.contains(&word))
        || (edition >= 2024 && word == KEYWORD_2024)
}

/// The words after which an identifier is the name an item declares, not a
/// path: `fn helper` declares `helper`. `const` is not one of them: the name
/// of a constant is a run of one segment before a single `:`, and after
/// `*const` and `&raw const` comes a path.
const DECLARES: [&str; 8] = [
    "fn", "struct", "enum", "union", "trait", "type", "mod", "static",
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
    /// `self`, after whose `.` a called name is a method of `Self`.
    SelfValue,
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

/// The run scanner of one delimiter level: the last three tokens, and
/// whether a `where` clause is open, from its keyword to the `;` or the
/// braced group that ends it.
#[derive(Debug, Default)]
struct Level {
    last: [Option<Tok>; 3],
    in_where: bool,
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

    /// True when the next identifier is a name called after `self.`, if a
    /// `(` or a `::` follows it.
    fn after_self_dot(&self) -> bool {
        self.single_dot()
            && matches!(
                self.last[1],
                Some(Tok::Ident {
                    word: Word::SelfValue,
                    ..
                })
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
        } else if word == "self" {
            Word::SelfValue
        } else {
            Word::Plain
        }
    }
}

/// Where a scanned identifier goes.
enum Place {
    /// It goes on the open run.
    Extend,
    /// It starts a run, with whether a `::` opens it and how it is read.
    Start(bool, RunKind),
    /// It is no segment of a run.
    Skip,
}

/// The open run: its place, and whether each of its segments is `self` or
/// `super`.
#[derive(Debug, Clone, Copy)]
struct Open {
    at: usize,
    supers: bool,
}

/// Decide where the identifier `word` goes after the tokens of `level` in
/// `edition`, `supers` telling whether the open run, when there is one, is
/// made of `self` and `super` alone. A raw identifier (`raw`) is never a
/// keyword.
fn place(level: &Level, word: &str, raw: bool, supers: bool, edition: u16) -> Place {
    let blocked = |word: &str| !raw && blocked(word, edition);
    let path_word = !raw && matches!(word, "crate" | "self" | "super");
    if level.colons() {
        return match level.last[0] {
            Some(Tok::Ident { in_run: true, .. }) => {
                if blocked(word) || (path_word && !(word == "super" && supers)) {
                    Place::Skip
                } else {
                    Place::Extend
                }
            }
            Some(Tok::Ident { in_run: false, .. } | Tok::Punct { ch: '>', .. }) => Place::Skip,
            _ if blocked(word) || path_word => Place::Skip,
            _ => Place::Start(true, RunKind::Path),
        };
    }
    let self_type = !raw && word == "Self";
    if (blocked(word) && !self_type) || level.single_dot() || level.declaring() {
        return Place::Skip;
    }
    match level.last[2] {
        Some(Tok::Punct { ch: '\'', .. }) => Place::Skip,
        Some(Tok::Punct { ch: '$', .. }) if word != "crate" => Place::Skip,
        _ if self_type => Place::Start(false, RunKind::SelfPath),
        _ => Place::Start(false, RunKind::Path),
    }
}

/// The runs of `stream`, whose byte offsets are those of its text plus
/// `base`, in text order, and the tokens read, with the keywords of
/// `edition`. The levels of delimiters are walked on an explicit stack.
///
/// A run of one segment directly followed by a single `:` is a name being
/// bound or a field, not a path (`helper: u8`, `S { helper: 1 }`), and is
/// dropped, except in a `where` clause, where it is the type a bound is put
/// on (`where Helper: Clone`); a `::` after it goes on with it. A name after
/// `self.` that a `(` or a `::` follows is a method called on `self`.
fn scan(stream: TokenStream, base: usize, edition: u16) -> Lexed {
    let mut lexed = Lexed::default();
    let mut stack: Vec<(proc_macro2::token_stream::IntoIter, Level)> =
        vec![(stream.into_iter(), Level::default())];
    let mut open: Option<Open> = None;
    // The run of one segment that the `:` just read follows, if any, and
    // whether that `:` joins the next character.
    let mut colon: Option<(usize, bool)> = None;
    // The name after `self.` just read, where it starts and its segment,
    // until the next token tells whether it is called.
    let mut method: Option<(usize, (usize, usize))> = None;
    while let Some((tokens, level)) = stack.last_mut() {
        let tree = tokens.next();
        if let Some((start, seg)) = method.take() {
            let called = match &tree {
                Some(TokenTree::Group(group)) => group.delimiter() == Delimiter::Parenthesis,
                Some(TokenTree::Punct(punct)) => {
                    punct.as_char() == ':' && punct.spacing() == Spacing::Joint
                }
                _ => false,
            };
            if called {
                lexed.start(start, false, RunKind::Method, seg);
            }
        }
        let next_colon = matches!(&tree, Some(TokenTree::Punct(punct)) if punct.as_char() == ':');
        if let Some((at, joint)) = colon.take()
            && !(joint && next_colon)
            && lexed.runs.len() == at + 1
        {
            lexed.drop_last();
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
                if group.delimiter() == Delimiter::Brace {
                    level.in_where = false;
                }
                stack.push((group.stream().into_iter(), Level::default()));
            }
            TokenTree::Ident(ident) => {
                open = scan_ident(&mut lexed, level, open, &ident, base, edition, &mut method);
            }
            TokenTree::Punct(punct) => {
                let ch = punct.as_char();
                let joint = punct.spacing() == Spacing::Joint;
                if ch == ':' {
                    // The first `:` of a pair, or a single one: a run of one
                    // segment before it is dropped unless a second follows,
                    // or a `where` clause is open.
                    let first = !matches!(
                        level.last[2],
                        Some(Tok::Punct {
                            ch: ':',
                            joint: true
                        })
                    );
                    if first
                        && !level.in_where
                        && let (Some(run), Some(Tok::Ident { in_run: true, .. })) =
                            (open, level.last[2])
                        && lexed.runs[run.at].len == 1
                    {
                        colon = Some((run.at, joint));
                    }
                } else {
                    // A run stays open only across the `::` that may go on
                    // with it.
                    open = None;
                    if ch == ';' {
                        level.in_where = false;
                    }
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
/// run `open`, and return the open run after it. A name after `self.` is
/// kept in `method` until the next token.
fn scan_ident(
    lexed: &mut Lexed,
    level: &mut Level,
    open: Option<Open>,
    ident: &proc_macro2::Ident,
    base: usize,
    edition: u16,
    method: &mut Option<(usize, (usize, usize))>,
) -> Option<Open> {
    let text = ident.to_string();
    // A raw identifier is never a keyword.
    let (word, raw) = match text.strip_prefix("r#") {
        Some(word) => (word, true),
        None => (text.as_str(), false),
    };
    let kind = if raw { Word::Plain } else { level.word(word) };
    let range = ident.span().byte_range();
    let start = base + range.start;
    let seg = (base + range.end - word.len(), base + range.end);
    if !raw && word == "where" {
        level.in_where = true;
    }
    if level.after_self_dot() {
        *method = Some((start, seg));
    }
    let path_word = !raw && matches!(word, "self" | "super");
    let supers = open.is_some_and(|run| run.supers);
    let (in_run, open) = match place(level, word, raw, supers, edition) {
        Place::Extend => {
            let open = open.map(|run| {
                lexed.extend(run.at, seg);
                Open {
                    at: run.at,
                    supers: run.supers && path_word,
                }
            });
            (true, open)
        }
        Place::Start(leading_colon, kind) => {
            let at = lexed.start(start, leading_colon, kind, seg);
            (
                true,
                Some(Open {
                    at,
                    supers: path_word,
                }),
            )
        }
        Place::Skip => (false, None),
    };
    level.push(Tok::Ident { in_run, word: kind });
    open
}

/// Lex `text` with the keywords of `edition` and scan its runs. A byte-order
/// mark and a first line that is a shebang, as `syn` reads them, are left
/// out, and offsets stay those of `text`.
fn lex_text(text: &str, edition: u16) -> Result<Lexed, LexFailure> {
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
    let mut lexed = scan(stream, base, edition);
    lexed.runs.shrink_to_fit();
    lexed.segs.shrink_to_fit();
    Ok(lexed)
}

/// Lex the text of `file` on a thread of its own, whose span data is dropped
/// with it, with the stack of the parse thread.
fn lex(file: &RelPath, text: &str, edition: u16) -> Result<Lexed, ReachError> {
    thread::scope(|scope| {
        let worker = thread::Builder::new()
            .name(file.to_string())
            .stack_size(PARSE_STACK_BYTES)
            .spawn_scoped(scope, move || lex_text(text, edition))
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

/// The runs of one file and their segments, each run given to the innermost
/// unit whose span holds its start: an item or an impl block, by its span.
#[derive(Debug, Default)]
struct FileRuns {
    runs: Vec<Run>,
    segs: Vec<(usize, usize)>,
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

/// What `Self` stands for in a unit: the impl block or the trait that holds
/// it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Owner {
    /// A member of the impl block, or the block's own tokens.
    Block(ImplId),
    /// A member of the trait, or the trait's own tokens.
    Trait(ItemId),
}

/// Where the runs of a unit are read: as code of a module, test code when
/// the flag is set, or in the impl block or trait that `Self` stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Ctx {
    Module(usize, bool),
    Owner(Owner),
}

/// Where the runs of an item are read: the module of its paths (its own
/// module, the module of its impl block for a member of one, the module of
/// its trait for a member of a trait), and what `Self` stands for in it.
#[derive(Debug, Clone, Copy, Default)]
struct Home {
    module: Option<usize>,
    owner: Option<Owner>,
}

/// What the walk reads of an impl block for the paths that `Self` starts:
/// its members by name when no type keys it, and the traits of the walk
/// that its trait path names from its module.
#[derive(Debug, Default)]
struct BlockNames<'a> {
    members: BTreeMap<&'a str, Vec<ItemId>>,
    traits: Vec<ItemId>,
}

/// A unit whose own runs are read: the file and the line it is charged at,
/// its span, where its own tokens end, its module and test flag and its
/// owner.
struct Unit<'u> {
    file: &'u RelPath,
    line: usize,
    span: (usize, usize),
    before: usize,
    module: usize,
    test: bool,
    owner: Option<Owner>,
}

/// An entry of the queue of the walk: a node, the edge it is reached by and
/// the node that edge comes from.
type Queued = (Node, Edge, Option<Node>);

/// What a map of lists of impl blocks holds at most.
fn lists_bytes<K>(map: &BTreeMap<K, Vec<ImplId>>) -> usize {
    btree_map::<K, Vec<ImplId>>()
        + map
            .values()
            .map(|list| btree_entry::<K, Vec<ImplId>>() + list.capacity() * size_of::<ImplId>())
            .sum::<usize>()
}

/// The members called `name` of the type or trait `owner`.
fn members_named<'a>(index: &'a Index, owner: ItemId, name: &str) -> &'a [ItemId] {
    work::add(Work::ReachSteps, 1);
    index
        .assoc
        .get(&owner)
        .and_then(|members| members.get(name))
        .map_or(&[], Vec::as_slice)
}

/// The segments of a trait path as an impl block writes it without spaces,
/// and whether `::` opens it: its generic arguments left out; `None` for a
/// negative impl (`!Sync`) or a path that is not one of identifiers.
fn trait_segments(written: &str) -> Option<(bool, Vec<String>)> {
    if written.starts_with('!') {
        return None;
    }
    let path = &written[..written.find(['<', '(']).unwrap_or(written.len())];
    let (leading_colon, path) = match path.strip_prefix("::") {
        Some(rest) => (true, rest),
        None => (false, path),
    };
    let segs: Vec<String> = path
        .split("::")
        .map(|seg| seg.strip_prefix("r#").unwrap_or(seg).to_owned())
        .collect();
    segs.iter()
        .all(|seg| !seg.is_empty() && seg.chars().all(|c| c == '_' || c.is_alphanumeric()))
        .then_some((leading_colon, segs))
}

/// The walk: the index, the files, the tables built for it and what it has
/// found.
struct Walk<'a> {
    index: &'a Index,
    files: &'a BTreeMap<RelPath, SourceFile>,
    meter: &'a Meter,
    /// Where the runs of each item are read.
    homes: Vec<Home>,
    /// The impl blocks of each struct, enum and union.
    blocks_of_type: BTreeMap<ItemId, Vec<ImplId>>,
    /// The impl blocks each module holds.
    blocks_of_module: BTreeMap<usize, Vec<ImplId>>,
    /// The crate root items.
    roots: BTreeSet<ItemId>,
    /// The runs of each file lexed so far.
    files_runs: BTreeMap<RelPath, FileRuns>,
    /// The texts of the runs resolved so far, by the place they were read
    /// in. What a run named was given to the walk when it was resolved, so a
    /// run of the same text read in the same place again gives nothing.
    seen: BTreeMap<Ctx, BTreeSet<Box<str>>>,
    /// What the walk read of each impl block for the paths `Self` starts.
    blocks: BTreeMap<ImplId, BlockNames<'a>>,
}

impl<'a> Walk<'a> {
    /// The tables of the walk, charged to `meter` at `file` and `line`.
    fn new(
        index: &'a Index,
        files: &'a BTreeMap<RelPath, SourceFile>,
        meter: &'a Meter,
        (file, line): (&RelPath, usize),
    ) -> Result<Self, ReachError> {
        let mut homes = vec![Home::default(); index.items.len()];
        for (module, entry) in index.modules.iter().enumerate() {
            for id in &entry.defs {
                work::add(Work::ReachSteps, 1);
                homes[id.0].module = Some(module);
            }
        }
        for (module, id) in &index.root_items {
            work::add(Work::ReachSteps, 1);
            homes[id.0].module = Some(*module);
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
                homes[member.0] = Home {
                    module: Some(block.module_index),
                    owner: Some(Owner::Block(block.id)),
                };
            }
        }
        for (owner, members) in &index.assoc {
            if index.items[owner.0].kind != SymbolKind::Trait {
                continue;
            }
            // The trait's own tokens read `Self` as the trait too.
            homes[owner.0].owner = Some(Owner::Trait(*owner));
            for id in members.values().flatten() {
                work::add(Work::ReachSteps, 1);
                homes[id.0] = Home {
                    module: homes[owner.0].module,
                    owner: Some(Owner::Trait(*owner)),
                };
            }
        }
        let roots: BTreeSet<ItemId> = index.root_items.values().copied().collect();
        let held = homes.capacity() * size_of::<Home>()
            + lists_bytes(&blocks_of_type)
            + lists_bytes(&blocks_of_module)
            + btree_map::<ItemId, ()>()
            + roots.len() * btree_entry::<ItemId, ()>()
            + btree_map::<RelPath, FileRuns>()
            + btree_map::<Ctx, BTreeSet<Box<str>>>()
            + btree_map::<ImplId, BlockNames<'_>>();
        meter
            .items(held, file, Some(line))
            .map_err(ReachError::Budget)?;
        Ok(Self {
            index,
            files,
            meter,
            homes,
            blocks_of_type,
            blocks_of_module,
            roots,
            files_runs: BTreeMap::new(),
            seen: BTreeMap::new(),
            blocks: BTreeMap::new(),
        })
    }

    /// Lex and place the runs of `file` the first time they are asked, with
    /// the keywords of the edition of the package of `module`, and charge
    /// what they hold.
    fn load(&mut self, file: &RelPath, module: usize) -> Result<(), ReachError> {
        if self.files_runs.contains_key(file) {
            return Ok(());
        }
        let text = self
            .files
            .get(file)
            .map_or("", |source| source.text.as_str());
        let lexed = lex(file, text, self.index.edition(module))?;
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
        let mut held = btree_entry::<RelPath, FileRuns>()
            + lexed.runs.capacity() * size_of::<Run>()
            + lexed.segs.capacity() * size_of::<(usize, usize)>();
        if !by_unit.is_empty() {
            held += btree_map::<(usize, usize), Vec<usize>>();
        }
        for positions in by_unit.values() {
            held += btree_entry::<(usize, usize), Vec<usize>>()
                + positions.capacity() * size_of::<usize>();
        }
        self.meter
            .items(held, file, None)
            .map_err(ReachError::Budget)?;
        self.files_runs.insert(
            file.clone(),
            FileRuns {
                runs: lexed.runs,
                segs: lexed.segs,
                by_unit,
            },
        );
        Ok(())
    }

    /// The items the own tokens of `unit` name: the runs of its file given
    /// to its span that start before where its own tokens end, each read in
    /// its place the first time its text is read there.
    fn references(&mut self, unit: &Unit<'_>) -> Result<Vec<ItemId>, ReachError> {
        self.load(unit.file, unit.module)?;
        let index = self.index;
        let text = self
            .files
            .get(unit.file)
            .map_or("", |source| source.text.as_str());
        let runs = &self.files_runs[unit.file];
        let mut out = Vec::new();
        for position in runs.by_unit.get(&unit.span).into_iter().flatten() {
            let run = runs.runs[*position];
            if run.start >= unit.before {
                break;
            }
            work::add(Work::ReachSteps, 1);
            let segs = &runs.segs[run.first..run.first + run.len];
            let word = |at: usize| &text[segs[at].0..segs[at].1];
            let (ctx, key) = match (run.kind, unit.owner) {
                (RunKind::Path, _) => {
                    let mut key = String::from(if run.leading_colon { "::" } else { "" });
                    for at in 0..segs.len() {
                        if at > 0 {
                            key.push_str("::");
                        }
                        key.push_str(word(at));
                    }
                    (Ctx::Module(unit.module, unit.test), key)
                }
                // Only the name after `Self` is read.
                (RunKind::SelfPath, Some(owner)) if run.len > 1 => {
                    (Ctx::Owner(owner), format!("Self::{}", word(1)))
                }
                (RunKind::Method, Some(owner)) => (Ctx::Owner(owner), format!(".{}", word(0))),
                (RunKind::SelfPath | RunKind::Method, _) => continue,
            };
            let mut held = btree_entry::<Box<str>, ()>() + key.len();
            if !self.seen.contains_key(&ctx) {
                held += btree_entry::<Ctx, BTreeSet<Box<str>>>() + btree_map::<Box<str>, ()>();
            }
            let seen = self.seen.entry(ctx).or_default();
            if seen.contains(key.as_str()) {
                continue;
            }
            self.meter
                .items(held, unit.file, Some(unit.line))
                .map_err(ReachError::Budget)?;
            seen.insert(key.into_boxed_str());
            work::add(Work::ReachPaths, 1);
            match (run.kind, ctx) {
                (RunKind::Path, _) => {
                    let words: Vec<String> = (0..segs.len()).map(|at| word(at).to_owned()).collect();
                    out.extend(index.reach_run(unit.module, &words, run.leading_colon, unit.test));
                }
                (kind, Ctx::Owner(owner)) => {
                    let (name, variant) = match kind {
                        RunKind::SelfPath => (word(1), true),
                        _ => (word(0), false),
                    };
                    let names = owner_names(
                        index,
                        self.meter,
                        &mut self.blocks,
                        owner,
                        (name, variant),
                        unit,
                    )?;
                    out.extend(names);
                }
                (_, Ctx::Module(..)) => {}
            }
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
                let home = self.homes[id.0];
                if first && let Some(module) = home.module {
                    // A module's own tokens end where its body starts.
                    let before = match (item.kind, &item.body) {
                        (SymbolKind::Mod, Some(body)) => body.start_byte,
                        _ => usize::MAX,
                    };
                    let unit = Unit {
                        file: &item.file,
                        line: item.name_line,
                        span: (item.span.start_byte, item.span.end_byte),
                        before,
                        module,
                        test: item.test != TestRole::None,
                        owner: home.owner,
                    };
                    for found in self.references(&unit)? {
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
                    let unit = Unit {
                        file: &block.file,
                        line: block.line,
                        span: (block.span.start_byte, block.span.end_byte),
                        before: usize::MAX,
                        module: block.module_index,
                        test: block.test != TestRole::None,
                        owner: Some(Owner::Block(id)),
                    };
                    for found in self.references(&unit)? {
                        work::add(Work::ReachSteps, 1);
                        out.push((Node::Item(found), Edge::References));
                    }
                }
            }
        }
        Ok(out)
    }
}

/// The items that `Self::name` (`variant`) or `self.name(` names in
/// `owner`: in a trait, its members called `name`; in an impl block, the
/// members called `name` of the types its self type keys (every block of
/// them, the block itself among them) or, when no type keys it, of the block
/// itself, and the members called `name` of the trait it implements. A
/// `Self::name` that names none of them in a block on an enum names the enum:
/// `name` is a variant, as `E::V` names `E`.
fn owner_names<'a>(
    index: &'a Index,
    meter: &Meter,
    blocks: &mut BTreeMap<ImplId, BlockNames<'a>>,
    owner: Owner,
    (name, variant): (&str, bool),
    unit: &Unit<'_>,
) -> Result<Vec<ItemId>, ReachError> {
    let mut out: Vec<ItemId> = Vec::new();
    match owner {
        Owner::Trait(id) => out.extend_from_slice(members_named(index, id, name)),
        Owner::Block(id) => {
            let block = &index.impl_blocks[id.0];
            let names = block_names(index, meter, blocks, id, unit)?;
            for ty in &block.self_items {
                out.extend_from_slice(members_named(index, *ty, name));
            }
            if let Some(members) = names.members.get(name) {
                out.extend_from_slice(members);
            }
            for found in &names.traits {
                out.extend_from_slice(members_named(index, *found, name));
            }
            if variant && out.is_empty() {
                out.extend(
                    block
                        .self_items
                        .iter()
                        .filter(|ty| index.items[ty.0].kind == SymbolKind::Enum),
                );
            }
        }
    }
    Ok(out)
}

/// What the walk reads of the impl block `id` for `Self` paths, built and
/// charged the first time it is asked: its members by name when no type
/// keys it, and the traits its trait path names from its module.
fn block_names<'a, 'b>(
    index: &'a Index,
    meter: &Meter,
    blocks: &'b mut BTreeMap<ImplId, BlockNames<'a>>,
    id: ImplId,
    unit: &Unit<'_>,
) -> Result<&'b BlockNames<'a>, ReachError> {
    if !blocks.contains_key(&id) {
        let block = &index.impl_blocks[id.0];
        let mut names = BlockNames::default();
        if block.self_items.is_empty() {
            for member in &block.members {
                work::add(Work::ReachSteps, 1);
                names
                    .members
                    .entry(index.items[member.0].name.as_str())
                    .or_default()
                    .push(*member);
            }
        }
        if let Some((leading_colon, segs)) = block
            .trait_path
            .as_ref()
            .map(SharedStr::as_str)
            .and_then(trait_segments)
        {
            work::add(Work::ReachPaths, 1);
            names.traits = index
                .reach_run(
                    block.module_index,
                    &segs,
                    leading_colon,
                    block.test != TestRole::None,
                )
                .into_iter()
                .filter(|found| index.items[found.0].kind == SymbolKind::Trait)
                .collect();
        }
        let mut held =
            btree_entry::<ImplId, BlockNames<'_>>() + names.traits.capacity() * size_of::<ItemId>();
        if !names.members.is_empty() {
            held += btree_map::<&str, Vec<ItemId>>();
        }
        for members in names.members.values() {
            held += btree_entry::<&str, Vec<ItemId>>() + members.capacity() * size_of::<ItemId>();
        }
        meter
            .items(held, unit.file, Some(unit.line))
            .map_err(ReachError::Budget)?;
        blocks.insert(id, names);
    }
    Ok(&blocks[&id])
}

/// The file and the line of `node`, which a charge for it names.
fn place_of(index: &Index, node: Node) -> (&RelPath, usize) {
    match node {
        Node::Item(id) => {
            let item = &index.items[id.0];
            (&item.file, item.name_line)
        }
        Node::Impl(id) => {
            let block = &index.impl_blocks[id.0];
            (&block.file, block.line)
        }
    }
}

/// The nodes that `roots` reach in `index`, whose files are `files`, what
/// the walk holds charged to the item-bytes budget of `options` on from the
/// `held` bytes of the load.
pub(crate) fn reachable(
    index: &Index,
    files: &BTreeMap<RelPath, SourceFile>,
    (options, held): (&LoadOptions, usize),
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
    let Some(first) = sorted.first() else {
        return Ok(Reachability {
            reached: Vec::new(),
            at: BTreeMap::new(),
        });
    };
    let (file, line) = place_of(index, *first);
    let meter = Meter::new(options);
    meter
        .items(
            held + vec_first::<Reached>()
                + btree_map::<Node, usize>()
                + btree_map::<ItemId, ()>()
                + vec_first::<Queued>(),
            file,
            Some(line),
        )
        .map_err(ReachError::Budget)?;
    let mut walk = Walk::new(index, files, &meter, (file, line))?;
    let mut queue: VecDeque<Queued> = sorted
        .into_iter()
        .map(|node| (node, Edge::Root, None))
        .collect();
    let mut peak = queue.len();
    meter
        .items(peak * vec_slot::<Queued>(), file, Some(line))
        .map_err(ReachError::Budget)?;
    let mut reached: Vec<Reached> = Vec::new();
    let mut at: BTreeMap<Node, usize> = BTreeMap::new();
    let mut opened: BTreeSet<ItemId> = BTreeSet::new();
    while let Some((node, edge, from)) = queue.pop_front() {
        work::add(Work::ReachSteps, 1);
        let first = !at.contains_key(&node);
        let (file, line) = place_of(index, node);
        if first {
            meter
                .items(
                    vec_slot::<Reached>() + btree_entry::<Node, usize>(),
                    file,
                    Some(line),
                )
                .map_err(ReachError::Budget)?;
            at.insert(node, reached.len());
            reached.push(Reached { node, edge, from });
        }
        let was_opened = opened.len();
        for (next, by) in walk.next(node, edge, first, &mut opened)? {
            let opens = matches!(next, Node::Item(id) if index.items[id.0].kind == SymbolKind::Mod && by == Edge::Contains && !opened.contains(&id));
            if opens || !at.contains_key(&next) {
                queue.push_back((next, by, Some(node)));
            }
        }
        // What the queue holds at its most, and each module opened.
        let grown = queue.len().saturating_sub(peak) * vec_slot::<Queued>()
            + (opened.len() - was_opened) * btree_entry::<ItemId, ()>();
        peak = peak.max(queue.len());
        if grown > 0 {
            meter
                .items(grown, file, Some(line))
                .map_err(ReachError::Budget)?;
        }
    }
    Ok(Reachability { reached, at })
}

