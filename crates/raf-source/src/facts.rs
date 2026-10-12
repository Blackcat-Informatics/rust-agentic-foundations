// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Owned facts about one source file.
//!
//! The facts are read on the parse thread by [`extract`] and leave it as plain
//! data, so no syntax tree and no span outlives its thread. They hold what the
//! module walk needs (module declarations with their `path` attributes) and
//! what the symbol index needs (items, `use` trees and impl headers with
//! their positions, attributes, documentation and test role).

use quote::ToTokens as _;
use syn::spanned::Spanned as _;
use syn::{Attribute, Expr, ExprLit, Item, Lit, Meta, Visibility};

use crate::cfg::{CfgExpr, parse_list};

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
    /// The self type as written.
    pub self_type: String,
    /// The segments of the self type when it is a plain path.
    pub self_path: Option<Vec<String>>,
    /// True when the self type path starts with `::`, so it names an external
    /// crate and not a local item.
    pub self_leading_colon: bool,
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
}

type Found<T> = Result<T, ExtractError>;

/// Line starts of the text, for turning positions into byte offsets.
struct Positions<'t> {
    text: &'t str,
    starts: Vec<usize>,
}

impl<'t> Positions<'t> {
    fn new(text: &'t str) -> Self {
        let mut starts = vec![0];
        starts.extend(text.match_indices('\n').map(|(at, _)| at + 1));
        Self { text, starts }
    }

    fn byte(&self, line: usize, column: usize) -> Found<usize> {
        let outside = || ExtractError::Position { line };
        let start = *line
            .checked_sub(1)
            .and_then(|index| self.starts.get(index))
            .ok_or_else(outside)?;
        let rest = self.text.get(start..).ok_or_else(outside)?;
        match rest.char_indices().nth(column) {
            Some((offset, _)) => Ok(start + offset),
            None if rest.chars().count() == column => Ok(self.text.len()),
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

fn unraw(ident: &syn::Ident) -> String {
    let text = ident.to_string();
    match text.strip_prefix("r#") {
        Some(plain) => plain.to_owned(),
        None => text,
    }
}

fn vis(visibility: &Visibility) -> Vis {
    match visibility {
        Visibility::Public(_) => Vis::Public,
        Visibility::Restricted(restricted) => {
            let path = restricted.path.to_token_stream().to_string();
            Vis::Restricted(if restricted.in_token.is_some() {
                format!("in {path}")
            } else {
                path
            })
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
                summary.paths.conditional.push(ConditionalPath {
                    predicate: combined.to_string(),
                    implies_test: combined.implies_test(),
                    path: path_value(&pair.value, line)?,
                });
            }
            Meta::List(list) if list.path.is_ident("cfg_attr") => {
                apply_cfg_attr(&list.tokens, Some(&combined), summary, line)?;
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

/// Read the attributes of an item.
fn summarize(attrs: &[Attribute]) -> Found<AttrSummary> {
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
        let path = path_text(attr.path());
        let args = match &attr.meta {
            Meta::Path(_) => String::new(),
            Meta::List(list) => format!("({})", list.tokens),
            Meta::NameValue(pair) => format!("= {}", pair.value.to_token_stream()),
        };
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
                summary
                    .paths
                    .unconditional
                    .push(path_value(&pair.value, line)?);
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
                apply_cfg_attr(&list.tokens, None, &mut summary, line)?;
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
        let summary = summarize(attrs)?;
        Ok(ItemFact {
            name: unraw(name),
            renamed_from: None,
            kind,
            name_line,
            span: self.positions.span(whole)?,
            body: body.map(|span| self.positions.span(span)).transpose()?,
            vis: vis(vis_of),
            attrs: summary.facts,
            documented: summary.documented,
            doc_hidden: summary.doc_hidden,
            test: role(summary.own_test, inherited),
            macro_export: summary.macro_export,
            members,
            shape: None,
        })
    }

    fn module(&self, items: &[Item], inherited: TestRole) -> Found<ModBody> {
        let mut body = ModBody::default();
        for item in items {
            self.one(item, inherited, &mut body)?;
        }
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
        let own = summarize(&t.attrs)?;
        let inside = role(own.own_test, inherited);
        let members = t
            .items
            .iter()
            .map(|member| self.trait_member(member, inside))
            .collect::<Found<Vec<_>>>()?;
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
            Item::Type(t) => self.item(
                &t.ident,
                ItemKind::TypeAlias,
                &t.vis,
                &t.attrs,
                whole,
                None,
                inherited,
                Vec::new(),
            )?,
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
                    fact.renamed_from = Some(unraw(&e.ident));
                }
                body.items.push(fact);
            }
            Item::ForeignMod(block) => {
                let own = summarize(&block.attrs)?;
                let inside = role(own.own_test, inherited);
                for foreign in &block.items {
                    self.foreign(foreign, inside, body)?;
                }
            }
            Item::Mod(m) => body.mods.push(self.module_fact(m, inherited)?),
            Item::Impl(i) => body.impls.push(self.impl_fact(i, inherited)?),
            Item::Use(u) => {
                let line = whole.start().line;
                let summary = summarize(&u.attrs)?;
                body.uses.push(UseFact {
                    line,
                    span: self.positions.span(whole)?,
                    vis: vis(&u.vis),
                    leading_colon: u.leading_colon.is_some(),
                    tree: use_tree(&u.tree),
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
        let summary = summarize(&block.attrs)?;
        let test = role(summary.own_test, inherited);
        let mut items = Vec::new();
        for member in &block.items {
            if let Some(fact) = self.impl_member(member, test)? {
                items.push(fact);
            }
        }
        let self_path = match &*block.self_ty {
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
        Ok(ImplFact {
            line,
            span: self.positions.span(whole)?,
            body: self.positions.span(block.brace_token.span.join())?,
            trait_path: block
                .trait_
                .as_ref()
                .map(|(path, _)| path.to_token_stream().to_string()),
            self_type: block.self_ty.to_token_stream().to_string(),
            self_path,
            self_leading_colon,
            items,
            test,
        })
    }

    fn module_fact(&self, module: &syn::ItemMod, inherited: TestRole) -> Found<ModFact> {
        let name_line = module.ident.span().start().line;
        let summary = summarize(&module.attrs)?;
        let test = role(summary.own_test, inherited);
        let (body_span, inline) = match &module.content {
            Some((brace, items)) => (
                Some(self.positions.span(brace.span.join())?),
                Some(self.module(items, test)?),
            ),
            None => (None, None),
        };
        Ok(ModFact {
            name: unraw(&module.ident),
            name_line,
            span: self.positions.span(module.span())?,
            body_span,
            vis: vis(&module.vis),
            attrs: summary.facts,
            documented: summary.documented,
            doc_hidden: summary.doc_hidden,
            test,
            paths: summary.paths,
            inline,
        })
    }
}

fn use_tree(tree: &syn::UseTree) -> UseTree {
    match tree {
        syn::UseTree::Path(path) => UseTree::Path {
            name: unraw(&path.ident),
            rest: Box::new(use_tree(&path.tree)),
        },
        syn::UseTree::Name(name) => UseTree::Name(unraw(&name.ident)),
        syn::UseTree::Rename(rename) => UseTree::Rename {
            name: unraw(&rename.ident),
            alias: unraw(&rename.rename),
        },
        syn::UseTree::Glob(_) => UseTree::Glob,
        syn::UseTree::Group(group) => UseTree::Group(group.items.iter().map(use_tree).collect()),
    }
}

/// Read the facts of a parsed file. Runs on the parse thread.
///
/// # Errors
///
/// Returns an [`ExtractError`] for a `cfg` predicate that cannot be read or an
/// item the index does not model.
pub(crate) fn extract(tree: &syn::File, text: &str) -> Result<FileFacts, ExtractError> {
    let extractor = Extractor {
        positions: Positions::new(text),
    };
    let own = summarize(&tree.attrs)?;
    let inherited = if own.own_test == TestRole::CfgTest {
        TestRole::CfgTest
    } else {
        TestRole::None
    };
    Ok(FileFacts {
        body: extractor.module(&tree.items, inherited)?,
        documented: own.documented,
        doc_hidden: own.doc_hidden,
        attrs: own.facts,
    })
}
