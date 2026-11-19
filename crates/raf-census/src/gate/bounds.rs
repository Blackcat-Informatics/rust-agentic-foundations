// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Physical numeric bound inventory, independent of job homes and variants.

pub(in crate::gate) mod syntax;

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use proc_macro2::Span;
use purrdf_jsonschema::ecma::MatchLimits;
use raf_source::{CodeResolver, ItemId, RelPath, SourceFile, Workspace};
use syn::spanned::Spanned;

use super::{patterns, ratchet};
use crate::detectors::ReceiverMode;
use crate::report::bounds::{BoundRegistry, REFUSING, TRUNCATING, UNCLASSIFIED};
use crate::{DebtSite, Error, SiteKey};

pub use crate::report::bounds::BoundClassification;

/// The syntactic evidence for how a numeric bound is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoundRole {
    /// A numeric constant or static with a limit name.
    Named,
    /// A comparison against a length or count.
    Comparison,
    /// A call argument in a limiting operation.
    Limiter,
    /// A literal passed to an indexed size-named parameter.
    SizeArgument,
}

impl BoundRole {
    fn label(self) -> &'static str {
        match self {
            Self::Named => "named limit",
            Self::Comparison => "length or count comparison",
            Self::Limiter => "limiting operation",
            Self::SizeArgument => "size argument",
        }
    }

    fn debt_rule(self) -> Option<&'static str> {
        match self {
            Self::Comparison => Some(REFUSING),
            Self::Limiter => Some(TRUNCATING),
            Self::Named | Self::SizeArgument => None,
        }
    }
}

/// A distinct numeric bound and its ledger classification, when declared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericBound {
    /// The canonical file to use in a ledger row.
    pub file: RelPath,
    /// The exact site key to use in a ledger row.
    pub key: SiteKey,
    /// The one-based line of the identifier or literal.
    pub line: usize,
    /// The zero-based byte offset of the identifier or literal.
    pub start_byte: usize,
    /// The evidence recognized at this position.
    pub roles: BTreeSet<BoundRole>,
    /// The declared classification, if the ledger supplies one.
    pub classification: Option<BoundClassification>,
}

#[derive(Default)]
pub(super) struct Scan {
    signatures: BTreeMap<(RelPath, usize), syntax::Slots>,
    sites: BTreeMap<(RelPath, usize), Pending>,
    calls: Vec<Call>,
    declarations: BTreeMap<(RelPath, usize), usize>,
    uses: Vec<(BoundRole, Vec<Option<ItemId>>)>,
    retained: usize,
}

struct Pending {
    line: usize,
    context: Range<usize>,
    roles: BTreeSet<BoundRole>,
}

#[derive(Clone, Copy)]
struct Token {
    byte: usize,
    line: usize,
}

impl From<Span> for Token {
    fn from(span: Span) -> Self {
        Self {
            byte: span.byte_range().start,
            line: span.start().line,
        }
    }
}

struct Call {
    file: RelPath,
    context: Range<usize>,
    arguments: Vec<(usize, Token)>,
    definitions: Vec<Option<ItemId>>,
    arity: usize,
    receiver: Option<ReceiverMode>,
}

impl Scan {
    pub(super) fn file(
        &mut self,
        (workspace, code): (&Workspace, &CodeResolver<'_>),
        file: &SourceFile,
        syntax: &syn::File,
        limits: &mut MatchLimits,
    ) -> Result<(), Error> {
        syntax::collect(self, file, syntax, limits)?;
        patterns::paths::visit::numeric_calls(workspace, code, file, syntax, limits, self)
    }

    fn retain(&mut self, amount: usize, limits: &MatchLimits) -> Result<(), Error> {
        let Some(retained) = self
            .retained
            .checked_add(amount)
            .filter(|total| *total <= limits.states)
        else {
            return Err(patterns::failure(
                UNCLASSIFIED,
                "numeric bounds inventory state limit exhausted; narrow the roots or raise the explicit matching grant",
            ));
        };
        self.retained = retained;
        Ok(())
    }

    fn site(
        &mut self,
        file: &RelPath,
        token: Token,
        context: Range<usize>,
        role: BoundRole,
        limits: &MatchLimits,
    ) -> Result<(), Error> {
        let byte = token.byte;
        if let Some(site) = self.sites.get_mut(&(file.clone(), byte)) {
            site.roles.insert(role);
            return Ok(());
        }
        self.retain(1, limits)?;
        self.sites.insert(
            (file.clone(), byte),
            Pending {
                line: token.line,
                context,
                roles: BTreeSet::from([role]),
            },
        );
        Ok(())
    }

    pub(in crate::gate) fn literal(
        &mut self,
        file: &RelPath,
        token: Span,
        context: Span,
        role: BoundRole,
        limits: &MatchLimits,
    ) -> Result<(), Error> {
        self.site(file, token.into(), context.byte_range(), role, limits)
    }

    pub(in crate::gate) fn call(
        &mut self,
        file: &RelPath,
        call: &syn::ExprCall,
        limits: &mut MatchLimits,
        resolve: impl FnOnce(&mut MatchLimits) -> Result<Vec<Option<ItemId>>, Error>,
    ) -> Result<(), Error> {
        self.queue_call(
            file,
            call.span().byte_range(),
            &call.args,
            None,
            limits,
            resolve,
        )
    }

    pub(in crate::gate) fn method(
        &mut self,
        file: &RelPath,
        call: &syn::ExprMethodCall,
        receiver: ReceiverMode,
        limits: &mut MatchLimits,
        resolve: impl FnOnce(&mut MatchLimits) -> Result<Vec<Option<ItemId>>, Error>,
    ) -> Result<(), Error> {
        self.queue_call(
            file,
            call.span().byte_range(),
            &call.args,
            Some(receiver),
            limits,
            resolve,
        )
    }

    fn queue_call(
        &mut self,
        file: &RelPath,
        context: Range<usize>,
        arguments: &syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
        receiver: Option<ReceiverMode>,
        limits: &mut MatchLimits,
        resolve: impl FnOnce(&mut MatchLimits) -> Result<Vec<Option<ItemId>>, Error>,
    ) -> Result<(), Error> {
        let literals: Vec<_> = arguments
            .iter()
            .enumerate()
            .filter_map(|(index, argument)| {
                syntax::integer(argument).map(|span| (index, Token::from(span)))
            })
            .collect();
        if literals.is_empty() {
            return Ok(());
        }
        let definitions = resolve(limits)?;
        if definitions.iter().any(Option::is_some) {
            self.retain(
                1usize
                    .saturating_add(literals.len())
                    .saturating_add(definitions.len()),
                limits,
            )?;
            self.calls.push(Call {
                file: file.clone(),
                context,
                arguments: literals,
                definitions,
                arity: arguments.len(),
                receiver,
            });
        }
        Ok(())
    }

    fn classify_arguments(
        &mut self,
        workspace: &Workspace,
        limits: &mut MatchLimits,
    ) -> Result<(), Error> {
        let signatures = std::mem::take(&mut self.signatures);
        for call in std::mem::take(&mut self.calls) {
            let slots: Vec<_> = call
                .definitions
                .iter()
                .map(|id| {
                    let item = workspace.item((*id)?)?;
                    signatures.get(&(item.file.clone(), item.span.start_byte))
                })
                .collect();
            for (index, token) in call.arguments {
                self.retained -= 1;
                patterns::charge(
                    UNCLASSIFIED,
                    limits,
                    slots.len().try_into().unwrap_or(u64::MAX),
                )?;
                let sits_in_size_slot = |slot: Option<&syntax::Slots>| {
                    slot.is_some_and(|slots| slots.accepts(call.receiver, call.arity, index))
                };
                if slots.iter().copied().any(sits_in_size_slot) {
                    if !slots.iter().copied().all(sits_in_size_slot) {
                        return Err(patterns::failure(
                            UNCLASSIFIED,
                            &format!(
                                "{}:{}: size-parameter evidence for argument {} changes across source definitions; use a callable with a consistent signature in every reach",
                                call.file,
                                token.line,
                                index + 1
                            ),
                        ));
                    }
                    self.site(
                        &call.file,
                        token,
                        call.context.clone(),
                        BoundRole::SizeArgument,
                        limits,
                    )?;
                }
            }
            self.retained -= 1 + call.definitions.len();
        }
        self.retained -= signatures
            .values()
            .map(syntax::Slots::states)
            .sum::<usize>();
        Ok(())
    }

    pub(super) fn evaluate(
        mut self,
        workspace: &Workspace,
        registry: &BoundRegistry,
        limits: &mut MatchLimits,
        findings: &mut Vec<String>,
    ) -> Result<(Vec<NumericBound>, ratchet::Counted), Error> {
        self.classify_arguments(workspace, limits)?;
        self.classify_named_uses(workspace, limits)?;
        let mut inventory = Vec::new();
        let mut counted = ratchet::Counted::default();
        let mut live = BTreeSet::new();
        for ((file, start_byte), hit) in self.sites {
            let text = workspace
                .source(&file)
                .and_then(|source| source.get(hit.context))
                .ok_or_else(|| {
                    patterns::failure(
                        UNCLASSIFIED,
                        &format!("{file}:{}: bound context lies outside source", hit.line),
                    )
                })?;
            patterns::charge(
                UNCLASSIFIED,
                limits,
                text.len().try_into().unwrap_or(u64::MAX),
            )?;
            let key = SiteKey::numeric_bound(start_byte, text);
            let file_text = file.to_string();
            let site = (file.clone(), key);
            let classification = registry.rows.get(&site).cloned();
            let roles = hit
                .roles
                .iter()
                .map(|role| role.label())
                .collect::<Vec<_>>()
                .join(", ");
            let location = format!(
                "{file}:{}: byte {start_byte}: {roles}; siteKey {}",
                hit.line,
                key.to_hex()
            );
            if let Some(BoundClassification::Sanctioned { anchor, .. }) = &classification {
                super::evaluate::documented(workspace, anchor, findings);
            }
            if classification.is_none() && !registry.ratcheted {
                findings.push(format!(
                    "{location}: unclassified numeric bound; add a ledger classification"
                ));
            }
            if registry.ratcheted {
                let debt_key = SiteKey::digest(&file_text, None);
                let guards = hit
                    .roles
                    .iter()
                    .filter_map(|role| role.debt_rule())
                    .chain(classification.is_none().then_some(UNCLASSIFIED));
                for rule in guards {
                    let site = DebtSite::new(&file_text, rule, debt_key)?;
                    counted.record(site, location.clone());
                }
            }
            live.insert(site);
            inventory.push(NumericBound {
                file,
                key,
                line: hit.line,
                start_byte,
                roles: hit.roles,
                classification,
            });
        }
        for (file, key) in registry.rows.keys().filter(|site| !live.contains(*site)) {
            findings.push(format!("{file}: stale numeric bound siteKey {}; remove the classification or update it to a current site", key.to_hex()));
        }
        Ok((inventory, counted))
    }

    pub(in crate::gate) fn named_use(
        &mut self,
        role: BoundRole,
        definitions: Vec<Option<ItemId>>,
        limits: &MatchLimits,
    ) -> Result<(), Error> {
        if definitions.iter().any(Option::is_some) {
            self.retain(1usize.saturating_add(definitions.len()), limits)?;
            self.uses.push((role, definitions));
        }
        Ok(())
    }

    fn classify_named_uses(
        &mut self,
        workspace: &Workspace,
        limits: &mut MatchLimits,
    ) -> Result<(), Error> {
        for (role, definitions) in std::mem::take(&mut self.uses) {
            self.retained -= 1 + definitions.len();
            patterns::charge(
                UNCLASSIFIED,
                limits,
                definitions.len().try_into().unwrap_or(u64::MAX),
            )?;
            for id in definitions.into_iter().flatten() {
                if let Some(item) = workspace.item(id)
                    && let Some(byte) = self
                        .declarations
                        .get(&(item.file.clone(), item.span.start_byte))
                    && let Some(pending) = self.sites.get_mut(&(item.file.clone(), *byte))
                {
                    pending.roles.insert(role);
                }
            }
        }
        Ok(())
    }
}
