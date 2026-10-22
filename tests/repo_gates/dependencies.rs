// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! First-party manifests cannot restore the obsolete JSON result dependency.

use super::{gate_root, read_text, tracked_paths};

fn check_table(
    table: &toml::Table,
    section: &str,
    findings: &mut Vec<String>,
) -> Result<(), String> {
    for (alias, dependency) in table {
        let package = match dependency {
            toml::Value::String(_) => alias.as_str(),
            toml::Value::Table(fields) => match fields.get("package") {
                Some(toml::Value::String(package)) => package,
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
    let root: toml::Table = manifest
        .parse()
        .map_err(|e| format!("invalid Cargo manifest: {e}"))?;
    let mut findings = Vec::new();
    for name in ["dependencies", "build-dependencies", "dev-dependencies"] {
        if let Some(value) = root.get(name) {
            check_table(
                value
                    .as_table()
                    .ok_or_else(|| format!("{name} is not a table"))?,
                name,
                &mut findings,
            )?;
        }
    }
    if let Some(workspace) = root.get("workspace") {
        let workspace = workspace.as_table().ok_or("workspace is not a table")?;
        if let Some(value) = workspace.get("dependencies") {
            check_table(
                value
                    .as_table()
                    .ok_or("workspace.dependencies is not a table")?,
                "workspace.dependencies",
                &mut findings,
            )?;
        }
    }
    if let Some(targets) = root.get("target") {
        for (target, value) in targets.as_table().ok_or("target is not a table")? {
            let target_table = value.as_table().ok_or("target entry is not a table")?;
            for name in ["dependencies", "build-dependencies", "dev-dependencies"] {
                if let Some(value) = target_table.get(name) {
                    check_table(
                        value
                            .as_table()
                            .ok_or("target dependencies is not a table")?,
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
}

#[test]
fn rdf_dependency_scan_refuses_malformed_manifests_instead_of_skipping_them() {
    assert!(dependency_problems("[dependencies\n").is_err());
    assert!(dependency_problems("[dependencies]\ncarrier={package=false,version='1'}\n").is_err());
    assert!(dependency_problems("workspace=false\n").is_err());
    assert_eq!(
        dependency_problems("[dependencies]\ncarrier={package='raf-rdf',version='1'}\n").unwrap(),
        [] as [String; 0]
    );
}
