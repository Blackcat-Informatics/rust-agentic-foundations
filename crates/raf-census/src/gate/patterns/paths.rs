// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Syntax bans with indexed workspace identity kept separate from spelling.

mod cache;
pub(in crate::gate) mod scopes;
pub(in crate::gate) mod visit;

use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::walk::WorkList;
use raf_source::{
    CodeResolveError, CodeResolver, ImplBlock, ItemId, Namespace, PackageId, Resolution,
    ResolutionAttempt, ResolveError, SourceFile, SymbolKind, TargetId, TargetKind, Workspace,
};
use syn::Fields;
use syn::ext::IdentExt;

use super::{Found, charge, failure, push};
use crate::syntax::{Alias, path_text};
use crate::{Census, Error, LedgerPattern, PatternMatcher, VisibilityError};
use cache::OwnerCache;
use scopes::{Expanded, Glob, Scopes};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Call,
    Path,
    Pattern,
    Macro,
    Type,
    Attribute,
    Import,
    Glob,
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
    census: &Census,
    code: &CodeResolver<'_>,
    references: &[String],
    limits: &mut MatchLimits,
) -> Result<Vec<Found<'p, 'w>>, Error> {
    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    let compiled = compile(patterns, workspace, references, limits)?;
    let mut found = Vec::new();
    for file in workspace.files() {
        if census.source_context(&file.path).is_none() {
            continue;
        }
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
                hit.owners,
                limits,
            )?;
        }
    }
    Ok(found)
}

fn compile<'p>(
    patterns: &[&'p LedgerPattern],
    workspace: &Workspace,
    references: &[String],
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
        let mut selected_reference = false;
        for namespace in Namespace::ALL {
            charge(
                &pattern.id,
                limits,
                text.len().try_into().unwrap_or(u64::MAX),
            )?;
            match workspace.resolve_in_observed(text.trim_start_matches("::"), namespace) {
                Ok(ResolutionAttempt::Resolved(resolution)) => {
                    sites.insert(namespace, resolution.sites.into_iter().collect());
                }
                Ok(ResolutionAttempt::NotFound {
                    attempted_targets, ..
                }) => {
                    for id in attempted_targets {
                        let target = workspace.target(id).ok_or_else(|| {
                            failure(
                                &pattern.id,
                                "failed lookup observed a target absent from its source index",
                            )
                        })?;
                        selected_reference |= references.contains(&target.package);
                    }
                }
                Ok(_) => {
                    return Err(failure(
                        &pattern.id,
                        "unsupported native resolution observation",
                    ));
                }
                Err(error) => {
                    return Err(failure(
                        &pattern.id,
                        &format!("declared path {text:?}: {error}"),
                    ));
                }
            }
        }
        if sites.is_empty() && selected_reference {
            return Err(failure(
                &pattern.id,
                &format!(
                    "selected reference target {text:?} has no resolved definition; supply the packages needed by its exports and use an indexed symbol path"
                ),
            ));
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
    role: Role,
    file_scope: bool,
}

impl<'a> Site<'a> {
    fn at(
        file: &'a SourceFile,
        span: proc_macro2::Span,
        role: Role,
        namespace: Namespace,
        file_scope: bool,
    ) -> Self {
        Self {
            file,
            byte: span.byte_range().start,
            line: span.start().line,
            namespace,
            role,
            file_scope,
        }
    }

    fn is_import(&self) -> bool {
        matches!(self.role, Role::Import | Role::Glob)
    }
}

fn query_error(id: &str, site: &Site<'_>, source: CodeResolveError) -> Error {
    Error::CodePath {
        pattern: id.to_owned(),
        file: site.file.path.to_string(),
        line: site.line,
        source: Box::new(source),
    }
}

enum LexicalPath {
    Local,
    Accessible(Option<Expanded>),
}

enum PathDefinition<'a> {
    Written(Vec<ItemId>),
    Expanded(Vec<ItemId>),
    Unverified(&'a [String]),
    Missing,
}

#[derive(Default)]
struct GlobLookup {
    definition: Option<PathDefinition<'static>>,
    opaque: Option<String>,
}

struct ScopedGlob<'a> {
    declaration: &'a Glob,
    frame: usize,
    expanded: Expanded,
    text: String,
    direct: Option<GlobProvider>,
    chained: OnceCell<(GlobRoute, GlobProvider)>,
}

type GlobPrefixes<'a> = WorkList<ScopedGlob<'a>, 8>;

impl ScopedGlob<'_> {
    fn prefix(&self) -> &str {
        self.chained
            .get()
            .map_or(self.text.as_str(), |(route, _)| route.path.as_str())
    }

    fn provider(&self) -> Option<&GlobProvider> {
        self.direct
            .as_ref()
            .or_else(|| self.chained.get().map(|(_, provider)| provider))
    }

    fn has_prefix(&self) -> bool {
        // Source can identify the declaration of a module-scope shadow.
        !(self.expanded.segments.is_empty() || self.expanded.shadowed && self.expanded.block)
    }

    fn unresolved_relative(&self) -> bool {
        self.provider().is_none() && self.relative_prefix().is_some()
    }

    fn relative_prefix(&self) -> Option<(&str, &[String])> {
        let (head, tail) = self.expanded.segments.split_first()?;
        let head = head.as_str();
        (self.has_prefix()
            && !self.expanded.is_absolute()
            && !matches!(head, "crate" | "self" | "super"))
        .then_some((head, tail))
    }
}

#[derive(Clone)]
struct GlobRoute {
    path: String,
    sites: Vec<ItemId>,
}

#[derive(Clone)]
enum GlobProvider {
    Module,
    Enum(Rc<WorkList<Rc<EnumMembers>, 2>>),
    Other,
}

type EnumMembers = BTreeMap<String, bool>;

impl GlobProvider {
    fn retained_states(&self) -> usize {
        match self {
            Self::Enum(enums) => enums.len(),
            Self::Module | Self::Other => 0,
        }
    }

    fn variant(
        &self,
        head: &str,
        namespace: Namespace,
        limits: &mut MatchLimits,
    ) -> Result<bool, Error> {
        let Self::Enum(enums) = self else {
            return Ok(false);
        };
        if !matches!(namespace, Namespace::Type | Namespace::Value) {
            return Ok(false);
        }
        let mut present = None;
        for members in enums.iter() {
            charge(VISIBILITY, limits, u64::from(members.len().bit_width()) + 1)?;
            let found = members
                .get(head)
                .is_some_and(|value| namespace == Namespace::Type || *value);
            if present
                .replace(found)
                .is_some_and(|previous| previous != found)
            {
                return Err(failure(
                    VISIBILITY,
                    &format!(
                        "enum glob head {head:?} differs across indexed declarations; qualify the path"
                    ),
                ));
            }
        }
        Ok(present.unwrap_or(false))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ReadingContext {
    Item(ItemId, String),
    Impl(raf_source::ImplId),
    File(TargetId, String, bool),
}

#[derive(Default)]
struct GlobCache {
    contexts: BTreeMap<ReadingContext, SettledFrame>,
    retained: usize,
}

struct SettledFrame {
    prefixes: WorkList<SettledGlob, 8>,
    heads: BTreeMap<Namespace, BTreeMap<String, GlobHead>>,
}

struct SettledGlob {
    source_byte: Option<usize>,
    prefix: String,
    provider: Option<GlobProvider>,
}

#[derive(Clone, Default)]
struct GlobHead {
    route: Option<GlobRoute>,
    opaque: Option<String>,
    variant: bool,
}

struct Lookup<'a, 'w> {
    code: &'a CodeResolver<'w>,
    owner: PathOwner<'a>,
    site: &'a Site<'a>,
    label: &'a str,
    visibility: Option<(&'a Workspace, PackageId)>,
}

#[derive(Clone, Copy)]
enum PathOwner<'a> {
    Item(ItemId),
    Impl(raf_source::ImplId),
    File(&'a raf_source::Reach),
}

impl Lookup<'_, '_> {
    fn error(&self, path: &str, source: CodeResolveError) -> Error {
        if self.visibility.is_some() {
            return Error::Visibility(VisibilityError::Path {
                file: self.site.file.path.to_string(),
                line: self.site.line,
                path: path.to_owned(),
                source: Box::new(source),
            });
        }
        query_error(self.label, self.site, source)
    }

    fn eligible(&self, alias: &Alias, namespace: Namespace) -> Result<bool, Error> {
        if self.site.is_import() && alias.source_byte == Some(self.site.byte) {
            return Ok(false);
        }
        self.eligible_declaration(alias, namespace)
    }

    fn eligible_declaration(&self, alias: &Alias, namespace: Namespace) -> Result<bool, Error> {
        let text = path_text(&alias.path, alias.absolute);
        eligible_namespace(namespace, |space| match self.query(&text, space, true) {
            Ok(_) => Ok(true),
            Err(error)
                if self.visibility.is_some_and(|(workspace, consumer)| {
                    internal_ambiguity(workspace, consumer, &error)
                }) =>
            {
                Ok(true)
            }
            Err(CodeResolveError::AliasMember { .. }) if self.visibility.is_some() => Ok(false),
            Err(error) => Err(error),
        })
        .map_err(|error| self.error(&text, error))
    }

    fn lexical(
        &self,
        scopes: &Scopes,
        written: &[String],
        absolute: bool,
        limits: &mut MatchLimits,
    ) -> Result<LexicalPath, Error> {
        let mut eligible = |alias: &Alias, namespace| self.eligible(alias, namespace);
        if self.visibility.is_none()
            && !self.site.is_import()
            && !absolute
            && scopes.local(written, self.site.namespace, limits, &mut eligible)?
        {
            return Ok(LexicalPath::Local);
        }
        let expanded = if self.site.is_import() {
            scopes.expand_import(
                written,
                absolute,
                self.site.namespace,
                None,
                limits,
                &mut eligible,
            )
        } else {
            scopes.expand(
                written,
                absolute,
                self.site.namespace,
                limits,
                &mut eligible,
            )
        };
        expanded.map(LexicalPath::Accessible)
    }

    fn indexed(&self, written: &str, import: bool) -> Result<Option<Vec<ItemId>>, Error> {
        self.indexed_in(written, self.site.namespace, import)
    }

    fn indexed_in(
        &self,
        written: &str,
        namespace: Namespace,
        import: bool,
    ) -> Result<Option<Vec<ItemId>>, Error> {
        let result = self.query(written, namespace, import);
        if let Some((workspace, package)) = self.visibility {
            if namespace == Namespace::Value
                && let Err(CodeResolveError::Resolve(ResolveError::Ambiguous(items))) = &result
                && let Some(id) = self.inherent_at(workspace, written, items, import)?
            {
                return Ok(Some(vec![id]));
            }
            return visibility_resolution(workspace, package, result)
                .map(|result| result.map(|resolution| resolution.sites))
                .map_err(|error| self.error(written, error));
        }
        match result {
            Ok(actual) => Ok(Some(actual.sites)),
            Err(CodeResolveError::Resolve(ResolveError::NotFound(_))) => Ok(None),
            Err(error) => Err(query_error(self.label, self.site, error)),
        }
    }

    fn inherent_at(
        &self,
        workspace: &Workspace,
        written: &str,
        items: &[ItemId],
        import: bool,
    ) -> Result<Option<ItemId>, Error> {
        let candidate =
            super::super::rules::inherent_candidate(workspace, written, items, |nominal| {
                self.query(nominal, Namespace::Type, import)
            })
            .map_err(|source| self.error(written, source))?;
        let Some(candidate) = candidate else {
            return Ok(None);
        };
        match self.code.require_unconditional_inherent_method(candidate) {
            Ok(()) => Ok(Some(candidate)),
            Err(CodeResolveError::MethodApplicability(_)) => Ok(None),
            Err(source) => Err(self.error(written, source)),
        }
    }

    fn through_block_glob(
        &self,
        scopes: &Scopes,
        written: &[String],
        visible: impl Fn(usize) -> bool,
        limits: &mut MatchLimits,
    ) -> Result<GlobLookup, Error> {
        let Some(head) = written.first() else {
            return Ok(GlobLookup::default());
        };
        if matches!(head.as_str(), "crate" | "self" | "super") {
            return Ok(GlobLookup::default());
        }
        let namespace = if written.len() > 1 {
            Namespace::Type
        } else {
            self.site.namespace
        };
        let mut glob_lookup = GlobLookup::default();
        let exclusion = self.site.is_import().then_some(self.site.byte);
        let mut context = None;
        for (frame, declarations, cache) in
            scopes.block_globs().filter(|(frame, _, _)| visible(*frame))
        {
            if exclusion.is_some_and(|byte| {
                declarations
                    .iter()
                    .all(|glob| glob.source_byte == Some(byte))
            }) {
                continue;
            }
            if context.is_none() {
                context = Some(self.reading_context()?);
            }
            let context = context
                .as_ref()
                .expect("INVARIANT: a glob lookup establishes its Source context");
            let selected = self.cached_glob_head(
                scopes,
                frame,
                &mut cache.borrow_mut(),
                context,
                (head, namespace, exclusion),
                limits,
            )?;
            if glob_lookup.opaque.is_none() {
                glob_lookup.opaque = selected.opaque;
            }
            if selected.variant {
                glob_lookup.definition = Some(PathDefinition::Missing);
                break;
            }
            if let Some(route) = selected.route {
                glob_lookup.definition = Some(if written.len() == 1 {
                    PathDefinition::Written(route.sites)
                } else {
                    let path = format!("{}::{}", route.path, path_text(&written[1..], false));
                    charge(
                        self.label,
                        limits,
                        path.len().try_into().unwrap_or(u64::MAX),
                    )?;
                    self.indexed(&path, true)?
                        .map_or(PathDefinition::Missing, PathDefinition::Written)
                });
                break;
            }
        }
        Ok(glob_lookup)
    }

    fn glob_occurrence(
        &self,
        scopes: &Scopes,
        written: &[String],
        absolute: bool,
        expanded: Option<&Expanded>,
        limits: &mut MatchLimits,
    ) -> Result<GlobLookup, Error> {
        if absolute {
            return Ok(GlobLookup::default());
        }
        let mut glob = self.through_block_glob(
            scopes,
            written,
            |frame| expanded.is_none_or(|path| path.sees_glob(frame)),
            limits,
        )?;
        if glob.definition.is_none()
            && let Some(alias) = expanded.filter(|path| {
                !path.is_absolute() && !path.shadowed && path.alias_frame().is_some()
            })
        {
            let target = self.through_block_glob(
                scopes,
                &alias.segments,
                |frame| alias.sees_target_glob(frame),
                limits,
            )?;
            glob.definition = target.definition;
            glob.opaque = glob.opaque.or(target.opaque);
        }
        Ok(glob)
    }

    fn reading_context(&self) -> Result<ReadingContext, Error> {
        Ok(match self.owner {
            PathOwner::Item(item) => ReadingContext::Item(
                item,
                self.code
                    .reading_module(item, self.site.byte)
                    .map_err(|error| self.error("block glob", error))?
                    .to_owned(),
            ),
            PathOwner::Impl(id) => ReadingContext::Impl(id),
            PathOwner::File(reach) => {
                ReadingContext::File(reach.target, reach.module_path.to_string(), reach.test)
            }
        })
    }

    fn cached_glob_head(
        &self,
        scopes: &Scopes,
        frame: usize,
        cache: &mut GlobCache,
        context: &ReadingContext,
        (head, namespace, exclusion): (&str, Namespace, Option<usize>),
        limits: &mut MatchLimits,
    ) -> Result<GlobHead, Error> {
        charge(
            self.label,
            limits,
            u64::from(cache.contexts.len().bit_width()) + 1,
        )?;
        if !cache.contexts.contains_key(context) {
            let globs = self.resolve_block_globs(scopes, frame, limits)?;
            let frame_globs = || globs.iter().take_while(move |glob| glob.frame == frame);
            let retained = frame_globs().fold(1usize, |total, glob| {
                total
                    .saturating_add(1)
                    .saturating_add(glob.provider().map_or(0, GlobProvider::retained_states))
            });
            if retained > scopes.remaining_glob_states(limits) {
                return Err(failure(
                    self.label,
                    "block glob cache state limit exhausted",
                ));
            }
            let mut prefixes = WorkList::new();
            for glob in frame_globs() {
                let prefix = if glob.has_prefix() {
                    glob.prefix().to_owned()
                } else {
                    path_text(&glob.declaration.path, glob.declaration.absolute)
                };
                push(
                    self.label,
                    &mut prefixes,
                    SettledGlob {
                        source_byte: glob.declaration.source_byte,
                        prefix,
                        provider: glob.provider().cloned(),
                    },
                    0,
                    limits,
                )?;
            }
            scopes.retain_glob_states(retained, limits)?;
            cache.retained += retained;
            cache.contexts.insert(
                context.clone(),
                SettledFrame {
                    prefixes,
                    heads: BTreeMap::new(),
                },
            );
        }
        let settled = cache
            .contexts
            .get_mut(context)
            .expect("INVARIANT: the exact Source context has settled prefixes");
        if exclusion.is_some() {
            return self.lookup_glob_head(settled, head, namespace, exclusion, limits);
        }
        let heads = settled.heads.get(&namespace);
        charge(
            self.label,
            limits,
            u64::from(heads.map_or(0, BTreeMap::len).bit_width()) + 1,
        )?;
        if let Some(result) = heads.and_then(|heads| heads.get(head)) {
            return Ok(result.clone());
        }
        let result = self.lookup_glob_head(settled, head, namespace, None, limits)?;
        let retained = usize::from(heads.is_none())
            + result
                .route
                .as_ref()
                .map_or(1, |route| route.sites.capacity() + 1);
        scopes.retain_glob_states(retained, limits)?;
        cache.retained += retained;
        settled
            .heads
            .entry(namespace)
            .or_default()
            .insert(head.to_owned(), result.clone());
        Ok(result)
    }

    fn lookup_glob_head(
        &self,
        settled: &SettledFrame,
        head: &str,
        namespace: Namespace,
        exclusion: Option<usize>,
        limits: &mut MatchLimits,
    ) -> Result<GlobHead, Error> {
        let mut found = GlobHead::default();
        for glob in settled.prefixes.iter() {
            charge(self.label, limits, 1)?;
            if exclusion.is_some_and(|byte| glob.source_byte == Some(byte)) {
                continue;
            }
            match &glob.provider {
                Some(GlobProvider::Module) => {
                    if let Some(route) = self.glob_member(&glob.prefix, head, namespace, limits)? {
                        self.select_glob(&mut found.route, route, head)?;
                    }
                }
                Some(provider) => found.variant |= provider.variant(head, namespace, limits)?,
                None => {
                    found.opaque.get_or_insert_with(|| glob.prefix.clone());
                }
            }
        }
        if found.variant && found.route.is_some() {
            return Err(failure(
                self.label,
                &format!("written path head {head:?} has competing block glob bindings"),
            ));
        }
        Ok(found)
    }

    fn glob_provider(
        &self,
        scopes: &Scopes,
        sites: &[ItemId],
        limits: &mut MatchLimits,
    ) -> Result<GlobProvider, Error> {
        let (workspace, _) = self
            .visibility
            .expect("INVARIANT: block globs are collected for visibility");
        let mut enums = WorkList::new();
        let mut modules = 0;
        for &id in sites {
            charge(self.label, limits, 1)?;
            let item = workspace.item(id).ok_or_else(|| {
                failure(self.label, "block glob prefix has no indexed declaration")
            })?;
            match item.kind {
                SymbolKind::Mod => modules += 1,
                SymbolKind::Enum => push(
                    self.label,
                    &mut enums,
                    self.enum_members(scopes, workspace, item, limits)?,
                    0,
                    limits,
                )?,
                _ => {}
            }
        }
        if modules == sites.len() {
            Ok(GlobProvider::Module)
        } else if enums.len() == sites.len() {
            Ok(GlobProvider::Enum(Rc::new(enums)))
        } else if modules == 0 && enums.is_empty() {
            Ok(GlobProvider::Other)
        } else {
            Err(failure(
                self.label,
                "block glob prefix has competing declaration kinds; qualify the import",
            ))
        }
    }

    fn enum_members(
        &self,
        scopes: &Scopes,
        workspace: &Workspace,
        item: &raf_source::Item,
        limits: &mut MatchLimits,
    ) -> Result<Rc<EnumMembers>, Error> {
        let mut enums = scopes.enums.borrow_mut();
        if let Some(members) = enums.get(&item.id) {
            return Ok(Rc::clone(members));
        }
        let source = workspace
            .source(&item.file)
            .and_then(|source| source.get(item.span.start_byte..item.span.end_byte))
            .ok_or_else(|| failure(self.label, "indexed enum source span is unavailable"))?;
        charge(
            self.label,
            limits,
            source.len().try_into().unwrap_or(u64::MAX),
        )?;
        let capacity = scopes
            .remaining_glob_states(limits)
            .checked_sub(1)
            .ok_or_else(|| failure(self.label, "enum glob roster state limit exhausted"))?;
        let members = raf_source::parse_source(&item.file, source, |syntax| {
            let [syn::Item::Enum(declaration)] = syntax.items.as_slice() else {
                return Err(failure(
                    self.label,
                    "indexed enum span does not name an enum declaration",
                ));
            };
            let mut members = EnumMembers::new();
            for variant in &declaration.variants {
                let name = variant.ident.unraw().to_string();
                let value = !matches!(variant.fields, Fields::Named(_));
                if let Some(known) = members.get_mut(&name) {
                    *known |= value;
                } else {
                    if members.len() >= capacity {
                        return Err(failure(
                            self.label,
                            "enum glob roster state limit exhausted",
                        ));
                    }
                    members.insert(name, value);
                }
            }
            Ok(members)
        })??;
        scopes.retain_glob_states(members.len() + 1, limits)?;
        let members = Rc::new(members);
        enums.insert(item.id, Rc::clone(&members));
        Ok(members)
    }

    fn resolve_block_globs<'s>(
        &self,
        scopes: &'s Scopes,
        wanted_frame: usize,
        limits: &mut MatchLimits,
    ) -> Result<GlobPrefixes<'s>, Error> {
        let mut globs = WorkList::new();
        for (frame, declarations, _) in scopes
            .block_globs()
            .skip_while(|(frame, _, _)| *frame > wanted_frame)
        {
            self.prepare_glob_frame(scopes, frame, declarations, &mut globs, limits)?;
            self.settle_globs(scopes, &globs, limits)?;
            if !globs
                .iter()
                .take_while(|glob| glob.frame == wanted_frame)
                .any(|glob| {
                    glob.relative_prefix().is_some()
                        && (glob.expanded.sees_target_glob(frame)
                            || glob.expanded.alias_frame().is_some())
                })
            {
                break;
            }
        }
        // A later anchored chain can introduce a second candidate in the
        // declaration scope. Recheck each inferred prefix against that scope.
        for glob in globs.iter().filter(|glob| glob.relative_prefix().is_some()) {
            let settled = self.chain_glob(glob, &globs, limits)?;
            if glob.direct.is_none()
                && settled.as_ref().map(|route| &route.sites)
                    != glob.chained.get().map(|(route, _)| &route.sites)
            {
                return Err(failure(
                    self.label,
                    "chained block glob prefix changes binding as imports resolve",
                ));
            }
        }
        Ok(globs)
    }

    fn prepare_glob_frame<'s>(
        &self,
        scopes: &'s Scopes,
        frame: usize,
        declarations: &'s [Glob],
        globs: &mut GlobPrefixes<'s>,
        limits: &mut MatchLimits,
    ) -> Result<(), Error> {
        for declaration in declarations {
            let mut eligible =
                |alias: &Alias, namespace| self.eligible_declaration(alias, namespace);
            let Some(expanded) = scopes.expand_import(
                &declaration.path,
                declaration.absolute,
                Namespace::Type,
                Some(frame + 1),
                limits,
                &mut eligible,
            )?
            else {
                continue;
            };
            let text = expanded.text();
            charge(
                self.label,
                limits,
                text.len().try_into().unwrap_or(u64::MAX),
            )?;
            let mut prefix = ScopedGlob {
                declaration,
                frame,
                expanded,
                text,
                direct: None,
                chained: OnceCell::new(),
            };
            if prefix.has_prefix()
                && let Some(sites) = self.indexed_in(&prefix.text, Namespace::Type, true)?
            {
                prefix.direct = Some(self.glob_provider(scopes, &sites, limits)?);
            }
            push(self.label, globs, prefix, 0, limits)?;
        }
        Ok(())
    }

    fn settle_globs(
        &self,
        scopes: &Scopes,
        globs: &GlobPrefixes<'_>,
        limits: &mut MatchLimits,
    ) -> Result<(), Error> {
        // A chain gets a definition only from a previously proved prefix. Each
        // productive pass resolves at least one declaration, and every candidate
        // examination spends the shared work grant. Unanchored cycles stay opaque.
        loop {
            let mut changed = false;
            for glob in globs.iter() {
                if glob.unresolved_relative()
                    && let Some(route) = self.chain_glob(glob, globs, limits)?
                {
                    let provider = self.glob_provider(scopes, &route.sites, limits)?;
                    glob.chained.set((route, provider)).map_err(|_| {
                        failure(self.label, "block glob prefix already has a settled route")
                    })?;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        Ok(())
    }

    fn chain_glob(
        &self,
        glob: &ScopedGlob<'_>,
        globs: &GlobPrefixes<'_>,
        limits: &mut MatchLimits,
    ) -> Result<Option<GlobRoute>, Error> {
        let Some((head, tail)) = glob.relative_prefix() else {
            return Ok(None);
        };
        let mut selected = None;
        let mut variant = false;
        for candidate in globs.iter() {
            charge(self.label, limits, 1)?;
            if candidate.frame > glob.frame
                || !glob.expanded.sees_target_glob(candidate.frame)
                || std::ptr::eq(candidate.declaration, glob.declaration)
            {
                continue;
            }
            match candidate.provider() {
                Some(GlobProvider::Module) => {
                    if let Some(route) =
                        self.glob_member(candidate.prefix(), head, Namespace::Type, limits)?
                    {
                        self.select_glob(&mut selected, route, head)?;
                    }
                }
                Some(provider) => variant |= provider.variant(head, Namespace::Type, limits)?,
                None => {}
            }
        }
        let Some(route) = selected else {
            return Ok(None);
        };
        if variant {
            return Err(failure(
                self.label,
                &format!("written path head {head:?} has competing block glob bindings"),
            ));
        }
        // Imports cannot select a glob binding over a distinct outer candidate.
        if self.conflicting_outer_glob(head, &route.sites)? {
            return Err(failure(
                self.label,
                &format!(
                    "block glob prefix {head:?} has distinct glob and outer bindings; qualify the import"
                ),
            ));
        }
        if tail.is_empty() {
            return Ok(Some(route));
        }
        let path = format!("{}::{}", route.path, path_text(tail, false));
        charge(
            self.label,
            limits,
            path.len().try_into().unwrap_or(u64::MAX),
        )?;
        Ok(self
            .indexed_in(&path, Namespace::Type, true)?
            .map(|sites| GlobRoute { path, sites }))
    }

    fn conflicting_outer_glob(&self, head: &str, sites: &[ItemId]) -> Result<bool, Error> {
        match self.query(head, Namespace::Type, true) {
            Ok(outer) => Ok(outer.sites != sites),
            Err(CodeResolveError::Resolve(ResolveError::Ambiguous(_))) => Ok(true),
            Err(
                CodeResolveError::Resolve(
                    ResolveError::NotFound(_) | ResolveError::UnresolvedSelf(_),
                )
                | CodeResolveError::AliasMember { .. },
            ) => Ok(false),
            Err(error) => Err(self.error(head, error)),
        }
    }

    fn glob_member(
        &self,
        prefix: &str,
        head: &str,
        namespace: Namespace,
        limits: &mut MatchLimits,
    ) -> Result<Option<GlobRoute>, Error> {
        let path = format!("{prefix}::{head}");
        charge(
            self.label,
            limits,
            path.len().try_into().unwrap_or(u64::MAX),
        )?;
        Ok(self
            .indexed_in(&path, namespace, true)?
            .map(|sites| GlobRoute { path, sites }))
    }

    fn select_glob(
        &self,
        selected: &mut Option<GlobRoute>,
        route: GlobRoute,
        head: &str,
    ) -> Result<(), Error> {
        if selected
            .as_ref()
            .is_some_and(|previous| previous.sites != route.sites)
        {
            return Err(failure(
                self.label,
                &format!("written path head {head:?} has competing block glob bindings"),
            ));
        }
        *selected = Some(route);
        Ok(())
    }

    fn query(
        &self,
        written: &str,
        namespace: Namespace,
        import: bool,
    ) -> Result<Resolution, CodeResolveError> {
        match self.owner {
            PathOwner::Item(item) if import => {
                self.code
                    .resolve_import_from_item_at(item, self.site.byte, written, namespace)
            }
            PathOwner::Item(item) => {
                self.code
                    .resolve_from_item_at(item, self.site.byte, written, namespace)
            }
            PathOwner::Impl(block) => self
                .code
                .resolve_from_impl(block, written, namespace, import),
            PathOwner::File(reach) => self.code.resolve_from_file_reach(reach, written, namespace),
        }
    }

    fn resolve<'a>(
        &self,
        raw: &str,
        expanded: Option<&'a Expanded>,
        limits: &mut MatchLimits,
    ) -> Result<PathDefinition<'a>, Error> {
        charge(self.label, limits, raw.len().try_into().unwrap_or(u64::MAX))?;
        // A block binding shadows the module spelling even when its target
        // has no indexed definition. Module imports use the Source index.
        if let Some(block) = expanded.filter(|path| path.block) {
            return self.expanded(block);
        }
        if let Some(actual) = self.indexed(raw, self.site.is_import())? {
            return Ok(PathDefinition::Written(actual));
        }
        expanded.map_or(Ok(PathDefinition::Missing), |path| self.expanded(path))
    }

    fn expanded<'a>(&self, path: &'a Expanded) -> Result<PathDefinition<'a>, Error> {
        if path.shadowed {
            return Ok(PathDefinition::Missing);
        }
        Ok(self.indexed(&path.text(), true)?.map_or(
            PathDefinition::Unverified(&path.segments),
            PathDefinition::Expanded,
        ))
    }
}

const VISIBILITY: &str = "visibility";

pub(in crate::gate) fn visibility_resolution(
    workspace: &Workspace,
    consumer: PackageId,
    result: Result<Resolution, CodeResolveError>,
) -> Result<Option<Resolution>, CodeResolveError> {
    match result {
        Ok(actual) => Ok(Some(actual)),
        Err(
            CodeResolveError::Resolve(ResolveError::NotFound(_) | ResolveError::UnresolvedSelf(_))
            | CodeResolveError::AliasMember { .. },
        ) => Ok(None),
        // Structural indexing can see alternate internal definitions of one
        // spelling, such as the test and shipping versions of a function.
        Err(ref error) if internal_ambiguity(workspace, consumer, error) => Ok(None),
        // Inherited associated candidates are outside the public declaration
        // inventory. The independently observed type prefix still has evidence.
        Err(CodeResolveError::Resolve(ResolveError::Ambiguous(items)))
            if items.iter().all(|&id| {
                workspace.item(id).is_some_and(|item| {
                    item.visibility != raf_source::Vis::Public
                        && matches!(
                            item.kind,
                            SymbolKind::InherentMethod
                                | SymbolKind::TraitImplMethod
                                | SymbolKind::TraitDefaultMethod
                                | SymbolKind::TraitRequiredMethod
                                | SymbolKind::AssocType
                                | SymbolKind::AssocConst
                        )
                })
            }) =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn internal_ambiguity(
    workspace: &Workspace,
    consumer: PackageId,
    error: &CodeResolveError,
) -> bool {
    let CodeResolveError::Resolve(ResolveError::Ambiguous(items)) = error else {
        return false;
    };
    items.iter().all(|&id| {
        workspace
            .item(id)
            .and_then(|item| workspace.target(item.target))
            .is_some_and(|target| target.package_id == consumer)
    })
}

/// Distinct written paths by defining item and consuming package.
#[derive(Default)]
pub(in crate::gate) struct CrossPackagePaths {
    pub(in crate::gate) by_item: BTreeMap<ItemId, BTreeMap<PackageId, BTreeSet<String>>>,
    retained: usize,
}

impl CrossPackagePaths {
    fn record_occurrence(
        &mut self,
        workspace: &Workspace,
        consumer: PackageId,
        definition: ItemId,
        written: &str,
        (site, opaque): (&Site<'_>, Option<&str>),
        limits: &MatchLimits,
    ) -> Result<(), Error> {
        if let Some(opaque) = opaque
            && workspace
                .item(definition)
                .and_then(|item| workspace.target(item.target))
                .is_some_and(|target| {
                    target.kind == TargetKind::Lib && target.package_id != consumer
                })
        {
            return Err(Error::Source {
                file: site.file.path.to_string(),
                line: site.line,
                reason: format!(
                    "visibility path {written:?}: unindexed block glob {opaque:?} may change its binding; qualify the path or use an explicit import"
                ),
            });
        }
        self.record(workspace, consumer, definition, written, limits)
    }

    pub(in crate::gate) fn record(
        &mut self,
        workspace: &Workspace,
        consumer: PackageId,
        definition: ItemId,
        written: &str,
        limits: &MatchLimits,
    ) -> Result<(), Error> {
        let item = workspace
            .item(definition)
            .ok_or_else(|| failure(VISIBILITY, "resolved path has no defining item"))?;
        let target = workspace
            .target(item.target)
            .ok_or_else(|| failure(VISIBILITY, "resolved path has no defining target"))?;
        if target.kind != TargetKind::Lib
            || target.package_id == consumer
            || workspace.same_source(item).is_empty()
        {
            return Ok(());
        }
        let paths = self
            .by_item
            .entry(definition)
            .or_default()
            .entry(consumer)
            .or_default();
        if paths.contains(written) {
            return Ok(());
        }
        if self.retained >= limits.states {
            return Err(failure(VISIBILITY, "written path state limit exhausted"));
        }
        paths.insert(written.to_owned());
        self.retained += 1;
        Ok(())
    }
}

fn cross_package_uses(
    workspace: &Workspace,
    code: &CodeResolver<'_>,
    (owners, blocks): (&[&raf_source::Item], &[ImplBlock]),
    scopes: &Scopes,
    path: &syn::Path,
    site: &Site<'_>,
    (paths, limits): (&mut CrossPackagePaths, &mut MatchLimits),
) -> Result<bool, Error> {
    let written = segments(path);
    let absolute = path.leading_colon.is_some();
    let raw = path_text(&written, absolute);
    let observed = if site.role == Role::Glob {
        format!("{raw}::*")
    } else {
        raw.clone()
    };
    let in_impl = |owner: &&raf_source::Item| blocks.is_empty() || owner.kind != SymbolKind::Mod;
    let owner_contexts = owners
        .iter()
        .copied()
        .filter(in_impl)
        .map(|item| (PathOwner::Item(item.id), item.target));
    let impl_contexts = blocks
        .iter()
        .filter(|block| {
            !owners
                .iter()
                .copied()
                .filter(in_impl)
                .any(|item| item.target == block.target)
        })
        .map(|block| (PathOwner::Impl(block.id), block.target));
    let file_contexts = site
        .file
        .reached
        .iter()
        .filter(|reach| {
            !site.is_import()
                && blocks.is_empty()
                && (owners.is_empty() || site.file_scope)
                && !owners.iter().any(|item| {
                    item.target == reach.target
                        && code
                            .reading_module(item.id, site.byte)
                            .is_ok_and(|module| module == reach.module_path.as_str())
                })
        })
        .map(|reach| (PathOwner::File(reach), reach.target));
    let mut classified = None;
    for (owner, target) in owner_contexts.chain(impl_contexts).chain(file_contexts) {
        let consumer = workspace
            .target(target)
            .ok_or_else(|| failure(VISIBILITY, "path occurrence has no consuming target"))?
            .package_id;
        let lookup = Lookup {
            code,
            owner,
            site,
            label: VISIBILITY,
            visibility: Some((workspace, consumer)),
        };
        let LexicalPath::Accessible(expanded) =
            lookup.lexical(scopes, &written, absolute, limits)?
        else {
            continue;
        };
        let glob = lookup.glob_occurrence(scopes, &written, absolute, expanded.as_ref(), limits)?;
        let definition = match glob.definition {
            Some(definition) => definition,
            None => lookup.resolve(&raw, expanded.as_ref(), limits)?,
        };
        let actual = match &definition {
            PathDefinition::Written(actual) | PathDefinition::Expanded(actual) => actual.as_slice(),
            PathDefinition::Missing | PathDefinition::Unverified(_) => &[],
        };
        if site.role == Role::Pattern
            && !classify_pattern(workspace, VISIBILITY, actual, &mut classified)?
        {
            continue;
        }
        for &definition in actual {
            paths.record_occurrence(
                workspace,
                consumer,
                definition,
                &observed,
                (site, glob.opaque.as_deref()),
                limits,
            )?;
        }
    }
    Ok(classified.unwrap_or(false))
}

fn classify_pattern(
    workspace: &Workspace,
    label: &str,
    sites: &[ItemId],
    classified: &mut Option<bool>,
) -> Result<bool, Error> {
    use raf_source::FieldShape;

    let mut kind = None;
    for &site in sites {
        let item = workspace
            .item(site)
            .ok_or_else(|| failure(label, "resolved pattern constant has no source item"))?;
        let constant = matches!(item.kind, SymbolKind::Const | SymbolKind::AssocConst)
            || (item.kind == SymbolKind::Struct && item.shape == Some(FieldShape::Unit));
        if kind
            .replace(constant)
            .is_some_and(|previous| previous != constant)
        {
            return Err(failure(
                label,
                "pattern identifier has mixed constant/binding definitions; provide an unambiguous source context",
            ));
        }
    }
    let constant = kind.unwrap_or(false);
    if classified
        .replace(constant)
        .is_some_and(|previous| previous != constant)
    {
        return Err(failure(
            label,
            "pattern identifier changes binding meaning across source reaches; provide an unambiguous source context",
        ));
    }
    Ok(constant)
}

/// Indexed namespaces select imports; an entirely unindexed target stays
/// an unverified syntax observation, never a provider identity.
pub(in crate::gate) fn import_eligible(
    code: &CodeResolver<'_>,
    owner: ItemId,
    byte: usize,
    alias: &Alias,
    namespace: Namespace,
) -> Result<bool, CodeResolveError> {
    let text = path_text(&alias.path, alias.absolute);
    eligible_namespace(namespace, |space| {
        code.resolve_import_from_item_at(owner, byte, &text, space)
            .map(|_| true)
    })
}

fn eligible_namespace(
    namespace: Namespace,
    mut resolve: impl FnMut(Namespace) -> Result<bool, CodeResolveError>,
) -> Result<bool, CodeResolveError> {
    match resolve(namespace) {
        Ok(true) => return Ok(true),
        Ok(false) | Err(CodeResolveError::Resolve(ResolveError::NotFound(_))) => {}
        Err(error) => return Err(error),
    }
    for other in Namespace::ALL {
        if other == namespace {
            continue;
        }
        match resolve(other) {
            Ok(true) => return Ok(false),
            Ok(false) | Err(CodeResolveError::Resolve(ResolveError::NotFound(_))) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(true)
}

fn constant<'w>(
    workspace: &'w Workspace,
    code: &CodeResolver<'_>,
    file: &SourceFile,
    id: &str,
    name: &syn::Ident,
    cache: &mut OwnerCache<'w>,
    limits: &mut MatchLimits,
) -> Result<bool, Error> {
    let site = Site::at(file, name.span(), Role::Path, Namespace::Value, false);
    let written = name.unraw().to_string();
    let mut classified = None;
    let owners = cache.get(workspace, file, site.byte, limits)?;
    for owner in owners {
        let sites = match code.resolve_from_item_at(owner.id, site.byte, &written, Namespace::Value)
        {
            Ok(resolution) => resolution.sites,
            Err(CodeResolveError::Resolve(ResolveError::NotFound(_))) => Vec::new(),
            Err(error) => return Err(query_error(id, &site, error)),
        };
        classify_pattern(workspace, id, &sites, &mut classified)?;
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
    let absolute = path.leading_colon.is_some();
    let raw = path_text(&written, absolute);
    if owners.is_empty() {
        return Err(failure(
            &expected.pattern.id,
            "path occurrence has no source owner",
        ));
    }
    let declares = |actual: &[ItemId]| {
        expected
            .sites
            .get(&site.namespace)
            .is_some_and(|wanted| actual.iter().any(|id| wanted.contains(id)))
    };
    for owner in owners {
        let lookup = Lookup {
            code,
            owner: PathOwner::Item(owner.id),
            site,
            label: &expected.pattern.id,
            visibility: None,
        };
        let LexicalPath::Accessible(expanded) =
            lookup.lexical(scopes, &written, absolute, limits)?
        else {
            continue;
        };
        if named_macro {
            if expanded
                .as_ref()
                .is_some_and(|expanded| expanded.segments.last() == expected.spelling.last())
            {
                return Ok(true);
            }
            continue;
        }
        let matched = match lookup.resolve(&raw, expanded.as_ref(), limits)? {
            PathDefinition::Written(actual) => {
                declares(&actual)
                    || (expected.spelling.len() == 1
                        && expected.sites.is_empty()
                        && written == expected.spelling)
            }
            PathDefinition::Expanded(actual) => declares(&actual),
            PathDefinition::Unverified(spelling) => {
                expected.sites.is_empty() && spelling == expected.spelling
            }
            PathDefinition::Missing => false,
        };
        if matched {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::{CrossPackagePaths, OwnerCache, visit};
    use purrdf_jsonschema::ecma::MatchLimits;
    use raf_source::{LoadOptions, MemoryTree, RelPath, Workspace};

    #[test]
    fn repeated_glob_paths_reuse_work_without_merging_distinct_source_reaches() {
        // The two reaches use different actual providers of the same head.
        // Repeated calls fit the work grant only with retained declaration work.
        let mut definitions = String::from(
            "pub mod scope_0 { pub fn hit() {} }\npub mod scope_1 { pub fn hit() {} }\n",
        );
        let mut consumer = String::from("fn observe() { use super::provider::*;\n");
        for index in 2..30 {
            writeln!(definitions, "pub mod scope_{index} {{}}").unwrap();
            writeln!(consumer, "use dep::scope_{index}::*;").unwrap();
        }
        for _ in 0..500 {
            consumer.push_str("hit();\n");
        }
        consumer.push('}');
        let tree = MemoryTree::from([
            (
                "Cargo.toml",
                "[workspace]\nmembers=['dep','client']\nresolver='2'\n",
            ),
            (
                "dep/Cargo.toml",
                "[package]\nname='dep'\nversion='0.1.0'\nedition='2024'\n",
            ),
            ("dep/src/lib.rs", definitions.as_str()),
            (
                "client/Cargo.toml",
                "[package]\nname='client'\nversion='0.1.0'\nedition='2024'\n[dependencies]\ndep={path='../dep'}\n",
            ),
            ("client/src/lib.rs", "mod a; mod b;"),
            (
                "client/src/a.rs",
                "pub use dep::scope_0 as provider; #[path=\"shared.rs\"] mod shared;",
            ),
            (
                "client/src/b.rs",
                "pub use dep::scope_1 as provider; #[path=\"shared.rs\"] mod shared;",
            ),
            ("client/src/shared.rs", consumer.as_str()),
        ]);
        let workspace =
            Workspace::load_members(&tree, &RelPath::root(), &LoadOptions::default()).unwrap();
        let file = workspace
            .file(&RelPath::new("client/src/shared.rs").unwrap())
            .unwrap();
        let code = workspace.code_resolver().unwrap();
        let observe = |steps| {
            let mut paths = CrossPackagePaths::default();
            let mut limits = MatchLimits {
                steps,
                states: 4_096,
            };
            raf_source::parse_source(&file.path, &file.text, |syntax| {
                visit::references(&workspace, &code, file, syntax, &mut limits, &mut paths)
            })
            .unwrap()?;
            Ok::<_, crate::Error>(paths)
        };
        let paths = observe(250_000).unwrap();
        for symbol in ["dep::scope_0::hit", "dep::scope_1::hit"] {
            let sites = workspace.resolve(symbol).unwrap().sites;
            assert!(
                sites
                    .iter()
                    .any(|id| paths.by_item.get(id).is_some_and(|uses| {
                        uses.values().any(|written| written.contains("hit"))
                    })),
                "missing {symbol}"
            );
        }
        assert!(matches!(observe(4_096), Err(error) if error.to_string().contains("step limit")));
    }

    #[test]
    fn ownership_scans_spend_the_grant_once_per_byte_and_bound_retention() {
        let mut source = String::new();
        for index in 0..64 {
            writeln!(source, "pub fn owner_{index}(){{}}").unwrap();
        }
        let tree = MemoryTree::from([
            ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\n"),
            ("src/lib.rs", source.as_str()),
        ]);
        let workspace = Workspace::load(&tree, &["."]).unwrap();
        let file = workspace.files().next().unwrap();
        let count = workspace.item_count_in(&file.path);
        assert_eq!(count, workspace.items_in(&file.path).count());
        let mut cache = OwnerCache::default();
        let mut limits = MatchLimits {
            steps: 10_000,
            states: 128,
        };
        let first = cache.get(&workspace, file, 0, &mut limits).unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].path, "demo::owner_0");
        // A repeated lookup can afford only the already stored result.
        limits.steps = 32;
        assert_eq!(
            cache.get(&workspace, file, 0, &mut limits).unwrap()[0].path,
            "demo::owner_0"
        );
        let second = file.text.find("owner_42").unwrap();
        limits.steps = 32;
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
            "demo::owner_42"
        );
        // Visibility retains a single result but reuses the span index for
        // distinct, nonsequential bytes under a grant too small for a scan.
        let mut visibility = OwnerCache::one_occurrence();
        visibility
            .get(&workspace, file, second, &mut limits)
            .unwrap();
        for index in [63, 2, 29, 0] {
            limits.steps = 64;
            let byte = file.text.find(&format!("owner_{index}(")).unwrap();
            let owners = visibility.get(&workspace, file, byte, &mut limits).unwrap();
            assert_eq!(owners.len(), 1);
            assert_eq!(owners[0].path, format!("demo::owner_{index}"));
        }
        let mut refused = OwnerCache::default();
        limits.states = 0;
        assert!(
            refused
                .get(&workspace, file, 0, &mut limits)
                .unwrap_err()
                .to_string()
                .contains("state limit")
        );
        let mut generous = OwnerCache::default();
        limits.steps = 10_000;
        limits.states = usize::MAX;
        assert_eq!(
            generous.get(&workspace, file, 0, &mut limits).unwrap()[0].path,
            "demo::owner_0"
        );
    }
}
