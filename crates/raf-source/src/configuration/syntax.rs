// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Select declaration syntax on the existing bounded parse worker.
use super::{
    ActiveConfiguration, CfgExpr, ItemAdmission, LoadError, Memory, Meter, RelPath, WorkList,
    evaluate, failure, native, recover,
};
use syn::ext::IdentExt as _;
use syn::spanned::Spanned as _;
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
    }
    Ok(())
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
