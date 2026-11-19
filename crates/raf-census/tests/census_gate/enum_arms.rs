// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Enum policy through the production gate.

use super::{Gate, Term, base, detector_fixtures, job, put, tree};

/// Only a resolved shipping expression discharges a variant's use obligation.
#[test]
fn designated_enum_requires_shipping_values_and_current_retentions() {
    let source = r#"
        pub enum State { Unit, Tuple(u8), Named { value: u8 }, Idle }
        mod names { pub use crate::State::Tuple as New; }
        mod exports { pub use crate::names::New as Make; }
        use exports::Make as Constructor;
        macro_rules! ignore { ($value:expr) => { () }; }
        type StateAlias = State;
        const INITIAL: State = <StateAlias>::Unit;
        fn constructor() -> fn(u8) -> State { Constructor }
        impl State {
            fn next() -> Self { Self::Named { value: 1 } }
        }
        fn inspect(value: State) {
            match value { State::Idle => (), _ => () }
            ignore!(State::Idle);
            #[cfg(any())] let _ = State::Idle;
        }
        #[doc = stringify!(State::Idle)] fn documented() {}
        #[test] fn test_only() { let _ = State::Idle; }
        #[cfg(any())] mod disabled;
        enum Marker {
            #[cfg(test)] Probe = { let _ = State::Idle; 1 },
            Present = 2,
        }
        struct Header {
            #[cfg(test)] field: [u8; { let _ = State::Idle; 0 }],
        }
        fn configured_input(#[cfg(test)] _: [u8; { let _ = State::Idle; 0 }]) {}
        pub mod fast_route { pub use crate::State as Selected; }
        mod fallback {
            pub enum Selected { Idle }
            pub mod slow_route { pub use super::Selected; }
            pub use self::slow_route as Route;
        }
        mod facade {
            #[cfg(feature = "fast")] pub use crate::State as Selected;
            #[cfg(feature = "fast")] pub use crate::fast_route as Route;
            pub use crate::fallback::*;
            pub use self::Route::Selected as Choice;
        }
        fn conditional_route() { let _ = facade::Selected::Idle; }
        fn conditional_reexport() { let _ = facade::Choice::Idle; }
        fn conditional_glob() { use facade::Selected::*; let _ = Idle; }
        fn conditional_glob_type() { use facade::*; let _ = Selected::Idle; }
        fn conditional_chained() { use facade::*; use Selected::*; let _ = Idle; }
    "#;
    let check = |source: &str, retentions: &[(&str, &str)]| {
        let mut fixture = tree(source, enum_policy(retentions), 0);
        fixture
            .create_new(
                &raf_source::RelPath::new("src/disabled.rs").unwrap(),
                "fn disabled() -> crate::State { crate::State::Idle }".into(),
            )
            .unwrap();
        Gate::workspace(".").check_tree(&fixture)
    };
    let missing = check(source, &[]).unwrap();
    let [finding] = missing.findings() else {
        panic!("expected one unused variant: {:?}", missing.findings());
    };
    assert!(finding.contains("demo::State::Idle: no confirmed shipping value use"));

    let retained = [("Idle", "Reserved for the decoder")];
    assert!(check(source, &retained).unwrap().is_clean());
    let shipping = format!("{source}\nimpl StateAlias {{ fn idle() -> Self {{ Self::Idle }} }}");
    assert!(check(&shipping, &[]).unwrap().is_clean());
    let stale = check(&shipping, &retained).unwrap();
    let [finding] = stale.findings() else {
        panic!("expected one stale retention: {:?}", stale.findings());
    };
    assert!(finding.contains("demo::State::Idle: retention is stale"));
    for route in [
        "mod idle { pub use crate::State::Idle as Rest; } use idle::Rest;
        fn copy(value: State) -> State { match value { Rest => Rest, other => other } }",
        "mod exported { pub use crate::State::Idle; }
        fn imported() -> State { use exported::*; Idle }",
        "mod all { pub use crate::State::*; } mod other { pub enum Other { Idle { code: u8 } } }
        mod front { pub use crate::all::*; pub use crate::other::Other::*; }
        fn imported() -> State { front::Idle }",
        "mod kinds { pub use crate::State; } mod api { pub use crate::kinds::*; }
        fn imported() -> State { api::State::Idle }",
        "use crate as here; fn imported() -> State { here::State::Idle }",
    ] {
        let verdict = check(&format!("{source}\n{route}"), &[]).unwrap();
        assert!(verdict.is_clean(), "{route}: {:?}", verdict.findings());
    }

    for (retentions, expected) in [
        (
            &[("Missing", "Reserved")][..],
            "unknown variant demo::State::Missing",
        ),
        (&[("Idle", "")][..], "reason"),
        (
            &[("Idle", "Reserved"), ("r#Idle", "Also reserved")][..],
            "retention for variant Idle",
        ),
    ] {
        let error = check(source, retentions).unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
    }
}

/// Destructuring constructor heads behave as patterns even in assignment syntax.
#[test]
fn enum_assignees_preserve_real_value_expressions_in_places() {
    let source = r"
        enum State { Pair(u8) }
        fn unpack(value: State) {
            let mut element;
            State::Pair(element) = value;
        }
    ";
    let check = |source| {
        Gate::workspace(".")
            .check_tree(&tree(source, enum_policy(&[]), 0))
            .unwrap()
    };
    let report = check(source);
    let [missing] = report.findings() else {
        panic!("expected one unused constructor: {:?}", report.findings());
    };
    assert!(missing.contains("demo::State::Pair: no confirmed shipping value use"));

    let shipping = format!(
        "{source}\nfn write(array: &mut [u8], at: fn(State) -> usize) {{ array[at(State::Pair(0))] = 1; }}"
    );
    assert!(check(&shipping).is_clean());
}

pub(super) fn enum_policy(retentions: &[(&str, &str)]) -> super::Builder {
    use super::vocab::REPORT;

    let mut graph = base();
    put(&mut graph, "", "enumUsage", Term::iri("#enum"));
    put(
        &mut graph,
        "#enum",
        "type",
        Term::iri(format!("{REPORT}EnumUsage")),
    );
    put(
        &mut graph,
        "#enum",
        "symbol",
        Term::simple_literal("demo::State"),
    );
    for (index, (member, reason)) in retentions.iter().enumerate() {
        let row = format!("#enum/retained-{index}");
        put(&mut graph, "#enum", "retainedVariant", Term::iri(&row));
        put(
            &mut graph,
            &row,
            "type",
            Term::iri(format!("{REPORT}EnumVariantRetention")),
        );
        put(
            &mut graph,
            &row,
            "variantName",
            Term::simple_literal(*member),
        );
        put(&mut graph, &row, "reason", Term::simple_literal(*reason));
    }
    graph
}

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
