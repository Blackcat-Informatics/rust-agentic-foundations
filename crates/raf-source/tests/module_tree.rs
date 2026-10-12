// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Following `mod` declarations as rustc does.

use std::cell::RefCell;
use std::fmt::Write as _;

use raf_source::{
    Entry, EntryKind, LoadError, LoadErrorKind, MemoryTree, Origin, RelPath, ResolveError,
    TestRole, Tree, TreeError, Workspace,
};

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";

fn load(files: &[(&str, &str)]) -> Result<Workspace, LoadError> {
    let mut all = vec![("Cargo.toml", DEMO)];
    all.extend_from_slice(files);
    let tree: MemoryTree = all.into_iter().collect();
    Workspace::load(&tree, &["."])
}

fn ok(files: &[(&str, &str)]) -> Workspace {
    load(files).unwrap_or_else(|e| panic!("{e}"))
}

/// Every loaded file with the module paths it was reached under.
fn reached(ws: &Workspace) -> Vec<(String, Vec<String>)> {
    ws.files()
        .map(|f| {
            let mut paths: Vec<String> = f.reached.iter().map(|r| r.module_path.clone()).collect();
            paths.sort();
            (f.path.to_string(), paths)
        })
        .collect()
}

fn files(pairs: &[(&str, &[&str])]) -> Vec<(String, Vec<String>)> {
    pairs
        .iter()
        .map(|(path, modules)| {
            (
                (*path).to_owned(),
                modules.iter().map(|m| (*m).to_owned()).collect(),
            )
        })
        .collect()
}

#[test]
fn a_flat_file_and_a_directory_module_are_both_followed_beside_each_other() {
    let ws = ok(&[
        ("src/lib.rs", "mod flat; mod nested;"),
        ("src/flat.rs", ""),
        ("src/nested/mod.rs", "mod deeper;"),
        ("src/nested/deeper.rs", ""),
    ]);
    assert_eq!(
        reached(&ws),
        files(&[
            ("src/flat.rs", &["demo::flat"]),
            ("src/lib.rs", &["demo"]),
            ("src/nested/deeper.rs", &["demo::nested::deeper"]),
            ("src/nested/mod.rs", &["demo::nested"]),
        ])
    );
}

#[test]
fn a_non_mod_file_keeps_its_children_in_a_directory_of_its_own_name() {
    let ws = ok(&[
        ("src/lib.rs", "mod a;"),
        ("src/a.rs", "mod b;"),
        ("src/a/b.rs", "mod c;"),
        ("src/a/b/c.rs", ""),
    ]);
    let paths: Vec<_> = ws.files().map(|f| f.path.to_string()).collect();
    assert_eq!(
        paths,
        ["src/a.rs", "src/a/b.rs", "src/a/b/c.rs", "src/lib.rs"]
    );
}

#[test]
fn a_module_with_both_name_rs_and_name_mod_rs_is_an_error_naming_both_files() {
    let error = load(&[
        ("src/lib.rs", "mod both;"),
        ("src/both.rs", ""),
        ("src/both/mod.rs", ""),
    ])
    .expect_err("both");
    match &error.kind {
        LoadErrorKind::BothModFiles {
            module,
            first,
            second,
        } => {
            assert_eq!(module, "both");
            assert_eq!(first.as_str(), "src/both.rs");
            assert_eq!(second.as_str(), "src/both/mod.rs");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(error.file.as_str(), "src/lib.rs");
    assert_eq!(error.line, Some(1));
    let text = error.to_string();
    assert!(
        text.contains("src/both.rs") && text.contains("src/both/mod.rs"),
        "{text}"
    );
    // Either one alone is accepted.
    assert!(load(&[("src/lib.rs", "mod both;"), ("src/both.rs", "")]).is_ok());
    assert!(load(&[("src/lib.rs", "mod both;"), ("src/both/mod.rs", "")]).is_ok());
}

#[test]
fn an_unresolved_mod_names_the_declaring_file_and_line() {
    let error = load(&[("src/lib.rs", "// one\n// two\nmod missing;\n")]).expect_err("missing");
    assert!(matches!(&error.kind, LoadErrorKind::UnresolvedMod { module } if module == "missing"));
    assert_eq!(error.file.as_str(), "src/lib.rs");
    assert_eq!(error.line, Some(3));
    assert!(error.to_string().starts_with("src/lib.rs:3:"), "{error}");
    assert!(load(&[("src/lib.rs", "mod present;"), ("src/present.rs", "")]).is_ok());
}

#[test]
fn inline_modules_add_directory_components_for_their_file_children() {
    let ws = ok(&[
        ("src/lib.rs", "mod outer { mod inner { mod leaf; } }"),
        ("src/outer/inner/leaf.rs", ""),
    ]);
    assert_eq!(
        reached(&ws)[0],
        ("src/lib.rs".to_owned(), vec!["demo".to_owned()])
    );
    assert_eq!(
        reached(&ws)[1],
        (
            "src/outer/inner/leaf.rs".to_owned(),
            vec!["demo::outer::inner::leaf".to_owned()]
        )
    );
    // The same module outside the inline blocks looks elsewhere.
    let error = load(&[
        ("src/lib.rs", "mod outer { mod leaf; }"),
        ("src/leaf.rs", ""),
    ])
    .expect_err("wrong place");
    assert!(matches!(error.kind, LoadErrorKind::UnresolvedMod { .. }));
}

#[test]
fn an_inline_module_inside_a_non_mod_file_nests_below_the_files_own_directory() {
    let ws = ok(&[
        ("src/lib.rs", "mod a;"),
        ("src/a.rs", "mod inline { mod leaf; }"),
        ("src/a/inline/leaf.rs", ""),
    ]);
    assert!(
        ws.file(&RelPath::new("src/a/inline/leaf.rs").expect("path"))
            .is_some()
    );
}

#[test]
fn a_path_attribute_in_a_mod_rs_file_is_relative_to_that_files_directory() {
    let ws = ok(&[
        ("src/lib.rs", "#[path = \"elsewhere/x.rs\"] mod x;"),
        ("src/elsewhere/x.rs", ""),
    ]);
    assert!(
        ws.file(&RelPath::new("src/elsewhere/x.rs").expect("path"))
            .is_some()
    );
}

#[test]
fn a_path_attribute_in_a_non_mod_file_is_relative_to_the_files_directory_not_its_module_directory()
{
    let ws = ok(&[
        ("src/lib.rs", "mod a;"),
        ("src/a.rs", "#[path = \"x.rs\"] mod x;"),
        ("src/x.rs", ""),
    ]);
    assert_eq!(
        reached(&ws)
            .into_iter()
            .find(|(path, _)| path == "src/x.rs")
            .map(|(_, modules)| modules),
        Some(vec!["demo::a::x".to_owned()])
    );
    // The module directory `src/a/x.rs` is not consulted for the attribute.
    let error = load(&[
        ("src/lib.rs", "mod a;"),
        ("src/a.rs", "#[path = \"x.rs\"] mod x;"),
        ("src/a/x.rs", ""),
    ])
    .expect_err("only the file directory counts");
    assert!(
        matches!(error.kind, LoadErrorKind::UnresolvedMod { .. }),
        "{error}"
    );
}

#[test]
fn a_path_attribute_inside_an_inline_module_is_relative_to_the_module_directory() {
    // In a mod-rs file.
    let ws = ok(&[
        ("src/lib.rs", "mod inline { #[path = \"x.rs\"] mod x; }"),
        ("src/inline/x.rs", ""),
    ]);
    assert!(
        ws.file(&RelPath::new("src/inline/x.rs").expect("path"))
            .is_some()
    );
    // In a non-mod-rs file the module directory starts with the file's name.
    let ws = ok(&[
        ("src/lib.rs", "mod a;"),
        ("src/a.rs", "mod inline { #[path = \"x.rs\"] mod x; }"),
        ("src/a/inline/x.rs", ""),
    ]);
    assert!(
        ws.file(&RelPath::new("src/a/inline/x.rs").expect("path"))
            .is_some()
    );
}

#[test]
fn a_path_attribute_on_an_inline_module_replaces_its_directory_component() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "#[path = \"custom\"] mod inline { mod leaf; }",
        ),
        ("src/custom/leaf.rs", ""),
    ]);
    assert!(
        ws.file(&RelPath::new("src/custom/leaf.rs").expect("path"))
            .is_some()
    );
    let error = load(&[
        (
            "src/lib.rs",
            "#[path = \"custom\"] mod inline { mod leaf; }",
        ),
        ("src/inline/leaf.rs", ""),
    ])
    .expect_err("the plain name is not used");
    assert!(matches!(error.kind, LoadErrorKind::UnresolvedMod { .. }));
}

#[test]
fn a_file_named_by_a_path_attribute_is_a_mod_rs_file_for_its_children() {
    let ws = ok(&[
        ("src/lib.rs", "#[path = \"sub/explicit.rs\"] mod explicit;"),
        ("src/sub/explicit.rs", "mod child;"),
        ("src/sub/child.rs", ""),
    ]);
    let paths: Vec<_> = ws.files().map(|f| f.path.to_string()).collect();
    assert_eq!(
        paths,
        ["src/lib.rs", "src/sub/child.rs", "src/sub/explicit.rs"]
    );
    // The children are not looked up in `src/sub/explicit/`.
    let error = load(&[
        ("src/lib.rs", "#[path = \"sub/explicit.rs\"] mod explicit;"),
        ("src/sub/explicit.rs", "mod child;"),
        ("src/sub/explicit/child.rs", ""),
    ])
    .expect_err("not there");
    assert!(matches!(error.kind, LoadErrorKind::UnresolvedMod { .. }));
}

#[test]
fn two_unconditional_path_attributes_are_an_error_beside_one() {
    let error = load(&[
        (
            "src/lib.rs",
            "#[path = \"a.rs\"]\n#[path = \"b.rs\"]\nmod x;",
        ),
        ("src/a.rs", ""),
        ("src/b.rs", ""),
    ])
    .expect_err("duplicate");
    assert!(matches!(&error.kind, LoadErrorKind::DuplicatePathAttr { module } if module == "x"));
    assert!(
        matches!(error.origin.as_deref(), Some(Origin::PathAttr { module, .. }) if module == "x")
    );
    assert!(
        load(&[
            ("src/lib.rs", "#[path = \"a.rs\"] mod x;"),
            ("src/a.rs", "")
        ])
        .is_ok()
    );
}

#[test]
fn every_cfg_attr_path_alternative_is_followed_carrying_its_predicate() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "#[cfg_attr(unix, path = \"unix.rs\")]\n#[cfg_attr(windows, path = \"win.rs\")]\nmod sys;",
        ),
        ("src/unix.rs", "pub fn go() {}"),
        ("src/win.rs", "pub fn go() {}"),
    ]);
    let unix = ws
        .file(&RelPath::new("src/unix.rs").expect("path"))
        .expect("loaded");
    assert_eq!(unix.reached[0].predicate.as_deref(), Some("unix"));
    assert!(!unix.reached[0].test);
    let win = ws
        .file(&RelPath::new("src/win.rs").expect("path"))
        .expect("loaded");
    assert_eq!(win.reached[0].predicate.as_deref(), Some("windows"));
    assert_eq!(unix.reached[0].module_path, "demo::sys");
}

#[test]
fn the_default_file_is_also_followed_beside_cfg_attr_alternatives_when_it_exists() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "#[cfg_attr(unix, path = \"unix.rs\")] mod sys;",
        ),
        ("src/unix.rs", ""),
        ("src/sys.rs", ""),
    ]);
    let paths: Vec<_> = ws.files().map(|f| f.path.to_string()).collect();
    assert_eq!(paths, ["src/lib.rs", "src/sys.rs", "src/unix.rs"]);
    // An alternative whose file is missing is an error, whatever the default.
    let error = load(&[
        (
            "src/lib.rs",
            "#[cfg_attr(unix, path = \"unix.rs\")] mod sys;",
        ),
        ("src/sys.rs", ""),
    ])
    .expect_err("alternative missing");
    assert!(matches!(error.kind, LoadErrorKind::UnresolvedMod { .. }));
    assert!(matches!(
        error.origin.as_deref(),
        Some(Origin::PathAttr { predicate: Some(p), .. }) if p == "unix"
    ));
}

#[test]
fn a_cfg_attr_test_path_is_tagged_test_and_nothing_is_dropped() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "#[cfg_attr(test, path = \"tests_impl.rs\")] mod m;",
        ),
        ("src/m.rs", "pub fn real() {}"),
        ("src/tests_impl.rs", "pub fn only_in_tests() {}"),
    ]);
    let alternative = ws
        .file(&RelPath::new("src/tests_impl.rs").expect("path"))
        .expect("loaded");
    assert!(alternative.reached[0].test);
    assert!(alternative.reached_only_through_tests());
    let default = ws
        .file(&RelPath::new("src/m.rs").expect("path"))
        .expect("loaded");
    assert!(!default.reached[0].test);
}

#[test]
fn a_raw_identifier_module_reads_the_plain_file_name() {
    let ws = ok(&[("src/lib.rs", "mod r#match;"), ("src/match.rs", "")]);
    assert_eq!(
        reached(&ws)[1],
        ("src/match.rs".to_owned(), vec!["demo::match".to_owned()])
    );
    let ws = ok(&[
        ("src/lib.rs", "mod r#type { mod r#fn; }"),
        ("src/type/fn.rs", ""),
    ]);
    assert!(
        ws.file(&RelPath::new("src/type/fn.rs").expect("path"))
            .is_some()
    );
    assert!(load(&[("src/lib.rs", "mod r#match;"), ("src/r#match.rs", "")]).is_err());
}

#[test]
fn a_module_cycle_is_an_error_naming_both_files() {
    let error = load(&[("src/lib.rs", "#[path = \"lib.rs\"]\nmod again;")]).expect_err("cycle");
    match &error.kind {
        LoadErrorKind::ModuleCycle { module, again } => {
            assert_eq!(module, "again");
            assert_eq!(again.as_str(), "src/lib.rs");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(error.file.as_str(), "src/lib.rs");
    assert_eq!(error.line, Some(2));

    let error = load(&[
        ("src/lib.rs", "mod a;"),
        ("src/a.rs", "#[path = \"b.rs\"] mod b;"),
        ("src/b.rs", "#[path = \"a.rs\"] mod a;"),
    ])
    .expect_err("longer cycle");
    match &error.kind {
        LoadErrorKind::ModuleCycle { again, .. } => assert_eq!(again.as_str(), "src/a.rs"),
        other => panic!("{other:?}"),
    }
    assert_eq!(error.file.as_str(), "src/b.rs");
}

#[test]
fn a_file_reached_twice_without_a_cycle_is_parsed_once_and_recorded_under_each_path() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "#[path = \"shared.rs\"] mod one;\n#[path = \"shared.rs\"] mod two;",
        ),
        ("src/shared.rs", "pub fn go() {}"),
    ]);
    let shared = ws
        .file(&RelPath::new("src/shared.rs").expect("path"))
        .expect("loaded");
    let paths: Vec<_> = shared
        .reached
        .iter()
        .map(|r| r.module_path.as_str())
        .collect();
    assert_eq!(paths, ["demo::one", "demo::two"]);
    assert_eq!(
        ws.files()
            .filter(|f| f.path.as_str() == "src/shared.rs")
            .count(),
        1
    );
}

#[test]
fn module_nesting_beyond_the_depth_budget_is_refused_and_the_budget_itself_loads() {
    let nest = |levels: usize| {
        // The file-level module path counts as a level of its own.
        format!("{}{}", "mod m { ".repeat(levels), "}".repeat(levels))
    };
    assert!(load(&[("src/lib.rs", &nest(100))]).is_ok());
    let error = load(&[("src/lib.rs", &nest(129))]).err();
    // Beyond the delimiter budget the pre-scan refuses first; either way a
    // budget error names the file.
    let error = error.expect("refused");
    assert!(
        matches!(error.kind, LoadErrorKind::Budget { .. }),
        "{error}"
    );
    // A chain of files hits the module-depth budget.
    let mut chain: Vec<(String, String)> = vec![("src/lib.rs".into(), "mod m0;".into())];
    for index in 0..130 {
        chain.push((
            format!("src/m{index}.rs"),
            format!("#[path = \"m{}.rs\"] mod m{};", index + 1, index + 1),
        ));
    }
    chain.push(("src/m130.rs".into(), String::new()));
    let pairs: Vec<(&str, &str)> = chain
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let error = load(&pairs).expect_err("too deep");
    assert!(
        matches!(
            error.kind,
            LoadErrorKind::Budget {
                budget: raf_source::Budget::ModuleDepth,
                ..
            }
        ),
        "{error}"
    );
    chain.truncate(100);
    let pairs: Vec<(&str, &str)> = chain
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    // The last kept file declares a module that is gone.
    assert!(matches!(
        load(&pairs).expect_err("unresolved").kind,
        LoadErrorKind::UnresolvedMod { .. }
    ));
}

/// A tree that records every path it is asked to read.
struct Recording {
    inner: MemoryTree,
    reads: RefCell<Vec<String>>,
}

impl Tree for Recording {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        self.reads.borrow_mut().push(path.to_string());
        self.inner.read(path, max_bytes)
    }
    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        self.inner.list(path)
    }
    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        self.inner.entry(path)
    }
}

#[test]
fn a_path_that_leaves_the_tree_is_refused_naming_module_and_attribute_without_reading_it() {
    for escape in ["../../x.rs", "/etc/x.rs", "../../../x.rs"] {
        let lib = format!("#[path = \"{escape}\"]\nmod outside;");
        let tree = Recording {
            inner: [
                ("Cargo.toml", DEMO),
                ("src/lib.rs", lib.as_str()),
                ("x.rs", ""),
            ]
            .into_iter()
            .collect(),
            reads: RefCell::new(Vec::new()),
        };
        let result = Workspace::load(&tree, &["."]);
        let error = result.expect_err(escape);
        assert!(
            matches!(&error.kind, LoadErrorKind::Tree(e) if matches!(**e, TreeError::OutsideRoot { .. })),
            "{error}"
        );
        assert_eq!(error.file.as_str(), "src/lib.rs");
        assert_eq!(error.line, Some(2));
        assert!(
            matches!(error.origin.as_deref(), Some(Origin::PathAttr { module, .. }) if module == "outside")
        );
        let text = error.to_string();
        assert!(text.contains("outside") && text.contains("path"), "{text}");
        let reads = tree.reads.borrow();
        assert_eq!(*reads, ["Cargo.toml", "src/lib.rs"], "{escape}");
    }
    // A path that stays inside is read.
    let tree = Recording {
        inner: [
            ("Cargo.toml", DEMO),
            ("src/lib.rs", "#[path = \"../inside.rs\"] mod inside;"),
            ("inside.rs", ""),
        ]
        .into_iter()
        .collect(),
        reads: RefCell::new(Vec::new()),
    };
    Workspace::load(&tree, &["."]).expect("inside path loads");
    assert!(tree.reads.borrow().contains(&"inside.rs".to_owned()));
}

#[test]
fn a_cfg_test_module_tags_every_item_reached_only_through_it() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "pub fn real() {}\n#[cfg(test)] mod t;\nmod live;",
        ),
        ("src/t.rs", "fn helper() {}\nmod deeper;"),
        ("src/t/deeper.rs", "fn more() {}"),
        ("src/live.rs", "pub fn alive() {}"),
    ]);
    for (path, only_tests) in [
        ("src/t.rs", true),
        ("src/t/deeper.rs", true),
        ("src/live.rs", false),
        ("src/lib.rs", false),
    ] {
        let file = ws.file(&RelPath::new(path).expect("path")).expect("loaded");
        assert_eq!(file.reached_only_through_tests(), only_tests, "{path}");
    }
}

#[test]
fn the_cfg_test_matrix_tags_items_and_leaves_their_neighbours_alone() {
    let ws = ok(&[(
        "src/lib.rs",
        "#[cfg(all(test, unix))] fn all_test() {}\n\
         #[cfg(any(test, test))] fn any_test() {}\n\
         #[cfg(any(test, unix))] fn any_mixed() {}\n\
         #[cfg(not(test))] fn not_test() {}\n\
         #[cfg(unix)] fn plain_cfg() {}\n\
         #[test] fn marked() {}\n\
         fn bare() {}\n\
         #[cfg(test)] mod tests { fn inside() {} #[test] fn inner_test() {} }\n",
    )]);
    let file = ws
        .file(&RelPath::new("src/lib.rs").expect("path"))
        .expect("loaded");
    let roles: Vec<(&str, TestRole)> = file
        .facts
        .body
        .items
        .iter()
        .map(|item| (item.name.as_str(), item.test))
        .collect();
    assert_eq!(
        roles,
        [
            ("all_test", TestRole::CfgTest),
            ("any_test", TestRole::CfgTest),
            ("any_mixed", TestRole::None),
            ("not_test", TestRole::None),
            ("plain_cfg", TestRole::None),
            ("marked", TestRole::TestFn),
            ("bare", TestRole::None),
        ]
    );
    let tests = &file.facts.body.mods[0];
    assert_eq!(tests.test, TestRole::CfgTest);
    let inner = tests.inline.as_ref().expect("inline");
    let inner_roles: Vec<_> = inner
        .items
        .iter()
        .map(|i| (i.name.as_str(), i.test))
        .collect();
    assert_eq!(
        inner_roles,
        [
            ("inside", TestRole::CfgTest),
            ("inner_test", TestRole::TestFn)
        ]
    );
}

#[test]
fn an_unparseable_cfg_is_an_error_naming_file_and_line_beside_a_parseable_one() {
    for bad in [
        "#[cfg(foo(bar))]",
        "#[cfg(not(a, b))]",
        "#[cfg(x = 1)]",
        "#[cfg()]",
    ] {
        let text = format!("fn ok() {{}}\n{bad}\nfn broken() {{}}\n");
        let error = load(&[("src/lib.rs", &text)]).expect_err(bad);
        assert!(
            matches!(error.kind, LoadErrorKind::CfgParse { .. }),
            "{bad}: {error}"
        );
        assert_eq!(error.file.as_str(), "src/lib.rs");
        assert_eq!(error.line, Some(2), "{bad}");
    }
    assert!(load(&[("src/lib.rs", "#[cfg(all(unix, not(test)))] fn fine() {}")]).is_ok());
}

#[test]
fn facts_carry_names_lines_documentation_visibility_and_macro_export() {
    let ws = ok(&[(
        "src/lib.rs",
        "/// Documented.\npub fn documented() {}\n\n#[doc(hidden)]\npub fn hidden() {}\n\n///\nfn blank() {}\n\n#[macro_export]\nmacro_rules! shout { () => {} }\n\npub(crate) struct S { x: u8 }\nextern crate alloc as heap;\n",
    )]);
    let file = ws
        .file(&RelPath::new("src/lib.rs").expect("path"))
        .expect("loaded");
    let items = &file.facts.body.items;
    assert_eq!(items[0].name, "documented");
    assert!(items[0].documented && !items[0].doc_hidden);
    assert_eq!(items[0].name_line, 2);
    assert!(!items[1].documented && items[1].doc_hidden);
    assert!(!items[2].documented);
    assert!(items[3].macro_export);
    assert_eq!(
        items[4].vis,
        raf_source::Vis::Restricted("crate".to_owned())
    );
    assert!(items[4].body.is_some());
    assert_eq!(items[5].name, "heap");
    assert_eq!(items[5].renamed_from.as_deref(), Some("alloc"));
    let span = items[0].span;
    assert_eq!(
        &file.text[span.start_byte..span.end_byte],
        "/// Documented.\npub fn documented() {}"
    );
}

#[test]
fn use_trees_and_impl_headers_are_recorded_as_written() {
    let ws = ok(&[(
        "src/lib.rs",
        "use ::std::{fmt, io::{self, Read as R}};\nuse crate::a::*;\nstruct Holder<T>(T);\nimpl<T> Holder<T> { fn get(&self) {} }\nimpl Trait for a::b::Thing { fn run(&self) {} }\ntrait Trait {}\n",
    )]);
    let file = ws
        .file(&RelPath::new("src/lib.rs").expect("path"))
        .expect("loaded");
    assert_eq!(file.facts.body.uses.len(), 2);
    assert!(file.facts.body.uses[0].leading_colon);
    assert_eq!(file.facts.body.impls.len(), 2);
    let inherent = &file.facts.body.impls[0];
    assert_eq!(
        inherent.self_path.as_deref(),
        Some(&["Holder".to_owned()][..])
    );
    assert!(inherent.trait_path.is_none());
    assert_eq!(inherent.items[0].name, "get");
    let by_trait = &file.facts.body.impls[1];
    assert_eq!(by_trait.trait_path.as_deref(), Some("Trait"));
    assert_eq!(
        by_trait.self_path.as_deref(),
        Some(&["a".to_owned(), "b".to_owned(), "Thing".to_owned()][..])
    );
}

#[test]
fn one_file_spelled_two_ways_is_one_file_recorded_under_both_module_paths() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "mod inner;\n#[path = \"../src/inner.rs\"] mod again;",
        ),
        ("src/inner.rs", "pub fn go() {}"),
    ]);
    let inner = ws
        .file(&RelPath::new("src/inner.rs").expect("path"))
        .expect("loaded");
    let paths: Vec<_> = inner
        .reached
        .iter()
        .map(|r| r.module_path.as_str())
        .collect();
    assert_eq!(paths, ["demo::inner", "demo::again"]);
    assert_eq!(ws.files().count(), 2);
}

/// Files `m0.rs` .. `m{n}.rs`, where each declares the next `fan` times.
fn chain(fan: usize, n: usize) -> Vec<(String, String)> {
    let mut files = vec![(
        "src/lib.rs".to_owned(),
        "#[path = \"m0.rs\"] mod m0;".to_owned(),
    )];
    for index in 0..n {
        let mut body = String::new();
        for k in 0..fan {
            writeln!(body, "#[path = \"m{}.rs\"] mod x{k};", index + 1).expect("write to a string");
        }
        files.push((format!("src/m{index}.rs"), body));
    }
    files.push((format!("src/m{n}.rs"), String::new()));
    files
}

fn borrowed(files: &[(String, String)]) -> Vec<(&str, &str)> {
    files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect()
}

#[test]
fn a_chain_of_files_that_each_declare_the_next_twice_is_refused_quickly_beside_a_small_diamond() {
    let started = std::time::Instant::now();
    let files = chain(2, 30);
    let error = load(&borrowed(&files)).expect_err("2^30 module paths");
    assert!(
        matches!(
            error.kind,
            LoadErrorKind::Budget {
                budget: raf_source::Budget::ModulePaths,
                limit: 262_144
            }
        ),
        "{error}"
    );
    assert!(error.file.as_str().starts_with("src/m"), "{error}");
    assert!(error.line.is_some());
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );

    // A small diamond is every file once per path and loads.
    let files = chain(2, 4);
    let ws = ok(&borrowed(&files));
    let last = ws
        .file(&RelPath::new("src/m4.rs").expect("path"))
        .expect("loaded");
    assert_eq!(last.reached.len(), 16);
    // A chain just inside the budget loads: 2^17 - 1 paths.
    let files = chain(2, 16);
    assert!(load(&borrowed(&files)).is_ok());
}

#[test]
fn module_depth_of_exactly_the_budget_loads_and_one_more_is_refused() {
    let nested = |depth: usize| {
        let mut files = vec![(
            "src/lib.rs".to_owned(),
            "#[path = \"m1.rs\"] mod m1;".to_owned(),
        )];
        for index in 1..depth {
            files.push((
                format!("src/m{index}.rs"),
                format!("#[path = \"m{}.rs\"] mod m{};", index + 1, index + 1),
            ));
        }
        files.push((format!("src/m{depth}.rs"), String::new()));
        files
    };
    let at = nested(128);
    assert!(load(&borrowed(&at)).is_ok());
    let over = nested(129);
    let error = load(&borrowed(&over)).expect_err("129 levels");
    assert!(
        matches!(
            error.kind,
            LoadErrorKind::Budget {
                budget: raf_source::Budget::ModuleDepth,
                limit: 128
            }
        ),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "src/m128.rs");
}

#[test]
fn a_path_attribute_on_an_inline_module_in_a_non_mod_file_replaces_its_directory_component() {
    let ws = ok(&[
        ("src/lib.rs", "mod a;"),
        ("src/a.rs", "#[path = \"custom\"] mod inline { mod leaf; }"),
        ("src/custom/leaf.rs", ""),
    ]);
    assert!(
        ws.file(&RelPath::new("src/custom/leaf.rs").expect("path"))
            .is_some()
    );
    // The module directory of the non-mod file is not consulted.
    let error = load(&[
        ("src/lib.rs", "mod a;"),
        ("src/a.rs", "#[path = \"custom\"] mod inline { mod leaf; }"),
        ("src/a/custom/leaf.rs", ""),
    ])
    .expect_err("only the file directory counts");
    assert!(matches!(error.kind, LoadErrorKind::UnresolvedMod { .. }));
}

#[test]
fn a_nested_cfg_attr_path_is_flattened_with_the_combined_predicate() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "#[cfg_attr(unix, cfg_attr(test, path = \"nested.rs\"))] mod m;",
        ),
        ("src/nested.rs", ""),
        ("src/m.rs", ""),
    ]);
    let nested = ws
        .file(&RelPath::new("src/nested.rs").expect("path"))
        .expect("loaded");
    assert_eq!(
        nested.reached[0].predicate.as_deref(),
        Some("all(unix, test)")
    );
    assert!(nested.reached[0].test);
    // The default file is still followed, with no predicate.
    let default = ws
        .file(&RelPath::new("src/m.rs").expect("path"))
        .expect("loaded");
    assert_eq!(default.reached[0].predicate, None);
}

#[test]
fn a_malformed_cfg_path_or_test_attribute_is_an_error_naming_its_line() {
    for bad in [
        "#[cfg = \"x\"]",
        "#[cfg_attr]",
        "#[path]",
        "#[path(\"x.rs\")]",
        "#[test(x)]",
    ] {
        let text = format!("fn ok() {{}}\n{bad}\nmod m;\n");
        let error = load(&[("src/lib.rs", &text), ("src/m.rs", "")]).expect_err(bad);
        assert!(
            matches!(error.kind, LoadErrorKind::CfgParse { .. }),
            "{bad}: {error}"
        );
        assert_eq!(error.line, Some(2), "{bad}");
    }
}

#[test]
fn facts_record_struct_shapes_unnamed_consts_leading_colons_and_documentation_forms() {
    let ws = ok(&[
        (
            "src/lib.rs",
            "//! The crate.
             pub struct Unit;
pub struct Tuple(u8);
pub struct Named { x: u8 }
             const _: () = ();
             #[doc = include_str!(\"lib.rs\")]
pub fn guide() {}
             impl Foo for ::other::Thing {}
impl Foo for local::Thing {}
             mod documented;
trait Foo {}
",
        ),
        (
            "src/documented.rs",
            "//! Inner docs.
",
        ),
    ]);
    let lib = ws
        .file(&RelPath::new("src/lib.rs").expect("path"))
        .expect("loaded");
    assert!(lib.facts.documented);
    let shapes: Vec<_> = lib
        .facts
        .body
        .items
        .iter()
        .map(|i| (i.name.as_str(), i.shape))
        .collect();
    assert_eq!(
        shapes[..3],
        [
            ("Unit", Some(raf_source::FieldShape::Unit)),
            ("Tuple", Some(raf_source::FieldShape::Tuple)),
            ("Named", Some(raf_source::FieldShape::Named)),
        ]
    );
    let unnamed = &lib.facts.body.items[3];
    assert_eq!(
        (unnamed.name.as_str(), unnamed.kind),
        ("_", raf_source::ItemKind::UnnamedConst)
    );
    assert!(lib.facts.body.items[4].documented);
    assert!(lib.facts.body.impls[0].self_leading_colon);
    assert!(!lib.facts.body.impls[1].self_leading_colon);
    let inner = ws
        .file(&RelPath::new("src/documented.rs").expect("path"))
        .expect("loaded");
    assert!(inner.facts.documented);
    let bare = ok(&[("src/lib.rs", "pub fn plain() {}")]);
    assert!(
        !bare
            .file(&RelPath::new("src/lib.rs").expect("path"))
            .expect("loaded")
            .facts
            .documented
    );
}

/// `levels` nested inline modules, each with two `cfg_attr` path alternatives.
fn alternatives(levels: usize) -> String {
    let open = "#[cfg_attr(unix, path = \"u\")] #[cfg_attr(windows, path = \"w\")] mod m { ";
    format!("{}{}", open.repeat(levels), "}".repeat(levels))
}

#[test]
fn inline_modules_with_path_alternatives_are_bounded_by_the_module_path_budget() {
    let started = std::time::Instant::now();
    let error = load(&[("src/lib.rs", &alternatives(40))]).expect_err("3^40 walks");
    assert!(
        matches!(
            error.kind,
            LoadErrorKind::Budget {
                budget: raf_source::Budget::ModulePaths,
                ..
            }
        ),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "src/lib.rs");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
    // A shallow case walks each body three times and loads.
    assert!(load(&[("src/lib.rs", &alternatives(5))]).is_ok());
}

// --- a module file that sorts before the file that declares it ---------------

/// Every loaded file with the paths of the items it defines.
fn item_paths(ws: &Workspace) -> Vec<(String, Vec<String>)> {
    ws.files()
        .map(|f| {
            let mut paths: Vec<String> = ws.items_in(&f.path).map(|i| i.path.clone()).collect();
            paths.sort();
            (f.path.to_string(), paths)
        })
        .collect()
}

/// `src/a/mod.rs` and its child `src/a/b.rs`, which sorts before it.
const MOD_RS_CHILD: [(&str, &str); 3] = [
    ("src/lib.rs", "pub mod a;\npub fn top() {}\n"),
    ("src/a/mod.rs", "pub mod b;\npub fn in_a() {}\n"),
    ("src/a/b.rs", "pub fn in_b() {}\n"),
];

#[test]
fn a_child_file_that_sorts_before_its_mod_rs_parent_is_placed_under_its_own_path() {
    let ws = ok(&MOD_RS_CHILD);
    assert_eq!(
        item_paths(&ws),
        files(&[
            ("src/a/b.rs", &["demo::a::b::in_b"]),
            ("src/a/mod.rs", &["demo::a::b", "demo::a::in_a"]),
            ("src/lib.rs", &["demo", "demo::a", "demo::top"]),
        ])
    );
}

#[test]
fn a_flat_module_file_and_its_child_below_a_parent_that_sorts_first_keep_their_own_paths() {
    // `src/d/a.rs` and `src/d/a/b.rs` both sort before `src/d/mod.rs`, which
    // declares `a`, and before `src/lib.rs`, which declares `d`.
    let ws = ok(&[
        ("src/lib.rs", "pub mod d;\n"),
        ("src/d/mod.rs", "pub mod a;\npub fn in_d() {}\n"),
        ("src/d/a.rs", "pub mod b;\npub fn in_a() {}\n"),
        ("src/d/a/b.rs", "pub fn in_b() {}\n"),
    ]);
    assert_eq!(
        item_paths(&ws),
        files(&[
            ("src/d/a.rs", &["demo::d::a::b", "demo::d::a::in_a"]),
            ("src/d/a/b.rs", &["demo::d::a::b::in_b"]),
            ("src/d/mod.rs", &["demo::d::a", "demo::d::in_d"]),
            ("src/lib.rs", &["demo", "demo::d"]),
        ])
    );
}

#[test]
fn a_file_module_declared_inside_an_inline_module_is_placed_under_the_inline_module() {
    // `src/a/b.rs` sorts before `src/lib.rs`, whose inline `a` declares `b`.
    let ws = ok(&[
        ("src/lib.rs", "pub mod a {\n    pub mod b;\n}\n"),
        ("src/a/b.rs", "pub fn in_b() {}\n"),
    ]);
    assert_eq!(
        item_paths(&ws),
        files(&[
            ("src/a/b.rs", &["demo::a::b::in_b"]),
            ("src/lib.rs", &["demo", "demo::a", "demo::a::b"]),
        ])
    );
}

#[test]
fn the_children_of_a_path_attribute_file_are_placed_under_its_module_whatever_their_names() {
    // `a` and `ga` sort before `generated.rs` and `z` after it.
    let children = ["a", "ga", "z"];
    let mut placed = Vec::new();
    let mut expected = Vec::new();
    for child in children {
        let child_file = format!("src/gen/{child}.rs");
        let parent = format!("pub mod {child};\npub fn in_generated() {{}}\n");
        let ws = ok(&[
            ("src/lib.rs", "#[path = \"gen/generated.rs\"]\npub mod g;\n"),
            ("src/gen/generated.rs", &parent),
            (&child_file, "pub struct Thing;\n"),
        ]);
        let file = ws
            .file(&RelPath::new(&child_file).expect("path"))
            .expect("loaded");
        let modules: Vec<String> = file.reached.iter().map(|r| r.module_path.clone()).collect();
        let thing = format!("demo::g::{child}::Thing");
        placed.push((
            child,
            item_paths(&ws),
            modules,
            ws.resolve(&thing).map(|r| r.path),
        ));
        let mut in_generated = vec![
            format!("demo::g::{child}"),
            "demo::g::in_generated".to_owned(),
        ];
        in_generated.sort();
        let mut files = vec![
            (child_file.clone(), vec![thing.clone()]),
            ("src/gen/generated.rs".to_owned(), in_generated),
            (
                "src/lib.rs".to_owned(),
                vec!["demo".to_owned(), "demo::g".to_owned()],
            ),
        ];
        files.sort();
        expected.push((child, files, vec![format!("demo::g::{child}")], Ok(thing)));
    }
    assert_eq!(placed, expected);
}

#[test]
fn a_shared_test_module_reached_from_a_target_that_sorts_after_it_keeps_its_own_paths() {
    let ws = ok(&[
        ("src/lib.rs", ""),
        (
            "tests/zz.rs",
            "mod common;\n#[test]\nfn case() {\n    common::helper();\n    common::a::in_common_a();\n}\n",
        ),
        ("tests/common/mod.rs", "pub mod a;\npub fn helper() {}\n"),
        ("tests/common/a.rs", "pub fn in_common_a() {}\n"),
    ]);
    assert_eq!(
        item_paths(&ws),
        files(&[
            ("src/lib.rs", &["demo"]),
            (
                "tests/common/a.rs",
                &["demo::test_crate::zz::common::a::in_common_a"]
            ),
            (
                "tests/common/mod.rs",
                &[
                    "demo::test_crate::zz::common::a",
                    "demo::test_crate::zz::common::helper",
                ]
            ),
            (
                "tests/zz.rs",
                &[
                    "demo::test_crate::zz",
                    "demo::test_crate::zz::case",
                    "demo::test_crate::zz::common",
                ]
            ),
        ])
    );
    assert!(matches!(
        ws.resolve("demo::test_crate::zz::common::in_common_a"),
        Err(ResolveError::NotFound(_))
    ));
}

#[test]
fn super_super_from_a_child_that_sorts_before_its_parent_climbs_to_the_crate_root() {
    let ws = ok(&[
        ("src/lib.rs", "pub mod a;\npub fn top() {}\n"),
        ("src/a/mod.rs", "pub mod b;\npub fn in_a() {}\n"),
        (
            "src/a/b.rs",
            "pub use super::super::top as again;\npub use super::in_a as sibling;\npub fn in_b() {\n    super::super::top();\n}\n",
        ),
    ]);
    assert_eq!(
        ws.resolve("demo::a::b::again").map(|r| r.path),
        Ok("demo::top".to_owned())
    );
    assert_eq!(
        ws.resolve("demo::a::b::sibling").map(|r| r.path),
        Ok("demo::a::in_a".to_owned())
    );
}

#[test]
fn an_item_is_found_only_under_its_own_module_path() {
    let ws = ok(&MOD_RS_CHILD);
    assert_eq!(
        ws.resolve("demo::a::b::in_b").map(|r| r.path),
        Ok("demo::a::b::in_b".to_owned())
    );
    for missing in ["demo::a::in_b", "demo::a::b::in_a"] {
        assert_eq!(
            ws.resolve(missing).map(|r| r.path),
            Err(ResolveError::NotFound(missing.to_owned())),
            "{missing}"
        );
    }
}
