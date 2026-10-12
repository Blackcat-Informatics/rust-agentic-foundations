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

use quote::ToTokens as _;
use syn::spanned::Spanned as _;
use syn::{Attribute, Expr, ExprLit, Item, Lit, Meta, Visibility};

use crate::cfg::{CfgExpr, parse_list};
use crate::meter::FactAllowance;

/// A region of the file in lines and bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpanFact {
    /// The 1-based first line.
    pub start_line: usize,
    /// The 1-based last line.
    pub end_line: usize,
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
#[non_exhaustive]
pub enum ItemKind {
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
pub struct PathFact {
    /// The segments, generic arguments left out.
    pub segments: Vec<String>,
    /// True when the path starts with `::`.
    pub leading_colon: bool,
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
pub struct AttrFact {
    /// The attribute path, such as `derive` or `cfg`.
    pub path: String,
    /// The rest of the attribute, such as `(Debug)` or `= "text"`; empty when
    /// there is none.
    pub args: String,
}

/// A named item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemFact {
    /// The name without a raw-identifier prefix. For `extern crate x as y`
    /// this is `y`.
    pub name: String,
    /// The original crate name of `extern crate x as y`.
    pub renamed_from: Option<String>,
    /// What the item is.
    pub kind: ItemKind,
    /// The 1-based line of the name.
    pub name_line: usize,
    /// The whole item, attributes included.
    pub span: SpanFact,
    /// The braced body, or the function block, when there is one.
    pub body: Option<SpanFact>,
    /// The declared visibility.
    pub vis: Vis,
    /// Every attribute, in order.
    pub attrs: Vec<AttrFact>,
    /// True when an unconditional `#[doc]` attribute (or a doc comment) holds
    /// text, or is `#[doc = include_str!(..)]`. `#[doc(hidden)]` alone does
    /// not count.
    pub documented: bool,
    /// True when the item has `#[doc(hidden)]`.
    pub doc_hidden: bool,
    /// The test role.
    pub test: TestRole,
    /// True when the item has `#[macro_export]`.
    pub macro_export: bool,
    /// The items of a trait, in order.
    pub members: Vec<ItemFact>,
    /// The field shape of a struct; `None` for every other kind.
    pub shape: Option<FieldShape>,
    /// The aliased path of `type A = path::B;` when the right side is a plain
    /// path; `None` for every other item.
    pub alias: Option<PathFact>,
}

/// The `path` attributes of a module declaration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PathAttrs {
    /// Every `#[path = "..."]` that applies unconditionally.
    pub unconditional: Vec<String>,
    /// Every `path` set by `#[cfg_attr(predicate, path = "...")]`.
    pub conditional: Vec<ConditionalPath>,
}

/// A `path` that applies when a predicate holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalPath {
    /// The `cfg_attr` predicate, as written.
    pub predicate: String,
    /// True when the predicate holds only in a test build.
    pub implies_test: bool,
    /// The path value.
    pub path: String,
}

/// A module declaration: `mod name;` or `mod name { .. }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModFact {
    /// The name without a raw-identifier prefix.
    pub name: String,
    /// The 1-based line of the name.
    pub name_line: usize,
    /// The whole declaration, attributes included.
    pub span: SpanFact,
    /// The braced body of an inline module.
    pub body_span: Option<SpanFact>,
    /// The declared visibility.
    pub vis: Vis,
    /// Every attribute, in order.
    pub attrs: Vec<AttrFact>,
    /// True when the module is documented.
    pub documented: bool,
    /// True when the module has `#[doc(hidden)]`.
    pub doc_hidden: bool,
    /// The test role of the module itself.
    pub test: TestRole,
    /// The `path` attributes.
    pub paths: PathAttrs,
    /// The contents of an inline module; `None` for `mod name;`.
    pub inline: Option<ModBody>,
}

/// One `use` tree, owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UseTree {
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
pub struct UseFact {
    /// The 1-based line of the `use` keyword.
    pub line: usize,
    /// The whole declaration.
    pub span: SpanFact,
    /// The declared visibility (a re-export when not inherited).
    pub vis: Vis,
    /// True for `use ::name`.
    pub leading_colon: bool,
    /// The tree.
    pub tree: UseTree,
    /// The test role.
    pub test: TestRole,
}

/// An `impl` block header and its items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplFact {
    /// The 1-based line of the `impl` keyword.
    pub line: usize,
    /// The whole block.
    pub span: SpanFact,
    /// The braced body.
    pub body: SpanFact,
    /// The trait path as written, for a trait impl.
    pub trait_path: Option<String>,
    /// The segments of the trait path, for a trait impl.
    pub trait_segments: Option<PathFact>,
    /// The self type as written.
    pub self_type: String,
    /// The segments of the self type when it is a plain path.
    pub self_path: Option<Vec<String>>,
    /// True when the self type path starts with `::`, so it names an external
    /// crate and not a local item.
    pub self_leading_colon: bool,
    /// The names of the generic type and const parameters of the block.
    pub generics: Vec<String>,
    /// The associated items.
    pub items: Vec<ItemFact>,
    /// The test role.
    pub test: TestRole,
}

/// The contents of a module, in source order within each list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModBody {
    /// Named items other than modules, `use` declarations and impls.
    pub items: Vec<ItemFact>,
    /// Module declarations.
    pub mods: Vec<ModFact>,
    /// `use` declarations.
    pub uses: Vec<UseFact>,
    /// `impl` blocks.
    pub impls: Vec<ImplFact>,
}

/// The facts of one file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileFacts {
    /// The file's own module.
    pub body: ModBody,
    /// True when the file starts with documentation (`//!` or `#![doc = ..]`),
    /// which documents the module that the file is.
    pub documented: bool,
    /// True when the file has `#![doc(hidden)]`.
    pub doc_hidden: bool,
    /// The inner attributes of the file, in order.
    pub attrs: Vec<AttrFact>,
}

/// Why the facts of a file could not be read.
#[derive(Debug)]
pub(crate) enum ExtractError {
    /// A `cfg` or `cfg_attr` predicate is not readable.
    Cfg { line: usize, message: String },
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
        let mut starts = vec![0];
        starts.extend(text.match_indices('\n').map(|(at, _)| at + 1));
        Self {
            text,
            starts,
            long: RefCell::new(HashMap::new()),
        }
    }

    fn byte(&self, line: usize, column: usize) -> Found<usize> {
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
                    LineIndex::Ascii
                } else {
                    LineIndex::Marks(
                        own.char_indices()
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
        let mut chars = rest.char_indices();
        match chars.nth(skip) {
            Some((offset, _)) => Ok(start + base + offset),
            // Just past the last character of the line.
            None if rest.chars().count() == skip => Ok(start + base + rest.len()),
            None => Err(outside()),
        }
    }

    fn span(&self, span: proc_macro2::Span) -> Found<SpanFact> {
        let (start, end) = (span.start(), span.end());
        Ok(SpanFact {
            start_line: start.line,
            end_line: end.line,
            start_byte: self.byte(start.line, start.column)?,
            end_byte: self.byte(end.line, end.column)?,
        })
    }
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

/// The `path` attribute value, which must be a string literal.
fn path_value(expr: &Expr, line: usize) -> Found<String> {
    string_value(expr).ok_or_else(|| cfg_error(line, "`path` is not a string literal".to_owned()))
}

/// Read the meta item inside `cfg(..)`.
fn cfg_predicate(tokens: &proc_macro2::TokenStream, line: usize) -> Found<CfgExpr> {
    let meta =
        syn::parse2::<Meta>(tokens.clone()).map_err(|error| cfg_error(line, error.to_string()))?;
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
                    path: fit(path_value(&pair.value, line)?),
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
                let value = fit(path_value(&pair.value, line)?);
                meter.charge(size_of::<String>() + value.capacity(), line)?;
                summary.paths.unconditional.push(value);
            }
            ("path", _) => {
                return Err(cfg_error(line, "`path` must be `path = \"..\"`".to_owned()));
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
        let mut members = t
            .items
            .iter()
            .map(|member| self.trait_member(member, inside))
            .collect::<Found<Vec<_>>>()?;
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
            Item::Fn(f) => self.item(
                &f.sig.ident,
                ItemKind::Fn,
                &f.vis,
                &f.attrs,
                whole,
                Some(f.block.brace_token.span.join()),
                inherited,
                Vec::new(),
            )?,
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
            Item::Const(c) => self.item(
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
            )?,
            Item::Static(s) => self.item(
                &s.ident,
                ItemKind::Static,
                &s.vis,
                &s.attrs,
                whole,
                None,
                inherited,
                Vec::new(),
            )?,
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
                let line = whole.start().line;
                let summary = summarize(&u.attrs, &FactMeter::scratch())?;
                let vis = vis(&u.vis);
                self.meter
                    .charge(size_of::<UseFact>() + vis_bytes(&vis), line)?;
                body.uses.push(UseFact {
                    line,
                    span: self.positions.span(whole)?,
                    vis,
                    leading_colon: u.leading_colon.is_some(),
                    tree: self.use_tree(&u.tree, line)?,
                    test: role(summary.own_test, inherited),
                });
            }
            other => {
                return Err(ExtractError::Unsupported {
                    line: whole.start().line,
                    what: format!("{}", other.to_token_stream())
                        .chars()
                        .take(60)
                        .collect(),
                });
            }
        }
        Ok(())
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
            other => {
                return Err(ExtractError::Unsupported {
                    line: whole.start().line,
                    what: format!("{}", other.to_token_stream())
                        .chars()
                        .take(60)
                        .collect(),
                });
            }
        }
        Ok(())
    }

    fn trait_member(&self, member: &syn::TraitItem, inherited: TestRole) -> Found<ItemFact> {
        let whole = member.span();
        match member {
            syn::TraitItem::Fn(f) => self.item(
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
            ),
            syn::TraitItem::Const(c) => self.item(
                &c.ident,
                ItemKind::Const,
                &Visibility::Inherited,
                &c.attrs,
                whole,
                None,
                inherited,
                Vec::new(),
            ),
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
            other => Err(ExtractError::Unsupported {
                line: whole.start().line,
                what: format!("{}", other.to_token_stream())
                    .chars()
                    .take(60)
                    .collect(),
            }),
        }
    }

    fn impl_member(&self, member: &syn::ImplItem, inherited: TestRole) -> Found<Option<ItemFact>> {
        let whole = member.span();
        let fact = match member {
            syn::ImplItem::Fn(f) => self.item(
                &f.sig.ident,
                ItemKind::Fn,
                &f.vis,
                &f.attrs,
                whole,
                Some(f.block.brace_token.span.join()),
                inherited,
                Vec::new(),
            )?,
            syn::ImplItem::Const(c) => self.item(
                &c.ident,
                ItemKind::Const,
                &c.vis,
                &c.attrs,
                whole,
                None,
                inherited,
                Vec::new(),
            )?,
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
            other => {
                return Err(ExtractError::Unsupported {
                    line: whole.start().line,
                    what: format!("{}", other.to_token_stream())
                        .chars()
                        .take(60)
                        .collect(),
                });
            }
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
        let self_path: Option<Vec<String>> = match &*block.self_ty {
            syn::Type::Path(path) if path.qself.is_none() => Some(
                path.path
                    .segments
                    .iter()
                    .map(|segment| unraw(&segment.ident))
                    .collect(),
            ),
            _ => None,
        };
        let self_leading_colon = matches!(
            &*block.self_ty,
            syn::Type::Path(path) if path.qself.is_none() && path.path.leading_colon.is_some()
        );
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
        let trait_path = block
            .trait_
            .as_ref()
            .map(|(path, _)| fit(path.to_token_stream().to_string()));
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
        let summary = summarize(&module.attrs, &self.meter)?;
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
}
