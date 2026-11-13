// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Repository navigation through the public immutable publisher.

use super::{
    component_model,
    support::disk::{Scratch, put as write, snapshot},
};
use purrdf_core::BaseIri;
use purrdf_lex::json::{self, record::FromJson};
use purrdf_rdf::viz::VizPresentedExport;
use raf_workplan::{Gate, diagram::publication};
use std::{fs, path::Path};

fn fixture(root: &Path) {
    write(
        root,
        "Cargo.toml",
        "[package]\nname = 'parcel'\nversion = '0.1.0'\nedition = '2024'\n",
    );
    write(
        root,
        "src/lib.rs",
        "#[path = \"escaped # café.rs\"] mod implementation;\npub use implementation::*;\n",
    );
    write(root, "src/escaped # café.rs", component_model::SOURCE);
    write(root, "tests/flow.rs", "#[test]\nfn case() {}\n");
    let model = component_model::MODEL.replace(
        "responsibility: Own parcel delivery",
        "responsibility: Own parcel delivery\n      bindings:\n        capabilities: [cap:empty]",
    );
    let declarations = "diagrams:\n  - name: components\n    kind: components\n    scope: cap:routing\n  - name: artifacts\n    kind: artifacts\n    scope: cap:routing\n  - name: scenario\n    kind: scenario\n    scope: cap:routing#deliver\n  - name: provenance\n    kind: model-provenance\n    scope: cap:routing\n";
    write(
        root,
        "docs/capabilities/routing.md",
        &component_model::document(&model).replacen("model:", &format!("{declarations}model:"), 1),
    );
    let empty = component_model::document("")
        .replace("cap:routing", "cap:empty")
        .replace(
            "code: [parcel::Parcel, parcel::route, parcel::store]",
            "code: []",
        )
        .replace("tests: [parcel::test_crate::flow::case]", "tests: []");
    write(root, "docs/capabilities/empty.md", &empty);
}

#[test]
fn repository_links_resolve_from_two_depths_without_changing_cached_views() {
    let scratch = Scratch::new("repository-navigation");
    let root = scratch.path();
    fixture(root);
    let opened = Gate::workspace(root).open().unwrap();
    let plan = opened.workplan();
    for output in ["site # café", "published/nested output"] {
        fs::create_dir_all(root.join(output)).unwrap();
        write(root, &format!("{output}/host.html"), "unrelated host bytes");
        for name in ["components", "artifacts", "scenario", "provenance"] {
            let reference = format!("cap:routing#{name}");
            let cached = plan.named_diagram(&reference).unwrap();
            let before = cached.json().to_owned();
            let directory =
                publication::publish_repository(&opened, &reference, &root.join(output)).unwrap();
            let files = snapshot(root);
            assert_eq!(
                publication::publish_repository(&opened, &reference, &root.join(output)).unwrap(),
                directory
            );
            if output == "site # café" && name == "components" {
                let alias = root.join(output).join("..").join(output);
                assert_eq!(
                    publication::publish_repository(&opened, &reference, &alias).unwrap(),
                    directory
                );
            }
            assert_eq!(snapshot(root), files);
            assert_eq!(plan.named_diagram(&reference).unwrap().json(), before);
            let baseline = publication::publish(&cached, &root.join(output)).unwrap();
            assert_ne!(directory, baseline);
            for current in std::iter::once(cached.as_ref()).chain(cached.details()) {
                check_links(root, &directory, current);
            }
        }
        assert_eq!(
            fs::read_to_string(root.join(output).join("host.html")).unwrap(),
            "unrelated host bytes"
        );
    }
}

fn check_links(root: &Path, directory: &Path, cached: &raf_workplan::diagram::DiagramBundle) {
    let name = cached.view().name();
    let text = fs::read_to_string(directory.join(format!("{name}.json"))).unwrap();
    let value = json::read(&text).unwrap();
    let contextual = VizPresentedExport::from_json(value.get("export").unwrap()).unwrap();
    let original = cached.presented_export().unwrap();
    assert_eq!(contextual.export.model, original.export.model);
    assert_eq!(contextual.export.layout, original.export.layout);
    assert_eq!(contextual.geometry, original.geometry);
    let page = directory.join(format!("{name}.html"));
    let location = format!(
        "https://example.org/non-root/project/{}",
        purrdf_lex::percent::encode(
            page.strip_prefix(root).unwrap().to_str().unwrap(),
            purrdf_lex::percent::UNRESERVED_SLASH
        )
    );
    let base = BaseIri::parse(&location).unwrap();
    let svg = fs::read_to_string(directory.join(format!("{name}.svg"))).unwrap();
    let mut links = 0;
    for display in &contextual.presentation.displays {
        let Some(navigation) = &display.navigation else {
            continue;
        };
        let href = navigation.as_str();
        assert!(!href.contains("#L"));
        assert!(svg.contains(href));
        let resolved = base.resolve(href).unwrap();
        let expected = if href.contains("escaped") {
            "src/escaped # café.rs"
        } else if href.contains("tests") {
            "tests/flow.rs"
        } else {
            "docs/capabilities/empty.md"
        };
        assert!(root.join(expected).is_file());
        assert_eq!(
            resolved.as_str(),
            format!(
                "https://example.org/non-root/project/{}",
                purrdf_lex::percent::encode(expected, purrdf_lex::percent::UNRESERVED_SLASH)
            )
        );
        links += 1;
    }
    assert!(links > 0);
    let html = fs::read_to_string(page).unwrap();
    for detail in cached.details() {
        assert!(html.contains(&format!("{}.html", detail.view().name())));
        assert!(
            directory
                .join(format!("{}.html", detail.view().name()))
                .is_file()
        );
        assert!(
            fs::read_to_string(directory.join(format!("{}.html", detail.view().name())))
                .unwrap()
                .contains(&format!("{name}.html"))
        );
    }
}

#[test]
fn repository_publication_refuses_missing_changed_and_outside_bindings() {
    let scratch = Scratch::new("repository-navigation-refusals");
    let root = scratch.path();
    fixture(root);
    let output = root.join("site");
    fs::create_dir(&output).unwrap();
    let opened = Gate::workspace(root).open().unwrap();
    let reference = "cap:routing#components";
    for rejected in [root.join("missing"), scratch.target()] {
        let before = snapshot(root);
        assert!(publication::publish_repository(&opened, reference, &rejected).is_err());
        assert_eq!(snapshot(root), before);
    }
    let source = root.join("src/escaped # café.rs");
    for content in [
        None,
        Some("pub struct Parcel;\npub fn route() { let changed = 1; }\npub fn store() {}\n"),
    ] {
        if let Some(content) = content {
            fs::write(&source, content).unwrap();
        } else {
            fs::remove_file(&source).unwrap();
        }
        let before = snapshot(root);
        assert!(publication::publish_repository(&opened, reference, &output).is_err());
        assert_eq!(snapshot(root), before);
        fs::write(&source, component_model::SOURCE).unwrap();
    }
    let capability = root.join("docs/capabilities/empty.md");
    let original = fs::read(&capability).unwrap();
    fs::remove_file(&capability).unwrap();
    let before = snapshot(root);
    assert!(publication::publish_repository(&opened, reference, &output).is_err());
    assert_eq!(snapshot(root), before);
    fs::write(capability, original).unwrap();
    #[cfg(target_os = "linux")]
    {
        let outside = scratch.target().join("outside.rs");
        fs::write(&outside, component_model::SOURCE).unwrap();
        fs::remove_file(&source).unwrap();
        std::os::unix::fs::symlink(&outside, &source).unwrap();
        let before = snapshot(root);
        assert!(publication::publish_repository(&opened, reference, &output).is_err());
        assert_eq!(snapshot(root), before);
        fs::remove_file(&source).unwrap();
        fs::write(&source, component_model::SOURCE).unwrap();
    }
    publication::publish_repository(&opened, reference, &output).unwrap();
    let moved = scratch.target().join("moved-workspace");
    fs::rename(root, &moved).unwrap();
    fs::create_dir(root).unwrap();
    let before = snapshot(&moved);
    assert!(publication::publish_repository(&opened, reference, &output).is_err());
    assert_eq!(snapshot(&moved), before);
    assert!(snapshot(root).is_empty());
}
