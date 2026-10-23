// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Lexical syntax observations; workspace definition lookup stays in Source.

use super::super::{charge, failure, push};
use crate::{Error, syntax::Names};
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

pub(super) struct Expanded {
    pub(super) segments: Vec<String>,
    absolute: bool,
    pub(super) block: bool,
}

impl Expanded {
    pub(super) fn text(&self) -> String {
        format!(
            "{}{}",
            if self.absolute { "::" } else { "" },
            self.segments.join("::")
        )
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

    pub(super) fn local(&self, path: &[String], namespace: Namespace) -> bool {
        let Some(head) = path.first() else {
            return false;
        };
        for frame in self.frames.iter().rev() {
            let local = match namespace {
                Namespace::Type => frame.types.contains(head),
                Namespace::Value => {
                    frame.values.contains(head) || (path.len() > 1 && frame.types.contains(head))
                }
                Namespace::Macro => frame.macros.contains(head),
                _ => unreachable!("syntax classification passes only known namespaces"),
            };
            if local {
                return true;
            }
            if frame.names.aliases.contains_key(head) || frame.module {
                break;
            }
        }
        false
    }

    pub(super) fn expand(
        &self,
        path: &[String],
        absolute: bool,
        limits: &mut MatchLimits,
    ) -> Result<Option<Expanded>, Error> {
        let mut expanded = Expanded {
            segments: path.to_vec(),
            absolute,
            block: false,
        };
        let mut seen = WorkList::<String, 8>::new();
        while !expanded.absolute {
            let Some(head) = expanded.segments.first() else {
                return Ok(None);
            };
            let mut binding = None;
            for frame in self.frames.iter().rev() {
                charge("paths", limits, 1)?;
                if let Some(alias) = frame.names.aliases.get(head) {
                    binding = Some((alias, frame.names.absolute.contains(head), !frame.module));
                    break;
                }
                if frame.module {
                    break;
                }
            }
            let Some((alias, absolute, block)) = binding else {
                break;
            };
            if alias.is_empty() {
                return Ok(None);
            }
            let mut next = alias.clone();
            next.extend(expanded.segments.iter().skip(1).cloned());
            if next == expanded.segments {
                break;
            }
            if seen.iter().any(|name| name == head) {
                return Err(failure(
                    "paths",
                    &format!("cyclic written import alias {head:?}"),
                ));
            }
            push("paths", &mut seen, head.clone(), self.frames.len(), limits)?;
            charge("paths", limits, next.len().try_into().unwrap_or(u64::MAX))?;
            expanded.segments = next;
            expanded.absolute = absolute;
            expanded.block |= block;
        }
        Ok(Some(expanded))
    }
}
