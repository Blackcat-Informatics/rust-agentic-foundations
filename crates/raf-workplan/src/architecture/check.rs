// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! RAF authoring/reference policy over native source and public-export queries.

use super::AuthoredModel;
use crate::{Finding, Workplan, finding::kind};
use raf_okf::{Location, ModelBindings, Scenario};
use raf_source::TestRole;
use std::collections::{BTreeMap, BTreeSet};

struct Check<'a, 's> {
    plan: &'a Workplan<'s>,
    model: AuthoredModel<'a>,
    findings: Vec<Finding>,
}

impl Check<'_, '_> {
    fn reject(&mut self, key: impl Into<String>, reason: impl Into<String>) {
        self.findings.push(Finding::new(kind::MODEL_INVALID, self.model.path,
            Location::Key(key.into()), reason.into(),
            "repair the named model declaration or binding; source identity and verification associations never establish observed interactions"));
    }

    fn bindings(&mut self, key: &str, bindings: &ModelBindings) {
        for (position, capability) in bindings.capabilities.iter().enumerate() {
            let Some(target) = self
                .plan
                .capabilities()
                .iter()
                .find(|cap| &cap.value().header.id == capability)
            else {
                self.reject(
                    format!("{key}.capabilities[{position}]"),
                    format!("capability `{capability}` did not load"),
                );
                continue;
            };
            if let Some(finding) = self.plan.findings().iter().find(|finding| {
                finding.file == target.path()
                    && matches!(
                        finding.kind,
                        kind::BINDING_UNRESOLVED
                            | kind::BINDING_AMBIGUOUS
                            | kind::BINDING_WRONG_ROLE
                            | kind::BINDING_MACRO_GENERATED
                    )
            }) {
                self.reject(
                    format!("{key}.capabilities[{position}]"),
                    format!(
                        "capability `{capability}` has an invalid native binding in {}: {}",
                        finding.file, finding.message
                    ),
                );
            }
        }
        for (role, paths) in [
            ("code", &bindings.code),
            ("interfaces", &bindings.interfaces),
            ("tests", &bindings.tests),
            ("examples", &bindings.examples),
        ] {
            for (position, path) in paths.iter().enumerate() {
                let key = format!("{key}.{role}[{position}]");
                let source = self.plan.source();
                let resolution = match source.resolve(path) {
                    Ok(resolution) => resolution,
                    Err(error) => {
                        self.reject(key, format!("binding `{path}` does not resolve: {error}"));
                        continue;
                    }
                };
                let [id] = resolution.sites.as_slice() else {
                    self.reject(key, format!("binding `{path}` needs one source definition; select a unique configured source path"));
                    continue;
                };
                let Some(item) = source.item(*id) else {
                    self.reject(
                        key,
                        format!("resolved binding `{path}` has no indexed definition"),
                    );
                    continue;
                };
                let accepted = match role {
                    "code" => source.is_shipping(item),
                    "tests" => item.test == TestRole::TestFn,
                    "examples" => crate::bindings::example_kind(source, item).is_some(),
                    "interfaces" => match source.exported_items(item.target) {
                        Ok(exports) => exports.items.iter().any(|export| {
                            export.item == *id && export.path == *path && !export.doc_hidden
                        }),
                        Err(error) => {
                            self.reject(
                                key,
                                format!("public interface `{path}` cannot be enumerated: {error}"),
                            );
                            continue;
                        }
                    },
                    _ => unreachable!("the role comes from the closed binding table"),
                };
                if !accepted {
                    self.reject(key, format!("binding `{path}` is not an admitted {role} identity; use the actual shipping definition, public export, test or example"));
                }
            }
        }
    }

    fn evidence(&mut self, key: &str, ids: &[String]) {
        for id in ids {
            if !self
                .plan
                .evidence()
                .iter()
                .any(|record| record.value().header.id == *id)
            {
                self.reject(
                    key,
                    format!(
                        "verification record `{id}` did not load; record or repair that association"
                    ),
                );
            }
        }
    }

    fn model(&mut self) {
        let model = self.model.declarations;
        let mut participants = BTreeSet::new();
        let components: BTreeMap<_, _> = model
            .components
            .iter()
            .map(|component| (component.id.as_str(), component))
            .collect();
        for (index, component) in model.components.iter().enumerate() {
            let key = format!("model.components[{index}]");
            if !participants.insert(component.id.as_str()) {
                self.reject(&key, format!("duplicate participant `{}`", component.id));
            }
            self.bindings(&format!("{key}.bindings"), &component.bindings);
            if let Some(parent) = &component.parent
                && !components.contains_key(parent.as_str())
            {
                self.reject(
                    format!("{key}.parent"),
                    format!(
                        "parent component `{parent}` is absent; actors cannot contain components"
                    ),
                );
            }
        }
        for (index, actor) in model.actors.iter().enumerate() {
            if !participants.insert(actor.id.as_str()) {
                self.reject(
                    format!("model.actors[{index}].id"),
                    format!("duplicate participant `{}`", actor.id),
                );
            }
        }
        let mut artifacts = BTreeMap::new();
        for (index, artifact) in model.artifacts.iter().enumerate() {
            let key = format!("model.artifacts[{index}]");
            if artifacts.insert(artifact.id.as_str(), artifact).is_some() {
                self.reject(&key, format!("duplicate artifact `{}`", artifact.id));
            }
            self.bindings(&format!("{key}.bindings"), &artifact.bindings);
            for id in artifact.producers.iter().chain(&artifact.consumers) {
                if !participants.contains(id.as_str()) {
                    self.reject(&key, format!("artifact participant `{id}` is absent"));
                }
            }
        }
        let mut relations = BTreeMap::new();
        for (index, relation) in model.relations.iter().enumerate() {
            let key = format!("model.relations[{index}]");
            if relations.insert(relation.id.as_str(), relation).is_some() {
                self.reject(&key, format!("duplicate relation `{}`", relation.id));
            }
            if !participants.contains(relation.from.as_str())
                || !participants.contains(relation.to.as_str())
            {
                self.reject(
                    &key,
                    "relation endpoints must name declared components or actors",
                );
            }
            if let Some(artifact) = &relation.artifact
                && !artifacts.get(artifact.as_str()).is_some_and(|flow| {
                    flow.producers.contains(&relation.from) && flow.consumers.contains(&relation.to)
                })
            {
                self.reject(format!("{key}.artifact"), format!("artifact `{artifact}` must name this relationship's sender as producer and receiver as consumer"));
            }
        }
        let mut scenarios = BTreeSet::new();
        for (index, scenario) in model.scenarios.iter().enumerate() {
            let key = format!("model.scenarios[{index}]");
            if !scenarios.insert(&scenario.id) {
                self.reject(&key, format!("duplicate scenario `{}`", scenario.id));
            }
            for participant in &scenario.participants {
                if !participants.contains(participant.as_str()) {
                    self.reject(
                        &key,
                        format!("scenario participant `{participant}` is absent"),
                    );
                }
            }
            self.evidence(&format!("{key}.evidence"), &scenario.evidence);
            self.scenario(&key, scenario, &relations);
        }
    }

    fn scenario(
        &mut self,
        key: &str,
        scenario: &Scenario,
        relations: &BTreeMap<&str, &raf_okf::DeclaredRelation>,
    ) {
        let regions: BTreeMap<_, _> = scenario
            .regions
            .iter()
            .map(|region| (region.id.as_str(), region))
            .collect();
        let mut ids = BTreeSet::new();
        for (index, region) in scenario.regions.iter().enumerate() {
            let site = format!("{key}.regions[{index}]");
            if !ids.insert(region.id.as_str()) {
                self.reject(&site, "region and step identifiers must be distinct");
            }
            if let Some(parent) = &region.parent
                && !regions.contains_key(parent.as_str())
            {
                self.reject(&site, format!("region parent `{parent}` is absent"));
            }
        }
        for (index, step) in scenario.steps.iter().enumerate() {
            let site = format!("{key}.steps[{index}]");
            if !ids.insert(step.id.as_str()) {
                self.reject(&site, "region and step identifiers must be distinct");
            }
            self.bindings(&format!("{site}.bindings"), &step.bindings);
            self.evidence(&format!("{site}.evidence"), &step.evidence);
            match relations.get(step.relation.as_str()) {
                Some(relation)
                    if scenario.participants.contains(&relation.from)
                        && scenario.participants.contains(&relation.to) => {}
                _ => self.reject(
                    format!("{site}.relation"),
                    "step relationship must exist and both endpoints must be scenario participants",
                ),
            }
            if let Some(region) = &step.region
                && !regions.contains_key(region.as_str())
            {
                self.reject(&site, format!("step region `{region}` is absent"));
            }
        }
    }
}

pub(super) fn models(plan: &Workplan<'_>) -> Vec<Finding> {
    let mut findings = Vec::new();
    for model in plan.architecture_models() {
        let mut check = Check {
            plan,
            model,
            findings: Vec::new(),
        };
        check.model();
        if check.findings.is_empty()
            && let Err((key, error)) = crate::diagram::authored::validate(plan, model)
        {
            check.reject(key, error.to_string());
        }
        findings.extend(check.findings);
    }
    findings
}
