// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Owned facts about one source file.
//!
//! The facts are read on the parse thread by [`extract`] and leave it as plain
//! data, so no syntax tree and no span outlives its thread. They hold what the
//! module walk needs (module declarations with their `path` attributes) and
//! what the symbol index needs (items, `use` trees and impl headers with
//! their positions, attributes, documentation and test role).

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::mem::size_of;
use std::ops::Range;
use std::sync::Arc;

use quote::ToTokens as _;
use syn::parse::Parser as _;
use syn::spanned::Spanned as _;
use syn::visit::Visit;
use syn::{Attribute, Expr, ExprLit, Item, Lit, Meta, Visibility};

use crate::cfg::{CfgExpr, parse_list};
use crate::meter::FactAllowance;
use crate::verbatim::{self, Bodiless, Verbatim};
use crate::work::{self, Work};

/// A region of the file in lines and bytes.
///
/// Line ranges are 1-based and half-open, as in `raf_okf::Span`: an item on
/// line `n` alone has the lines `n..n + 1`.
///
/// The struct is `#[non_exhaustive]`: adding a field is not a breaking change,
/// so a pattern names the fields it reads and ends in `..`.
///
/// ```compile_fail,E0638
/// # fn names(span: &raf_source::SpanFact) {
/// let raf_source::SpanFact { lines, start_byte, end_byte } = span;
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SpanFact {
    /// The lines the region covers: from its first line to the line after
    /// its last.
    pub lines: Range<usize>,
    /// The byte offset of the start.
    pub start_byte: usize,
    /// The byte offset just past the end.
    pub end_byte: usize,
}

/// Whether, and how, an item belongs to test code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum TestRole {
    /// Not test code by its own attributes or its enclosing modules.
    None,
    /// Guarded by a `cfg` that implies `test`, directly or by an enclosing
    /// module.
    CfgTest,
    /// A function marked `#[test]`.
    TestFn,
}

/// The declared visibility of an item.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Vis {
    /// No visibility keyword.
    Inherited,
    /// `pub`.
    Public,
    /// `pub(crate)`, `pub(super)`, `pub(self)` or `pub(in path)`, as written
    /// inside the parentheses.
    Restricted(String),
}

/// What a named item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemKind {
    /// A function or a method.
    Fn,
    /// A struct.
    Struct,
    /// An enum.
    Enum,
    /// A union.
    Union,
    /// A trait.
    Trait,
    /// A type alias or an associated type.
    TypeAlias,
    /// A constant or an associated constant.
    Const,
    /// `const _: T = ..;`, which defines no name. It is recorded so that
    /// nothing is dropped silently, and is not indexed under a path.
    UnnamedConst,
    /// A static.
    Static,
    /// A `macro_rules!` definition.
    MacroRules,
    /// An `extern crate` item.
    ExternCrate,
    /// A function declared in an `extern` block.
    ForeignFn,
    /// A static declared in an `extern` block.
    ForeignStatic,
    /// A type declared in an `extern` block.
    ForeignType,
}

/// A plain path as written: its segments without raw-identifier prefixes and
/// whether it starts with `::`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PathFact {
    /// The segments, generic arguments left out.
    pub(crate) segments: Vec<String>,
    /// True when the path starts with `::`.
    pub(crate) leading_colon: bool,
}

/// How a struct declares its fields. A unit or tuple struct also defines a
/// name in the value namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FieldShape {
    /// `struct S;`.
    Unit,
    /// `struct S(..);`.
    Tuple,
    /// `struct S { .. }`.
    Named,
}

/// An attribute as written: its path and the tokens after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AttrFact {
    /// The attribute path, such as `derive` or `cfg`.
    pub(crate) path: String,
    /// The rest of the attribute, such as `(Debug)` or `= "text"`; empty when
    /// there is none.
    pub(crate) args: String,
}

/// What the parameters of a function and the value of a constant add to the
/// item, shared by every record of the item.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ItemDetail {
    /// The names the parameters bind, in signature order, with `self` first
    /// for a receiver.
    pub(crate) bindings: Vec<String>,
    /// The byte range between the `=` and the `;` of a constant or a static
    /// that has a value.
    pub(crate) initializer: Option<Range<usize>>,
}

/// A named item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ItemFact {
    /// The name without a raw-identifier prefix. For `extern crate x as y`
    /// this is `y`.
    pub(crate) name: String,
    /// The original crate name of `extern crate x as y`.
    pub(crate) renamed_from: Option<String>,
    /// What the item is.
    pub(crate) kind: ItemKind,
    /// The 1-based line of the name.
    pub(crate) name_line: usize,
    /// The whole item, attributes included.
    pub(crate) span: SpanFact,
    /// The braced body, or the function block, when there is one.
    pub(crate) body: Option<SpanFact>,
    /// What a signature or a constant adds; `None` when it adds nothing.
    pub(crate) detail: Option<Arc<ItemDetail>>,
    /// The declared visibility.
    pub(crate) vis: Vis,
    /// Every attribute, in order.
    pub(crate) attrs: Vec<AttrFact>,
    /// True when an unconditional `#[doc]` attribute (or a doc comment) holds
    /// text, or is `#[doc = include_str!(..)]`. `#[doc(hidden)]` alone does
    /// not count.
    pub(crate) documented: bool,
    /// True when the item has `#[doc(hidden)]`.
    pub(crate) doc_hidden: bool,
    /// The test role.
    pub(crate) test: TestRole,
    /// True when the item has `#[macro_export]`.
    pub(crate) macro_export: bool,
    /// The items of a trait, in order.
    pub(crate) members: Vec<ItemFact>,
    /// The field shape of a struct; `None` for every other kind.
    pub(crate) shape: Option<FieldShape>,
    /// The aliased path of `type A = path::B;` when the right side is a plain
    /// path; `None` for every other item.
    pub(crate) alias: Option<PathFact>,
}

/// The `path` attributes of a module declaration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct PathAttrs {
    /// Every `#[path = "..."]` that applies unconditionally.
    pub(crate) unconditional: Vec<String>,
    /// Every `path` set by `#[cfg_attr(predicate, path = "...")]`.
    pub(crate) conditional: Vec<ConditionalPath>,
}

/// A `path` that applies when a predicate holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConditionalPath {
    /// The `cfg_attr` predicate, as written.
    pub(crate) predicate: String,
    /// True when the predicate holds only in a test build.
    pub(crate) implies_test: bool,
    /// The path value.
    pub(crate) path: String,
}

/// A module declaration: `mod name;` or `mod name { .. }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModFact {
    /// The name without a raw-identifier prefix.
    pub(crate) name: String,
    /// The 1-based line of the name.
    pub(crate) name_line: usize,
    /// The whole declaration, attributes included.
    pub(crate) span: SpanFact,
    /// The braced body of an inline module.
    pub(crate) body_span: Option<SpanFact>,
    /// The declared visibility.
    pub(crate) vis: Vis,
    /// Every attribute, in order.
    pub(crate) attrs: Vec<AttrFact>,
    /// True when the module is documented.
    pub(crate) documented: bool,
    /// True when the module has `#[doc(hidden)]`.
    pub(crate) doc_hidden: bool,
    /// The test role of the module itself.
    pub(crate) test: TestRole,
    /// The `path` attributes.
    pub(crate) paths: PathAttrs,
    /// The contents of an inline module; `None` for `mod name;`.
    pub(crate) inline: Option<ModBody>,
}

/// One `use` tree, owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UseTree {
    /// `name::rest`.
    Path {
        /// The segment.
        name: String,
        /// What follows it.
        rest: Box<UseTree>,
    },
    /// `name`.
    Name(String),
    /// `name as alias`.
    Rename {
        /// The imported name.
        name: String,
        /// The local name.
        alias: String,
    },
    /// `*`.
    Glob,
    /// `{a, b::c}`.
    Group(Vec<UseTree>),
}

/// A `use` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UseFact {
    /// The 1-based line of the `use` keyword.
    pub(crate) line: usize,
    /// The whole declaration.
    pub(crate) span: SpanFact,
    /// The declared visibility (a re-export when not inherited).
    pub(crate) vis: Vis,
    /// True for `use ::name`.
    pub(crate) leading_colon: bool,
    /// The tree.
    pub(crate) tree: UseTree,
    /// The test role.
    pub(crate) test: TestRole,
    /// True when the declaration has `#[doc(hidden)]`.
    pub(crate) doc_hidden: bool,
}

/// An `impl` block header and its items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImplFact {
    /// The 1-based line of the `impl` keyword.
    pub(crate) line: usize,
    /// The whole block.
    pub(crate) span: SpanFact,
    /// The braced body.
    pub(crate) body: SpanFact,
    /// The trait path as written, for a trait impl.
    pub(crate) trait_path: Option<String>,
    /// The segments of the trait path, for a trait impl.
    pub(crate) trait_segments: Option<PathFact>,
    /// The self type as written.
    pub(crate) self_type: String,
    /// The segments of the path the self type is written with: a plain path,
    /// or the first trait of a trait object (`dyn Tr + Send` gives `Tr`),
    /// inside any parentheses. `None` for every other type.
    pub(crate) self_path: Option<Vec<String>>,
    /// True when that path starts with `::`, so it names an external crate
    /// and not a local item.
    pub(crate) self_leading_colon: bool,
    /// The names of the generic type and const parameters of the block.
    pub(crate) generics: Vec<String>,
    /// The associated items.
    pub(crate) items: Vec<ItemFact>,
    /// The test role.
    pub(crate) test: TestRole,
}

/// The contents of a module, in source order within each list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ModBody {
    /// Named items other than modules, `use` declarations and impls.
    pub(crate) items: Vec<ItemFact>,
    /// Module declarations.
    pub(crate) mods: Vec<ModFact>,
    /// `use` declarations.
    pub(crate) uses: Vec<UseFact>,
    /// `impl` blocks.
    pub(crate) impls: Vec<ImplFact>,
}

/// The facts of one file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct FileFacts {
    /// The file's own module.
    pub(crate) body: ModBody,
    /// True when the file starts with documentation (`//!` or `#![doc = ..]`),
    /// which documents the module that the file is.
    pub(crate) documented: bool,
    /// True when the file has `#![doc(hidden)]`.
    pub(crate) doc_hidden: bool,
    /// The inner attributes of the file, in order.
    pub(crate) attrs: Vec<AttrFact>,
}

/// Why the facts of a file could not be read.
#[derive(Debug)]
pub(crate) enum ExtractError {
    /// A `cfg` or `cfg_attr` predicate is not readable.
    Cfg { line: usize, message: String },
    /// A `path` attribute is not `path = "literal"`: on the module `module`
    /// when it is on a module, under the `cfg_attr` predicate `predicate`
    /// when one guards it.
    PathAttr {
        line: usize,
        module: Option<String>,
        predicate: Option<String>,
        message: String,
    },
    /// An item the index does not model.
    Unsupported { line: usize, what: String },
    /// A position outside the text.
    Position { line: usize },
    /// The facts passed the fact-bytes budget of the load.
    Budget { line: usize, limit: usize },
}

type Found<T> = Result<T, ExtractError>;

/// Characters between two marks of a long line's index.
const MARK_EVERY: usize = 64;

/// How a long line turns a column into a byte offset.
enum LineIndex {
    /// Every character is one byte.
    Ascii,
    /// The byte offset, from the line start, of every [`MARK_EVERY`]th
    /// character.
    Marks(Vec<usize>),
}

/// Line starts of the text, for turning positions into byte offsets.
///
/// A column counts characters, so a line with a character of more than one
/// byte is walked. A long line is indexed once, the first time it is asked
/// about, so the cost of a position does not grow with the length of its line.
struct Positions<'t> {
    text: &'t str,
    starts: Vec<usize>,
    long: RefCell<HashMap<usize, LineIndex>>,
}

impl<'t> Positions<'t> {
    fn new(text: &'t str) -> Self {
        // The parser reads the text without a byte-order mark, so its columns
        // on the first line count from the character after it.
        let mut starts = vec![
            text.strip_prefix('\u{feff}')
                .map_or(0, |_| '\u{feff}'.len_utf8()),
        ];
        starts.extend(text.match_indices('\n').map(|(at, _)| at + 1));
        Self {
            text,
            starts,
            long: RefCell::new(HashMap::new()),
        }
    }

    fn byte(&self, line: usize, column: usize) -> Found<usize> {
        work::add(Work::Positions, 1);
        let outside = || ExtractError::Position { line };
        let index = line.checked_sub(1).ok_or_else(outside)?;
        let start = *self.starts.get(index).ok_or_else(outside)?;
        let end = self
            .starts
            .get(index + 1)
            .copied()
            .unwrap_or(self.text.len());
        let own = self.text.get(start..end).ok_or_else(outside)?;
        let (base, rest, skip) = if own.len() <= MARK_EVERY {
            (0, own, column)
        } else {
            let mut long = self.long.borrow_mut();
            let entry = long.entry(index).or_insert_with(|| {
                if own.is_ascii() {
                    work::add(Work::ColumnSteps, own.len());
                    LineIndex::Ascii
                } else {
                    LineIndex::Marks(
                        own.char_indices()
                            .inspect(|_| work::add(Work::ColumnSteps, 1))
                            .step_by(MARK_EVERY)
                            .map(|(at, _)| at)
                            .collect(),
                    )
                }
            });
            match entry {
                LineIndex::Ascii => {
                    return if column <= own.len() {
                        Ok(start + column)
                    } else {
                        Err(outside())
                    };
                }
                LineIndex::Marks(marks) => {
                    let mark = (column / MARK_EVERY).min(marks.len().saturating_sub(1));
                    let base = marks.get(mark).copied().unwrap_or(0);
                    let rest = own.get(base..).ok_or_else(outside)?;
                    (base, rest, column - mark * MARK_EVERY)
                }
            }
        };
        let mut chars = rest
            .char_indices()
            .inspect(|_| work::add(Work::ColumnSteps, 1));
        if let Some((offset, _)) = chars.nth(skip) {
            return Ok(start + base + offset);
        }
        let all = rest.chars().count();
        work::add(Work::ColumnSteps, all);
        // Just past the last character of the line.
        if all == skip {
            Ok(start + base + rest.len())
        } else {
            Err(outside())
        }
    }

    fn span(&self, span: proc_macro2::Span) -> Found<SpanFact> {
        let (start, end) = (span.start(), span.end());
        Ok(SpanFact {
            lines: start.line..end.line + 1,
            start_byte: self.byte(start.line, start.column)?,
            end_byte: self.byte(end.line, end.column)?,
        })
    }
}

/// Every name a parameter pattern binds, in source order. Types are not
/// visited, so a path in a type is not a binding.
#[derive(Default)]
struct Bindings(Vec<String>);

impl<'ast> Visit<'ast> for Bindings {
    fn visit_pat_ident(&mut self, pat: &'ast syn::PatIdent) {
        self.0.push(pat.ident.to_string());
        syn::visit::visit_pat_ident(self, pat);
    }

    fn visit_type(&mut self, _: &'ast syn::Type) {}
}

/// The fact bytes a file takes, charged on its parse thread as each fact is
/// built: its fixed size and the length of every string and list it owns.
struct FactMeter {
    allowance: FactAllowance,
    spent: Cell<usize>,
}

impl FactMeter {
    /// A meter for attributes that are read for their meaning and not kept,
    /// whose facts are dropped at once.
    fn scratch() -> Self {
        Self {
            allowance: FactAllowance {
                spent: 0,
                limit: usize::MAX,
            },
            spent: Cell::new(0),
        }
    }

    /// Charge `bytes` for a fact built at `line`.
    fn charge(&self, bytes: usize, line: usize) -> Found<()> {
        let spent = self.spent.get().saturating_add(bytes);
        self.spent.set(spent);
        if self.allowance.spent.saturating_add(spent) > self.allowance.limit {
            Err(ExtractError::Budget {
                line,
                limit: self.allowance.limit,
            })
        } else {
            Ok(())
        }
    }
}

/// The bytes a list of strings owns beyond its own header.
fn strings_bytes(strings: &[String]) -> usize {
    strings
        .iter()
        .map(|text| size_of::<String>() + text.capacity())
        .sum()
}

/// The bytes a visibility owns beyond its own size.
fn vis_bytes(vis: &Vis) -> usize {
    match vis {
        Vis::Restricted(text) => text.capacity(),
        Vis::Inherited | Vis::Public => 0,
    }
}

/// A string that holds no room beyond its text, so what it is charged is
/// what it holds.
fn fit(mut text: String) -> String {
    text.shrink_to_fit();
    text
}

/// Trim a list to its length and give the bytes of room it still holds beyond
/// the elements, which are charged one by one.
fn slack<T>(list: &mut Vec<T>) -> usize {
    list.shrink_to_fit();
    (list.capacity() - list.len()) * size_of::<T>()
}

fn unraw(ident: &syn::Ident) -> String {
    let text = ident.to_string();
    fit(match text.strip_prefix("r#") {
        Some(plain) => plain.to_owned(),
        None => text,
    })
}

fn vis(visibility: &Visibility) -> Vis {
    match visibility {
        Visibility::Public(_) => Vis::Public,
        Visibility::Restricted(restricted) => {
            let path = restricted.path.to_token_stream().to_string();
            Vis::Restricted(fit(if restricted.in_token.is_some() {
                format!("in {path}")
            } else {
                path
            }))
        }
        Visibility::Inherited => Vis::Inherited,
    }
}

/// What the attributes of an item say.
struct AttrSummary {
    facts: Vec<AttrFact>,
    documented: bool,
    doc_hidden: bool,
    own_test: TestRole,
    macro_export: bool,
    paths: PathAttrs,
}

fn string_value(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => Some(value.value()),
        _ => None,
    }
}

fn path_text(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn cfg_error(line: usize, message: String) -> ExtractError {
    ExtractError::Cfg { line, message }
}

/// A `path` attribute that cannot be read, under `predicate` when a
/// `cfg_attr` applies it. The module it is on is filled in by the module's
/// own fact.
fn path_error(line: usize, predicate: Option<&CfgExpr>, message: String) -> ExtractError {
    ExtractError::PathAttr {
        line,
        module: None,
        predicate: predicate.map(ToString::to_string),
        message,
    }
}

/// The `path` attribute value, which must be a string literal.
fn path_value(expr: &Expr, line: usize, predicate: Option<&CfgExpr>) -> Found<String> {
    string_value(expr).ok_or_else(|| {
        let written: String = expr
            .to_token_stream()
            .to_string()
            .chars()
            .take(60)
            .collect();
        path_error(
            line,
            predicate,
            format!("the value of `path` is not a string literal: `{written}`"),
        )
    })
}

/// Read the meta item inside `cfg(..)`.
fn cfg_predicate(tokens: &proc_macro2::TokenStream, line: usize) -> Found<CfgExpr> {
    let meta = crate::cfg::parse_meta
        .parse2(tokens.clone())
        .map_err(|error| cfg_error(line, error.to_string()))?;
    CfgExpr::from_meta(&meta).map_err(|message| cfg_error(line, message))
}

/// Read `cfg_attr(predicate, attributes..)`, flattening nested `cfg_attr`
/// into the combined predicate `all(outer, inner)`.
fn apply_cfg_attr(
    tokens: &proc_macro2::TokenStream,
    outer: Option<&CfgExpr>,
    summary: &mut AttrSummary,
    line: usize,
    meter: &FactMeter,
) -> Found<()> {
    let metas = parse_list(tokens).map_err(|message| cfg_error(line, message))?;
    let Some((first, applied)) = metas.split_first() else {
        return Err(cfg_error(line, "`cfg_attr` has no predicate".to_owned()));
    };
    let own = CfgExpr::from_meta(first).map_err(|message| cfg_error(line, message))?;
    let combined = match outer {
        Some(outer) => CfgExpr::All(vec![outer.clone(), own]),
        None => own,
    };
    for meta in applied {
        match meta {
            Meta::NameValue(pair) if pair.path.is_ident("path") => {
                let conditional = ConditionalPath {
                    predicate: fit(combined.to_string()),
                    implies_test: combined.implies_test(),
                    path: fit(path_value(&pair.value, line, Some(&combined))?),
                };
                meter.charge(
                    size_of::<ConditionalPath>()
                        + conditional.predicate.capacity()
                        + conditional.path.capacity(),
                    line,
                )?;
                summary.paths.conditional.push(conditional);
            }
            Meta::List(list) if list.path.is_ident("cfg_attr") => {
                apply_cfg_attr(&list.tokens, Some(&combined), summary, line, meter)?;
            }
            Meta::List(list) if list.path.is_ident("cfg") => {
                let inner = cfg_predicate(&list.tokens, line)?;
                let both = CfgExpr::All(vec![combined.clone(), inner]);
                if both.implies_test() && summary.own_test != TestRole::TestFn {
                    summary.own_test = TestRole::CfgTest;
                }
            }
            Meta::Path(path) if path.is_ident("test") => summary.own_test = TestRole::TestFn,
            // Any other attribute that `cfg_attr` applies is kept in the
            // arguments of the `cfg_attr` fact and changes nothing the index
            // records.
            Meta::Path(_) | Meta::List(_) | Meta::NameValue(_) => {}
        }
    }
    Ok(())
}

/// True for `include_str!(..)` and its qualified spellings.
fn is_include_str(expr: &Expr) -> bool {
    matches!(expr, Expr::Macro(call) if call.mac.path.segments.last().is_some_and(|s| s.ident == "include_str"))
}

/// Read the attributes of an item, charging each fact to `meter`.
fn summarize(attrs: &[Attribute], meter: &FactMeter) -> Found<AttrSummary> {
    let mut summary = AttrSummary {
        facts: Vec::new(),
        documented: false,
        doc_hidden: false,
        own_test: TestRole::None,
        macro_export: false,
        paths: PathAttrs::default(),
    };
    for attr in attrs {
        work::add(Work::Attributes, 1);
        let line = attr.span().start().line;
        let path = fit(path_text(attr.path()));
        let args = fit(match &attr.meta {
            Meta::Path(_) => String::new(),
            Meta::List(list) => format!("({})", list.tokens),
            Meta::NameValue(pair) => format!("= {}", pair.value.to_token_stream()),
        });
        meter.charge(
            size_of::<AttrFact>() + path.capacity() + args.capacity(),
            line,
        )?;
        summary.facts.push(AttrFact {
            path: path.clone(),
            args,
        });
        match (path.as_str(), &attr.meta) {
            ("doc", Meta::NameValue(pair)) => {
                // `include_str!` documentation is non-blank by construction.
                if is_include_str(&pair.value)
                    || string_value(&pair.value).is_some_and(|text| !text.trim().is_empty())
                {
                    summary.documented = true;
                }
            }
            ("doc", Meta::List(list)) => {
                if list.tokens.to_string().trim() == "hidden" {
                    summary.doc_hidden = true;
                }
            }
            ("macro_export", _) => summary.macro_export = true,
            ("test", Meta::Path(_)) => summary.own_test = TestRole::TestFn,
            ("test", _) => {
                return Err(cfg_error(line, "`test` takes no arguments".to_owned()));
            }
            ("path", Meta::NameValue(pair)) => {
                let value = fit(path_value(&pair.value, line, None)?);
                meter.charge(size_of::<String>() + value.capacity(), line)?;
                summary.paths.unconditional.push(value);
            }
            ("path", _) => {
                return Err(path_error(
                    line,
                    None,
                    "`path` must be `path = \"..\"`".to_owned(),
                ));
            }
            ("cfg", Meta::List(list)) => {
                let predicate = cfg_predicate(&list.tokens, line)?;
                if predicate.implies_test() && summary.own_test != TestRole::TestFn {
                    summary.own_test = TestRole::CfgTest;
                }
            }
            ("cfg", _) => {
                return Err(cfg_error(line, "`cfg` must be `cfg(predicate)`".to_owned()));
            }
            ("cfg_attr", Meta::List(list)) => {
                apply_cfg_attr(&list.tokens, None, &mut summary, line, meter)?;
            }
            ("cfg_attr", _) => {
                return Err(cfg_error(
                    line,
                    "`cfg_attr` must be `cfg_attr(predicate, attributes)`".to_owned(),
                ));
            }
            // Every other attribute is kept as written in `facts`; none of
            // them changes what the index records.
            _ => {}
        }
    }
    let line = attrs.last().map_or(1, |attr| attr.span().start().line);
    let room = slack(&mut summary.facts)
        + slack(&mut summary.paths.unconditional)
        + slack(&mut summary.paths.conditional);
    meter.charge(room, line)?;
    Ok(summary)
}

/// Combine an item's own role with the role its enclosing modules give it.
fn role(own: TestRole, inherited: TestRole) -> TestRole {
    if own == TestRole::TestFn {
        TestRole::TestFn
    } else if own == TestRole::CfgTest || inherited != TestRole::None {
        TestRole::CfgTest
    } else {
        TestRole::None
    }
}

/// The path a self type is written with: a plain path, or the first trait
/// of a trait object (`dyn Tr + Send` is written with `Tr`), inside any
/// parentheses. `None` for every other type.
fn self_path_of(ty: &syn::Type) -> Option<&syn::Path> {
    let mut ty = ty;
    // Parentheses nest no deeper than the pre-scan's delimiter budget.
    loop {
        match ty {
            syn::Type::Paren(inner) => ty = &inner.elem,
            syn::Type::Group(inner) => ty = &inner.elem,
            syn::Type::Path(path) if path.qself.is_none() => return Some(&path.path),
            syn::Type::TraitObject(object) => {
                return object.bounds.iter().find_map(|bound| match bound {
                    syn::TypeParamBound::Trait(found) => Some(&found.path),
                    _ => None,
                });
            }
            _ => return None,
        }
    }
}

/// The refusal of an item the index does not model, naming its first line
/// and the start of its text.
fn unsupported(item: &impl quote::ToTokens) -> ExtractError {
    ExtractError::Unsupported {
        line: item.span().start().line,
        what: item
            .to_token_stream()
            .to_string()
            .chars()
            .take(60)
            .collect(),
    }
}

/// The test role of a `use` declaration with the attributes `attrs`, and
/// whether they hold `#[doc(hidden)]`; the attributes are read for their
/// meaning and not kept.
fn use_role(attrs: &[Attribute], inherited: TestRole) -> Found<UseRole> {
    let summary = summarize(attrs, &FactMeter::scratch())?;
    Ok(UseRole {
        test: role(summary.own_test, inherited),
        doc_hidden: summary.doc_hidden,
    })
}

/// What the attributes of a `use` declaration say of it.
#[derive(Debug, Clone, Copy)]
struct UseRole {
    test: TestRole,
    doc_hidden: bool,
}

struct Extractor<'t> {
    positions: Positions<'t>,
    meter: FactMeter,
}

impl Extractor<'_> {
    #[allow(
        clippy::too_many_arguments,
        reason = "every field of an item fact comes from the syntax node separately"
    )]
    fn item(
        &self,
        name: &syn::Ident,
        kind: ItemKind,
        vis_of: &Visibility,
        attrs: &[Attribute],
        whole: proc_macro2::Span,
        body: Option<proc_macro2::Span>,
        inherited: TestRole,
        members: Vec<ItemFact>,
    ) -> Found<ItemFact> {
        let name_line = name.span().start().line;
        let summary = summarize(attrs, &self.meter)?;
        let name = unraw(name);
        let vis = vis(vis_of);
        self.meter.charge(
            size_of::<ItemFact>() + name.capacity() + vis_bytes(&vis),
            name_line,
        )?;
        Ok(ItemFact {
            name,
            renamed_from: None,
            kind,
            name_line,
            span: self.positions.span(whole)?,
            body: body.map(|span| self.positions.span(span)).transpose()?,
            detail: None,
            vis,
            attrs: summary.facts,
            documented: summary.documented,
            doc_hidden: summary.doc_hidden,
            test: role(summary.own_test, inherited),
            macro_export: summary.macro_export,
            members,
            shape: None,
            alias: None,
        })
    }

    /// `fact` with the parameter bindings of `sig`.
    fn with_bindings(&self, mut fact: ItemFact, sig: &syn::Signature) -> Found<ItemFact> {
        let mut names = Bindings::default();
        for input in &sig.inputs {
            match input {
                syn::FnArg::Receiver(_) => names.0.push("self".to_owned()),
                syn::FnArg::Typed(typed) => names.visit_pat(&typed.pat),
            }
        }
        if !names.0.is_empty() {
            self.meter.charge(
                size_of::<ItemDetail>() + strings_bytes(&names.0),
                fact.name_line,
            )?;
            fact.detail = Some(Arc::new(ItemDetail {
                bindings: names.0,
                initializer: None,
            }));
        }
        Ok(fact)
    }

    /// `fact` with the range between `eq` and `semi` as its initializer.
    fn with_initializer(
        &self,
        mut fact: ItemFact,
        eq: proc_macro2::Span,
        semi: proc_macro2::Span,
    ) -> Found<ItemFact> {
        let start = self.positions.span(eq)?.end_byte;
        let end = self.positions.span(semi)?.start_byte;
        self.meter.charge(size_of::<ItemDetail>(), fact.name_line)?;
        fact.detail = Some(Arc::new(ItemDetail {
            bindings: Vec::new(),
            initializer: Some(start..end),
        }));
        Ok(fact)
    }

    fn module(&self, items: &[Item], inherited: TestRole, line: usize) -> Found<ModBody> {
        let mut body = ModBody::default();
        for item in items {
            self.one(item, inherited, &mut body)?;
        }
        let room = slack(&mut body.items)
            + slack(&mut body.mods)
            + slack(&mut body.uses)
            + slack(&mut body.impls);
        self.meter.charge(room, line)?;
        Ok(body)
    }

    fn struct_fact(
        &self,
        s: &syn::ItemStruct,
        whole: proc_macro2::Span,
        inherited: TestRole,
    ) -> Found<ItemFact> {
        let (braces, shape) = match &s.fields {
            syn::Fields::Named(named) => (Some(named.brace_token.span.join()), FieldShape::Named),
            syn::Fields::Unnamed(_) => (None, FieldShape::Tuple),
            syn::Fields::Unit => (None, FieldShape::Unit),
        };
        let mut fact = self.item(
            &s.ident,
            ItemKind::Struct,
            &s.vis,
            &s.attrs,
            whole,
            braces,
            inherited,
            Vec::new(),
        )?;
        fact.shape = Some(shape);
        Ok(fact)
    }

    fn union_fact(
        &self,
        u: &syn::ItemUnion,
        whole: proc_macro2::Span,
        inherited: TestRole,
    ) -> Found<ItemFact> {
        let mut fact = self.item(
            &u.ident,
            ItemKind::Union,
            &u.vis,
            &u.attrs,
            whole,
            Some(u.fields.brace_token.span.join()),
            inherited,
            Vec::new(),
        )?;
        fact.shape = Some(FieldShape::Named);
        Ok(fact)
    }

    fn trait_fact(
        &self,
        t: &syn::ItemTrait,
        whole: proc_macro2::Span,
        inherited: TestRole,
    ) -> Found<ItemFact> {
        let own = summarize(&t.attrs, &FactMeter::scratch())?;
        let inside = role(own.own_test, inherited);
        let mut members = Vec::new();
        for member in &t.items {
            if let Some(fact) = self.trait_member(member, inside)? {
                members.push(fact);
            }
        }
        self.meter
            .charge(slack(&mut members), t.ident.span().start().line)?;
        self.item(
            &t.ident,
            ItemKind::Trait,
            &t.vis,
            &t.attrs,
            whole,
            Some(t.brace_token.span.join()),
            inherited,
            members,
        )
    }

    /// The items that are one named item with no further structure.
    fn named(&self, item: &Item, inherited: TestRole) -> Found<Option<ItemFact>> {
        let whole = item.span();
        let fact = match item {
            Item::Fn(f) => {
                let fact = self.item(
                    &f.sig.ident,
                    ItemKind::Fn,
                    &f.vis,
                    &f.attrs,
                    whole,
                    Some(f.block.brace_token.span.join()),
                    inherited,
                    Vec::new(),
                )?;
                self.with_bindings(fact, &f.sig)?
            }
            Item::Struct(s) => self.struct_fact(s, whole, inherited)?,
            Item::Enum(e) => self.item(
                &e.ident,
                ItemKind::Enum,
                &e.vis,
                &e.attrs,
                whole,
                Some(e.brace_token.span.join()),
                inherited,
                Vec::new(),
            )?,
            Item::Union(u) => self.union_fact(u, whole, inherited)?,
            Item::Trait(t) => self.trait_fact(t, whole, inherited)?,
            Item::Type(t) => {
                let mut fact = self.item(
                    &t.ident,
                    ItemKind::TypeAlias,
                    &t.vis,
                    &t.attrs,
                    whole,
                    None,
                    inherited,
                    Vec::new(),
                )?;
                fact.alias = match &*t.ty {
                    syn::Type::Path(path) if path.qself.is_none() => Some(PathFact {
                        segments: path.path.segments.iter().map(|s| unraw(&s.ident)).collect(),
                        leading_colon: path.path.leading_colon.is_some(),
                    }),
                    _ => None,
                };
                if let Some(alias) = &fact.alias {
                    self.meter
                        .charge(strings_bytes(&alias.segments), fact.name_line)?;
                }
                fact
            }
            Item::Const(c) => {
                let fact = self.item(
                    &c.ident,
                    if c.ident == "_" {
                        ItemKind::UnnamedConst
                    } else {
                        ItemKind::Const
                    },
                    &c.vis,
                    &c.attrs,
                    whole,
                    None,
                    inherited,
                    Vec::new(),
                )?;
                self.with_initializer(fact, c.eq_token.span(), c.semi_token.span())?
            }
            Item::Static(s) => {
                let fact = self.item(
                    &s.ident,
                    ItemKind::Static,
                    &s.vis,
                    &s.attrs,
                    whole,
                    None,
                    inherited,
                    Vec::new(),
                )?;
                self.with_initializer(fact, s.eq_token.span(), s.semi_token.span())?
            }
            // Modules, impls, `use`, macros and `extern` items are read by the
            // caller.
            _ => return Ok(None),
        };
        Ok(Some(fact))
    }

    fn one(&self, item: &Item, inherited: TestRole, body: &mut ModBody) -> Found<()> {
        if let Some(fact) = self.named(item, inherited)? {
            body.items.push(fact);
            return Ok(());
        }
        let whole = item.span();
        match item {
            Item::Macro(m) => {
                // A macro invocation at item level is not expanded and defines
                // no name the index can see; only `macro_rules!` names one.
                if let Some(ident) = &m.ident {
                    body.items.push(self.item(
                        ident,
                        ItemKind::MacroRules,
                        &Visibility::Inherited,
                        &m.attrs,
                        whole,
                        Some(m.mac.delimiter.span().join()),
                        inherited,
                        Vec::new(),
                    )?);
                }
            }
            Item::ExternCrate(e) => {
                let mut fact = self.item(
                    e.rename.as_ref().map_or(&e.ident, |(_, alias)| alias),
                    ItemKind::ExternCrate,
                    &e.vis,
                    &e.attrs,
                    whole,
                    None,
                    inherited,
                    Vec::new(),
                )?;
                if e.rename.is_some() {
                    let original = unraw(&e.ident);
                    self.meter.charge(original.capacity(), fact.name_line)?;
                    fact.renamed_from = Some(original);
                }
                body.items.push(fact);
            }
            Item::ForeignMod(block) => {
                let own = summarize(&block.attrs, &FactMeter::scratch())?;
                let inside = role(own.own_test, inherited);
                for foreign in &block.items {
                    self.foreign(foreign, inside, body)?;
                }
            }
            Item::Mod(m) => body.mods.push(self.module_fact(m, inherited)?),
            Item::Impl(i) => body.impls.push(self.impl_fact(i, inherited)?),
            Item::Use(u) => {
                let role = use_role(&u.attrs, inherited)?;
                body.uses.push(self.use_fact(
                    role,
                    &u.vis,
                    u.leading_colon.is_some(),
                    &u.tree,
                    whole,
                )?);
            }
            Item::Verbatim(tokens) => match verbatim::item(tokens) {
                // Each member is a `use` of its own, spanning the whole
                // declaration, as the separate declarations would bind.
                Some(Verbatim::UseGroup(group)) => {
                    // The attributes of the whole declaration are read once,
                    // so the work stays linear in the text however many
                    // members share them.
                    let role = use_role(&group.attrs, inherited)?;
                    for member in &group.members {
                        body.uses.push(self.use_fact(
                            role,
                            &group.vis,
                            member.leading_colon,
                            &member.tree,
                            whole,
                        )?);
                    }
                }
                Some(Verbatim::Bodiless(read)) => {
                    body.items.push(self.bodiless(&read, whole, inherited)?);
                }
                None => return Err(unsupported(item)),
            },
            other => return Err(unsupported(other)),
        }
        Ok(())
    }

    /// The fact of one `use` declaration spanning `whole`, of the test role
    /// and the `doc(hidden)` flag that [`use_role`] read from its attributes.
    fn use_fact(
        &self,
        role: UseRole,
        vis_of: &Visibility,
        leading_colon: bool,
        tree: &syn::UseTree,
        whole: proc_macro2::Span,
    ) -> Found<UseFact> {
        let line = whole.start().line;
        let vis = vis(vis_of);
        self.meter
            .charge(size_of::<UseFact>() + vis_bytes(&vis), line)?;
        Ok(UseFact {
            line,
            span: self.positions.span(whole)?,
            vis,
            leading_colon,
            tree: self.use_tree(tree, line)?,
            test: role.test,
            doc_hidden: role.doc_hidden,
        })
    }

    /// The fact of a function with no body or a type with no definition, read
    /// from verbatim tokens: a function or a type alias with no body span.
    fn bodiless(
        &self,
        read: &Bodiless,
        whole: proc_macro2::Span,
        inherited: TestRole,
    ) -> Found<ItemFact> {
        match read {
            Bodiless::Fn(f) => {
                let fact = self.item(
                    &f.sig.ident,
                    ItemKind::Fn,
                    &f.vis,
                    &f.attrs,
                    whole,
                    None,
                    inherited,
                    Vec::new(),
                )?;
                self.with_bindings(fact, &f.sig)
            }
            Bodiless::Type(t) => self.item(
                &t.ident,
                ItemKind::TypeAlias,
                &t.vis,
                &t.attrs,
                whole,
                None,
                inherited,
                Vec::new(),
            ),
        }
    }

    fn foreign(
        &self,
        item: &syn::ForeignItem,
        inherited: TestRole,
        body: &mut ModBody,
    ) -> Found<()> {
        let whole = item.span();
        match item {
            syn::ForeignItem::Fn(f) => body.items.push(self.item(
                &f.sig.ident,
                ItemKind::ForeignFn,
                &f.vis,
                &f.attrs,
                whole,
                None,
                inherited,
                Vec::new(),
            )?),
            syn::ForeignItem::Static(s) => body.items.push(self.item(
                &s.ident,
                ItemKind::ForeignStatic,
                &s.vis,
                &s.attrs,
                whole,
                None,
                inherited,
                Vec::new(),
            )?),
            syn::ForeignItem::Type(t) => body.items.push(self.item(
                &t.ident,
                ItemKind::ForeignType,
                &t.vis,
                &t.attrs,
                whole,
                None,
                inherited,
                Vec::new(),
            )?),
            // A macro invocation inside an `extern` block names nothing the
            // index can see.
            syn::ForeignItem::Macro(_) => {}
            other => return Err(unsupported(other)),
        }
        Ok(())
    }

    fn trait_member(
        &self,
        member: &syn::TraitItem,
        inherited: TestRole,
    ) -> Found<Option<ItemFact>> {
        let whole = member.span();
        let fact = match member {
            syn::TraitItem::Fn(f) => self
                .item(
                    &f.sig.ident,
                    ItemKind::Fn,
                    &Visibility::Inherited,
                    &f.attrs,
                    whole,
                    f.default
                        .as_ref()
                        .map(|block| block.brace_token.span.join()),
                    inherited,
                    Vec::new(),
                )
                .and_then(|fact| self.with_bindings(fact, &f.sig)),
            syn::TraitItem::Const(c) => self
                .item(
                    &c.ident,
                    ItemKind::Const,
                    &Visibility::Inherited,
                    &c.attrs,
                    whole,
                    None,
                    inherited,
                    Vec::new(),
                )
                .and_then(|fact| match &c.default {
                    Some((eq, _)) => self.with_initializer(fact, eq.span(), c.semi_token.span()),
                    None => Ok(fact),
                }),
            syn::TraitItem::Type(t) => self.item(
                &t.ident,
                ItemKind::TypeAlias,
                &Visibility::Inherited,
                &t.attrs,
                whole,
                None,
                inherited,
                Vec::new(),
            ),
            // A macro invocation in a trait body defines nothing visible, as
            // in an impl block.
            syn::TraitItem::Macro(_) => return Ok(None),
            other => Err(unsupported(other)),
        };
        fact.map(Some)
    }

    fn impl_member(&self, member: &syn::ImplItem, inherited: TestRole) -> Found<Option<ItemFact>> {
        let whole = member.span();
        let fact = match member {
            syn::ImplItem::Fn(f) => {
                let fact = self.item(
                    &f.sig.ident,
                    ItemKind::Fn,
                    &f.vis,
                    &f.attrs,
                    whole,
                    Some(f.block.brace_token.span.join()),
                    inherited,
                    Vec::new(),
                )?;
                self.with_bindings(fact, &f.sig)?
            }
            syn::ImplItem::Const(c) => {
                let fact = self.item(
                    &c.ident,
                    ItemKind::Const,
                    &c.vis,
                    &c.attrs,
                    whole,
                    None,
                    inherited,
                    Vec::new(),
                )?;
                self.with_initializer(fact, c.eq_token.span(), c.semi_token.span())?
            }
            syn::ImplItem::Type(t) => self.item(
                &t.ident,
                ItemKind::TypeAlias,
                &t.vis,
                &t.attrs,
                whole,
                None,
                inherited,
                Vec::new(),
            )?,
            // A macro invocation in an impl block defines nothing visible.
            syn::ImplItem::Macro(_) => return Ok(None),
            syn::ImplItem::Verbatim(tokens) => match verbatim::member(tokens) {
                Some(read) => self.bodiless(&read, whole, inherited)?,
                None => return Err(unsupported(member)),
            },
            other => return Err(unsupported(other)),
        };
        Ok(Some(fact))
    }

    fn impl_fact(&self, block: &syn::ItemImpl, inherited: TestRole) -> Found<ImplFact> {
        let whole = block.span();
        let line = block.impl_token.span.start().line;
        let summary = summarize(&block.attrs, &FactMeter::scratch())?;
        let test = role(summary.own_test, inherited);
        let mut items = Vec::new();
        for member in &block.items {
            if let Some(fact) = self.impl_member(member, test)? {
                items.push(fact);
            }
        }
        let written_path = self_path_of(&block.self_ty);
        let self_path: Option<Vec<String>> = written_path.map(|path| {
            path.segments
                .iter()
                .map(|segment| unraw(&segment.ident))
                .collect()
        });
        let self_leading_colon = written_path.is_some_and(|path| path.leading_colon.is_some());
        let mut generics = block
            .generics
            .params
            .iter()
            .filter_map(|param| match param {
                syn::GenericParam::Type(param) => Some(unraw(&param.ident)),
                syn::GenericParam::Const(param) => Some(unraw(&param.ident)),
                syn::GenericParam::Lifetime(_) => None,
            })
            .collect::<Vec<_>>();
        // A negative impl keeps its `!` in the written trait path.
        let trait_path = block.trait_.as_ref().map(|(path, _)| {
            let written = path.to_token_stream().to_string();
            fit(match block.modifiers.polarity {
                Some(_) => format!("!{written}"),
                None => written,
            })
        });
        let trait_segments = block.trait_.as_ref().map(|(path, _)| PathFact {
            segments: path.segments.iter().map(|s| unraw(&s.ident)).collect(),
            leading_colon: path.leading_colon.is_some(),
        });
        let self_type = fit(block.self_ty.to_token_stream().to_string());
        self.meter.charge(
            size_of::<ImplFact>()
                + trait_path.as_ref().map_or(0, String::capacity)
                + trait_segments
                    .as_ref()
                    .map_or(0, |path| strings_bytes(&path.segments))
                + self_type.capacity()
                + self_path.as_deref().map_or(0, strings_bytes)
                + strings_bytes(&generics)
                + slack(&mut generics)
                + slack(&mut items),
            line,
        )?;
        Ok(ImplFact {
            line,
            span: self.positions.span(whole)?,
            body: self.positions.span(block.brace_token.span.join())?,
            trait_path,
            trait_segments,
            self_type,
            self_path,
            self_leading_colon,
            generics,
            items,
            test,
        })
    }

    fn module_fact(&self, module: &syn::ItemMod, inherited: TestRole) -> Found<ModFact> {
        let name_line = module.ident.span().start().line;
        let summary = summarize(&module.attrs, &self.meter).map_err(|error| match error {
            ExtractError::PathAttr {
                line,
                module: None,
                predicate,
                message,
            } => ExtractError::PathAttr {
                line,
                module: Some(unraw(&module.ident)),
                predicate,
                message,
            },
            other => other,
        })?;
        let test = role(summary.own_test, inherited);
        let (body_span, inline) = match &module.content {
            Some((brace, items)) => (
                Some(self.positions.span(brace.span.join())?),
                Some(self.module(items, test, name_line)?),
            ),
            None => (None, None),
        };
        let name = unraw(&module.ident);
        let vis = vis(&module.vis);
        self.meter.charge(
            size_of::<ModFact>() + name.capacity() + vis_bytes(&vis),
            name_line,
        )?;
        Ok(ModFact {
            name,
            name_line,
            span: self.positions.span(module.span())?,
            body_span,
            vis,
            attrs: summary.facts,
            documented: summary.documented,
            doc_hidden: summary.doc_hidden,
            test,
            paths: summary.paths,
            inline,
        })
    }
}

impl Extractor<'_> {
    /// The owned form of a `use` tree, each node charged as it is built: its
    /// size and the names it holds.
    fn use_tree(&self, tree: &syn::UseTree, line: usize) -> Found<UseTree> {
        let node = match tree {
            syn::UseTree::Path(path) => {
                let name = unraw(&path.ident);
                self.meter.charge(name.capacity(), line)?;
                UseTree::Path {
                    name,
                    rest: Box::new(self.use_tree(&path.tree, line)?),
                }
            }
            syn::UseTree::Name(name) => {
                let name = unraw(&name.ident);
                self.meter.charge(name.capacity(), line)?;
                UseTree::Name(name)
            }
            syn::UseTree::Rename(rename) => {
                let (name, alias) = (unraw(&rename.ident), unraw(&rename.rename));
                self.meter
                    .charge(name.capacity() + alias.capacity(), line)?;
                UseTree::Rename { name, alias }
            }
            syn::UseTree::Glob(_) => UseTree::Glob,
            syn::UseTree::Group(group) => {
                let mut items = group
                    .items
                    .iter()
                    .map(|item| self.use_tree(item, line))
                    .collect::<Found<Vec<_>>>()?;
                self.meter.charge(slack(&mut items), line)?;
                UseTree::Group(items)
            }
        };
        self.meter.charge(size_of::<UseTree>(), line)?;
        Ok(node)
    }
}

/// Read the facts of a parsed file. Runs on the parse thread.
///
/// Every fact is charged as it is built, against what `allowance` leaves of
/// the fact-bytes budget of the load. Returns the facts and the bytes they
/// were charged.
///
/// # Errors
///
/// Returns an [`ExtractError`] for a `cfg` predicate that cannot be read, an
/// item the index does not model, and facts that pass the budget, naming the
/// line of the fact that passed it.
pub(crate) fn extract(
    tree: &syn::File,
    text: &str,
    allowance: FactAllowance,
) -> Result<(FileFacts, usize), ExtractError> {
    let extractor = Extractor {
        positions: Positions::new(text),
        meter: FactMeter {
            allowance,
            spent: Cell::new(0),
        },
    };
    extractor.meter.charge(size_of::<FileFacts>(), 1)?;
    let own = summarize(&tree.attrs, &extractor.meter)?;
    let inherited = if own.own_test == TestRole::CfgTest {
        TestRole::CfgTest
    } else {
        TestRole::None
    };
    let facts = FileFacts {
        body: extractor.module(&tree.items, inherited, 1)?,
        documented: own.documented,
        doc_hidden: own.doc_hidden,
        attrs: own.facts,
    };
    Ok((facts, extractor.meter.spent.get()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RelPath;

    #[test]
    fn a_column_is_turned_into_its_byte_offset_on_short_long_and_wide_lines() {
        let wide: String = "é".repeat(200);
        let long = "x".repeat(300);
        let text = format!("ab\n{long}\n{wide}\ntail");
        let positions = Positions::new(&text);
        // A short line, its end and its newline.
        assert_eq!(positions.byte(1, 1).ok(), Some(1));
        assert_eq!(positions.byte(1, 2).ok(), Some(2));
        // A long line of one-byte characters.
        assert_eq!(positions.byte(2, 0).ok(), Some(3));
        assert_eq!(positions.byte(2, 299).ok(), Some(3 + 299));
        // A long line of two-byte characters, before, on and after a mark.
        let start = 3 + 301;
        for column in [0, 1, 63, 64, 65, 130, 199, 200] {
            assert_eq!(
                positions.byte(3, column).ok(),
                Some(start + 2 * column),
                "column {column}"
            );
        }
        // The last line ends at the end of the text.
        assert_eq!(positions.byte(4, 4).ok(), Some(text.len()));
        // A column past its line and a line past the text are refused.
        assert!(matches!(
            positions.byte(3, 202),
            Err(ExtractError::Position { line: 3 })
        ));
        assert!(matches!(
            positions.byte(2, 302),
            Err(ExtractError::Position { line: 2 })
        ));
        assert!(matches!(
            positions.byte(9, 0),
            Err(ExtractError::Position { line: 9 })
        ));
    }

    const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";

    /// The workspace of the package `demo` with `files` beside its manifest.
    fn loaded(files: &[(&str, &str)]) -> crate::Workspace {
        let mut all = vec![("Cargo.toml", DEMO)];
        all.extend_from_slice(files);
        let tree: crate::MemoryTree = all.into_iter().collect();
        crate::Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn facts_carry_names_lines_documentation_visibility_and_macro_export() {
        let ws = loaded(&[(
            "src/lib.rs",
            "/// Documented.\npub fn documented() {}\n\n#[doc(hidden)]\npub fn hidden() {}\n\n///\nfn blank() {}\n\n#[macro_export]\nmacro_rules! shout { () => {} }\n\npub(crate) struct S { x: u8 }\nextern crate alloc as heap;\n",
        )]);
        let file = ws
            .file(&RelPath::new("src/lib.rs").expect("path"))
            .expect("loaded");
        let items = &file.facts.body.items;
        assert_eq!(items[0].name, "documented");
        assert!(items[0].documented && !items[0].doc_hidden);
        assert_eq!(items[0].name_line, 2);
        assert!(!items[1].documented && items[1].doc_hidden);
        assert!(!items[2].documented);
        assert!(items[3].macro_export);
        assert_eq!(items[4].vis, Vis::Restricted("crate".to_owned()));
        assert!(items[4].body.is_some());
        assert_eq!(items[5].name, "heap");
        assert_eq!(items[5].renamed_from.as_deref(), Some("alloc"));
        let span = &items[0].span;
        assert_eq!(
            &file.text[span.start_byte..span.end_byte],
            "/// Documented.\npub fn documented() {}"
        );
    }

    #[test]
    fn use_trees_and_impl_headers_are_recorded_as_written() {
        let ws = loaded(&[(
            "src/lib.rs",
            "use ::std::{fmt, io::{self, Read as R}};\nuse crate::a::*;\nstruct Holder<T>(T);\nimpl<T> Holder<T> { fn get(&self) {} }\nimpl Trait for a::b::Thing { fn run(&self) {} }\ntrait Trait {}\n",
        )]);
        let file = ws
            .file(&RelPath::new("src/lib.rs").expect("path"))
            .expect("loaded");
        assert_eq!(file.facts.body.uses.len(), 2);
        assert!(file.facts.body.uses[0].leading_colon);
        assert_eq!(file.facts.body.impls.len(), 2);
        let inherent = &file.facts.body.impls[0];
        assert_eq!(
            inherent.self_path.as_deref(),
            Some(&["Holder".to_owned()][..])
        );
        assert!(inherent.trait_path.is_none());
        assert_eq!(inherent.items[0].name, "get");
        let by_trait = &file.facts.body.impls[1];
        assert_eq!(by_trait.trait_path.as_deref(), Some("Trait"));
        assert_eq!(
            by_trait.self_path.as_deref(),
            Some(&["a".to_owned(), "b".to_owned(), "Thing".to_owned()][..])
        );
    }

    #[test]
    fn facts_record_struct_shapes_unnamed_consts_leading_colons_and_documentation_forms() {
        let ws = loaded(&[
            (
                "src/lib.rs",
                "//! The crate.\n\
                 pub struct Unit;\n\
                 pub struct Tuple(u8);\n\
                 pub struct Named { x: u8 }\n\
                 const _: () = ();\n\
                 #[doc = include_str!(\"lib.rs\")]\n\
                 pub fn guide() {}\n\
                 impl Foo for ::other::Thing {}\n\
                 impl Foo for local::Thing {}\n\
                 mod documented;\n\
                 trait Foo {}\n",
            ),
            ("src/documented.rs", "//! Inner docs.\n"),
        ]);
        let lib = ws
            .file(&RelPath::new("src/lib.rs").expect("path"))
            .expect("loaded");
        assert!(lib.facts.documented);
        let shapes: Vec<_> = lib
            .facts
            .body
            .items
            .iter()
            .map(|i| (i.name.as_str(), i.shape))
            .collect();
        assert_eq!(
            shapes[..3],
            [
                ("Unit", Some(FieldShape::Unit)),
                ("Tuple", Some(FieldShape::Tuple)),
                ("Named", Some(FieldShape::Named)),
            ]
        );
        let unnamed = &lib.facts.body.items[3];
        assert_eq!(
            (unnamed.name.as_str(), unnamed.kind),
            ("_", ItemKind::UnnamedConst)
        );
        assert!(lib.facts.body.items[4].documented);
        assert!(lib.facts.body.impls[0].self_leading_colon);
        assert!(!lib.facts.body.impls[1].self_leading_colon);
        let inner = ws
            .file(&RelPath::new("src/documented.rs").expect("path"))
            .expect("loaded");
        assert!(inner.facts.documented);
        let bare = loaded(&[("src/lib.rs", "pub fn plain() {}")]);
        assert!(
            !bare
                .file(&RelPath::new("src/lib.rs").expect("path"))
                .expect("loaded")
                .facts
                .documented
        );
    }

    #[test]
    fn a_verbatim_use_group_is_one_use_fact_per_member_in_its_module_with_the_whole_span() {
        let text = "pub mod a { pub fn f() {} }\n\
                    pub mod m {\n    #[cfg(test)]\n    pub(crate) use {\n        ::std::fmt::Debug as D,\n        crate::a::f,\n    };\n}\n";
        let ws = loaded(&[("src/lib.rs", text)]);
        let lib = ws
            .file(&RelPath::new("src/lib.rs").expect("path"))
            .expect("loaded");
        assert_eq!(lib.facts.body.uses, []);
        let m = lib.facts.body.mods[1].inline.as_ref().expect("inline");
        let start = text.find("#[cfg(test)]").expect("attr");
        let end = text.find("};").expect("semi") + 2;
        let read: Vec<_> = m
            .uses
            .iter()
            .map(|u| {
                (
                    u.line,
                    u.span.lines.clone(),
                    u.span.start_byte,
                    u.span.end_byte,
                    u.leading_colon,
                    u.test,
                )
            })
            .collect();
        assert_eq!(
            read,
            [
                (3, 3..8, start, end, true, TestRole::CfgTest),
                (3, 3..8, start, end, false, TestRole::CfgTest),
            ]
        );
        assert!(
            m.uses
                .iter()
                .all(|u| u.vis == Vis::Restricted("crate".to_owned()))
        );
        assert_eq!(
            m.uses[0].tree,
            UseTree::Path {
                name: "std".to_owned(),
                rest: Box::new(UseTree::Path {
                    name: "fmt".to_owned(),
                    rest: Box::new(UseTree::Rename {
                        name: "Debug".to_owned(),
                        alias: "D".to_owned(),
                    }),
                }),
            }
        );
    }

    #[test]
    fn a_large_attribute_on_a_verbatim_use_group_is_read_once_for_all_its_members() {
        use std::fmt::Write as _;
        // Reading the attribute once per member made this quadratic: 6000
        // members under a 6000-name predicate took tens of seconds.
        const MEMBERS: usize = 6000;
        let names: Vec<String> = (0..MEMBERS).map(|i| format!("a{i}")).collect();
        let mut members = String::new();
        for i in 0..MEMBERS {
            writeln!(members, "    ::std::m{i},").expect("a string takes the text");
        }
        let text = format!(
            "#[cfg(all(test, any({})))]\npub use {{\n{members}}};\n",
            names.join(", ")
        );
        let tree = syn::parse_file(&text).expect("the text parses");
        let allowance = FactAllowance {
            spent: 0,
            limit: usize::MAX,
        };
        // The attribute is read once for the whole declaration, so its count
        // stops at the ceiling of two if it is read for each member instead.
        let (extracted, tally) =
            work::measure_kind(Work::Attributes, 2, || extract(&tree, &text, allowance));
        let (facts, _) = extracted.expect("the facts are read");
        assert_eq!(tally.get(Work::Attributes), 1);
        let uses = &facts.body.uses;
        assert_eq!(uses.len(), MEMBERS);
        for (i, u) in uses.iter().enumerate() {
            assert_eq!(
                (
                    u.line,
                    u.span.lines.clone(),
                    u.span.start_byte,
                    u.span.end_byte
                ),
                (1, 1..MEMBERS + 4, 0, text.len() - 1),
                "{i}"
            );
            assert!(u.leading_colon, "{i}");
            assert_eq!((u.test, &u.vis), (TestRole::CfgTest, &Vis::Public), "{i}");
            assert_eq!(
                u.tree,
                UseTree::Path {
                    name: "std".to_owned(),
                    rest: Box::new(UseTree::Name(format!("m{i}"))),
                },
                "{i}"
            );
        }
    }

    #[test]
    fn a_long_line_of_wide_characters_is_indexed_once_for_all_its_positions() {
        use std::fmt::Write as _;
        // Twenty thousand functions on one line, each name holding a character
        // of two bytes, so a column is not a byte offset.
        const ITEMS: usize = 20_000;
        let mut text = String::new();
        let mut starts = Vec::new();
        for index in 0..ITEMS {
            starts.push(text.len());
            write!(text, "pub fn é{index}() {{}} ").expect("a string takes the text");
        }
        text.push('\n');
        let line_chars = text.chars().count();
        let tree = syn::parse_file(&text).expect("the text parses");
        let allowance = FactAllowance {
            spent: 0,
            limit: usize::MAX,
        };
        // A position costs at most one mark interval beside the one pass that
        // indexes the line. The ceiling allows eight positions per item; a
        // walk from the line start for each position, or an index built for
        // each item, passes it within a few thousand items.
        let ceiling = line_chars + 8 * ITEMS * (MARK_EVERY + 1);
        let (extracted, tally) = work::measure(ceiling, || extract(&tree, &text, allowance));
        let (facts, _) = extracted.expect("the facts are read");
        let positions = tally.get(Work::Positions);
        assert!(positions >= 2 * ITEMS, "{positions}");
        assert!(tally.get(Work::ColumnSteps) > 0);
        assert!(
            tally.get(Work::ColumnSteps) <= line_chars + positions * (MARK_EVERY + 1),
            "{} column steps for {positions} positions on a line of {line_chars}",
            tally.get(Work::ColumnSteps)
        );
        assert_eq!(facts.body.items.len(), ITEMS);
        for (index, start) in starts.iter().enumerate().step_by(997) {
            let item = &facts.body.items[index];
            assert_eq!(item.span.start_byte, *start, "é{index}");
        }
    }
}
