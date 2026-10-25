// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One repository policy shared verbatim by the host and real check target.

use raf_census::{FileCtx, Gate, LedgerExtension, Rule, RuleFixtures, RuleHit};
use std::path::Path;

struct NoLocalCopy;

impl Rule for NoLocalCopy {
    fn id(&self) -> &'static str {
        "no-local-copy"
    }

    fn check(&self, ctx: &FileCtx, file: &syn::File, out: &mut Vec<RuleHit>) {
        for item in &file.items {
            if let syn::Item::Fn(item) = item
                && item.sig.ident == "local_copy"
            {
                out.push(RuleHit {
                    line: item.sig.ident.span().start().line,
                    symbol: format!("{}::local_copy", ctx.package),
                    message: "use home".into(),
                });
            }
        }
    }

    fn fixtures(&self) -> RuleFixtures {
        RuleFixtures {
            positive: vec!["fn local_copy() {}"],
            negative: vec!["fn home() {}"],
        }
    }
}

struct Owner;

const CAPACITY_PROOF: &str = "urn:example:capacity-proof";
const TOML: &str = "https://w3id.org/purrdf/toml#";
const MEMBERS: &str = "http://www.w3.org/2000/01/rdf-schema#member";

struct CapacityProof;

impl LedgerExtension for CapacityProof {
    fn validate(
        &self,
        _: &str,
        graph: raf_census::LedgerExtensionGraph<'_>,
    ) -> Option<Result<(), String>> {
        let mut roots = graph.root().values(CAPACITY_PROOF);
        let root = roots.next()?;
        Some((|| {
            if roots.next().is_some() {
                return Err("capacity proof requires one field root".into());
            }
            let node = root
                .iri()
                .and_then(|id| graph.get(id))
                .ok_or_else(|| "capacity proof root is not owned".to_owned())?;
            let rows = proof_members(graph, node, &["array", "array-of-tables"])?;
            if rows.len() != 2 {
                return Err("capacity proof requires both declared rows".into());
            }
            for row in rows {
                validate_proof_row(graph, row)?;
            }
            Ok(())
        })())
    }
}

fn validate_proof_row(
    graph: raf_census::LedgerExtensionGraph<'_>,
    row: raf_rdf::Node<'_>,
) -> Result<(), String> {
    let mut seen = purrdf_lex::allocation::OrderedTable::default();
    let mut resident = purrdf_lex::allocation::Resident;
    let mut memory = purrdf_lex::allocation::Memory::new(&mut resident);
    for field in proof_members(graph, row, &["inline-table", "table-element"])? {
        let key = proof_literal(field, "key", raf_rdf::vocab::STRING)?;
        if seen
            .insert(key, (), &mut memory)
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Err(format!("capacity proof repeats field {key}"));
        }
        match key {
            "file" if proof_string(field)? == "src/lib.rs" => {}
            "file" => return Err("capacity proof file changed".into()),
            "source" if proof_string(field)? == "source:original-proof-pin" => {}
            "source" => return Err("capacity proof source pin is stale".into()),
            "reason" if proof_string(field)? == "Declared capacity argument" => {}
            "reason" => return Err("capacity proof reason changed".into()),
            "authorities" | "constructors" => {
                let items = proof_members(graph, field, &["array"])?;
                let values = items
                    .into_iter()
                    .map(proof_string)
                    .collect::<Result<Vec<_>, _>>()?;
                let expected = if key == "authorities" {
                    vec!["A", "B", "A"]
                } else {
                    vec!["Make"]
                };
                if values != expected {
                    return Err(format!("capacity proof {key} changed"));
                }
            }
            _ => return Err(format!("capacity proof has unknown field {key}")),
        }
    }
    if seen.len() != 5 {
        return Err("capacity proof is missing required fields".into());
    }
    Ok(())
}

fn proof_members<'a>(
    graph: raf_census::LedgerExtensionGraph<'a>,
    node: raf_rdf::Node<'a>,
    kinds: &[&str],
) -> Result<Vec<raf_rdf::Node<'a>>, String> {
    if !kinds.contains(&proof_kind(node)?) {
        return Err("capacity proof container kind changed".into());
    }
    let size = proof_literal(node, "size", raf_rdf::vocab::INTEGER)?
        .parse::<usize>()
        .map_err(|error| format!("capacity proof size: {error}"))?;
    let mut members = Vec::new();
    for value in node.values(MEMBERS) {
        let child = value
            .iri()
            .and_then(|id| graph.get(id))
            .ok_or_else(|| "capacity proof member is not owned".to_owned())?;
        let ordinal = proof_literal(child, "ordinal", raf_rdf::vocab::INTEGER)?
            .parse::<usize>()
            .map_err(|error| format!("capacity proof ordinal: {error}"))?;
        members.push((ordinal, child));
    }
    if members.len() != size {
        return Err("capacity proof members do not match its declared size".into());
    }
    members.sort_unstable_by_key(|(ordinal, _)| *ordinal);
    for (expected, (ordinal, _)) in members.iter().enumerate() {
        if *ordinal != expected {
            return Err("capacity proof member ordinals are not contiguous".into());
        }
    }
    Ok(members.into_iter().map(|(_, node)| node).collect())
}

fn proof_kind(node: raf_rdf::Node<'_>) -> Result<&str, String> {
    let mut classes = node.values(raf_rdf::vocab::TYPE);
    if classes.next().and_then(raf_rdf::Value::iri) != Some(format!("{TOML}Value").as_str())
        || classes.next().is_some()
    {
        return Err("capacity proof requires a native Value occurrence".into());
    }
    proof_literal(node, "kind", raf_rdf::vocab::STRING)
}

fn proof_string(node: raf_rdf::Node<'_>) -> Result<&str, String> {
    if proof_kind(node)? != "string" {
        return Err("capacity proof requires a native string value".into());
    }
    proof_literal(node, "value", raf_rdf::vocab::STRING)
}

fn proof_literal<'a>(
    node: raf_rdf::Node<'a>,
    key: &str,
    datatype: &str,
) -> Result<&'a str, String> {
    let mut values = node.values(&format!("{TOML}{key}"));
    let (text, stated) = values
        .next()
        .and_then(raf_rdf::Value::literal)
        .ok_or_else(|| format!("capacity proof is missing native {key}"))?;
    if stated != datatype || values.next().is_some() {
        return Err(format!(
            "capacity proof native {key} has the wrong type or multiplicity"
        ));
    }
    Ok(text)
}

impl LedgerExtension for Owner {
    fn validate(
        &self,
        _: &str,
        graph: raf_census::LedgerExtensionGraph<'_>,
    ) -> Option<Result<(), String>> {
        let node = graph.root();
        let mut values = node.values("urn:example:owner");
        let value = values.next()?;
        let owner = value.iri().and_then(|id| graph.get(id));
        Some(
            if values.next().is_none()
                && graph.nodes().count() == 2
                && node
                    .properties()
                    .all(|key| key == raf_rdf::vocab::TYPE || key == "urn:example:owner")
                && owner.is_some_and(|owner| {
                    let mut names = owner.values("urn:example:name");
                    let mut classes = owner.values(raf_rdf::vocab::TYPE);
                    names.next().and_then(raf_rdf::Value::literal)
                        == Some(("demo", raf_rdf::vocab::STRING))
                        && names.next().is_none()
                        && classes.next().and_then(raf_rdf::Value::iri) == Some("urn:example:Owner")
                        && classes.next().is_none()
                        && owner
                            .properties()
                            .all(|key| key == raf_rdf::vocab::TYPE || key == "urn:example:name")
                })
            {
                Ok(())
            } else {
                Err(
                    "owner extension requires exactly nested owner demo and no unknown fields"
                        .into(),
                )
            },
        )
    }
}

pub(crate) fn registered_gate(root: impl AsRef<Path>) -> Gate {
    rule_gate(root).extensions(vec![
        Box::new(Owner) as Box<dyn LedgerExtension>,
        Box::new(CapacityProof),
    ])
}

pub(crate) fn rule_gate(root: impl AsRef<Path>) -> Gate {
    Gate::workspace(root).rules(vec![Box::new(NoLocalCopy) as Box<dyn Rule>])
}
