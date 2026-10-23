// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Syntax bans with indexed workspace identity kept separate from spelling.

mod scopes;
mod visit;

use std::collections::{BTreeMap, BTreeSet};

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::walk::WorkList;
use raf_source::{
    CodeResolveError, CodeResolver, ItemId, Namespace, ResolveError, SourceFile, Workspace,
};
use syn::ext::IdentExt;

use super::{Found, charge, failure};
use crate::syntax::{Alias, path_text};
use crate::{Error, LedgerPattern, PatternMatcher};
use scopes::Scopes;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Call,
    Path,
    Macro,
    Type,
    Attribute,
    Import,
}

struct Compiled<'p> {
    pattern: &'p LedgerPattern,
    role: Role,
    spelling: Vec<String>,
    sites: BTreeMap<Namespace, BTreeSet<ItemId>>,
}

impl Compiled<'_> {
    fn selects(&self, role: Role) -> bool {
        self.role == role || (self.role == Role::Path && role != Role::Call)
    }
}

pub(super) fn collect<'p, 'w>(
    patterns: &[&'p LedgerPattern],
    workspace: &'w Workspace,
    code: &CodeResolver<'_>,
    limits: &mut MatchLimits,
) -> Result<Vec<Found<'p, 'w>>, Error> {
    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    let compiled = compile(patterns, workspace, limits)?;
    let mut found = Vec::new();
    for file in workspace.files() {
        let hits = raf_source::parse_source(&file.path, &file.text, |syntax| {
            visit::collect(&compiled, workspace, code, file, syntax, limits)
        })??;
        for hit in hits {
            super::retain(
                &mut found,
                hit.pattern,
                hit.file,
                hit.byte,
                hit.line,
                limits,
            )?;
        }
    }
    Ok(found)
}

fn compile<'p>(
    patterns: &[&'p LedgerPattern],
    workspace: &Workspace,
    limits: &mut MatchLimits,
) -> Result<Vec<Compiled<'p>>, Error> {
    let mut compiled = Vec::new();
    for pattern in patterns {
        let (role, text) = match &pattern.matcher {
            PatternMatcher::Call(text) => (Role::Call, text),
            PatternMatcher::Path(text) => (Role::Path, text),
            PatternMatcher::Macro(text) => (Role::Macro, text),
            PatternMatcher::Type(text) => (Role::Type, text),
            _ => {
                return Err(failure(
                    &pattern.id,
                    "non-path matcher entered path execution",
                ));
            }
        };
        let mut sites = BTreeMap::new();
        for namespace in Namespace::ALL {
            charge(
                &pattern.id,
                limits,
                text.len().try_into().unwrap_or(u64::MAX),
            )?;
            match workspace.resolve_in(text.trim_start_matches("::"), namespace) {
                Ok(resolution) => {
                    sites.insert(namespace, resolution.sites.into_iter().collect());
                }
                Err(ResolveError::NotFound(_)) => {}
                Err(error) => {
                    return Err(failure(
                        &pattern.id,
                        &format!("declared path {text:?}: {error}"),
                    ));
                }
            }
        }
        compiled.push(Compiled {
            pattern,
            role,
            spelling: text
                .trim_start_matches("::")
                .split("::")
                .map(|part| part.strip_prefix("r#").unwrap_or(part).to_owned())
                .collect(),
            sites,
        });
    }
    Ok(compiled)
}

fn segments(path: &syn::Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|part| part.ident.unraw().to_string())
        .collect()
}

struct Site<'a> {
    file: &'a SourceFile,
    byte: usize,
    line: usize,
    namespace: Namespace,
    import: bool,
}

fn query_error(pattern: &LedgerPattern, site: &Site<'_>, source: CodeResolveError) -> Error {
    Error::CodePath {
        pattern: pattern.id.clone(),
        file: site.file.path.to_string(),
        line: site.line,
        source: Box::new(source),
    }
}

#[derive(Default)]
struct OwnerCache<'w> {
    entries: WorkList<(usize, Vec<&'w raf_source::Item>), 8>,
    retained: usize,
}

impl<'w> OwnerCache<'w> {
    fn get(
        &mut self,
        workspace: &'w Workspace,
        file: &SourceFile,
        byte: usize,
        limits: &mut MatchLimits,
    ) -> Result<&[&'w raf_source::Item], Error> {
        charge(
            "paths",
            limits,
            self.entries
                .len()
                .saturating_mul(2)
                .try_into()
                .unwrap_or(u64::MAX),
        )?;
        if let Some(index) = self.entries.iter().position(|(known, _)| *known == byte) {
            return Ok(&self
                .entries
                .iter()
                .nth(index)
                .expect("cached owner position")
                .1);
        }
        // item_at makes three candidate passes; root/source classification
        // makes one indexed query per candidate, including its file key.
        let candidates = workspace.item_count_in(&file.path);
        let work = candidates.saturating_mul(file.path.as_str().len().saturating_add(4));
        charge("paths", limits, work.try_into().unwrap_or(u64::MAX))?;
        if self
            .entries
            .len()
            .saturating_add(self.retained)
            .saturating_add(candidates)
            .saturating_add(1)
            > limits.states
        {
            return Err(failure(
                "paths",
                "matching owner-cache state limit exhausted",
            ));
        }
        let mut owners = workspace.item_at(&file.path, byte);
        let mut real = Vec::new();
        for item in &owners {
            if !workspace.same_source(item).is_empty() {
                real.push(*item);
            }
        }
        if !real.is_empty() {
            owners = real;
        }
        self.retained = self.retained.saturating_add(owners.len());
        super::push(
            "paths",
            &mut self.entries,
            (byte, owners),
            self.retained,
            limits,
        )?;
        Ok(&self
            .entries
            .iter()
            .last()
            .expect("inserted owner position")
            .1)
    }
}

fn indexed_match(
    code: &CodeResolver<'_>,
    owner: ItemId,
    written: &str,
    expected: &Compiled<'_>,
    site: &Site<'_>,
    import: bool,
) -> Result<Option<bool>, Error> {
    let resolved = if import {
        code.resolve_import_from_item_at(owner, site.byte, written, site.namespace)
    } else {
        code.resolve_from_item_at(owner, site.byte, written, site.namespace)
    };
    match resolved {
        Ok(actual) => {
            Ok(Some(expected.sites.get(&site.namespace).is_some_and(
                |wanted| actual.sites.iter().any(|id| wanted.contains(id)),
            )))
        }
        Err(CodeResolveError::Resolve(ResolveError::NotFound(_))) => Ok(None),
        Err(error) => Err(query_error(expected.pattern, site, error)),
    }
}

/// Indexed namespaces select imports; an entirely unindexed target stays
/// an unverified syntax observation, never a provider identity.
pub(super) fn import_eligible(
    code: &CodeResolver<'_>,
    owner: ItemId,
    byte: usize,
    alias: &Alias,
    namespace: Namespace,
) -> Result<bool, CodeResolveError> {
    let text = path_text(&alias.path, alias.absolute);
    match code.resolve_import_from_item_at(owner, byte, &text, namespace) {
        Ok(_) => return Ok(true),
        Err(CodeResolveError::Resolve(ResolveError::NotFound(_))) => {}
        Err(error) => return Err(error),
    }
    for other in Namespace::ALL {
        if other == namespace {
            continue;
        }
        match code.resolve_import_from_item_at(owner, byte, &text, other) {
            Ok(_) => return Ok(false),
            Err(CodeResolveError::Resolve(ResolveError::NotFound(_))) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(true)
}

fn constant<'w>(
    workspace: &'w Workspace,
    code: &CodeResolver<'_>,
    file: &SourceFile,
    pattern: &LedgerPattern,
    name: &syn::Ident,
    cache: &mut OwnerCache<'w>,
    limits: &mut MatchLimits,
) -> Result<bool, Error> {
    let span = name.span();
    let site = Site {
        file,
        byte: span.byte_range().start,
        line: span.start().line,
        namespace: Namespace::Value,
        import: false,
    };
    let written = name.unraw().to_string();
    let mut classified = None;
    let owners = cache.get(workspace, file, site.byte, limits)?;
    for owner in owners {
        let native = match code.resolve_from_item_at(
            owner.id,
            site.byte,
            &written,
            Namespace::Value,
        ) {
            Ok(resolution) => {
                let mut kinds = BTreeSet::new();
                for id in resolution.sites {
                    let item = workspace.item(id).ok_or_else(|| {
                        failure(&pattern.id, "resolved pattern constant has no source item")
                    })?;
                    kinds.insert(
                        matches!(
                            item.kind,
                            raf_source::SymbolKind::Const | raf_source::SymbolKind::AssocConst
                        ) || (item.kind == raf_source::SymbolKind::Struct
                            && item.shape == Some(raf_source::FieldShape::Unit)),
                    );
                }
                if kinds.len() != 1 {
                    return Err(failure(
                        &pattern.id,
                        "pattern identifier has mixed constant/binding definitions; provide an unambiguous source context",
                    ));
                }
                *kinds.first().expect("one resolved definition kind")
            }
            Err(CodeResolveError::Resolve(ResolveError::NotFound(_))) => false,
            Err(error) => return Err(query_error(pattern, &site, error)),
        };
        if classified.is_some_and(|previous| previous != native) {
            return Err(failure(
                &pattern.id,
                "pattern identifier changes binding meaning across source reaches; provide an unambiguous source context",
            ));
        }
        classified = Some(native);
    }
    Ok(classified.unwrap_or(false))
}

fn matches(
    owners: &[&raf_source::Item],
    code: &CodeResolver<'_>,
    scopes: &Scopes,
    expected: &Compiled<'_>,
    path: &syn::Path,
    site: &Site<'_>,
    limits: &mut MatchLimits,
) -> Result<bool, Error> {
    let written = segments(path);
    let named_macro = expected.role == Role::Macro && expected.spelling.len() == 1;
    if named_macro && written.last() == expected.spelling.last() {
        return Ok(true);
    }
    let raw = path_text(&written, path.leading_colon.is_some());
    if owners.is_empty() {
        return Err(failure(
            &expected.pattern.id,
            "path occurrence has no source owner",
        ));
    }
    for owner in owners {
        let mut eligible = |alias: &Alias, namespace| {
            if site.import && alias.source_byte == Some(site.byte) {
                return Ok(false);
            }
            import_eligible(code, owner.id, site.byte, alias, namespace)
                .map_err(|error| query_error(expected.pattern, site, error))
        };
        if !site.import
            && path.leading_colon.is_none()
            && scopes.local(&written, site.namespace, limits, &mut eligible)?
        {
            continue;
        }
        let expanded = scopes.expand(
            &written,
            path.leading_colon.is_some(),
            site.namespace,
            limits,
            &mut eligible,
        )?;
        if named_macro {
            if expanded
                .as_ref()
                .is_some_and(|expanded| expanded.segments.last() == expected.spelling.last())
            {
                return Ok(true);
            }
            continue;
        }
        charge(
            &expected.pattern.id,
            limits,
            raw.len().try_into().unwrap_or(u64::MAX),
        )?;
        // Block imports are absent from the module index: resolve their actual
        // expanded path first. Module imports/reexports use the shared resolver.
        if let Some(expanded) = &expanded
            && expanded.block
        {
            if expanded.shadowed {
                continue;
            }
            if let Some(result) =
                indexed_match(code, owner.id, &expanded.text(), expected, site, true)?
            {
                if result {
                    return Ok(true);
                }
                continue;
            }
            if expected.sites.is_empty() && expanded.segments == expected.spelling {
                return Ok(true);
            }
            // The syntactic block binding shadows the raw module name even
            // when its external target has no indexed source definition.
            continue;
        }
        if let Some(result) = indexed_match(code, owner.id, &raw, expected, site, site.import)? {
            if result {
                return Ok(true);
            }
            // An indexed different definition is never replaced by spelling.
            if expected.spelling.len() == 1
                && expected.sites.is_empty()
                && written == expected.spelling
            {
                return Ok(true);
            }
            continue;
        }
        if let Some(expanded) = &expanded
            && !expanded.shadowed
        {
            if let Some(result) =
                indexed_match(code, owner.id, &expanded.text(), expected, site, true)?
            {
                if result {
                    return Ok(true);
                }
                continue;
            }
            // This is an explicitly unverified syntax observation. Provider
            // policy must never consume it as source or delegation evidence.
            if expected.sites.is_empty() && expanded.segments == expected.spelling {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::OwnerCache;
    use purrdf_jsonschema::ecma::MatchLimits;
    use raf_source::{MemoryTree, Workspace};

    #[test]
    fn ownership_scans_spend_the_grant_once_per_byte_and_bound_retention() {
        let tree = MemoryTree::from([
            ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\n"),
            ("src/lib.rs", "pub fn one(){} pub fn two(){}"),
        ]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        let file = workspace.files().next().unwrap();
        let count = workspace.item_count_in(&file.path);
        assert_eq!(count, workspace.items_in(&file.path).count());
        let mut cache = OwnerCache::default();
        let mut limits = MatchLimits {
            steps: 10_000,
            states: 64,
        };
        let first = cache.get(&workspace, file, 0, &mut limits).unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].path, "demo::one");
        // A second lookup can afford only the cache walk, not another scan.
        limits.steps = 2;
        assert_eq!(
            cache.get(&workspace, file, 0, &mut limits).unwrap()[0].path,
            "demo::one"
        );
        let second = file.text.find("two").unwrap();
        limits.steps = 2;
        assert!(
            cache
                .get(&workspace, file, second, &mut limits)
                .unwrap_err()
                .to_string()
                .contains("step limit")
        );
        limits.steps = 10_000;
        assert_eq!(
            cache.get(&workspace, file, second, &mut limits).unwrap()[0].path,
            "demo::two"
        );
        let mut refused = OwnerCache::default();
        limits.states = 0;
        assert!(
            refused
                .get(&workspace, file, 0, &mut limits)
                .unwrap_err()
                .to_string()
                .contains("state limit")
        );
    }
}
