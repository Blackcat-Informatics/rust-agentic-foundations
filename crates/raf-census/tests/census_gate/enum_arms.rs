// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One physical catch-all inventory through required, forbidden and counted policy.

use super::{Gate, Term, base, detector_fixtures, job, put, tree};

/// The required inventory coexists with numeric bounds, and job policy counts
/// both arms in the home package.
#[test]
fn enum_catch_alls_share_the_required_forbidden_and_ratcheted_routes() {
    let id = detector_fixtures::ENUM_MATCH.0;
    let source = r"
        /// The job's approved home.
        pub fn home() {}
        pub use home as alias;
        enum State { Ready(u8), Busy, Idle }
        impl State {
            pub fn inspect(value: State, next: Self) {
                match value { State::Ready { 0: _ } => (), | _ => () }
                assert!(match next { Self::Busy {} => true, remainder => true });
            }
        }
        const WORK_LIMIT: usize = 9;
        fn exceeds_limit(items: &[u8]) -> bool { items.len() > WORK_LIMIT }
    ";
    let numeric = super::numeric_bounds::registry(base(), "strict");
    let required = Gate::workspace(".")
        .require_rules([id])
        .check_tree(&tree(source, numeric, 0))
        .unwrap();
    let findings = required.findings();
    assert_eq!(findings.len(), 3, "{findings:?}");
    assert!(findings[0].contains("src/lib.rs:8:"));
    assert!(findings[1].contains("src/lib.rs:9:"));
    assert!(findings[2].contains("unclassified numeric bound"));
    let [limit] = required.numeric_bounds() else {
        panic!(
            "expected one numeric bound: {:?}",
            required.numeric_bounds()
        );
    };
    assert!(source[limit.start_byte..].starts_with("WORK_LIMIT"));
    assert!(limit.roles.contains(&raf_census::BoundRole::Comparison));

    let policy = || {
        let mut graph = job(true);
        put(
            &mut graph,
            "#job",
            "forbiddenFingerprint",
            Term::simple_literal(format!("rule:{id}")),
        );
        graph
    };
    let forbidden = Gate::workspace(".")
        .check_tree(&tree(source, policy(), 0))
        .unwrap();
    assert_eq!(forbidden.jobs()[0].copies, 1);
    assert_eq!(
        forbidden
            .findings()
            .iter()
            .filter(|finding| finding.contains(id))
            .count(),
        2
    );

    let mut ratcheted = policy();
    put(
        &mut ratcheted,
        "#job",
        "ratchetedRule",
        Term::simple_literal(id),
    );
    let report = Gate::workspace(".")
        .check_tree(&tree(source, ratcheted, 0))
        .unwrap();
    assert!(report.debt().ratcheted().contains(id));
    assert_eq!(report.debt().sites().values().copied().sum::<usize>(), 2);
    assert_eq!(report.findings().len(), 1, "{:?}", report.findings());
    assert!(report.findings()[0].contains("adopt the newly ratcheted rule"));
}

/// One shared arm needs enum evidence at each source reach and reports once.
#[test]
fn enum_catch_alls_keep_only_arms_proved_across_shared_reaches() {
    let ledger = base()
        .finish()
        .unwrap()
        .to_text(super::Format::Turtle)
        .unwrap();
    let check = |right| {
        let tree = super::MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='demo'\nversion='0.1.0'\nedition='2024'\n",
            ),
            ("src/lib.rs", "mod left; mod right;"),
            (
                "src/left.rs",
                "enum State { Ready, Busy } use State as Input; #[path=\"shared.rs\"] mod shared;",
            ),
            ("src/right.rs", right),
            (
                "src/shared.rs",
                "fn inspect(value: super::Input) { match value { super::State::Ready => (), _ => () } }\n\
                 impl super::Input { fn inspect(self) { match self { Self::Ready => (), _ => () } } }",
            ),
            ("helpers-ledger.ttl", ledger.as_str()),
        ]);
        Gate::workspace(".")
            .require_rules([detector_fixtures::ENUM_MATCH.0])
            .check_tree(&tree)
            .unwrap()
    };
    let enums =
        check("enum State { Ready, Busy } use State as Input; #[path=\"shared.rs\"] mod shared;");
    assert_eq!(enums.findings().len(), 2, "{:?}", enums.findings());
    for line in ["src/shared.rs:1:", "src/shared.rs:2:"] {
        assert!(
            enums
                .findings()
                .iter()
                .any(|finding| finding.contains(line)),
            "{:?}",
            enums.findings()
        );
    }

    let mixed = check(
        "#[derive(PartialEq, Eq)] struct Input; impl Input { const Ready: Self = Self; }
         mod State { pub const Ready: super::Input = super::Input; }
         #[path=\"shared.rs\"] mod shared;",
    );
    assert!(mixed.is_clean(), "{:?}", mixed.findings());
}
