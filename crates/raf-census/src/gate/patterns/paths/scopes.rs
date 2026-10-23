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

#[derive(Clone, Copy)]
struct Key<'a> {
    alias: &'a Alias,
    namespace: Namespace,
    frame: usize,
}

impl Key<'_> {
    fn same(self, other: Self) -> bool {
        std::ptr::eq(self.alias, other.alias)
            && self.namespace == other.namespace
            && self.frame == other.frame
    }
}

enum LookupError<'a> {
    Pending(Key<'a>),
    Failure(Error),
}

impl From<Error> for LookupError<'_> {
    fn from(error: Error) -> Self {
        Self::Failure(error)
    }
}

type Facts<'a> = WorkList<(Key<'a>, bool), 8>;

fn cached<'a>(
    facts: &Facts<'a>,
    key: Key<'a>,
    limits: &mut MatchLimits,
) -> Result<bool, LookupError<'a>> {
    charge("paths", limits, facts.len().try_into().unwrap_or(u64::MAX))?;
    facts
        .iter()
        .find(|(known, _)| known.same(key))
        .map(|(_, value)| *value)
        .ok_or(LookupError::Pending(key))
}

fn binding<'a>(
    (index, frame): (usize, &'a Frame),
    head: &str,
    namespace: Namespace,
    own: Option<&Alias>,
    limits: &mut MatchLimits,
    eligible: &mut impl FnMut(Key<'a>, &mut MatchLimits) -> Result<bool, LookupError<'a>>,
) -> Result<Option<&'a Alias>, LookupError<'a>> {
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
            eligible(
                Key {
                    alias,
                    namespace,
                    frame: index,
                },
                limits,
            )?
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
            ).into());
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

#[derive(Clone, Copy)]
enum Query<'a> {
    Local(&'a [String], Namespace),
    Expand(&'a [String], bool, Namespace),
}

enum Answer {
    Local(bool),
    Expanded(Option<Expanded>),
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

    fn local_with<'a>(
        &'a self,
        path: &[String],
        namespace: Namespace,
        limits: &mut MatchLimits,
        eligible: &mut impl FnMut(Key<'a>, &mut MatchLimits) -> Result<bool, LookupError<'a>>,
    ) -> Result<bool, LookupError<'a>> {
        let Some(head) = path.first() else {
            return Ok(false);
        };
        let namespace = head_namespace(path, namespace);
        for (index, frame) in (0..self.frames.len()).rev().zip(self.frames.iter().rev()) {
            charge("paths", limits, 1)?;
            if frame.local(head, namespace) {
                return Ok(true);
            }
            if frame.module
                || binding((index, frame), head, namespace, None, limits, eligible)?.is_some()
            {
                break;
            }
        }
        Ok(false)
    }

    fn expand_with<'a>(
        &'a self,
        path: &[String],
        absolute: bool,
        namespace: Namespace,
        (initial_own, mut frame_limit): (Option<&'a Alias>, usize),
        limits: &mut MatchLimits,
        eligible: &mut impl FnMut(Key<'a>, &mut MatchLimits) -> Result<bool, LookupError<'a>>,
    ) -> Result<Option<Expanded>, LookupError<'a>> {
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
            let own = seen.iter().last().copied().or(initial_own);
            let mut selected = None;
            for (index, frame) in (0..self.frames.len()).rev().zip(self.frames.iter().rev()) {
                charge("paths", limits, 1)?;
                if index >= frame_limit {
                    continue;
                }
                if frame.local(head, namespace) {
                    expanded.shadowed = true;
                    expanded.block = true;
                    return Ok(Some(expanded));
                }
                if let Some(alias) =
                    binding((index, frame), head, namespace, own, limits, eligible)?
                {
                    selected = Some((alias, !frame.module, index + 1));
                    break;
                }
                if frame.module {
                    break;
                }
            }
            let Some((alias, block, declaration_limit)) = selected else {
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
                return Err(
                    failure("paths", &format!("cyclic written import alias {head:?}")).into(),
                );
            }
            push("paths", &mut seen, alias, self.frames.len(), limits)?;
            charge("paths", limits, next.len().try_into().unwrap_or(u64::MAX))?;
            expanded.segments = next;
            expanded.absolute = alias.absolute;
            expanded.block |= block;
            frame_limit = declaration_limit;
        }
        Ok(Some(expanded))
    }

    // Namespace eligibility is evaluated against the same expanded block
    // bindings as the occurrence. Dependencies use a native work list instead
    // of recursive Rust calls, so alias depth spends the shared grant.
    fn resolve_pending<'a>(
        &'a self,
        first: Key<'a>,
        facts: &mut Facts<'a>,
        limits: &mut MatchLimits,
        eligible: &mut impl FnMut(&Alias, Namespace) -> Result<bool, Error>,
    ) -> Result<(), Error> {
        let mut pending = WorkList::<Key<'a>, 8>::new();
        push("paths", &mut pending, first, 0, limits)?;
        while let Some(key) = pending.iter().last().copied() {
            let mut check = |key, limits: &mut MatchLimits| cached(facts, key, limits);
            match self.expand_with(
                &key.alias.path,
                key.alias.absolute,
                key.namespace,
                (Some(key.alias), key.frame + 1),
                limits,
                &mut check,
            ) {
                Ok(expanded) => {
                    let value = if let Some(expanded) = expanded {
                        if expanded.shadowed && expanded.block {
                            true
                        } else {
                            eligible(
                                &Alias {
                                    path: expanded.segments,
                                    absolute: expanded.absolute,
                                    source_byte: key.alias.source_byte,
                                },
                                key.namespace,
                            )?
                        }
                    } else {
                        false
                    };
                    push("paths", facts, (key, value), pending.len(), limits)?;
                    pending.pop();
                }
                Err(LookupError::Failure(error)) => return Err(error),
                Err(LookupError::Pending(next)) => {
                    charge(
                        "paths",
                        limits,
                        pending.len().try_into().unwrap_or(u64::MAX),
                    )?;
                    if pending.iter().any(|known| known.same(next)) {
                        return Err(failure(
                            "paths",
                            "cyclic written import namespace dependency; provide an unambiguous source context",
                        ));
                    }
                    push("paths", &mut pending, next, 0, limits)?;
                }
            }
        }
        Ok(())
    }

    fn resolve(
        &self,
        query: Query<'_>,
        limits: &mut MatchLimits,
        eligible: &mut impl FnMut(&Alias, Namespace) -> Result<bool, Error>,
    ) -> Result<Answer, Error> {
        let mut facts = Facts::new();
        loop {
            let mut check = |key, limits: &mut MatchLimits| cached(&facts, key, limits);
            let result = match query {
                Query::Local(path, namespace) => self
                    .local_with(path, namespace, limits, &mut check)
                    .map(Answer::Local),
                Query::Expand(path, absolute, namespace) => self
                    .expand_with(
                        path,
                        absolute,
                        namespace,
                        (None, self.frames.len()),
                        limits,
                        &mut check,
                    )
                    .map(Answer::Expanded),
            };
            match result {
                Ok(answer) => return Ok(answer),
                Err(LookupError::Failure(error)) => return Err(error),
                Err(LookupError::Pending(key)) => {
                    self.resolve_pending(key, &mut facts, limits, eligible)?;
                }
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
        match self.resolve(Query::Local(path, namespace), limits, eligible)? {
            Answer::Local(value) => Ok(value),
            Answer::Expanded(_) => unreachable!("a local query returns a local answer"),
        }
    }

    pub(super) fn expand(
        &self,
        path: &[String],
        absolute: bool,
        namespace: Namespace,
        limits: &mut MatchLimits,
        eligible: &mut impl FnMut(&Alias, Namespace) -> Result<bool, Error>,
    ) -> Result<Option<Expanded>, Error> {
        match self.resolve(Query::Expand(path, absolute, namespace), limits, eligible)? {
            Answer::Expanded(value) => Ok(value),
            Answer::Local(_) => unreachable!("an expansion query returns an expansion answer"),
        }
    }
}
