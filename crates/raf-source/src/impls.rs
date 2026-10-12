// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Impl blocks: the members of an impl are recorded under the type the impl is
//! for, or under the canonical path of its trait when the type is not local.

use std::collections::BTreeSet;
use std::mem::size_of;
use std::sync::Arc;

use crate::cost::shared_text;
use crate::error::{LoadError, LoadErrorKind};
use crate::facts::{ItemKind, PathFact, TestRole};
use crate::index::{ImplDecl, Index, MEMBERS_BYTES, Store, UseDecl, UseKind, member_bytes};
use crate::item::{ItemId, Namespace, SymbolKind, TraitKey};
use crate::resolve::Fail;
use crate::target::TargetKind;

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
    fn nominal_types(&self, module: usize, segs: &[String], leading_colon: bool) -> Vec<ItemId> {
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
            let walked = match self.walk(at, &path, colon, Namespace::Type, true) {
                Ok(walked) => walked,
                // A path that names nothing names no type.
                Err(Fail::Missing(_) | Fail::SuperAboveRoot) => continue,
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

    /// True when `name` is a crate the impl's package can name.
    fn crate_known(&self, decl: &ImplDecl<'_>, name: &str) -> bool {
        if SYSROOT.contains(&name) {
            return true;
        }
        let target = &self.targets[decl.target.index()];
        let Some(info) = self.packages.get(&target.package) else {
            return false;
        };
        let test_code = decl.reach_test || decl.fact.test != TestRole::None;
        let dependencies = &info.dependencies;
        match target.kind {
            TargetKind::BuildScript => dependencies.build.contains(name),
            TargetKind::Test | TargetKind::Bench | TargetKind::Example => {
                dependencies.normal.contains(name) || dependencies.dev.contains(name)
            }
            TargetKind::Lib | TargetKind::Bin => {
                dependencies.normal.contains(name) || (test_code && dependencies.dev.contains(name))
            }
        }
    }

    /// The path of the standard prelude trait `name` for the edition of the
    /// impl's package; `None` if the name is not in that edition's prelude.
    fn prelude_trait(&self, decl: &ImplDecl<'_>, name: &str) -> Option<&'static str> {
        let target = &self.targets[decl.target.index()];
        let edition = self.packages.get(&target.package).map(|info| info.edition);
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
    /// confirmed.
    fn external_prefixes(
        &self,
        decl: &ImplDecl<'_>,
        module: usize,
        name: &str,
        prelude: bool,
        depth: usize,
        visited: &mut BTreeSet<(usize, String)>,
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
                    out.extend(self.absolute(decl, module, segs, u.leading_colon, depth + 1)?);
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
            for u in &self.uses {
                let UseKind::Glob { segs } = &u.kind else {
                    continue;
                };
                if u.module != module {
                    continue;
                }
                match self.glob_targets(u, segs) {
                    Ok(targets) if !targets.is_empty() => {
                        for target in targets {
                            out.extend(self.external_prefixes(
                                decl,
                                target,
                                name,
                                prelude,
                                depth + 1,
                                visited,
                            )?);
                        }
                    }
                    // A glob that names no local module: a dependency, if the
                    // crate is one this package can name.
                    _ => {
                        let bases =
                            match self.absolute(decl, module, segs, u.leading_colon, depth + 1) {
                                Ok(bases) => bases,
                                // A glob of something that is neither a local
                                // module nor a crate the package can name.
                                Err(None) => return Err(Some(Box::new(u.clone()))),
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
                // An `extern crate` item stands for the crate it names.
                if let Some(real) = bound.keys().find_map(|id| self.extern_names.get(id)) {
                    if !self.crate_known(decl, real) {
                        return Err(None);
                    }
                    let mut path = vec![real.to_string()];
                    path.extend(segs.iter().skip(at + 1).cloned());
                    return Ok(vec![path]);
                }
                // A module the path goes on through; anything else bound
                // locally would have resolved the trait already.
                let next = bound.keys().find_map(|id| self.module_for(*id));
                match (next, at + 1 < segs.len()) {
                    (Some(inner), true) => {
                        cur = inner;
                        at += 1;
                        continue;
                    }
                    _ => return Err(None),
                }
            }
            let single = segs.len() == 1;
            let mut visited = BTreeSet::new();
            let prefixes = self.external_prefixes(decl, cur, seg, single, depth, &mut visited)?;
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
        if let Ok(walked) = self.walk(
            decl.module,
            &tr.segments,
            tr.leading_colon,
            Namespace::Type,
            true,
        ) && let Some(first) = self.sorted(walked.items.keys().copied()).first()
        {
            return Ok(TraitKey::Canonical(self.items[first.0].path.to_string()));
        }
        let paths = self
            .absolute(decl, decl.module, &tr.segments, tr.leading_colon, 0)
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
        let mut candidates: Vec<String> = paths
            .into_iter()
            .filter(|path| self.crate_known(decl, &path[0]))
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
                written: format!(
                    "{}{}",
                    if tr.leading_colon { "::" } else { "" },
                    tr.segments.join("::")
                ),
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
            let local = match &own {
                Some(path)
                    if !(path.segments.len() == 1 && fact.generics.contains(&path.segments[0])) =>
                {
                    self.super_check_impl(&decl, path)?;
                    self.nominal_types(decl.module, &path.segments, path.leading_colon)
                }
                _ => Vec::new(),
            };
            let self_type: String = fact.self_type.split_whitespace().collect();
            let trait_key = match (&fact.trait_segments, local.is_empty()) {
                (Some(tr), true) => Some(Arc::new(self.trait_key(&decl, tr)?)),
                _ => None,
            };
            if fact.trait_segments.is_none() && local.is_empty() {
                return Err(Index::error(
                    &decl,
                    LoadErrorKind::InherentImplSelf {
                        self_type: fact.self_type.clone(),
                    },
                ));
            }
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
                let path = match (&anchor, trait_key.as_deref()) {
                    (Some(base), _) => format!("{base}::{}", member.name),
                    (None, Some(TraitKey::Canonical(key))) => format!("{key}::{}", member.name),
                    (None, Some(TraitKey::Unresolved { written, .. })) => {
                        format!("{written}::{}", member.name)
                    }
                    // An inherent impl without a local type was refused above.
                    (None, None) => continue,
                };
                let id = self.push_member(
                    &decl,
                    member,
                    kind,
                    &path,
                    &self_type,
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
}
