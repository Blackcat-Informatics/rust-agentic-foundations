// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicit semantic identity records, independent of Rust debug formatting.

use purrdf_lex::json::{self, Object, Value};
use raf_source::{
    ActiveConfiguration, CfgSelection, LockedBinding, LockedGitReference, LockedSource,
    LockedSourceKind, TargetKind,
};

use super::{DiagramError, NODES, NamedDiagramView, Relation, hash};

pub(super) fn selection_iri(view: &NamedDiagramView) -> String {
    let scope = match &view.scope {
        crate::diagram::DiagramScope::Automatic { scope, .. } => scope.as_str(),
        crate::diagram::DiagramScope::Evidence { id } => id.as_str(),
    };
    let value = Value::from(
        Object::new()
            .with("profile", "raf-diagram-selection-1")
            .with("owner", view.owner.as_deref())
            .with("name", view.name.as_str())
            .with("kind", view.kind().as_str())
            .with("scope", scope)
            .with("grouping", view.grouping.as_str())
            .with("focus", view.focus.as_slice())
            .with("relations", view.relations.as_slice()),
    );
    iri("selection", &value)
}

pub(super) fn relation_iri(relation: &Relation) -> String {
    let fields: Vec<_> = relation
        .fields
        .iter()
        .map(|(key, value)| vec![key.as_str(), value.as_str()])
        .collect();
    let value = Value::from(
        Object::new()
            .with("profile", "raf-diagram-relation-1")
            .with("from", relation.from.as_str())
            .with("kind", relation.kind.as_str())
            .with("to", relation.to.as_str())
            .with("fields", fields),
    );
    iri("relation", &value)
}

pub(super) fn locked_package_iri(binding: &LockedBinding) -> String {
    let value = Value::from(
        Object::new()
            .with("profile", "raf-diagram-locked-package-1")
            .with("name", binding.name())
            .with("version", binding.version())
            .with("source", binding.source().map(source_value)),
    );
    iri("locked-package", &value)
}

fn iri(family: &str, value: &Value) -> String {
    format!("{NODES}{family}/{}", hash(&json::write_compact(value)))
}

pub(super) fn source_value(source: &LockedSource) -> Value {
    Value::from(
        Object::new()
            .with("kind", source_kind(source.kind()))
            .with("repository", source.repository())
            .with("reference", source.reference().map(reference_value))
            .with("revision", source.revision()),
    )
}

pub(super) fn source_kind(kind: LockedSourceKind) -> &'static str {
    match kind {
        LockedSourceKind::Git => "git",
        LockedSourceKind::Registry => "registry",
        LockedSourceKind::SparseRegistry => "sparse-registry",
        LockedSourceKind::Path => "path",
        LockedSourceKind::LocalRegistry => "local-registry",
        LockedSourceKind::Directory => "directory",
    }
}

pub(super) fn reference_value(reference: &LockedGitReference) -> Value {
    let (kind, value) = match reference {
        LockedGitReference::DefaultBranch => ("default-branch", None),
        LockedGitReference::Branch(value) => ("branch", Some(value.as_str())),
        LockedGitReference::Tag(value) => ("tag", Some(value.as_str())),
        LockedGitReference::Revision(value) => ("revision", Some(value.as_str())),
    };
    Value::from(Object::new().with("kind", kind).with("value", value))
}

pub(super) fn configuration_value(
    configuration: &ActiveConfiguration,
) -> Result<Value, DiagramError> {
    Ok(Value::from(
        Object::new()
            .with("target", cfg_value(configuration.target()))
            .with("host", configuration.host().map(cfg_value))
            .with("features", configuration.features())
            .with(
                "optional_dependencies",
                configuration.optional_dependencies(),
            )
            .with("target_kind", target_kind(configuration.target_kind())?)
            .with("target_name", configuration.target_name()),
    ))
}

fn cfg_value(configuration: &CfgSelection) -> Value {
    Value::from(
        Object::new()
            .with("triple", configuration.triple())
            .with("predicates", configuration.predicates()),
    )
}

pub(super) fn target_kind(kind: TargetKind) -> Result<&'static str, DiagramError> {
    Ok(match kind {
        TargetKind::Lib => "lib",
        TargetKind::Bin => "bin",
        TargetKind::Test => "test",
        TargetKind::Bench => "bench",
        TargetKind::Example => "example",
        TargetKind::BuildScript => "build-script",
        _ => {
            return Err(DiagramError::refused(
                "diagram identity cannot encode this source target kind",
            ));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagram::{DiagramGrouping, DiagramKind, DiagramScope};

    #[test]
    fn persisted_identity_profiles_pin_field_boundaries_and_escaping() {
        let view = NamedDiagramView::new(
            "dependencies",
            DiagramScope::Automatic {
                kind: DiagramKind::CapabilityDependencies,
                scope: "workspace".into(),
            },
        )
        .unwrap()
        .with_grouping(DiagramGrouping::Capability)
        .with_focus(vec!["cap:alpha".into()])
        .with_relations(vec!["dependsOn".into()]);
        let selection = r#"{"profile":"raf-diagram-selection-1","owner":null,"name":"dependencies","kind":"capability-dependencies","scope":"workspace","grouping":"capability","focus":["cap:alpha"],"relations":["dependsOn"]}"#;
        assert_eq!(
            selection_iri(&view),
            format!("{NODES}selection/{}", hash(selection))
        );
        let relation = Relation {
            from: "urn:example:from".into(),
            kind: "dependsOn".into(),
            to: "urn:example:to".into(),
            fields: vec![("note".into(), "a|b".into()), ("note".into(), "c".into())],
        };
        let encoded = r#"{"profile":"raf-diagram-relation-1","from":"urn:example:from","kind":"dependsOn","to":"urn:example:to","fields":[["note","a|b"],["note","c"]]}"#;
        assert_eq!(
            relation_iri(&relation),
            format!("{NODES}relation/{}", hash(encoded))
        );
        let reference = LockedGitReference::Branch("quoted\"branch".into());
        assert_eq!(
            json::write_compact(&reference_value(&reference)),
            r#"{"kind":"branch","value":"quoted\"branch"}"#
        );
    }
}
