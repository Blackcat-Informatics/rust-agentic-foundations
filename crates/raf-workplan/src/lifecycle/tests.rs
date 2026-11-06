// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Verification judges declarations without executing their test or example bodies.

use super::verification::verify_plan;
use crate::Workplan;
use raf_source::{MemoryTree, Workspace};

#[test]
fn verification_returns_static_bindings_without_running_their_bodies() {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n",
        ),
        (
            "src/lib.rs",
            "/// Declared API.\npub fn api() {}\n#[test]\nfn dangerous() { panic!(\"must never execute\"); }\n",
        ),
        (
            "examples/destructive.rs",
            "fn main() { panic!(\"must never execute\"); }",
        ),
    ]);
    let source = Workspace::load(&tree, &["."]).unwrap();
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
    let capability = format!(
        "---\ntype: Capability\nid: cap:demo\ntitle: Demo\ndescription: Demo.\nstate: implementing\nbindings:\n  code: [demo::api]\n  tests: [demo::dangerous]\n  examples: [demo::example_crate::destructive]\ndepends: []\n---\n\n{sections}"
    );
    let documents = MemoryTree::try_from_iter([("docs/capabilities/demo.md", capability)]).unwrap();
    let plan = Workplan::load(&documents, &source);
    assert_eq!(plan.capabilities().len(), 1, "{:?}", plan.findings());
    let report = verify_plan(&plan, None).unwrap();
    assert!(
        report
            .bindings()
            .iter()
            .any(|binding| binding.path() == "demo::dangerous")
    );
    assert!(
        report
            .bindings()
            .iter()
            .any(|binding| binding.path() == "demo::example_crate::destructive")
    );
    assert!(report.lines().iter().all(|line| !line.contains("pass")));
}
