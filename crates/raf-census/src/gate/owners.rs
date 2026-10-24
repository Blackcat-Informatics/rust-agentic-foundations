// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Map every source reach to the one physical census owner.

use std::collections::{BTreeMap, BTreeSet};

use raf_source::{RelPath, SymbolKind, Workspace};

use crate::{Census, Unit};

pub(super) type Owners<'a> = BTreeMap<(RelPath, usize), &'a Unit>;

pub(super) fn index(census: &Census) -> Owners<'_> {
    census
        .units()
        .iter()
        .map(|unit| ((unit.source_file.clone(), unit.start_byte), unit))
        .collect()
}

pub(super) fn resolve<'a>(
    workspace: &Workspace,
    owners: &Owners<'a>,
    symbol: &str,
) -> Result<Vec<&'a Unit>, String> {
    let resolution = workspace
        .resolve(symbol)
        .map_err(|error| error.to_string())?;
    let mut units = BTreeMap::new();
    for site in resolution.sites {
        let item = workspace
            .item(site)
            .ok_or_else(|| format!("source site {site:?} is absent"))?;
        if workspace.same_source(item).is_empty() {
            return Err(format!(
                "{symbol:?} does not name a census implementation unit"
            ));
        }
        let key = (item.file.clone(), item.span.start_byte);
        let unit = owners.get(&key).ok_or_else(|| {
            format!(
                "{symbol:?} does not name a census unit at {}:{}",
                item.file, item.span.start_byte
            )
        })?;
        units.insert(key, *unit);
    }
    if units.is_empty() {
        return Err(format!("{symbol:?} has no census source sites"));
    }
    Ok(units.into_values().collect())
}

pub(super) fn key(unit: &Unit) -> (String, String, usize) {
    (unit.symbol.clone(), unit.file.clone(), unit.start_byte)
}

pub(super) struct Home<'a> {
    pub(super) source_items: BTreeSet<raf_source::ItemId>,
    pub(super) path: String,
    pub(super) packages: BTreeSet<String>,
    pub(super) units: Vec<&'a Unit>,
}

/// A home may be an implementation item or a module containing implementations.
/// Distinct members and variants continue to require one actual unit path.
pub(super) fn home<'a>(
    workspace: &Workspace,
    code: &raf_source::CodeResolver<'_>,
    owners: &Owners<'a>,
    symbol: &str,
) -> Result<Home<'a>, String> {
    let resolution = workspace
        .resolve(symbol)
        .map_err(|error| error.to_string())?;
    let mut packages = BTreeSet::new();
    let mut units = BTreeMap::new();
    let mut module = false;
    let mut source_items = BTreeSet::new();
    for site in &resolution.sites {
        let item = workspace
            .item(*site)
            .ok_or_else(|| format!("source site {site:?} is absent"))?;
        let target = workspace
            .target(item.target)
            .ok_or_else(|| format!("{symbol:?} has no owning source target"))?;
        packages.insert(target.package.clone());
        source_items.insert(*site);
        if item.kind == SymbolKind::Mod {
            module = true;
            for child in workspace.items() {
                if child.target != item.target {
                    continue;
                }
                if !code
                    .is_defined_in(child.id, *site)
                    .map_err(|error| error.to_string())?
                {
                    continue;
                }
                source_items.insert(child.id);
                if workspace.same_source(child).is_empty() {
                    continue;
                }
                if let Some(unit) = owners.get(&(child.file.clone(), child.span.start_byte)) {
                    units.insert(key(unit), *unit);
                }
            }
        } else {
            let unit = owners
                .get(&(item.file.clone(), item.span.start_byte))
                .ok_or_else(|| {
                    format!("{symbol:?} does not name an implementation unit or module")
                })?;
            units.insert(key(unit), *unit);
        }
    }
    if packages.is_empty() {
        return Err(format!("{symbol:?} has no source definition"));
    }
    let path = if module {
        resolution.path
    } else {
        units.values().next().expect("resolved unit").symbol.clone()
    };
    Ok(Home {
        source_items,
        path,
        packages,
        units: units.into_values().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CensusOptions;
    use raf_source::{LoadOptions, MemoryTree, SourceAuthorityId, SourceTrees, Tree};

    #[test]
    fn equal_display_paths_keep_distinct_authorities_and_consumer_scope() {
        let first = MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='consumer'\nversion='1.0.0'\nedition='2024'\n",
            ),
            ("src/lib.rs", "pub fn left() -> u32 { 1 }"),
        ]);
        let second = MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='provider'\nversion='1.0.0'\nedition='2024'\n",
            ),
            ("src/lib.rs", "pub fn right() -> u32 { 2 }"),
        ]);
        let external: [&dyn Tree; 1] = [&second];
        let sources = SourceTrees::new(&first, &external);
        let roots: Vec<_> = sources
            .authorities()
            .map(|id| sources.root(id).unwrap())
            .collect();
        let workspace = Workspace::load_from(&sources, &roots, &LoadOptions::default()).unwrap();
        let all = Census::from_workspace(&workspace, &CensusOptions::default()).unwrap();
        let owners = index(&all);
        assert_eq!(owners.len(), 2);
        assert_eq!(
            resolve(&workspace, &owners, "consumer::left").unwrap()[0].name,
            "left"
        );
        assert_eq!(
            resolve(&workspace, &owners, "provider::right").unwrap()[0].name,
            "right"
        );
        let consumers = workspace
            .package_ids()
            .filter(|id| {
                workspace.package(*id).unwrap().manifest.authority()
                    == SourceAuthorityId::FIRST_PARTY
            })
            .collect();
        let selected =
            Census::from_packages(&workspace, &CensusOptions::default(), &consumers).unwrap();
        assert_eq!(selected.units().len(), 1);
        assert_eq!(selected.units()[0].name, "left");
        let provider = workspace
            .item(workspace.resolve("provider::right").unwrap().sites[0])
            .unwrap();
        assert!(selected.source_context(&provider.file).is_none());
        assert!(resolve(&workspace, &index(&selected), "provider::right").is_err());
        assert_eq!(selected.files().len(), 1);
    }

    fn pattern_ledger() -> crate::Ledger {
        use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};

        let mut graph = Builder::new();
        let mut put = |node: &str, key: &str, object: Term| {
            graph
                .insert(Triple {
                    subject: Term::iri(node),
                    predicate: Term::iri(if key == "type" {
                        vocab::TYPE.into()
                    } else {
                        format!("{}{key}", vocab::REPORT)
                    }),
                    object,
                })
                .unwrap();
        };
        put("", "type", Term::iri(format!("{}Ledger", vocab::REPORT)));
        put(
            "",
            "normalForm",
            Term::typed_literal(crate::NORMAL_FORM.to_string(), vocab::INTEGER),
        );
        put(
            "",
            "reportForm",
            Term::typed_literal(crate::REPORT_FORM.to_string(), vocab::INTEGER),
        );
        put("", "job", Term::iri("#job"));
        put(
            "#job",
            "type",
            Term::iri(format!("{}LedgerJob", vocab::REPORT)),
        );
        for (key, value) in [
            ("jobId", "selected"),
            ("summary", "Selected syntax"),
            ("homeSymbol", "consumer::left"),
        ] {
            put("#job", key, Term::simple_literal(value));
        }
        put("#job", "enforced", Term::boolean(true));
        for (key, matcher) in [
            ("call", "std::mem::drop"),
            ("path", "std::string::String"),
            ("typePath", "std::string::String"),
            ("macro", "todo"),
            ("tokens", "3 + 4"),
            ("literalRegex", "^blocked$"),
        ] {
            let node = format!("#{key}");
            put("", "pattern", Term::iri(&node));
            put(
                &node,
                "type",
                Term::iri(format!("{}Pattern", vocab::REPORT)),
            );
            put(&node, "patternId", Term::simple_literal(key));
            put(&node, "summary", Term::simple_literal("Selected syntax"));
            put(&node, key, Term::simple_literal(matcher));
            put("#job", "forbiddenPattern", Term::simple_literal(key));
        }
        crate::Ledger::from_turtle(&graph.finish().unwrap().to_text(Format::Turtle).unwrap())
            .unwrap()
    }

    #[test]
    fn every_pattern_kind_uses_the_selected_consumer_file_set() {
        let source = |name: &str| {
            format!(
                "#[doc=\"blocked\"] pub fn {name}() {{ std::mem::drop(3 + 4); let _: std::string::String; todo!(); }}"
            )
        };
        let left = source("left");
        let right = source("right");
        let first = MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='consumer'\nversion='1.0.0'\nedition='2024'\n",
            ),
            ("src/lib.rs", left.as_str()),
        ]);
        let second = MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='provider'\nversion='1.0.0'\nedition='2024'\n",
            ),
            ("src/lib.rs", right.as_str()),
        ]);
        let external: [&dyn Tree; 1] = [&second];
        let trees = SourceTrees::new(&first, &external);
        let roots: Vec<_> = trees
            .authorities()
            .map(|id| trees.root(id).unwrap())
            .collect();
        let workspace = Workspace::load_from(&trees, &roots, &LoadOptions::default()).unwrap();
        let ledger = pattern_ledger();
        for (selected, copies) in [(true, 0), (false, 1)] {
            let packages = workspace
                .package_ids()
                .filter(|id| {
                    !selected
                        || workspace.package(*id).unwrap().manifest.authority()
                            == SourceAuthorityId::FIRST_PARTY
                })
                .collect();
            let census =
                Census::from_packages(&workspace, &CensusOptions::default(), &packages).unwrap();
            let report = super::super::evaluate::evaluate(
                &ledger,
                &workspace,
                census,
                &[],
                &[],
                purrdf_jsonschema::ecma::MatchLimits::default(),
            )
            .unwrap();
            assert_eq!(report.jobs()[0].copies, copies, "{:?}", report.findings());
            assert_eq!(report.is_clean(), selected, "{:?}", report.findings());
        }
    }
}
