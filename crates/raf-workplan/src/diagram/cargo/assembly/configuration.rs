// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Observed compiler predicates remain distinct from requested metadata features.

use super::{DiagramError, MAX_ITEMS, refused};
use crate::exec::Output;
use raf_source::{CfgAtom, DeclaredCfg, DeclaredConfiguration, TargetKind, syn};

pub(super) struct Configuration {
    triple: String,
    atoms: Vec<(String, Option<String>)>,
    features: Vec<String>,
}

impl Configuration {
    pub(super) fn read(
        cfg: &Output,
        version: &Output,
        target: Option<&str>,
    ) -> Result<Self, DiagramError> {
        if !cfg.success() || !version.success() {
            return Err(refused(
                "assembly requires successful actual compiler information queries",
            ));
        }
        let text = std::str::from_utf8(&version.stdout).map_err(DiagramError::data)?;
        let mut hosts = text.lines().filter_map(|line| line.strip_prefix("host: "));
        let host = hosts
            .next()
            .filter(|host| !host.is_empty())
            .ok_or_else(|| refused("compiler verbose version has no host identity"))?;
        if hosts.next().is_some() {
            return Err(refused(
                "compiler verbose version repeats its host identity",
            ));
        }
        let triple = target.unwrap_or(host);
        if std::path::Path::new(triple)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            || triple.contains('/')
            || triple.contains('\\')
            || triple.chars().any(char::is_whitespace)
        {
            return Err(refused(
                "custom target specifications require separately observed compiler input evidence",
            ));
        }
        let mut atoms = Vec::new();
        let mut features = Vec::new();
        let mut observed = std::collections::BTreeSet::new();
        for line in std::str::from_utf8(&cfg.stdout)
            .map_err(DiagramError::data)?
            .lines()
        {
            if observed.len() == MAX_ITEMS || !observed.insert(line) {
                return Err(refused(
                    "compiler cfg predicates are repeated or exceed their bound",
                ));
            }
            let meta: syn::Meta = syn::parse_str(line).map_err(DiagramError::data)?;
            let (key, value) = match meta {
                syn::Meta::Path(path) => (
                    path.get_ident()
                        .ok_or_else(|| refused("compiler cfg flag is not one identifier"))?
                        .to_string(),
                    None,
                ),
                syn::Meta::NameValue(pair) => {
                    let key = pair
                        .path
                        .get_ident()
                        .ok_or_else(|| refused("compiler cfg key is not one identifier"))?
                        .to_string();
                    let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(value),
                        ..
                    }) = pair.value
                    else {
                        return Err(refused("compiler cfg values must be native Rust strings"));
                    };
                    (key, Some(value.value()))
                }
                syn::Meta::List(_) => {
                    return Err(refused(
                        "compiler cfg output contains a predicate expression",
                    ));
                }
            };
            if key == "feature" {
                features.push(
                    value.ok_or_else(|| refused("compiler feature cfg requires a string value"))?,
                );
            } else {
                atoms.push((key, value));
            }
        }
        if !atoms
            .iter()
            .any(|(key, value)| key == "doc" && value.is_none())
        {
            return Err(refused("compiler cfg has no actual doc predicate"));
        }
        Ok(Self {
            triple: triple.into(),
            atoms,
            features,
        })
    }

    pub(super) fn with<R>(
        &self,
        name: &str,
        use_selection: impl FnOnce(DeclaredConfiguration<'_>) -> R,
    ) -> R {
        let atoms: Vec<_> = self
            .atoms
            .iter()
            .map(|(name, value)| {
                value
                    .as_ref()
                    .map_or(CfgAtom::Name(name), |value| CfgAtom::Value(name, value))
            })
            .collect();
        let features: Vec<_> = self.features.iter().map(String::as_str).collect();
        use_selection(DeclaredConfiguration::new(
            DeclaredCfg::complete(&self.triple, &atoms),
            &features,
            TargetKind::Lib,
            name,
        ))
    }
}
