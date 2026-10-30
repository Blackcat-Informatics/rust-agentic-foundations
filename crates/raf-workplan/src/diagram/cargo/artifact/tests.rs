// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Selected Cargo identity and actual contained HTML observations.

use super::*;
use crate::exec::CargoDocOptions;
use crate::exec::{FakeRunner, Invocation, Output};
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
        metadata_output: Output::new(Some(0), b"scripted metadata selection".to_vec(), Vec::new()),
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
        ("features", Value::Array(vec![Value::from("unselected")])),
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

/// Dependency resolution must stay equal even when the selected library does not change.
#[test]
fn documentation_refuses_dependency_metadata_drift_with_unchanged_libraries() {
    let (_scratch, fixture, path) = fixture();
    let library = &fixture.libraries[0];
    let metadata = metadata(&fixture, &path);
    let initial = metadata_runner_with_stderr(&fixture, &metadata, b"before diagnostics");
    let selection = super::super::capture(&initial, &fixture.root, &fixture.options).unwrap();
    let artifact = receipt(library, &path);
    let stdout = format!(
        "{}\n{{\"reason\":\"build-finished\",\"success\":true}}\n",
        purrdf_lex::json::write_compact(&artifact)
    );
    let doc = Invocation::cargo_doc(&fixture.options);
    let args = doc.argv();
    let args: Vec<_> = args.iter().map(String::as_str).collect();
    let run = |after: &Value| {
        let runner = metadata_runner_with_stderr(&fixture, after, b"after diagnostics").on(
            doc.program(),
            &args,
            Ok(Output::new(Some(0), stdout.as_bytes().to_vec(), Vec::new())),
        );
        super::super::document(&runner, &selection)
    };
    let observed = run(&metadata).unwrap();
    assert_eq!(observed.artifacts().len(), 1);
    assert_eq!(
        observed.selection().metadata_output().stderr,
        b"before diagnostics"
    );
    assert_eq!(
        observed.recaptured_metadata_output().stderr,
        b"after diagnostics"
    );
    let mut changed = metadata.clone();
    *changed
        .get_mut("resolve")
        .unwrap()
        .get_mut("nodes")
        .unwrap()
        .as_array_mut()
        .unwrap()[1]
        .get_mut("features")
        .unwrap() = Value::Array(vec![Value::from("changed")]);
    let after = super::super::capture(
        &metadata_runner(&fixture, &changed),
        &fixture.root,
        &fixture.options,
    )
    .unwrap();
    assert_eq!(selection.libraries(), after.libraries());
    let error = run(&changed).unwrap_err().to_string();
    assert!(error.contains("scope changed during the run"), "{error}");
}

fn metadata_runner(selection: &Selection, value: &Value) -> FakeRunner {
    metadata_runner_with_stderr(selection, value, &[])
}

fn metadata_runner_with_stderr(selection: &Selection, value: &Value, stderr: &[u8]) -> FakeRunner {
    let invocation = Invocation::cargo_doc_metadata(&selection.options);
    let args = invocation.argv();
    let args: Vec<_> = args.iter().map(String::as_str).collect();
    FakeRunner::new().on(
        invocation.program(),
        &args,
        Ok(Output::new(
            Some(0),
            purrdf_lex::json::write_compact(value).into_bytes(),
            stderr.to_vec(),
        )),
    )
}

#[test]
fn typed_requests_and_bounded_protocol_refusals_keep_a_valid_neighbor() {
    for (packages, features) in [
        (vec![], vec![]),
        (vec!["demo", "demo"], vec![]),
        (vec!["--workspace"], vec![]),
        (vec!["demo"], vec!["unqualified"]),
        (vec!["demo"], vec!["other/feature"]),
        (vec!["demo"], vec!["demo/feature", "demo/feature"]),
    ] {
        assert!(CargoDocOptions::new(&packages, &features, false, None).is_err());
    }
    for path in ["", "bad\npath"] {
        assert!(CargoDocOptions::new(&["demo"], &[], false, Some(Path::new(path))).is_err());
    }
    assert_request_bounds();
    let (_scratch, fixture, path) = fixture();
    let metadata = metadata(&fixture, &path);
    let selection = super::super::capture(
        &metadata_runner(&fixture, &metadata),
        &fixture.root,
        &fixture.options,
    )
    .unwrap();
    assert_eq!(
        selection.metadata_output().stdout,
        purrdf_lex::json::write_compact(&metadata).as_bytes()
    );
    let invocation = Invocation::cargo_doc(&fixture.options);
    let args = invocation.argv();
    let args: Vec<_> = args.iter().map(String::as_str).collect();
    let footer = "{\"reason\":\"build-finished\",\"success\":true}\n";
    let message = purrdf_lex::json::write_compact(&receipt(&fixture.libraries[0], &path));
    let prefix = format!("{message}\n");
    let valid = format!("{prefix}{footer}");
    let run = |output: Output| {
        let runner =
            metadata_runner(&fixture, &metadata).on(invocation.program(), &args, Ok(output));
        super::super::document(&runner, &selection)
    };
    for (invalid, diagnostic) in [
        (format!("{prefix}not JSON\n{footer}"), ""),
        (format!("{prefix}{{}}\n{footer}"), "missing field `reason`"),
        (prefix.clone(), "no successful build-finished receipt"),
        (
            format!("{prefix}{{\"reason\":\"build-finished\",\"success\":false}}\n"),
            "unsuccessful completion receipt",
        ),
        (
            format!("{valid}{footer}"),
            "duplicate or unsuccessful completion receipt",
        ),
        (
            format!("{valid}{{\"reason\":\"build-script-executed\"}}\n"),
            "messages after",
        ),
        (
            format!(
                "{prefix}{{\"reason\":\"compiler-message\",\"message\":{{\"code\":{{\"code\":\"rustdoc::broken_intra_doc_links\"}},\"message\":\"unresolved link\"}}}}\n{footer}"
            ),
            "required Rustdoc link is unresolved",
        ),
    ] {
        let error = run(Output::new(Some(0), invalid.into_bytes(), Vec::new()))
            .unwrap_err()
            .to_string();
        assert!(error.contains(diagnostic), "{error}");
    }
    assert!(
        run(Output::new(
            Some(1),
            b"failed output".to_vec(),
            b"compiler failed".to_vec()
        ))
        .is_err()
    );
    for (stdout, stderr, stream) in [
        (
            vec![b' '; super::super::DEFAULT_OUTPUT_BOUND + 1],
            Vec::new(),
            "standard output",
        ),
        (
            valid.as_bytes().to_vec(),
            vec![b' '; super::super::DEFAULT_OUTPUT_BOUND + 1],
            "standard error",
        ),
    ] {
        let error = run(Output::new(Some(0), stdout, stderr))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("wrote more than 16777216 bytes") && error.contains(stream),
            "{error}"
        );
    }
    assert!(run(Output::new(Some(0), valid.into_bytes(), Vec::new())).is_ok());
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
