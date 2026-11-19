// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed declarations for the repository-wide numeric bounds inventory.

use std::collections::{BTreeMap, BTreeSet};

use raf_rdf::{Graph, Node};
use raf_source::RelPath;

use super::{E, Error, anchor, fields, node_at, required_text, shape, targets};
use crate::SiteKey;

pub(crate) const UNCLASSIFIED: &str = "numeric-bound-unclassified";
pub(crate) const REFUSING: &str = "numeric-bound-refusing";
pub(crate) const TRUNCATING: &str = "numeric-bound-truncating";
pub(crate) const IDS: [&str; 3] = [REFUSING, TRUNCATING, UNCLASSIFIED];

/// A ledger's justification for one numeric bound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoundClassification {
    /// The limit is fixed by a format or protocol.
    Structural,
    /// The limit is a documented policy choice.
    Sanctioned {
        /// The reason for the policy choice.
        reason: String,
        /// Absolute IRI of its documented source item.
        anchor: String,
    },
    /// The limit belongs to an experiment.
    Experimental,
}

#[derive(Clone, Debug)]
pub(crate) struct BoundRegistry {
    pub(crate) ratcheted: bool,
    pub(crate) rows: BTreeMap<(RelPath, SiteKey), BoundClassification>,
}

pub(super) fn read(
    graph: &Graph,
    root: &Node<'_>,
    reached: &mut BTreeSet<String>,
) -> Result<Option<BoundRegistry>, Error> {
    let ids = targets(root, "numericBounds")?;
    let id = match ids.as_slice() {
        [] => return Ok(None),
        [id] => id,
        _ => return Err(E::Repeated("ledger numericBounds".into()).into()),
    };
    let node = owned(graph, id, "NumericBounds", reached)?;
    let ratcheted = match required_text(&node, "mode")?.as_str() {
        "strict" => false,
        "ratcheted" => true,
        mode => {
            return Err(E::Value(format!(
                "{id}: numericBounds mode {mode:?}; use strict or ratcheted"
            ))
            .into());
        }
    };
    let mut rows = BTreeMap::new();
    for id in targets(&node, "bound")? {
        let row = owned(graph, &id, "NumericBound", reached)?;
        let file = required_text(&row, "file")?;
        let key = required_text(&row, "siteKey")?;
        let key = SiteKey::from_hex(&key).ok_or_else(|| {
            E::Value(format!(
                "{id}: siteKey must contain 64 lowercase hexadecimal digits"
            ))
        })?;
        let path = RelPath::new(&file)
            .map_err(|error| E::Value(format!("{id}: numeric bound file {file:?}: {error}")))?;
        if path.is_root() || path.to_string() != file {
            return Err(E::Value(format!("{id}: use a canonical workspace-relative file")).into());
        }
        let site = (path, key);
        if rows.insert(site, classification(&row, &id)?).is_some() {
            return Err(E::Repeated(format!("{id}: numeric bound {file}:{}", key.to_hex())).into());
        }
    }
    Ok(Some(BoundRegistry { ratcheted, rows }))
}

fn owned<'a>(
    graph: &'a Graph,
    id: &str,
    class: &str,
    reached: &mut BTreeSet<String>,
) -> Result<Node<'a>, E> {
    if !reached.insert(id.to_owned()) {
        return Err(E::Repeated(format!("ledger node {id}")));
    }
    let node = node_at(graph, id, class)?;
    fields(&node, class)?;
    Ok(node)
}

fn classification(node: &Node<'_>, id: &str) -> Result<BoundClassification, E> {
    let value = required_text(node, "classification")?;
    if value == "sanctioned" {
        return Ok(BoundClassification::Sanctioned {
            reason: required_text(node, "reason")?,
            anchor: anchor(node)?,
        });
    }
    if ["reason", "anchor"]
        .iter()
        .any(|key| node.values(&shape::property(key)).next().is_some())
    {
        return Err(E::Value(format!(
            "{id}: only sanctioned numeric bounds admit reason and anchor"
        )));
    }
    match value.as_str() {
        "structural" => Ok(BoundClassification::Structural),
        "experimental" => Ok(BoundClassification::Experimental),
        _ => Err(E::Value(format!(
            "{id}: numeric bound classification {value:?}; use structural, sanctioned or experimental"
        ))),
    }
}
