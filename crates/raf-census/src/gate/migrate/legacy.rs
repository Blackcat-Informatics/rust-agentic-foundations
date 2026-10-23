// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One-way foreign TOML import; the gate has no legacy reader.

mod baseline;
mod fields;
mod rows;

use raf_rdf::{Builder, TermValue as Term, Triple, vocab};
use raf_source::{LoadOptions, Tree, Workspace};
use toml::Value;

use super::{Migration, MigrationInput, finish, input};
use crate::{
    Census, CensusOptions, Error, MAX_LEDGER_BYTES, NORMAL_FORM, REPORT_FORM, ReportError,
};
use fields::Fields;

/// Import a foreign TOML ledger with its explicitly selected legacy JSON baseline.
///
/// The existing complete TOML parser reads the foreign authoring syntax; native
/// bounded JSON records read the baseline. Output uses the one RDF policy model.
/// Dialect metadata becomes RDF values in one extension that its consumer must
/// validate. Unknown fields and unmapped fingerprints remain visible for review.
///
/// # Errors
/// Refuses inputs above one MiB or 32 levels, malformed syntax, ambiguous field
/// aliases, invalid known fields, source failures or output above the ledger cap.
/// Neither input nor the source tree is written.
///
/// ```rust
/// use raf_census::{MigrationInput, migrate_legacy};
/// use raf_source::MemoryTree;
/// let tree = MemoryTree::from([
///     ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n"),
///     ("src/lib.rs", "/// Operation.\npub fn home() {}"),
/// ]);
/// let result = migrate_legacy(
///     &tree,
///     MigrationInput { path: "old.toml", text: "[[job]]\nid='operation'\nsummary='The operation'\nhome='demo::home'\nenforced=true\n" },
///     MigrationInput { path: "old.json", text: r#"{"min_tokens":20,"units":1,"body_groups":[],"shim_groups":[],"hex_tables":[],"shared_constants":[]}"# },
/// )?;
/// assert!(result.unmapped.is_empty());
/// # Ok::<(), raf_census::Error>(())
/// ```
pub fn migrate_legacy(
    tree: &impl Tree,
    ledger: MigrationInput<'_>,
    baseline: MigrationInput<'_>,
) -> Result<Migration, Error> {
    let result = || {
        if ledger.text.len() > MAX_LEDGER_BYTES {
            return Err(Error::Options(format!(
                "foreign ledger exceeds {MAX_LEDGER_BYTES} bytes"
            )));
        }
        let table: toml::Table = toml::from_str(ledger.text).map_err(Error::ImportToml)?;
        check_depth(&table)?;
        if !["census", "job", "distinct"]
            .iter()
            .any(|key| table.contains_key(*key))
        {
            return Err(Error::Options(
                "unrecognized foreign ledger layout; expected census, job or distinct tables"
                    .into(),
            ));
        }
        let historical = baseline::read(baseline)?;
        let mut source = Fields::new("ledger", table);
        let mut settings = source.table("census")?;
        if settings
            .integer("normal_form")?
            .is_some_and(|version| version >= u64::from(NORMAL_FORM))
        {
            return Err(Error::Options(
                "ledger is already at the implemented normal form; nothing to migrate".into(),
            ));
        }
        let min_tokens = settings
            .integer("min_tokens")?
            .unwrap_or(crate::DEFAULT_MIN_TOKENS as u64);
        let mut options = CensusOptions::default().min_tokens(
            usize::try_from(min_tokens)
                .map_err(|_| settings.bad("min_tokens", "value exceeds usize"))?,
        )?;
        options.max_file_bytes = settings
            .integer("max_file_bytes")?
            .unwrap_or(options.max_file_bytes);
        let mut roots = settings.strings("roots")?;
        if roots.is_empty() {
            roots.push(".".into());
        }
        let reference = settings.strings("reference")?;
        let mut load = LoadOptions::default();
        load.max_file_bytes = options.max_file_bytes;
        let borrowed: Vec<_> = roots.iter().map(String::as_str).collect();
        let workspace = Workspace::load_with(tree, &borrowed, &load)?;
        let census = Census::from_workspace(&workspace, &options)?;
        let owners = super::super::owners::index(&census);
        let mut graph = Builder::new();
        class(&mut graph, "", "Ledger")?;
        for (key, value) in [
            ("normalForm", u128::from(NORMAL_FORM)),
            ("reportForm", u128::from(REPORT_FORM)),
            ("minTokens", options.min_tokens as u128),
            ("maxFileBytes", u128::from(options.max_file_bytes)),
        ] {
            put(&mut graph, "", key, integer(value))?;
        }
        for (key, values) in [("root", roots), ("reference", reference)] {
            for value in values {
                put(&mut graph, "", key, Term::simple_literal(value))?;
            }
        }
        let mut unmapped = Vec::new();
        let mut ids = std::collections::BTreeSet::new();
        for row in source.rows("job")? {
            rows::job(
                &mut graph,
                row,
                &historical,
                &workspace,
                &owners,
                &mut ids,
                &mut unmapped,
            )?;
        }
        for (index, row) in source.rows("distinct")?.into_iter().enumerate() {
            rows::distinct(&mut graph, index, row, &mut unmapped)?;
        }
        settings.finish(&mut unmapped);
        source.finish(&mut unmapped);
        finish(graph, unmapped, ledger.path)
    };
    result().map_err(|error| input(ledger.path, error))
}

fn check_depth(table: &toml::Table) -> Result<(), Error> {
    let mut pending: purrdf_lex::walk::WorkList<_, 32> = purrdf_lex::walk::WorkList::new();
    for value in table.values() {
        pending
            .try_push((value, 1usize))
            .map_err(Error::ImportStorage)?;
    }
    while let Some((value, depth)) = pending.pop() {
        if matches!(value, Value::Array(_) | Value::Table(_)) && depth >= 32 {
            return Err(Error::Options(
                "foreign ledger exceeds 32 levels of nesting".into(),
            ));
        }
        match value {
            Value::Array(values) => {
                for value in values {
                    pending
                        .try_push((value, depth + 1))
                        .map_err(Error::ImportStorage)?;
                }
            }
            Value::Table(values) => {
                for value in values.values() {
                    pending
                        .try_push((value, depth + 1))
                        .map_err(Error::ImportStorage)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn put(graph: &mut Builder, node: &str, key: &str, value: Term) -> Result<(), Error> {
    let predicate = if key == "type" {
        vocab::TYPE.into()
    } else {
        format!("{}{key}", vocab::REPORT)
    };
    let triple = Triple {
        subject: Term::iri(node),
        predicate: Term::iri(predicate),
        object: value,
    };
    if !graph
        .contains(triple.clone())
        .map_err(ReportError::Ground)?
    {
        graph.insert(triple).map_err(ReportError::Ground)?;
    }
    Ok(())
}

fn class(graph: &mut Builder, node: &str, name: &str) -> Result<(), Error> {
    put(
        graph,
        node,
        "type",
        Term::iri(format!("{}{name}", vocab::REPORT)),
    )
}
fn integer(value: u128) -> Term {
    Term::typed_literal(
        purrdf_xsd::exact::Integer::from(value).canonical_lexical(),
        vocab::INTEGER,
    )
}
fn anchor(value: &str) -> Term {
    Term::iri(if value.starts_with("urn:") || value.contains("://") {
        value.into()
    } else {
        format!(
            "{}{}",
            crate::SYMBOL_ANCHOR_PREFIX,
            crate::report::mint::part(value)
        )
    })
}
