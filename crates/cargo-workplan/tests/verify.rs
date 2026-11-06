// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! CLI verification reports static declarations and never executes consumer bodies.

mod support;
use support::{BLOCKS, Repo, capability, change};

fn fixture() -> Repo {
    let repo = Repo::real("static-verification");
    repo.put("src/lib.rs", "/// API.\npub fn add(a:u32,b:u32)->u32 {a+b}\n#[cfg(test)] mod tests { #[test] fn fails() { panic!(\"consumer test must not execute\"); } }\n");
    repo.put(
        "examples/exiting.rs",
        "fn main() { panic!(\"consumer example must not execute\"); }",
    );
    repo.put(
        "docs/capabilities/fixture.md",
        &capability(
            "fixture",
            "implementing",
            [
                "[demo::add]",
                "[demo::tests::fails]",
                "[demo::example_crate::exiting]",
            ],
            "[]",
            &[("Verification", BLOCKS)],
        ),
    );
    repo.put(
        "work/changes/shape.md",
        &change("shape", "[cap:fixture]", &["cap:fixture#contract"]),
    );
    repo.sync();
    repo
}

#[test]
fn verify_prints_static_results_without_git_or_consumer_execution() {
    let repo = fixture();
    let before = repo.snapshot();
    let opened = raf_workplan::Gate::workspace(repo.root()).open().unwrap();
    let expected = raf_workplan::lifecycle::verify(&opened, Some("shape")).unwrap();
    let run = repo
        .both(&["verify", "shape"])
        .expect(i32::from(!expected.is_valid()));
    assert_eq!(run.stdout, expected.to_string());
    assert!(
        run.stdout
            .contains("declared cap:fixture tests `demo::tests::fails`")
    );
    assert!(
        run.stdout
            .contains("declared cap:fixture examples `demo::example_crate::exiting`")
    );
    assert!(
        !run.stdout
            .lines()
            .any(|line| line.starts_with("pass ") || line.starts_with("fail "))
    );
    assert_eq!(repo.snapshot(), before);
    repo.both(&["verify", "missing"]).expect(2);
    assert_eq!(repo.snapshot(), before);
}

#[test]
fn trusted_docs_is_refused_before_root_discovery_and_general_execution_is_absent() {
    let repo = fixture();
    let before = repo.snapshot();
    let run = repo
        .run(&[
            "--root",
            "/raf-absent-workspace",
            "docs",
            "build",
            "rustdoc",
            "--package",
            "demo",
            "--target-directory",
            "/raf-absent-output",
            "--toolchain-directory",
            "/raf-absent-toolchain",
        ])
        .expect(2);
    assert!(run.stderr.contains("--trust-docs"), "{run:?}");
    assert!(!run.stderr.contains("cannot open discovery"), "{run:?}");
    repo.run(&["evidence", "record", "--", "true"]).expect(2);
    assert_eq!(repo.snapshot(), before);
}
