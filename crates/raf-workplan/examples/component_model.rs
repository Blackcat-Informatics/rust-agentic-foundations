// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A consuming parcel service declares structure and interactions independently
//! of source identity and execution evidence. Run with `cargo run -p raf-workplan
//! --example component_model`. SVG, text and JSON share the native semantic model.

use raf_source::{MemoryTree, Workspace};
use raf_workplan::{
    Workplan,
    diagram::{DiagramKind, DiagramScope, NamedDiagramView},
};

pub(crate) const SOURCE: &str = "pub struct Parcel;\npub fn route() {}\npub fn store() {}\n";
pub(crate) const MODEL: &str = r"model:
  components:
    - id: delivery
      label: Delivery service
      kind: boundary
      responsibility: Own parcel delivery
    - id: router
      label: Router
      kind: component
      parent: delivery
      responsibility: Validate and route parcels
      bindings:
        code: [parcel::route]
        interfaces: [parcel::route]
    - id: storage
      label: Storage
      kind: component
      parent: delivery
      responsibility: Retain accepted parcels
      bindings:
        code: [parcel::store]
        capabilities: [cap:routing]
  actors:
    - id: customer
      label: Customer
      responsibility: Submit parcels and receive decisions
  relations:
    - id: submit
      from: customer
      to: router
      label: Submit parcel
      protocol: HTTPS
      artifact: parcel
      claim: declared
    - id: persist
      from: router
      to: storage
      label: Store accepted parcel
      protocol: Repository interface
      artifact: stored-parcel
    - id: respond
      from: router
      to: customer
      label: Report decision
      protocol: HTTPS response
  artifacts:
    - id: parcel
      label: Submitted parcel
      type: Parcel
      producers: [customer]
      consumers: [router]
      bindings:
        interfaces: [parcel::Parcel]
    - id: stored-parcel
      label: Accepted parcel
      type: Parcel
      producers: [router]
      consumers: [storage]
  scenarios:
    - id: deliver
      label: Accept or reject a submitted parcel
      claim: declared
      participants: [customer, router, storage]
      regions:
        - id: decision
          kind: alternative
          order: 1
          label: Validation decision
        - id: accepted
          parent: decision
          kind: branch
          order: 0
          label: Parcel is valid
        - id: rejected
          parent: decision
          kind: error
          order: 1
          label: Parcel is invalid
        - id: fulfill
          parent: accepted
          kind: parallel
          order: 0
          label: Declared independent effects
        - id: store-lane
          parent: fulfill
          kind: lane
          order: 0
          label: Persistence
        - id: reply-lane
          parent: fulfill
          kind: lane
          order: 1
          label: Customer reply
      steps:
        - id: receive
          relation: submit
          order: 0
          label: Receive parcel
          bindings:
            code: [parcel::route]
            tests: [parcel::test_crate::flow::case]
        - id: save
          relation: persist
          region: store-lane
          order: 0
          label: Persist valid parcel
          bindings:
            code: [parcel::store]
        - id: confirm
          relation: respond
          region: reply-lane
          order: 0
          label: Confirm acceptance
        - id: reject
          relation: respond
          region: rejected
          order: 0
          label: Explain validation error
";

pub(crate) fn document(model: &str) -> String {
    format!(
        "---\ntype: Capability\nid: cap:routing\ntitle: Parcel routing\ndescription: Declared parcel delivery architecture.\nstate: implementing\nbindings:\n  code: [parcel::Parcel, parcel::route, parcel::store]\n  tests: [parcel::test_crate::flow::case]\n  examples: []\ndepends: []\n{model}---\n\n# Parcel routing\n\n## Purpose\n\nExplain parcel routing.\n\n## Contract\n\nComponents bind source identities. Scenario steps, branch conditions and parallel lanes are declared; the associated test is no interaction witness.\n\n## Usage\n\nSelect cap:routing for components or artifacts and cap:routing#deliver for a scenario.\n\n## Examples\n\nThe delivery scenario names success and error alternatives.\n\n## Constraints\n\nNo runtime sequence is inferred.\n\n## Explanation\n\nTyped artifacts declare their producers and consumers.\n\n## Verification\n\nThe associated test checks a separate implementation behavior.\n"
    )
}

pub(crate) fn with_plan<R>(
    document: &str,
    source: &str,
    run: impl FnOnce(&Workplan<'_>) -> R,
) -> R {
    let files = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname = \"parcel\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        ),
        ("src/lib.rs", source),
        ("tests/flow.rs", "#[test]\nfn case() {}\n"),
    ]);
    let workspace = Workspace::load(&files, &["."]).expect("the consuming source loads");
    let docs = MemoryTree::from([("docs/capabilities/routing.md", document)]);
    run(&Workplan::load(&docs, &workspace))
}

pub(crate) fn main() -> Result<(), Box<dyn std::error::Error>> {
    with_plan(&document(MODEL), SOURCE, |plan| {
        let view = NamedDiagramView::new(
            "delivery",
            DiagramScope::Automatic {
                kind: DiagramKind::Components,
                scope: "cap:routing".into(),
            },
        )?;
        let bundle = plan.diagram(&view)?;
        println!("{}", bundle.text());
        for detail in bundle
            .details()
            .iter()
            .filter(|detail| detail.view().kind() == DiagramKind::Scenario)
        {
            println!("{}", detail.text());
        }
        let provenance = bundle
            .details()
            .iter()
            .find(|detail| detail.view().kind() == DiagramKind::ModelProvenance)
            .ok_or("the complete provenance detail is required")?;
        println!(
            "Full RDF provenance: {} ({})",
            provenance.view().name(),
            provenance
                .full_model_hash()
                .ok_or("the full-model hash is required")?
        );
        Ok(())
    })
}
