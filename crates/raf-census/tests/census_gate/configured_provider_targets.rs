// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Preferred-provider proof follows every selected target and physical reach.

use raf_source::{
    CfgAtom, DeclaredCfg, DeclaredConfiguration, LoadOptions, LockedCatalog, LockedEdge,
    LockedManifest, MemoryTree, RelPath, TargetKind,
};

use super::{Builder, Format, Gate, Term, Triple, provider_policy, put, vocab};

const ORDINARY: &[CfgAtom<'_>] = &[CfgAtom::Name("unix")];
const TEST: &[CfgAtom<'_>] = &[CfgAtom::Name("unix"), CfgAtom::Name("test")];

fn context(kind: TargetKind, name: &str, test: bool) -> DeclaredConfiguration<'_> {
    let configuration = DeclaredConfiguration::new(
        DeclaredCfg::complete("fixture-target", if test { TEST } else { ORDINARY }),
        &[],
        kind,
        name,
    );
    if kind == TargetKind::BuildScript {
        configuration.with_host(DeclaredCfg::complete("fixture-target", ORDINARY))
    } else {
        configuration
    }
}

struct Selected {
    kind: TargetKind,
    consumer: &'static str,
    declaration: &'static str,
    path: &'static str,
    test: bool,
}

const TARGETS: &[Selected] = &[
    Selected {
        kind: TargetKind::BuildScript,
        consumer: "demo::build_script::run",
        declaration: "",
        path: "consumer/build.rs",
        test: false,
    },
    Selected {
        kind: TargetKind::Bin,
        consumer: "demo::bin_crate::selected::run",
        declaration: "[[bin]]\nname='selected'\npath='src/main.rs'\n",
        path: "consumer/src/main.rs",
        test: false,
    },
    Selected {
        kind: TargetKind::Bin,
        consumer: "demo::bin_crate::selected::run",
        declaration: "[[bin]]\nname='selected'\npath='src/main.rs'\n",
        path: "consumer/src/main.rs",
        test: true,
    },
    Selected {
        kind: TargetKind::Test,
        consumer: "demo::test_crate::selected::run",
        declaration: "[[test]]\nname='selected'\npath='tests/integration.rs'\n",
        path: "consumer/tests/integration.rs",
        test: true,
    },
    Selected {
        kind: TargetKind::Bench,
        consumer: "demo::bench_crate::selected::run",
        declaration: "[[bench]]\nname='selected'\npath='benches/measure.rs'\nharness=false\n",
        path: "consumer/benches/measure.rs",
        test: false,
    },
    Selected {
        kind: TargetKind::Example,
        consumer: "demo::example_crate::selected::run",
        declaration: "[[example]]\nname='selected'\npath='examples/sample.rs'\n",
        path: "consumer/examples/sample.rs",
        test: false,
    },
];

fn policy(selected: &Selected) -> String {
    let mut policy: Builder = provider_policy("provider", "native_library::api::encode");
    for (subject, key, old) in [
        ("#provider", "consumer", "demo::run"),
        ("#job", "homeSymbol", "demo::alias"),
    ] {
        policy
            .remove(Triple {
                subject: Term::iri(subject),
                predicate: Term::iri(format!("{}{key}", vocab::REPORT)),
                object: Term::simple_literal(old),
            })
            .unwrap();
        put(
            &mut policy,
            subject,
            key,
            Term::simple_literal(selected.consumer),
        );
    }
    policy.finish().unwrap().to_text(Format::Turtle).unwrap()
}

fn check(
    selected: &Selected,
    dependency: &str,
    library: &str,
    program: &str,
    shared: bool,
) -> Result<raf_census::GateReport, raf_census::Error> {
    let manifest = format!(
        "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n[lib]\nname='demo_native'\n{}[{dependency}]\nrenamed={{package='provider',path='../provider',version='1'}}\n",
        selected.declaration
    );
    let ledger = policy(selected);
    let shared_root = "#[path=\"../shared.rs\"] mod shared; pub use shared::run; fn main() {}";
    let tree = MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['consumer','provider']"),
        ("consumer/Cargo.toml", manifest.as_str()),
        (
            "consumer/src/lib.rs",
            if shared { shared_root } else { library },
        ),
        (
            selected.path,
            if shared {
                if selected.kind == TargetKind::BuildScript {
                    "#[path=\"shared.rs\"] mod shared; pub use shared::run; fn main() {}"
                } else {
                    shared_root
                }
            } else {
                program
            },
        ),
        ("consumer/shared.rs", program),
        (
            "provider/Cargo.toml",
            "[package]\nname='provider'\nversion='1.0.0'\nedition='2024'\n[lib]\nname='native_library'",
        ),
        (
            "provider/src/lib.rs",
            "pub mod api { pub fn encode(value:u32)->u32 {value+1} }",
        ),
        (
            "Cargo.lock",
            "version=4\n[[package]]\nname='demo'\nversion='1.0.0'\ndependencies=['provider']\n[[package]]\nname='provider'\nversion='1.0.0'",
        ),
        ("helpers-ledger.ttl", ledger.as_str()),
    ]);
    let catalog = LockedCatalog::load(
        &tree,
        &RelPath::new("Cargo.lock").unwrap(),
        &LoadOptions::default(),
    )
    .unwrap();
    let row = |name| {
        catalog
            .packages()
            .find(|row| row.name() == name)
            .unwrap()
            .id()
    };
    let configs = [
        context(TargetKind::Lib, "demo_native", false),
        context(
            selected.kind,
            if selected.kind == TargetKind::BuildScript {
                "build-script-build"
            } else {
                "selected"
            },
            selected.test,
        ),
    ];
    let edge = format!("{dependency}.renamed");
    let edges = [LockedEdge::new(&edge, None)];
    let manifests = [
        LockedManifest::new(
            row("demo"),
            RelPath::new("consumer/Cargo.toml").unwrap(),
            &edges,
        )
        .with_configurations(&configs),
        LockedManifest::new(
            row("provider"),
            RelPath::new("provider/Cargo.toml").unwrap(),
            &[],
        )
        .with_configuration(context(TargetKind::Lib, "native_library", false)),
    ];
    Gate::workspace(".").check_locked(&tree, &catalog, &manifests, &[])
}

#[test]
fn every_selected_target_proves_native_calls_through_its_ordinary_library() {
    for selected in TARGETS {
        if selected.kind == TargetKind::BuildScript {
            let program = "pub fn run(value:u32)->u32 {renamed::api::encode(value)} fn main() {}";
            let report = check(
                selected,
                "build-dependencies",
                "pub struct Local;",
                program,
                false,
            )
            .unwrap();
            assert!(report.is_clean(), "{:?}", report.findings());
            for dependency in ["dependencies", "dev-dependencies"] {
                let error =
                    check(selected, dependency, "pub struct Local;", program, false).unwrap_err();
                let diagnostic = error.to_string();
                assert!(
                    diagnostic.contains(if dependency == "dependencies" {
                        "no admitted dependency route"
                    } else {
                        "inactive or ineligible"
                    }),
                    "{error}"
                );
            }
            // A build-only edge cannot authorize the same body in the ordinary
            // library, even when its build-script reach is otherwise valid.
            let error = check(selected, "build-dependencies", "", program, true).unwrap_err();
            assert!(
                error.to_string().contains("no admitted dependency route"),
                "{error}"
            );
            let through_library =
                "pub fn run(value:u32)->u32 {demo_native::delegate(value)} fn main() {}";
            assert!(
                check(
                    selected,
                    "dependencies",
                    "pub use renamed::api::encode as delegate;",
                    through_library,
                    false
                )
                .is_err()
            );
            continue;
        }
        let report = check(
            selected,
            "dependencies",
            "pub use renamed::api::encode as delegate;",
            "pub fn run(value:u32)->u32 {demo_native::delegate(value)} fn main() {}",
            false,
        )
        .unwrap();
        assert!(report.is_clean(), "{:?}", report.findings());
    }
}

#[test]
fn development_routes_prove_selected_consumers_but_never_their_ordinary_reach() {
    let program = "pub fn run(value:u32)->u32 {renamed::api::encode(value)} fn main() {}";
    for selected in TARGETS {
        if selected.kind == TargetKind::BuildScript {
            // Its build/ordinary/development route neighbors run together above.
            continue;
        }
        if selected.kind == TargetKind::Bin && !selected.test {
            let error = check(
                selected,
                "dev-dependencies",
                "pub struct Local;",
                program,
                false,
            )
            .unwrap_err();
            assert!(matches!(&error, raf_census::Error::Load(_)), "{error}");
            assert!(
                error.to_string().contains("inactive or ineligible"),
                "{error}"
            );
            continue;
        }
        let report = check(
            selected,
            "dev-dependencies",
            "pub struct Local;",
            program,
            false,
        )
        .unwrap();
        assert!(report.is_clean(), "{:?}", report.findings());
        if !selected.test {
            let error = check(selected, "dev-dependencies", "", program, true).unwrap_err();
            assert!(
                error.to_string().contains("no admitted dependency route"),
                "{error}"
            );
            let report = check(selected, "dependencies", "", program, true).unwrap();
            assert!(report.is_clean(), "{:?}", report.findings());
        }
    }
}
