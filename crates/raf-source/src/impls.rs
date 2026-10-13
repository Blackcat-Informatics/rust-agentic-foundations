// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Impl blocks: the members of an impl are recorded under the type the impl is
//! for, under the canonical path of its trait when the type is not local, or,
//! for an inherent impl whose type the index cannot see, under the self path
//! written in its module.

use std::collections::BTreeSet;
use std::mem::size_of;
use std::sync::Arc;

use crate::cost::{shared_text, vec_first, vec_slot};
use crate::error::{LoadError, LoadErrorKind};
use crate::facts::{ItemKind, PathFact, TestRole};
use crate::index::{
    ImplDecl, Index, MEMBERS_BYTES, Store, UseDecl, UseKind, combine, member_bytes,
};
use crate::item::{ItemId, Namespace, SelfType, SymbolKind, TraitKey, UnresolvedImpl};
use crate::resolve::{Fail, Heads};

/// The crates every package can name without declaring them.
const SYSROOT: [&str; 5] = ["std", "core", "alloc", "proc_macro", "test"];

/// The traits of the standard prelude, the path they are defined at and the
/// first edition whose prelude has them. Paths are written with `std`, the
/// crate that re-exports every one of them, so `core::` and `alloc::` spellings
/// of a trait and the prelude spelling are one key.
const PRELUDE_TRAITS: [(&str, &str, u16); 31] = [
    ("AsMut", "std::convert::AsMut", 2015),
    ("AsRef", "std::convert::AsRef", 2015),
    ("Clone", "std::clone::Clone", 2015),
    ("Copy", "std::marker::Copy", 2015),
    ("Default", "std::default::Default", 2015),
    (
        "DoubleEndedIterator",
        "std::iter::DoubleEndedIterator",
        2015,
    ),
    ("Drop", "std::ops::Drop", 2015),
    ("Eq", "std::cmp::Eq", 2015),
    ("ExactSizeIterator", "std::iter::ExactSizeIterator", 2015),
    ("Extend", "std::iter::Extend", 2015),
    ("Fn", "std::ops::Fn", 2015),
    ("FnMut", "std::ops::FnMut", 2015),
    ("FnOnce", "std::ops::FnOnce", 2015),
    ("From", "std::convert::From", 2015),
    ("Into", "std::convert::Into", 2015),
    ("IntoIterator", "std::iter::IntoIterator", 2015),
    ("Iterator", "std::iter::Iterator", 2015),
    ("Ord", "std::cmp::Ord", 2015),
    ("PartialEq", "std::cmp::PartialEq", 2015),
    ("PartialOrd", "std::cmp::PartialOrd", 2015),
    ("Send", "std::marker::Send", 2015),
    ("Sized", "std::marker::Sized", 2015),
    ("Sync", "std::marker::Sync", 2015),
    ("ToOwned", "std::borrow::ToOwned", 2015),
    ("ToString", "std::string::ToString", 2015),
    ("Unpin", "std::marker::Unpin", 2015),
    ("FromIterator", "std::iter::FromIterator", 2021),
    ("TryFrom", "std::convert::TryFrom", 2021),
    ("TryInto", "std::convert::TryInto", 2021),
    ("Future", "std::future::Future", 2024),
    ("IntoFuture", "std::future::IntoFuture", 2024),
];

/// Why a trait path has no absolute form: `None` for the path itself, or the
/// glob whose crate is not one the package can name.
type Unknown = Option<Box<UseDecl>>;

impl Index {
    /// The local structs, enums and unions that a self type path names,
    /// following type aliases.
    fn nominal_types(
        &self,
        module: usize,
        segs: &[String],
        leading_colon: bool,
        heads: Heads,
    ) -> Vec<ItemId> {
        let mut pending: Vec<(usize, Vec<String>, bool)> =
            vec![(module, segs.to_vec(), leading_colon)];
        // Aliases can point at each other. A path that has been walked once
        // adds nothing the first walk did not, so the guard ends the cycle
        // and the caller reports a self type that names no type.
        let mut visited: BTreeSet<(usize, Vec<String>)> = BTreeSet::new();
        let mut found = BTreeSet::new();
        while let Some((at, path, colon)) = pending.pop() {
            if !visited.insert((at, path.clone())) {
                continue;
            }
            let walked = match self.walk(at, &path, colon, Namespace::Type, heads) {
                Ok(walked) => walked,
                // A path that names nothing names no type.
                Err(Fail::Missing(_) | Fail::SuperAboveRoot | Fail::SeveralCrates { .. }) => {
                    continue;
                }
            };
            for id in walked.items.keys() {
                match self.items[id.0].kind {
                    SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Union => {
                        found.insert(*id);
                    }
                    SymbolKind::TypeAlias => {
                        if let Some((home, alias)) = self.aliases.get(id) {
                            pending.push((*home, alias.segments.clone(), alias.leading_colon));
                        }
                    }
                    // These name something, but not a type an impl can add
                    // members to.
                    SymbolKind::Fn
                    | SymbolKind::Trait
                    | SymbolKind::Const
                    | SymbolKind::Static
                    | SymbolKind::Mod
                    | SymbolKind::MacroRules
                    | SymbolKind::ExternCrate
                    | SymbolKind::ForeignFn
                    | SymbolKind::ForeignStatic
                    | SymbolKind::ForeignType
                    | SymbolKind::InherentMethod
                    | SymbolKind::TraitImplMethod
                    | SymbolKind::TraitDefaultMethod
                    | SymbolKind::TraitRequiredMethod
                    | SymbolKind::AssocConst
                    | SymbolKind::AssocType => {}
                }
            }
        }
        self.sorted(found)
    }

    /// True when the impl is test code.
    fn test_code(decl: &ImplDecl<'_>) -> bool {
        decl.reach_test || decl.fact.test != TestRole::None
    }

    /// True when `name` is a crate the impl's code can name: a sysroot crate
    /// or a crate of its target's extern prelude.
    fn crate_known(&self, decl: &ImplDecl<'_>, name: &str) -> bool {
        SYSROOT.contains(&name)
            || self
                .extern_name(decl.module, name, Self::test_code(decl))
                .is_some()
    }

    /// The path of the standard prelude trait `name` for the edition of the
    /// impl's package; `None` if the name is not in that edition's prelude.
    fn prelude_trait(&self, decl: &ImplDecl<'_>, name: &str) -> Option<&'static str> {
        let target = &self.targets[decl.target.index()];
        let edition = self.editions.get(&target.package).copied();
        PRELUDE_TRAITS
            .iter()
            .find(|(trait_name, _, first)| {
                *trait_name == name && edition.is_some_and(|year| *first <= year)
            })
            .map(|(_, path, _)| *path)
    }

    /// The absolute paths a name that the module does not bind to a local item
    /// stands for: what its explicit imports say, the name of a declared
    /// crate, or what its globs could provide. A glob of a local module is
    /// looked into; a glob of anything else is a candidate that cannot be
    /// confirmed. A glob whose own path is being made absolute (`active`, by
    /// position among the imports) provides nothing to that path.
    #[allow(
        clippy::too_many_arguments,
        reason = "the search carries its depth, the names it looked for and the globs it is inside"
    )]
    fn external_prefixes(
        &self,
        decl: &ImplDecl<'_>,
        module: usize,
        name: &str,
        prelude: bool,
        depth: usize,
        visited: &mut BTreeSet<(usize, String)>,
        active: &mut BTreeSet<usize>,
    ) -> Result<Vec<Vec<String>>, Unknown> {
        if depth > self.uses.len() + 8 {
            return Err(None);
        }
        let named: Vec<&UseDecl> = self
            .uses
            .iter()
            .filter(|u| {
                u.module == module && matches!(&u.kind, UseKind::Named { name: n, .. } if n == name)
            })
            .collect();
        if !named.is_empty() {
            let mut out = Vec::new();
            for u in named {
                if let UseKind::Named { segs, .. } = &u.kind {
                    out.extend(self.absolute(
                        decl,
                        module,
                        segs,
                        u.leading_colon,
                        depth + 1,
                        active,
                    )?);
                }
            }
            return Ok(out);
        }
        let extern_crate = self.modules[module].defs.iter().find(|id| {
            self.items[id.0].kind == SymbolKind::ExternCrate && self.items[id.0].name == name
        });
        if let Some(real) = extern_crate.and_then(|id| self.extern_names.get(id)) {
            return if self.crate_known(decl, real) {
                Ok(vec![vec![real.to_string()]])
            } else {
                Err(None)
            };
        }
        // The name of a crate this package can name stands for that crate.
        if self.crate_known(decl, name) {
            return Ok(vec![vec![name.to_owned()]]);
        }
        let mut out = Vec::new();
        if visited.insert((module, name.to_owned())) {
            for (position, u) in self.uses.iter().enumerate() {
                let UseKind::Glob { segs } = &u.kind else {
                    continue;
                };
                if u.module != module || active.contains(&position) {
                    continue;
                }
                match self.glob_targets(u, segs) {
                    // A glob of a local enum brings in its variants, in the
                    // type and the value name spaces. A variant is neither a
                    // trait nor a module, so the glob gives no prefix.
                    Ok(targets) if targets.is_empty() && self.globs_local_enum(u, segs) => {}
                    Ok(targets) if !targets.is_empty() => {
                        for target in targets {
                            out.extend(self.external_prefixes(
                                decl,
                                target,
                                name,
                                prelude,
                                depth + 1,
                                visited,
                                active,
                            )?);
                        }
                    }
                    // A glob that names no local module: a dependency, if the
                    // crate is one this package can name, or a path in the
                    // crate that the index cannot see.
                    _ => {
                        active.insert(position);
                        let found =
                            self.absolute(decl, module, segs, u.leading_colon, depth + 1, active);
                        active.remove(&position);
                        let bases = match found {
                            Ok(bases) => bases,
                            Err(None) => {
                                // A glob of a path in the crate that names
                                // nothing the index can see (a module or an
                                // enum a macro generates): a candidate that
                                // cannot be confirmed, under that path.
                                let bases = self.unseen_local(module, segs, u.leading_colon);
                                if bases.is_empty() {
                                    // A glob of something that is neither a
                                    // local path nor a crate the package can
                                    // name.
                                    return Err(Some(Box::new(u.clone())));
                                }
                                bases
                            }
                            Err(inner) => return Err(inner),
                        };
                        for mut base in bases {
                            base.push(name.to_owned());
                            out.push(base);
                        }
                    }
                }
            }
        }
        if let (true, Some(path)) = (prelude, self.prelude_trait(decl, name)) {
            out.push(path.split("::").map(str::to_owned).collect());
        }
        Ok(out)
    }

    /// Every absolute path that `segs`, written in `module`, could stand for.
    /// The head of each is a crate the package can name; a path whose head is
    /// none is an error.
    fn absolute(
        &self,
        decl: &ImplDecl<'_>,
        module: usize,
        segs: &[String],
        leading_colon: bool,
        depth: usize,
        active: &mut BTreeSet<usize>,
    ) -> Result<Vec<Vec<String>>, Unknown> {
        if depth > self.uses.len() + 8 || segs.is_empty() {
            return Err(None);
        }
        if leading_colon {
            return if self.crate_known(decl, &segs[0]) {
                Ok(vec![segs.to_vec()])
            } else {
                Err(None)
            };
        }
        let mut cur = module;
        let mut at = 0;
        match segs[0].as_str() {
            "crate" => {
                cur = self.modules[module].root;
                at = 1;
            }
            "self" => at = 1,
            "super" => {
                while segs.get(at).is_some_and(|s| s == "super") {
                    cur = self.modules[cur].parent.ok_or(None)?;
                    at += 1;
                }
            }
            _ => {}
        }
        while at < segs.len() {
            let seg = &segs[at];
            if let Some((bound, _)) = self.lookup(cur, Namespace::Type, seg) {
                // Every item the name is bound to is a way the path can go:
                // an `extern crate` item stands for the crate it names, and a
                // module the path goes on through; anything else bound
                // locally would have resolved the trait already. Several ways
                // give several candidates, never the first of them.
                let rest = &segs[at + 1..];
                let mut out = Vec::new();
                let mut inner: BTreeSet<usize> = BTreeSet::new();
                for id in bound.keys() {
                    if let Some(real) = self.extern_names.get(id) {
                        if !self.crate_known(decl, real) {
                            return Err(None);
                        }
                        let mut path = vec![real.to_string()];
                        path.extend(rest.iter().cloned());
                        out.push(path);
                    } else {
                        inner.extend(self.modules_for(*id));
                    }
                }
                if rest.is_empty() {
                    // A path that ends at a module names no trait; one that
                    // ends at an `extern crate` item names its crate.
                    return if out.is_empty() { Err(None) } else { Ok(out) };
                }
                if out.is_empty()
                    && let [only] = inner.iter().copied().collect::<Vec<_>>().as_slice()
                {
                    cur = *only;
                    at += 1;
                    continue;
                }
                let mut from_self = vec!["self".to_owned()];
                from_self.extend(rest.iter().cloned());
                for module in inner {
                    out.extend(self.absolute(
                        decl,
                        module,
                        &from_self,
                        false,
                        depth + 1,
                        active,
                    )?);
                }
                if out.is_empty() {
                    return Err(None);
                }
                return Ok(out);
            }
            let single = segs.len() == 1;
            let mut visited = BTreeSet::new();
            let prefixes =
                self.external_prefixes(decl, cur, seg, single, depth, &mut visited, active)?;
            if prefixes.is_empty() {
                return Err(None);
            }
            return Ok(prefixes
                .into_iter()
                .map(|mut prefix| {
                    prefix.extend(segs.iter().skip(at + 1).cloned());
                    prefix
                })
                .collect());
        }
        Err(None)
    }

    /// The key of the trait of an impl that is not on a local type: a local
    /// trait at its definition, a dependency trait at its absolute path, or an
    /// explicit unresolved key when globs of dependencies leave the path open.
    fn trait_key(&self, decl: &ImplDecl<'_>, tr: &PathFact) -> Result<TraitKey, LoadError> {
        let unresolved = || {
            Index::error(
                decl,
                LoadErrorKind::UnresolvedTrait {
                    path: tr.segments.join("::"),
                },
            )
        };
        let written = || {
            format!(
                "{}{}",
                if tr.leading_colon { "::" } else { "" },
                tr.segments.join("::")
            )
        };
        match self.walk(
            decl.module,
            &tr.segments,
            tr.leading_colon,
            Namespace::Type,
            Heads::Code {
                test: Self::test_code(decl),
            },
        ) {
            Ok(walked) => {
                if let Some(first) = self.sorted(walked.items.keys().copied()).first() {
                    return Ok(TraitKey::Canonical(self.items[first.0].path.to_string()));
                }
            }
            // A crate name that stands for several libraries: the trait is in
            // one of them, and the key lists the path in each.
            Err(Fail::SeveralCrates { roots, rest }) => {
                let mut candidates: Vec<String> = roots
                    .iter()
                    .map(|root| {
                        let mut path = self.items[root.0].path.to_string();
                        for segment in tr.segments.iter().skip(rest) {
                            path.push_str("::");
                            path.push_str(segment);
                        }
                        path
                    })
                    .collect();
                candidates.sort();
                candidates.dedup();
                return Ok(TraitKey::Unresolved {
                    written: written(),
                    candidates,
                });
            }
            Err(Fail::Missing(_) | Fail::SuperAboveRoot) => {}
        }
        let paths = self
            .absolute(
                decl,
                decl.module,
                &tr.segments,
                tr.leading_colon,
                0,
                &mut BTreeSet::new(),
            )
            .map_err(|bad| match bad {
                None => unresolved(),
                Some(glob) => LoadError::new(
                    &glob.file,
                    Some(glob.line),
                    LoadErrorKind::UnresolvedTrait {
                        path: match &glob.kind {
                            UseKind::Glob { segs } | UseKind::Named { segs, .. } => {
                                format!("{}::*", segs.join("::"))
                            }
                        },
                    },
                ),
            })?;
        // A candidate is a path of a crate the impl's code can name, or a path
        // in the impl's own crate that the index cannot see.
        let own = &self.modules[self.modules[decl.module].root].path;
        let in_own_crate = |path: &[String]| {
            path.join("::")
                .strip_prefix(own.as_str())
                .is_some_and(|rest| rest.starts_with("::"))
        };
        let mut candidates: Vec<String> = paths
            .into_iter()
            .filter(|path| self.crate_known(decl, &path[0]) || in_own_crate(path))
            .map(|mut path| {
                // `core` and `alloc` define what `std` re-exports under the
                // same path, so the three spellings are one key.
                if matches!(path[0].as_str(), "core" | "alloc") {
                    "std".clone_into(&mut path[0]);
                }
                path.join("::")
            })
            .collect();
        candidates.sort();
        candidates.dedup();
        match candidates.len() {
            0 => Err(unresolved()),
            1 => Ok(TraitKey::Canonical(candidates.remove(0))),
            _ => Ok(TraitKey::Unresolved {
                written: written(),
                candidates,
            }),
        }
    }

    fn super_check_impl(&self, decl: &ImplDecl<'_>, path: &PathFact) -> Result<(), LoadError> {
        let skip = usize::from(path.segments.first().is_some_and(|s| s == "self"));
        let supers = path
            .segments
            .iter()
            .skip(skip)
            .take_while(|s| *s == "super")
            .count();
        if !path.leading_colon && supers > self.module_depth(decl.module) {
            return Err(Index::error(
                decl,
                LoadErrorKind::SuperAboveRoot {
                    path: path.segments.join("::"),
                },
            ));
        }
        Ok(())
    }

    /// The structs, enums and unions of the workspace that the self type of
    /// an impl names; none for a self type that has no path or is one of the
    /// impl's generic parameters.
    fn local_types(
        &self,
        decl: &ImplDecl<'_>,
        own: Option<&PathFact>,
    ) -> Result<Vec<ItemId>, LoadError> {
        match own {
            Some(path)
                if !(path.segments.len() == 1
                    && decl.fact.generics.contains(&path.segments[0])) =>
            {
                self.super_check_impl(decl, path)?;
                Ok(self.nominal_types(
                    decl.module,
                    &path.segments,
                    path.leading_colon,
                    Heads::Code {
                        test: Self::test_code(decl),
                    },
                ))
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Attach the members of every impl block, charging each member to the
    /// item-bytes budget.
    pub(crate) fn attach_impls(
        &mut self,
        impls: Vec<ImplDecl<'_>>,
        store: &mut Store<'_>,
    ) -> Result<(), LoadError> {
        for decl in impls {
            let fact = decl.fact;
            let own = Index::path_fact(fact);
            let local = self.local_types(&decl, own.as_ref())?;
            let self_type: String = fact.self_type.split_whitespace().collect();
            if fact.trait_segments.is_none() && local.is_empty() {
                self.attach_unresolved(&decl, own.as_ref(), &self_type, store)?;
                continue;
            }
            let trait_key = match (&fact.trait_segments, local.is_empty()) {
                (Some(tr), true) => Some(Arc::new(self.trait_key(&decl, tr)?)),
                _ => None,
            };
            if let Some(key) = &trait_key {
                // One key for the members of the block, charged once.
                let bytes = match &**key {
                    TraitKey::Canonical(path) => path.capacity(),
                    TraitKey::Unresolved {
                        written,
                        candidates,
                    } => {
                        written.capacity()
                            + candidates.capacity() * size_of::<String>()
                            + candidates.iter().map(String::capacity).sum::<usize>()
                    }
                };
                store.charge(
                    shared_text(size_of::<TraitKey>()) + bytes,
                    &decl.file,
                    Some(fact.line),
                )?;
            }
            let anchor = local.first().map(|id| self.items[id.0].path.clone());
            for member in &fact.items {
                let kind = match (member.kind, fact.trait_segments.is_some()) {
                    (ItemKind::Fn, false) => SymbolKind::InherentMethod,
                    (ItemKind::Fn, true) => SymbolKind::TraitImplMethod,
                    (ItemKind::TypeAlias, _) => SymbolKind::AssocType,
                    _ => SymbolKind::AssocConst,
                };
                let (path, self_key) = match (&anchor, trait_key.as_deref()) {
                    (Some(base), _) => (format!("{base}::{}", member.name), SelfType::Local),
                    (None, Some(TraitKey::Canonical(key))) => {
                        (format!("{key}::{}", member.name), SelfType::Trait)
                    }
                    (None, Some(TraitKey::Unresolved { written, .. })) => {
                        (format!("{written}::{}", member.name), SelfType::Trait)
                    }
                    (None, None) => {
                        return Err(Index::error(
                            &decl,
                            LoadErrorKind::Internal {
                                what: "an impl without a local type or a trait was attached"
                                    .to_owned(),
                            },
                        ));
                    }
                };
                let id = self.push_member(
                    &decl,
                    member,
                    kind,
                    &path,
                    &self_type,
                    self_key,
                    trait_key.clone(),
                    store,
                )?;
                if local.is_empty() {
                    store.charge(member_bytes(&path), &decl.file, Some(member.name_line))?;
                    self.foreign.entry(path).or_default().push(id);
                } else {
                    for ty in &local {
                        let table = if self.assoc.contains_key(ty) {
                            0
                        } else {
                            MEMBERS_BYTES
                        };
                        store.charge(
                            member_bytes(&member.name) + table,
                            &decl.file,
                            Some(member.name_line),
                        )?;
                        self.assoc
                            .entry(*ty)
                            .or_default()
                            .entry(member.name.clone())
                            .or_default()
                            .push(id);
                    }
                }
            }
        }
        Ok(())
    }

    /// True when the glob path of `decl` names local enums only.
    fn globs_local_enum(&self, decl: &UseDecl, segs: &[String]) -> bool {
        self.walk(
            decl.module,
            segs,
            decl.leading_colon,
            Namespace::Type,
            Heads::Code { test: decl.test },
        )
        .is_ok_and(|walked| {
            !walked.items.is_empty()
                && walked
                    .items
                    .keys()
                    .all(|id| self.items[id.0].kind == SymbolKind::Enum)
        })
    }

    /// The self path of an impl, written in its module: the module path and
    /// the path the self type is written with (`crate`, `self` and `super`
    /// taken from the module; a path with a leading `::` from its first
    /// segment, alone), or the
    /// module path and the written self type when the self type has no path.
    fn written_self_path(
        &self,
        decl: &ImplDecl<'_>,
        own: Option<&PathFact>,
        self_type: &str,
    ) -> Result<String, LoadError> {
        let Some(path) = own else {
            return Ok(format!("{}::{self_type}", self.modules[decl.module].path));
        };
        if path.leading_colon {
            return Ok(path.segments.join("::"));
        }
        let mut at = decl.module;
        let mut rest = path.segments.as_slice();
        match rest.first().map(String::as_str) {
            Some("crate") => {
                at = self.modules[at].root;
                rest = &rest[1..];
            }
            Some("self") => rest = &rest[1..],
            _ => {}
        }
        while rest.first().is_some_and(|segment| segment == "super") {
            // `super` above the crate root was refused before the impl was
            // attached.
            at = self.modules[at].parent.ok_or_else(|| {
                Index::error(
                    decl,
                    LoadErrorKind::Internal {
                        what: "a self path climbs above the crate root".to_owned(),
                    },
                )
            })?;
            rest = &rest[1..];
        }
        let mut written = self.modules[at].path.to_string();
        for segment in rest {
            written.push_str("::");
            written.push_str(segment);
        }
        Ok(written)
    }

    /// Record the members of an inherent impl whose self type names no
    /// struct, enum or union the index can see under the self path written
    /// in its module, marked [`SelfType::Unresolved`], and the impl among the
    /// unresolved impls. No scope or type names the members.
    fn attach_unresolved(
        &mut self,
        decl: &ImplDecl<'_>,
        own: Option<&PathFact>,
        self_type: &str,
        store: &mut Store<'_>,
    ) -> Result<(), LoadError> {
        let fact = decl.fact;
        let written = self.written_self_path(decl, own, self_type)?;
        let (path, path_cost) = store.intern(&written);
        let (shared_type, type_cost) = store.intern(self_type);
        store.charge(
            vec_slot::<UnresolvedImpl>()
                + path_cost
                + type_cost
                + if fact.items.is_empty() {
                    0
                } else {
                    vec_first::<ItemId>()
                },
            &decl.file,
            Some(fact.line),
        )?;
        let mut members = Vec::new();
        for member in &fact.items {
            let kind = match member.kind {
                ItemKind::Fn => SymbolKind::InherentMethod,
                ItemKind::TypeAlias => SymbolKind::AssocType,
                _ => SymbolKind::AssocConst,
            };
            store.charge(vec_slot::<ItemId>(), &decl.file, Some(member.name_line))?;
            let id = self.push_member(
                decl,
                member,
                kind,
                &format!("{written}::{}", member.name),
                self_type,
                SelfType::Unresolved,
                None,
                store,
            )?;
            members.push(id);
        }
        self.unresolved.push(UnresolvedImpl {
            path,
            self_type: shared_type,
            file: decl.file.clone(),
            line: fact.line,
            span: fact.span,
            target: decl.target,
            test: combine(fact.test, decl.reach_test),
            members,
        });
        Ok(())
    }

    /// The paths in the crate of a glob path written in `module` that starts
    /// in the crate (`crate`, `self`, `super`, or a local module the module
    /// binds) and goes on through names the index cannot see: for each way
    /// the path can go, the path of the last local module it reaches,
    /// followed by the rest as written. A name bound to several modules gives
    /// a path through each. None for a path that starts with `::` or with a
    /// name that is not a local module.
    fn unseen_local(
        &self,
        module: usize,
        segs: &[String],
        leading_colon: bool,
    ) -> Vec<Vec<String>> {
        if leading_colon {
            return Vec::new();
        }
        let bound_modules = |at: usize, name: &str| -> BTreeSet<usize> {
            self.lookup(at, Namespace::Type, name)
                .map(|(bound, _)| bound.keys().flat_map(|id| self.modules_for(*id)).collect())
                .unwrap_or_default()
        };
        let mut work: Vec<(usize, &[String])> = Vec::new();
        match segs.first().map(String::as_str) {
            None => return Vec::new(),
            Some("crate") => work.push((self.modules[module].root, &segs[1..])),
            Some("self") => work.push((module, &segs[1..])),
            Some("super") => work.push((module, segs)),
            Some(first) => {
                for start in bound_modules(module, first) {
                    work.push((start, &segs[1..]));
                }
            }
        }
        let mut out = Vec::new();
        'ways: while let Some((mut at, mut rest)) = work.pop() {
            while rest.first().is_some_and(|segment| segment == "super") {
                // `super` above the crate root is refused for every import
                // before a trait key consults it.
                let Some(parent) = self.modules[at].parent else {
                    continue 'ways;
                };
                at = parent;
                rest = &rest[1..];
            }
            while let Some((first, tail)) = rest.split_first() {
                let mut inner = bound_modules(at, first).into_iter();
                let Some(next) = inner.next() else {
                    break;
                };
                for other in inner {
                    work.push((other, tail));
                }
                at = next;
                rest = tail;
            }
            let mut path: Vec<String> = self.modules[at]
                .path
                .split("::")
                .map(str::to_owned)
                .collect();
            path.extend(rest.iter().cloned());
            out.push(path);
        }
        out.sort();
        out.dedup();
        out
    }
}
