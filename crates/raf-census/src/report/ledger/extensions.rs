// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! RAF ownership policy over the one native RDF graph.

use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

use purrdf_lex::walk::WorkList;
use raf_rdf::{Graph, Node, Value};

use super::{E, Error, LedgerJob};

/// A borrowed, uniquely owned extension closure over native RDF assertions.
///
/// Keys are normalized local fragment ids, exactly as [`Value::iri`] presents
/// owned references. The validator must check its own field meanings and schema;
/// ownership validation does not attest their content or authorize a sanction.
#[derive(Clone, Copy)]
pub struct LedgerExtensionGraph<'a> {
    graph: &'a Graph,
    root: &'a str,
    members: &'a BTreeSet<String>,
}

impl<'a> LedgerExtensionGraph<'a> {
    pub(super) const fn new(
        graph: &'a Graph,
        root: &'a str,
        members: &'a BTreeSet<String>,
    ) -> Self {
        Self {
            graph,
            root,
            members,
        }
    }

    /// The normalized local id of the job's extension root.
    #[must_use]
    pub const fn root_id(self) -> &'a str {
        self.root
    }

    /// The extension root, whose class is the ledger's `Extension`.
    ///
    /// # Panics
    /// If the ledger's validated root-membership invariant is broken.
    #[must_use]
    pub fn root(self) -> Node<'a> {
        self.graph.get(self.root).expect("validated extension root")
    }

    /// An owned subject, or `None` for a foreign, core or absolute id.
    ///
    /// # Panics
    /// If the ledger's validated subject-membership invariant is broken.
    #[must_use]
    pub fn get(self, id: &str) -> Option<Node<'a>> {
        self.members
            .contains(id)
            .then(|| self.graph.get(id).expect("validated extension member"))
    }

    /// Every owned subject in lexical local-id order, including the root.
    ///
    /// # Panics
    /// If the ledger's validated subject-membership invariant is broken.
    pub fn nodes(self) -> impl Iterator<Item = (&'a str, Node<'a>)> + use<'a> {
        self.members.iter().map(move |id| {
            (
                id.as_str(),
                self.graph.get(id).expect("validated extension member"),
            )
        })
    }
}

pub(super) fn collect(
    graph: &Graph,
    jobs: &[LedgerJob],
    core: &BTreeSet<String>,
) -> Result<BTreeMap<String, BTreeSet<String>>, Error> {
    let mut owners = BTreeMap::new();
    // Reserve all roots before any traversal: a closure must never acquire
    // another extension's root just because that job sorts after its owner.
    for job in jobs {
        if let Some(root) = &job.extension {
            if core.contains(root) {
                return Err(
                    E::Value(format!("extension {root}: core subject cannot be owned")).into(),
                );
            }
            if owners.insert(root.as_str(), root.as_str()).is_some() {
                return Err(E::Repeated(format!("extension root {root}: shared by jobs")).into());
            }
        }
    }
    for job in jobs {
        let Some(root) = job.extension.as_deref() else {
            continue;
        };
        let mut pending = WorkList::<_, 16>::with(root);
        while let Some(id) = pending.pop() {
            let node = graph.get(id).ok_or_else(|| E::Dangling(id.into()))?;
            for value in node.objects() {
                let atoms = match value.quoted() {
                    Some((subject, _, object)) => [Some(subject), Some(object)],
                    None => [Some(value), None],
                };
                for target in atoms.into_iter().flatten().filter_map(Value::iri) {
                    if !target.is_empty() && !target.starts_with('#') {
                        continue;
                    }
                    if core.contains(target) {
                        return Err(E::Value(format!(
                            "extension {root}: {id} references core subject {target:?}"
                        ))
                        .into());
                    }
                    match owners.entry(target) {
                        Entry::Occupied(owner) if *owner.get() != root => {
                            return Err(E::Value(format!(
                                "extension {root}: {id} references {target}, owned by extension {}",
                                owner.get()
                            ))
                            .into());
                        }
                        Entry::Occupied(_) => {}
                        Entry::Vacant(owner) => {
                            // Mark on discovery: cycles and converging paths
                            // schedule each subject once under its sole owner.
                            owner.insert(root);
                            pending.push(target);
                        }
                    }
                }
            }
        }
    }
    for (id, _) in graph.nodes() {
        if !core.contains(&id) && !owners.contains_key(id.as_str()) {
            return Err(E::Orphan(format!("ledger {id}")).into());
        }
    }
    let mut closures = BTreeMap::<String, BTreeSet<String>>::new();
    for (id, owner) in owners {
        closures.entry(owner.into()).or_default().insert(id.into());
    }
    Ok(closures)
}
