// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Select declaration syntax on the existing bounded parse worker.
use super::{
    ActiveConfiguration, CfgExpr, ItemAdmission, LoadError, Memory, Meter, RelPath, WorkList,
    evaluate, failure, native, recover,
};
use syn::ext::IdentExt as _;
use syn::spanned::Spanned as _;
use syn::visit_mut::VisitMut;
use syn::{Attribute, Meta};

pub(crate) fn predicate(
    meta: &Meta,
    cfg: &ActiveConfiguration,
    dependency: bool,
    meter: &Meter,
    path: &RelPath,
) -> Result<bool, LoadError> {
    let expr = CfgExpr::from_meta_configured(meta).map_err(|message| failure(path, &message))?;
    evaluate(&expr, cfg, dependency, meter, path)
}

pub(crate) fn filter(
    tree: &mut syn::File,
    cfg: &ActiveConfiguration,
    meter: &Meter,
    path: &RelPath,
) -> Result<(), LoadError> {
    if attributes(&mut tree.attrs, cfg, meter, path)? {
        items(&mut tree.items, cfg, meter, path)?;
    } else {
        tree.items.clear();
    }
    Ok(())
}
fn attributes(
    attrs: &mut Vec<Attribute>,
    cfg: &ActiveConfiguration,
    meter: &Meter,
    path: &RelPath,
) -> Result<bool, LoadError> {
    let mut admission = ItemAdmission::new(meter, path);
    let mut memory = Memory::new(&mut admission);
    let result = (|| {
        let mut pending = WorkList::<Attribute, 8>::new();
        for attr in std::mem::take(attrs).into_iter().rev() {
            pending
                .try_push_admitted(attr, &mut memory)
                .map_err(native)?;
        }
        let mut active = true;
        while let Some(mut attr) = pending.pop() {
            let attribute_path = match &mut attr.meta {
                Meta::Path(path) => path,
                Meta::List(list) => &mut list.path,
                Meta::NameValue(pair) => &mut pair.path,
            };
            for segment in &mut attribute_path.segments {
                segment.ident = segment.ident.unraw();
            }
            meter.work(1, path, Some(attr.span().start().line))?;
            if attr.path().is_ident("cfg") {
                let meta = attr
                    .parse_args_with(crate::cfg::parse_meta)
                    .map_err(|error| failure(path, &error.to_string()))?;
                active &= predicate(&meta, cfg, false, meter, path)?;
                memory.push(attrs, attr).map_err(native)?;
            } else if attr.path().is_ident("cfg_attr") {
                let Meta::List(list) = &attr.meta else {
                    return Err(failure(path, "cfg_attr needs a predicate and attributes"));
                };
                let metas = crate::cfg::parse_punctuated(&list.tokens)
                    .map_err(|message| failure(path, &message))?;
                if metas.len() == 1 && !metas.trailing_punct() {
                    return Err(failure(
                        path,
                        "cfg_attr requires a comma after its predicate",
                    ));
                }
                let mut metas = metas.into_iter();
                let first = metas
                    .next()
                    .ok_or_else(|| failure(path, "cfg_attr has no predicate"))?;
                let applies = predicate(&first, cfg, false, meter, path)?;
                if applies {
                    for meta in metas.rev() {
                        let style = match &attr.style {
                            syn::AttrStyle::Outer => syn::AttrStyle::Outer,
                            syn::AttrStyle::Inner(bang) => {
                                syn::AttrStyle::Inner(syn::Token![!](bang.spans))
                            }
                        };
                        pending
                            .try_push_admitted(
                                Attribute {
                                    pound_token: syn::token::Pound {
                                        spans: attr.pound_token.spans,
                                    },
                                    style,
                                    bracket_token: syn::token::Bracket {
                                        span: attr.bracket_token.span,
                                    },
                                    meta,
                                },
                                &mut memory,
                            )
                            .map_err(native)?;
                    }
                }
            } else if attr.path().is_ident("test") {
                active &= evaluate(&CfgExpr::Test, cfg, false, meter, path)?;
                memory.push(attrs, attr).map_err(native)?;
            } else {
                memory.push(attrs, attr).map_err(native)?;
            }
        }
        Ok(active)
    })();
    result.map_err(|error| recover(error, &mut admission))
}
fn select<T>(
    values: &mut Vec<T>,
    get: fn(&mut T) -> Option<&mut Vec<Attribute>>,
    cfg: &ActiveConfiguration,
    meter: &Meter,
    path: &RelPath,
) -> Result<(), LoadError> {
    let mut admission = ItemAdmission::new(meter, path);
    let mut output = Vec::new();
    Memory::new(&mut admission)
        .reserve(&mut output, values.len())
        .map_err(|source| admission.error(source))?;
    for mut value in std::mem::take(values) {
        meter.work(1, path, None)?;
        if let Some(attrs) = get(&mut value)
            && !attributes(attrs, cfg, meter, path)?
        {
            continue;
        }
        output.push(value);
    }
    *values = output;
    Ok(())
}
fn items(
    values: &mut Vec<syn::Item>,
    cfg: &ActiveConfiguration,
    meter: &Meter,
    path: &RelPath,
) -> Result<(), LoadError> {
    select(values, item_attrs, cfg, meter, path)?;
    for item in values {
        meter.work(1, path, None)?;
        match item {
            syn::Item::Mod(module) => {
                if let Some((_, items_)) = &mut module.content {
                    items(items_, cfg, meter, path)?;
                }
            }
            syn::Item::Trait(item) => select(&mut item.items, trait_attrs, cfg, meter, path)?,
            syn::Item::Impl(item) => select(&mut item.items, impl_attrs, cfg, meter, path)?,
            syn::Item::ForeignMod(item) => {
                select(&mut item.items, foreign_attrs, cfg, meter, path)?;
            }
            _ => {}
        }
        blocks(item, cfg, meter, path)?;
    }
    Ok(())
}

/// Select the module declarations inside the function bodies and initializers
/// of an item, as `items` selects those of an item list.
fn blocks(
    item: &mut syn::Item,
    cfg: &ActiveConfiguration,
    meter: &Meter,
    path: &RelPath,
) -> Result<(), LoadError> {
    if !matches!(
        item,
        syn::Item::Fn(_)
            | syn::Item::Const(_)
            | syn::Item::Static(_)
            | syn::Item::Impl(_)
            | syn::Item::Trait(_)
    ) {
        return Ok(());
    }
    let mut selection = BlockSelection {
        cfg,
        meter,
        path,
        failed: None,
    };
    selection.visit_item_mut(item);
    selection.failed.map_or(Ok(()), Err)
}

/// Applies `cfg` and `cfg_attr` to the items declared in blocks.
struct BlockSelection<'a> {
    cfg: &'a ActiveConfiguration,
    meter: &'a Meter,
    path: &'a RelPath,
    failed: Option<LoadError>,
}

impl BlockSelection<'_> {
    /// Apply `cfg` and `cfg_attr` to a list of attributed positions inside an
    /// expression, dropping the inactive ones.
    fn select<T>(&mut self, values: &mut Vec<T>, get: fn(&mut T) -> Option<&mut Vec<Attribute>>) {
        // Most lists carry no attribute; they cost nothing, and the selection
        // of the rest is charged once each.
        if values
            .iter_mut()
            .all(|value| get(value).is_none_or(|attrs| attrs.is_empty()))
        {
            return;
        }
        if self.failed.is_none()
            && let Err(error) = select(values, get, self.cfg, self.meter, self.path)
        {
            self.failed = Some(error);
        }
    }
}

impl BlockSelection<'_> {
    /// Select the elements of a comma separated list of expressions.
    fn select_list<P: Default>(&mut self, list: &mut syn::punctuated::Punctuated<syn::Expr, P>) {
        if list
            .iter_mut()
            .all(|value| expr_attrs(value).is_none_or(|attrs| attrs.is_empty()))
        {
            return;
        }
        let mut values: Vec<_> = std::mem::take(list).into_iter().collect();
        self.select(&mut values, expr_attrs);
        *list = values.into_iter().collect();
    }
}

impl VisitMut for BlockSelection<'_> {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        for mut statement in std::mem::take(&mut block.stmts) {
            if self.failed.is_some() {
                return;
            }
            if let syn::Stmt::Item(item) = statement {
                let mut one = vec![item];
                if let Err(error) = items(&mut one, self.cfg, self.meter, self.path) {
                    self.failed = Some(error);
                    return;
                }
                if let Some(item) = one.pop() {
                    block.stmts.push(syn::Stmt::Item(item));
                }
                continue;
            }
            let active = match statement_attrs(&mut statement) {
                Some(attrs) if !attrs.is_empty() => {
                    attributes(attrs, self.cfg, self.meter, self.path)
                }
                _ => Ok(true),
            };
            match active {
                Ok(true) => {
                    self.visit_stmt_mut(&mut statement);
                    block.stmts.push(statement);
                }
                Ok(false) => {}
                Err(error) => {
                    self.failed = Some(error);
                    return;
                }
            }
        }
    }

    fn visit_expr_match_mut(&mut self, expression: &mut syn::ExprMatch) {
        self.select(&mut expression.arms, |arm| Some(&mut arm.attrs));
        syn::visit_mut::visit_expr_match_mut(self, expression);
    }

    fn visit_expr_array_mut(&mut self, expression: &mut syn::ExprArray) {
        self.select_list(&mut expression.elems);
        syn::visit_mut::visit_expr_array_mut(self, expression);
    }

    fn visit_expr_call_mut(&mut self, expression: &mut syn::ExprCall) {
        self.select_list(&mut expression.args);
        syn::visit_mut::visit_expr_call_mut(self, expression);
    }

    fn visit_expr_method_call_mut(&mut self, expression: &mut syn::ExprMethodCall) {
        self.select_list(&mut expression.args);
        syn::visit_mut::visit_expr_method_call_mut(self, expression);
    }

    fn visit_expr_tuple_mut(&mut self, expression: &mut syn::ExprTuple) {
        self.select_list(&mut expression.elems);
        syn::visit_mut::visit_expr_tuple_mut(self, expression);
    }

    fn visit_expr_struct_mut(&mut self, expression: &mut syn::ExprStruct) {
        if expression.fields.iter().all(|field| field.attrs.is_empty()) {
            syn::visit_mut::visit_expr_struct_mut(self, expression);
            return;
        }
        let mut fields: Vec<_> = std::mem::take(&mut expression.fields).into_iter().collect();
        self.select(&mut fields, |field| Some(&mut field.attrs));
        expression.fields = fields.into_iter().collect();
        syn::visit_mut::visit_expr_struct_mut(self, expression);
    }
}

/// The attributes of a statement that is not an item: those of a `let`, of a
/// macro statement and of an expression statement.
fn statement_attrs(statement: &mut syn::Stmt) -> Option<&mut Vec<Attribute>> {
    match statement {
        syn::Stmt::Local(local) => Some(&mut local.attrs),
        syn::Stmt::Macro(mac) => Some(&mut mac.attrs),
        syn::Stmt::Expr(expression, _) => expr_attrs(expression),
        syn::Stmt::Item(_) => None,
    }
}

fn expr_attrs(expression: &mut syn::Expr) -> Option<&mut Vec<Attribute>> {
    use syn::Expr;
    match expression {
        Expr::Array(e) => Some(&mut e.attrs),
        Expr::Assign(e) => Some(&mut e.attrs),
        Expr::Async(e) => Some(&mut e.attrs),
        Expr::Await(e) => Some(&mut e.attrs),
        Expr::Binary(e) => Some(&mut e.attrs),
        Expr::Block(e) => Some(&mut e.attrs),
        Expr::Break(e) => Some(&mut e.attrs),
        Expr::Call(e) => Some(&mut e.attrs),
        Expr::Cast(e) => Some(&mut e.attrs),
        Expr::Closure(e) => Some(&mut e.attrs),
        Expr::Const(e) => Some(&mut e.attrs),
        Expr::Continue(e) => Some(&mut e.attrs),
        Expr::Field(e) => Some(&mut e.attrs),
        Expr::ForLoop(e) => Some(&mut e.attrs),
        Expr::Group(e) => Some(&mut e.attrs),
        Expr::If(e) => Some(&mut e.attrs),
        Expr::Index(e) => Some(&mut e.attrs),
        Expr::Infer(e) => Some(&mut e.attrs),
        Expr::Let(e) => Some(&mut e.attrs),
        Expr::Lit(e) => Some(&mut e.attrs),
        Expr::Loop(e) => Some(&mut e.attrs),
        Expr::Macro(e) => Some(&mut e.attrs),
        Expr::Match(e) => Some(&mut e.attrs),
        Expr::MethodCall(e) => Some(&mut e.attrs),
        Expr::Paren(e) => Some(&mut e.attrs),
        Expr::Path(e) => Some(&mut e.attrs),
        Expr::Range(e) => Some(&mut e.attrs),
        Expr::RawAddr(e) => Some(&mut e.attrs),
        Expr::Reference(e) => Some(&mut e.attrs),
        Expr::Repeat(e) => Some(&mut e.attrs),
        Expr::Return(e) => Some(&mut e.attrs),
        Expr::Struct(e) => Some(&mut e.attrs),
        Expr::Try(e) => Some(&mut e.attrs),
        Expr::TryBlock(e) => Some(&mut e.attrs),
        Expr::Tuple(e) => Some(&mut e.attrs),
        Expr::Unary(e) => Some(&mut e.attrs),
        Expr::Unsafe(e) => Some(&mut e.attrs),
        Expr::While(e) => Some(&mut e.attrs),
        Expr::Yield(e) => Some(&mut e.attrs),
        _ => None,
    }
}

fn item_attrs(item: &mut syn::Item) -> Option<&mut Vec<Attribute>> {
    match item {
        syn::Item::Const(item) => Some(&mut item.attrs),
        syn::Item::Enum(item) => Some(&mut item.attrs),
        syn::Item::ExternCrate(item) => Some(&mut item.attrs),
        syn::Item::Fn(item) => Some(&mut item.attrs),
        syn::Item::ForeignMod(item) => Some(&mut item.attrs),
        syn::Item::Impl(item) => Some(&mut item.attrs),
        syn::Item::Macro(item) => Some(&mut item.attrs),
        syn::Item::Mod(item) => Some(&mut item.attrs),
        syn::Item::Static(item) => Some(&mut item.attrs),
        syn::Item::Struct(item) => Some(&mut item.attrs),
        syn::Item::Trait(item) => Some(&mut item.attrs),
        syn::Item::TraitAlias(item) => Some(&mut item.attrs),
        syn::Item::Type(item) => Some(&mut item.attrs),
        syn::Item::Union(item) => Some(&mut item.attrs),
        syn::Item::Use(item) => Some(&mut item.attrs),
        _ => None,
    }
}
fn trait_attrs(item: &mut syn::TraitItem) -> Option<&mut Vec<Attribute>> {
    match item {
        syn::TraitItem::Const(item) => Some(&mut item.attrs),
        syn::TraitItem::Fn(item) => Some(&mut item.attrs),
        syn::TraitItem::Type(item) => Some(&mut item.attrs),
        syn::TraitItem::Macro(item) => Some(&mut item.attrs),
        _ => None,
    }
}
fn impl_attrs(item: &mut syn::ImplItem) -> Option<&mut Vec<Attribute>> {
    match item {
        syn::ImplItem::Const(item) => Some(&mut item.attrs),
        syn::ImplItem::Fn(item) => Some(&mut item.attrs),
        syn::ImplItem::Type(item) => Some(&mut item.attrs),
        syn::ImplItem::Macro(item) => Some(&mut item.attrs),
        _ => None,
    }
}
fn foreign_attrs(item: &mut syn::ForeignItem) -> Option<&mut Vec<Attribute>> {
    match item {
        syn::ForeignItem::Static(item) => Some(&mut item.attrs),
        syn::ForeignItem::Fn(item) => Some(&mut item.attrs),
        syn::ForeignItem::Type(item) => Some(&mut item.attrs),
        syn::ForeignItem::Macro(item) => Some(&mut item.attrs),
        _ => None,
    }
}
