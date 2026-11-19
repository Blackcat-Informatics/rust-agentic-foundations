// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded enum variant evidence and top-level match-arm classification.

use std::collections::BTreeSet;

use syn::spanned::Spanned;

use super::{
    Error, GlobRead, ItemId, LexicalPath, Lookup, MatchLimits, Namespace, PathDefinition, Scopes,
    Site, SymbolKind, VariantForms, charge, failure, path_text, segments,
};
use crate::detectors::{ENUM_CATCH_ALL, RawHit};

#[derive(Clone, Copy)]
enum Form {
    Unit,
    Tuple,
    Record,
}

impl Form {
    const fn namespace(self) -> Namespace {
        match self {
            Self::Unit | Self::Tuple => Namespace::Value,
            Self::Record => Namespace::Type,
        }
    }

    const fn admits(self, forms: VariantForms) -> bool {
        match self {
            Self::Unit => forms.unit && !forms.tuple && !forms.named,
            Self::Tuple => forms.tuple && !forms.unit && !forms.named,
            Self::Record => forms.named && !forms.unit && !forms.tuple,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Variant {
    enumeration: ItemId,
    member: String,
}

enum Arm {
    Variants(BTreeSet<Variant>),
    CatchAll,
    Other,
}

#[derive(Clone, Copy)]
struct VariantPath<'a> {
    segments: &'a [String],
    absolute: bool,
    alias_scope: Option<&'a dyn Fn(usize) -> bool>,
}

pub(super) fn has_candidate(node: &syn::ExprMatch) -> bool {
    node.arms.iter().any(|arm| match strip_parens(&arm.pat) {
        syn::Pat::Wild(_) => true,
        syn::Pat::Ident(ident) => ident.subpat.is_none(),
        _ => false,
    })
}

pub(super) fn bare_binding(
    name: &syn::Ident,
    lookup: &Lookup<'_, '_>,
    scopes: &Scopes,
    limits: &mut MatchLimits,
) -> Result<bool, Error> {
    let path = syn::Path::from(name.clone());
    Ok(matches!(
        path_arm(&path, Form::Unit, true, lookup, scopes, limits)?,
        Arm::CatchAll
    ))
}

pub(super) fn catch_alls(
    node: &syn::ExprMatch,
    lookup: &Lookup<'_, '_>,
    scopes: &Scopes,
    limits: &mut MatchLimits,
) -> Result<BTreeSet<usize>, Error> {
    let mut enumeration = None;
    let mut hits = BTreeSet::new();
    for (index, arm) in node.arms.iter().enumerate() {
        charge(ENUM_CATCH_ALL, limits, 1)?;
        if !crate::syntax::inert_attributes(&arm.attrs) {
            continue;
        }
        let (pattern, guarded) = match strip_parens(&arm.pat) {
            syn::Pat::Guard(guard) => (guard.pat.as_ref(), true),
            pattern => (pattern, false),
        };
        match classify(pattern, lookup, scopes, limits)? {
            Arm::Variants(variants) => {
                let definitions: BTreeSet<_> = variants
                    .into_iter()
                    .map(|variant| variant.enumeration)
                    .collect();
                if enumeration
                    .replace(definitions.clone())
                    .is_some_and(|other| other != definitions)
                {
                    return Ok(BTreeSet::new());
                }
            }
            Arm::CatchAll if !guarded => {
                if hits.len() >= limits.states {
                    return Err(failure(
                        ENUM_CATCH_ALL,
                        "catch-all arm state limit exhausted",
                    ));
                }
                hits.insert(index);
            }
            Arm::CatchAll | Arm::Other => {}
        }
    }
    Ok(if enumeration.is_some() {
        hits
    } else {
        BTreeSet::new()
    })
}

pub(super) fn hit(arm: &syn::Arm) -> RawHit {
    let span = arm.pat.span();
    RawHit {
        rule: ENUM_CATCH_ALL,
        byte: span.byte_range().start,
        line: span.start().line,
        message: "catch-all arm over an indexed enum; name each variant explicitly".into(),
        call_evidence: None,
    }
}

fn classify(
    pattern: &syn::Pat,
    lookup: &Lookup<'_, '_>,
    scopes: &Scopes,
    limits: &mut MatchLimits,
) -> Result<Arm, Error> {
    let pattern = strip_parens(pattern);
    let (path, form) = match pattern {
        syn::Pat::Wild(_) => return Ok(Arm::CatchAll),
        syn::Pat::Or(or) => return alternatives(&or.cases, lookup, scopes, limits),
        syn::Pat::Ident(ident) if ident.subpat.is_none() => {
            if ident.by_ref.is_some() || ident.mutability.is_some() {
                return Ok(Arm::CatchAll);
            }
            let path = syn::Path::from(ident.ident.clone());
            return path_arm(&path, Form::Unit, true, lookup, scopes, limits);
        }
        syn::Pat::Path(path) if path.qself.is_none() => (&path.path, Form::Unit),
        syn::Pat::TupleStruct(path) if path.qself.is_none() => (&path.path, Form::Tuple),
        syn::Pat::Struct(path) if path.qself.is_none() => (&path.path, Form::Record),
        _ => return Ok(Arm::Other),
    };
    path_arm(path, form, false, lookup, scopes, limits)
}

fn alternatives(
    cases: &syn::punctuated::Punctuated<syn::Pat, syn::Token![|]>,
    lookup: &Lookup<'_, '_>,
    scopes: &Scopes,
    limits: &mut MatchLimits,
) -> Result<Arm, Error> {
    let mut family = None;
    let mut members = BTreeSet::new();
    for case in cases {
        charge(ENUM_CATCH_ALL, limits, 1)?;
        let Arm::Variants(variants) = classify(case, lookup, scopes, limits)? else {
            return Ok(Arm::Other);
        };
        let definitions: BTreeSet<_> = variants.iter().map(|variant| variant.enumeration).collect();
        if family
            .replace(definitions.clone())
            .is_some_and(|other| other != definitions)
        {
            return Ok(Arm::Other);
        }
        for variant in variants {
            if members.len() >= limits.states && !members.contains(&variant) {
                return Err(failure(
                    ENUM_CATCH_ALL,
                    "enum arm alternative state limit exhausted",
                ));
            }
            members.insert(variant);
        }
    }
    Ok(if members.is_empty() {
        Arm::Other
    } else {
        Arm::Variants(members)
    })
}

fn strip_parens(mut pattern: &syn::Pat) -> &syn::Pat {
    while let syn::Pat::Paren(paren) = pattern {
        pattern = &paren.pat;
    }
    pattern
}

fn expand_variant_path(
    (written, absolute): (&[String], bool),
    form: Form,
    lookup: &Lookup<'_, '_>,
    scopes: &Scopes,
    limits: &mut MatchLimits,
) -> Result<Option<super::Expanded>, Error> {
    let mut alias = |target: super::AliasTarget<'_>, namespace, limits: &mut MatchLimits| {
        let expanded = &target.expanded;
        if matches!(namespace, Namespace::Type | Namespace::Value)
            && !(expanded.has_uncertain_alias() || expanded.shadowed && expanded.block)
            && !variants(
                lookup,
                scopes,
                VariantPath {
                    segments: &expanded.segments,
                    absolute: expanded.is_absolute(),
                    alias_scope: Some(&|frame| target.sees_glob(frame)),
                },
                form,
                limits,
            )?
            .is_empty()
        {
            return Ok(true);
        }
        lookup.visible_alias(scopes, &target, namespace, limits)
    };
    scopes.expand_checked(
        (written, absolute, form.namespace()),
        false,
        None,
        limits,
        &mut alias,
    )
}

fn path_arm(
    path: &syn::Path,
    form: Form,
    bare: bool,
    lookup: &Lookup<'_, '_>,
    scopes: &Scopes,
    limits: &mut MatchLimits,
) -> Result<Arm, Error> {
    let site = in_namespace(lookup.site, form.namespace());
    let lookup = Lookup {
        site: &site,
        ..*lookup
    };
    let written = segments(path);
    let absolute = path.leading_colon.is_some();
    let expanded = expand_variant_path((&written, absolute), form, &lookup, scopes, limits)?;
    if expanded
        .as_ref()
        .is_some_and(super::Expanded::has_uncertain_alias)
    {
        return Ok(Arm::Other);
    }
    if let Some(local) = expanded.as_ref().filter(|path| path.shadowed && path.block) {
        let binding = if bare
            && local.alias_frame().is_none()
            && let Some(frame) = local.first_frame()
            && let Some(name) = written.first()
        {
            scopes.runtime_binding(frame, name, limits)?
        } else {
            false
        };
        return Ok(if binding { Arm::CatchAll } else { Arm::Other });
    }
    let glob = lookup.glob_occurrence(scopes, &written, absolute, expanded.as_ref(), limits)?;
    if glob.opaque.is_some() {
        return Ok(Arm::Other);
    }
    if let Some(candidate) = glob.variant {
        let sites = roster_matches(&candidate.enums, &candidate.member, form, limits)?;
        return Ok(if sites.is_empty() {
            Arm::Other
        } else {
            Arm::Variants(sites)
        });
    }
    if written.len() == 1 && glob.definition.is_some() {
        return Ok(Arm::Other);
    }
    let declaration = |frame| {
        expanded
            .as_ref()
            .is_some_and(|path| path.sees_target_glob(frame))
    };
    let declared_path = expanded
        .as_ref()
        .filter(|path| path.alias_frame().is_some());
    let path = declared_path.map_or(
        VariantPath {
            segments: &written,
            absolute,
            alias_scope: None,
        },
        |path| VariantPath {
            segments: &path.segments,
            absolute: path.is_absolute(),
            alias_scope: Some(&declaration),
        },
    );
    let actual = variants(&lookup, scopes, path, form, limits)?;
    if !actual.is_empty() {
        return Ok(Arm::Variants(actual));
    }
    if bare
        && expanded
            .as_ref()
            .is_some_and(|expanded| expanded.alias_frame().is_none())
    {
        let raw = path_text(&written, absolute);
        if matches!(
            lookup.resolve(&raw, expanded.as_ref(), limits)?,
            PathDefinition::Missing | PathDefinition::Unverified(_)
        ) {
            return Ok(Arm::CatchAll);
        }
    }
    Ok(Arm::Other)
}

fn variants(
    lookup: &Lookup<'_, '_>,
    scopes: &Scopes,
    path: VariantPath<'_>,
    form: Form,
    limits: &mut MatchLimits,
) -> Result<BTreeSet<Variant>, Error> {
    let Some((member, prefix)) = path.segments.split_last() else {
        return Ok(BTreeSet::new());
    };
    if prefix.is_empty() {
        return Ok(BTreeSet::new());
    }
    let site = in_namespace(lookup.site, Namespace::Type);
    let lookup = Lookup {
        site: &site,
        ..*lookup
    };
    let actual = enum_prefix(
        &lookup,
        scopes,
        prefix,
        path.absolute,
        path.alias_scope,
        limits,
    )?;
    let Some((workspace, _)) = lookup.visibility else {
        return Err(failure(
            ENUM_CATCH_ALL,
            "indexed enum lookup requires the source workspace",
        ));
    };
    let mut result = BTreeSet::new();
    for id in actual {
        let item = workspace
            .item(id)
            .ok_or_else(|| failure(ENUM_CATCH_ALL, "enum prefix has no indexed item"))?;
        if item.kind != SymbolKind::Enum {
            return Ok(BTreeSet::new());
        }
        let members = lookup.enum_members(scopes, workspace, item, limits)?;
        if !admit_member(&mut result, (id, &members), member, form, limits)? {
            return Ok(BTreeSet::new());
        }
    }
    Ok(result)
}

fn enum_prefix(
    lookup: &Lookup<'_, '_>,
    scopes: &Scopes,
    prefix: &[String],
    absolute: bool,
    alias_scope: Option<&dyn Fn(usize) -> bool>,
    limits: &mut MatchLimits,
) -> Result<Vec<ItemId>, Error> {
    let raw = path_text(prefix, absolute);
    let definition = if let Some(visible) = alias_scope {
        // The alias target has already been expanded in its declaration scope.
        // A closer alias at the pattern occurrence cannot change its target.
        let glob = if absolute {
            super::GlobLookup::default()
        } else {
            lookup.through_block_glob(
                scopes,
                prefix,
                visible,
                GlobRead {
                    namespace: Namespace::Type,
                    early: true,
                    exclusion: None,
                    keep_head: false,
                },
                limits,
            )?
        };
        if glob.opaque.is_some() {
            return Ok(Vec::new());
        }
        if let Some(actual) = glob.definition {
            actual
        } else {
            return lookup.indexed(&raw, true).map(Option::unwrap_or_default);
        }
    } else {
        let LexicalPath::Accessible(expanded) = lookup.lexical(scopes, prefix, absolute, limits)?
        else {
            return Ok(Vec::new());
        };
        if expanded
            .as_ref()
            .is_some_and(|path| path.has_uncertain_alias() || path.shadowed && path.block)
        {
            return Ok(Vec::new());
        }
        let glob = lookup.glob_occurrence(scopes, prefix, absolute, expanded.as_ref(), limits)?;
        if glob.opaque.is_some() {
            return Ok(Vec::new());
        }
        match glob.definition {
            Some(actual) => actual,
            None => match lookup.resolve(&raw, expanded.as_ref(), limits)? {
                PathDefinition::Written(actual) | PathDefinition::Expanded(actual) => {
                    return Ok(actual);
                }
                PathDefinition::Missing | PathDefinition::Unverified(_) => return Ok(Vec::new()),
            },
        }
    };
    Ok(match definition {
        PathDefinition::Written(actual) | PathDefinition::Expanded(actual) => actual,
        PathDefinition::Missing | PathDefinition::Unverified(_) => Vec::new(),
    })
}

fn roster_matches(
    candidates: &purrdf_lex::walk::WorkList<super::EnumAlternative, 2>,
    member: &str,
    form: Form,
    limits: &mut MatchLimits,
) -> Result<BTreeSet<Variant>, Error> {
    let mut result = BTreeSet::new();
    for candidate in candidates.iter() {
        if !admit_member(
            &mut result,
            (candidate.id, &candidate.members),
            member,
            form,
            limits,
        )? {
            return Ok(BTreeSet::new());
        }
    }
    Ok(result)
}

fn admit_member(
    result: &mut BTreeSet<Variant>,
    (id, members): (ItemId, &super::EnumMembers),
    member: &str,
    form: Form,
    limits: &mut MatchLimits,
) -> Result<bool, Error> {
    charge(
        ENUM_CATCH_ALL,
        limits,
        u64::from(members.len().bit_width()) + 1,
    )?;
    if members
        .get(member)
        .is_none_or(|member| member.conditional || !form.admits(member.forms))
    {
        return Ok(false);
    }
    if result.len() >= limits.states {
        return Err(failure(
            ENUM_CATCH_ALL,
            "enum variant evidence state limit exhausted",
        ));
    }
    result.insert(Variant {
        enumeration: id,
        member: member.to_owned(),
    });
    Ok(true)
}

fn in_namespace<'a>(site: &Site<'a>, namespace: Namespace) -> Site<'a> {
    Site { namespace, ..*site }
}
