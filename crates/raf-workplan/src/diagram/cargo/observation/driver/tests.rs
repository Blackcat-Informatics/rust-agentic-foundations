// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Receipt admission must bind the actual input, rather than a named option value.

use super::*;
use raf_rdf::{Builder, Format, Graph, TermValue, Triple};

struct Fixture {
    _scratch: purrdf_testkit::TempDir,
    observed: Observation,
    controls: RustdocObservationOptions,
    library: Library,
    other: String,
}

fn fixture() -> Fixture {
    let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let root = scratch.path().canonicalize().unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='sample'\nversion='1.0.0'\n",
    )
    .unwrap();
    std::fs::write(root.join("selected.rs"), "pub fn selected() {}\n").unwrap();
    std::fs::write(root.join("other.rs"), "pub fn other() {}\n").unwrap();
    let controls = RustdocObservationOptions::new(
        &root.join("driver"),
        &root.join("compiler"),
        &root.join("receipts"),
    )
    .unwrap();
    let mut cargo_environment: Vec<_> = controls
        .environment()
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_str().unwrap().to_owned()))
        .collect();
    cargo_environment.extend([
        ("CARGO_MANIFEST_DIR".into(), root.to_str().unwrap().into()),
        ("CARGO_PKG_NAME".into(), "sample".into()),
        ("CARGO_PKG_VERSION".into(), "1.0.0".into()),
    ]);
    sort_environment(&mut cargo_environment).unwrap();
    let arguments = vec!["--crate-name".into(), "sample".into(), "selected.rs".into()];
    let observed = Observation {
        driver: locator(controls.driver()).unwrap(),
        delegate: locator(controls.delegate()).unwrap(),
        directory: locator(&root).unwrap(),
        query_arguments: query_arguments(&arguments).unwrap(),
        arguments,
        compiler_environment: compiler_environment(&cargo_environment, &controls).unwrap(),
        cargo_environment,
        configuration: Output::new(Some(0), b"doc\n".to_vec(), Vec::new()),
        html: Output::new(Some(0), Vec::new(), Vec::new()),
        assembly: None,
    };
    let library = Library {
        package_id: "opaque-sample".into(),
        package: "sample".into(),
        version: "1.0.0".into(),
        manifest: root.join("Cargo.toml"),
        name: "sample".into(),
        kind: vec!["lib".into()],
        crate_types: vec!["lib".into()],
        source: root.join("selected.rs"),
        features: Vec::new(),
    };
    Fixture {
        _scratch: scratch,
        observed,
        controls,
        library,
        other: "other.rs".into(),
    }
}

fn decode(observed: &Observation) -> Observation {
    let encoded = super::super::receipt::encode(observed).unwrap();
    Observation::read(&Graph::from_text(&encoded, Format::NTriples).unwrap()).unwrap()
}

#[test]
fn receipts_bind_the_input_operand_and_preserve_option_value_pairing() {
    let fixture = fixture();
    let admitted = decode(&fixture.observed);
    super::super::validate_observation(&admitted, &fixture.controls).unwrap();
    assert!(matches_library(&admitted, &fixture.library).unwrap());

    let mut substituted = fixture.observed.clone();
    substituted.arguments = vec![
        "--crate-name".into(),
        "sample".into(),
        "--html-after-content".into(),
        "selected.rs".into(),
        fixture.other,
    ];
    substituted.query_arguments = query_arguments(&substituted.arguments).unwrap();
    let substituted = decode(&substituted);
    super::super::validate_observation(&substituted, &fixture.controls).unwrap();
    assert!(!matches_library(&substituted, &fixture.library).unwrap());

    let values: Vec<String> = [
        "--crate-name",
        "sample",
        "--resource-suffix",
        "--",
        "--",
        "selected.rs",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let query = query_arguments(&values).unwrap();
    assert_eq!(&query[..4], &values[..4]);
    assert_eq!(&query[4..7], &["-Z", "unstable-options", "--print=cfg"]);
    assert_eq!(&query[7..], &values[4..]);
    checked_link_arguments(&values);
    let mut boundary = vec!["--crate-name=sample".into(), "selected.rs".into()];
    boundary.resize(MAX_ITEMS - 3, "--cfg=fixture".into());
    assert!(query_arguments(&boundary).is_ok());
    boundary.push("--cfg=fixture".into());
    assert!(
        query_arguments(&boundary)
            .unwrap_err()
            .to_string()
            .contains("bounded configuration query")
    );
    reject_bad_shapes(&fixture.observed);
    for invalid in [
        vec!["--crate-name", "sample", "--unknown-option", "selected.rs"],
        vec!["--crate-name", "sample", "selected.rs", "other.rs"],
        vec![
            "--crate-name",
            "sample",
            "--output-format=json",
            "selected.rs",
        ],
        vec!["--crate-name", "sample", "@response"],
        vec!["--crate-name", "sample", "-"],
    ] {
        let invalid: Vec<_> = invalid.into_iter().map(str::to_owned).collect();
        assert!(query_arguments(&invalid).is_err(), "{invalid:?}");
    }
    capability_bindings_keep_both_public_aliases();
}

fn capability_bindings_keep_both_public_aliases() {
    use crate::diagram::rustdoc::checked_bindings;
    use raf_source::{MemoryTree, Workspace};

    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='sample'\nversion='1.0.0'\nedition='2024'\n",
        ),
        (
            "src/lib.rs",
            "/// The shared definition.\npub fn selected() {} pub use selected as first; pub use selected as second;",
        ),
        (
            "docs/capabilities/sample.md",
            "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\ntype: Capability\nid: cap:sample\ntitle: Public aliases\ndescription: Both declared aliases remain required.\nstate: implementing\nbindings:\n  code: [sample::first, sample::second]\n  tests: []\n  examples: []\ndepends: []\n---\n\n## Purpose\n\nDocument both aliases.\n\n## Contract\n\nBoth aliases are required.\n\n## Usage\n\nSelect both names.\n\n## Examples\n\nThe names share one definition.\n\n## Constraints\n\nMissing and duplicate names refuse.\n\n## Explanation\n\nSpelling retains alias identity.\n\n## Verification\n\nCheck admission beside refusal.\n",
        ),
    ]);
    let source = Workspace::load(&tree, &["."]).unwrap();
    let plan = crate::Workplan::load(&tree, &source);
    let names = vec!["sample::r#first".to_owned(), "sample::second".to_owned()];
    assert!(checked_bindings(&plan, "cap:sample", &names).is_ok());
    let missing = checked_bindings(&plan, "cap:sample", &names[..1]).unwrap_err();
    assert!(
        missing
            .to_string()
            .contains("every declared capability code binding")
    );
    let duplicate = vec!["sample::first".to_owned(), "sample::r#first".to_owned()];
    let duplicate = checked_bindings(&plan, "cap:sample", &duplicate).unwrap_err();
    assert!(
        duplicate
            .to_string()
            .contains("duplicate required API spelling")
    );
    assert!(crate::diagram::rustdoc::api_names(["sample::selected::<u8>"]).is_err());
}

fn checked_link_arguments(values: &[String]) {
    let effective = crate::diagram::cargo::assembly::html_arguments(values).unwrap();
    assert_eq!(&effective[..4], &values[..4]);
    assert_eq!(
        &effective[4..6],
        &[
            "--forbid=rustdoc::broken_intra_doc_links",
            "--forbid=rustdoc::private_intra_doc_links"
        ]
    );
    assert_eq!(&effective[6..], &values[4..]);
    for option in [
        "--cap-lints=allow",
        "--force-warn=rustdoc::broken_intra_doc_links",
        "--extern=priv:std=foreign.rlib",
        "--emit=toolchain-shared-resources",
        "--output-format=json",
    ] {
        let arguments = vec![
            "--crate-name=sample".into(),
            option.into(),
            "selected.rs".into(),
        ];
        assert!(
            crate::diagram::cargo::assembly::html_arguments(&arguments).is_err(),
            "{option}"
        );
    }
}

fn reject_bad_shapes(observed: &Observation) {
    let graph = observed.graph().unwrap();
    for (key, expected) in [("driver", "repeated"), ("unknown", "unknown")] {
        let mut altered = Builder::from_graph(&graph);
        altered
            .insert(Triple {
                subject: TermValue::iri(""),
                predicate: TermValue::iri(format!("urn:raf:rustdoc-observation:{key}")),
                object: TermValue::simple_literal("extra"),
            })
            .unwrap();
        let error = Observation::read(&altered.finish().unwrap()).unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
    let mut altered = Builder::from_graph(&graph);
    let driver = graph
        .assertions()
        .find(|triple| triple.predicate == TermValue::iri("urn:raf:rustdoc-observation:driver"))
        .unwrap();
    assert!(altered.remove(driver).unwrap());
    assert!(
        Observation::read(&altered.finish().unwrap())
            .unwrap_err()
            .to_string()
            .contains("missing")
    );
}
