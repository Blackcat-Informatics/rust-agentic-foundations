// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed authoring shapes; source and cross-reference checks belong to Workplan.

use super::{
    ArchitectureModel, ArtifactFlow, Component, ComponentKind, DeclaredRelation, ExternalActor,
    ModelBindings, Scenario, ScenarioRegion, ScenarioRegionKind, ScenarioStep,
};
use crate::profile::{Check, is_capability_id, is_kebab};
use crate::{Entry, ItemValue, Map, Rule, Scalar};
use std::collections::HashSet;

struct Invalid {
    path: String,
    expected: &'static str,
}

type Read<T> = Result<T, Invalid>;

struct Row<'a> {
    map: &'a Map,
    path: String,
}

impl Row<'_> {
    fn invalid(&self, key: &str, expected: &'static str) -> Invalid {
        Invalid {
            path: format!("{}.{key}", self.path),
            expected,
        }
    }

    fn closed(&self, keys: &[&str]) -> Read<()> {
        if let Some(entry) = self
            .map
            .entries()
            .iter()
            .find(|entry| !keys.contains(&entry.key()))
        {
            return Err(self.invalid(entry.key(), "a key from this closed model row"));
        }
        Ok(())
    }

    fn optional(&self, key: &str) -> Read<Option<String>> {
        self.map
            .get(key)
            .map(|entry| {
                entry
                    .value()
                    .as_scalar()
                    .map(Scalar::text)
                    .filter(|text| !text.trim().is_empty() && text.len() <= 4096)
                    .map(str::to_owned)
                    .ok_or_else(|| self.invalid(key, "a nonempty scalar of at most 4096 bytes"))
            })
            .transpose()
    }

    fn text(&self, key: &str) -> Read<String> {
        self.optional(key)?
            .ok_or_else(|| self.invalid(key, "a required nonempty scalar"))
    }

    fn id(&self, key: &str) -> Read<String> {
        let id = self.text(key)?;
        if !is_kebab(&id) {
            return Err(self.invalid(key, "a local lowercase kebab identifier"));
        }
        Ok(id)
    }

    fn optional_id(&self, key: &str) -> Read<Option<String>> {
        self.map.get(key).map(|_| self.id(key)).transpose()
    }

    fn strings(&self, key: &str) -> Read<Vec<String>> {
        let Some(entry) = self.map.get(key) else {
            return Ok(Vec::new());
        };
        let list = entry
            .value()
            .as_list()
            .filter(|list| list.items().len() <= 256)
            .ok_or_else(|| self.invalid(key, "at most 256 distinct nonempty strings"))?;
        let mut found = Vec::new();
        let mut seen = HashSet::new();
        for item in list.items() {
            let value = item
                .value()
                .as_scalar()
                .map(Scalar::text)
                .filter(|value| !value.trim().is_empty() && value.len() <= 4096)
                .ok_or_else(|| {
                    self.invalid(key, "distinct nonempty strings of at most 4096 bytes")
                })?;
            if !seen.insert(value) {
                return Err(self.invalid(key, "distinct strings; remove the duplicate"));
            }
            found.push(value.to_owned());
        }
        Ok(found)
    }

    fn ids(&self, key: &str) -> Read<Vec<String>> {
        let values = self.strings(key)?;
        if values.iter().any(|id| !is_kebab(id)) {
            return Err(self.invalid(key, "distinct local lowercase kebab identifiers"));
        }
        Ok(values)
    }

    fn evidence(&self) -> Read<Vec<String>> {
        let values = self.strings("evidence")?;
        if values
            .iter()
            .any(|id| !id.strip_prefix("evidence:").is_some_and(is_kebab))
        {
            return Err(self.invalid(
                "evidence",
                "evidence:<slug> verification associations; these do not observe interactions",
            ));
        }
        Ok(values)
    }

    fn declared(&self) -> Read<()> {
        if self
            .optional("claim")?
            .is_some_and(|claim| claim != "declared")
        {
            return Err(self.invalid("claim", "declared; observed interactions/order/branches/concurrency require a native execution witness, which is unavailable"));
        }
        Ok(())
    }

    fn order(&self) -> Read<u32> {
        let value = self.text("order")?;
        value
            .parse()
            .ok()
            .filter(|order| *order < 4096 && value.bytes().all(|byte| byte.is_ascii_digit()))
            .ok_or_else(|| {
                self.invalid(
                    "order",
                    "an explicit zero-based sibling position below 4096",
                )
            })
    }

    fn rows<T>(&self, key: &str, parse: fn(&Row<'_>) -> Read<T>) -> Read<Vec<T>> {
        let Some(entry) = self.map.get(key) else {
            return Ok(Vec::new());
        };
        let list = entry
            .value()
            .as_list()
            .filter(|list| list.items().len() <= 256)
            .ok_or_else(|| self.invalid(key, "at most 256 closed model rows"))?;
        list.items()
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let ItemValue::Map(map) = item.value() else {
                    return Err(self.invalid(key, "a list of closed model maps"));
                };
                parse(&Row {
                    map,
                    path: format!("{}.{key}[{index}]", self.path),
                })
            })
            .collect()
    }

    fn bindings(&self) -> Read<ModelBindings> {
        let Some(entry) = self.map.get("bindings") else {
            return Ok(ModelBindings::default());
        };
        let row = Row {
            map: entry
                .value()
                .as_map()
                .ok_or_else(|| self.invalid("bindings", "a closed binding map"))?,
            path: format!("{}.bindings", self.path),
        };
        row.closed(&["code", "interfaces", "capabilities", "tests", "examples"])?;
        let capabilities = row.strings("capabilities")?;
        if capabilities.iter().any(|id| !is_capability_id(id)) {
            return Err(row.invalid("capabilities", "cap:<slug> identifiers"));
        }
        Ok(ModelBindings {
            code: row.strings("code")?,
            interfaces: row.strings("interfaces")?,
            capabilities,
            tests: row.strings("tests")?,
            examples: row.strings("examples")?,
        })
    }
}

fn component(row: &Row<'_>) -> Read<Component> {
    row.closed(&[
        "id",
        "label",
        "kind",
        "parent",
        "responsibility",
        "bindings",
    ])?;
    let kind = match row.text("kind")?.as_str() {
        "component" => ComponentKind::Component,
        "boundary" => ComponentKind::Boundary,
        _ => return Err(row.invalid("kind", "component or boundary")),
    };
    let bindings = row.bindings()?;
    if kind == ComponentKind::Component
        && bindings.code.is_empty()
        && bindings.interfaces.is_empty()
        && bindings.capabilities.is_empty()
    {
        return Err(row.invalid(
            "bindings",
            "a component needs implementation, public-interface or capability bindings",
        ));
    }
    Ok(Component {
        id: row.id("id")?,
        label: row.text("label")?,
        kind,
        parent: row.optional_id("parent")?,
        responsibility: row.text("responsibility")?,
        bindings,
    })
}

fn actor(row: &Row<'_>) -> Read<ExternalActor> {
    row.closed(&["id", "label", "responsibility"])?;
    Ok(ExternalActor {
        id: row.id("id")?,
        label: row.text("label")?,
        responsibility: row.text("responsibility")?,
    })
}

fn relation(row: &Row<'_>) -> Read<DeclaredRelation> {
    row.closed(&["id", "from", "to", "label", "protocol", "artifact", "claim"])?;
    row.declared()?;
    Ok(DeclaredRelation {
        id: row.id("id")?,
        from: row.id("from")?,
        to: row.id("to")?,
        label: row.text("label")?,
        protocol: row.optional("protocol")?,
        artifact: row.optional_id("artifact")?,
    })
}

fn artifact(row: &Row<'_>) -> Read<ArtifactFlow> {
    row.closed(&[
        "id",
        "label",
        "type",
        "producers",
        "consumers",
        "bindings",
        "claim",
    ])?;
    row.declared()?;
    let producers = row.ids("producers")?;
    let consumers = row.ids("consumers")?;
    if producers.is_empty() {
        return Err(row.invalid(
            "producers",
            "an artifact flow needs explicit producers and consumers",
        ));
    }
    if consumers.is_empty() {
        return Err(row.invalid(
            "consumers",
            "an artifact flow needs explicit producers and consumers",
        ));
    }
    Ok(ArtifactFlow {
        id: row.id("id")?,
        label: row.text("label")?,
        artifact_type: row.text("type")?,
        producers,
        consumers,
        bindings: row.bindings()?,
    })
}

fn region(row: &Row<'_>) -> Read<ScenarioRegion> {
    row.closed(&["id", "parent", "order", "kind", "label", "claim"])?;
    row.declared()?;
    let kind = match row.text("kind")?.as_str() {
        "sequence" => ScenarioRegionKind::Sequence,
        "alternative" => ScenarioRegionKind::Alternative,
        "branch" => ScenarioRegionKind::Branch,
        "error" => ScenarioRegionKind::Error,
        "parallel" => ScenarioRegionKind::Parallel,
        "lane" => ScenarioRegionKind::Lane,
        _ => {
            return Err(row.invalid(
                "kind",
                "sequence, alternative, branch, error, parallel or lane",
            ));
        }
    };
    Ok(ScenarioRegion {
        id: row.id("id")?,
        parent: row.optional_id("parent")?,
        order: row.order()?,
        kind,
        label: row.text("label")?,
    })
}

fn step(row: &Row<'_>) -> Read<ScenarioStep> {
    row.closed(&[
        "id", "relation", "region", "order", "label", "bindings", "evidence", "claim",
    ])?;
    row.declared()?;
    Ok(ScenarioStep {
        id: row.id("id")?,
        relation: row.id("relation")?,
        region: row.optional_id("region")?,
        order: row.order()?,
        label: row.text("label")?,
        bindings: row.bindings()?,
        evidence: row.evidence()?,
    })
}

fn scenario(row: &Row<'_>) -> Read<Scenario> {
    row.closed(&[
        "id",
        "label",
        "participants",
        "regions",
        "steps",
        "evidence",
        "claim",
    ])?;
    row.declared()?;
    let participants = row.ids("participants")?;
    let steps = row.rows("steps", step)?;
    if participants.is_empty() {
        return Err(row.invalid(
            "participants",
            "a scenario needs participants and declared interaction steps",
        ));
    }
    if steps.is_empty() {
        return Err(row.invalid(
            "steps",
            "a scenario needs participants and declared interaction steps",
        ));
    }
    Ok(Scenario {
        id: row.id("id")?,
        label: row.text("label")?,
        participants,
        regions: row.rows("regions", region)?,
        steps,
        evidence: row.evidence()?,
    })
}

fn model(entry: &Entry) -> Read<ArchitectureModel> {
    let row = Row {
        map: entry.value().as_map().ok_or_else(|| Invalid {
            path: "model".into(),
            expected: "a closed architecture model map",
        })?,
        path: "model".into(),
    };
    row.closed(&[
        "components",
        "actors",
        "relations",
        "artifacts",
        "scenarios",
    ])?;
    Ok(ArchitectureModel {
        components: row.rows("components", component)?,
        actors: row.rows("actors", actor)?,
        relations: row.rows("relations", relation)?,
        artifacts: row.rows("artifacts", artifact)?,
        scenarios: row.rows("scenarios", scenario)?,
    })
}

pub(in crate::profile) fn check(check: &mut Check<'_>, entry: &Entry) -> Option<ArchitectureModel> {
    match model(entry) {
        Ok(model) => Some(model),
        Err(invalid) => {
            check.key(
                &invalid.path,
                Rule::WrongShape {
                    expected: invalid.expected,
                },
            );
            None
        }
    }
}
