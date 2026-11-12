// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The analysis-library boundary rejects process launch capabilities in source.

use super::support::{decode_text, gate_root, read_tracked, tracked_paths};
use raf_source::syn::{self, ext::IdentExt as _, visit::Visit as _};

#[derive(Default)]
struct Boundary {
    refused: Vec<String>,
}

fn test_only(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && attribute
                .parse_args::<syn::Path>()
                .is_ok_and(|path| path.is_ident("test"))
    })
}

impl<'ast> syn::visit::Visit<'ast> for Boundary {
    fn visit_use_tree(&mut self, tree: &'ast syn::UseTree) {
        let name = match tree {
            syn::UseTree::Path(path) => Some(&path.ident),
            syn::UseTree::Name(name) => Some(&name.ident),
            syn::UseTree::Rename(rename) => Some(&rename.ident),
            syn::UseTree::Glob(_) | syn::UseTree::Group(_) => None,
        };
        if let Some(name) = name
            && matches!(
                name.unraw().to_string().as_str(),
                "process" | "async_process" | "subprocess" | "command_fds"
            )
        {
            self.refused.push(name.to_string());
        }
        syn::visit::visit_use_tree(self, tree);
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if !test_only(&item.attrs) {
            syn::visit::visit_item_mod(self, item);
        }
    }
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let names: Vec<_> = path
            .segments
            .iter()
            .map(|part| part.ident.unraw().to_string())
            .collect();
        let permitted_id = names == ["std", "process", "id"];
        if (!permitted_id
            && names.len() > 1
            && names.iter().any(|name| {
                matches!(
                    name.as_str(),
                    "process" | "async_process" | "subprocess" | "command_fds"
                )
            }))
            || names
                .iter()
                .any(|name| matches!(name.as_str(), "posix_spawn" | "execve" | "execvp" | "fork"))
        {
            self.refused.push(names.join("::"));
        }
        syn::visit::visit_path(self, path);
    }
    fn visit_item_foreign_mod(&mut self, item: &'ast syn::ItemForeignMod) {
        self.refused.push("foreign function interface".into());
        syn::visit::visit_item_foreign_mod(self, item);
    }
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        // Literal strings remain data. Macro token paths must not conceal a launch API.
        fn inspect(tokens: proc_macro2::TokenStream, refused: &mut Vec<String>) {
            fn identity_call(tokens: &[proc_macro2::TokenTree]) -> bool {
                use proc_macro2::TokenTree::{Group, Ident, Punct};
                matches!(tokens, [Ident(root), Punct(a), Punct(b), Ident(module), Punct(c), Punct(d), Ident(function), Group(arguments)]
                    if root == "std" && module == "process" && function == "id"
                    && [a, b, c, d].iter().all(|value| value.as_char() == ':')
                    && arguments.delimiter() == proc_macro2::Delimiter::Parenthesis
                    && arguments.stream().is_empty())
            }
            let tokens: Vec<_> = tokens.into_iter().collect();
            let mut at = 0;
            while at < tokens.len() {
                if tokens
                    .get(at..)
                    .and_then(|rest| rest.get(..8))
                    .is_some_and(identity_call)
                {
                    at += 8;
                    continue;
                }
                let token = &tokens[at];
                match token {
                    proc_macro2::TokenTree::Ident(name)
                        if matches!(
                            name.unraw().to_string().as_str(),
                            "process"
                                | "async_process"
                                | "subprocess"
                                | "command_fds"
                                | "execve"
                                | "execvp"
                                | "posix_spawn"
                                | "fork"
                        ) =>
                    {
                        refused.push(name.to_string());
                    }
                    proc_macro2::TokenTree::Group(group) => inspect(group.stream(), refused),
                    _ => {}
                }
                at += 1;
            }
        }
        inspect(item.tokens.clone(), &mut self.refused);
        syn::visit::visit_macro(self, item);
    }
}

fn violations(text: &str) -> Vec<String> {
    let syntax = syn::parse_file(text).expect("boundary input must parse");
    let mut boundary = Boundary::default();
    boundary.visit_file(&syntax);
    boundary.refused
}

#[test]
fn runtime_analysis_libraries_have_no_process_authority() {
    let root = gate_root();
    let mut inspected = 0;
    for path in tracked_paths() {
        let text_path = path.to_string_lossy();
        let parts: Vec<_> = text_path.split('/').collect();
        if parts.len() < 4
            || parts[0] != "crates"
            || !parts[1].starts_with("raf-")
            || parts[2] != "src"
            || path.extension().is_none_or(|extension| extension != "rs")
            || parts[3..].contains(&"tests")
            || parts.last() == Some(&"tests.rs")
        {
            continue;
        }
        // Two files of the unpublished fixture crate run Git in scratch repositories for the
        // tests of several packages. It is an analysis library of no package: no
        // shipped crate depends on it.
        if parts.get(1) == Some(&"raf-fixtures")
            && matches!(parts.get(3), Some(&("git_repo.rs" | "fixture_signing.rs")))
        {
            continue;
        }
        let bytes = read_tracked(&root, &path);
        let text = decode_text(&bytes).expect("Rust source must be text");
        let problems = violations(&text);
        assert!(
            problems.is_empty(),
            "{} imports process authority: {problems:?}",
            path.display()
        );
        inspected += 1;
    }
    assert!(inspected > 0, "no analysis source was inspected");
}

#[test]
fn runtime_refuses_launches_aliases_and_hidden_macro_paths() {
    for source in [
        "unsafe extern \"C\" { fn external(); }",
        "use std::process as p;",
        "use std::r#process as p;",
        "use std::process::Command as C;",
        "use std::{process as p}; fn launch() { p::Command::new(\"x\"); }",
        "fn launch() { std::process::Command::new(\"x\"); }",
        "macro_rules! launch { () => { std::r#process::Command::new(\"x\") }; }",
    ] {
        assert!(!violations(source).is_empty(), "{source}");
    }
    assert_eq!(
        violations(
            "fn identity() -> u32 { std::process::id() } fn display() -> String { format!(\"{}\", std::process::id()) } fn data() -> &'static str { \"std::process::Command\" } fn flag(process: bool) -> bool { process }"
        ),
        Vec::<String>::new()
    );
}

/// Lines of a manifest that list `raf-fixtures` in a table other than
/// `[dev-dependencies]`.
fn shipped_fixture_dependencies(manifest: &str) -> Vec<String> {
    let mut table = "";
    let mut found = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            table = line;
        } else if line.starts_with("raf-fixtures")
            && table.contains("dependencies")
            && !table.contains("dev-dependencies")
        {
            found.push(format!("{table} {line}"));
        }
    }
    found
}

#[test]
fn no_shipped_crate_depends_on_the_fixture_crate_outside_its_dev_dependencies() {
    let root = gate_root();
    let mut inspected = 0;
    for path in tracked_paths() {
        let text_path = path.to_string_lossy();
        let parts: Vec<_> = text_path.split('/').collect();
        if parts.len() != 3
            || parts[0] != "crates"
            || parts[1] == "raf-fixtures"
            || parts[2] != "Cargo.toml"
        {
            continue;
        }
        let bytes = read_tracked(&root, &path);
        let text = decode_text(&bytes).expect("a manifest is text");
        assert_eq!(
            shipped_fixture_dependencies(&text),
            Vec::<String>::new(),
            "{}",
            path.display()
        );
        inspected += 1;
    }
    assert!(inspected > 0, "no crate manifest was inspected");
}

#[test]
fn a_fixture_dependency_is_refused_outside_dev_dependencies_beside_the_accepted_form() {
    assert_eq!(
        shipped_fixture_dependencies(
            "[dependencies]\nraf-fixtures = { path = \"../raf-fixtures\" }\n"
        ),
        ["[dependencies] raf-fixtures = { path = \"../raf-fixtures\" }"]
    );
    assert_eq!(
        shipped_fixture_dependencies(
            "[dev-dependencies]\nraf-fixtures = { path = \"../raf-fixtures\" }\n"
        ),
        Vec::<String>::new()
    );
}
