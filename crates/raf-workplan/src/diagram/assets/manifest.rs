// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed RDF ownership statements; these are publication policy, not source proof.

use purrdf_core::{GraphMatch, RdfDataset, RdfDatasetBuilder, RdfLiteral, TermId, TermRef};
use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use purrdf_rdf::{
    NativeRdfFormat, ParseOptions, SerializeGraph, SerializeOptions, StatementLayer,
    parse_dataset_with, serialize_dataset_with,
};

use super::{DiagramError, MANIFEST, MAX_ASSETS, Row, stem};
use crate::diagram::semantic::hash;

const ROOT: &str = "https://blackcatinformatics.ca/raf/workplan/diagram-assets";
const VOCAB: &str = "https://blackcatinformatics.ca/raf/workplan/diagram-assets#";
const STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct Manifest {
    pub copyright: String,
    pub licence: String,
    pub rows: Vec<Row>,
}

pub(super) fn serialize(dataset: &RdfDataset) -> Result<String, DiagramError> {
    let result = serialize_dataset_with(
        dataset,
        rdf_format(dataset).0,
        None,
        &SerializeOptions {
            selection: SerializeGraph::Dataset,
            statement_layer: StatementLayer::Emit,
            jsonld_options: None,
        },
    )
    .map_err(DiagramError::data)?;
    if result.statement_rows_dropped != 0
        || result.directional_literals_dropped != 0
        || result.named_graph_rows_dropped != 0
    {
        return Err(DiagramError::refused(
            "native diagram serialization would discard RDF facts",
        ));
    }
    String::from_utf8(result.bytes).map_err(DiagramError::data)
}

pub(super) fn rdf_format(dataset: &RdfDataset) -> (NativeRdfFormat, &'static str) {
    if dataset.named_graphs().next().is_some() {
        (NativeRdfFormat::TriG, "diagram.trig")
    } else {
        (NativeRdfFormat::Turtle, "diagram.ttl")
    }
}

fn spdx(copyright: &str, licence: &str) -> Result<(), DiagramError> {
    for value in [copyright, licence] {
        if value.trim().is_empty() || value.contains(['\r', '\n']) || value.contains("--") {
            return Err(DiagramError::refused(
                "diagram SPDX values must be nonempty single lines without XML comment delimiters",
            ));
        }
        // Native XML Char law; comments retain the exact original SPDX text.
        let _ = purrdf_core::xml_escape::escape(value, purrdf_core::xml_escape::Context::Text)
            .map_err(DiagramError::data)?;
    }
    Ok(())
}

impl Manifest {
    pub(super) fn text(&self) -> Result<String, DiagramError> {
        spdx(&self.copyright, &self.licence)?;
        let mut builder = RdfDatasetBuilder::new();
        let root = builder.intern_iri(ROOT).map_err(DiagramError::data)?;
        put_text(&mut builder, root, "schema", "raf-diagram-assets-1")?;
        put_text(&mut builder, root, "copyright", &self.copyright)?;
        put_text(&mut builder, root, "licence", &self.licence)?;
        for row in &self.rows {
            let id = builder
                .intern_iri(&format!("{ROOT}/{}", hash(row.path.as_str())))
                .map_err(DiagramError::data)?;
            put(&mut builder, root, "asset", id)?;
            let owner = builder.intern_iri(&row.owner).map_err(DiagramError::data)?;
            put(&mut builder, id, "owner", owner)?;
            put_text(&mut builder, id, "view", &row.view)?;
            put_text(&mut builder, id, "path", row.path.as_str())?;
            put_text(&mut builder, id, "digest", &row.digest)?;
        }
        let data = builder.freeze().map_err(DiagramError::data)?;
        Ok(format!(
            "# SPDX-FileCopyrightText: {}\n# SPDX-License-Identifier: {}\n\n{}",
            self.copyright,
            self.licence,
            serialize(&data)?
        ))
    }

    pub(super) fn parse(text: &str) -> Result<Self, DiagramError> {
        let parsed = parse_dataset_with(
            text.as_bytes(),
            "text/turtle",
            None,
            &ParseOptions::default(),
        )
        .map_err(DiagramError::data)?;
        let reader = Reader {
            data: &parsed.dataset,
        };
        if reader.data.named_graphs().next().is_some()
            || reader.data.reifiers().next().is_some()
            || reader.data.annotations().next().is_some()
            || reader.data.quads().count() > MAX_ASSETS * 5 + 3
        {
            return Err(DiagramError::refused(
                "diagram ownership manifest exceeds its closed graph or statement allowance",
            ));
        }
        let root = reader
            .data
            .term_id_by_iri(ROOT)
            .ok_or_else(|| DiagramError::refused("diagram ownership manifest root is missing"))?;
        if reader.text(root, "schema")? != "raf-diagram-assets-1" {
            return Err(DiagramError::refused(
                "unsupported diagram ownership schema",
            ));
        }
        let copyright = reader.text(root, "copyright")?.to_owned();
        let licence = reader.text(root, "licence")?.to_owned();
        spdx(&copyright, &licence)?;
        let mut known = OrderedTable::default();
        let mut memory_owner = Resident;
        let mut memory = Memory::new(&mut memory_owner);
        known
            .insert(root, (), &mut memory)
            .map_err(DiagramError::data)?;
        let mut rows = Vec::new();
        for id in reader.values(root, "asset") {
            if rows.len() >= MAX_ASSETS {
                return Err(DiagramError::refused("too many owned diagram assets"));
            }
            let owner = reader.iri(reader.one(id, "owner")?)?.to_owned();
            validate_owner(&owner)?;
            let view = reader.text(id, "view")?.to_owned();
            if !raf_okf::is_kebab(&view) {
                return Err(DiagramError::refused(
                    "manifest view name is not a kebab slug",
                ));
            }
            let path =
                raf_source::RelPath::new(reader.text(id, "path")?).map_err(DiagramError::data)?;
            let parent = stem(&owner, &view);
            if ![
                "diagram.svg",
                "diagram.json",
                "diagram.ttl",
                "diagram.trig",
                "diagram.txt",
            ]
            .iter()
            .any(|file| path.as_str() == format!("{parent}/{file}"))
            {
                return Err(DiagramError::refused(format!(
                    "manifest asset {path} is outside its owner's derived destination"
                )));
            }
            if reader.iri(id)? != format!("{ROOT}/{}", hash(path.as_str()))
                || known.get(&id).is_some()
            {
                return Err(DiagramError::refused(
                    "duplicate or mismatched manifest asset identity",
                ));
            }
            let digest = reader.text(id, "digest")?.to_owned();
            if purrdf_hash::hex::decode_32_canonical(&digest).is_none() {
                return Err(DiagramError::refused(
                    "manifest digest is not canonical BLAKE3 hex",
                ));
            }
            known
                .insert(id, (), &mut memory)
                .map_err(DiagramError::data)?;
            rows.push(Row {
                owner,
                view,
                path,
                digest,
            });
        }
        reader.closed(root, &known)?;
        purrdf_lex::walk::try_sort_unstable_by(&mut rows, |a, b| {
            Ok::<_, DiagramError>(a.path.cmp(&b.path))
        })
        .map_err(DiagramError::data)?;
        Ok(Self {
            copyright,
            licence,
            rows,
        })
    }
}

fn validate_owner(owner: &str) -> Result<(), DiagramError> {
    for kind in [raf_okf::DocType::Capability, raf_okf::DocType::Architecture] {
        let prefix = format!(
            "{}{}/",
            raf_okf::DOCUMENT_IRI_BASE,
            kind.name().to_ascii_lowercase()
        );
        if let Some(id) = owner.strip_prefix(&prefix) {
            let id = purrdf_lex::percent::decode(id).map_err(DiagramError::data)?;
            if kind.document_iri(&id).map_err(DiagramError::data)? == owner {
                return Ok(());
            }
        }
    }
    Err(DiagramError::refused(format!(
        "{MANIFEST} owner is not a checked capability or architecture identity"
    )))
}

fn put(
    builder: &mut RdfDatasetBuilder,
    subject: TermId,
    key: &str,
    object: TermId,
) -> Result<(), DiagramError> {
    let predicate = builder
        .intern_iri(&format!("{VOCAB}{key}"))
        .map_err(DiagramError::data)?;
    builder
        .push_quad(subject, predicate, object, None)
        .map_err(DiagramError::data)
}
fn put_text(
    builder: &mut RdfDatasetBuilder,
    subject: TermId,
    key: &str,
    value: &str,
) -> Result<(), DiagramError> {
    let object = builder
        .intern_literal(RdfLiteral {
            lexical_form: value.to_owned(),
            datatype: Some(STRING.into()),
            language: None,
            direction: None,
        })
        .map_err(DiagramError::data)?;
    put(builder, subject, key, object)
}

struct Reader<'a> {
    data: &'a RdfDataset,
}
impl<'a> Reader<'a> {
    fn closed(&self, root: TermId, known: &OrderedTable<TermId, ()>) -> Result<(), DiagramError> {
        for quad in self.data.quads() {
            if quad.g.is_some() || known.get(&quad.s).is_none() {
                return Err(DiagramError::refused(
                    "orphan or foreign-layer ownership fact",
                ));
            }
            let predicate = self.iri(quad.p)?;
            let key = predicate
                .strip_prefix(VOCAB)
                .ok_or_else(|| DiagramError::refused("foreign ownership predicate"))?;
            let allowed: &[&str] = if quad.s == root {
                &["schema", "copyright", "licence", "asset"]
            } else {
                &["owner", "view", "path", "digest"]
            };
            if !allowed.contains(&key) {
                return Err(DiagramError::refused(format!(
                    "unknown ownership field {key}"
                )));
            }
        }
        Ok(())
    }
    fn values(&self, subject: TermId, key: &str) -> impl Iterator<Item = TermId> + '_ {
        let property = self.data.term_id_by_iri(&format!("{VOCAB}{key}"));
        self.data
            .quads_for_pattern_with_plan(
                &RdfDataset::probe_plan(true, false, false, GraphMatch::Default),
                Some(subject),
                None,
                None,
                GraphMatch::Default,
            )
            .filter(move |quad| Some(quad.p) == property)
            .map(|quad| quad.o)
    }
    fn one(&self, subject: TermId, key: &str) -> Result<TermId, DiagramError> {
        let mut values = self.values(subject, key);
        let first = values.next();
        match (first, values.next()) {
            (Some(value), None) => Ok(value),
            _ => Err(DiagramError::refused(format!(
                "ownership field {key} requires exactly one value"
            ))),
        }
    }
    fn iri(&self, id: TermId) -> Result<&'a str, DiagramError> {
        match self.data.resolve(id) {
            TermRef::Iri(value) => Ok(value),
            _ => Err(DiagramError::refused("ownership node must be an IRI")),
        }
    }
    fn text(&self, subject: TermId, key: &str) -> Result<&'a str, DiagramError> {
        match self.data.resolve(self.one(subject, key)?) {
            TermRef::Literal {
                lexical,
                datatype,
                language: None,
                direction: None,
            } if matches!(self.data.resolve(datatype), TermRef::Iri(STRING)) => Ok(lexical),
            _ => Err(DiagramError::refused(format!(
                "ownership field {key} requires an xsd:string"
            ))),
        }
    }
}
