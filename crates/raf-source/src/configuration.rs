// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Declared compilation inputs, checked against actual package declarations.

mod features;
mod syntax;
use crate::cfg::CfgExpr;
use crate::meter::{ItemAdmission, Meter};
use crate::{DependencyDeclaration, LoadError, LoadErrorKind, RelPath, TargetKind};
pub(crate) use features::destination;
use purrdf_lex::allocation::Memory;
use purrdf_lex::walk::{WorkList, try_equal_range_by, try_sort_unstable_by};
use std::collections::BTreeMap;
use syn::ext::IdentExt as _;
use syn::parse::Parser as _;
pub(crate) use syntax::filter;
pub(crate) use syntax::predicate;

/// One explicitly enabled target cfg predicate; absence in a complete set is false.
#[derive(Debug, Clone, Copy)]
pub enum CfgAtom<'a> {
    /// A bare flag such as unix or test.
    Name(&'a str),
    /// A key/value predicate such as `target_os = "linux"`.
    Value(&'a str, &'a str),
}
/// A caller-declared complete target triple and cfg set, never inferred from this host.
///
/// The caller supplies package-specific flags, including build-script cfg. This
/// is not observed Cargo state: agreeing triples do not authenticate predicates
/// or require different packages' custom cfg sets to be identical.
#[derive(Debug, Clone, Copy)]
pub struct DeclaredCfg<'a> {
    triple: &'a str,
    atoms: &'a [CfgAtom<'a>],
}
impl<'a> DeclaredCfg<'a> {
    /// Declare completeness: predicates not listed are false. Features are separate.
    #[must_use]
    pub const fn complete(triple: &'a str, atoms: &'a [CfgAtom<'a>]) -> Self {
        Self { triple, atoms }
    }
}
/// One selected compilation target and its complete enabled package features.
///
/// This is declared build input, not observation of Cargo or this machine.
/// The caller supplies the complete already-unified feature set. The load checks
/// local manifest implications and requested destinations; it does not run Cargo's
/// resolver or infer omitted feature activation from an ambient build.
#[derive(Debug, Clone, Copy)]
pub struct DeclaredConfiguration<'a> {
    target: DeclaredCfg<'a>,
    host: Option<DeclaredCfg<'a>>,
    features: &'a [&'a str],
    kind: TargetKind,
    name: &'a str,
}
impl<'a> DeclaredConfiguration<'a> {
    /// Select one exact actual target and a complete, already unified feature set.
    #[must_use]
    pub const fn new(
        target: DeclaredCfg<'a>,
        features: &'a [&'a str],
        kind: TargetKind,
        name: &'a str,
    ) -> Self {
        Self {
            target,
            host: None,
            features,
            kind,
            name,
        }
    }
    /// Supply the independent host cfg required by build-script contexts.
    #[must_use]
    pub const fn with_host(mut self, host: DeclaredCfg<'a>) -> Self {
        self.host = Some(host);
        self
    }
}
/// Retained complete declared cfg, admitted under the load's original grant.
#[derive(Debug, PartialEq, Eq)]
pub struct CfgSelection {
    triple: String,
    predicates: Vec<String>,
}
impl CfgSelection {
    /// The explicit target triple, without host inference.
    #[must_use]
    pub fn triple(&self) -> &str {
        &self.triple
    }
    /// Canonical enabled cfg predicates, excluding separately retained features.
    #[must_use]
    pub fn predicates(&self) -> &[String] {
        &self.predicates
    }
}
/// A package's validated declared compilation context in a filtered index.
///
/// Only a completed configured load publishes this receipt. It is not a Cargo
/// invocation receipt, macro expansion or preferred-provider policy decision.
/// Selection covers modeled item/module/import and associated declaration
/// headers. Function-body statements/expressions, fields and enum variants are
/// not cfg-transformed, and original source bytes remain unchanged.
#[derive(Debug, PartialEq, Eq)]
pub struct ActiveConfiguration {
    target: CfgSelection,
    host: Option<CfgSelection>,
    pub(crate) features: Vec<String>,
    pub(crate) optional: Vec<String>,
    kind: TargetKind,
    name: String,
}
impl ActiveConfiguration {
    /// The declared target cfg.
    #[must_use]
    pub const fn target(&self) -> &CfgSelection {
        &self.target
    }
    /// The independent host cfg, when supplied.
    #[must_use]
    pub const fn host(&self) -> Option<&CfgSelection> {
        self.host.as_ref()
    }
    /// Complete enabled features, validated against actual manifest implications.
    #[must_use]
    pub fn features(&self) -> &[String] {
        &self.features
    }
    /// Optional dependency aliases activated by the actual feature graph.
    #[must_use]
    pub fn optional_dependencies(&self) -> &[String] {
        &self.optional
    }
    /// The single actual compilation target kind selected for this package.
    #[must_use]
    pub const fn target_kind(&self) -> TargetKind {
        self.kind
    }
    /// The exact selected Cargo target name.
    #[must_use]
    pub fn target_name(&self) -> &str {
        &self.name
    }
    pub(crate) fn selected(&self) -> Option<&CfgSelection> {
        if self.kind == TargetKind::BuildScript {
            self.host.as_ref()
        } else {
            Some(&self.target)
        }
    }
    pub(crate) fn same_features_under(
        &self,
        other: &Self,
        meter: &Meter,
        path: &RelPath,
    ) -> Result<bool, LoadError> {
        comparison_work(&self.features, meter, path)?;
        comparison_work(&other.features, meter, path)?;
        Ok(self.features == other.features)
    }
    pub(crate) fn same_syntax_under(
        &self,
        other: &Self,
        meter: &Meter,
        path: &RelPath,
    ) -> Result<bool, LoadError> {
        for cfg in [self.selected(), other.selected()].into_iter().flatten() {
            meter.work(cfg.triple.len().saturating_add(1), path, None)?;
            comparison_work(&cfg.predicates, meter, path)?;
        }
        let features_agree = self.same_features_under(other, meter, path)?;
        Ok(self.selected() == other.selected() && features_agree)
    }
}

fn comparison_work(values: &[String], meter: &Meter, path: &RelPath) -> Result<(), LoadError> {
    meter.work(values.len(), path, None)?;
    for value in values {
        meter.work(value.len().saturating_add(1), path, None)?;
    }
    Ok(())
}
pub(crate) fn admit(
    input: DeclaredConfiguration<'_>,
    graph: &BTreeMap<String, Vec<String>>,
    deps: &[DependencyDeclaration],
    meter: &Meter,
    path: &RelPath,
) -> Result<ActiveConfiguration, LoadError> {
    let before = meter.usage();
    let options = meter.options().clone();
    let ((result, after), _) =
        crate::parse::parse_counted(path, "", options.max_file_tokens, |_tree| {
            let local = Meter::resume(&options, before);
            let result = admit_inner(input, graph, deps, &local, path);
            (result, local.usage())
        })?;
    meter.configuration_spent(before, after, path)?;
    result
}
fn admit_inner(
    input: DeclaredConfiguration<'_>,
    graph: &BTreeMap<String, Vec<String>>,
    deps: &[DependencyDeclaration],
    meter: &Meter,
    path: &RelPath,
) -> Result<ActiveConfiguration, LoadError> {
    let mut admission = ItemAdmission::new(meter, path);
    let mut memory = Memory::new(&mut admission);
    let result = (|| {
        let target = cfg_set(input.target, meter, path, &mut memory)?;
        let host = input
            .host
            .map(|cfg| cfg_set(cfg, meter, path, &mut memory))
            .transpose()?;
        if input.kind == TargetKind::BuildScript && host.is_none() {
            return Err(failure(
                path,
                "a build-script context needs an explicit host cfg",
            ));
        }
        let name = memory.string(input.name).map_err(native)?;
        let (features, optional) =
            features::admit(input.features, graph, deps, meter, path, &mut memory)?;
        Ok(ActiveConfiguration {
            target,
            host,
            features,
            optional,
            kind: input.kind,
            name,
        })
    })();
    result.map_err(|error| recover(error, &mut admission))
}
fn cfg_set<S: purrdf_lex::allocation::Admission + ?Sized>(
    input: DeclaredCfg<'_>,
    meter: &Meter,
    path: &RelPath,
    memory: &mut Memory<'_, S>,
) -> Result<CfgSelection, LoadError> {
    if input.triple.is_empty() || input.triple.chars().any(char::is_whitespace) {
        return Err(failure(path, "supply an explicit nonempty target triple"));
    }
    let triple = memory.string(input.triple).map_err(native)?;
    let mut predicates = Vec::new();
    for atom in input.atoms {
        let (key, value) = match atom {
            CfgAtom::Name(key) => (*key, None),
            CfgAtom::Value(key, value) => (*key, Some(*value)),
        };
        meter.work(
            key.len()
                .saturating_add(value.map_or(0, str::len))
                .saturating_add(1),
            path,
            None,
        )?;
        if u64::try_from(key.len()).unwrap_or(u64::MAX) > meter.options().max_file_bytes {
            return Err(failure(
                path,
                "cfg name exceeds the original file-byte bound",
            ));
        }
        let identifier = syn::Ident::parse_any
            .parse_str(key)
            .map_err(|_| failure(path, "cfg names must be Rust identifiers"))?;
        let key = memory
            .format(&format_args!("{}", identifier.unraw()))
            .map_err(native)?;
        if key == "feature" {
            return Err(failure(
                path,
                "cfg names must be Rust identifiers; feature values belong in the package feature selection",
            ));
        }
        let value = match value {
            None => memory.string(&key),
            Some(value) => memory.format(&format_args!("{key} = {value:?}")),
        }
        .map_err(native)?;
        memory.push(&mut predicates, value).map_err(native)?;
    }
    sort_unique(&mut predicates, meter, path)?;
    Ok(CfgSelection { triple, predicates })
}
pub(crate) fn sort_unique(
    values: &mut [String],
    meter: &Meter,
    path: &RelPath,
) -> Result<(), LoadError> {
    try_sort_unstable_by(values, |a, b| {
        meter.work(
            a.len().saturating_add(b.len()).saturating_add(1),
            path,
            None,
        )?;
        Ok::<_, LoadError>(a.cmp(b))
    })?;
    for pair in values.windows(2) {
        meter.work(1, path, None)?;
        if pair[0] == pair[1] {
            return Err(failure(path, "duplicate declared configuration value"));
        }
    }
    Ok(())
}
pub(crate) fn contains(
    values: &[String],
    value: &str,
    meter: &Meter,
    path: &RelPath,
) -> Result<bool, LoadError> {
    let found = try_equal_range_by(values, |known| {
        meter.work(
            known.len().saturating_add(value.len()).saturating_add(1),
            path,
            None,
        )?;
        Ok::<_, LoadError>(known.as_str().cmp(value))
    })?;
    Ok(!found.is_empty())
}
pub(crate) fn evaluate(
    expr: &CfgExpr,
    cfg: &ActiveConfiguration,
    dependency: bool,
    meter: &Meter,
    path: &RelPath,
) -> Result<bool, LoadError> {
    #[derive(Clone, Copy)]
    enum Frame<'a> {
        Node(&'a CfgExpr),
        Not,
        Fold(bool, usize),
    }
    let selected = cfg
        .selected()
        .ok_or_else(|| failure(path, "configured target lost its required host cfg"))?;
    let mut admission = ItemAdmission::new(meter, path);
    let mut memory = Memory::new(&mut admission);
    let result = (|| {
        let mut pending = WorkList::<_, 16>::new();
        let mut values = WorkList::<bool, 16>::new();
        pending
            .try_push_admitted(Frame::Node(expr), &mut memory)
            .map_err(native)?;
        while let Some(frame) = pending.pop() {
            meter.work(1, path, None)?;
            let value = match frame {
                Frame::Node(CfgExpr::Boolean(value)) => Some(*value),
                Frame::Node(CfgExpr::Test) => {
                    if dependency {
                        return Err(failure(
                            path,
                            "Cargo target dependency predicates cannot select test",
                        ));
                    }
                    Some(contains(&selected.predicates, "test", meter, path)?)
                }
                Frame::Node(CfgExpr::Other(text)) => Some(atom(
                    text,
                    cfg,
                    selected,
                    dependency,
                    meter,
                    path,
                    &mut memory,
                )?),
                Frame::Node(CfgExpr::Not(inner)) => {
                    pending
                        .try_push_admitted(Frame::Not, &mut memory)
                        .map_err(native)?;
                    pending
                        .try_push_admitted(Frame::Node(inner), &mut memory)
                        .map_err(native)?;
                    None
                }
                Frame::Node(CfgExpr::All(children) | CfgExpr::Any(children)) => {
                    let all = matches!(frame, Frame::Node(CfgExpr::All(_)));
                    pending
                        .try_push_admitted(Frame::Fold(all, children.len()), &mut memory)
                        .map_err(native)?;
                    for child in children.iter().rev() {
                        pending
                            .try_push_admitted(Frame::Node(child), &mut memory)
                            .map_err(native)?;
                    }
                    None
                }
                Frame::Not => Some(
                    !values
                        .pop()
                        .ok_or_else(|| failure(path, "cfg evaluation lost an operand"))?,
                ),
                Frame::Fold(all, count) => {
                    let mut result = all;
                    for _ in 0..count {
                        meter.work(1, path, None)?;
                        let value = values
                            .pop()
                            .ok_or_else(|| failure(path, "cfg evaluation lost an operand"))?;
                        if all {
                            result &= value;
                        } else {
                            result |= value;
                        }
                    }
                    Some(result)
                }
            };
            if let Some(value) = value {
                values
                    .try_push_admitted(value, &mut memory)
                    .map_err(native)?;
            }
        }
        let value = values
            .pop()
            .ok_or_else(|| failure(path, "cfg evaluation produced no decision"))?;
        if !values.is_empty() {
            return Err(failure(path, "cfg evaluation left extra decisions"));
        }
        Ok(value)
    })();
    result.map_err(|error| recover(error, &mut admission))
}
fn atom<S: purrdf_lex::allocation::Admission + ?Sized>(
    text: &str,
    cfg: &ActiveConfiguration,
    selected: &CfgSelection,
    dependency: bool,
    meter: &Meter,
    path: &RelPath,
    memory: &mut Memory<'_, S>,
) -> Result<bool, LoadError> {
    let key = text.split_once(" = ").map_or(text, |(key, _)| key);
    if key.contains("::") {
        return Err(failure(
            path,
            "qualified cfg predicate names are unsupported",
        ));
    }
    if dependency && ["test", "feature", "debug_assertions", "proc_macro"].contains(&key) {
        return Err(failure(
            path,
            "Cargo target dependency predicates cannot select test, feature, debug_assertions or proc_macro",
        ));
    }
    let mut found = contains(&selected.predicates, text, meter, path)?;
    for feature in &cfg.features {
        let predicate = memory
            .format(&format_args!("feature = {feature:?}"))
            .map_err(native)?;
        meter.work(
            predicate.len().saturating_add(text.len()).saturating_add(1),
            path,
            None,
        )?;
        found |= predicate == text;
    }
    Ok(found)
}
pub(crate) fn failure(path: &RelPath, message: &str) -> LoadError {
    LoadError::new(
        path,
        None,
        LoadErrorKind::Locked {
            field: "configuration",
            message: message.to_owned(),
            source: None,
        },
    )
}
pub(crate) fn condition(
    condition: &str,
    cfg: &ActiveConfiguration,
    meter: &Meter,
    path: &RelPath,
) -> Result<bool, LoadError> {
    let selected = cfg
        .selected()
        .ok_or_else(|| failure(path, "target condition needs an explicit host cfg"))?;
    meter.work(
        condition
            .len()
            .saturating_add(selected.triple.len())
            .saturating_add(1),
        path,
        None,
    )?;
    if !condition.trim_start().starts_with("cfg(") {
        return Ok(condition == selected.triple);
    }
    let mut admission = ItemAdmission::new(meter, path);
    let text = Memory::new(&mut admission)
        .format(&format_args!("#![{condition}]\n"))
        .map_err(|source| admission.error(source))?;
    if u64::try_from(text.len()).unwrap_or(u64::MAX) > meter.options().max_file_bytes {
        return Err(failure(
            path,
            "target predicate exceeds the original file-byte bound",
        ));
    }
    let before = meter.usage();
    let options = meter.options().clone();
    let ((result, after), scan) =
        crate::parse::parse_counted(path, &text, options.max_file_tokens, |tree| {
            let local = Meter::resume(&options, before);
            let result = (|| {
                if tree.attrs.len() != 1
                    || !tree.items.is_empty()
                    || !tree.attrs[0].path().is_ident("cfg")
                {
                    return Err(failure(
                        path,
                        "target table must contain exactly one cfg predicate",
                    ));
                }
                let meta = tree.attrs[0]
                    .parse_args_with(crate::cfg::parse_meta)
                    .map_err(|error| failure(path, &error.to_string()))?;
                predicate(&meta, cfg, true, &local, path)
            })();
            (result, local.usage())
        })?;
    meter.configuration_spent(before, after, path)?;
    meter.scanned(scan);
    result
}
fn native(source: purrdf_lex::allocation::StorageError) -> LoadError {
    LoadError::new(&RelPath::root(), None, LoadErrorKind::Storage(source))
}
fn recover(error: LoadError, admission: &mut ItemAdmission<'_>) -> LoadError {
    match error.kind {
        LoadErrorKind::Storage(source) => admission.error(source),
        _ => error,
    }
}
