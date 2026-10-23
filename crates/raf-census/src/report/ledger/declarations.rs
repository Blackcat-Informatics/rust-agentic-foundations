// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed declarations; executable policy is supplied by the gate.

use super::{
    E, Error, LedgerJob, NamePattern, ecma, fields, node_at, required_text, strings, targets,
};
use raf_rdf::{Graph, Node};
use std::collections::BTreeSet;

/// One declared syntactic matcher. Declaration does not establish executable support.
#[derive(Clone, Debug)]
pub enum PatternMatcher {
    /// A called function path.
    Call(String),
    /// Any use of a path.
    Path(String),
    /// A macro invocation path.
    Macro(String),
    /// A named type path.
    Type(String),
    /// A bounded native ECMA expression over ordinary string literals.
    LiteralRegex(NamePattern),
    /// Normalized tokens with one-token or balanced-group `$_` wildcards.
    Tokens(String),
}

/// A unique named pattern and exactly one matcher.
#[derive(Clone, Debug)]
pub struct LedgerPattern {
    /// The id referenced by jobs.
    pub id: String,
    /// One-line description of the prohibited construction.
    pub summary: String,
    /// The declared matcher.
    pub matcher: PatternMatcher,
}

/// A functionality's preferred dependency and APIs, resolved by executable policy.
/// A package selector is not an identity proof; the executor must resolve one
/// exact locked source and every actual definition before accepting delegation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferredProvider {
    /// Cargo package selector, including any dependency alias used by the consumer.
    pub package: String,
    /// Allowed provider API paths, including public reexports.
    pub entry_points: Vec<String>,
    /// Local consumer paths whose actual delegation must be checked.
    pub consumers: Vec<String>,
}

pub(super) fn provider(
    graph: &Graph,
    job: &Node<'_>,
    reached: &mut BTreeSet<String>,
) -> Result<Option<PreferredProvider>, Error> {
    let ids = targets(job, "preferredProvider")?;
    if ids.len() > 1 {
        return Err(E::Repeated("job preferredProvider".into()).into());
    }
    let Some(id) = ids.first() else {
        return Ok(None);
    };
    if !reached.insert(id.clone()) {
        return Err(E::Repeated(format!("provider {id}")).into());
    }
    let node = node_at(graph, id, "PreferredProvider")?;
    fields(&node, &["package", "entryPoint", "consumer"])?;
    Ok(Some(PreferredProvider {
        package: required_text(&node, "package")?,
        entry_points: nonempty(&node, "entryPoint")?,
        consumers: nonempty(&node, "consumer")?,
    }))
}

fn nonempty(node: &Node<'_>, key: &str) -> Result<Vec<String>, E> {
    let values = strings(node, key)?;
    if values.is_empty() {
        return Err(E::Missing(format!("provider {key}")));
    }
    Ok(values)
}

pub(super) fn patterns(
    graph: &Graph,
    root: &Node<'_>,
    reached: &mut BTreeSet<String>,
    jobs: &[LedgerJob],
) -> Result<Vec<LedgerPattern>, Error> {
    let mut result = Vec::new();
    let mut ids = BTreeSet::new();
    for node_id in targets(root, "pattern")? {
        if !reached.insert(node_id.clone()) {
            return Err(E::Repeated(format!("pattern {node_id}")).into());
        }
        let node = node_at(graph, &node_id, "Pattern")?;
        fields(
            &node,
            &[
                "patternId",
                "summary",
                "call",
                "path",
                "macro",
                "typePath",
                "literalRegex",
                "tokens",
            ],
        )?;
        let id = required_text(&node, "patternId")?;
        crate::RuleId::new(&id).map_err(|error| E::Value(error.to_string()))?;
        if !ids.insert(id.clone()) || crate::rule::BUILTIN_RULE_IDS.contains(&id.as_str()) {
            return Err(E::Value(format!(
                "pattern {id:?}: id must be unique and must not replace a built-in"
            ))
            .into());
        }
        let summary = required_text(&node, "summary")?;
        if summary.contains(['\r', '\n']) {
            return Err(E::Value(format!("pattern {id:?}: summary must be one line")).into());
        }
        result.push(LedgerPattern {
            id,
            summary,
            matcher: matcher(&node)?,
        });
    }
    let used: BTreeSet<_> = jobs.iter().flat_map(|job| job.patterns.iter()).collect();
    for id in &used {
        if !ids.contains(*id) {
            return Err(E::Value(format!("unknown pattern {id:?}")).into());
        }
    }
    for pattern in &result {
        if !used.contains(&pattern.id) {
            return Err(E::Value(format!("unused pattern {:?}", pattern.id)).into());
        }
    }
    result.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(result)
}

fn matcher(node: &Node<'_>) -> Result<PatternMatcher, E> {
    let mut selected = None;
    for key in [
        "call",
        "path",
        "macro",
        "typePath",
        "literalRegex",
        "tokens",
    ] {
        if node.values(&super::shape::property(key)).next().is_none() {
            continue;
        }
        if selected.is_some() {
            return Err(E::Value("pattern requires exactly one matcher".into()));
        }
        let source = required_text(node, key)?;
        selected = Some(match key {
            "call" => PatternMatcher::Call(source),
            "path" => PatternMatcher::Path(source),
            "macro" => PatternMatcher::Macro(source),
            "typePath" => PatternMatcher::Type(source),
            "tokens" => PatternMatcher::Tokens(source),
            "literalRegex" => {
                let compiled = ecma::compile(&source)
                    .map_err(|error| E::Value(format!("literalRegex: {error}")))?;
                PatternMatcher::LiteralRegex(NamePattern { source, compiled })
            }
            _ => unreachable!("closed matcher set"),
        });
    }
    selected.ok_or_else(|| E::Missing("pattern matcher".into()))
}
