// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed enum usage and retention declarations.

use std::collections::{BTreeMap, BTreeSet};

use raf_rdf::{Graph, Node};
use syn::ext::IdentExt as _;

use super::{E, Error, owned, required_text, targets};

pub(crate) type Declarations = BTreeMap<String, BTreeSet<String>>;

pub(super) fn read(
    graph: &Graph,
    root: &Node<'_>,
    reached: &mut BTreeSet<String>,
) -> Result<Declarations, Error> {
    let mut declarations = BTreeMap::new();
    for id in targets(root, "enumUsage")? {
        let row = owned(graph, &id, "EnumUsage", reached)?;
        let symbol = required_text(&row, "symbol")?;
        let mut retained = BTreeSet::new();
        for id in targets(&row, "retainedVariant")? {
            let row = owned(graph, &id, "EnumVariantRetention", reached)?;
            let name = required_text(&row, "variantName")?;
            let name = syn::parse_str::<syn::Ident>(&name)
                .map_err(|cause| {
                    E::Value(format!(
                        "{id}: variantName must be one Rust identifier: {cause}"
                    ))
                })?
                .unraw()
                .to_string();
            required_text(&row, "reason")?;
            if !retained.insert(name.clone()) {
                return Err(E::Repeated(format!("{id}: retention for variant {name}")).into());
            }
        }
        if declarations.insert(symbol.clone(), retained).is_some() {
            return Err(E::Repeated(format!("{id}: enum usage for {symbol}")).into());
        }
    }
    Ok(declarations)
}
