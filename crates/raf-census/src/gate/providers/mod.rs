// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Grounded declared-provider definitions and returned-value policy.

mod flow;
mod routes;
mod storage;

use std::collections::BTreeSet;

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::{
    allocation::{Memory, OrderedTable, Resident},
    walk::WorkList,
};
use raf_source::{
    CodeResolveError, CodeResolver, ItemId, Namespace, RelPath, SourceFile, SymbolKind, TargetId,
    Workspace,
};

use super::patterns::{
    charge, failure,
    paths::{
        scopes::Scopes,
        visit::{self, Container, FunctionObserver},
    },
};
use crate::{Census, Error, Ledger, LedgerJob};

struct Expected<'a> {
    row: &'a LedgerJob,
    owner: ItemId,
    allowed: routes::Allowed<'a>,
}

type SourceFiles = OrderedTable<(TargetId, RelPath), ()>;

pub(super) struct Providers<'a> {
    workspace: &'a Workspace,
    code: &'a CodeResolver<'a>,
    expected: OrderedTable<ItemId, WorkList<Expected<'a>, 4>>,
    checked: OrderedTable<ItemId, ()>,
    apis: OrderedTable<ItemId, &'a LedgerJob>,
    checked_apis: OrderedTable<ItemId, ()>,
    headers: OrderedTable<ItemId, &'a LedgerJob>,
    checked_headers: OrderedTable<ItemId, ()>,
    current_target: Option<TargetId>,
    header_only: bool,
    pending_ancestry: bool,
}

fn compile<'a>(
    ledger: &'a Ledger,
    workspace: &'a Workspace,
    census: &Census,
    code: &'a CodeResolver<'a>,
    limits: &mut MatchLimits,
) -> Result<Providers<'a>, Error> {
    let mut providers = Providers {
        workspace,
        code,
        expected: OrderedTable::default(),
        checked: OrderedTable::default(),
        apis: OrderedTable::default(),
        checked_apis: OrderedTable::default(),
        headers: OrderedTable::default(),
        checked_headers: OrderedTable::default(),
        current_target: None,
        header_only: false,
        pending_ancestry: false,
    };
    let mut states = 0usize;
    for row in ledger.jobs() {
        let Some(provider) = &row.preferred_provider else {
            continue;
        };
        for consumer in &provider.consumers {
            charge(&row.job.id, limits, consumer.len() as u64 + 1)?;
            let resolved = workspace
                .resolve_in(consumer, Namespace::Value)
                .map_err(|error| Error::ProviderPath {
                    job: row.job.id.clone(),
                    file: super::LEDGER_FILE.into(),
                    line: 1,
                    path: consumer.clone(),
                    source: Box::new(CodeResolveError::Resolve(error)),
                })?;
            if resolved.sites.is_empty() {
                return Err(failure(
                    &row.job.id,
                    "provider consumer has no actual source definition",
                ));
            }
            for (owner, ()) in consumer_sites(workspace, &resolved.sites, row, limits)?.ordered() {
                let owner = *owner;
                let item = workspace
                    .item(owner)
                    .ok_or_else(|| failure(&row.job.id, "consumer source item is absent"))?;
                if census.source_context(&item.file).is_none() || item.body.is_none() {
                    return Err(failure(
                        &row.job.id,
                        "provider consumer must be an actual first-party function with a body",
                    ));
                }
                configured(workspace, owner, &row.job.id)?;
                let allowed = routes::allowed(workspace, code, row, owner, limits)?;
                for (site, ()) in allowed.sites.ordered() {
                    storage::insert(&mut providers.apis, *site, row, &row.job.id)?;
                }
                for (alias, ()) in allowed.aliases.ordered() {
                    if providers.headers.get(alias).is_none()
                        && providers.headers.len() >= limits.states
                    {
                        return Err(failure(
                            &row.job.id,
                            "provider declaration-state limit exhausted",
                        ));
                    }
                    storage::insert(&mut providers.headers, *alias, row, &row.job.id)?;
                }
                providers.insert_consumer(owner, allowed, row, &mut states, limits)?;
            }
        }
    }
    Ok(providers)
}

fn consumer_sites(
    workspace: &Workspace,
    declared: &[ItemId],
    row: &LedgerJob,
    limits: &mut MatchLimits,
) -> Result<OrderedTable<ItemId, ()>, Error> {
    let mut sites = OrderedTable::default();
    for site in declared {
        let item = workspace
            .item(*site)
            .ok_or_else(|| failure(&row.job.id, "consumer source item is absent"))?;
        let reaches = workspace.same_source(item);
        charge(&row.job.id, limits, reaches.len() as u64 + 1)?;
        for reach in reaches {
            if sites.get(reach).is_none() && sites.len() >= limits.states {
                return Err(failure(
                    &row.job.id,
                    "provider consumer identity limit exceeded",
                ));
            }
            storage::insert(&mut sites, *reach, (), &row.job.id)?;
        }
    }
    Ok(sites)
}

pub(super) fn check<'a>(
    ledger: &'a Ledger,
    workspace: &'a Workspace,
    census: &Census,
    code: &'a CodeResolver<'a>,
    limits: &mut MatchLimits,
) -> Result<Providers<'a>, Error> {
    let mut providers = compile(ledger, workspace, census, code, limits)?;
    providers.nominal_headers(limits)?;
    if providers.expected.is_empty() {
        return Ok(providers);
    }
    let files = providers.files(limits)?;
    crate::units::on_thread(|| {
        providers.visit_files(&files, limits)?;
        if providers.checked.len() != providers.expected.len() {
            return Err(failure(
                "preferred-provider",
                "some declared consumer bodies were not observed; provide supported source function definitions",
            ));
        }
        if providers.checked_apis.len() != providers.apis.len() {
            return Err(failure(
                "preferred-provider",
                "some provider definitions were not observed; provide supported source function definitions",
            ));
        }
        if providers.pending_ancestry || providers.checked_headers.len() != providers.headers.len()
        {
            providers.header_only = true;
            providers.current_target = None;
            let files = providers.files(limits)?;
            providers.visit_files(&files, limits)?;
        }
        if providers.checked_headers.len() != providers.headers.len() {
            return Err(failure(
                "preferred-provider",
                "some selected nominal declarations were not observed; provide their actual configured source headers",
            ));
        }
        Ok(())
    })??;
    Ok(providers)
}

impl<'a> Providers<'a> {
    fn insert_consumer(
        &mut self,
        owner: ItemId,
        allowed: routes::Allowed<'a>,
        row: &'a LedgerJob,
        states: &mut usize,
        limits: &MatchLimits,
    ) -> Result<(), Error> {
        if self.expected.get(&owner).is_none() {
            if *states >= limits.states {
                return Err(failure(
                    &row.job.id,
                    "provider consumer-state limit exhausted",
                ));
            }
            storage::insert(&mut self.expected, owner, WorkList::new(), &row.job.id)?;
        }
        let entries = self.expected.get_mut(&owner).expect("inserted consumer");
        if !entries
            .iter()
            .any(|expected| expected.row.job.id == row.job.id)
        {
            if *states >= limits.states {
                return Err(failure(
                    &row.job.id,
                    "provider consumer-state limit exhausted",
                ));
            }
            *states += 1;
            entries
                .try_push(Expected {
                    row,
                    owner,
                    allowed,
                })
                .map_err(|source| Error::PatternStorage {
                    pattern: format!("provider:{}", row.job.id),
                    source,
                })?;
        }
        Ok(())
    }

    pub(super) fn api_home<'u>(
        &self,
        owners: &super::owners::Owners<'u>,
        row: &LedgerJob,
        symbol: &str,
        limits: &mut MatchLimits,
    ) -> Result<Option<super::owners::Home<'u>>, Error> {
        if row.preferred_provider.is_none() {
            return Ok(None);
        }
        let mut actual = None;
        for expected in self
            .expected
            .ordered()
            .flat_map(|(_, entries)| entries.iter())
        {
            charge(&row.job.id, limits, expected.allowed.apis.len() as u64 + 1)?;
            if expected.row.job.id != row.job.id {
                continue;
            }
            for (api, site) in expected.allowed.apis.iter() {
                charge(&row.job.id, limits, (api.len() + symbol.len()) as u64 + 1)?;
                if api.trim_start_matches("::") != symbol.trim_start_matches("::") {
                    continue;
                }
                if actual.is_some_and(|previous| previous != *site) {
                    return Err(failure(
                        &row.job.id,
                        &format!(
                            "provider home {symbol:?} resolves to different API definitions across consumer contexts; use one unambiguous locked provider identity"
                        ),
                    ));
                }
                actual = Some(*site);
            }
        }
        let Some(site) = actual else {
            return Ok(None);
        };
        let item = self.workspace.item(site).expect("validated provider API");
        let target = self
            .workspace
            .target(item.target)
            .expect("validated API target");
        Ok(Some(super::owners::Home {
            source_items: BTreeSet::from([site]),
            path: symbol.to_owned(),
            packages: BTreeSet::from([target.package.clone()]),
            units: owners
                .get(&(item.file.clone(), item.span.start_byte))
                .into_iter()
                .copied()
                .collect(),
        }))
    }

    pub(super) fn home_sites(
        &self,
        row: &LedgerJob,
        limits: &mut MatchLimits,
    ) -> Result<OrderedTable<ItemId, ()>, Error> {
        let mut sites = OrderedTable::default();
        if row.preferred_provider.is_none() {
            return Ok(sites);
        }
        for expected in self
            .expected
            .ordered()
            .flat_map(|(_, entries)| entries.iter())
        {
            charge(&row.job.id, limits, expected.allowed.sites.len() as u64 + 1)?;
            if expected.row.job.id == row.job.id {
                for site in std::iter::once(&expected.owner)
                    .chain(expected.allowed.sites.ordered().map(|(site, ())| site))
                {
                    if sites.get(site).is_none() && sites.len() >= limits.states {
                        return Err(failure(
                            &row.job.id,
                            "provider home identity limit exceeded",
                        ));
                    }
                    storage::insert(&mut sites, *site, (), &row.job.id)?;
                }
            }
        }
        let mut proved = OrderedTable::default();
        for (site, ()) in sites.ordered() {
            let item = self.workspace.item(*site).expect("checked provider home");
            let reaches = self.workspace.same_source(item);
            charge(&row.job.id, limits, reaches.len() as u64 + 1)?;
            if !reaches.is_empty() && reaches.iter().all(|reach| sites.get(reach).is_some()) {
                storage::insert(&mut proved, *site, (), &row.job.id)?;
            }
        }
        Ok(proved)
    }

    fn files(&self, limits: &mut MatchLimits) -> Result<SourceFiles, Error> {
        let mut files = OrderedTable::default();
        for (owner, row) in self.owners() {
            let item = self.workspace.item(owner).expect("checked owner");
            storage::insert(
                &mut files,
                (item.target, item.file.clone()),
                (),
                &row.job.id,
            )?;
            for module in self
                .workspace
                .items()
                .filter(|item| item.kind == SymbolKind::Mod)
            {
                charge(&row.job.id, limits, 1)?;
                if self.code.is_defined_in(owner, module.id).map_err(|error| {
                    path_error(self.workspace, owner, &row.job.id, &module.path, error)
                })? {
                    storage::insert(
                        &mut files,
                        (module.target, module.file.clone()),
                        (),
                        &row.job.id,
                    )?;
                }
            }
        }
        Ok(files)
    }

    fn visit_files(&mut self, files: &SourceFiles, limits: &mut MatchLimits) -> Result<(), Error> {
        let workspace = self.workspace;
        let code = self.code;
        for ((target, path), ()) in files.ordered() {
            self.current_target = Some(*target);
            code.with_configured_syntax(*target, path, |file, syntax| {
                visit::functions(workspace, code, file, syntax, limits, self)
            })
            .map_err(|source| Error::ProviderPath {
                job: "preferred-provider".into(),
                file: path.to_string(),
                line: 1,
                path: path.to_string(),
                source: Box::new(source),
            })??;
        }
        Ok(())
    }

    fn nominal_headers(&mut self, limits: &mut MatchLimits) -> Result<(), Error> {
        // ItemId and borrowed LedgerJob clones allocate no child payload.
        let bytes = self
            .headers
            .storage_bytes()
            .map_err(|source| storage::error("preferred-provider", source))?;
        let mut resident = Resident;
        let mut memory = Memory::resume(&mut resident, bytes);
        let mut headers = self
            .headers
            .try_clone(&mut memory)
            .map_err(|source| storage::error("preferred-provider", source))?;
        for (owner, row) in self.owners() {
            for block in self.workspace.impls() {
                charge(&row.job.id, limits, block.members.len() as u64 + 1)?;
                if !block.members.contains(&owner) {
                    continue;
                }
                for site in &block.self_items {
                    configured(self.workspace, *site, &row.job.id)?;
                    if headers.get(site).is_none() && headers.len() >= limits.states {
                        return Err(failure(
                            &row.job.id,
                            "provider declaration-state limit exhausted",
                        ));
                    }
                    storage::insert(&mut headers, *site, row, &row.job.id)?;
                }
            }
        }
        self.headers = headers;
        Ok(())
    }

    fn owners(&self) -> impl Iterator<Item = (ItemId, &'a LedgerJob)> {
        self.expected
            .ordered()
            .flat_map(|(owner, entries)| entries.iter().map(move |entry| (*owner, entry.row)))
            .chain(self.apis.ordered().map(|(owner, row)| (*owner, *row)))
            .chain(self.headers.ordered().map(|(owner, row)| (*owner, *row)))
            .filter(|(owner, _)| {
                self.current_target.is_none_or(|target| {
                    self.workspace
                        .item(*owner)
                        .is_some_and(|item| item.target == target)
                })
            })
    }
}

fn configured(workspace: &Workspace, site: ItemId, job: &str) -> Result<(), Error> {
    let binding = workspace
        .item(site)
        .and_then(|item| workspace.target(item.target))
        .and_then(|target| {
            workspace
                .locked_binding(target.package_id)?
                .configuration_for(target.kind, &target.name)
        });
    if binding.is_none() {
        return Err(failure(
            job,
            "preferred provider requires grounded locked-source and delegation execution with configured owner and definition identities; use Gate::check_locked",
        ));
    }
    Ok(())
}

fn path_error(
    workspace: &Workspace,
    owner: ItemId,
    job: &str,
    path: &str,
    source: CodeResolveError,
) -> Error {
    let owner = workspace.item(owner).expect("checked provider owner");
    Error::ProviderPath {
        job: job.into(),
        file: owner.file.to_string(),
        line: owner.name_line,
        path: path.into(),
        source: Box::new(source),
    }
}

impl FunctionObserver for Providers<'_> {
    fn container(
        &mut self,
        file: &SourceFile,
        container: Container,
        attributes: &[syn::Attribute],
        limits: &mut MatchLimits,
    ) -> Result<(), Error> {
        if let Container::Declaration(byte) = container {
            for item in self.workspace.item_at(&file.path, byte) {
                if Some(item.target) != self.current_target {
                    continue;
                }
                if let Some(row) = self.headers.get(&item.id) {
                    charge(&row.job.id, limits, attributes.len() as u64 + 1)?;
                    flow::type_attributes(&row.job.id, attributes)?;
                    storage::insert(&mut self.checked_headers, item.id, (), &row.job.id)?;
                }
            }
            return Ok(());
        }
        if attributes.is_empty() {
            return Ok(());
        }
        for (owner, row) in self.owners() {
            charge(&row.job.id, limits, attributes.len() as u64 + 1)?;
            let item = self.workspace.item(owner).expect("checked owner");
            let applies = match &container {
                Container::File => true,
                Container::Item(span) => {
                    item.file == file.path && span.contains(&item.span.start_byte)
                }
                Container::Module(byte) => {
                    let mut applies = false;
                    for module in self
                        .workspace
                        .item_at(&file.path, *byte)
                        .iter()
                        .filter(|item| item.kind == SymbolKind::Mod)
                    {
                        charge(&row.job.id, limits, 1)?;
                        applies |= self.code.is_defined_in(owner, module.id).map_err(|error| {
                            path_error(self.workspace, owner, &row.job.id, &module.path, error)
                        })?;
                    }
                    applies
                }
                Container::Declaration(_) => {
                    unreachable!("declarations are audited by actual selected identity")
                }
            };
            if applies {
                if matches!(container, Container::Module(_)) {
                    flow::module_attributes(&row.job.id, attributes)?;
                } else {
                    flow::attributes(&row.job.id, attributes)?;
                }
            }
        }
        Ok(())
    }
    fn interested(&self, file: &SourceFile, signature: &syn::Signature) -> bool {
        if self.header_only {
            return false;
        }
        self.workspace
            .item_at(&file.path, signature.ident.span().byte_range().start)
            .iter()
            .any(|item| {
                Some(item.target) == self.current_target
                    && (self.expected.get(&item.id).is_some() || self.apis.get(&item.id).is_some())
            })
    }

    fn observe(
        &mut self,
        file: &SourceFile,
        scopes: &mut Scopes,
        signature: &syn::Signature,
        attributes: &[syn::Attribute],
        body: &syn::Block,
        limits: &mut MatchLimits,
    ) -> Result<(), Error> {
        for item in self
            .workspace
            .item_at(&file.path, signature.ident.span().byte_range().start)
        {
            if Some(item.target) != self.current_target {
                continue;
            }
            if let Some(row) = self.apis.get(&item.id) {
                flow::attributes(&row.job.id, attributes)?;
                storage::insert(&mut self.checked_apis, item.id, (), &row.job.id)?;
            }
            let Some(entries) = self.expected.get(&item.id) else {
                continue;
            };
            for expected in entries.iter() {
                flow::attributes(&expected.row.job.id, attributes)?;
                scopes.enter(false, std::iter::empty(), limits)?;
                let result = flow::check(
                    self.workspace,
                    self.code,
                    expected,
                    scopes,
                    signature,
                    body,
                    limits,
                );
                scopes.leave();
                for (alias, ()) in result?.ordered() {
                    if self.headers.get(alias).is_none() {
                        self.pending_ancestry = true;
                    }
                    if self.headers.get(alias).is_none() && self.headers.len() >= limits.states {
                        return Err(failure(
                            &expected.row.job.id,
                            "provider declaration-state limit exhausted",
                        ));
                    }
                    storage::insert(
                        &mut self.headers,
                        *alias,
                        expected.row,
                        &expected.row.job.id,
                    )?;
                }
            }
            storage::insert(&mut self.checked, item.id, (), "preferred-provider")?;
        }
        Ok(())
    }
}
