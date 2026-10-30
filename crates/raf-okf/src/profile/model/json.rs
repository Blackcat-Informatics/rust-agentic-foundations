// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Versioned native JSON projection for declaration-input currency.

use super::{
    ArchitectureModel, ArtifactFlow, Component, ComponentKind, DeclaredRelation, ExternalActor,
    ModelBindings, Scenario, ScenarioRegion, ScenarioRegionKind, ScenarioStep,
};
use purrdf_lex::{
    json::{Object, Value, record::ToJson},
    walk::try_sort_unstable_by,
};

fn ordered<T: ToJson>(values: &[T], key: impl Fn(&T) -> &str) -> Value {
    let mut rows: Vec<_> = values.iter().collect();
    try_sort_unstable_by(&mut rows, |a, b| {
        Ok::<_, std::convert::Infallible>(key(a).cmp(key(b)))
    })
    .unwrap_or_else(|never| match never {});
    Value::from(rows.into_iter().map(ToJson::to_json).collect::<Vec<_>>())
}

fn strings(values: &[String]) -> Value {
    ordered(values, String::as_str)
}

impl ToJson for crate::Bindings {
    fn to_json(&self) -> Value {
        Value::from(
            Object::new()
                .with("code", strings(&self.code))
                .with("tests", strings(&self.tests))
                .with("examples", strings(&self.examples)),
        )
    }
}

impl ToJson for ArchitectureModel {
    fn to_json(&self) -> Value {
        Value::from(
            Object::new()
                .with("profile", "raf-architecture-model-1")
                .with("components", ordered(&self.components, |row| &row.id))
                .with("actors", ordered(&self.actors, |row| &row.id))
                .with("relations", ordered(&self.relations, |row| &row.id))
                .with("artifacts", ordered(&self.artifacts, |row| &row.id))
                .with("scenarios", ordered(&self.scenarios, |row| &row.id)),
        )
    }
}

impl ToJson for ModelBindings {
    fn to_json(&self) -> Value {
        Value::from(
            Object::new()
                .with("code", strings(&self.code))
                .with("interfaces", strings(&self.interfaces))
                .with("capabilities", strings(&self.capabilities))
                .with("tests", strings(&self.tests))
                .with("examples", strings(&self.examples)),
        )
    }
}

impl ToJson for Component {
    fn to_json(&self) -> Value {
        Value::from(
            Object::new()
                .with("id", self.id.as_str())
                .with("label", self.label.as_str())
                .with(
                    "kind",
                    match self.kind {
                        ComponentKind::Component => "component",
                        ComponentKind::Boundary => "boundary",
                    },
                )
                .with("parent", self.parent.as_deref())
                .with("responsibility", self.responsibility.as_str())
                .with("bindings", self.bindings.to_json()),
        )
    }
}

impl ToJson for ExternalActor {
    fn to_json(&self) -> Value {
        Value::from(
            Object::new()
                .with("id", self.id.as_str())
                .with("label", self.label.as_str())
                .with("responsibility", self.responsibility.as_str()),
        )
    }
}

impl ToJson for DeclaredRelation {
    fn to_json(&self) -> Value {
        Value::from(
            Object::new()
                .with("id", self.id.as_str())
                .with("from", self.from.as_str())
                .with("to", self.to.as_str())
                .with("label", self.label.as_str())
                .with("protocol", self.protocol.as_deref())
                .with("artifact", self.artifact.as_deref())
                .with("claim", "declared"),
        )
    }
}

impl ToJson for ArtifactFlow {
    fn to_json(&self) -> Value {
        Value::from(
            Object::new()
                .with("id", self.id.as_str())
                .with("label", self.label.as_str())
                .with("type", self.artifact_type.as_str())
                .with("producers", strings(&self.producers))
                .with("consumers", strings(&self.consumers))
                .with("bindings", self.bindings.to_json())
                .with("claim", "declared"),
        )
    }
}

impl ScenarioRegionKind {
    /// Closed authoring and semantic spelling of this structural region.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sequence => "sequence",
            Self::Alternative => "alternative",
            Self::Branch => "branch",
            Self::Error => "error",
            Self::Parallel => "parallel",
            Self::Lane => "lane",
        }
    }
}

impl ToJson for ScenarioRegion {
    fn to_json(&self) -> Value {
        Value::from(
            Object::new()
                .with("id", self.id.as_str())
                .with("parent", self.parent.as_deref())
                .with("order", u64::from(self.order))
                .with("kind", self.kind.as_str())
                .with("label", self.label.as_str())
                .with("claim", "declared"),
        )
    }
}

impl ToJson for ScenarioStep {
    fn to_json(&self) -> Value {
        Value::from(
            Object::new()
                .with("id", self.id.as_str())
                .with("relation", self.relation.as_str())
                .with("region", self.region.as_deref())
                .with("order", u64::from(self.order))
                .with("label", self.label.as_str())
                .with("bindings", self.bindings.to_json())
                .with("evidence", strings(&self.evidence))
                .with("claim", "declared"),
        )
    }
}

impl ToJson for Scenario {
    fn to_json(&self) -> Value {
        Value::from(
            Object::new()
                .with("id", self.id.as_str())
                .with("label", self.label.as_str())
                .with("participants", self.participants.as_slice())
                .with("regions", ordered(&self.regions, |row| &row.id))
                .with("steps", ordered(&self.steps, |row| &row.id))
                .with("evidence", strings(&self.evidence))
                .with("claim", "declared"),
        )
    }
}
