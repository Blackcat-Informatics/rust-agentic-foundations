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
use crate::item::{ItemId, Namespace, SelfType, SharedStr, SymbolKind, TraitKey, UnresolvedImpl};
use crate::meter::Steps;
use crate::path::RelPath;
use crate::resolve::{Fail, Heads, LOAD_WALK, Walked};

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

/// How a path that a trait key follows is read.
#[derive(Debug, Clone, Copy)]
enum Reading {
    /// The trait path of an impl, written in code.
    Code,
    /// The path of a `use` declaration; `own` is the named import whose path
    /// it is.
    Use { own: Option<usize> },
}

impl Reading {
    /// What the first segment may name, for test code (`test`) or other code.
    fn heads(self, test: bool) -> Heads {
        match self {
            Self::Code => Heads::Code { test },
            Self::Use { own } => Heads::Use { test, own },
        }
    }

    /// The named import whose path this is.
    fn own(self) -> Option<usize> {
        match self {
            Self::Code => None,
            Self::Use { own } => own,
        }
    }
}

/// A name the search looks up in a module that does not bind it to a local
/// item.
#[derive(Debug)]
struct Prefixes {
    module: usize,
    name: String,
    /// True when the name is a whole path, so a prelude trait can stand for
    /// it.
    prelude: bool,
    /// The named import whose own path looks the name up: it is not a way
    /// the name goes.
    skip: Option<usize>,
    /// The scope of the search through globs this lookup belongs to.
    visited: usize,
    /// True for a name read from the crate root in edition 2015, which names
    /// a crate only through an `extern crate` item or as the crate the
    /// compiler puts at the root.
    root_only: bool,
}

/// What the search tracks across its frames.
#[derive(Debug, Default)]
struct Search {
    /// For each name a path looks up, the modules and names its search
    /// through globs has looked into, innermost last.
    visited: Vec<BTreeSet<(usize, String)>>,
    /// The globs whose own paths are being made absolute.
    globs: BTreeSet<usize>,
    /// The named imports whose own paths are being made absolute.
    named: BTreeSet<usize>,
    /// The globs met that are neither a local path nor a crate the impl's
    /// code can name, by written path, file and line.
    failed: BTreeSet<(String, RelPath, usize, usize)>,
}

/// A call the search makes on its explicit stack.
#[derive(Debug)]
enum Call {
    /// Make a path, written in a module, absolute.
    Absolute {
        module: usize,
        segs: Vec<String>,
        leading_colon: bool,
        reading: Reading,
    },
    /// Find what a name stands for in a module that does not bind it.
    Prefixes(Prefixes),
}

/// What a frame waits for the result of its call to go on with.
#[derive(Debug)]
enum Frame {
    /// A path whose segment looked up gives prefixes, to be followed by
    /// `tail`.
    Tail { tail: Vec<String> },
    /// A path that can go on in several modules: the paths found so far and
    /// the modules still to go through, each with `from_self`.
    Inner {
        out: Vec<Vec<String>>,
        inner: Vec<usize>,
        from_self: Vec<String>,
        reading: Reading,
    },
    /// The explicit imports of a name: what they gave so far, those still to
    /// follow and the one being followed.
    Named {
        at: Prefixes,
        named: Vec<usize>,
        out: Vec<Vec<String>>,
        position: usize,
    },
    /// A glob of local modules: what the globs gave so far, the globs and
    /// the modules of this glob still to look into.
    Target {
        at: Prefixes,
        globs: Vec<usize>,
        out: Vec<Vec<String>>,
        targets: Vec<usize>,
    },
    /// A glob of something else, whose own path is being made absolute.
    Glob {
        at: Prefixes,
        globs: Vec<usize>,
        out: Vec<Vec<String>>,
        position: usize,
    },
}

/// The next move of the search: make a call and wait in a frame, or return.
#[derive(Debug)]
enum Step {
    Call(Call, Frame),
    Return(Result<Vec<Vec<String>>, Unknown>),
}

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
            let walked = match self.walk(at, &path, colon, Namespace::Type, heads, Steps::Load) {
                Ok(walked) => walked,
                // A path that names nothing names no type.
                Err(
                    Fail::Missing(_)
                    | Fail::SuperAboveRoot
                    | Fail::SeveralCrates { .. }
                    | Fail::Ambiguous(_),
                ) => {
                    continue;
                }
                Err(Fail::Budget(_)) => unreachable!("{LOAD_WALK}"),
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

    /// The readings of an explicit import's path, written in `module`, whose
    /// first segment a glob of the module brings in from an import outside
    /// the index and that is also a crate of the extern prelude: rustc's
    /// E0659, so the path through the crate and the path through each such
    /// import are candidates. `None` when the first segment is not such.
    fn carried_readings(
        &self,
        decl: &ImplDecl<'_>,
        module: usize,
        segs: &[String],
    ) -> Option<Vec<Vec<String>>> {
        let (head, rest) = segs.split_first()?;
        if self.edition(module) == 2015 || !self.in_use_prelude(module, head, Self::test_code(decl))
        {
            return None;
        }
        let carried = self.globbed_outside(module, head, Steps::Load);
        if carried.is_empty() {
            return None;
        }
        let mut out: Vec<Vec<String>> = Vec::new();
        if let Some(real) = self.crate_reading(decl, module, head) {
            let mut path = vec![real];
            path.extend(rest.iter().cloned());
            out.push(path);
        }
        for position in carried {
            let (Some(real), UseKind::Named { segs: through, .. }) = (
                self.outside_crate(position, Steps::Load),
                &self.uses[position].kind,
            ) else {
                continue;
            };
            let mut path = vec![real];
            path.extend(through.iter().skip(1).cloned());
            path.extend(rest.iter().cloned());
            out.push(path);
        }
        Some(out)
    }

    /// The key a walk to local items gives: their path when they are at one
    /// (`cfg` alternatives are one trait), every path as a candidate when
    /// they are at several, never the first of them; `None` for no item.
    fn walked_key(&self, walked: &Walked, written: impl Fn() -> String) -> Option<TraitKey> {
        let mut paths: Vec<String> = self
            .sorted(walked.items.keys().copied())
            .iter()
            .map(|id| self.items[id.0].path.to_string())
            .collect();
        paths.dedup();
        match paths.len() {
            0 => None,
            1 => Some(TraitKey::Canonical(paths.remove(0))),
            _ => Some(TraitKey::Unresolved {
                written: written(),
                candidates: paths,
            }),
        }
    }

    /// The crate that `name`, the first segment of a `use` path in `module`,
    /// stands for as a crate of the extern prelude: a crate the impl's code
    /// can name, or the crate an `extern crate` item at the crate root names.
    fn crate_reading(&self, decl: &ImplDecl<'_>, module: usize, name: &str) -> Option<String> {
        if self.crate_known(decl, name) {
            return Some(name.to_owned());
        }
        let root = self.modules[module].root;
        self.modules[root]
            .defs
            .iter()
            .filter(|id| {
                self.items[id.0].kind == SymbolKind::ExternCrate && self.items[id.0].name == name
            })
            .filter_map(|id| self.extern_names.get(id))
            .find(|real| self.crate_known(decl, real))
            .map(ToString::to_string)
    }

    /// The path of the standard prelude trait `name` for the edition of the
    /// impl's package; `None` if the name is not in that edition's prelude.
    fn prelude_trait(&self, decl: &ImplDecl<'_>, name: &str) -> Option<&'static str> {
        let edition = self.target_editions[decl.target.index()];
        PRELUDE_TRAITS
            .iter()
            .find(|(trait_name, _, first)| *trait_name == name && *first <= edition)
            .map(|(_, path, _)| *path)
    }

    /// Every absolute path that `segs`, written in `module` and read as
    /// `reading` says, could stand for. The head of each is a crate the
    /// impl's code can name, or the path is in the impl's own crate where the
    /// index cannot see; a path that has neither is an error.
    ///
    /// The search follows explicit imports, `extern crate` items and globs
    /// from module to module. It runs on an explicit stack of [`Frame`]s, so
    /// its depth on the caller's stack does not grow with the input. An
    /// import never resolves through its own binding, a glob whose own path
    /// is being made absolute provides nothing to that path, and an explicit
    /// import met again while its own path is being made absolute is a cycle,
    /// which is an error.
    fn absolute(
        &self,
        decl: &ImplDecl<'_>,
        module: usize,
        segs: &[String],
        leading_colon: bool,
        reading: Reading,
    ) -> Result<Vec<Vec<String>>, Unknown> {
        let mut search = Search::default();
        let mut stack: Vec<Frame> = Vec::new();
        let mut step = self.enter_absolute(
            decl,
            &mut search,
            module,
            segs.to_vec(),
            leading_colon,
            reading,
        );
        loop {
            step = match step {
                Step::Call(call, frame) => {
                    stack.push(frame);
                    match call {
                        Call::Absolute {
                            module,
                            segs,
                            leading_colon,
                            reading,
                        } => self.enter_absolute(
                            decl,
                            &mut search,
                            module,
                            segs,
                            leading_colon,
                            reading,
                        ),
                        Call::Prefixes(prefixes) => {
                            self.enter_prefixes(decl, &mut search, prefixes)
                        }
                    }
                }
                Step::Return(result) => match stack.pop() {
                    // A glob that names nothing fails the path: the error
                    // names the first such glob by written path, file and
                    // line, whatever order the search met them in.
                    None => {
                        return match search.failed.first() {
                            Some((.., position)) if !matches!(result, Err(Some(_))) => {
                                Err(Some(Box::new(self.uses[*position].clone())))
                            }
                            _ => result,
                        };
                    }
                    Some(frame) => self.resume(decl, &mut search, frame, result),
                },
            };
        }
    }

    /// The module a path written in `module` is read from, and the index of
    /// its first segment to look up there: the crate root for a path read
    /// from it (`crate_relative`) or one that starts with `crate`, the module
    /// itself after `self` or for a plain name, and the module above for each
    /// `super`; `None` for `super` above the crate root.
    fn path_start(
        &self,
        module: usize,
        segs: &[String],
        crate_relative: bool,
    ) -> Option<(usize, usize)> {
        if crate_relative {
            return Some((self.modules[module].root, 0));
        }
        match segs.first().map(String::as_str) {
            Some("crate") => Some((self.modules[module].root, 1)),
            Some("self") => Some((module, 1)),
            Some("super") => {
                let mut cur = module;
                let mut at = 0;
                while segs.get(at).is_some_and(|s| s == "super") {
                    cur = self.modules[cur].parent?;
                    at += 1;
                }
                Some((cur, at))
            }
            _ => Some((module, 0)),
        }
    }

    /// Start making `segs`, written in `module`, absolute.
    fn enter_absolute(
        &self,
        decl: &ImplDecl<'_>,
        search: &mut Search,
        module: usize,
        segs: Vec<String>,
        leading_colon: bool,
        reading: Reading,
    ) -> Step {
        let Some(head) = segs.first() else {
            return Step::Return(Err(None));
        };
        let heads = reading.heads(Self::test_code(decl));
        let crate_relative = self.crate_relative(module, head, leading_colon, heads);
        if leading_colon && !crate_relative {
            return Step::Return(if self.crate_known(decl, head) {
                Ok(vec![segs])
            } else {
                Err(None)
            });
        }
        let Some((mut cur, mut at)) = self.path_start(module, &segs, crate_relative) else {
            return Step::Return(Err(None));
        };
        while at < segs.len() {
            let seg = &segs[at];
            if let Some((bound, from_glob)) =
                self.lookup_in(cur, Namespace::Type, seg, reading.own(), Steps::Load)
            {
                // Every item the name is bound to is a way the path can go:
                // an `extern crate` item stands for the crate it names, a
                // module the path goes on through, and a local trait at the
                // end of the path is a candidate under its own path (which
                // only a path an ambiguous first segment leaves open meets:
                // anything bound locally otherwise resolved the trait
                // already). Several ways give several candidates, never the
                // first of them.
                let rest = &segs[at + 1..];
                let mut out = Vec::new();
                let mut inner: BTreeSet<usize> = BTreeSet::new();
                for id in bound.keys() {
                    let item = &self.items[id.0];
                    if let Some(real) = self.extern_names.get(id) {
                        if !self.crate_known(decl, real) {
                            return Step::Return(Err(None));
                        }
                        let mut path = vec![real.to_string()];
                        path.extend(rest.iter().cloned());
                        out.push(path);
                    } else if rest.is_empty() && item.kind == SymbolKind::Trait {
                        out.push(item.path.split("::").map(str::to_owned).collect());
                    } else {
                        inner.extend(self.modules_for(*id));
                    }
                }
                // The first segment of a `use` path of edition 2018 or later
                // that a glob binds and that is also a different crate of the
                // extern prelude is ambiguous: the path through the crate is
                // a candidate too.
                if let (0, true, Reading::Use { .. }) = (at, from_glob, reading)
                    && !crate_relative
                    && self
                        .glob_against_prelude(cur, seg, Self::test_code(decl), &bound)
                        .is_some()
                    && let Some(real) = self.crate_reading(decl, cur, seg)
                {
                    let mut path = vec![real];
                    path.extend(rest.iter().cloned());
                    out.push(path);
                }
                if rest.is_empty() {
                    // A path that ends at a module names no trait; one that
                    // ends at an `extern crate` item names its crate.
                    return Step::Return(if out.is_empty() { Err(None) } else { Ok(out) });
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
                return Self::next_inner(
                    out,
                    inner.into_iter().rev().collect(),
                    from_self,
                    reading,
                );
            }
            if let (0, Reading::Use { own: Some(_) }) = (at, reading)
                && !crate_relative
                && let Some(out) = self.carried_readings(decl, cur, &segs)
            {
                return Step::Return(Ok(out));
            }
            // The scope of the search through the module's globs, fresh for
            // each name a path looks up.
            search.visited.push(BTreeSet::new());
            let prefixes = Prefixes {
                module: cur,
                name: seg.clone(),
                prelude: segs.len() == 1,
                skip: reading.own(),
                visited: search.visited.len() - 1,
                root_only: crate_relative && at == 0,
            };
            return Step::Call(
                Call::Prefixes(prefixes),
                Frame::Tail {
                    tail: segs[at + 1..].to_vec(),
                },
            );
        }
        Step::Return(Err(None))
    }

    /// Go on through the next of the `inner` modules a path can go on in, or
    /// return what the ways found.
    fn next_inner(
        out: Vec<Vec<String>>,
        mut inner: Vec<usize>,
        from_self: Vec<String>,
        reading: Reading,
    ) -> Step {
        let Some(module) = inner.pop() else {
            return Step::Return(if out.is_empty() { Err(None) } else { Ok(out) });
        };
        Step::Call(
            Call::Absolute {
                module,
                segs: from_self.clone(),
                leading_colon: false,
                reading,
            },
            Frame::Inner {
                out,
                inner,
                from_self,
                reading,
            },
        )
    }

    /// Start finding the absolute paths a name that the module does not bind
    /// to a local item stands for: what its explicit imports say (but the one
    /// whose own path asks), the name of a declared crate, or what its globs
    /// could provide. A glob of a local module is looked into; a glob of
    /// anything else is a candidate that cannot be confirmed.
    fn enter_prefixes(&self, decl: &ImplDecl<'_>, search: &mut Search, at: Prefixes) -> Step {
        let named: Vec<usize> = self
            .imports_of(at.module, &at.name)
            .iter()
            .copied()
            .filter(|position| Some(*position) != at.skip)
            .rev()
            .collect();
        if !named.is_empty() {
            return self.next_named(search, at, named, Vec::new());
        }
        // Every `extern crate` item of the name stands for its crate (`cfg`
        // alternatives can name two), never the first of them.
        let crates: Vec<&SharedStr> = self.modules[at.module]
            .defs
            .iter()
            .filter(|id| {
                self.items[id.0].kind == SymbolKind::ExternCrate && self.items[id.0].name == at.name
            })
            .filter_map(|id| self.extern_names.get(id))
            .collect();
        if !crates.is_empty() {
            return Step::Return(if crates.iter().all(|real| self.crate_known(decl, real)) {
                Ok(crates.iter().map(|real| vec![real.to_string()]).collect())
            } else {
                Err(None)
            });
        }
        // The name of a crate this package can name stands for that crate. A
        // name read from the crate root in edition 2015 names only the crate
        // the compiler puts there; any other crate needs an `extern crate`
        // item.
        let crate_named = if at.root_only {
            at.name == self.injected_crate(at.module)
        } else {
            self.crate_known(decl, &at.name)
        };
        if crate_named {
            return Step::Return(Ok(vec![vec![at.name]]));
        }
        if search.visited[at.visited].insert((at.module, at.name.clone())) {
            let globs: Vec<usize> = self
                .globs_of(at.module)
                .iter()
                .copied()
                .filter(|position| !search.globs.contains(position))
                .rev()
                .collect();
            return self.next_glob(decl, search, at, globs, Vec::new());
        }
        Step::Return(Ok(self.with_prelude(decl, &at, Vec::new())))
    }

    /// Make the path of the next of the `named` imports absolute, or return
    /// what they gave.
    fn next_named(
        &self,
        search: &mut Search,
        at: Prefixes,
        mut named: Vec<usize>,
        out: Vec<Vec<String>>,
    ) -> Step {
        // An import met again while its own path is being made absolute adds
        // nothing to this way, as a glob met again does: a cycle of imports
        // binds nothing, and a name it alone could give is not found.
        let position = loop {
            let Some(position) = named.pop() else {
                return Step::Return(Ok(out));
            };
            if search.named.insert(position) {
                break position;
            }
        };
        let u = &self.uses[position];
        let UseKind::Named { segs, .. } = &u.kind else {
            return Step::Return(Err(None));
        };
        Step::Call(
            Call::Absolute {
                module: at.module,
                segs: segs.clone(),
                leading_colon: u.leading_colon,
                reading: Reading::Use {
                    own: Some(position),
                },
            },
            Frame::Named {
                at,
                named,
                out,
                position,
            },
        )
    }

    /// Look into the next of the `globs` of the module, or return what they
    /// gave with the prelude trait of the name.
    fn next_glob(
        &self,
        decl: &ImplDecl<'_>,
        search: &mut Search,
        at: Prefixes,
        mut globs: Vec<usize>,
        out: Vec<Vec<String>>,
    ) -> Step {
        while let Some(position) = globs.pop() {
            let u = &self.uses[position];
            let UseKind::Glob { segs } = &u.kind else {
                continue;
            };
            match self.glob_targets(position, Steps::Load) {
                // A glob of a local enum brings in its variants, in the type
                // and the value name spaces. A variant is neither a trait nor
                // a module, so the glob gives no prefix.
                Ok(targets) if targets.is_empty() && self.globs_local_enum(u, segs) => {}
                Ok(targets) if !targets.is_empty() => {
                    return self.next_target(
                        decl,
                        search,
                        at,
                        globs,
                        out,
                        targets.into_iter().rev().collect(),
                    );
                }
                // A glob that names no local module: a dependency, if the
                // crate is one this package can name, or a path in the crate
                // that the index cannot see.
                _ => {
                    search.globs.insert(position);
                    return Step::Call(
                        Call::Absolute {
                            module: at.module,
                            segs: segs.clone(),
                            leading_colon: u.leading_colon,
                            reading: Reading::Use { own: None },
                        },
                        Frame::Glob {
                            at,
                            globs,
                            out,
                            position,
                        },
                    );
                }
            }
        }
        let out = self.with_prelude(decl, &at, out);
        Step::Return(Ok(out))
    }

    /// Look for the name in the next of the `targets` of a glob of local
    /// modules, or go on with the next glob.
    fn next_target(
        &self,
        decl: &ImplDecl<'_>,
        search: &mut Search,
        at: Prefixes,
        globs: Vec<usize>,
        out: Vec<Vec<String>>,
        mut targets: Vec<usize>,
    ) -> Step {
        let Some(target) = targets.pop() else {
            return self.next_glob(decl, search, at, globs, out);
        };
        let call = Prefixes {
            module: target,
            name: at.name.clone(),
            prelude: at.prelude,
            skip: None,
            visited: at.visited,
            root_only: at.root_only,
        };
        Step::Call(
            Call::Prefixes(call),
            Frame::Target {
                at,
                globs,
                out,
                targets,
            },
        )
    }

    /// `out` with the path of the prelude trait `at` names, when the name is
    /// a whole path and in the prelude of the impl's edition.
    fn with_prelude(
        &self,
        decl: &ImplDecl<'_>,
        at: &Prefixes,
        mut out: Vec<Vec<String>>,
    ) -> Vec<Vec<String>> {
        if let (true, Some(path)) = (at.prelude, self.prelude_trait(decl, &at.name)) {
            out.push(path.split("::").map(str::to_owned).collect());
        }
        out
    }

    /// Go on with `frame` now that the call it made returned `result`.
    fn resume(
        &self,
        decl: &ImplDecl<'_>,
        search: &mut Search,
        frame: Frame,
        result: Result<Vec<Vec<String>>, Unknown>,
    ) -> Step {
        match frame {
            Frame::Tail { tail } => {
                search.visited.pop();
                Step::Return(result.and_then(|prefixes| {
                    if prefixes.is_empty() {
                        return Err(None);
                    }
                    Ok(prefixes
                        .into_iter()
                        .map(|mut prefix| {
                            prefix.extend(tail.iter().cloned());
                            prefix
                        })
                        .collect())
                }))
            }
            Frame::Inner {
                mut out,
                inner,
                from_self,
                reading,
            } => match result {
                Ok(paths) => {
                    out.extend(paths);
                    Self::next_inner(out, inner, from_self, reading)
                }
                Err(bad) => Step::Return(Err(bad)),
            },
            Frame::Named {
                at,
                named,
                mut out,
                position,
            } => {
                search.named.remove(&position);
                match result {
                    Ok(paths) => {
                        out.extend(paths);
                        self.next_named(search, at, named, out)
                    }
                    Err(bad) => Step::Return(Err(bad)),
                }
            }
            Frame::Target {
                at,
                globs,
                mut out,
                targets,
            } => match result {
                Ok(paths) => {
                    out.extend(paths);
                    self.next_target(decl, search, at, globs, out, targets)
                }
                Err(bad) => Step::Return(Err(bad)),
            },
            Frame::Glob {
                at,
                globs,
                mut out,
                position,
            } => {
                search.globs.remove(&position);
                let u = &self.uses[position];
                let UseKind::Glob { segs } = &u.kind else {
                    return Step::Return(Err(None));
                };
                let bases = match result {
                    Ok(bases) => bases,
                    Err(None) => {
                        // A glob of a path in the crate that names nothing
                        // the index can see (a module or an enum a macro
                        // generates): a candidate that cannot be confirmed,
                        // under that path.
                        let bases = self.unseen_local(u.module, segs, u.leading_colon, u.test);
                        if bases.is_empty() {
                            // A glob of something that is neither a local
                            // path nor a crate the package can name fails the
                            // path; the search goes on, so that the error
                            // names one glob whatever the order.
                            search.failed.insert((
                                format!("{}::*", segs.join("::")),
                                u.file.clone(),
                                u.line,
                                position,
                            ));
                        }
                        bases
                    }
                    Err(inner) => return Step::Return(Err(inner)),
                };
                for mut base in bases {
                    base.push(at.name.clone());
                    out.push(base);
                }
                self.next_glob(decl, search, at, globs, out)
            }
        }
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
            Steps::Load,
        ) {
            Ok(walked) => {
                if let Some(key) = self.walked_key(&walked, written) {
                    return Ok(key);
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
            // A path in code is never ambiguous at its first segment.
            Err(Fail::Missing(_) | Fail::SuperAboveRoot | Fail::Ambiguous(_)) => {}
            Err(Fail::Budget(_)) => unreachable!("{LOAD_WALK}"),
        }
        let paths = self
            .absolute(
                decl,
                decl.module,
                &tr.segments,
                tr.leading_colon,
                Reading::Code,
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
                self.settle_impl_lookups(store, &decl)?;
                continue;
            }
            let trait_key = match (&fact.trait_segments, local.is_empty()) {
                (Some(tr), true) => Some(Arc::new(self.trait_key(&decl, tr)?)),
                _ => None,
            };
            self.settle_impl_lookups(store, &decl)?;
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

    /// A checkpoint of the lookups of the load for the impl block `decl`:
    /// what its paths looked up since the last one is charged to the work of
    /// the load at the impl's line (see [`Index::settle_lookups`]), so what
    /// passes the limit is the lookups of one impl.
    fn settle_impl_lookups(&self, store: &Store<'_>, decl: &ImplDecl<'_>) -> Result<(), LoadError> {
        self.drain_lookups(store.meter(), &decl.file, Some(decl.fact.line))
    }

    /// True when the glob path of `decl` names local enums only.
    fn globs_local_enum(&self, decl: &UseDecl, segs: &[String]) -> bool {
        self.walk(
            decl.module,
            segs,
            decl.leading_colon,
            Namespace::Type,
            Heads::Use {
                test: decl.test,
                own: None,
            },
            Steps::Load,
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
        let mut at = decl.module;
        if path.leading_colon {
            // In edition 2015 a path with a leading `::` starts at the crate
            // root; in later ones at a crate.
            if self.edition(decl.module) != 2015 {
                return Ok(path.segments.join("::"));
            }
            at = self.modules[at].root;
        }
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
            span: fact.span.clone(),
            target: decl.target,
            test: combine(fact.test, decl.reach_test),
            members,
        });
        Ok(())
    }

    /// The paths in the crate of a glob path written in `module` that starts
    /// in the crate (`crate`, `self`, `super`, a local module the module
    /// binds, or in edition 2015 a local module the crate root binds) and
    /// goes on through names the index cannot see: for each way the path can
    /// go, the path of the last local module it reaches, followed by the rest
    /// as written. A name bound to several modules gives a path through each.
    /// None for a path that starts at a crate of the extern prelude or with a
    /// name that is not a local module.
    fn unseen_local(
        &self,
        module: usize,
        segs: &[String],
        leading_colon: bool,
        test: bool,
    ) -> Vec<Vec<String>> {
        let Some(head) = segs.first() else {
            return Vec::new();
        };
        let heads = Heads::Use { test, own: None };
        let start = if self.crate_relative(module, head, leading_colon, heads) {
            self.modules[module].root
        } else if leading_colon {
            return Vec::new();
        } else {
            module
        };
        let bound_modules = |at: usize, name: &str| -> BTreeSet<usize> {
            self.lookup(at, Namespace::Type, name, Steps::Load)
                .map_or_default(|(bound, _)| {
                    bound.keys().flat_map(|id| self.modules_for(*id)).collect()
                })
        };
        let mut work: Vec<(usize, &[String])> = Vec::new();
        match head.as_str() {
            "crate" => work.push((self.modules[module].root, &segs[1..])),
            "self" => work.push((module, &segs[1..])),
            "super" => work.push((module, segs)),
            first => {
                for at in bound_modules(start, first) {
                    work.push((at, &segs[1..]));
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
