// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Borrowed lexical value inventories; unresolved names never prove binders.
//!
//! Items are hoisted when a frame opens. Locals become visible only through
//! `bind`; the collector must first visit initializers and let-else divergence.
//! A standard prelude requires an actual Source-owned declared edition profile.
//! Otherwise closed absence requires an actual `no_implicit_prelude` source
//! attribute, inherited by child modules.
//! Name equality uses the admitted native canonical-equivalence profile while
//! source spellings and locations remain unchanged.

use syn::punctuated::Pair;

use super::coordinates::Coordinates;
use super::names;
use super::prelude::Prelude;
use super::resource::{Meter, Refusal, Scratch};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PatternMeaning {
    Binding,
    Fixed,
    Unsupported(&'static str),
}

#[derive(Clone, Copy)]
enum Kind {
    Local(usize),
    Fixed,
    Other,
    Forbidden,
    Uncertain,
    Open {
        reason: &'static str,
        inherited: bool,
    },
}

#[derive(Clone, Copy)]
struct Entry<'source> {
    name: &'source str,
    kind: Kind,
}

#[derive(Clone, Copy)]
enum ScopeKind {
    Module,
    Lexical,
}

#[derive(Clone, Copy)]
struct Frame {
    start: usize,
    kind: ScopeKind,
    closed_prelude: bool,
    no_std: bool,
    local_fence: bool,
}

#[derive(Clone, Copy)]
enum Meaning {
    Local(usize),
    BlockedLocal,
    Fixed,
    Other,
    Forbidden,
    Absent,
    Unsupported(&'static str),
}

/// Original-source names and native-admitted typed lexical frames.
pub(super) struct Scopes<'source, 'meter> {
    source: &'source str,
    coordinates: &'meter Coordinates<'source, 'meter>,
    meter: &'meter Meter,
    entries: Scratch<'meter, Entry<'source>>,
    frames: Scratch<'meter, Frame>,
    standard: Option<Prelude>,
}

impl<'source, 'meter> Scopes<'source, 'meter> {
    pub(super) const fn new(
        source: &'source str,
        meter: &'meter Meter,
        coordinates: &'meter Coordinates<'source, 'meter>,
    ) -> Self {
        Self {
            source,
            coordinates,
            meter,
            entries: Scratch::new(meter),
            frames: Scratch::new(meter),
            standard: None,
        }
    }

    pub(super) fn with_prelude(
        source: &'source str,
        meter: &'meter Meter,
        profile: Prelude,
        coordinates: &'meter Coordinates<'source, 'meter>,
    ) -> Self {
        Self {
            standard: Some(profile),
            ..Self::new(source, meter, coordinates)
        }
    }

    pub(super) fn asynchronous_profile(&self) -> Result<(), Refusal> {
        self.meter.work(1)?;
        match self.standard.map(Prelude::edition) {
            Some(2018 | 2021 | 2024) => Ok(()),
            Some(_) => Err(Refusal::Unsupported(
                "async syntax requires a declared edition of at least 2018",
            )),
            None => Err(Refusal::Unsupported(
                "async syntax needs an actual declared Source edition profile",
            )),
        }
    }

    /// Preserve actual declared policy at a function's lexical module, even
    /// when impl or trait generic-value frames sit above that module.
    pub(super) fn profile(&self) -> Result<Prelude, Refusal> {
        let declared = self.standard.ok_or(Refusal::Invariant(
            "source provenance requires an actual declared edition profile",
        ))?;
        let (disabled, no_std) = self.inherited_prelude()?;
        Ok(declared.with_flags(disabled, no_std))
    }

    /// A nested module inherits prelude policy, but not its parent's values.
    pub(super) fn enter_module(
        &mut self,
        items: &[syn::Item],
        attrs: &[syn::Attribute],
        crate_root: bool,
    ) -> Result<(), Refusal> {
        self.enter_module_at(items, attrs, false, crate_root)
    }

    pub(super) fn enter_declared_module(&mut self, module: &syn::ItemMod) -> Result<(), Refusal> {
        self.enter_module_at(
            module
                .content
                .as_ref()
                .map_or(&[], |(_, items)| items.as_slice()),
            &module.attrs,
            true,
            false,
        )
    }

    fn enter_module_at(
        &mut self,
        items: &[syn::Item],
        attrs: &[syn::Attribute],
        declaration: bool,
        crate_root: bool,
    ) -> Result<(), Refusal> {
        let (mut closed, mut no_std) = self.inherited_prelude()?;
        for attr in attrs {
            self.attribute_work(attr)?;
            if self.attribute_named(attr, "feature")? {
                self.language_features(attr, crate_root)?;
            }
            if self.attribute_named(attr, "no_core")?
                || self.attribute_named(attr, "prelude_import")?
            {
                return Err(Refusal::Unsupported(
                    "custom or absent core preludes need a separate admitted language profile",
                ));
            }
            if self.attribute_named(attr, "no_std")? {
                if !crate_root || !matches!(attr.style, syn::AttrStyle::Inner(_)) {
                    return Err(Refusal::Unsupported(
                        "no_std is admitted only as an inner crate-root attribute",
                    ));
                }
                if !matches!(attr.meta, syn::Meta::Path(_)) {
                    return Err(Refusal::Unsupported(
                        "no_std must be an unconditional bare attribute",
                    ));
                }
                no_std = true;
            }
            if self.attribute_named(attr, "no_implicit_prelude")? {
                if !matches!(attr.meta, syn::Meta::Path(_)) {
                    return Err(Refusal::Unsupported(
                        "no_implicit_prelude must be an unconditional bare attribute",
                    ));
                }
                closed = true;
            }
        }
        self.frames.push(Frame {
            start: self.entries.len(),
            kind: ScopeKind::Module,
            closed_prelude: closed,
            no_std,
            local_fence: false,
        })?;
        if self.attributes(attrs, true, declaration)? {
            self.open(
                "a conditional module requires admitted active selection",
                true,
            )?;
        }
        for item in items {
            self.item(item)?;
        }
        Ok(())
    }

    pub(super) fn enter_block(&mut self, block: &syn::Block) -> Result<(), Refusal> {
        self.enter_values()?;
        for statement in &block.stmts {
            self.meter.work(1)?;
            match statement {
                syn::Stmt::Item(item) => self.item(item)?,
                syn::Stmt::Macro(_) => {
                    self.open(
                        "an unexpanded block macro may introduce lexical values",
                        false,
                    )?;
                }
                syn::Stmt::Local(local) => {
                    if self.attributes(&local.attrs, false, false)? {
                        self.open(
                            "a conditional local needs an admitted active context",
                            false,
                        )?;
                    }
                }
                syn::Stmt::Expr(_, _) => {}
            }
        }
        Ok(())
    }

    pub(super) fn enter_values(&mut self) -> Result<(), Refusal> {
        if self.frames.as_slice().is_empty() {
            return Err(Refusal::Invariant("a value scope requires a module frame"));
        }
        self.frames.push(Frame {
            start: self.entries.len(),
            kind: ScopeKind::Lexical,
            closed_prelude: false,
            no_std: false,
            local_fence: false,
        })
    }

    /// Inline constants and const arguments retain item/generic names while
    /// blocking dynamic locals across this fence. New locals inside remain
    /// visible, including to nested closures.
    pub(super) fn enter_const(&mut self) -> Result<(), Refusal> {
        self.enter_values()?;
        self.meter.work(1)?;
        self.frames
            .as_mut_slice()
            .last_mut()
            .ok_or(Refusal::Invariant("const scope lost its frame"))?
            .local_fence = true;
        Ok(())
    }

    pub(super) fn leave(&mut self) -> Result<(), Refusal> {
        let frame = self
            .frames
            .as_slice()
            .last()
            .ok_or(Refusal::Invariant("cannot leave an absent lexical frame"))?;
        self.entries.truncate(frame.start)?;
        let _ = self.frames.pop()?;
        Ok(())
    }

    /// Activate a declaration only after its pattern meaning has been proved.
    pub(super) fn bind(&mut self, name: &'source str, id: usize) -> Result<(), Refusal> {
        match self.pattern(name)? {
            PatternMeaning::Binding => self.entry(name, Kind::Local(id)),
            PatternMeaning::Fixed => Err(Refusal::Unsupported(
                "a local binding cannot replace a fixed pattern name",
            )),
            PatternMeaning::Unsupported(reason) => Err(Refusal::Unsupported(reason)),
        }
    }

    /// Register a syntactically explicit const parameter in its value frame.
    pub(super) fn constant(&mut self, name: &'source str) -> Result<(), Refusal> {
        self.entry(name, Kind::Fixed)
    }

    pub(super) fn pattern(&self, name: &str) -> Result<PatternMeaning, Refusal> {
        Ok(match self.lookup(name)? {
            Meaning::Local(_) | Meaning::BlockedLocal | Meaning::Other | Meaning::Absent => {
                PatternMeaning::Binding
            }
            Meaning::Fixed => PatternMeaning::Fixed,
            Meaning::Forbidden => PatternMeaning::Unsupported(
                "a static or non-unit constructor cannot be a bare pattern binding",
            ),
            Meaning::Unsupported(reason) => PatternMeaning::Unsupported(reason),
        })
    }

    /// Only actual lexical locals yield IDs; fixed global names stay written.
    pub(super) fn value(&self, name: &str) -> Result<Option<usize>, Refusal> {
        match self.lookup(name)? {
            Meaning::Local(id) => Ok(Some(id)),
            Meaning::BlockedLocal => Err(Refusal::Unsupported(
                "a const context cannot capture an outer dynamic local",
            )),
            Meaning::Fixed | Meaning::Other | Meaning::Forbidden | Meaning::Absent => Ok(None),
            Meaning::Unsupported(reason) => Err(Refusal::Unsupported(reason)),
        }
    }

    /// Keep the identifier's exact checked source spelling without allocation.
    pub(super) fn ident(&self, ident: &syn::Ident) -> Result<&'source str, Refusal> {
        let range = self.range(ident.span())?;
        self.meter.work(range.end - range.start)?;
        let text = self
            .source
            .get(range)
            .filter(|text| !text.is_empty())
            .ok_or(Refusal::Invariant(
                "a lexical identifier has no nonempty UTF-8 source span",
            ))?;
        let text = text.strip_prefix("r#").unwrap_or(text);
        if text.is_empty() {
            return Err(Refusal::Invariant("a raw identifier has no source name"));
        }
        Ok(text)
    }

    pub(super) fn range(&self, span: proc_macro2::Span) -> Result<std::ops::Range<usize>, Refusal> {
        self.coordinates.range(span)
    }

    pub(super) const fn coordinates(&self) -> &'meter Coordinates<'source, 'meter> {
        self.coordinates
    }

    fn inherited_prelude(&self) -> Result<(bool, bool), Refusal> {
        for frame in self.frames.iter().rev() {
            self.meter.work(1)?;
            if matches!(frame.kind, ScopeKind::Module) {
                return Ok((frame.closed_prelude, frame.no_std));
            }
        }
        Ok(self.standard.map_or((false, false), Prelude::flags))
    }

    /// Only module policy is inherited; a parent's imported values are not.
    /// Unknown expansion can alter a descendant prelude even when the
    /// standard prelude has already been disabled.
    pub(super) fn descendant_prelude(&self) -> Result<(bool, bool), Refusal> {
        let frame = self
            .frames
            .as_slice()
            .last()
            .filter(|frame| matches!(frame.kind, ScopeKind::Module))
            .ok_or(Refusal::Invariant(
                "descendant policy requires an actual module frame",
            ))?;
        for entry in &self.entries.as_slice()[frame.start..] {
            self.meter.work(1)?;
            if let Kind::Open {
                reason,
                inherited: true,
            } = entry.kind
            {
                return Err(Refusal::Unsupported(reason));
            }
        }
        Ok((frame.closed_prelude, frame.no_std))
    }

    fn entry(&mut self, name: &'source str, kind: Kind) -> Result<(), Refusal> {
        if self.frames.as_slice().is_empty() {
            return Err(Refusal::Invariant("a lexical entry requires a frame"));
        }
        self.meter.work(name.len())?;
        self.entries.push(Entry { name, kind })
    }

    fn open(&mut self, reason: &'static str, inherited: bool) -> Result<(), Refusal> {
        self.entry("", Kind::Open { reason, inherited })
    }

    pub(super) fn same_name(&self, a: &str, b: &str) -> Result<bool, Refusal> {
        names::same(a, b, self.meter)
    }

    fn lookup(&self, name: &str) -> Result<Meaning, Refusal> {
        if self.frames.as_slice().is_empty() {
            return Err(Refusal::Invariant("a lexical lookup requires a frame"));
        }
        self.meter.work(name.len())?;
        let mut end = self.entries.len();
        let mut blocked_locals = false;
        for frame in self.frames.iter().rev() {
            self.meter.work(1)?;
            let mut local = None;
            let mut fixed = false;
            let mut other = false;
            let mut forbidden = false;
            let mut uncertain = false;
            for entry in self.entries.as_slice()[frame.start..end].iter().rev() {
                self.meter.work(1)?;
                if let Kind::Open { reason, .. } = entry.kind {
                    return Ok(Meaning::Unsupported(reason));
                }
                if !self.same_name(entry.name, name)? {
                    continue;
                }
                match entry.kind {
                    Kind::Local(id) => {
                        if local.is_none() {
                            local = Some(id);
                        }
                    }
                    Kind::Fixed => fixed = true,
                    Kind::Other => other = true,
                    Kind::Forbidden => forbidden = true,
                    Kind::Uncertain => uncertain = true,
                    Kind::Open { .. } => unreachable!("open inventories were handled before names"),
                }
            }
            if uncertain || (fixed || forbidden) && (other || local.is_some()) || fixed && forbidden
            {
                return Ok(Meaning::Unsupported(
                    "the name has an unresolved, conditional or conflicting value declaration",
                ));
            }
            if let Some(id) = local {
                return Ok(if blocked_locals {
                    Meaning::BlockedLocal
                } else {
                    Meaning::Local(id)
                });
            }
            if fixed {
                return Ok(Meaning::Fixed);
            }
            if forbidden {
                return Ok(Meaning::Forbidden);
            }
            if other {
                return Ok(Meaning::Other);
            }
            if matches!(frame.kind, ScopeKind::Module) {
                return Ok(if frame.closed_prelude {
                    Meaning::Absent
                } else if self.standard.is_some() {
                    if Prelude::fixed(name, self.meter)? {
                        Meaning::Fixed
                    } else {
                        Meaning::Absent
                    }
                } else {
                    Meaning::Unsupported("the implicit prelude has no admitted value inventory")
                });
            }
            blocked_locals |= frame.local_fence;
            end = frame.start;
        }
        Err(Refusal::Invariant("lexical frames have no module boundary"))
    }

    fn attribute_work(&self, attr: &syn::Attribute) -> Result<(), Refusal> {
        self.meter.work(1)?;
        for segment in attr.path().segments.pairs().map(Pair::into_value) {
            self.meter.work(1)?;
            // Desugared doc-comment attributes can have synthetic identifier
            // spans. Their native path is used only for fixed builtin names.
            let span = self.range(segment.ident.span())?;
            self.meter.work((span.end - span.start).max(3))?;
        }
        Ok(())
    }

    fn attribute_named(&self, attr: &syn::Attribute, name: &str) -> Result<bool, Refusal> {
        self.meter.work(1)?;
        self.meter.work(name.len())?;
        Ok(attr.path().is_ident(name))
    }

    /// Only an already modeled syntax extension is admitted. This reads the
    /// native token list; it never loads, builds or expands any feature.
    fn language_features(&self, attr: &syn::Attribute, crate_root: bool) -> Result<(), Refusal> {
        if !crate_root || !matches!(attr.style, syn::AttrStyle::Inner(_)) {
            return Err(Refusal::Unsupported(
                "language features require an inner crate-root declaration",
            ));
        }
        let syn::Meta::List(list) = &attr.meta else {
            return Err(Refusal::Unsupported(
                "language features require a closed identifier list",
            ));
        };
        if !matches!(list.delimiter, syn::MacroDelimiter::Paren(_)) {
            return Err(Refusal::Unsupported(
                "language features require parenthesized identifiers",
            ));
        }
        let mut name = true;
        for token in list.tokens.clone() {
            self.meter.work(1)?;
            let span = self.range(token.span())?;
            self.meter.work(span.end - span.start)?;
            match token {
                proc_macro2::TokenTree::Ident(feature) if name && feature == "try_blocks" => {
                    name = false;
                }
                proc_macro2::TokenTree::Punct(comma) if !name && comma.as_char() == ',' => {
                    name = true;
                }
                _ => {
                    return Err(Refusal::Unsupported(
                        "a declared language feature has no admitted syntax profile",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Returns conditional presence; unknown expansion opens the entire frame.
    fn attributes(
        &mut self,
        attrs: &[syn::Attribute],
        module: bool,
        declaration: bool,
    ) -> Result<bool, Refusal> {
        let mut conditional = false;
        for attr in attrs {
            self.attribute_work(attr)?;
            if self.attribute_named(attr, "cfg")? {
                conditional = true;
            } else {
                let mut known = module
                    && (self.attribute_named(attr, "no_implicit_prelude")?
                        || self.attribute_named(attr, "no_std")?
                        || self.attribute_named(attr, "feature")?);
                if declaration && self.attribute_named(attr, "path")? {
                    known = matches!(&attr.meta, syn::Meta::NameValue(value)
                        if matches!(&value.value, syn::Expr::Lit(value) if matches!(value.lit, syn::Lit::Str(_))));
                }
                for name in ["doc", "allow", "warn", "deny", "forbid", "expect"] {
                    if known {
                        break;
                    }
                    known = self.attribute_named(attr, name)?;
                }
                if !known {
                    self.open(
                        "an unmodeled item attribute may introduce value declarations",
                        true,
                    )?;
                }
            }
        }
        Ok(conditional)
    }

    fn named(&mut self, name: &syn::Ident, kind: Kind) -> Result<(), Refusal> {
        let name = self.ident(name)?;
        self.entry(name, kind)
    }

    fn item(&mut self, item: &syn::Item) -> Result<(), Refusal> {
        self.meter.work(1)?;
        let module = matches!(item, syn::Item::Mod(_));
        let conditional = self.attributes(item_attributes(item), module, module)?;
        let kind = |kind| if conditional { Kind::Uncertain } else { kind };
        match item {
            syn::Item::Const(item) => self.named(&item.ident, kind(Kind::Fixed)),
            syn::Item::Static(item) => self.named(&item.ident, kind(Kind::Forbidden)),
            syn::Item::Fn(item) => self.named(&item.sig.ident, kind(Kind::Other)),
            syn::Item::Struct(item) => match &item.fields {
                syn::Fields::Unit => self.named(&item.ident, kind(Kind::Fixed)),
                syn::Fields::Unnamed(_) => self.named(&item.ident, kind(Kind::Forbidden)),
                syn::Fields::Named(_) => Ok(()),
            },
            syn::Item::Use(item) => self.import(&item.tree),
            syn::Item::ForeignMod(item) => {
                for item in &item.items {
                    self.meter.work(1)?;
                    match item {
                        syn::ForeignItem::Fn(item) => {
                            let uncertain =
                                self.attributes(&item.attrs, false, false)? || conditional;
                            self.named(
                                &item.sig.ident,
                                if uncertain {
                                    Kind::Uncertain
                                } else {
                                    Kind::Other
                                },
                            )?;
                        }
                        syn::ForeignItem::Static(item) => {
                            let uncertain =
                                self.attributes(&item.attrs, false, false)? || conditional;
                            self.named(
                                &item.ident,
                                if uncertain {
                                    Kind::Uncertain
                                } else {
                                    Kind::Forbidden
                                },
                            )?;
                        }
                        syn::ForeignItem::Type(item) => {
                            self.attributes(&item.attrs, false, false)?;
                        }
                        _ => self.open("an unexpanded foreign item may introduce values", true)?,
                    }
                }
                Ok(())
            }
            syn::Item::Macro(item) if item.ident.is_some() => Ok(()),
            syn::Item::Enum(_)
            | syn::Item::Union(_)
            | syn::Item::Type(_)
            | syn::Item::Trait(_)
            | syn::Item::TraitAlias(_)
            | syn::Item::Impl(_)
            | syn::Item::Mod(_)
            | syn::Item::ExternCrate(_) => Ok(()),
            _ => self.open("an unexpanded item may introduce value declarations", true),
        }
    }

    fn import(&mut self, tree: &syn::UseTree) -> Result<(), Refusal> {
        // visit_import_paths owns an unmetered identifier-prefix Vec. This
        // inventory needs aliases only, so it walks borrowed nodes instead.
        let mut pending = Scratch::new(self.meter);
        pending.push((tree, None))?;
        while let Some((tree, prefix)) = pending.pop()? {
            self.meter.work(1)?;
            match tree {
                syn::UseTree::Path(path) => {
                    pending.push((path.tree.as_ref(), Some(&path.ident)))?;
                }
                syn::UseTree::Group(group) => {
                    for item in group.items.pairs().rev().map(Pair::into_value) {
                        pending.push((item, prefix))?;
                    }
                }
                syn::UseTree::Name(name) => {
                    let spelling = self.ident(&name.ident)?;
                    if self.same_name(spelling, "self")? {
                        if let Some(prefix) = prefix {
                            self.named(prefix, Kind::Uncertain)?;
                        } else {
                            return Err(Refusal::Unsupported("a self import has no named prefix"));
                        }
                    } else {
                        self.entry(spelling, Kind::Uncertain)?;
                    }
                }
                syn::UseTree::Rename(rename) => {
                    let spelling = self.ident(&rename.rename)?;
                    if !self.same_name(spelling, "_")? {
                        self.entry(spelling, Kind::Uncertain)?;
                    }
                }
                syn::UseTree::Glob(_) => {
                    self.open("an unresolved glob may introduce values", false)?;
                }
            }
        }
        Ok(())
    }
}

fn item_attributes(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Const(item) => &item.attrs,
        syn::Item::Static(item) => &item.attrs,
        syn::Item::Fn(item) => &item.attrs,
        syn::Item::Struct(item) => &item.attrs,
        syn::Item::Enum(item) => &item.attrs,
        syn::Item::Union(item) => &item.attrs,
        syn::Item::Type(item) => &item.attrs,
        syn::Item::Trait(item) => &item.attrs,
        syn::Item::TraitAlias(item) => &item.attrs,
        syn::Item::Impl(item) => &item.attrs,
        syn::Item::Mod(item) => &item.attrs,
        syn::Item::Use(item) => &item.attrs,
        syn::Item::ForeignMod(item) => &item.attrs,
        syn::Item::ExternCrate(item) => &item.attrs,
        syn::Item::Macro(item) => &item.attrs,
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture<'source>(
        source: &'source str,
        check: impl FnOnce(&syn::File, &mut Scopes<'source, '_>) + Send,
    ) {
        let path = raf_source::RelPath::new("scopes.rs").unwrap();
        raf_source::parse_source(&path, source, |file| {
            let meter = Meter::new(100_000, 100_000);
            let coordinates = Coordinates::new(source, file.shebang.as_deref(), &meter).unwrap();
            let mut scopes = Scopes::new(source, &meter, &coordinates);
            scopes.enter_module(&file.items, &file.attrs, true).unwrap();
            check(file, &mut scopes);
        })
        .unwrap();
    }

    fn function(file: &syn::File) -> &syn::ItemFn {
        file.items
            .iter()
            .find_map(|item| match item {
                syn::Item::Fn(item) => Some(item),
                _ => None,
            })
            .expect("fixture function")
    }

    #[test]
    fn closed_source_names_separate_constants_constructors_and_type_only_items() {
        fixture(
            "//! Inventory fixture.\n#![no_implicit_prelude]\nconst fixed: u8 = 1; struct Unit; \
             struct Tuple(u8); struct Named { fixed: u8 }\n\
             fn helper() { struct fixed { field: u8 } let ordinary = 1; const later: u8 = 2; }",
            |file, scopes| {
                assert_eq!(scopes.pattern("ordinary").unwrap(), PatternMeaning::Binding);
                assert_eq!(scopes.pattern("fixed").unwrap(), PatternMeaning::Fixed);
                assert_eq!(scopes.pattern("Unit").unwrap(), PatternMeaning::Fixed);
                assert!(matches!(
                    scopes.pattern("Tuple").unwrap(),
                    PatternMeaning::Unsupported(_)
                ));
                assert_eq!(scopes.value("Tuple").unwrap(), None);
                scopes.enter_block(&function(file).block).unwrap();
                assert_eq!(scopes.pattern("fixed").unwrap(), PatternMeaning::Fixed);
                assert_eq!(scopes.pattern("later").unwrap(), PatternMeaning::Fixed);
                assert_eq!(scopes.value("fixed").unwrap(), None);
                scopes.leave().unwrap();
            },
        );
    }

    #[test]
    fn imports_conditionals_and_unexpanded_attributes_never_prove_binding_absence() {
        fixture(
            "#![no_implicit_prelude]\nuse external::value as maybe; \
             #[cfg(any())] const conditional: u8 = 0; \
             #[cfg(any())] struct Conditional; fn helper() { use external::*; }",
            |file, scopes| {
                for name in ["maybe", "conditional", "Conditional"] {
                    assert!(matches!(
                        scopes.pattern(name).unwrap(),
                        PatternMeaning::Unsupported(_)
                    ));
                }
                assert_eq!(scopes.pattern("ordinary").unwrap(), PatternMeaning::Binding);
                scopes.enter_values().unwrap();
                scopes.bind("ordinary", 7).unwrap();
                scopes.enter_block(&function(file).block).unwrap();
                assert!(matches!(
                    scopes.pattern("ordinary").unwrap(),
                    PatternMeaning::Unsupported(_)
                ));
                assert!(matches!(
                    scopes.value("ordinary"),
                    Err(Refusal::Unsupported(_))
                ));
            },
        );
        fixture(
            "#![no_implicit_prelude]\n#[derive(Unknown)] struct Named { value: u8 }",
            |_, scopes| {
                assert!(matches!(
                    scopes.pattern("ordinary").unwrap(),
                    PatternMeaning::Unsupported(_)
                ));
            },
        );
        fixture(
            "#![no_implicit_prelude]\nfn helper() { unknown!(); let ordinary = 1; }",
            |file, scopes| {
                scopes.enter_block(&function(file).block).unwrap();
                assert!(matches!(
                    scopes.pattern("ordinary").unwrap(),
                    PatternMeaning::Unsupported(_)
                ));
            },
        );
    }

    #[test]
    fn local_activation_and_scope_exit_preserve_shadow_identity_and_raw_names() {
        fixture(
            "#![no_implicit_prelude]\nfn helper() { let r#value = 1; { let value = 2; } let café = 3; }",
            |file, scopes| {
                let block = &function(file).block;
                let syn::Stmt::Local(local) = &block.stmts[0] else {
                    panic!("local")
                };
                let syn::Pat::Ident(pattern) = &local.pat else {
                    panic!("identifier")
                };
                let name = scopes.ident(&pattern.ident).unwrap();
                assert_eq!(name, "value");
                let syn::Stmt::Local(unicode) = &block.stmts[2] else {
                    panic!("Unicode local")
                };
                let syn::Pat::Ident(unicode) = &unicode.pat else {
                    panic!("Unicode identifier")
                };
                let unicode_name = scopes.ident(&unicode.ident).unwrap();
                scopes.enter_values().unwrap();
                scopes.enter_block(block).unwrap();
                assert_eq!(scopes.value(name).unwrap(), None);
                scopes.bind(name, 1).unwrap();
                assert_eq!(scopes.value(name).unwrap(), Some(1));
                let syn::Stmt::Expr(syn::Expr::Block(inner), _) = &block.stmts[1] else {
                    panic!("block")
                };
                scopes.enter_block(&inner.block).unwrap();
                assert_eq!(scopes.value(name).unwrap(), Some(1));
                scopes.bind(name, 2).unwrap();
                assert_eq!(scopes.value(name).unwrap(), Some(2));
                scopes.leave().unwrap();
                assert_eq!(scopes.value(name).unwrap(), Some(1));
                scopes.bind(unicode_name, 3).unwrap();
                assert_eq!(scopes.value("cafe\u{301}").unwrap(), Some(3));
                scopes.leave().unwrap();
                scopes.leave().unwrap();
                assert_eq!(scopes.value(name).unwrap(), None);
            },
        );
    }

    #[test]
    fn unmodeled_preludes_refuse_while_explicit_const_parameters_remain_fixed() {
        fixture("fn helper<const N: usize>() {}", |file, scopes| {
            assert!(matches!(
                scopes.pattern("ordinary").unwrap(),
                PatternMeaning::Unsupported(_)
            ));
            scopes.enter_values().unwrap();
            let syn::GenericParam::Const(parameter) = &function(file).sig.generics.params[0] else {
                panic!("const parameter")
            };
            scopes
                .constant(scopes.ident(&parameter.ident).unwrap())
                .unwrap();
            assert_eq!(scopes.pattern("N").unwrap(), PatternMeaning::Fixed);
        });
    }
}
