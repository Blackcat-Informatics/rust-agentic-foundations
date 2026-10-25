// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! RAF ownership and claims over the supplied native TOML value projection.

use std::{collections::BTreeMap, sync::Arc};

use purrdf_core::{BaseIri, GraphMatch, ir::RdfDataset};
use purrdf_toml::{Document, Node, Profile};
use raf_rdf::{Builder, TermValue as Term, Triple};

use super::{class, fields::Fields, insert, put};
use crate::{Error, report::mint};

/// A borrowed original whole immediate field of one foreign job.
///
/// Native nodes, source spellings and occurrence identities remain available to
/// the consumer. Construction is private: a claim cannot substitute a different
/// occurrence or drop any of the selected field's descendants.
#[derive(Clone, Copy)]
pub struct ForeignField<'d, 's> {
    document: &'d Document<'s>,
    profile: &'d Profile,
    occurrence: usize,
}

impl<'d, 's> ForeignField<'d, 's> {
    /// The already-parsed native document; no second grammar is required.
    #[must_use]
    pub const fn document(self) -> &'d Document<'s> {
        self.document
    }

    /// The exact native profile that admitted this document and its identity.
    #[must_use]
    pub const fn profile(self) -> &'d Profile {
        self.profile
    }

    /// The field's immutable native Value occurrence index.
    #[must_use]
    pub const fn occurrence(self) -> usize {
        self.occurrence
    }

    /// The complete native field node, including decoded key and child indices.
    #[must_use]
    pub fn node(self) -> &'d Node {
        &self.document.nodes()[self.occurrence]
    }

    /// The native path distinguishing this field from repeated job occurrences.
    #[must_use]
    pub fn location(self) -> &'d str {
        &self.node().path
    }

    fn error(self, source: Error) -> Error {
        Error::Input {
            file: self.location().into(),
            source: Box::new(source),
        }
    }
}

/// A consumer's source-schema acceptance and vocabulary for foreign job fields.
///
/// The framework retains every native Value fact below an accepted whole field.
/// A consumer never receives the output builder and cannot omit descendants.
/// Consumed core or descriptive metadata fields cannot be commandeered. The
/// resulting native extension still requires its registered `LedgerExtension`;
/// accepting foreign values does not establish evidence validity or a sanction.
pub trait ForeignLedgerAdapter: Sync {
    /// Decline with None, refuse with Some(Err), or claim with an absolute
    /// consumer-owned predicate IRI in Some(Ok). Every adapter is consulted;
    /// multiple claims on one field refuse, including competing failed claims.
    /// The predicate links the job's extension root to the retained field root.
    ///
    /// # Errors
    /// Return the source-schema or policy reason to refuse this claimed field.
    fn claim(&self, job: &str, field: ForeignField<'_, '_>) -> Option<Result<String, String>>;
}

pub(super) struct Transport<'d, 's> {
    document: &'d Document<'s>,
    profile: &'d Profile,
    projected: Option<Arc<RdfDataset>>,
}

impl<'d, 's> Transport<'d, 's> {
    pub(super) const fn new(document: &'d Document<'s>, profile: &'d Profile) -> Self {
        Self {
            document,
            profile,
            projected: None,
        }
    }

    pub(super) fn apply(
        &mut self,
        graph: &mut Builder,
        job: &str,
        original: &[usize],
        remaining: &mut Fields<'_, '_>,
        adapters: &[&dyn ForeignLedgerAdapter],
    ) -> Result<(), Error> {
        let mut claims = Vec::new();
        for &occurrence in original {
            let field = ForeignField {
                document: self.document,
                profile: self.profile,
                occurrence,
            };
            let mut answers: Vec<_> = adapters
                .iter()
                .filter_map(|adapter| adapter.claim(job, field))
                .collect();
            if answers.len() > 1 {
                return Err(field.error(Error::Options(
                    "multiple foreign adapters claim the same whole field; register one owner"
                        .into(),
                )));
            }
            let Some(answer) = answers.pop() else {
                continue;
            };
            if !remaining.table.contains(&occurrence) {
                return Err(field.error(Error::Options(
                    "foreign adapter claims a core-owned or already consumed field".into(),
                )));
            }
            let predicate = answer.map_err(|reason| field.error(Error::Options(reason)))?;
            BaseIri::parse(&predicate).map_err(|source| {
                field.error(Error::ImportPredicate {
                    predicate: predicate.clone(),
                    source,
                })
            })?;
            if predicate.starts_with(raf_rdf::vocab::REPORT)
                || predicate == purrdf_core::vocab::rdf::TYPE
            {
                return Err(field.error(Error::Options(
                    "foreign field predicate must not replace core report vocabulary or rdf:type"
                        .into(),
                )));
            }
            claims.push((field, predicate));
        }
        for (field, predicate) in claims {
            self.retain(graph, job, field.occurrence, &predicate)
                .map_err(|source| field.error(source))?;
            remaining.table.retain(|id| *id != field.occurrence);
        }
        Ok(())
    }

    fn retain(
        &mut self,
        graph: &mut Builder,
        job: &str,
        root: usize,
        predicate: &str,
    ) -> Result<(), Error> {
        let job = mint::job(job);
        let extension = format!("{job}/extension");
        let selected = self.descendants(root)?;
        let mut relocation = BTreeMap::from([(
            self.document.id().to_owned(),
            format!("{extension}/document"),
        )]);
        for &occurrence in &selected {
            relocation.insert(
                self.value_id(occurrence),
                format!("{extension}/value/{occurrence}"),
            );
        }
        if self.projected.is_none() {
            self.projected =
                Some(purrdf_toml::project(self.document, self.profile).map_err(Error::ImportToml)?);
        }
        let Some(native) = self.projected.as_deref() else {
            return Err(Error::Options(
                "native foreign projection was not retained after successful construction".into(),
            ));
        };
        put(graph, &job, "extension", Term::iri(&extension))?;
        class(graph, &extension, "Extension")?;
        insert(
            graph,
            Triple {
                subject: Term::iri(&extension),
                predicate: Term::iri(predicate),
                object: Term::iri(&relocation[&self.value_id(root)]),
            },
        )?;
        copy_subject(native, self.document.id(), &relocation, graph)?;
        for occurrence in selected {
            let original = self.value_id(occurrence);
            copy_subject(native, &original, &relocation, graph)?;
            if occurrence != root {
                let parent = self.document.nodes()[occurrence].parent.ok_or_else(|| {
                    Error::Options("selected foreign descendant has no native parent".into())
                })?;
                let parent = relocation.get(&self.value_id(parent)).ok_or_else(|| {
                    Error::Options(
                        "selected foreign descendant is outside its retained parent".into(),
                    )
                })?;
                insert(
                    graph,
                    Triple {
                        subject: Term::iri(parent),
                        predicate: Term::iri(purrdf_core::vocab::rdfs::MEMBER),
                        object: Term::iri(&relocation[&original]),
                    },
                )?;
            }
        }
        Ok(())
    }

    fn descendants(&self, root: usize) -> Result<Vec<usize>, Error> {
        let mut selected = Vec::new();
        let mut pending = purrdf_lex::walk::WorkList::<_, 32>::with(root);
        while let Some(id) = pending.pop() {
            selected.push(id);
            for &child in &self.document.nodes()[id].children {
                pending.try_push(child).map_err(Error::ImportStorage)?;
            }
        }
        Ok(selected)
    }

    fn value_id(&self, occurrence: usize) -> String {
        // The supplied native TOML wire contract specifies Value subjects as #vN.
        format!("{}#v{occurrence}", self.document.id())
    }
}

fn copy_subject(
    native: &RdfDataset,
    original: &str,
    relocation: &BTreeMap<String, String>,
    graph: &mut Builder,
) -> Result<(), Error> {
    let id = native.term_id_by_iri(original).ok_or_else(|| {
        Error::Options(format!(
            "native foreign projection omitted subject {original}"
        ))
    })?;
    let plan = RdfDataset::probe_plan(true, false, false, GraphMatch::Any);
    let mut count = 0usize;
    for fact in native.quads_for_pattern_with_plan(&plan, Some(id), None, None, GraphMatch::Any) {
        if fact.g.is_some() {
            return Err(Error::Options(
                "foreign value projection must use the native default graph".into(),
            ));
        }
        let mut object = native.term_value(fact.o);
        if let Term::Iri(iri) = &mut object
            && let Some(local) = relocation.get(iri)
        {
            local.clone_into(iri);
        }
        insert(
            graph,
            Triple {
                subject: Term::iri(&relocation[original]),
                predicate: native.term_value(fact.p),
                object,
            },
        )?;
        count += 1;
    }
    if count == 0 {
        return Err(Error::Options(format!(
            "native foreign projection has no facts for {original}"
        )));
    }
    insert(
        graph,
        Triple {
            subject: Term::iri(&relocation[original]),
            predicate: Term::iri(format!("{}wasDerivedFrom", purrdf_core::vocab::prov::NS)),
            object: Term::iri(original),
        },
    )
}
