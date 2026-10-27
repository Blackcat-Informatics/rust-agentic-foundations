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
//!   Positively identified standard `format!` calls also name their decoded
//!   implicit value and count captures. Named argument binding identifiers are
//!   not value paths; their expressions retain the ordinary reference rules.
//!
//! The tokens of an item are those of its span outside the spans of the items
//! and blocks below it: a type's members and a block's members are their own
//! units. A module's own tokens are its declaration before its body; what a
//! module body holds outside its items and blocks (its `use` declarations and
//! the macro invocations at module level) is no item's, and a crate root has
//! none. Each file is lexed once per package edition, the first time an item
//! of it is reached in that edition, and
//! each run is resolved, and what it names given to the walk, at most once
//! for each place it is read in: a module with a test flag, or an impl block
//! or a trait. What the walk holds is charged to the item-bytes budget of the
//! load.

use std::collections::btree_map::Entry;
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
use crate::meter::{Meter, Steps};
use crate::options::LoadOptions;
use crate::parse::PARSE_STACK_BYTES;
use crate::path::RelPath;
use crate::work::{self, Work};
use crate::workspace::SourceFile;

mod format;

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
    /// ([`LoadErrorKind::Syntax`]), which a file the load parsed does not do;
    /// or native format-context storage could not be allocated
    /// ([`LoadErrorKind::Storage`]).
    Source(LoadError),
    /// The walk passed its query-work limit or the item-bytes budget of the load
    /// ([`Budget::ItemBytes`](crate::Budget::ItemBytes)), counted on from
    /// what the load holds: a `Budget` load error naming the budget, its
    /// limit and the file of the runs or the node being held.
    Budget(LoadError),
}

impl From<LoadError> for ReachError {
    fn from(error: LoadError) -> Self {
        if matches!(error.kind, LoadErrorKind::Budget { .. }) {
            Self::Budget(error)
        } else {
            Self::Source(error)
        }
    }
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
    formats: Option<Box<format::Formats>>,
    format_tokens: usize,
    format_bytes: usize,
}

impl Lexed {
    /// Start a run of one segment and return its place.
    fn start(
        &mut self,
        start: usize,
        leading_colon: bool,
        kind: RunKind,
        seg: (usize, usize),
    ) -> usize {
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

/// The runs of `stream`, lexed from `text` past its first `base` bytes, so
/// that its byte offsets plus `base` are those of `text`, in text order, and
/// the tokens read, with the keywords of `edition`. The levels of delimiters
/// are walked on an explicit stack.
///
/// A run of one segment directly followed by a single `:` is a name being
/// bound or a field, not a path (`helper: u8`, `S { helper: 1 }`), and is
/// dropped, except in a `where` clause, where it is the type a bound is put
/// on (`where Helper: Clone`); a `::` after it goes on with it. A name after
/// `self.` that a `(` or a `::` follows is a method called on `self`.
///
/// An identifier is a segment only where `text` spells it. The lexer reads a
/// doc comment as the attribute `#[doc = "..."]` and gives its `doc` the span
/// of the whole comment, so that `doc` is written nowhere: it starts no run,
/// and no byte of the comment is read as a segment.
fn scan(stream: TokenStream, text: &str, base: usize, edition: u16) -> Lexed {
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
                let range = ident.span().byte_range();
                let span = (base + range.start, base + range.end);
                // A span's ends are character boundaries of the lexed text:
                // the lexer counts characters and gives their byte offsets.
                if ident == text[span.0..span.1] {
                    open = scan_ident(&mut lexed, level, open, &ident, span, edition, &mut method);
                } else {
                    open = None;
                    level.push(Tok::Ident {
                        in_run: false,
                        word: Word::Plain,
                    });
                }
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

/// Scan the identifier `ident`, written in the text at the byte range
/// `span`, after the tokens of `level`, with the open run `open`, and return
/// the open run after it. A name after `self.` is kept in `method` until the
/// next token.
fn scan_ident(
    lexed: &mut Lexed,
    level: &mut Level,
    open: Option<Open>,
    ident: &proc_macro2::Ident,
    span: (usize, usize),
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
    let (start, end) = span;
    let seg = (end - word.len(), end);
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
    let mut lexed = scan(stream.clone(), text, base, edition);
    (lexed.formats, lexed.format_tokens, lexed.format_bytes) = format::scan(stream, text, base);
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
    formats: Option<Box<format::Formats>>,
    calls_by_unit: BTreeMap<(usize, usize), Vec<usize>>,
}

/// Give each run, in text order, to the innermost of `units` (spans as
/// `(start, end)`, nested or apart) that holds its start.
fn place_runs(runs: &[Run], units: Vec<(usize, usize)>) -> BTreeMap<(usize, usize), Vec<usize>> {
    place_starts(
        runs.iter().enumerate().map(|(at, run)| (at, run.start)),
        units,
        Work::ReachUnits,
    )
}

fn place_starts(
    starts: impl Iterator<Item = (usize, usize)>,
    mut units: Vec<(usize, usize)>,
    kind: Work,
) -> BTreeMap<(usize, usize), Vec<usize>> {
    // Outer units first where two start at one byte.
    units.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
    units.dedup();
    let mut by_unit: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    let mut open: Vec<(usize, usize)> = Vec::new();
    let mut next = 0;
    for (position, start) in starts {
        while let Some(&unit) = units.get(next).filter(|unit| unit.0 <= start) {
            work::add(kind, 1);
            while open.last().is_some_and(|top| top.1 <= unit.0) {
                work::add(kind, 1);
                open.pop();
            }
            open.push(unit);
            next += 1;
        }
        while open.last().is_some_and(|top| top.1 <= start) {
            work::add(kind, 1);
            open.pop();
        }
        if let Some(unit) = open.last() {
            work::add(kind, 1);
            by_unit.entry(*unit).or_default().push(position);
        }
    }
    for positions in by_unit.values_mut() {
        positions.shrink_to_fit();
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
    Value(usize, bool),
    Owner(Owner),
}

/// Where the runs of an item are read: the module of its paths (its own
/// module, the module of its impl block for a member of one, the module of
/// its trait for a member of a trait), and what `Self` stands for in it.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Home {
    module: Option<usize>,
    owner: Option<Owner>,
}

impl Home {
    pub(crate) const fn module(self) -> Option<usize> {
        self.module
    }
}

/// Populate the shared source-owned reading context of every indexed item.
pub(crate) fn fill_homes(index: &Index, homes: &mut [Home], lexical: bool) {
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
    if lexical {
        for (id, module) in &index.macro_modules {
            work::add(Work::ReachSteps, 1);
            homes[id.0].module = Some(*module);
        }
    }
    for block in &index.impl_blocks {
        work::add(Work::ReachSteps, 1);
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
        homes[owner.0].owner = Some(Owner::Trait(*owner));
        for id in members.values().flatten() {
            work::add(Work::ReachSteps, 1);
            homes[id.0] = Home {
                module: homes[owner.0].module,
                owner: Some(Owner::Trait(*owner)),
            };
        }
    }
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
    /// The runs of each file lexed so far, with the edition whose keywords
    /// were used. One physical file can be shared by packages of different editions.
    files_runs: BTreeMap<(RelPath, u16), FileRuns>,
    /// The texts of the runs resolved so far, by the place they were read
    /// in. What a run named was given to the walk when it was resolved, so a
    /// run of the same text read in the same place again gives nothing.
    seen: BTreeMap<Ctx, BTreeSet<Box<str>>>,
    /// What the walk read of each impl block for the paths `Self` starts.
    blocks: BTreeMap<ImplId, BlockNames<'a>>,
    format_identity: Option<format::Resolver>,
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
        // Reachability retains its documented export-placement approximation;
        // lexical source observations retain actual macro declaration modules.
        fill_homes(index, &mut homes, false);
        let mut blocks_of_type: BTreeMap<ItemId, Vec<ImplId>> = BTreeMap::new();
        let mut blocks_of_module: BTreeMap<usize, Vec<ImplId>> = BTreeMap::new();
        for block in &index.impl_blocks {
            blocks_of_module
                .entry(block.module_index)
                .or_default()
                .push(block.id);
            for ty in &block.self_items {
                work::add(Work::ReachSteps, 1);
                blocks_of_type.entry(*ty).or_default().push(block.id);
            }
        }
        let roots: BTreeSet<ItemId> = index.root_items.values().copied().collect();
        let held = homes.capacity() * size_of::<Home>()
            + lists_bytes(&blocks_of_type)
            + lists_bytes(&blocks_of_module)
            + btree_map::<ItemId, ()>()
            + roots.len() * btree_entry::<ItemId, ()>()
            + btree_map::<(RelPath, u16), FileRuns>()
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
            format_identity: None,
        })
    }

    /// Lex and place the runs of `file` the first time they are asked, with
    /// the keywords of the edition of the package of `module`, and charge
    /// what they hold.
    fn load(&mut self, file: &RelPath, module: usize) -> Result<(RelPath, u16), ReachError> {
        let key = (file.clone(), self.index.edition(module));
        if self.files_runs.contains_key(&key) {
            return Ok(key);
        }
        let text = self
            .files
            .get(file)
            .map_or("", |source| source.text.as_str());
        let lexed = lex(file, text, key.1)?;
        work::add(Work::ReachTokens, lexed.tokens);
        work::add(Work::ReachFormatTokens, lexed.format_tokens);
        work::add(Work::ReachFormatBytes, lexed.format_bytes);
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
        let calls_by_unit = lexed
            .formats
            .as_ref()
            .map_or_else(BTreeMap::new, |formats| {
                place_starts(
                    formats
                        .calls
                        .iter()
                        .enumerate()
                        .map(|(at, call)| (at, call.start)),
                    units.clone(),
                    Work::ReachFormatTokens,
                )
            });
        let by_unit = place_runs(&lexed.runs, units);
        let mut held = btree_entry::<(RelPath, u16), FileRuns>()
            + lexed.runs.capacity() * size_of::<Run>()
            + lexed.segs.capacity() * size_of::<(usize, usize)>();
        if !by_unit.is_empty() {
            held += btree_map::<(usize, usize), Vec<usize>>();
        }
        for positions in by_unit.values() {
            held += btree_entry::<(usize, usize), Vec<usize>>()
                + positions.capacity() * size_of::<usize>();
        }
        if let Some(formats) = &lexed.formats {
            held += formats.retained_bytes();
        }
        if !calls_by_unit.is_empty() {
            held += btree_map::<(usize, usize), Vec<usize>>();
            for positions in calls_by_unit.values() {
                held += btree_entry::<(usize, usize), Vec<usize>>()
                    + positions.capacity() * size_of::<usize>();
            }
        }
        if lexed.formats.is_some() && self.format_identity.is_none() {
            let identity = format::Resolver::new(self.index, self.files, self.meter, file)?;
            self.format_identity = Some(identity);
        }
        self.meter
            .items(held, file, None)
            .map_err(ReachError::Budget)?;
        self.files_runs.insert(
            key.clone(),
            FileRuns {
                runs: lexed.runs,
                segs: lexed.segs,
                by_unit,
                formats: lexed.formats,
                calls_by_unit,
            },
        );
        Ok(key)
    }

    /// The items the own tokens of `unit` name: the runs of its file given
    /// to its span that start before where its own tokens end, each read in
    /// its place the first time its text is read there.
    fn references(&mut self, unit: &Unit<'_>) -> Result<Vec<ItemId>, ReachError> {
        let key = self.load(unit.file, unit.module)?;
        let index = self.index;
        let text = self
            .files
            .get(unit.file)
            .map_or("", |source| source.text.as_str());
        let runs = &self.files_runs[&key];
        let mut out = Vec::new();
        let suppressed = format_references(
            index,
            self.meter,
            self.format_identity.as_ref(),
            &mut self.seen,
            unit,
            runs,
            &mut out,
        )?;
        for position in runs.by_unit.get(&unit.span).into_iter().flatten() {
            let run = runs.runs[*position];
            if run.start >= unit.before {
                break;
            }
            if suppressed.binary_search(&run.start).is_ok() {
                continue;
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
                    let words: Vec<String> =
                        (0..segs.len()).map(|at| word(at).to_owned()).collect();
                    out.extend(index.reach_run(
                        unit.module,
                        &words,
                        run.leading_colon,
                        unit.test,
                        Steps::QueryItems {
                            meter: self.meter,
                            file: unit.file,
                            line: Some(unit.line),
                        },
                    )?);
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
                (_, Ctx::Module(..) | Ctx::Value(..)) => {}
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

fn format_references(
    index: &Index,
    meter: &Meter,
    identity: Option<&format::Resolver>,
    seen_paths: &mut BTreeMap<Ctx, BTreeSet<Box<str>>>,
    unit: &Unit<'_>,
    runs: &FileRuns,
    out: &mut Vec<ItemId>,
) -> Result<Vec<usize>, ReachError> {
    let mut suppressed = Vec::new();
    if let (Some(formats), Some(identity)) = (&runs.formats, identity) {
        for &position in runs.calls_by_unit.get(&unit.span).into_iter().flatten() {
            let call = &formats.calls[position];
            if call.start >= unit.before {
                break;
            }
            if !identity.standard(
                formats,
                call,
                index,
                unit,
                Steps::QueryItems {
                    meter,
                    file: unit.file,
                    line: Some(unit.line),
                },
            )? {
                continue;
            }
            let Some(arguments) = &call.arguments else {
                continue;
            };
            let arguments = arguments
                .as_ref()
                .map_err(|refusal| ReachError::Source(refusal.error(unit.file)))?;
            suppressed.extend_from_slice(&arguments.bindings);
            for name in &arguments.captures {
                let ctx = Ctx::Value(unit.module, unit.test);
                let mut held = btree_entry::<Box<str>, ()>() + name.len();
                if !seen_paths.contains_key(&ctx) {
                    held += btree_entry::<Ctx, BTreeSet<Box<str>>>() + btree_map::<Box<str>, ()>();
                }
                let seen = seen_paths.entry(ctx).or_default();
                if seen.contains(name.as_ref()) {
                    continue;
                }
                meter
                    .items(held, unit.file, Some(unit.line))
                    .map_err(ReachError::Budget)?;
                seen.insert(name.clone());
                work::add(Work::ReachPaths, 1);
                out.extend(index.reach_value(
                    unit.module,
                    name,
                    unit.test,
                    Steps::QueryItems {
                        meter,
                        file: unit.file,
                        line: Some(unit.line),
                    },
                )?);
            }
        }
    }
    suppressed.sort_unstable();
    Ok(suppressed)
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
    let slot = match blocks.entry(id) {
        Entry::Occupied(found) => return Ok(found.into_mut()),
        Entry::Vacant(slot) => slot,
    };
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
                Steps::QueryItems {
                    meter,
                    file: unit.file,
                    line: Some(unit.line),
                },
            )?
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
    Ok(slot.insert(names))
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
    let meter = Meter::query(options);
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

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;
    use std::mem::size_of;

    use super::{DECLARES, Run, RunKind, Walk, lex_text, place_runs};
    use crate::budget::Budget;
    use crate::error::LoadErrorKind;
    use crate::item::Namespace;
    use crate::meter::{Meter, Steps};
    use crate::reach::{Node, ReachError};
    use crate::work::{self, Work};
    use crate::{LoadOptions, MemoryTree, Workspace};

    /// The runs of `text` in `edition` as their text and the byte where each
    /// starts: a path as written without spaces, a run that `Self` starts as
    /// `Self::...`, and a method called on `self` as `.name`.
    fn runs_in(edition: u16, text: &str) -> Vec<(String, usize)> {
        let lexed = lex_text(text, edition).unwrap_or_else(|failure| panic!("{}", failure.message));
        lexed
            .runs
            .iter()
            .map(|run| {
                let words: Vec<&str> = lexed.segs[run.first..run.first + run.len]
                    .iter()
                    .map(|(start, end)| &text[*start..*end])
                    .collect();
                let shown = match run.kind {
                    RunKind::Method => format!(".{}", words.join("::")),
                    RunKind::Path | RunKind::SelfPath => format!(
                        "{}{}",
                        if run.leading_colon { "::" } else { "" },
                        words.join("::")
                    ),
                };
                (shown, run.start)
            })
            .collect()
    }

    fn runs(text: &str) -> Vec<(String, usize)> {
        runs_in(2021, text)
    }

    /// The texts of the runs of `text` in `edition`.
    fn texts_in(edition: u16, text: &str) -> Vec<String> {
        runs_in(edition, text)
            .into_iter()
            .map(|(text, _)| text)
            .collect()
    }

    /// The texts of the runs of `text` in edition 2021.
    fn texts(text: &str) -> Vec<String> {
        texts_in(2021, text)
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
        // A raw segment is read without its prefix, where it starts.
        assert_eq!(runs("r#gen::x"), [("gen::x".to_owned(), 0)]);
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
        assert_eq!(texts("pub use a as b;"), ["a", "b"]);
        // `Self` starts a run of its own and is no later segment.
        assert_eq!(
            texts("Self::new(); impl Tr for S; a::Self::b; ::Self::c"),
            ["Self::new", "Tr", "S", "a"]
        );
        assert_eq!(texts("Self { a: 1 }; Self(2)"), ["Self", "Self"]);
    }

    #[test]
    fn the_keywords_are_those_of_the_edition() {
        let text = "async(); await(); dyn(); try(); gen(); r#gen(); r#try()";
        assert_eq!(
            texts_in(2015, text),
            ["async", "await", "dyn", "try", "gen", "gen", "try"]
        );
        for edition in [2018, 2021] {
            assert_eq!(texts_in(edition, text), ["gen", "gen", "try"], "{edition}");
        }
        assert_eq!(texts_in(2024, text), ["gen", "try"]);
        // Every edition's keywords end a run.
        assert_eq!(texts_in(2015, "a::fn::b; a::dyn::b"), ["a", "a::dyn::b"]);
        assert_eq!(texts_in(2024, "a::gen::b; a::r#gen::b"), ["a", "a::gen::b"]);
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
    fn const_declares_nothing_so_a_pointer_and_a_raw_borrow_name_their_path() {
        assert!(!DECLARES.contains(&"const"));
        assert_eq!(
            texts("fn f(p: *const Helper, q: *mut Other) {}"),
            ["Helper", "Other"]
        );
        assert_eq!(
            texts("&raw const VALUE; &raw mut OTHER"),
            ["raw", "VALUE", "raw", "OTHER"]
        );
        // Where `const` declares, what it declares is still no run: a
        // constant, a const fn, a const block and a const parameter.
        assert_eq!(
            texts("const C: u8 = 1; pub const D: E = F;"),
            ["u8", "E", "F"]
        );
        assert_eq!(
            texts("const fn helper() {} const unsafe fn other() {}"),
            Vec::<String>::new()
        );
        assert_eq!(texts("const { g() }"), ["g"]);
        assert_eq!(texts("fn f<const N: usize>() {}"), ["usize"]);
        assert_eq!(texts("impl<const N: usize> S<N> {}"), ["usize", "S", "N"]);
    }

    #[test]
    fn a_where_clause_keeps_the_type_its_bound_is_put_on() {
        assert_eq!(
            texts("fn f<T>() where Helper: Clone, T: Into<Other> {}"),
            ["T", "Helper", "Clone", "T", "Into", "Other"]
        );
        // The clause ends at the body or at `;`, and a bound name after it is
        // no run again.
        assert_eq!(
            texts("fn f(helper: u8) where T: X { S { field: 1 } }"),
            ["u8", "T", "X", "S"]
        );
        assert_eq!(
            texts("fn g() where Self: Sized; fn h(x: u8) {}"),
            ["Self", "Sized", "u8"]
        );
        assert_eq!(
            texts(
                "struct S<T> where T: X { field: u8 } struct P(u8) where Q: Y; static Z: u8 = 0;"
            ),
            ["T", "T", "X", "u8", "u8", "Q", "Y", "u8"]
        );
        // A generic parameter's own bound is a name being bound.
        assert_eq!(texts("fn f<T: Clone>() {}"), ["Clone"]);
    }

    #[test]
    fn a_name_called_on_self_is_a_method_run_and_a_field_of_self_is_not() {
        assert_eq!(
            texts("self.helper(); self.field; self.turbo::<u8>(); other.method(); self.0.x()"),
            [
                "self", ".helper", "self", "self", ".turbo", "u8", "other", "self"
            ]
        );
        let text = "fn f(&self) { self.r#match() }";
        assert_eq!(
            runs(text),
            [
                ("self".to_owned(), text.find("self)").expect("the receiver")),
                ("self".to_owned(), text.find("self.").expect("self")),
                (
                    ".match".to_owned(),
                    text.find("r#match").expect("the method")
                ),
            ]
        );
        // `self` as a path, and a name after a `.` of anything else, are not.
        assert_eq!(
            texts("self::helper(); s.helper(); Self.x()"),
            ["self::helper", "s", "Self"]
        );
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
        let failure = lex_text("fn f() {\n\"", 2021).expect_err("an open string");
        assert_eq!(failure.line, 2);
    }

    /// A doc comment lexes as a `doc` attribute whose `doc` is written
    /// nowhere in the text, so it starts no run and no byte of its text is
    /// read as a segment, whatever character ends it; a written `#[doc]`
    /// is a run as any attribute is.
    #[test]
    fn a_doc_comment_is_no_run_whatever_character_ends_it() {
        let text = "//! Crate §\n/// Calls abc\n/// See §1,\n/** Block ß */\n\
                    #[doc = \"x\"]\npub fn f() { a::b() }\nfn abc() {}\n";
        let ws = load(text);
        let site = |path: &str| {
            Node::Item(
                ws.resolve_in(path, Namespace::Value)
                    .unwrap_or_else(|e| panic!("{path}: {e}"))
                    .sites[0],
            )
        };
        let reached = ws
            .reachable(&[site("demo::f")])
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(!reached.contains(site("demo::abc")));
        assert_eq!(
            runs(text),
            [
                ("doc".to_owned(), text.find("doc").expect("the attribute")),
                ("a::b".to_owned(), text.find("a::b").expect("the call")),
            ]
        );
    }

    fn run_at(start: usize) -> Run {
        Run {
            start,
            first: 0,
            len: 1,
            leading_colon: false,
            kind: RunKind::Path,
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

    const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

    fn load_with(text: &str, options: &LoadOptions) -> Workspace {
        let tree = MemoryTree::from([("Cargo.toml", MANIFEST), ("src/lib.rs", text)]);
        Workspace::load_with(&tree, &["."], options).unwrap_or_else(|e| panic!("{e}"))
    }

    fn load(text: &str) -> Workspace {
        load_with(text, &LoadOptions::default())
    }

    /// A library of `items` public functions, each calling its own private
    /// helper `references` times.
    fn fan(items: usize, references: usize) -> Workspace {
        let mut text = String::new();
        for i in 0..items {
            let calls = format!("h{i}(); ").repeat(references);
            writeln!(text, "pub fn f{i}() {{ {calls}}}\nfn h{i}() {{}}").expect("write");
        }
        load(&text)
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

    /// The work of reaching `roots` in `ws`, by kind, and how many nodes
    /// were reached.
    fn work_of(ws: &Workspace, roots: &[Node]) -> ([usize; 6], usize) {
        let (reached, tally) = work::measure(usize::MAX, || ws.reachable(roots));
        let reached = reached.unwrap_or_else(|e| panic!("{e}")).reached.len();
        (KINDS.map(|kind| tally.get(kind)), reached)
    }

    /// The work of reaching every public function of `fan(items,
    /// references)`.
    fn fan_work(items: usize, references: usize) -> ([usize; 6], usize) {
        let ws = fan(items, references);
        let roots: Vec<Node> = (0..items)
            .map(|i| {
                let path = format!("demo::f{i}");
                Node::Item(ws.resolve_in(&path, Namespace::Value).expect("f").sites[0])
            })
            .collect();
        work_of(&ws, &roots)
    }

    #[test]
    fn the_work_of_a_walk_is_counted_exactly() {
        for (n, r) in [(1, 1), (3, 2), (5, 4)] {
            let ([tokens, units, paths, steps, ..], reached) = fan_work(n, r);
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
            // roots, each node taken from the queue (a root and its helper),
            // each run of a function, and the one site its first run names:
            // a run read again in its module gives nothing.
            assert_eq!(steps, 6 * n + 1 + n * r, "{n} {r}");
        }
    }

    #[test]
    fn the_work_grows_with_the_items_and_the_references_and_not_their_product() {
        // Three sizes in items at a fixed number of references, then in
        // references at a fixed number of items: each step adds the same
        // work, so no kind grows with a product of the two.
        let by_items: Vec<[usize; 6]> = [10, 20, 30].map(|n| fan_work(n, 3).0).to_vec();
        let by_references: Vec<[usize; 6]> = [2, 4, 6].map(|r| fan_work(10, r).0).to_vec();
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

    /// A library of a trait of `members` methods and `types` modules below
    /// one module `top`, each holding a struct with an inherent block of
    /// `members` methods and a block of the trait, an enum with a block,
    /// a type alias, a constant, a static and a function. Each method calls
    /// its next sibling through `Self::`, through `self.` and names the
    /// constant and the static; each trait method calls its inherent
    /// namesake on `self`.
    fn kinds(types: usize, members: usize) -> String {
        let mut text = String::from("pub trait Tr {\n");
        for j in 0..members {
            writeln!(text, "    fn t{j}(&self) -> u8;").expect("write");
        }
        text.push_str("}\npub mod top {\n");
        for i in 0..types {
            writeln!(
                text,
                "pub mod m{i} {{\n\
                 pub struct T{i} {{ f: u8 }}\n\
                 pub enum E{i} {{ A, B }}\n\
                 pub type A{i} = T{i};\n\
                 const K{i}: u8 = 1;\n\
                 static S{i}: u8 = 2;\n\
                 pub fn helper{i}() -> E{i} {{ E{i}::A }}\n\
                 impl E{i} {{ pub fn is_a(&self) -> bool {{ matches!(self, Self::A) }} }}"
            )
            .expect("write");
            writeln!(text, "impl T{i} {{").expect("write");
            for j in 0..members {
                let next = (j + 1) % members;
                writeln!(
                    text,
                    "    pub fn a{j}(&self) -> u8 {{ Self::a{next}(self) + self.a{next}() + K{i} + S{i} }}"
                )
                .expect("write");
            }
            writeln!(text, "}}\nimpl crate::Tr for T{i} {{").expect("write");
            for j in 0..members {
                writeln!(text, "    fn t{j}(&self) -> u8 {{ self.a{j}() }}").expect("write");
            }
            text.push_str("}\n}\n");
        }
        text.push_str("}\n");
        text
    }

    /// The work of reaching the module `top` of `kinds(types, members)`.
    fn kinds_work(types: usize, members: usize) -> ([usize; 6], usize) {
        let ws = load(&kinds(types, members));
        let top = ws
            .resolve_in("demo::top", Namespace::Type)
            .expect("top")
            .sites[0];
        work_of(&ws, &[Node::Item(top)])
    }

    #[test]
    fn the_work_over_types_impl_blocks_traits_and_members_grows_with_each_and_not_their_product() {
        // Every item and block below `top` is reached: the module, and for
        // each type its module, the struct, the enum, the alias, the
        // constant, the static, the function, the method of the enum, the
        // three blocks and the 2k methods.
        for (n, k) in [(1, 1), (2, 3), (4, 2)] {
            let (_, reached) = kinds_work(n, k);
            assert_eq!(reached, 2 + k + n * (11 + 2 * k), "{n} {k}");
        }
        // Equal steps in the types at a fixed number of members, and in the
        // members at a fixed number of types, in every counted kind: no work
        // grows with the types times the blocks, nor with the members times
        // the paths.
        let by_types: Vec<[usize; 6]> = [4, 8, 12].map(|n| kinds_work(n, 3).0).to_vec();
        let by_members: Vec<[usize; 6]> = [2, 4, 6].map(|k| kinds_work(4, k).0).to_vec();
        for sizes in [&by_types, &by_members] {
            for (kind, name) in KINDS.iter().enumerate() {
                let first = sizes[1][kind] - sizes[0][kind];
                let second = sizes[2][kind] - sizes[1][kind];
                assert_eq!(first, second, "{name:?}: {sizes:?}");
            }
        }
        // The exact counts, by kind, for n types of k members: the tokens
        // of the trait (four, and nine for each method) and of `top` (four),
        // and of each module; the steps of placing their runs; the runs
        // resolved (two at the top, and in each module fourteen and three
        // for each method: `Self::a` and `.a` in the inherent block, and `.a`
        // in the block of the trait);
        // the steps of the walk; and the segment reads and lookups of the
        // paths resolved, which the members do not change: a path that
        // `Self` starts and a method called on `self` are read in their
        // block, not walked as paths.
        for (n, k) in [(1, 1), (2, 3), (4, 2), (3, 5)] {
            let ([tokens, units, paths, steps, segments, lookups], _) = kinds_work(n, k);
            assert_eq!(tokens, 8 + 9 * k + n * (83 + 38 * k), "{n} {k}");
            assert_eq!(units, 4 * k + n * (39 + 16 * k), "{n} {k}");
            assert_eq!(paths, 2 + n * (14 + 3 * k), "{n} {k}");
            assert_eq!(steps, 5 + 5 * k + n * (68 + 31 * k), "{n} {k}");
            assert_eq!(segments, 9 * n, "{n} {k}");
            assert_eq!(lookups, 3 + 34 * n, "{n} {k}");
        }
    }

    #[test]
    fn a_path_of_fifty_thousand_segments_is_walked_once_and_not_once_for_each_prefix() {
        const SEGMENTS: usize = 50_000;
        let text = format!(
            "pub mod m {{ pub use crate::m; pub struct End; }}\npub type T = {}End;\n",
            "m::".repeat(SEGMENTS)
        );
        let ws = load(&text);
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

    /// A library whose public function calls `a` `runs` times.
    fn dense(runs: usize) -> String {
        format!("pub fn f() {{ {}}}\nfn a() {{}}\n", "a; ".repeat(runs))
    }

    /// The least item-bytes limit under which the walk from `demo::f` of
    /// `text` is given, less what the load holds: what the walk charges.
    fn walk_charge(text: &str) -> usize {
        let ws = load(text);
        let held = ws.usage().item_bytes;
        let walk = |limit: usize| {
            let ws = load_with(
                text,
                &LoadOptions {
                    max_item_bytes: limit,
                    ..LoadOptions::default()
                },
            );
            let f = ws.resolve_in("demo::f", Namespace::Value).expect("f").sites[0];
            ws.reachable(&[Node::Item(f)])
        };
        let (mut low, mut high) = (held, held + (1 << 24));
        assert!(walk(high).is_ok());
        while high - low > 1 {
            let mid = low + (high - low) / 2;
            if walk(mid).is_ok() {
                high = mid;
            } else {
                low = mid;
            }
        }
        let error = walk(low).expect_err("one byte less is refused");
        match error {
            ReachError::Budget(error) => {
                assert!(
                    matches!(
                        error.kind,
                        LoadErrorKind::Budget {
                            budget: Budget::ItemBytes,
                            limit
                        } if limit == low
                    ),
                    "{error}"
                );
                assert_eq!(error.file.as_str(), "src/lib.rs");
            }
            other => panic!("{other}"),
        }
        high - held
    }

    #[test]
    fn what_the_walk_holds_is_charged_to_the_item_bytes_budget_on_from_the_load() {
        // Each run holds its record, its segment and its place in its unit,
        // and nothing else: the text it resolves to is held once.
        let per_run = size_of::<Run>() + size_of::<(usize, usize)>() + size_of::<usize>();
        let charges = [1000, 2000, 3000].map(|runs| walk_charge(&dense(runs)));
        assert_eq!(charges[1] - charges[0], 1000 * per_run);
        assert_eq!(charges[2] - charges[1], 1000 * per_run);
        // `a; ` is three bytes of text for each run.
        #[cfg(target_pointer_width = "64")]
        assert_eq!(per_run, 56);
        assert!(per_run < 20 * 3, "{per_run}");
    }

    #[test]
    fn format_candidates_charge_owned_bytes_and_return_worker_work_to_the_caller() {
        let source = |calls| {
            format!(
                "const VALUE:u32=7; pub fn f(){{ {} }}",
                "let _=format!(\"{VALUE}\");".repeat(calls)
            )
        };
        let samples = [32, 64, 96].map(|calls| {
            let text = source(calls);
            let ws = load(&text);
            let root = ws.resolve_in("demo::f", Namespace::Value).expect("f").sites[0];
            let capture = ws
                .resolve_in("demo::VALUE", Namespace::Value)
                .expect("VALUE")
                .sites[0];
            let (reached, tally) = work::measure(100_000, || ws.reachable(&[Node::Item(root)]));
            assert!(reached.expect("reachability").contains(Node::Item(capture)));
            let tokens = tally.get(Work::ReachFormatTokens);
            let bytes = tally.get(Work::ReachFormatBytes);
            assert!(tokens > calls, "worker token work was lost");
            assert!(
                bytes >= calls * "{VALUE}".len(),
                "worker decode work was lost"
            );
            (tokens, bytes, walk_charge(&text))
        });
        for at in 0..2 {
            let get = |n: usize| if at == 0 { samples[n].0 } else { samples[n].1 };
            assert_eq!(get(2) - get(1), get(1) - get(0));
        }
        assert!(samples[1].2 > samples[0].2);
        assert_eq!(samples[2].2 - samples[1].2, samples[1].2 - samples[0].2);
    }

    #[test]
    fn code_prefix_reads_share_one_query_account_and_refuse_one_less() {
        let tree = MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n",
            ),
            (
                "src/lib.rs",
                "mod a { pub struct X {} } mod b { pub use crate::a::*; } pub use b::*;",
            ),
        ]);
        let ws = Workspace::load(&tree, &["."]).expect("glob source");
        let index = ws.index();
        let root = index.target_roots[0];
        let (file, line) = index.module_site(root);
        let words = ["X".to_owned()];
        let run = |limit| {
            let options = LoadOptions {
                max_query_work: limit,
                ..LoadOptions::default()
            };
            let meter = Meter::query(&options);
            let answer = index.reach_run(
                root,
                &words,
                false,
                false,
                Steps::QueryItems {
                    meter: &meter,
                    file: &file,
                    line,
                },
            );
            (answer, meter.usage())
        };
        let (answer, usage) = run(usize::MAX);
        let expected = answer.expect("native admitted resolver");
        assert!(!expected.is_empty());
        assert!(usage.work > 0);
        assert!(usage.item_bytes > 0);
        assert_eq!(usage.glob_bytes, 0);
        assert_eq!(run(usage.work).0.expect("exact work"), expected);
        let error = run(usage.work - 1).0.expect_err("one work unit short");
        assert!(
            matches!(error.kind, LoadErrorKind::Budget { budget: Budget::Work, limit }
            if limit == usage.work - 1)
        );
        assert_eq!(error.file.as_str(), "src/lib.rs");
    }

    #[test]
    fn a_shared_files_edition_caches_are_each_counted_and_charged_once() {
        let tree = MemoryTree::from([
            ("Cargo.toml", "[workspace]\nmembers = [\"old\", \"new\"]\n"),
            (
                "old/Cargo.toml",
                "[package]\nname = \"old\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            (
                "new/Cargo.toml",
                "[package]\nname = \"new\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
            ),
            (
                "old/src/lib.rs",
                "#[path = \"../../shared.rs\"] mod shared;\n",
            ),
            (
                "new/src/lib.rs",
                "#[path = \"../../shared.rs\"] mod shared;\n",
            ),
            ("shared.rs", "pub fn f() { r#gen() }\nfn r#gen() {}\n"),
        ]);
        let ws = Workspace::load(&tree, &["."]).expect("shared source");
        let options = LoadOptions::default();
        let meter = Meter::new(&options);
        let files = ws
            .files()
            .map(|source| {
                (
                    source.path.clone(),
                    crate::SourceFile {
                        path: source.path.clone(),
                        package: source.package.clone(),
                        shipping: source.shipping,
                        test: source.test,
                        text: source.text.clone(),
                        facts: source.facts.clone(),
                        reached: source.reached.clone(),
                    },
                )
            })
            .collect();
        let item = |name| {
            ws.resolve_in(name, Namespace::Value)
                .expect("function")
                .sites[0]
        };
        let old = item("old::shared::f");
        let new = item("new::shared::f");
        let file = &ws.item(old).expect("old function").file;
        assert_eq!(file, &ws.item(new).expect("new function").file);
        let mut walk = Walk::new(ws.index(), &files, &meter, (file, 1)).expect("walk tables");
        let old_module = walk.homes[old.0].module.expect("old module");
        let new_module = walk.homes[new.0].module.expect("new module");
        let (_, first) = work::measure(usize::MAX, || {
            walk.load(file, old_module).expect("old cache")
        });
        let first_bytes = meter.usage().item_bytes;
        let (_, repeated) = work::measure(usize::MAX, || {
            walk.load(file, old_module).expect("old cache again")
        });
        assert!(first.get(Work::ReachTokens) > 0);
        assert_eq!(repeated.get(Work::ReachTokens), 0);
        assert_eq!(meter.usage().item_bytes, first_bytes);
        let (_, second) = work::measure(usize::MAX, || {
            walk.load(file, new_module).expect("new cache")
        });
        let both_bytes = meter.usage().item_bytes;
        assert_eq!(second.get(Work::ReachTokens), first.get(Work::ReachTokens));
        assert!(both_bytes > first_bytes);
        assert_eq!(walk.files_runs.len(), 2);
        let (_, repeated) = work::measure(usize::MAX, || {
            walk.load(file, new_module).expect("new cache again")
        });
        assert_eq!(repeated.get(Work::ReachTokens), 0);
        assert_eq!(meter.usage().item_bytes, both_bytes);

        let under = |limit| -> Result<usize, ReachError> {
            let options = LoadOptions {
                max_item_bytes: limit,
                ..LoadOptions::default()
            };
            let meter = Meter::new(&options);
            let mut walk = Walk::new(ws.index(), &files, &meter, (file, 1))?;
            walk.load(file, old_module)?;
            walk.load(file, new_module)?;
            Ok(meter.usage().item_bytes)
        };
        assert_eq!(
            under(both_bytes).expect("exact retained-byte limit"),
            both_bytes
        );
        let error = under(both_bytes - 1).expect_err("one byte short refuses second cache");
        assert!(
            matches!(error, ReachError::Budget(ref load) if matches!(load.kind,
            LoadErrorKind::Budget { budget: Budget::ItemBytes, limit } if limit == both_bytes - 1))
        );
    }
}
