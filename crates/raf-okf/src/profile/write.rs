// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Writing a record: the typed value of a record type as a document whose
//! text [`Profile::validate`](crate::Profile::validate) reads back as the
//! same value.

use crate::document::Document;
use crate::error::ConstructError;
use crate::frontmatter::value::{
    Entry, Frontmatter, List, ListItem, Map, Scalar, ScalarStyle, Value,
};
use crate::profile::review::{ReviewDisposition, ReviewLocation, ReviewRecord, ReviewScope};
use crate::profile::schema::DocType;
use crate::profile::typed::{
    AssessmentRecord, CeilingClass, CeilingLimit, CoverageRecord, DebtRecord, EvidenceRecord,
    ExclusionItem, Input, Measure, Number, RecordHeader,
};

/// A scalar entry.
fn text(key: &str, value: &str) -> Result<Entry, ConstructError> {
    Entry::new(key, Value::Scalar(Scalar::new(value)?))
}

/// A flow list entry `key: [a, b]`.
fn flow(key: &str, items: &[String]) -> Result<Entry, ConstructError> {
    let scalars = items
        .iter()
        .map(Scalar::new)
        .collect::<Result<Vec<_>, _>>()?;
    Entry::new(key, Value::List(List::flow(scalars)?))
}

/// A number, written unquoted as the record holds it: the profile's number
/// form is digits with an optional `-` and fraction, which a reader takes as
/// the plain text it is.
fn number(value: &Number) -> Scalar {
    Scalar {
        text: value.text().to_string(),
        style: ScalarStyle::Plain,
    }
}

/// A list of maps of scalars, one map per row, as a block list; an empty
/// list is written `[]`.
fn rows(key: &str, rows: Vec<Vec<(&str, String)>>) -> Result<Entry, ConstructError> {
    let rows = rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|(field, value)| Ok((field, Scalar::new(value)?)))
                .collect::<Result<Vec<_>, ConstructError>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    scalar_rows(key, rows)
}

/// [`rows`] of scalars as given, so a number is written plain.
fn scalar_rows(key: &str, rows: Vec<Vec<(&str, Scalar)>>) -> Result<Entry, ConstructError> {
    if rows.is_empty() {
        return Entry::new(key, Value::List(List::flow(Vec::new())?));
    }
    let items = rows
        .into_iter()
        .map(|row| {
            let entries = row
                .into_iter()
                .map(|(field, value)| Entry::new(field, Value::Scalar(value)))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ListItem::map(Map::new(entries)?))
        })
        .collect::<Result<Vec<_>, ConstructError>>()?;
    Entry::new(key, Value::List(List::block(items)?))
}

/// The `inputs` entry.
fn inputs(inputs: &[Input]) -> Result<Entry, ConstructError> {
    rows(
        "inputs",
        inputs
            .iter()
            .map(|input| {
                vec![
                    ("kind", input.kind.name().to_string()),
                    ("name", input.name.clone()),
                    ("digest", input.digest.clone()),
                ]
            })
            .collect(),
    )
}

/// A `values` or `limits` entry, or nothing when there is no measure: the
/// profile reads an absent key as none, and an empty map cannot be written.
fn measures(key: &str, measures: &[Measure]) -> Result<Option<Entry>, ConstructError> {
    if measures.is_empty() {
        return Ok(None);
    }
    let entries = measures
        .iter()
        .map(|measure| Entry::new(measure.name.clone(), Value::Scalar(number(&measure.value))))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(Entry::new(key, Value::Map(Map::new(entries)?))?))
}

/// `type`, `id`, `tags` and `timestamp`.
fn header(doc_type: DocType, header: &RecordHeader) -> Result<Vec<Entry>, ConstructError> {
    let mut entries = vec![text("type", doc_type.name())?, text("id", &header.id)?];
    if let Some(tags) = &header.tags {
        entries.push(flow("tags", tags)?);
    }
    if let Some(timestamp) = &header.timestamp {
        entries.push(text("timestamp", timestamp)?);
    }
    Ok(entries)
}

/// A record document: the entries as frontmatter, in the order given, and an
/// empty body.
fn document(entries: Vec<Entry>) -> Result<Document, ConstructError> {
    Document::new(Frontmatter::new(Map::new(entries)?), "")
}

impl ReviewRecord {
    /// Write the closed review claim using the shared record carrier.
    ///
    /// # Errors
    /// Refuses text outside the authoring subset. The profile checks the
    /// required date, revision, input digests and finding dispositions.
    pub fn to_document(&self) -> Result<Document, ConstructError> {
        let mut entries = header(DocType::Review, &self.header)?;
        let scope = match &self.scope {
            ReviewScope::Crate(name) => vec![format!("crate:{name}")],
            ReviewScope::Module(path) => vec![format!("module:{path}")],
            ReviewScope::Items(paths) => paths.iter().map(|path| format!("item:{path}")).collect(),
            ReviewScope::Capability(id) => vec![id.clone()],
        };
        entries.extend([
            flow("scope", &scope)?,
            text("revision", &self.revision)?,
            text("kind", self.kind.name())?,
            inputs(&self.inputs)?,
            text("verdict", self.verdict.name())?,
            text("by", self.by.name())?,
        ]);
        let findings = self
            .findings
            .iter()
            .map(|finding| {
                let location = match &finding.location {
                    ReviewLocation::Item(path) => format!("item:{path}"),
                    ReviewLocation::Lines { file, first, last } => {
                        format!("file:{file}#L{first}-L{last}")
                    }
                };
                let (status, extra) = match &finding.disposition {
                    ReviewDisposition::Open => ("open", None),
                    ReviewDisposition::Fixed(revision) => {
                        ("fixed", Some(("revision", revision.clone())))
                    }
                    ReviewDisposition::Accepted(reason) => {
                        ("accepted", Some(("reason", reason.clone())))
                    }
                    ReviewDisposition::FalsePositive(reason) => {
                        ("false-positive", Some(("reason", reason.clone())))
                    }
                };
                let mut row = vec![
                    ("id", finding.id.clone()),
                    ("severity", finding.severity.name().to_string()),
                    ("location", location),
                    ("status", status.to_string()),
                ];
                row.extend(extra);
                row
            })
            .collect();
        entries.push(rows("findings", findings)?);
        if let Some(reference) = &self.findings_ref {
            entries.push(text("findings_ref", reference)?);
        }
        document(entries)
    }
}

impl EvidenceRecord {
    /// The record as a document, with its keys in the order of the profile's
    /// key table, the measures in record order and an empty body. Its
    /// [`Document::render`] is the text of `work/evidence/<slug>.md`, which
    /// [`Profile::validate`](crate::Profile::validate) reads back as a record
    /// equal to this one when the values satisfy the profile. Comments are
    /// not part of the typed value and are not written.
    ///
    /// ```rust
    /// use raf_okf::{
    ///     Actor, Document, EvidenceRecord, EvidenceSubject, EvidenceVerdict, Input, InputKind,
    ///     Profile, RafDocument,
    /// };
    ///
    /// let digest = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
    /// let mut record = EvidenceRecord::new(
    ///     "parse-bound",
    ///     EvidenceSubject::Command,
    ///     vec![Input::new(InputKind::Symbol, "raf_okf::Document::parse", digest)],
    ///     EvidenceVerdict::Pass,
    ///     Actor::Agent,
    ///     vec!["cap:okf-profile".to_string()],
    /// );
    /// record.command = Some("cargo bench parse".to_string());
    /// let text = record.to_document()?.render();
    /// let read = Document::parse(&text)?;
    /// match Profile.validate(&read, "work/evidence/parse-bound.md") {
    ///     Ok(RafDocument::Evidence(back)) => assert_eq!(back, record),
    ///     other => panic!("{other:?}"),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A [`ConstructError`] when a value cannot be written in the frontmatter
    /// subset, such as a control character with no escape or a measure name
    /// that is not a key.
    pub fn to_document(&self) -> Result<Document, ConstructError> {
        let mut entries = header(DocType::Evidence, &self.header)?;
        entries.push(text("subject", &self.subject.text())?);
        entries.push(inputs(&self.inputs)?);
        entries.push(text("verdict", self.verdict.name())?);
        entries.push(text("by", self.by.name())?);
        entries.push(flow("for", &self.covers)?);
        if let Some(command) = &self.command {
            entries.push(text("command", command)?);
        }
        if let Some(revision) = &self.revision {
            entries.push(text("revision", revision)?);
        }
        entries.extend(measures("values", &self.values)?);
        entries.extend(measures("limits", &self.limits)?);
        document(entries)
    }
}

impl AssessmentRecord {
    /// The record as a document, with its keys in the order of the profile's
    /// key table and an empty body; see [`EvidenceRecord::to_document`].
    ///
    /// # Errors
    ///
    /// A [`ConstructError`] when a value cannot be written in the frontmatter
    /// subset.
    pub fn to_document(&self) -> Result<Document, ConstructError> {
        let mut entries = header(DocType::Assessment, &self.header)?;
        entries.push(text("subject", &self.subject.text())?);
        entries.push(inputs(&self.inputs)?);
        entries.push(text("verdict", self.verdict.name())?);
        entries.push(text("by", self.by.name())?);
        document(entries)
    }
}

impl CoverageRecord {
    /// The record as a document, with its keys in the order of the profile's
    /// key table, the exclusions in record order and an empty body; an empty
    /// `scope` or `exclusions` is written `[]`.
    /// The ceilings and the exceptions are written in record order after the
    /// exclusions, each key left out when it holds none; a ceiling writes
    /// `crate`, `lines` or `percent`, and `of` unless it is `all`, and an
    /// exception its `item` or `file` and its `reason`.
    /// See [`EvidenceRecord::to_document`].
    ///
    /// # Errors
    ///
    /// A [`ConstructError`] when a value cannot be written in the frontmatter
    /// subset.
    pub fn to_document(&self) -> Result<Document, ConstructError> {
        let mut entries = header(DocType::Coverage, &self.header)?;
        entries.push(flow("scope", &self.scope)?);
        entries.push(rows(
            "exclusions",
            self.exclusions
                .iter()
                .map(|exclusion| {
                    let item = match &exclusion.item {
                        ExclusionItem::Item(path) => path.clone(),
                        ExclusionItem::Prefix(path) => format!("{path}::*"),
                    };
                    vec![("item", item), ("reason", exclusion.reason.clone())]
                })
                .collect(),
        )?);
        if !self.ceilings.is_empty() {
            let rows = self
                .ceilings
                .iter()
                .map(|ceiling| {
                    let mut row = vec![("crate", Scalar::new(ceiling.package.as_str())?)];
                    row.push(match &ceiling.limit {
                        CeilingLimit::Lines(lines) => ("lines", number(lines)),
                        CeilingLimit::Percent(percent) => ("percent", number(percent)),
                    });
                    if ceiling.of != CeilingClass::All {
                        row.push(("of", Scalar::new(ceiling.of.name())?));
                    }
                    Ok(row)
                })
                .collect::<Result<Vec<_>, ConstructError>>()?;
            entries.push(scalar_rows("ceilings", rows)?);
        }
        if !self.exceptions.is_empty() {
            entries.push(rows(
                "exceptions",
                self.exceptions
                    .iter()
                    .map(|exception| {
                        vec![
                            (exception.target.key(), exception.target.written()),
                            ("reason", exception.reason.clone()),
                        ]
                    })
                    .collect(),
            )?);
        }
        document(entries)
    }
}

impl DebtRecord {
    /// The record as a document, with its keys in the order of the profile's
    /// key table, the rows in record order and an empty body; an empty
    /// `findings` or `lines` is written `[]`. A finding writes `kind` and
    /// `subject`, and a line `item` and `orphans`, a whole number written
    /// plain. [`DebtRecord::new`] sorts the rows, so two equal debts write
    /// the same bytes. See [`EvidenceRecord::to_document`].
    ///
    /// # Errors
    ///
    /// A [`ConstructError`] when a value cannot be written in the frontmatter
    /// subset.
    pub fn to_document(&self) -> Result<Document, ConstructError> {
        let mut entries = header(DocType::Debt, &self.header)?;
        entries.push(rows(
            "findings",
            self.findings
                .iter()
                .map(|finding| {
                    vec![
                        ("kind", finding.kind.clone()),
                        ("subject", finding.subject.clone()),
                    ]
                })
                .collect(),
        )?);
        let lines = self
            .lines
            .iter()
            .map(|line| {
                Ok(vec![
                    ("item", Scalar::new(line.item.as_str())?),
                    (
                        "orphans",
                        Scalar {
                            text: line.orphans.to_string(),
                            style: ScalarStyle::Plain,
                        },
                    ),
                ])
            })
            .collect::<Result<Vec<_>, ConstructError>>()?;
        entries.push(scalar_rows("lines", lines)?);
        document(entries)
    }
}
