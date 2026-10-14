// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The graph: every kind of node and every kind of edge, section-granular
//! links, and the queries over it.

mod support;

use std::collections::BTreeSet;

use raf_okf::CapabilitySection as S;
use raf_source::{ItemId, MemoryTree, ResolveError, Workspace};
use raf_workplan::{BindingKind, Edge, EdgeKind, LinkEnd, Node, NodeKind, SectionId, Workplan};

const ALPHA: &str = "docs/capabilities/alpha.md";
const BETA: &str = "docs/capabilities/beta.md";
const CHANGE: &str = "work/changes/add-alpha.md";
const OVERVIEW: &str = "docs/architecture/overview.md";

const CASE: &str = "demo::test_crate::it::case";
const OTHER: &str = "demo::test_crate::it::other";
const THIRD: &str = "demo::test_crate::it::third";
const FOURTH: &str = "demo::test_crate::it::fourth";
const ABSENT: &str = "demo::test_crate::it::absent";

/// Require that a list holds nothing.
fn assert_none<T: std::fmt::Debug>(items: impl AsRef<[T]>) {
    let items = items.as_ref();
    assert_eq!(items.len(), 0, "{items:?}");
}

fn workspace() -> Workspace {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        ),
        (
            "src/lib.rs",
            "pub fn real() {}\npub mod inner { pub fn go() {} }\npub use inner::go as run;\n",
        ),
        (
            "tests/it.rs",
            "#[test]\nfn case() {}\n#[test]\nfn other() {}\n#[test]\nfn third() {}\n#[test]\nfn fourth() {}\n",
        ),
        ("examples/show.rs", "fn main() {}\n"),
    ]);
    Workspace::load(&tree, &["."]).expect("the workspace loads")
}

fn capability(slug: &str, front: &str, sections: [&str; 7]) -> String {
    let mut body = String::new();
    for (name, text) in support::fixtures::SECTIONS.iter().zip(sections) {
        body.push_str(&["## ", name, "\n\n", text, "\n\n"].concat());
    }
    format!(
        "---\ntype: Capability\nid: cap:{slug}\ntitle: Title of {slug}\n\
         description: A description of {slug}.\nstate: implementing\n{front}---\n\n# {slug}\n\n{body}"
    )
}

fn repo() -> support::Repo {
    let alpha = capability(
        "alpha",
        &format!(
            "bindings:\n  code: [demo::real, demo::run]\n  tests: [{CASE}, {OTHER}, {THIRD}, {FOURTH}, {ABSENT}]\n  \
             examples: [demo::example_crate::show]\ndepends: [cap:beta]\n"
        ),
        [
            "Why alpha exists. Jump to the [contract](#contract).",
            &format!(
                "See [beta usage](beta.md#usage), [the overview](../architecture/overview.md) and \
                 [the change](../../work/changes/add-alpha.md). The test {OTHER} is named here in prose only.\n\n\
                 > [!PROPOSED change:add-alpha]\n> Cover `{THIRD}` too."
            ),
            &format!("Run `{CASE}` to see it.\n\n```text\nuse `{FOURTH}` here\n```"),
            &format!(
                "Run the example `demo::example_crate::show`, not `{CASE}_extra` or `other::{CASE}`."
            ),
            &format!("Also `{CASE}` and `{ABSENT}` hold."),
            "Why.",
            "Links to [beta](beta.md) as a whole and [alpha contract](#contract).",
        ],
    );
    let beta = capability(
        "beta",
        &format!(
            "bindings:\n  code: [demo::inner::go, demo::gone]\n  tests: [{CASE}]\n  examples: []\ndepends: []\n"
        ),
        [
            "Beta.",
            "Contract text.",
            "Back to [alpha contract](alpha.md#contract).",
            "Ex.",
            "Con.",
            "Why.",
            "Ver.",
        ],
    );
    let change = "---\ntype: Change\nid: change:add-alpha\ntitle: Add alpha\n\
        description: Adds alpha.\nchanges: [cap:alpha, cap:beta]\n\
        affects:\n  - cap:beta#usage\n  - cap:alpha#contract\n  - cap:alpha#contract\n\
        contract_effect: changes\nitems:\n  - claim: \"alpha reads input\"\n---\n\n## Summary\n\nAdds both.\n"
        .to_string();
    let overview = "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
        description: How it fits.\n---\n\n# Overview\n\nSee [alpha contract](../capabilities/alpha.md#contract) and [beta](../capabilities/beta.md).\n"
        .to_string();
    let mut repo = support::Repo::new();
    repo.insert(ALPHA.to_string(), alpha);
    repo.insert(BETA.to_string(), beta);
    repo.insert(CHANGE.to_string(), change);
    repo.insert(OVERVIEW.to_string(), overview);
    repo
}

fn with_plan<T>(repo: &support::Repo, check: impl FnOnce(&Workplan<'_>, &Workspace) -> T) -> T {
    let tree: MemoryTree = repo.iter().collect();
    let workspace = workspace();
    let plan = Workplan::load(&tree, &workspace);
    check(&plan, &workspace)
}

fn item(workspace: &Workspace, path: &str) -> ItemId {
    workspace.resolve(path).expect(path).sites[0]
}

fn paths(workspace: &Workspace, items: &[ItemId]) -> Vec<String> {
    items
        .iter()
        .map(|id| workspace.item(*id).expect("item").path.to_string())
        .collect()
}

#[test]
fn the_fixture_loads_without_a_finding() {
    with_plan(&repo(), |plan, _| {
        assert!(plan.findings().is_empty(), "{:#?}", plan.findings());
    });
}

#[test]
fn the_graph_has_every_kind_of_node() {
    with_plan(&repo(), |plan, workspace| {
        let nodes = plan.graph().nodes();
        let kinds: BTreeSet<_> = nodes.iter().map(|n| format!("{:?}", n.kind())).collect();
        for kind in NodeKind::ALL {
            assert!(kinds.contains(&format!("{kind:?}")), "no {kind:?} node");
        }
        assert!(nodes.contains(&Node::Capability("cap:alpha".into())));
        assert!(nodes.contains(&Node::Change("change:add-alpha".into())));
        assert!(nodes.contains(&Node::Architecture("arch:overview".into())));
        let sections = nodes
            .iter()
            .filter(|n| n.kind() == NodeKind::Section)
            .count();
        assert_eq!(sections, 14, "seven for each of two capabilities");
        for section in S::ALL {
            assert!(nodes.contains(&Node::Section(SectionId::new("cap:beta", section))));
        }
        assert_eq!(plan.graph().proposals().len(), 1);
        let proposal = &plan.graph().proposals()[0];
        assert_eq!(
            (
                proposal.capability(),
                proposal.change(),
                proposal.section(),
                proposal.ordinal()
            ),
            ("cap:alpha", "add-alpha", Some(S::Contract), 0)
        );
        assert!(nodes.contains(&Node::Proposal(proposal.clone())));
        assert!(nodes.contains(&Node::Symbol(item(workspace, "demo::real"))));
        assert!(nodes.contains(&Node::Test(item(workspace, CASE))));
        assert!(nodes.contains(&Node::Example(item(workspace, "demo::example_crate::show"))));
        // The unresolved bindings make no node, and no item is a node twice.
        let unique: Vec<&Node> = nodes.iter().collect();
        assert!(
            unique
                .iter()
                .enumerate()
                .all(|(i, n)| !unique[..i].contains(n))
        );
    });
}

#[test]
fn the_graph_has_every_kind_of_edge() {
    with_plan(&repo(), |plan, _| {
        let edges = plan.graph().edges();
        for kind in EdgeKind::ALL {
            assert!(edges.iter().any(|e| e.kind() == kind), "no {kind:?} edge");
        }
        let count = |kind| edges.iter().filter(|e| e.kind() == kind).count();
        // alpha binds 2 + 5 + 1, beta 2 + 1.
        assert_eq!(count(EdgeKind::Binding), 11);
        assert_eq!(count(EdgeKind::Depends), 1);
        assert_eq!(count(EdgeKind::Changes), 2);
        // The duplicated `affects` entry is one edge.
        assert_eq!(count(EdgeKind::Affects), 2);
        assert!(matches!(&edges[0], Edge::Binding(b) if b.capability() == "cap:alpha"));
        let depends = &plan.graph().depends()[0];
        assert_eq!((depends.from(), depends.to()), ("cap:alpha", "cap:beta"));
        let changes = &plan.graph().changes_edges()[0];
        assert_eq!(
            (changes.change(), changes.capability()),
            ("change:add-alpha", "cap:alpha")
        );
        let affects = &plan.graph().affects()[0];
        assert_eq!(affects.change(), "change:add-alpha");
        assert_eq!(affects.section(), &SectionId::new("cap:alpha", S::Contract));
    });
}

#[test]
fn bindings_record_what_the_source_index_made_of_each_path() {
    with_plan(&repo(), |plan, workspace| {
        let find = |capability: &str, path: &str| {
            plan.graph()
                .bindings()
                .iter()
                .find(|b| b.capability() == capability && b.path() == path)
                .unwrap_or_else(|| panic!("no binding {path}"))
        };
        let run = find("cap:alpha", "demo::run");
        assert_eq!(run.kind(), BindingKind::Code);
        // A re-export resolves to the definition.
        assert_eq!(paths(workspace, run.items()), ["demo::inner::go"]);
        assert_eq!(run.resolution().expect("resolves"), run.items());
        let gone = find("cap:beta", "demo::gone");
        assert!(matches!(gone.resolution(), Err(ResolveError::NotFound(_))));
        assert_none(gone.items());
        let absent = find("cap:alpha", ABSENT);
        assert_eq!(absent.kind(), BindingKind::Test);
        assert!(absent.resolution().is_err());
        assert_eq!(
            find("cap:alpha", "demo::example_crate::show").kind(),
            BindingKind::Example
        );
        // A code binding substantiates no section.
        assert_none(run.sections());
    });
}

#[test]
fn links_run_from_the_section_that_holds_them_to_the_section_they_name() {
    with_plan(&repo(), |plan, _| {
        let s = |cap: &str, section| LinkEnd::Section(SectionId::new(cap, section));
        let links: Vec<(LinkEnd, LinkEnd)> = plan
            .graph()
            .links()
            .iter()
            .map(|l| (l.from().clone(), l.to().clone()))
            .collect();
        let has = |from: &LinkEnd, to: &LinkEnd| links.contains(&(from.clone(), to.clone()));
        // Section to section, by anchor.
        assert!(has(&s("cap:alpha", S::Contract), &s("cap:beta", S::Usage)));
        assert!(has(&s("cap:beta", S::Usage), &s("cap:alpha", S::Contract)));
        // A fragment is a link into the same document.
        assert!(has(
            &s("cap:alpha", S::Purpose),
            &s("cap:alpha", S::Contract)
        ));
        assert!(has(
            &s("cap:alpha", S::Verification),
            &s("cap:alpha", S::Contract)
        ));
        // No anchor: the target document.
        assert!(has(
            &s("cap:alpha", S::Verification),
            &LinkEnd::Capability("cap:beta".into())
        ));
        assert!(has(
            &s("cap:alpha", S::Contract),
            &LinkEnd::Architecture("arch:overview".into())
        ));
        assert!(has(
            &s("cap:alpha", S::Contract),
            &LinkEnd::Change("change:add-alpha".into())
        ));
        // A page outside the sections links as a document.
        assert!(has(
            &LinkEnd::Architecture("arch:overview".into()),
            &s("cap:alpha", S::Contract)
        ));
        assert!(has(
            &LinkEnd::Architecture("arch:overview".into()),
            &LinkEnd::Capability("cap:beta".into())
        ));
        assert_eq!(links.len(), 9);
    });
}

#[test]
fn a_link_that_leads_nowhere_in_the_model_is_no_edge() {
    let mut repo = repo();
    let text = repo.get_mut(OVERVIEW).expect("overview");
    text.push_str(
        "\n[web](https://example.com/x) [missing](nothing.md) [bad](../capabilities/alpha.md#nope)\n",
    );
    with_plan(&repo, |plan, _| {
        assert_eq!(plan.graph().links().len(), 9);
    });
}

#[test]
fn code_of_lists_the_items_that_realise_a_capability_in_path_order() {
    with_plan(&repo(), |plan, workspace| {
        let alpha = plan.graph().code_of("cap:alpha");
        assert_eq!(paths(workspace, &alpha), ["demo::inner::go", "demo::real"]);
        // Unresolved paths realise nothing.
        let beta = plan.graph().code_of("cap:beta");
        assert_eq!(paths(workspace, &beta), ["demo::inner::go"]);
        assert_none(plan.graph().code_of("cap:ghost"));
        // Tests are not code.
        assert!(!alpha.contains(&item(workspace, CASE)));
    });
}

#[test]
fn capabilities_of_names_every_capability_bound_to_a_symbol() {
    with_plan(&repo(), |plan, workspace| {
        let go = item(workspace, "demo::inner::go");
        assert_eq!(plan.graph().capabilities_of(go), ["cap:alpha", "cap:beta"]);
        let real = item(workspace, "demo::real");
        assert_eq!(plan.graph().capabilities_of(real), ["cap:alpha"]);
        let case = item(workspace, CASE);
        assert_eq!(
            plan.graph().capabilities_of(case),
            ["cap:alpha", "cap:beta"]
        );
        let example = item(workspace, "demo::example_crate::show");
        assert_eq!(plan.graph().capabilities_of(example), ["cap:alpha"]);
        let unbound = workspace
            .items()
            .map(|i| i.id)
            .find(|id| plan.graph().capabilities_of(*id).is_empty());
        assert!(unbound.is_some());
    });
}

fn tests_in(plan: &Workplan<'_>, cap: &str, section: S) -> Vec<String> {
    plan.graph()
        .tests_for(cap, section)
        .iter()
        .map(|b| b.path().to_string())
        .collect()
}

#[test]
fn tests_for_a_binding_named_in_two_sections_substantiates_both() {
    with_plan(&repo(), |plan, _| {
        assert_eq!(tests_in(plan, "cap:alpha", S::Usage), [CASE]);
        assert_eq!(tests_in(plan, "cap:alpha", S::Constraints), [ABSENT, CASE]);
        let case = plan
            .graph()
            .bindings()
            .iter()
            .find(|b| b.capability() == "cap:alpha" && b.path() == CASE)
            .expect("binding");
        assert_eq!(case.sections(), [S::Usage, S::Constraints]);
        // The same path bound by another capability whose text does not name
        // it substantiates that capability's Verification.
        assert_eq!(tests_in(plan, "cap:beta", S::Verification), [CASE]);
        assert_none(tests_in(plan, "cap:beta", S::Usage));
    });
}

#[test]
fn tests_for_a_binding_named_only_in_prose_substantiates_verification() {
    with_plan(&repo(), |plan, _| {
        let verification = tests_in(plan, "cap:alpha", S::Verification);
        assert!(
            verification.contains(&OTHER.to_string()),
            "{verification:?}"
        );
        assert!(!tests_in(plan, "cap:alpha", S::Contract).contains(&OTHER.to_string()));
    });
}

#[test]
fn tests_for_ignores_a_name_inside_a_proposed_block_or_a_fence() {
    with_plan(&repo(), |plan, _| {
        let verification = tests_in(plan, "cap:alpha", S::Verification);
        assert!(
            verification.contains(&THIRD.to_string()),
            "{verification:?}"
        );
        assert!(
            verification.contains(&FOURTH.to_string()),
            "{verification:?}"
        );
        assert!(!tests_in(plan, "cap:alpha", S::Contract).contains(&THIRD.to_string()));
        assert!(!tests_in(plan, "cap:alpha", S::Usage).contains(&FOURTH.to_string()));
        // Sorted by path.
        assert_eq!(verification, [FOURTH, OTHER, THIRD]);
    });
}

#[test]
fn tests_for_a_path_that_does_not_resolve_is_still_a_binding_of_its_sections() {
    with_plan(&repo(), |plan, _| {
        assert!(tests_in(plan, "cap:alpha", S::Constraints).contains(&ABSENT.to_string()));
        assert_none(tests_in(plan, "cap:alpha", S::Examples));
    });
}

#[test]
fn sections_of_a_change_are_its_affected_sections_sorted() {
    with_plan(&repo(), |plan, _| {
        assert_eq!(
            plan.graph().sections_of("change:add-alpha"),
            [
                SectionId::new("cap:alpha", S::Contract),
                SectionId::new("cap:beta", S::Usage)
            ]
        );
        assert_none(plan.graph().sections_of("change:ghost"));
    });
}

#[test]
fn linking_sections_are_section_granular() {
    with_plan(&repo(), |plan, _| {
        let contract = SectionId::new("cap:alpha", S::Contract);
        assert_eq!(
            plan.graph().linking_sections(&contract),
            [
                SectionId::new("cap:alpha", S::Purpose),
                SectionId::new("cap:alpha", S::Verification),
                SectionId::new("cap:beta", S::Usage),
            ]
        );
        // The architecture page links to the section from outside every
        // section, so it is not a linking section.
        let beta_usage = SectionId::new("cap:beta", S::Usage);
        assert_eq!(
            plan.graph().linking_sections(&beta_usage),
            [SectionId::new("cap:alpha", S::Contract)]
        );
        // A link to a whole document is not a link to one of its sections.
        let beta_purpose = SectionId::new("cap:beta", S::Purpose);
        assert_none(plan.graph().linking_sections(&beta_purpose));
        let alpha_usage = SectionId::new("cap:alpha", S::Usage);
        assert_none(plan.graph().linking_sections(&alpha_usage));
    });
}

#[test]
fn the_queries_are_deterministic() {
    let repo = repo();
    let first = with_plan(&repo, |plan, _| {
        (plan.graph().nodes(), plan.graph().edges())
    });
    let second = with_plan(&repo, |plan, _| {
        (plan.graph().nodes(), plan.graph().edges())
    });
    assert_eq!(first, second);
}
