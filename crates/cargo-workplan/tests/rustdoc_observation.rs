// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Real Cargo documentation must join compiler observations and HTML artifacts.

use raf_workplan::{
    diagram::cargo::{self, assembly, observation},
    exec::{CargoDocOptions, Programs, RustdocObservationOptions, SystemRunner},
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn put(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn workspace(root: &Path) {
    put(
        root,
        "Cargo.toml",
        "[workspace]\nmembers=['alpha','beta']\nresolver='2'\n",
    );
    put(
        root,
        "Cargo.lock",
        "version=4\n[[package]]\nname='alpha'\nversion='1.0.0'\ndependencies=['beta']\n[[package]]\nname='beta'\nversion='2.0.0'\n",
    );
    put(
        root,
        ".cargo/config.toml",
        "[build]\njobs=2\nrustdocflags=['-Z','unstable-options','--cfg','from_flags','--check-cfg','cfg(from_flags)','--resource-suffix','--']\n",
    );
    put(
        root,
        "alpha/Cargo.toml",
        "[package]\nname='alpha'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nbeta={path='../beta'}\n[features]\nchosen=[]\n",
    );
    put(
        root,
        "alpha/build.rs",
        "fn main() { println!(\"cargo::rustc-check-cfg=cfg(from_build)\"); println!(\"cargo::rustc-cfg=from_build\"); }\n",
    );
    put(
        root,
        "alpha/src/lib.rs",
        "#![deny(rustdoc::broken_intra_doc_links)]\n//! The selected library links [`beta::help`].\n#[cfg(all(doc,from_build,from_flags,feature=\"chosen\"))]\n/// Visible only in the observed compilation context.\npub fn selected() -> u32 { beta::help() }\n",
    );
    put(
        root,
        "beta/Cargo.toml",
        "[package]\nname='beta'\nversion='2.0.0'\nedition='2024'\n",
    );
    put(
        root,
        "beta/src/lib.rs",
        "/// The linked API.\npub fn help() -> u32 { 7 }\n",
    );
    put(
        root,
        "output/unrelated.txt",
        "cumulative output remains owned by its publisher",
    );
}

fn selected_compiler() -> PathBuf {
    // This fixture deliberately selects Rustup's current native compiler. The
    // library operation itself performs no ambient compiler-selection inference.
    let output = Command::new("rustup")
        .args(["which", "rustdoc"])
        .output()
        .expect("native Rustup locator is required");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let path = PathBuf::from(std::str::from_utf8(&output.stdout).unwrap().trim());
    assert!(path.is_absolute());
    path
}

#[test]
fn selected_rustdoc_queries_and_artifacts_share_the_actual_cargo_context() {
    let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let root = scratch.path().canonicalize().unwrap();
    workspace(&root);
    let runner = SystemRunner::new(Programs::new(&root));
    let options = CargoDocOptions::new(
        &["alpha", "beta"],
        &["alpha/chosen"],
        false,
        Some(&root.join("output")),
    )
    .unwrap();
    let compiler = selected_compiler();
    #[cfg(unix)]
    let compiler = {
        let alias = root.join("selected-rustdoc");
        std::os::unix::fs::symlink(&compiler, &alias).unwrap();
        alias
    };
    let driver = Path::new(env!("CARGO_BIN_EXE_cargo-workplan"));
    let controls =
        RustdocObservationOptions::new(driver, &compiler, &root.join("receipts")).unwrap();
    let selection = cargo::capture(&runner, &root, &options).unwrap();
    let observed = observation::document(&runner, &selection, &controls)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(observed.invocations().len(), 2);
    let alpha = observed
        .invocations()
        .iter()
        .find(|row| row.artifact().library().package == "alpha")
        .unwrap();
    let cfg = std::str::from_utf8(&alpha.configuration_output().stdout).unwrap();
    for predicate in ["doc", "from_build", "from_flags", "feature=\"chosen\""] {
        assert!(
            cfg.lines().any(|line| line == predicate),
            "missing native predicate {predicate}: {cfg}"
        );
    }
    assert_eq!(alpha.artifact().library().version, "1.0.0");
    assert!(
        fs::read_to_string(alpha.artifact().reported_path())
            .unwrap()
            .contains("Visible only in the observed compilation context.")
    );
    for invocation in observed.invocations() {
        assert!(invocation.artifact().reported_path().is_file());
        assert!(invocation.html_output().success());
        assert_eq!(
            invocation
                .compiler_environment()
                .iter()
                .find(|(key, _)| key == "RUSTDOC")
                .unwrap()
                .1,
            compiler.to_str().unwrap()
        );
        assert!(!invocation.compiler_environment().iter().any(|(key, _)| {
            [
                "RAF_RUSTDOC_OBSERVER",
                "RAF_RUSTDOC_DELEGATE",
                "RAF_RUSTDOC_RECEIPTS",
            ]
            .contains(&key.as_str())
        }));
    }
    assert_eq!(
        fs::read_to_string(root.join("output/unrelated.txt")).unwrap(),
        "cumulative output remains owned by its publisher"
    );
    let reused =
        RustdocObservationOptions::new(driver, &compiler, &root.join("cached-receipts")).unwrap();
    let current = cargo::capture(&runner, &root, &options).unwrap();
    let Err(error) = observation::document(&runner, &current, &reused) else {
        panic!("cached HTML must not count as a fresh compiler invocation");
    };
    assert!(
        error.to_string().contains("cached or incomplete"),
        "{error}"
    );
    assert!(
        observed
            .documentation()
            .artifacts()
            .iter()
            .all(|artifact| artifact.reported_path().is_file())
    );
    checked_assembly(&root, &runner, &options, driver, &compiler);
}

fn assembly_plan(
    root: &Path,
    runner: &SystemRunner,
    options: &CargoDocOptions,
) -> (cargo::Selection, assembly::AssemblyPlan) {
    put(
        root,
        "alpha/src/lib.rs",
        "#![doc=include_str!(\"../../includes/alpha.md\")]\n#[cfg(all(doc,from_build,from_flags,feature=\"chosen\"))]\n/// Visible only in the observed compilation context.\npub fn selected() -> u32 { beta::help() }\n#[cfg(all(doc,from_build,from_flags,feature=\"chosen\"))] pub use selected as first;\n#[cfg(all(doc,from_build,from_flags,feature=\"chosen\"))] pub use selected as second;\n#[doc=include_str!(\"../../includes/alpha-nested.md\")] pub mod nested {}\n#[doc=\"Keyword namespace.\"] pub mod r#match { #[doc=\"Keyword function.\"] pub fn r#type() {} }\n",
    );
    put(
        root,
        "beta/src/lib.rs",
        "#![doc=include_str!(\"../../includes/beta.md\")]\n/// The linked API.\npub fn help() -> u32 { 7 }\n",
    );
    fs::create_dir_all(root.join("includes")).unwrap();
    for (name, depends, declaration) in [
        (
            "alpha",
            "[cap:beta]",
            "diagrams:\n  - name: dependencies\n    kind: capability-dependencies\n    scope: workspace\n",
        ),
        ("beta", "[]", ""),
    ] {
        let bindings = if name == "alpha" {
            "[alpha::selected, alpha::r#match::r#type, alpha::first, alpha::second]"
        } else {
            "[beta::help]"
        };
        let document = format!(
            "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\ntype: Capability\nid: cap:{name}\ntitle: {name}\ndescription: Documented library dependency.\nstate: implementing\n{declaration}bindings:\n  code: {bindings}\n  tests: []\n  examples: []\ndepends: {depends}\n---\n\n## Purpose\n\nA documented library.\n\n## Contract\n\nThe library retains its public API.\n\n## Usage\n\nCall the documented function.\n\n## Examples\n\nThe shared view shows capability dependencies.\n\n## Constraints\n\nOnly selected compiler inputs are admitted.\n\n## Explanation\n\nThe library participates in the shared view.\n\n## Verification\n\nActual Rustdoc consumes its required links.\n"
        );
        put(root, &format!("docs/capabilities/{name}.md"), &document);
    }
    let selection = cargo::capture(runner, root, options).unwrap();
    let mut requests: Vec<_> = selection
        .libraries()
        .iter()
        .map(|library| {
            let required = if library.package == "alpha" {
                "alpha::selected"
            } else {
                "beta::help"
            };
            let mut required = vec![required.to_owned()];
            if library.package == "alpha" {
                required.push("alpha::r#match::r#type".into());
                required.push("alpha::first".into());
                required.push("alpha::second".into());
            }
            assembly::IncludeRequest::new(
                &library.package_id,
                &library.name,
                raf_source::RelPath::new(&format!("includes/{}.md", library.package)).unwrap(),
                "cap:alpha#dependencies",
                format!("cap:{}", library.package),
                required,
            )
        })
        .collect();
    let alpha = selection
        .libraries()
        .iter()
        .find(|library| library.package == "alpha")
        .unwrap();
    requests.push(assembly::IncludeRequest::new(
        &alpha.package_id,
        "alpha::nested",
        raf_source::RelPath::new("includes/alpha-nested.md").unwrap(),
        "cap:alpha#dependencies",
        "cap:alpha",
        [
            "alpha::selected",
            "alpha::r#match::r#type",
            "alpha::first",
            "alpha::second",
        ]
        .map(str::to_owned)
        .to_vec(),
    ));
    let plan = assembly::AssemblyPlan::new(&selection, requests).unwrap();
    (selection, plan)
}

fn checked_assembly(
    root: &Path,
    runner: &SystemRunner,
    options: &CargoDocOptions,
    driver: &Path,
    compiler: &Path,
) {
    let (selection, plan) = assembly_plan(root, runner, options);
    let controls =
        RustdocObservationOptions::new(driver, compiler, &root.join("assembly-receipts")).unwrap();
    let tree = raf_source::DiskTree::new(root).unwrap();
    let source = raf_source::Workspace::load(&tree, &["alpha", "beta"]).unwrap();
    let seed_model = raf_workplan::Workplan::load(&tree, &source);
    let assembled = assembly::document(
        runner,
        &selection,
        &controls,
        &plan,
        &root.join("assembly.nt"),
        &seed_model,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(assembled.publications().len(), 3);
    for invocation in assembled.observations().invocations() {
        let preparation = invocation.assembly_preparation().unwrap();
        for lint in [
            "--forbid=rustdoc::broken_intra_doc_links",
            "--forbid=rustdoc::private_intra_doc_links",
        ] {
            assert!(
                preparation
                    .html_arguments()
                    .iter()
                    .any(|argument| argument == lint)
            );
        }
        let version = std::str::from_utf8(&preparation.compiler_version().stdout).unwrap();
        assert!(version.lines().any(|line| line.starts_with("host: ")));
        assert!(
            !invocation
                .compiler_environment()
                .iter()
                .any(|(key, _)| key.starts_with("RAF_RUSTDOC_"))
        );
        let name = &invocation.artifact().library().name;
        let include = fs::read_to_string(root.join(format!("includes/{name}.md"))).unwrap();
        assert!(include.contains(if name == "alpha" {
            "(fn@crate::selected)"
        } else {
            "(fn@crate::help)"
        }));
        let html = fs::read_to_string(invocation.artifact().reported_path()).unwrap();
        assert!(html.contains("API references"));
        assert!(include.contains("Cargo-observed package"));
        assert!(include.contains("unframed content SHA-256"));
        assert!(include.contains("framed symbol SHA-256"));
        if name == "alpha" {
            assert!(include.contains("<code>alpha::r#match::r#type</code>"));
            assert!(include.contains("(fn@crate::match::type)"));
            assert!(include.contains("(fn@crate::first)"));
            assert!(include.contains("(fn@crate::second)"));
            assert!(html.contains("href=\"match/fn.type.html\""));
            checked_nested(invocation.artifact().reported_path());
        }
    }
    for publication in assembled.publications() {
        let context = fs::read_to_string(publication.join("assembly.ttl")).unwrap();
        assert!(context.contains("actual-rustdoc-forbid-broken-and-private-intra-doc-links"));
        for private in ["RAF_RUSTDOC_", "RUSTDOCFLAGS", root.to_str().unwrap()] {
            assert!(
                !context.contains(private),
                "private compiler context escaped: {private}"
            );
        }
        assert!(publication.join("dependencies.svg").is_file());
        assert!(publication.join("dependencies.html").is_file());
    }
    assert_eq!(
        fs::read_to_string(root.join("output/unrelated.txt")).unwrap(),
        "cumulative output remains owned by its publisher"
    );
    opaque_source_refuses_before_publication(
        root,
        runner,
        driver,
        compiler,
        &seed_model,
        &assembled,
    );
}

fn checked_nested(entry: &Path) {
    let directory = entry.parent().unwrap().join("nested");
    let nested = fs::read_to_string(directory.join("index.html")).unwrap();
    assert!(nested.contains("API references"));
    // Rustdoc can link re-exports to their common definition. Check both
    // actual label destinations without predicting alias files.
    for alias in ["first", "second"] {
        let label = format!("<code>alpha::{alias}</code>");
        let prefix = nested.split_once(&label).unwrap().0;
        let link = prefix.rsplit_once("<a href=\"").unwrap().1;
        let href = link.split_once('"').unwrap().0;
        assert!(
            directory.join(href).is_file(),
            "Rustdoc emitted no existing API destination for {alias}: {href}"
        );
    }
}

fn opaque_source_refuses_before_publication(
    root: &Path,
    runner: &SystemRunner,
    driver: &Path,
    compiler: &Path,
    seed_model: &raf_workplan::Workplan<'_>,
    published: &assembly::AssemblyDocumentation,
) {
    let manifests: Vec<_> = published
        .publications()
        .iter()
        .map(|directory| {
            let path = directory.join("manifest.ttl");
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect();
    let source = fs::read_to_string(root.join("alpha/src/lib.rs")).unwrap();
    put(
        root,
        "alpha/src/lib.rs",
        &source.replacen(
            "pub fn selected()",
            "#[an_unobserved_rewrite]\npub fn selected()",
            1,
        ),
    );
    let options = CargoDocOptions::new(
        &["alpha"],
        &["alpha/chosen"],
        false,
        Some(&root.join("output")),
    )
    .unwrap();
    let selection = cargo::capture(runner, root, &options).unwrap();
    let library = &selection.libraries()[0];
    let include = fs::read(root.join("includes/alpha.md")).unwrap();
    let request = assembly::IncludeRequest::new(
        &library.package_id,
        "alpha",
        raf_source::RelPath::new("includes/alpha.md").unwrap(),
        "cap:alpha#dependencies",
        "cap:alpha",
        [
            "alpha::selected",
            "alpha::r#match::r#type",
            "alpha::first",
            "alpha::second",
        ]
        .map(str::to_owned)
        .to_vec(),
    )
    .replace_owned(purrdf_rdf::ContentDigest::of(&include));
    let plan = assembly::AssemblyPlan::new(&selection, vec![request]).unwrap();
    let controls =
        RustdocObservationOptions::new(driver, compiler, &root.join("refused-receipts")).unwrap();
    let error = assembly::document(
        runner,
        &selection,
        &controls,
        &plan,
        &root.join("refused-assembly.nt"),
        seed_model,
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("opaque rewriting attributes"),
        "{error}"
    );
    for (path, bytes) in manifests {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
}
