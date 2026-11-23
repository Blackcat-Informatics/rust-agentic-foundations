// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Lexical syntax observations; workspace definition lookup stays in Source.

use super::super::{charge, failure, push};
use super::receivers::ReceiverBinding;
use super::{EnumMembers, GlobCache};
use crate::{
    Error,
    syntax::{Alias, AliasAttributes, ImportLeaf, Names, path_text},
};
use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::walk::WorkList;
use raf_source::{ItemId, Namespace};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use syn::ext::IdentExt;

#[derive(Default)]
struct Frame {
    globs: Vec<Glob>,
    names: Names,
    receiver_imports: Names,
    values: BTreeSet<String>,
    types: BTreeSet<String>,
    macros: BTreeSet<String>,
    // Key presence serves syntactic path lookup. The value records whether
    // the binding exists for runtime receiver lookup.
    local_values: BTreeMap<String, bool>,
    written_receivers: BTreeMap<String, ReceiverBinding>,
    generic_types: BTreeSet<String>,
    settled: RefCell<GlobCache>,
    module: bool,
}

pub(super) struct Glob {
    pub(super) path: Vec<String>,
    pub(super) absolute: bool,
    pub(super) source_byte: Option<usize>,
    pub(super) trusted: bool,
    pub(super) attributes: AliasAttributes,
}

impl Frame {
    fn import_globs(
        &mut self,
        tree: &syn::UseTree,
        absolute: bool,
        attributes: &[syn::Attribute],
        require_inert: bool,
    ) {
        let attributes = AliasAttributes::of(attributes);
        let trusted = !require_inert || attributes == AliasAttributes::Inert;
        crate::syntax::visit_import_paths(tree, |parts, leaf| {
            if matches!(leaf, ImportLeaf::Glob) {
                self.globs.push(Glob {
                    path: parts.iter().map(|part| part.unraw().to_string()).collect(),
                    absolute,
                    source_byte: parts.first().map(|part| part.span().byte_range().start),
                    trusted,
                    attributes,
                });
            }
        });
    }

    fn local(&self, head: &str, namespace: Namespace, runtime: bool, receiver: bool) -> bool {
        match namespace {
            Namespace::Type => {
                self.types.contains(head) || runtime && self.generic_types.contains(head)
            }
            Namespace::Value => {
                self.values.contains(head)
                    || runtime
                        && self
                            .local_values
                            .get(head)
                            .is_some_and(|&live| !receiver || live)
            }
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

enum WrittenBinding<'a> {
    Certain(&'a Alias),
    Uncertain,
}

fn binding<'a>(
    (index, frame, receiver_imports): (usize, &'a Frame, bool),
    head: &str,
    namespace: Namespace,
    own: Option<&Alias>,
    enum_patterns: bool,
    limits: &mut MatchLimits,
    eligible: &mut impl FnMut(Key<'a>, &mut MatchLimits) -> Result<bool, LookupError<'a>>,
) -> Result<Option<WrittenBinding<'a>>, LookupError<'a>> {
    let mut selected: Option<&Alias> = None;
    let mut uncertain = false;
    let inventories =
        std::iter::once(&frame.names).chain(receiver_imports.then_some(&frame.receiver_imports));
    for alias in inventories
        .filter_map(|names| names.aliases.get(head))
        .flatten()
    {
        if own.is_some_and(|own| std::ptr::eq(own, alias))
            || alias.type_only && namespace != Namespace::Type
        {
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
        if enum_patterns {
            match alias.attributes {
                AliasAttributes::Excluded => continue,
                AliasAttributes::Uncertain => {
                    uncertain = true;
                    continue;
                }
                AliasAttributes::Inert => {}
            }
        }
        let configured_import = receiver_imports && !frame.module && !alias.path.is_empty();
        if configured_import && alias.attributes == AliasAttributes::Excluded {
            continue;
        }
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
        if configured_import && alias.attributes == AliasAttributes::Uncertain {
            uncertain = true;
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
    Ok(if uncertain {
        Some(WrittenBinding::Uncertain)
    } else {
        selected.map(WrittenBinding::Certain)
    })
}

enum Shadowing {
    Block,
    Module,
}

pub(in crate::gate) struct Expanded {
    pub(in crate::gate) segments: Vec<String>,
    absolute: bool,
    pub(in crate::gate) block: bool,
    shadowing: Option<Shadowing>,
    alias_attributes: AliasAttributes,
    pub(super) guarded_alias: bool,
    first_frame: Option<usize>,
    alias_frame: Option<usize>,
}

impl Expanded {
    pub(in crate::gate) fn is_shadowed(&self) -> bool {
        self.shadowing.is_some()
    }

    pub(super) fn module_anchor(&self) -> bool {
        matches!(self.shadowing, Some(Shadowing::Module))
    }

    pub(super) fn has_uncertain_alias(&self) -> bool {
        self.alias_attributes != AliasAttributes::Inert
    }

    pub(in crate::gate) fn text(&self) -> String {
        path_text(&self.segments, self.absolute)
    }

    pub(super) fn sees_glob(&self, frame: usize) -> bool {
        self.first_frame.is_none_or(|first| frame > first)
    }

    pub(super) fn first_frame(&self) -> Option<usize> {
        self.first_frame
    }

    pub(super) fn is_absolute(&self) -> bool {
        self.absolute
    }

    pub(super) fn alias_frame(&self) -> Option<usize> {
        self.alias_frame
    }

    pub(super) fn sees_target_glob(&self, frame: usize) -> bool {
        self.alias_frame
            .map_or_else(|| self.sees_glob(frame), |target| frame <= target)
    }
}

pub(super) struct AliasTarget<'a> {
    pub(super) alias: &'a Alias,
    pub(super) expanded: Expanded,
    frame: usize,
}

impl AliasTarget<'_> {
    pub(super) fn sees_glob(&self, frame: usize) -> bool {
        frame <= self.frame && self.expanded.sees_target_glob(frame)
    }

    fn into_alias(self) -> Alias {
        Alias {
            path: self.expanded.segments,
            absolute: self.expanded.absolute,
            source_byte: self.alias.source_byte,
            external: self.alias.external,
            type_only: self.alias.type_only,
            attributes: self.alias.attributes,
            frame: self.alias.frame,
        }
    }
}

type AliasProbe<'a> =
    dyn FnMut(AliasTarget<'_>, Namespace, &mut MatchLimits) -> Result<bool, Error> + 'a;

#[derive(Default)]
pub(in crate::gate) struct Scopes {
    frames: WorkList<Frame, 8>,
    pub(super) enums: RefCell<BTreeMap<ItemId, Rc<EnumMembers>>>,
    glob_states: Cell<usize>,
    module_globs: bool,
    exclude_inactive: bool,
    track_written_receivers: bool,
    read_receiver_imports: Cell<bool>,
    receiver_floor: usize,
    receiver_states: usize,
}

#[derive(Clone, Copy)]
enum Query<'a> {
    Local(&'a [String], Namespace),
    Expand(&'a [String], bool, Namespace, usize, bool),
}

enum Answer {
    Local(bool),
    Expanded(Option<Expanded>),
}

impl Scopes {
    pub(super) fn for_patterns(written_receivers: bool) -> Self {
        Self {
            track_written_receivers: written_receivers,
            ..Self::default()
        }
    }

    pub(super) fn for_enum_patterns() -> Self {
        Self {
            module_globs: true,
            exclude_inactive: true,
            ..Self::default()
        }
    }

    pub(super) fn for_visibility_demand() -> Self {
        Self {
            exclude_inactive: true,
            ..Self::default()
        }
    }

    pub(in crate::gate) fn enter<'a>(
        &mut self,
        module: bool,
        track_globs: bool,
        items: impl Iterator<Item = &'a syn::Item>,
        limits: &MatchLimits,
    ) -> Result<(), Error> {
        let mut frame = Frame {
            module,
            ..Frame::default()
        };
        let keep_globs =
            (track_globs || self.track_written_receivers) && (!module || self.module_globs);
        let excluded = |attributes: &[syn::Attribute]| {
            self.exclude_inactive && crate::syntax::definitely_excluded(attributes)
        };
        for item in items {
            if excluded(crate::syntax::item_attributes(item)) {
                continue;
            }
            frame.names.item(item);
            if (track_globs || keep_globs)
                && let syn::Item::Verbatim(tokens) = item
            {
                raf_source::visit_grouped_import(tokens, |attributes, absolute, tree| {
                    if excluded(attributes) {
                        return;
                    }
                    if track_globs {
                        frame.names.import_tree(tree, absolute, attributes);
                    } else if keep_globs {
                        frame
                            .receiver_imports
                            .import_tree(tree, absolute, attributes);
                    }
                    if keep_globs {
                        frame.import_globs(tree, absolute, attributes, self.module_globs);
                    }
                });
            }
            if keep_globs && let syn::Item::Use(import) = item {
                frame.import_globs(
                    &import.tree,
                    import.leading_colon.is_some(),
                    &import.attrs,
                    self.module_globs,
                );
            }
            if !module || self.module_globs {
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
        let retained = if self.track_written_receivers {
            self.receiver_states.saturating_add(self.glob_states.get())
        } else {
            0
        };
        push("paths", &mut self.frames, frame, retained, limits)
    }

    pub(in crate::gate) fn leave(&mut self) {
        if let Some(frame) = self.frames.pop() {
            self.glob_states
                .set(self.glob_states.get() - frame.settled.into_inner().retained);
            self.receiver_states -= frame
                .written_receivers
                .values()
                .map(ReceiverBinding::states)
                .sum::<usize>();
        }
    }

    pub(super) fn begin_receiver_boundary(&mut self) -> usize {
        std::mem::replace(&mut self.receiver_floor, self.frames.len())
    }

    pub(super) fn with_receiver_lookup<T>(&self, query: impl FnOnce() -> T) -> T {
        let previous = self.read_receiver_imports.replace(true);
        let result = query();
        self.read_receiver_imports.set(previous);
        result
    }

    pub(super) fn receiver_lookup(&self) -> bool {
        self.read_receiver_imports.get()
    }

    pub(super) fn end_receiver_boundary(&mut self, floor: usize) {
        self.receiver_floor = floor;
    }

    pub(super) fn admit_receiver(
        &self,
        label: &str,
        states: usize,
        limits: &MatchLimits,
    ) -> Result<(), Error> {
        if states > self.remaining_glob_states(limits) {
            Err(failure(
                label,
                "written receiver binding-state limit exhausted",
            ))
        } else {
            Ok(())
        }
    }

    pub(super) fn record_receiver(
        &mut self,
        name: String,
        binding: ReceiverBinding,
        label: &str,
        limits: &MatchLimits,
    ) -> Result<(), Error> {
        self.admit_receiver(label, binding.states(), limits)?;
        let frame = self
            .frames
            .top_mut()
            .ok_or_else(|| failure(label, "receiver binding requires an active lexical scope"))?;
        self.receiver_states += binding.states();
        if let Some(previous) = frame.written_receivers.insert(name, binding) {
            self.receiver_states -= previous.states();
        }
        Ok(())
    }

    pub(super) fn written_receiver(
        &self,
        name: &str,
        label: &str,
        limits: &mut MatchLimits,
    ) -> Result<Option<(usize, &ReceiverBinding)>, Error> {
        let frames = self.receiver_floor..self.frames.len();
        for (index, frame) in frames.rev().zip(self.frames.iter().rev()) {
            let entries = frame
                .written_receivers
                .len()
                .saturating_add(frame.local_values.len())
                .saturating_add(frame.values.len());
            let work = name
                .len()
                .saturating_add(1)
                .saturating_mul(entries.bit_width() as usize + 1);
            charge(label, limits, work.try_into().unwrap_or(u64::MAX))?;
            if let Some(binding) = frame.written_receivers.get(name) {
                return Ok(Some((index, binding)));
            }
            if frame.local_values.get(name) == Some(&true) || frame.values.contains(name) {
                break;
            }
        }
        Ok(None)
    }

    pub(super) fn retain_glob_states(
        &self,
        count: usize,
        limits: &MatchLimits,
    ) -> Result<(), Error> {
        if count > self.remaining_glob_states(limits) {
            return Err(failure(
                "visibility",
                "block glob cache state limit exhausted",
            ));
        }
        self.glob_states.set(self.glob_states.get() + count);
        Ok(())
    }

    pub(super) fn remaining_glob_states(&self, limits: &MatchLimits) -> usize {
        let retained = if self.track_written_receivers {
            self.receiver_states
                .saturating_add(self.glob_states.get())
                .saturating_add(self.frames.len())
        } else {
            self.glob_states.get()
        };
        limits.states.saturating_sub(retained)
    }

    pub(super) fn in_module(&self) -> bool {
        self.frames
            .iter()
            .next_back()
            .is_some_and(|frame| frame.module)
    }

    pub(super) fn in_file_module(&self) -> bool {
        self.frames.len() == 1
    }

    pub(super) fn runtime_binding(
        &self,
        frame: usize,
        name: &str,
        limits: &mut MatchLimits,
    ) -> Result<bool, Error> {
        charge(
            "paths",
            limits,
            frame.saturating_add(1).try_into().unwrap_or(u64::MAX),
        )?;
        Ok(self.frames.iter().nth(frame).is_some_and(|frame| {
            frame.local_values.contains_key(name) && !frame.values.contains(name)
        }))
    }

    #[expect(
        clippy::expect_used,
        reason = "the path visitor creates a frame before admitting bindings within it"
    )]
    pub(in crate::gate) fn bind(
        &mut self,
        pattern: &syn::Pat,
        live_binding: bool,
        mut include: impl FnMut(&Self, &syn::Ident) -> Result<bool, Error>,
    ) -> Result<(), Error> {
        let mut failure = None;
        raf_source::visit_pattern_bindings(pattern, |name| {
            if failure.is_some() {
                return;
            }
            match include(self, name) {
                Ok(true) => {
                    let frame = self
                        .frames
                        .top_mut()
                        .expect("INVARIANT: entered lexical scope");
                    let name = name.unraw().to_string();
                    if live_binding {
                        if let Some(previous) = frame.written_receivers.remove(&name) {
                            self.receiver_states -= previous.states();
                        }
                        frame.local_values.insert(name, true);
                    } else {
                        frame.local_values.entry(name).or_insert(false);
                    }
                }
                Ok(false) => {}
                Err(error) => failure = Some(error),
            }
        });
        failure.map_or(Ok(()), Err)
    }

    #[expect(
        clippy::expect_used,
        reason = "the method visitor registers the receiver in its entered lexical scope"
    )]
    pub(in crate::gate) fn receiver(&mut self) {
        self.frames
            .top_mut()
            .expect("INVARIANT: entered lexical scope")
            .local_values
            .insert("self".into(), true);
    }

    #[expect(
        clippy::expect_used,
        reason = "the macro visitor registers the definition in its entered lexical scope"
    )]
    pub(in crate::gate) fn macro_name(&mut self, name: &syn::Ident) {
        self.frames
            .top_mut()
            .expect("INVARIANT: entered lexical scope")
            .macros
            .insert(name.unraw().to_string());
    }

    pub(super) fn has_textual_macro(
        &self,
        name: &str,
        limits: &mut MatchLimits,
    ) -> Result<bool, Error> {
        for frame in self.frames.iter().rev() {
            charge("paths", limits, 1)?;
            if frame.macros.contains(name) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[expect(
        clippy::expect_used,
        reason = "the item visitor opens the lexical scope before registering generic parameters"
    )]
    pub(in crate::gate) fn generics(&mut self, generics: &syn::Generics, self_type: bool) {
        let frame = self
            .frames
            .top_mut()
            .expect("INVARIANT: entered lexical scope");
        if self_type {
            frame.generic_types.insert("Self".into());
        }
        for parameter in &generics.params {
            match parameter {
                syn::GenericParam::Type(ty) => {
                    frame.generic_types.insert(ty.ident.unraw().to_string());
                }
                syn::GenericParam::Const(value) => {
                    frame
                        .local_values
                        .insert(value.ident.unraw().to_string(), true);
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
            if frame.local(head, namespace, true, self.read_receiver_imports.get()) {
                return Ok(true);
            }
            if frame.module
                || binding(
                    (index, frame, self.read_receiver_imports.get()),
                    head,
                    namespace,
                    None,
                    self.module_globs,
                    limits,
                    eligible,
                )?
                .is_some()
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
        (initial_own, mut frame_limit, mut runtime): (Option<&'a Alias>, usize, bool),
        limits: &mut MatchLimits,
        eligible: &mut impl FnMut(Key<'a>, &mut MatchLimits) -> Result<bool, LookupError<'a>>,
    ) -> Result<Option<Expanded>, LookupError<'a>> {
        let mut expanded = Expanded {
            segments: path.to_vec(),
            absolute,
            block: false,
            shadowing: None,
            alias_attributes: AliasAttributes::Inert,
            guarded_alias: false,
            first_frame: None,
            alias_frame: None,
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
                if frame.local(head, namespace, runtime, self.read_receiver_imports.get()) {
                    expanded.first_frame.get_or_insert(index);
                    expanded.shadowing = Some(if frame.module {
                        Shadowing::Module
                    } else {
                        Shadowing::Block
                    });
                    expanded.block = !frame.module || !self.module_globs;
                    return Ok(Some(expanded));
                }
                if let Some(alias) = binding(
                    (index, frame, self.read_receiver_imports.get()),
                    head,
                    namespace,
                    own,
                    self.module_globs,
                    limits,
                    eligible,
                )? {
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
            expanded.first_frame.get_or_insert(declaration_limit - 1);
            let WrittenBinding::Certain(alias) = alias else {
                expanded.alias_attributes = AliasAttributes::Uncertain;
                return Ok(Some(expanded));
            };
            expanded.guarded_alias |= alias.attributes != AliasAttributes::Inert;
            if alias.path.is_empty() {
                expanded.shadowing = Some(if block {
                    Shadowing::Block
                } else {
                    Shadowing::Module
                });
                expanded.block |= block;
                return Ok(Some(expanded));
            }
            expanded.alias_frame = Some(declaration_limit - 1);
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
            runtime = false;
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
        eligible: &mut AliasProbe<'_>,
    ) -> Result<(), Error> {
        let mut pending = WorkList::<Key<'a>, 8>::new();
        push("paths", &mut pending, first, 0, limits)?;
        while let Some(key) = pending.iter().last().copied() {
            let mut check = |key, limits: &mut MatchLimits| cached(facts, key, limits);
            match self.expand_with(
                &key.alias.path,
                key.alias.absolute,
                key.namespace,
                (Some(key.alias), key.frame + 1, false),
                limits,
                &mut check,
            ) {
                Ok(expanded) => {
                    let value = if let Some(expanded) = expanded {
                        if expanded.has_uncertain_alias()
                            || expanded.is_shadowed() && expanded.block
                        {
                            true
                        } else {
                            eligible(
                                AliasTarget {
                                    alias: key.alias,
                                    expanded,
                                    frame: key.frame,
                                },
                                key.namespace,
                                limits,
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
        eligible: &mut AliasProbe<'_>,
    ) -> Result<Answer, Error> {
        let mut facts = Facts::new();
        loop {
            let mut check = |key, limits: &mut MatchLimits| cached(&facts, key, limits);
            let result = match query {
                Query::Local(path, namespace) => self
                    .local_with(path, namespace, limits, &mut check)
                    .map(Answer::Local),
                Query::Expand(path, absolute, namespace, frame_limit, runtime) => self
                    .expand_with(
                        path,
                        absolute,
                        namespace,
                        (None, frame_limit, runtime),
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

    pub(in crate::gate) fn local(
        &self,
        path: &[String],
        namespace: Namespace,
        limits: &mut MatchLimits,
        eligible: &mut impl FnMut(&Alias, Namespace) -> Result<bool, Error>,
    ) -> Result<bool, Error> {
        let mut probe = |target: AliasTarget<'_>, namespace, _: &mut MatchLimits| {
            eligible(&target.into_alias(), namespace)
        };
        match self.resolve(Query::Local(path, namespace), limits, &mut probe)? {
            Answer::Local(value) => Ok(value),
            Answer::Expanded(_) => unreachable!("a local query returns a local answer"),
        }
    }

    pub(in crate::gate) fn expand(
        &self,
        path: &[String],
        absolute: bool,
        namespace: Namespace,
        limits: &mut MatchLimits,
        eligible: &mut impl FnMut(&Alias, Namespace) -> Result<bool, Error>,
    ) -> Result<Option<Expanded>, Error> {
        let mut probe = |target: AliasTarget<'_>, namespace, _: &mut MatchLimits| {
            eligible(&target.into_alias(), namespace)
        };
        self.expand_checked((path, absolute, namespace), false, None, limits, &mut probe)
    }

    pub(super) fn expand_checked(
        &self,
        (path, absolute, namespace): (&[String], bool, Namespace),
        import: bool,
        frame_limit: Option<usize>,
        limits: &mut MatchLimits,
        eligible: &mut AliasProbe<'_>,
    ) -> Result<Option<Expanded>, Error> {
        self.expansion(
            Query::Expand(
                path,
                absolute,
                namespace,
                frame_limit.unwrap_or(self.frames.len()),
                !import,
            ),
            limits,
            eligible,
        )
    }

    pub(super) fn expand_import(
        &self,
        path: &[String],
        absolute: bool,
        namespace: Namespace,
        frame_limit: Option<usize>,
        limits: &mut MatchLimits,
        eligible: &mut impl FnMut(&Alias, Namespace) -> Result<bool, Error>,
    ) -> Result<Option<Expanded>, Error> {
        let frame_limit = frame_limit.unwrap_or(self.frames.len());
        let mut probe = |target: AliasTarget<'_>, namespace, _: &mut MatchLimits| {
            eligible(&target.into_alias(), namespace)
        };
        self.expansion(
            Query::Expand(path, absolute, namespace, frame_limit, false),
            limits,
            &mut probe,
        )
    }

    fn expansion(
        &self,
        query: Query<'_>,
        limits: &mut MatchLimits,
        eligible: &mut AliasProbe<'_>,
    ) -> Result<Option<Expanded>, Error> {
        match self.resolve(query, limits, eligible)? {
            Answer::Expanded(value) => Ok(value),
            Answer::Local(_) => unreachable!("an expansion query returns an expansion answer"),
        }
    }

    pub(super) fn block_globs(
        &self,
    ) -> impl Iterator<Item = (usize, &[Glob], &RefCell<GlobCache>)> {
        (0..self.frames.len())
            .rev()
            .zip(self.frames.iter().rev())
            .scan(true, |blocks, (index, frame)| {
                if !*blocks {
                    return None;
                }
                *blocks = !frame.module;
                (!frame.module || self.module_globs).then_some((index, frame))
            })
            .filter(|(_, frame)| !frame.globs.is_empty())
            .map(|(index, frame)| (index, frame.globs.as_slice(), &frame.settled))
    }
}
