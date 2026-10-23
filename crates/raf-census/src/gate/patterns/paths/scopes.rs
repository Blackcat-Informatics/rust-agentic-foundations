// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Lexical syntax observations; workspace definition lookup stays in Source.

use super::super::{charge, failure, push};
use crate::{
    Error,
    syntax::{Alias, Names, path_text},
};
use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::walk::WorkList;
use raf_source::Namespace;
use std::collections::BTreeSet;
use syn::ext::IdentExt;

#[derive(Default)]
struct Frame {
    names: Names,
    values: BTreeSet<String>,
    types: BTreeSet<String>,
    macros: BTreeSet<String>,
    module: bool,
}

impl Frame {
    fn local(&self, head: &str, namespace: Namespace) -> bool {
        match namespace {
            Namespace::Type => self.types.contains(head),
            Namespace::Value => self.values.contains(head),
            Namespace::Macro => self.macros.contains(head),
            _ => unreachable!("syntax classification passes only known namespaces"),
        }
    }
}

fn head_namespace(path: &[String], namespace: Namespace) -> Namespace {
    if path.len() > 1 {
        Namespace::Type
    } else {
        namespace
    }
}

fn binding<'a>(
    frame: &'a Frame,
    head: &str,
    namespace: Namespace,
    own: Option<&Alias>,
    limits: &mut MatchLimits,
    eligible: &mut impl FnMut(&Alias, Namespace) -> Result<bool, Error>,
) -> Result<Option<&'a Alias>, Error> {
    let mut selected: Option<&Alias> = None;
    for alias in frame.names.aliases.get(head).into_iter().flatten() {
        if own.is_some_and(|own| std::ptr::eq(own, alias)) {
            continue;
        }
        let bytes = alias.path.iter().map(String::len).sum::<usize>();
        charge(
            "paths",
            limits,
            bytes
                .saturating_mul(Namespace::ALL.len())
                .saturating_add(1)
                .try_into()
                .unwrap_or(u64::MAX),
        )?;
        let applicable = if alias.path.is_empty() {
            namespace == Namespace::Type
        } else {
            eligible(alias, namespace)?
        };
        if !applicable {
            continue;
        }
        if selected.is_some_and(|previous| {
            previous.path != alias.path || previous.absolute != alias.absolute
        }) {
            return Err(failure(
                "paths",
                &format!(
                    "written import {head:?} has competing {namespace:?} bindings; provide an unambiguous indexed import context"
                ),
            ));
        }
        selected = Some(alias);
    }
    Ok(selected)
}

pub(super) struct Expanded {
    pub(super) segments: Vec<String>,
    absolute: bool,
    pub(super) block: bool,
    pub(super) shadowed: bool,
}

impl Expanded {
    pub(super) fn text(&self) -> String {
        path_text(&self.segments, self.absolute)
    }
}

#[derive(Default)]
pub(super) struct Scopes {
    frames: WorkList<Frame, 8>,
}

impl Scopes {
    pub(super) fn enter<'a>(
        &mut self,
        module: bool,
        items: impl Iterator<Item = &'a syn::Item>,
        limits: &MatchLimits,
    ) -> Result<(), Error> {
        let mut frame = Frame {
            module,
            ..Frame::default()
        };
        for item in items {
            frame.names.item(item);
            if !module {
                match item {
                    syn::Item::Fn(item) => {
                        frame.values.insert(item.sig.ident.unraw().to_string());
                    }
                    syn::Item::Const(item) => {
                        frame.values.insert(item.ident.unraw().to_string());
                    }
                    syn::Item::Static(item) => {
                        frame.values.insert(item.ident.unraw().to_string());
                    }
                    syn::Item::Struct(item) => {
                        frame.types.insert(item.ident.unraw().to_string());
                        if !matches!(item.fields, syn::Fields::Named(_)) {
                            frame.values.insert(item.ident.unraw().to_string());
                        }
                    }
                    syn::Item::Enum(item) => {
                        frame.types.insert(item.ident.unraw().to_string());
                    }
                    syn::Item::Type(item) => {
                        frame.types.insert(item.ident.unraw().to_string());
                    }
                    syn::Item::Trait(item) => {
                        frame.types.insert(item.ident.unraw().to_string());
                    }
                    syn::Item::Mod(item) => {
                        frame.types.insert(item.ident.unraw().to_string());
                    }
                    _ => {}
                }
            }
        }
        push("paths", &mut self.frames, frame, 0, limits)
    }

    pub(super) fn leave(&mut self) {
        self.frames.pop();
    }

    pub(super) fn bind(
        &mut self,
        pattern: &syn::Pat,
        mut include: impl FnMut(&syn::Ident) -> Result<bool, Error>,
    ) -> Result<(), Error> {
        let frame = self.frames.top_mut().expect("entered lexical scope");
        let mut failure = None;
        raf_source::visit_pattern_bindings(pattern, |name| {
            if failure.is_some() {
                return;
            }
            match include(name) {
                Ok(true) => {
                    frame.values.insert(name.unraw().to_string());
                }
                Ok(false) => {}
                Err(error) => failure = Some(error),
            }
        });
        failure.map_or(Ok(()), Err)
    }

    pub(super) fn receiver(&mut self) {
        self.frames
            .top_mut()
            .expect("entered lexical scope")
            .values
            .insert("self".into());
    }

    pub(super) fn macro_name(&mut self, name: &syn::Ident) {
        self.frames
            .top_mut()
            .expect("entered lexical scope")
            .macros
            .insert(name.unraw().to_string());
    }

    pub(super) fn generics(&mut self, generics: &syn::Generics, self_type: bool) {
        let frame = self.frames.top_mut().expect("entered lexical scope");
        if self_type {
            frame.types.insert("Self".into());
        }
        for parameter in &generics.params {
            match parameter {
                syn::GenericParam::Type(ty) => {
                    frame.types.insert(ty.ident.unraw().to_string());
                }
                syn::GenericParam::Const(value) => {
                    frame.values.insert(value.ident.unraw().to_string());
                }
                syn::GenericParam::Lifetime(_) => {}
            }
        }
    }

    pub(super) fn local(
        &self,
        path: &[String],
        namespace: Namespace,
        limits: &mut MatchLimits,
        eligible: &mut impl FnMut(&Alias, Namespace) -> Result<bool, Error>,
    ) -> Result<bool, Error> {
        let Some(head) = path.first() else {
            return Ok(false);
        };
        let namespace = head_namespace(path, namespace);
        for frame in self.frames.iter().rev() {
            charge("paths", limits, 1)?;
            if frame.local(head, namespace) {
                return Ok(true);
            }
            if frame.module || binding(frame, head, namespace, None, limits, eligible)?.is_some() {
                break;
            }
        }
        Ok(false)
    }

    pub(super) fn expand(
        &self,
        path: &[String],
        absolute: bool,
        namespace: Namespace,
        limits: &mut MatchLimits,
        eligible: &mut impl FnMut(&Alias, Namespace) -> Result<bool, Error>,
    ) -> Result<Option<Expanded>, Error> {
        let mut expanded = Expanded {
            segments: path.to_vec(),
            absolute,
            block: false,
            shadowed: false,
        };
        let mut seen = WorkList::<&Alias, 8>::new();
        while !expanded.absolute {
            let Some(head) = expanded.segments.first() else {
                return Ok(None);
            };
            let namespace = head_namespace(&expanded.segments, namespace);
            let own = seen.iter().last().copied();
            let mut selected = None;
            for frame in self.frames.iter().rev() {
                charge("paths", limits, 1)?;
                if frame.local(head, namespace) {
                    expanded.shadowed = true;
                    expanded.block = true;
                    return Ok(Some(expanded));
                }
                if let Some(alias) = binding(frame, head, namespace, own, limits, eligible)? {
                    selected = Some((alias, !frame.module));
                    break;
                }
                if frame.module {
                    break;
                }
            }
            let Some((alias, block)) = selected else {
                break;
            };
            if alias.path.is_empty() {
                expanded.shadowed = true;
                expanded.block |= block;
                return Ok(Some(expanded));
            }
            let mut next = alias.path.clone();
            next.extend(expanded.segments.iter().skip(1).cloned());
            if next == expanded.segments {
                expanded.block |= block;
                break;
            }
            charge("paths", limits, seen.len().try_into().unwrap_or(u64::MAX))?;
            if seen.iter().any(|seen| std::ptr::eq(*seen, alias)) {
                return Err(failure(
                    "paths",
                    &format!("cyclic written import alias {head:?}"),
                ));
            }
            push("paths", &mut seen, alias, self.frames.len(), limits)?;
            charge("paths", limits, next.len().try_into().unwrap_or(u64::MAX))?;
            expanded.segments = next;
            expanded.absolute = alias.absolute;
            expanded.block |= block;
        }
        Ok(Some(expanded))
    }
}
