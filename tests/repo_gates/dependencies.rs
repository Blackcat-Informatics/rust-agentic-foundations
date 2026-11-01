// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! First-party manifests cannot restore the obsolete JSON result dependency.

use super::{gate_root, read_text, tracked_paths};
use purrdf_lex::assoc::get_sorted as get;
use purrdf_toml::{Document, Float, Kind, Node, Profile, Scalar, SourceDocument};

// Cargo policy selects fields from the native arena; it owns no TOML grammar.
fn table<'a>(
    document: &'a Document<'_>,
    node: &Node,
    section: &str,
) -> Result<Vec<(&'a str, &'a Node)>, String> {
    if !matches!(node.kind, Kind::Table(_)) {
        return Err(format!("{section} is not a table"));
    }
    let mut fields: Vec<_> = node
        .children
        .iter()
        .map(|id| {
            let child = &document.nodes()[*id];
            (
                child
                    .key
                    .as_ref()
                    .expect("native table member key")
                    .decoded
                    .as_str(),
                child,
            )
        })
        .collect();
    // Preserve the dependency and target order of the original manifest gate.
    purrdf_lex::walk::try_sort_unstable_by(&mut fields, |a, b| {
        Ok::<_, std::convert::Infallible>(a.0.cmp(b.0))
    })
    .expect("infallible field ordering");
    Ok(fields)
}

fn check_table(
    document: &Document<'_>,
    node: &Node,
    section: &str,
    findings: &mut Vec<String>,
) -> Result<(), String> {
    for (alias, dependency) in table(document, node, section)? {
        let package = match &dependency.kind {
            Kind::Scalar(Scalar::String(_)) => alias,
            Kind::Table(_) => match get(&table(document, dependency, section)?, "package").copied()
            {
                Some(Node {
                    kind: Kind::Scalar(Scalar::String(package)),
                    ..
                }) => package,
                Some(_) => {
                    return Err(format!(
                        "{section}: package for {} is not a string",
                        alias.escape_debug()
                    ));
                }
                None => alias,
            },
            _ => {
                return Err(format!(
                    "{section}: dependency {} is not a string or table",
                    alias.escape_debug()
                ));
            }
        };
        if package.replace('-', "_") == "serde_json" {
            findings.push(format!(
                "{section}.{}: serde_json is forbidden; use the typed RDF report path",
                alias.escape_debug()
            ));
        }
    }
    Ok(())
}

fn dependency_problems(manifest: &str) -> Result<Vec<String>, String> {
    let profile = Profile::standard().map_err(|e| format!("invalid manifest profile: {e}"))?;
    let document = purrdf_toml::analyze(
        SourceDocument {
            id: "urn:raf:dependency-policy:manifest",
            bytes: manifest.as_bytes(),
        },
        &profile,
    )
    .map_err(|e| format!("invalid Cargo manifest: {e}"))?;
    // Cargo refuses decimal float overflow even in unrelated metadata. Explicit
    // TOML infinity remains valid; the native model retains its original span.
    for node in document.nodes() {
        if matches!(
            node.kind,
            Kind::Scalar(Scalar::Float(Float::Infinity { .. }))
        ) && !["inf", "+inf", "-inf"].contains(&&manifest[node.span.clone()])
        {
            return Err(format!(
                "invalid Cargo manifest at byte {}: floating-point overflow; explicit inf is required",
                node.span.start
            ));
        }
    }
    let root = table(&document, &document.nodes()[0], "manifest")?;
    let mut findings = Vec::new();
    for name in ["dependencies", "build-dependencies", "dev-dependencies"] {
        if let Some(value) = get(&root, name) {
            check_table(&document, value, name, &mut findings)?;
        }
    }
    if let Some(workspace) = get(&root, "workspace") {
        let workspace = table(&document, workspace, "workspace")?;
        if let Some(value) = get(&workspace, "dependencies") {
            check_table(&document, value, "workspace.dependencies", &mut findings)?;
        }
    }
    if let Some(targets) = get(&root, "target") {
        for (target, value) in table(&document, targets, "target")? {
            let target_table = table(&document, value, "target entry")?;
            for name in ["dependencies", "build-dependencies", "dev-dependencies"] {
                if let Some(value) = get(&target_table, name) {
                    check_table(
                        &document,
                        value,
                        &format!("target.{}.{name}", target.escape_debug()),
                        &mut findings,
                    )?;
                }
            }
        }
    }
    Ok(findings)
}

#[test]
fn rdf_first_party_manifests_have_no_json_result_dependency() {
    let root = gate_root();
    let manifests: Vec<_> = tracked_paths()
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|name| name == "Cargo.toml"))
        .collect();
    assert_ne!(manifests.len(), 0);
    for path in manifests {
        let name = path.to_str().unwrap();
        let findings =
            dependency_problems(&read_text(&root, name)).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(findings.is_empty(), "{name}: {}", findings.join("\n"));
    }
}

#[test]
fn rdf_dependency_refusal_covers_aliases_targets_workspace_and_every_dependency_class() {
    for manifest in [
        "[dependencies]\nserde_json='1'\n",
        "[dev-dependencies]\nserde-json='1'\n",
        "[build-dependencies]\ncarrier={package='serde_json',version='1'}\n",
        "[workspace.dependencies.carrier]\npackage='serde_json'\nversion='1'\n",
        "[target.'cfg(unix)'.dependencies]\ncarrier={package='serde_json',version='1'}\n",
        "[target.x86_64-pc-windows-gnu.dev-dependencies.carrier]\npackage='serde_json'\nversion='1'\n",
        "[target.'cfg(unix)'.build-dependencies]\nserde_json={version='1',optional=true}\n",
    ] {
        assert_eq!(
            dependency_problems(manifest).unwrap().len(),
            1,
            "{manifest}"
        );
        let valid = manifest
            .replace("serde_json", "raf-rdf")
            .replace("serde-json", "raf-rdf");
        assert!(dependency_problems(&valid).unwrap().is_empty(), "{valid}");
    }
    assert_eq!(
        dependency_problems(
            "[package.metadata]\nserializer='serde_json'\n[dependencies]\nserde_yaml='0.9'\n"
        )
        .unwrap(),
        [] as [String; 0]
    );
    let ordered = dependency_problems(
        "[target.z.dependencies]\nz={package='serde_json'}\na={package='serde_json'}\n[target.a.dependencies]\nb={package='serde_json'}\n[workspace.dependencies]\nw={package='serde_json'}\n[dev-dependencies]\nd={package='serde_json'}\n[build-dependencies]\nb={package='serde_json'}\n[dependencies]\nz={package='serde_json'}\na={package='serde_json'}\n",
    ).unwrap();
    let sections: Vec<_> = ordered
        .iter()
        .map(|finding| finding.split_once(':').unwrap().0)
        .collect();
    assert_eq!(
        sections,
        [
            "dependencies.a",
            "dependencies.z",
            "build-dependencies.b",
            "dev-dependencies.d",
            "workspace.dependencies.w",
            "target.a.dependencies.b",
            "target.z.dependencies.a",
            "target.z.dependencies.z"
        ]
    );
}

#[test]
fn rdf_dependency_scan_refuses_malformed_manifests_instead_of_skipping_them() {
    for invalid in [
        "[dependencies\n",
        "[dependencies]\ncarrier={package=false,version='1'}\n",
        "workspace=false\n",
        "dependencies=false\n",
        "[dependencies]\ncarrier=false\n",
        "[workspace]\ndependencies=false\n",
        "target=false\n",
        "[target]\nlinux=false\n",
        "[target.linux]\ndependencies=false\n",
        "[dependencies]\ncarrier='1'\ncarrier='2'\n",
        "[package.metadata]\nvalue=1e999\n",
    ] {
        assert!(dependency_problems(invalid).is_err(), "{invalid}");
    }
    for infinity in ["inf", "+inf", "-inf"] {
        assert_eq!(
            dependency_problems(&format!("[package.metadata]\nvalue={infinity}\n")).unwrap(),
            [] as [String; 0]
        );
    }
    let nested = |depth| {
        format!(
            "[package.metadata]\nvalue={}0{}\n",
            "[".repeat(depth),
            "]".repeat(depth)
        )
    };
    let limit = Profile::standard().unwrap().bounds().max_depth as usize;
    let error = dependency_problems(&nested(limit + 1)).expect_err("native depth admission");
    assert!(error.contains("depth"), "{error}");
    assert_eq!(dependency_problems(&nested(2)).unwrap(), [] as [String; 0]);
    assert_eq!(
        dependency_problems("[dependencies]\ncarrier={package='raf-rdf',version='1'}\n").unwrap(),
        [] as [String; 0]
    );
}
