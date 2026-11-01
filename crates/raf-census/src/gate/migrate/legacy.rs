// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One-way foreign TOML import; the gate has no legacy reader.

mod baseline;
mod extensions;
mod fields;
mod rows;

pub use extensions::{ForeignField, ForeignLedgerAdapter};

use purrdf_toml::{Bounds, Document, Float, Kind, Profile, Scalar, SourceDocument, Vocabulary};
use raf_rdf::{Builder, TermValue as Term, Triple, vocab};
use raf_source::{LoadOptions, Tree, Workspace};

use super::{Migration, MigrationInput, finish, input};
use crate::{
    Census, CensusOptions, Error, MAX_LEDGER_BYTES, NORMAL_FORM, REPORT_FORM, ReportError,
};
use fields::Fields;

/// Import a foreign TOML ledger with its explicitly selected legacy JSON baseline.
///
/// The supplied native TOML arena reads the foreign authoring syntax; native
/// bounded JSON records read the baseline. Output uses the one RDF policy model.
/// Dialect metadata becomes RDF values in one extension that its consumer must
/// validate. Unknown fields and unmapped fingerprints remain visible for review.
///
/// # Errors
/// Refuses ledgers above one MiB, baselines above [`crate::MAX_INDEX_BYTES`],
/// containers beyond the admitted depth, malformed syntax, ambiguous field aliases,
/// invalid known fields, source failures or output above the ledger cap.
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
    migrate_legacy_with_extensions(tree, ledger, baseline, &[])
}

/// Import foreign policy with consumer claims over whole immediate job fields.
///
/// Every claimed field's complete native Value subtree is retained by the
/// framework. The adapter supplies a predicate and source-schema acceptance;
/// the registered native extension validator still owns evidence validity.
/// Unclaimed fields retain the same explicit unmapped accounting as
/// `migrate_legacy`. Inputs and debt are never written.
///
/// # Errors
/// Refuses the same input and output boundaries as `migrate_legacy`, competing or
/// consumed-field claims, adapter refusals, invalid predicates and incomplete
/// native transport. Mapping failures retain the original field location.
pub fn migrate_legacy_with_extensions(
    tree: &impl Tree,
    ledger: MigrationInput<'_>,
    baseline: MigrationInput<'_>,
    adapters: &[&dyn ForeignLedgerAdapter],
) -> Result<Migration, Error> {
    let result = || {
        let (profile, document) = read(ledger.text)?;
        let mut extensions = extensions::Transport::new(&document, &profile);
        let table = &document.nodes()[0];
        if !["census", "job", "distinct"].iter().any(|key| {
            table.children.iter().any(|id| {
                document.nodes()[*id]
                    .key
                    .as_ref()
                    .is_some_and(|name| name.decoded == *key)
            })
        }) {
            return Err(Error::Options(
                "unrecognized foreign ledger layout; expected census, job or distinct tables"
                    .into(),
            ));
        }
        let historical = baseline::read(baseline)?;
        let mut source = Fields::new(&document, "ledger", table.children.clone());
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
            let original = row.table.clone();
            let (job, mut row) = rows::job(
                &mut graph,
                row,
                &historical,
                &workspace,
                &owners,
                &mut ids,
                &mut unmapped,
            )?;
            extensions.apply(&mut graph, &job, &original, &mut row, adapters)?;
            row.finish(&mut unmapped)?;
        }
        for (index, row) in source.rows("distinct")?.into_iter().enumerate() {
            rows::distinct(&mut graph, index, row, &mut unmapped)?;
        }
        settings.finish(&mut unmapped)?;
        source.finish(&mut unmapped)?;
        finish(graph, unmapped, ledger.path)
    };
    result().map_err(|error| input(ledger.path, error))
}

fn read(text: &str) -> Result<(Profile, Document<'_>), Error> {
    if text.len() > MAX_LEDGER_BYTES {
        return Err(Error::Options(format!(
            "foreign ledger exceeds {MAX_LEDGER_BYTES} bytes"
        )));
    }
    let profile = Profile::new(
        "raf-foreign-ledger",
        1,
        Vocabulary::standard().map_err(Error::ImportToml)?,
        Bounds {
            max_source_bytes: MAX_LEDGER_BYTES as u64,
            max_depth: 32,
            ..Bounds::standard()
        },
    )
    .map_err(Error::ImportToml)?;
    let document = purrdf_toml::analyze(
        SourceDocument {
            id: "urn:raf:foreign-ledger",
            bytes: text.as_bytes(),
        },
        &profile,
    )
    .map_err(Error::ImportToml)?;
    check_depth(&document)?;
    Ok((profile, document))
}

fn check_depth(document: &Document<'_>) -> Result<(), Error> {
    let mut pending = purrdf_lex::walk::WorkList::<_, 32>::with((0usize, 0usize));
    while let Some((id, depth)) = pending.pop() {
        let node = &document.nodes()[id];
        if !matches!(node.kind, Kind::Scalar(_)) && depth >= 32 {
            return Err(Error::Options(
                "foreign ledger exceeds 32 levels of nesting".into(),
            ));
        }
        if matches!(
            node.kind,
            Kind::Scalar(Scalar::Float(Float::Infinity { .. }))
        ) && !["inf", "+inf", "-inf"].contains(&&document.text()[node.span.clone()])
        {
            return Err(Error::Options(format!(
                "foreign floating-point number overflowed at bytes {}..{}",
                node.span.start, node.span.end
            )));
        }
        for child in &node.children {
            pending
                .try_push((*child, depth + 1))
                .map_err(Error::ImportStorage)?;
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
    insert(graph, triple)
}

fn insert(graph: &mut Builder, triple: Triple) -> Result<(), Error> {
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
