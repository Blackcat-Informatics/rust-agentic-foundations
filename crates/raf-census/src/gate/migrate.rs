// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One-way fingerprint migration through canonical report members and source sites.

mod legacy;
pub use legacy::{
    ForeignField, ForeignLedgerAdapter, migrate_legacy, migrate_legacy_with_extensions,
};

use std::collections::{BTreeMap, BTreeSet};

use raf_rdf::{Builder, Format, TermValue, Triple, vocab};
use raf_source::{LoadOptions, Tree, Workspace};

use super::owners;
use crate::{Census, Error, Fingerprint, Index, Ledger, MAX_LEDGER_BYTES, ReportError};

/// Maximum canonical index input, matching the source load's total-byte budget.
pub const MAX_INDEX_BYTES: usize = raf_source::Budget::TotalBytes.limit();

/// A named, borrowed migration input. Input bytes are never modified.
#[derive(Clone, Copy, Debug)]
pub struct MigrationInput<'a> {
    /// Path used in actionable parse and mapping refusals.
    pub path: &'a str,
    /// The complete original input bytes as UTF-8.
    pub text: &'a str,
}

/// One original fingerprint row that could not be mapped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnmappedRow {
    /// The original job, fingerprint or field location.
    pub location: String,
    /// The original value retained for review, even when it has no core field.
    pub original: String,
    /// Why the supplied index and current source cannot map it.
    pub reason: String,
}

/// A migrated ledger and every unmapped row. No filesystem writes are performed.
#[derive(Clone, Debug)]
pub struct Migration {
    /// Canonical RDF for mapped rows, with review comments for unmapped rows.
    /// Unmapped fingerprints remain asserted rather than silently disappearing.
    pub ledger: String,
    /// Original rows needing an explicit decision before the ledger is adopted.
    pub unmapped: Vec<UnmappedRow>,
}

/// Migrate a unified RDF ledger using exactly its older canonical RDF index.
///
/// Singleton fingerprints come from the index's units; a baseline is refused.
/// Current hashes are computed from the actual source sites under the ledger's
/// source limits. Body and shim rows may split; all other assertions survive.
///
/// # Errors
/// Refuses malformed or noncanonical inputs, unsupported report layout, a ledger
/// that is not older, mismatched versions, source failures and RDF construction
/// failures. Inputs remain read-only, and no partial file is written.
pub fn migrate_index(
    tree: &impl Tree,
    ledger: MigrationInput<'_>,
    index: MigrationInput<'_>,
) -> Result<Migration, Error> {
    if index.text.len() > MAX_INDEX_BYTES {
        return Err(input(
            index.path,
            Error::Options(format!("index exceeds {MAX_INDEX_BYTES} bytes")),
        ));
    }
    let (policy, old) =
        Ledger::import_index_ledger(ledger.text).map_err(|source| input(ledger.path, source))?;
    let previous = Index::from_turtle(index.text).map_err(|source| input(index.path, source))?;
    if previous.normal_form != old {
        return Err(input(
            index.path,
            Error::Options(format!(
                "index normal form {} does not match ledger normal form {old}",
                previous.normal_form
            )),
        ));
    }
    let mut load = LoadOptions::default();
    load.max_file_bytes = policy.options().max_file_bytes;
    let roots: Vec<_> = policy.roots().iter().map(String::as_str).collect();
    let workspace = Workspace::load_with(tree, &roots, &load)?;
    let census = Census::from_workspace(&workspace, policy.options())?;
    let owners = owners::index(&census);
    let historical = historical_members(&previous);
    let mut graph = Builder::from_graph(policy.graph());
    let mut unmapped = Vec::new();
    for (node, value) in policy.graph().nodes() {
        if !value
            .values(vocab::TYPE)
            .any(|value| value.iri() == Some(format!("{}LedgerJob", vocab::REPORT).as_str()))
        {
            continue;
        }
        let job = value
            .values(&format!("{}jobId", vocab::REPORT))
            .next()
            .and_then(raf_rdf::Value::literal)
            .ok_or_else(|| ReportError::Missing(format!("{node}: jobId")))?
            .0;
        let mut replacements = BTreeMap::new();
        for value in value.values(&format!("{}forbiddenFingerprint", vocab::REPORT)) {
            let original = value
                .literal()
                .ok_or_else(|| ReportError::Value(format!("{node}: fingerprint must be a string")))?
                .0;
            let fingerprint: Fingerprint = original
                .parse()
                .map_err(|error| ReportError::Value(format!("{node}: {error}")))?;
            if !matches!(fingerprint, Fingerprint::Body(_) | Fingerprint::Shim(_)) {
                continue;
            }
            match remap(&fingerprint, &historical, &workspace, &owners) {
                Ok(current) => {
                    replacements.insert(original.to_owned(), current);
                }
                Err(reason) => unmapped.push(UnmappedRow {
                    location: format!("job {job} fingerprint {fingerprint}"),
                    original: original.into(),
                    reason,
                }),
            }
        }
        replace(&mut graph, &node, &replacements)?;
    }
    finish(graph, unmapped, ledger.path)
}

fn finish(
    graph: Builder,
    unmapped: Vec<UnmappedRow>,
    ledger_path: &str,
) -> Result<Migration, Error> {
    let mut output = graph
        .finish()
        .map_err(ReportError::Ground)?
        .to_text(Format::Turtle)
        .map_err(ReportError::Ground)?;
    for row in &unmapped {
        output.push_str("# unmapped: ");
        output.push_str(&row.location.escape_debug().to_string());
        output.push_str(": ");
        output.push_str(&row.reason.escape_debug().to_string());
        output.push_str("; original: ");
        output.push_str(&row.original.escape_debug().to_string());
        output.push('\n');
    }
    if output.len() > MAX_LEDGER_BYTES {
        return Err(input(
            ledger_path,
            Error::Options(format!(
                "migrated ledger is {} bytes; maximum is {MAX_LEDGER_BYTES}",
                output.len()
            )),
        ));
    }
    if unmapped.is_empty() {
        Ledger::from_turtle(&output).map_err(|error| input(ledger_path, error))?;
    }
    Ok(Migration {
        ledger: output,
        unmapped,
    })
}

fn historical_members(index: &Index) -> BTreeMap<Fingerprint, BTreeSet<&str>> {
    let mut members: BTreeMap<Fingerprint, BTreeSet<&str>> = BTreeMap::new();
    for unit in &index.units {
        for fingerprint in [&unit.body, &unit.shim].into_iter().flatten() {
            members
                .entry(fingerprint.clone())
                .or_default()
                .insert(&unit.symbol);
        }
    }
    members
}

fn remap(
    fingerprint: &Fingerprint,
    historical: &BTreeMap<Fingerprint, BTreeSet<&str>>,
    workspace: &Workspace,
    owners: &owners::Owners<'_>,
) -> Result<BTreeSet<Fingerprint>, String> {
    let symbols = historical
        .get(fingerprint)
        .ok_or("index has no member symbol for the fingerprint")?;
    remap_members(
        matches!(fingerprint, Fingerprint::Shim(_)),
        symbols.iter().copied(),
        workspace,
        owners,
    )
}

fn remap_members<'a>(
    shim: bool,
    symbols: impl Iterator<Item = &'a str>,
    workspace: &Workspace,
    owners: &owners::Owners<'_>,
) -> Result<BTreeSet<Fingerprint>, String> {
    let mut current = BTreeSet::new();
    let mut resolved = false;
    let mut gaps = Vec::new();
    for symbol in symbols {
        let units = match owners::resolve(workspace, owners, symbol) {
            Ok(units) => units,
            Err(error) => {
                gaps.push(format!("member {symbol:?}: {error}"));
                continue;
            }
        };
        resolved = true;
        for unit in units {
            let value = if shim {
                unit.shim_key.as_ref()
            } else {
                unit.print.as_ref().and_then(|print| print.body.as_ref())
            };
            if let Some(value) = value {
                current.insert(value.clone());
            } else {
                gaps.push(format!(
                    "member {symbol:?} in {} at byte {} has no current fingerprint of this kind",
                    unit.file, unit.start_byte
                ));
            }
        }
    }
    if !gaps.is_empty() && resolved {
        Err(format!("incomplete member mapping: {}", gaps.join("; ")))
    } else if current.is_empty() {
        Err(if resolved {
            "resolved members have no current fingerprint of this kind"
        } else {
            "no member symbol resolves in the current source"
        }
        .into())
    } else {
        Ok(current)
    }
}

fn replace(
    graph: &mut Builder,
    node: &str,
    replacements: &BTreeMap<String, BTreeSet<Fingerprint>>,
) -> Result<(), Error> {
    let triple = |value: &str| Triple {
        subject: TermValue::iri(node),
        predicate: TermValue::iri(format!("{}forbiddenFingerprint", vocab::REPORT)),
        object: TermValue::simple_literal(value),
    };
    // Remove the old selection before inserting its union: a new hash can
    // equal another old row, which sequential replacement would then erase.
    for original in replacements.keys() {
        if !graph
            .remove(triple(original))
            .map_err(ReportError::Ground)?
        {
            return Err(ReportError::Missing(format!("{node}: fingerprint {original}")).into());
        }
    }
    for fingerprint in replacements.values().flatten() {
        let assertion = triple(&fingerprint.to_string());
        if !graph
            .contains(assertion.clone())
            .map_err(ReportError::Ground)?
        {
            graph.insert(assertion).map_err(ReportError::Ground)?;
        }
    }
    Ok(())
}

fn input(path: &str, source: Error) -> Error {
    Error::Input {
        file: path.into(),
        source: Box::new(source),
    }
}
