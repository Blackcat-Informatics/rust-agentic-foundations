// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Selected Cargo identity and actual contained HTML observations.

use super::*;
use crate::diagram::cargo::CargoDocOptions;
use purrdf_lex::json::Object;
use std::fs;

const HTML: &[u8] = b"<!doctype html><title>selected output</title>";

fn fixture() -> (purrdf_testkit::TempDir, Selection, PathBuf) {
    let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let root = scratch.path().canonicalize().unwrap();
    let target_directory = root.join("target");
    let path = target_directory.join("unexpected/named.html");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, HTML).unwrap();
    let library = Library {
        package_id: "opaque-selected-package".into(),
        package: "demo".into(),
        version: "1.0.0".into(),
        manifest: root.join("Cargo.toml"),
        name: "demo".into(),
        kind: vec!["lib".into()],
        crate_types: vec!["lib".into()],
        source: root.join("src/lib.rs"),
        features: Vec::new(),
    };
    let selection = Selection {
        options: CargoDocOptions::new(&["demo"], &[], false, None).unwrap(),
        root,
        target_directory,
        libraries: vec![library],
        metadata_sha256: ContentDigest::of(b"scripted metadata selection"),
        metadata_bytes: b"scripted metadata selection".to_vec(),
    };
    (scratch, selection, path)
}

fn string_set(values: &[String]) -> Value {
    Value::Array(
        values
            .iter()
            .map(|value| Value::from(value.as_str()))
            .collect(),
    )
}

/// Cargo's documented compiler-artifact shape with fixture-local locators.
fn receipt(library: &Library, path: &Path) -> Value {
    Value::from(
        Object::new()
            .with("reason", "compiler-artifact")
            .with("fresh", false)
            .with("package_id", library.package_id.as_str())
            .with("manifest_path", library.manifest.to_str().unwrap())
            .with("features", string_set(&library.features))
            .with(
                "filenames",
                Value::Array(vec![Value::from(path.to_str().unwrap())]),
            )
            .with(
                "target",
                Object::new()
                    .with("name", library.name.as_str())
                    .with("kind", string_set(&library.kind))
                    .with("crate_types", string_set(&library.crate_types))
                    .with("src_path", library.source.to_str().unwrap())
                    .with("doc", true),
            ),
    )
}

#[test]
fn reported_html_requires_exact_selected_receipts_and_contained_files() {
    let (_scratch, mut selection, path) = fixture();
    let valid = receipt(&selection.libraries[0], &path);
    let non_html = receipt(&selection.libraries[0], &path.with_extension("rlib"));
    let admitted = admit(&selection, &[non_html, valid.clone()]).unwrap();
    assert_eq!(admitted.len(), 1);
    assert_eq!(admitted[0].library().package_id, "opaque-selected-package");
    assert_eq!(admitted[0].reported_path(), path);
    assert_eq!(
        admitted[0].relative_path().as_str(),
        "unexpected/named.html"
    );
    assert_eq!(admitted[0].entry_directory().as_str(), "unexpected");
    assert_eq!(admitted[0].sha256(), ContentDigest::of(HTML));
    assert!(!admitted[0].fresh());
    let mut reused = valid.clone();
    *reused.get_mut("fresh").unwrap() = Value::Bool(true);
    assert!(admit(&selection, &[reused]).unwrap()[0].fresh());
    let mut malformed = valid.clone();
    *malformed.get_mut("fresh").unwrap() = Value::from("false");
    assert!(admit(&selection, &[malformed]).is_err());
    let mut missing = valid.clone();
    missing.as_object_mut().unwrap().remove("fresh");
    assert!(admit(&selection, &[missing]).is_err());
    assert_reported_features(&selection, &valid);
    assert_identity_refusals(&selection, &valid);
    assert_output_refusals(&selection, &valid, &path);

    let mut other = selection.libraries[0].clone();
    other.package_id = "opaque-other-package".into();
    other.package = "other".into();
    other.name = "other".into();
    other.manifest = selection.root.join("other/Cargo.toml");
    other.source = selection.root.join("other/src/lib.rs");
    selection.options = CargoDocOptions::new(&["demo", "other"], &[], false, None).unwrap();
    selection.libraries.push(other.clone());
    assert!(admit(&selection, &[valid.clone(), receipt(&other, &path)]).is_err());
    #[cfg(target_os = "linux")]
    {
        let alias = path.with_file_name("alias.html");
        fs::hard_link(&path, &alias).unwrap();
        assert!(admit(&selection, &[valid.clone(), receipt(&other, &alias)]).is_err());
    }
    let neighbor = path.with_file_name("neighbor.html");
    fs::write(&neighbor, b"<!doctype html><title>other output</title>").unwrap();
    assert_eq!(
        admit(&selection, &[valid, receipt(&other, &neighbor)])
            .unwrap()
            .len(),
        2
    );
}

fn assert_identity_refusals(selection: &Selection, valid: &Value) {
    for (key, value) in [
        ("name", Value::from("other")),
        ("kind", Value::Array(vec![Value::from("bin")])),
        ("crate_types", Value::Array(Vec::new())),
        (
            "src_path",
            Value::from(selection.root.join("other.rs").to_str().unwrap()),
        ),
        ("doc", Value::Bool(false)),
    ] {
        let mut changed = valid.clone();
        *changed.get_mut("target").unwrap().get_mut(key).unwrap() = value;
        assert!(admit(selection, &[changed]).is_err(), "target {key}");
    }
    for (key, value) in [
        ("package_id", Value::from("unselected")),
        (
            "features",
            Value::Array(vec![Value::from("extra"), Value::from("extra")]),
        ),
        (
            "manifest_path",
            Value::from(selection.root.join("other.toml").to_str().unwrap()),
        ),
    ] {
        let mut changed = valid.clone();
        *changed.get_mut(key).unwrap() = value;
        assert!(admit(selection, &[changed]).is_err(), "receipt {key}");
    }
}

fn assert_reported_features(selection: &Selection, valid: &Value) {
    let mut requested = selection.clone();
    requested.options = CargoDocOptions::new(&["demo"], &["demo/required"], false, None).unwrap();
    requested.libraries[0].features = vec!["extra".into(), "required".into()];
    let error = admit(&requested, std::slice::from_ref(valid))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("omits requested feature `required`"),
        "{error}"
    );
    let mut reported = valid.clone();
    *reported.get_mut("features").unwrap() = Value::Array(vec![Value::from("required")]);
    let observed = admit(&requested, &[reported.clone()]).unwrap();
    assert_eq!(observed[0].features(), ["required"]);
    assert_eq!(observed[0].library().features, ["extra", "required"]);
    *reported.get_mut("features").unwrap() =
        Value::Array(vec![Value::from("required"), Value::from("observed")]);
    assert_eq!(
        admit(&requested, &[reported]).unwrap()[0].features(),
        ["observed", "required"]
    );
}

fn assert_output_refusals(selection: &Selection, valid: &Value, path: &Path) {
    assert!(admit(selection, &[]).is_err());
    assert!(admit(selection, &[valid.clone(), valid.clone()]).is_err());
    for filename in [
        selection.root.join("outside.html"),
        path.with_file_name("missing.html"),
    ] {
        assert!(admit(selection, &[receipt(&selection.libraries[0], &filename)]).is_err());
    }
    let mut ambiguous = valid.clone();
    *ambiguous.get_mut("filenames").unwrap() = Value::Array(vec![
        Value::from(path.to_str().unwrap()),
        Value::from(path.with_file_name("second.html").to_str().unwrap()),
    ]);
    assert!(admit(selection, &[ambiguous]).is_err());
    let oversized = path.with_file_name("oversized.html");
    fs::File::create(&oversized)
        .unwrap()
        .set_len(MAX_HTML_BYTES + 1)
        .unwrap();
    assert!(admit(selection, &[receipt(&selection.libraries[0], &oversized)]).is_err());
    assert!(admit(selection, std::slice::from_ref(valid)).is_ok());
}

#[test]
fn imported_metadata_and_messages_are_bounded_data_with_explicit_roots() {
    use crate::diagram::cargo::{MAX_METADATA_BYTES, read_artifacts, read_metadata};
    assert_request_bounds();
    let (_scratch, fixture, path) = fixture();
    let metadata = purrdf_lex::json::write_compact(&metadata(&fixture, &path));
    let selected = read_metadata(
        &fixture.root,
        &fixture.target_directory,
        &fixture.options,
        metadata.as_bytes(),
    )
    .unwrap();
    assert_eq!(selected.metadata_bytes(), metadata.as_bytes());
    assert!(
        read_metadata(
            &fixture.root,
            &fixture.root,
            &fixture.options,
            metadata.as_bytes()
        )
        .is_err()
    );
    assert!(
        read_metadata(
            &fixture.root,
            &fixture.target_directory,
            &fixture.options,
            &vec![b' '; MAX_METADATA_BYTES + 1]
        )
        .is_err()
    );
    let entry = purrdf_lex::json::write_compact(&receipt(&fixture.libraries[0], &path));
    let valid = format!("{entry}\n{{\"reason\":\"build-finished\",\"success\":true}}\n");
    for invalid in [
        entry.clone(),
        format!("{entry}\nnot JSON\n"),
        format!("{entry}\n{{\"reason\":\"build-finished\",\"success\":false}}\n"),
    ] {
        assert!(read_artifacts(&selected, invalid.as_bytes()).is_err());
    }
    assert!(read_artifacts(&selected, &vec![b' '; MAX_METADATA_BYTES + 1]).is_err());
    assert_eq!(
        read_artifacts(&selected, valid.as_bytes())
            .unwrap()
            .artifacts()
            .len(),
        1
    );
}

fn assert_request_bounds() {
    let packages: Vec<_> = (0..257).map(|index| format!("package{index}")).collect();
    let packages: Vec<_> = packages.iter().map(String::as_str).collect();
    assert!(CargoDocOptions::new(&packages[..256], &[], false, None).is_ok());
    let error = CargoDocOptions::new(&packages, &[], false, None).unwrap_err();
    assert!(error.to_string().contains("1 to 256 packages"));
    let features: Vec<_> = (0..257)
        .map(|index| format!("demo/feature{index}"))
        .collect();
    let features: Vec<_> = features.iter().map(String::as_str).collect();
    assert!(CargoDocOptions::new(&["demo"], &features[..256], false, None).is_ok());
    let error = CargoDocOptions::new(&["demo"], &features, false, None).unwrap_err();
    assert!(error.to_string().contains("at most 256 qualified features"));
}

fn metadata(selection: &Selection, path: &Path) -> Value {
    let library = &selection.libraries[0];
    let artifact = receipt(library, path);
    let node = |id: &str| {
        Value::from(
            Object::new()
                .with("id", id)
                .with("features", Value::Array(Vec::new())),
        )
    };
    Value::from(
        Object::new()
            .with("version", 1_u64)
            .with("workspace_root", selection.root.to_str().unwrap())
            .with(
                "target_directory",
                selection.target_directory.to_str().unwrap(),
            )
            .with(
                "workspace_members",
                Value::Array(vec![Value::from(library.package_id.as_str())]),
            )
            .with(
                "packages",
                Value::Array(vec![Value::from(
                    Object::new()
                        .with("id", library.package_id.as_str())
                        .with("name", library.package.as_str())
                        .with("version", library.version.as_str())
                        .with("source", Value::Null)
                        .with("manifest_path", library.manifest.to_str().unwrap())
                        .with(
                            "targets",
                            Value::Array(vec![artifact.get("target").unwrap().clone()]),
                        ),
                )]),
            )
            .with(
                "resolve",
                Object::new().with(
                    "nodes",
                    Value::Array(vec![node(&library.package_id), node("opaque-dependency")]),
                ),
            ),
    )
}
