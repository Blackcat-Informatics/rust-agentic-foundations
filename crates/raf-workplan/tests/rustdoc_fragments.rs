// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static include generation requires public, exact source and package identities.

use raf_source::{DiskTree, MemoryTree, Workspace};
use raf_workplan::{
    Workplan,
    diagram::{
        DiagramKind, DiagramScope, NamedDiagramView,
        cargo::Library,
        rustdoc::{Context, fragment},
    },
};

#[test]
fn fragments_refuse_private_or_incomplete_bindings_beside_a_public_reference() {
    let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let root = scratch.path().canonicalize().unwrap();
    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/lib.rs"),
        "//! Crate docs.\n/// API.\npub fn api() {}\n/// Private.\nfn private() {}\n",
    )
    .unwrap();
    let source = Workspace::load(&DiskTree::new(&root).unwrap(), &["."]).unwrap();
    let target = source
        .target_ids()
        .find(|id| source.target(*id).unwrap().crate_path == "demo")
        .unwrap();
    let library = Library {
        package_id: "opaque-demo".into(),
        package: "demo".into(),
        version: "1.0.0".into(),
        manifest: root.join("Cargo.toml"),
        name: "demo".into(),
        kind: vec!["lib".into()],
        crate_types: vec!["lib".into()],
        source: root.join("src/lib.rs"),
        features: Vec::new(),
    };
    let sections = [
        "Purpose",
        "Contract",
        "Usage",
        "Examples",
        "Constraints",
        "Explanation",
        "Verification",
    ]
    .map(|name| format!("## {name}\n\nDeclared behavior.\n\n"))
    .join("");
    for (binding, valid) in [("demo::api", true), ("demo::private", false)] {
        let text = format!(
            "---\ntype: Capability\nid: cap:demo\ntitle: Demo\ndescription: Demo.\nstate: implementing\nbindings:\n  code: [{binding}]\n  tests: []\n  examples: []\ndepends: []\n---\n\n{sections}"
        );
        let documents = MemoryTree::try_from_iter([("docs/capabilities/demo.md", text)]).unwrap();
        let plan = Workplan::load(&documents, &source);
        let view = NamedDiagramView::new(
            "api",
            DiagramScope::Automatic {
                kind: DiagramKind::Ownership,
                scope: "cap:demo".into(),
            },
        )
        .unwrap();
        let bundle = plan.diagram(&view).unwrap();
        let required = vec![binding.to_owned()];
        let context = |required| Context {
            target,
            module: "demo",
            capability: "cap:demo",
            required,
            library: &library,
            root: &root,
            metadata: "imported-metadata-digest",
        };
        let generated = fragment(&plan, &source, &bundle, context(&required));
        assert_eq!(generated.is_ok(), valid, "{generated:?}");
        assert!(fragment(&plan, &source, &bundle, context(&[])).is_err());
        if let Ok(text) = generated {
            assert!(text.contains("crate::api"), "{text}");
            assert!(text.contains("imported Cargo package"), "{text}");
        }
    }
}
