// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Finite returned-provider-value proof; unsupported computation is a refusal.

mod generic;

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use raf_source::{CodeResolver, Namespace, Workspace};
use syn::ext::IdentExt;

use super::{Expected, Scopes, charge, configured};
use crate::Error;

pub(super) fn attributes(job: &str, attributes: &[syn::Attribute]) -> Result<(), Error> {
    header_attributes(job, attributes, false, false)
}

pub(super) fn type_attributes(job: &str, attributes: &[syn::Attribute]) -> Result<(), Error> {
    header_attributes(job, attributes, true, false)
}

pub(super) fn module_attributes(job: &str, attributes: &[syn::Attribute]) -> Result<(), Error> {
    header_attributes(job, attributes, false, true)
}

fn header_attributes(
    job: &str,
    attributes: &[syn::Attribute],
    nominal: bool,
    module: bool,
) -> Result<(), Error> {
    fn supported(meta: &syn::Meta, nominal: bool, module: bool) -> bool {
        [
            "cfg",
            "doc",
            "inline",
            "cold",
            "must_use",
            "allow",
            "warn",
            "deny",
            "forbid",
            "expect",
            "deprecated",
            "track_caller",
            "target_feature",
            "test",
            "bench",
        ]
        .iter()
        .any(|name| meta.path().is_ident(name))
            || (nominal
                && ["derive", "repr", "non_exhaustive"]
                    .iter()
                    .any(|name| meta.path().is_ident(name)))
            || (module && meta.path().is_ident("path"))
    }
    if attributes
        .iter()
        .any(|attribute| !supported(&attribute.meta, nominal, module))
    {
        return Err(super::failure(
            job,
            "preferred provider definition has an unsupported rewriting attribute; literal body syntax cannot prove its expanded implementation",
        ));
    }
    Ok(())
}

type Events = OrderedTable<usize, ()>;

enum Value {
    Input,
    Provider(Events),
}

impl Value {
    fn forwarded(&self, job: &str) -> Result<Self, Error> {
        match self {
            Self::Input => Ok(Self::Input),
            Self::Provider(events) => {
                // Event ordinals and unit payloads have allocation-free clones.
                let bytes = events
                    .storage_bytes()
                    .map_err(|source| super::storage::error(job, source))?;
                let mut resident = Resident;
                let mut memory = Memory::resume(&mut resident, bytes);
                events
                    .try_clone(&mut memory)
                    .map(Self::Provider)
                    .map_err(|source| super::storage::error(job, source))
            }
        }
    }
}

struct Binding {
    value: Value,
    mutable: bool,
}

struct Proof<'a> {
    workspace: &'a Workspace,
    code: &'a CodeResolver<'a>,
    expected: &'a Expected<'a>,
    scopes: &'a mut Scopes,
    limits: &'a mut MatchLimits,
    bindings: OrderedTable<String, Binding>,
    required: Events,
    state_cells: usize,
    aliases: OrderedTable<raf_source::ItemId, ()>,
}

pub(super) fn check(
    workspace: &Workspace,
    code: &CodeResolver<'_>,
    expected: &Expected<'_>,
    scopes: &mut Scopes,
    signature: &syn::Signature,
    body: &syn::Block,
    limits: &mut MatchLimits,
) -> Result<OrderedTable<raf_source::ItemId, ()>, Error> {
    let mut proof = Proof {
        workspace,
        code,
        expected,
        scopes,
        limits,
        bindings: OrderedTable::default(),
        required: Events::default(),
        state_cells: 0,
        aliases: OrderedTable::default(),
    };
    if signature.asyncness.is_some() {
        return Err(proof.refused(
            "async result propagation needs an explicit supported future-value contract",
        ));
    }
    for argument in &signature.inputs {
        let syn::FnArg::Typed(argument) = argument else {
            return Err(proof.refused("provider forwarding currently requires explicit function parameters rather than an inferred receiver"));
        };
        let syn::Pat::Ident(binding) = argument.pat.as_ref() else {
            return Err(proof.refused("provider parameters need supported identifier bindings"));
        };
        if binding.subpat.is_some() || !argument.attrs.is_empty() || !binding.attrs.is_empty() {
            return Err(proof.refused(
                "provider parameters cannot conceal nested or attributed pattern bindings",
            ));
        }
        proof.retain(1)?;
        proof.insert_binding(
            binding.ident.unraw().to_string(),
            Binding {
                value: Value::Input,
                mutable: false,
            },
        )?;
    }
    for (index, statement) in body.stmts.iter().enumerate() {
        charge(&expected.row.job.id, proof.limits, 1)?;
        match statement {
            syn::Stmt::Item(syn::Item::Use(import)) if import.attrs.is_empty() => {}
            syn::Stmt::Local(local) => proof.local(local)?,
            syn::Stmt::Expr(expression, Some(_)) if index + 1 < body.stmts.len() => {
                let syn::Expr::Call(call) = proof.plain(expression)? else {
                    return Err(proof.refused("only an actual native call may update provider state"));
                };
                if !proof.call(call)?.1 {
                    return Err(proof.refused("discarded provider call does not update admitted native state"));
                }
            }
            syn::Stmt::Expr(expression, semicolon) if index + 1 == body.stmts.len() => {
                let expression = proof.plain(expression)?;
                let value = if let syn::Expr::Return(returned) = expression {
                    proof.value(returned.expr.as_deref().ok_or_else(|| proof.refused("provider result is discarded by an empty return"))?)?
                } else {
                    if semicolon.is_some() { return Err(proof.refused("provider result is discarded instead of returned")); }
                    proof.value(expression)?
                };
                charge(&expected.row.job.id, proof.limits, proof.required.len() as u64)?;
                return match value {
                    Value::Provider(events) if proof.required.ordered().all(|(event, ())| events.get(event).is_some()) => Ok(proof.aliases),
                    Value::Provider(_) => Err(proof.refused("a native state update does not contribute to the returned provider value")),
                    Value::Input => Err(proof.refused("consumer returns a local result rather than the provider result")),
                };
            }
            _ => return Err(proof.refused("unsupported or competing computation beside the provider call; return the actual provider value through a supported adapter")),
        }
    }
    Err(proof.refused("consumer has no returned provider value"))
}

impl Proof<'_> {
    fn insert_binding(&mut self, name: String, binding: Binding) -> Result<(), Error> {
        super::storage::insert(&mut self.bindings, name, binding, &self.expected.row.job.id)?;
        Ok(())
    }

    fn retain(&mut self, cells: usize) -> Result<(), Error> {
        let next = self
            .state_cells
            .checked_add(cells)
            .ok_or_else(|| self.refused("provider value-state limit exhausted"))?;
        if next > self.limits.states {
            return Err(self.refused("provider value-state limit exhausted"));
        }
        self.state_cells = next;
        Ok(())
    }
    fn plain<'e>(&self, mut expression: &'e syn::Expr) -> Result<&'e syn::Expr, Error> {
        loop {
            let attributes = match expression {
                syn::Expr::Paren(node) => &node.attrs,
                syn::Expr::Group(node) => &node.attrs,
                syn::Expr::Path(node) => &node.attrs,
                syn::Expr::Lit(node) => &node.attrs,
                syn::Expr::Reference(node) => &node.attrs,
                syn::Expr::Call(node) => &node.attrs,
                syn::Expr::Return(node) => &node.attrs,
                _ => return Ok(expression),
            };
            if !attributes.is_empty() {
                return Err(self.refused(
                    "body expression attributes are not modeled provider execution evidence",
                ));
            }
            expression = match expression {
                syn::Expr::Paren(node) => &node.expr,
                syn::Expr::Group(node) => &node.expr,
                _ => return Ok(expression),
            };
        }
    }

    fn refused(&self, reason: &str) -> Error {
        let owner = self
            .workspace
            .item(self.expected.owner)
            .expect("checked owner");
        Error::Source {
            file: owner.file.to_string(),
            line: owner.name_line,
            reason: format!(
                "job {:?}, preferred provider {:?}: {reason}",
                self.expected.row.job.id,
                self.expected
                    .row
                    .preferred_provider
                    .as_ref()
                    .expect("provider")
                    .package
            ),
        }
    }

    fn local(&mut self, local: &syn::Local) -> Result<(), Error> {
        let syn::Pat::Ident(binding) = &local.pat else {
            return Err(self
                .refused("provider local result needs a supported immutable identifier binding"));
        };
        let name = binding.ident.unraw().to_string();
        if !local.attrs.is_empty()
            || !binding.attrs.is_empty()
            || binding.by_ref.is_some()
            || binding.subpat.is_some()
            || self.bindings.get(&name).is_some()
        {
            return Err(self.refused(
                "provider result bindings cannot be conditional, mutable, shadowed or replaced",
            ));
        }
        let initializer = local
            .init
            .as_ref()
            .ok_or_else(|| self.refused("provider result binding needs an initializer"))?;
        if initializer.diverge.is_some() {
            return Err(self.refused("provider result has unsupported let-else control flow"));
        }
        let value = self.value(&initializer.expr)?;
        let mutable = binding.mutability.is_some();
        if mutable && !matches!(value, Value::Provider(_)) {
            return Err(self.refused(
                "mutable adapter state must originate in an actual declared provider call",
            ));
        }
        self.retain(
            1 + match &value {
                Value::Input => 0,
                Value::Provider(events) => events.len(),
            },
        )?;
        self.insert_binding(name, Binding { value, mutable })?;
        self.scopes.bind(&local.pat, |_| Ok(true))
    }

    fn value(&mut self, expression: &syn::Expr) -> Result<Value, Error> {
        charge(&self.expected.row.job.id, self.limits, 1)?;
        match self.plain(expression)? {
            syn::Expr::Lit(literal) if !matches!(literal.lit, syn::Lit::Verbatim(_)) => Ok(Value::Input),
            syn::Expr::Path(path) if path.qself.is_none() && path.path.leading_colon.is_none() && path.path.segments.len() == 1 => {
                let value = &self.bindings.get(&path.path.segments[0].ident.unraw().to_string()).ok_or_else(|| self.refused("result/input path is not an actual admitted binding"))?.value;
                if let Value::Provider(events) = value { charge(&self.expected.row.job.id, self.limits, events.len() as u64)?; }
                value.forwarded(&self.expected.row.job.id)
            },
            syn::Expr::Reference(reference) if reference.mutability.is_none() => self.value(&reference.expr),
            syn::Expr::Reference(reference) if self.mutable_provider(&reference.expr)? => self.value(&reference.expr),
            syn::Expr::Call(call) => self.call(call).map(|(value, _)| value),
            _ => Err(self.refused("provider value has unsupported computation or propagation; no syntax/name-only delegation proof is accepted")),
        }
    }

    fn mutable_provider(&self, expression: &syn::Expr) -> Result<bool, Error> {
        let syn::Expr::Path(path) = self.plain(expression)? else {
            return Ok(false);
        };
        if path.qself.is_some()
            || path.path.leading_colon.is_some()
            || path.path.segments.len() != 1
        {
            return Ok(false);
        }
        Ok(self
            .bindings
            .get(&path.path.segments[0].ident.unraw().to_string())
            .is_some_and(|binding| binding.mutable && matches!(binding.value, Value::Provider(_))))
    }

    fn callee(&mut self, call: &syn::ExprCall) -> Result<(), Error> {
        if !call.attrs.is_empty() {
            return Err(self.refused(
                "body expression attributes are not modeled provider execution evidence",
            ));
        }
        let syn::Expr::Path(path) = self.plain(&call.func)? else {
            return Err(self.refused("provider call needs a source-resolved function path"));
        };
        if path.qself.is_some() {
            return Err(self.refused(
                "provider qualified-self call requires a supported exact definition route",
            ));
        }
        for segment in &path.path.segments {
            generic::check(self, &segment.arguments)?;
        }
        self.admit_call_path(path)
    }

    fn admit_call_path(&mut self, path: &syn::ExprPath) -> Result<(), Error> {
        let parts: Vec<_> = path
            .path
            .segments
            .iter()
            .map(|part| part.ident.unraw().to_string())
            .collect();
        let byte = path
            .path
            .segments
            .first()
            .ok_or_else(|| self.refused("provider path is empty"))?
            .ident
            .span()
            .byte_range()
            .start;
        let code = self.code;
        let workspace = self.workspace;
        let owner = self.expected.owner;
        let job = &self.expected.row.job.id;
        let mut eligible = |alias: &crate::syntax::Alias, namespace| {
            super::super::patterns::paths::import_eligible(code, owner, byte, alias, namespace)
                .map_err(|error| {
                    super::path_error(
                        workspace,
                        owner,
                        job,
                        &crate::syntax::path_text(&alias.path, alias.absolute),
                        error,
                    )
                })
        };
        if path.path.leading_colon.is_none()
            && self
                .scopes
                .local(&parts, Namespace::Value, self.limits, &mut eligible)?
        {
            return Err(self.refused("a lexical binding shadows the provider call"));
        }
        let expanded = self.scopes.expand(
            &parts,
            path.path.leading_colon.is_some(),
            Namespace::Value,
            self.limits,
            &mut eligible,
        )?;
        if expanded
            .as_ref()
            .is_some_and(|expanded| expanded.block && expanded.shadowed)
        {
            return Err(self.refused("a block-local definition shadows the provider call"));
        }
        let raw = crate::syntax::path_text(&parts, path.path.leading_colon.is_some());
        let resolved = if let Some(expanded) = expanded.filter(|expanded| expanded.block) {
            self.code.resolve_import_from_item_at(
                self.expected.owner,
                byte,
                &expanded.text(),
                Namespace::Value,
            )
        } else {
            self.code
                .resolve_from_item_at(self.expected.owner, byte, &raw, Namespace::Value)
        }
        .map_err(|error| {
            super::path_error(
                self.workspace,
                self.expected.owner,
                &self.expected.row.job.id,
                &raw,
                error,
            )
        })?;
        if resolved.sites.is_empty()
            || resolved
                .sites
                .iter()
                .any(|site| self.expected.allowed.sites.get(site).is_none())
        {
            return Err(
                self.refused("actual callee is not one of the declared provider definitions")
            );
        }
        for site in resolved.sites {
            configured(self.workspace, site, &self.expected.row.job.id)?;
        }
        for alias in resolved.type_aliases {
            configured(self.workspace, alias, &self.expected.row.job.id)?;
            if self.aliases.get(&alias).is_none() {
                self.retain(1)?;
                super::storage::insert(&mut self.aliases, alias, (), &self.expected.row.job.id)?;
            }
        }
        Ok(())
    }

    fn call(&mut self, call: &syn::ExprCall) -> Result<(Value, bool), Error> {
        self.callee(call)?;
        let mut targets = OrderedTable::default();
        let mut events = Events::default();
        for argument in &call.args {
            if let syn::Expr::Reference(reference) = self.plain(argument)?
                && reference.mutability.is_some()
                && self.mutable_provider(&reference.expr)?
            {
                let syn::Expr::Path(path) = self.plain(&reference.expr)? else {
                    unreachable!("mutable provider is a direct binding")
                };
                super::storage::insert(
                    &mut targets,
                    path.path.segments[0].ident.unraw().to_string(),
                    (),
                    &self.expected.row.job.id,
                )?;
            }
            if let Value::Provider(provenance) = self.value(argument)? {
                charge(
                    &self.expected.row.job.id,
                    self.limits,
                    provenance.len() as u64,
                )?;
                for (event, ()) in provenance.ordered() {
                    super::storage::insert(&mut events, *event, (), &self.expected.row.job.id)?;
                }
            }
        }
        let mutates_provider = !targets.is_empty();
        if mutates_provider {
            let event = self.required.len();
            let mut cells = 1usize;
            for (target, ()) in targets.ordered() {
                let Value::Provider(provenance) = &self
                    .bindings
                    .get(target)
                    .expect("checked native binding")
                    .value
                else {
                    unreachable!("checked native state")
                };
                charge(&self.expected.row.job.id, self.limits, events.len() as u64)?;
                cells = cells
                    .checked_add(
                        events
                            .ordered()
                            .filter(|(event, ())| provenance.get(*event).is_none())
                            .count()
                            + 1,
                    )
                    .ok_or_else(|| self.refused("provider value-state limit exhausted"))?;
            }
            self.retain(cells)?;
            super::storage::insert(&mut self.required, event, (), &self.expected.row.job.id)?;
            super::storage::insert(&mut events, event, (), &self.expected.row.job.id)?;
            for (target, ()) in targets.ordered() {
                let binding = self
                    .bindings
                    .get_mut(target)
                    .expect("checked native binding");
                let Value::Provider(provenance) = &mut binding.value else {
                    unreachable!("checked native state")
                };
                for (event, ()) in events.ordered() {
                    super::storage::insert(provenance, *event, (), &self.expected.row.job.id)?;
                }
            }
        }
        Ok((Value::Provider(events), mutates_provider))
    }
}
