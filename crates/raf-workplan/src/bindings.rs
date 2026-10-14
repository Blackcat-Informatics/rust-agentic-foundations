// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The rules for the entries of a capability's `bindings`.
//!
//! The graph resolves each entry through the source index and keeps the
//! result; the rules here judge it. An entry that does not resolve, or names
//! more than one symbol, is a finding. An entry that resolves is judged by
//! the list it is in, on the definitions it resolves to (a re-export is
//! followed to the definition):
//!
//! * `code` names shipping items: not test code, and in a target that ships.
//! * `tests` names functions marked `#[test]`.
//! * `examples` names the crate root of an example target
//!   (`<package>::example_crate::<name>`), or an item with documentation in
//!   the library of a package whose documentation tests run (an
//!   [`ExampleKind::Doctest`]). That a documentation test passes is judged by
//!   running it, not here.
//!
//! A symbol that a macro generates is not in the index. When the path does not
//! resolve, its parent is a module of the index, and a macro invocation at the
//! item level of that module names the missing symbol, the finding says so
//! (`binding-macro-generated`) instead of reporting a plain missing path.

use std::collections::BTreeMap;

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use raf_okf::{Capability, Location};
use raf_source::{Item, ItemId, ResolveError, SymbolKind, TargetKind, TestRole, Workspace};

use crate::finding::kind;
use crate::graph::{Binding, BindingKind, Graph};
use crate::{Finding, Loaded};

/// How an item bound in `examples` is an example.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ExampleKind {
    /// The crate root of an example target, which `cargo test` builds and
    /// `cargo run --example` runs.
    Target,
    /// An item with documentation in the library of a package whose
    /// documentation tests run: its documentation holds the example. That the
    /// documentation holds a fenced code block, and that the library's
    /// `crate-type` allows documentation tests, are not index facts and are
    /// not judged.
    Doctest,
}

/// How `item` is an example, or `None` when it is not one.
///
/// An example target counts by its crate root. Documentation counts only when
/// the item is not test code (rustdoc skips it) and the target is the library
/// with `doctest` on: cargo honours that key only for the library.
pub(crate) fn example_kind(source: &Workspace, item: &Item) -> Option<ExampleKind> {
    let target = source.target(item.target)?;
    if target.kind == TargetKind::Example
        && item.kind == SymbolKind::Mod
        && *item.path == *target.crate_path
    {
        Some(ExampleKind::Target)
    } else if item.documented
        && item.test == TestRole::None
        && target.kind == TargetKind::Lib
        && target.doctest
    {
        Some(ExampleKind::Doctest)
    } else {
        None
    }
}

/// The findings of the binding rules over every binding of the graph.
pub(crate) fn check(
    graph: &Graph,
    capabilities: &[Loaded<Capability>],
    source: &Workspace,
) -> Vec<Finding> {
    run(graph, capabilities, source).0
}

/// The findings, and how many module texts were lexed to find them.
///
/// The invocations of a module are listed once per run, however many
/// bindings go through it.
fn run(
    graph: &Graph,
    capabilities: &[Loaded<Capability>],
    source: &Workspace,
) -> (Vec<Finding>, usize) {
    let by_id: BTreeMap<&str, &Loaded<Capability>> = capabilities
        .iter()
        .map(|cap| (cap.value().header.id.as_str(), cap))
        .collect();
    let mut macros = Macros::new(source);
    let mut findings = Vec::new();
    for binding in graph.bindings() {
        let Some(cap) = by_id.get(binding.capability()) else {
            continue;
        };
        let at = Location::Key("bindings".to_string());
        let list = list_name(binding.kind());
        let who = format!(
            "{} {list} binding `{}`",
            binding.capability(),
            binding.path()
        );
        match binding.resolution() {
            Err(error) => findings.push(unresolved(
                cap.path(),
                at,
                &who,
                binding,
                error,
                source,
                &mut macros,
            )),
            Ok(sites) => {
                if let Some(why) = wrong_role(source, binding, sites) {
                    findings.push(Finding::new(
                        kind::BINDING_WRONG_ROLE,
                        cap.path(),
                        at,
                        format!("{who} {why}"),
                        role_fix(binding.kind()),
                    ));
                }
            }
        }
    }
    (findings, macros.lexes)
}

fn list_name(kind: BindingKind) -> &'static str {
    match kind {
        BindingKind::Code => "code",
        BindingKind::Test => "tests",
        BindingKind::Example => "examples",
    }
}

fn role_fix(kind: BindingKind) -> &'static str {
    match kind {
        BindingKind::Code => {
            "bind an item of a library, binary or build script that is not test code, or move the entry to `tests` or `examples`"
        }
        BindingKind::Test => {
            "bind a function marked `#[test]`, or move the entry to `code` or `examples`"
        }
        BindingKind::Example => {
            "bind the crate root of an example target (`<package>::example_crate::<name>`) or a documented item of a library whose documentation tests run, or move the entry to `code` or `tests`"
        }
    }
}

/// Why the definitions a binding resolved to do not fit its list, or nothing
/// when every one does. One failing site fails the binding: a binding is a
/// claim about the symbol, and a symbol defined twice (`cfg` twins) is judged
/// on both definitions.
fn wrong_role(source: &Workspace, binding: &Binding, sites: &[ItemId]) -> Option<&'static str> {
    let mut items = sites.iter().filter_map(|id| source.item(*id));
    match binding.kind() {
        BindingKind::Code if items.any(|item| !source.is_shipping(item)) => {
            Some("names an item that is test code or is not in a shipping target")
        }
        BindingKind::Test if items.any(|item| item.test != TestRole::TestFn) => {
            Some("names an item that is not a function marked `#[test]`")
        }
        BindingKind::Example if sites.iter().any(|id| binding.example_kind(*id).is_none()) => Some(
            "names an item that is neither the crate root of an example target nor a documented item of a library whose documentation tests run",
        ),
        _ => None,
    }
}

fn unresolved(
    file: &str,
    at: Location,
    who: &str,
    binding: &Binding,
    error: &ResolveError,
    source: &Workspace,
    macros: &mut Macros<'_>,
) -> Finding {
    match error {
        ResolveError::Ambiguous(ids) => {
            let mut paths: Vec<String> = ids
                .iter()
                .filter_map(|id| source.item(*id))
                .map(|item| format!("`{}` ({}:{})", item.path, item.file, item.name_line))
                .collect();
            paths.sort();
            Finding::new(
                kind::BINDING_AMBIGUOUS,
                file,
                at,
                format!("{who} names more than one symbol: {}", paths.join(", ")),
                "bind the canonical path of one definition, so the entry names one symbol",
            )
        }
        ResolveError::NotFound(_) => {
            if let Some(site) = macros.site(binding.path()) {
                Finding::new(
                    kind::BINDING_MACRO_GENERATED,
                    file,
                    at,
                    format!(
                        "{who} does not resolve: {error}; the name appears in the macro invocation at {site}, which may generate it; a generated symbol is not in the index and cannot be bound"
                    ),
                    "bind an item written in source, such as the function that calls the macro, or the macro definition",
                )
            } else {
                not_found(file, at, who, error)
            }
        }
        _ => not_found(file, at, who, error),
    }
}

fn not_found(file: &str, at: Location, who: &str, error: &ResolveError) -> Finding {
    Finding::new(
        kind::BINDING_UNRESOLVED,
        file,
        at,
        format!("{who} does not resolve: {error}"),
        "write the canonical path of an item the source index defines (check the spelling, the module declarations and the target the item is in), or remove the entry; a name that a macro, derive or attribute generates is not in the index and cannot be bound",
    )
}

/// The item-level macro invocations of the modules the unresolved bindings go
/// through, each module lexed at most once.
struct Macros<'a> {
    source: &'a Workspace,
    /// By module path: each identifier that appears in an invocation of the
    /// module, with the `file:line` of the first one.
    modules: BTreeMap<String, BTreeMap<String, String>>,
    /// How many module texts were lexed.
    lexes: usize,
}

impl<'a> Macros<'a> {
    fn new(source: &'a Workspace) -> Self {
        Self {
            source,
            modules: BTreeMap::new(),
            lexes: 0,
        }
    }

    /// `file:line` of an item-level macro invocation in the module that is the
    /// parent of `path` which names the last segment of `path`. Only a module
    /// has such invocations: the parent of a function or a type has none.
    fn site(&mut self, path: &str) -> Option<String> {
        let (parent, name) = path.rsplit_once("::")?;
        let sites = self.source.resolve(parent).ok()?.sites;
        for id in sites {
            let module = self.source.item(id)?;
            if module.kind != SymbolKind::Mod {
                continue;
            }
            let key = module.path.to_string();
            if !self.modules.contains_key(&key) {
                let listed = self.list(module);
                self.modules.insert(key.clone(), listed);
            }
            if let Some(site) = self.modules.get(&key).and_then(|names| names.get(name)) {
                return Some(site.clone());
            }
        }
        None
    }

    /// The identifiers of the invocations of `module`, read from its braced
    /// text when it is inline and from the files it owns otherwise.
    fn list(&mut self, module: &Item) -> BTreeMap<String, String> {
        let mut names = BTreeMap::new();
        if let Some(body) = &module.body {
            let slice = self.source.source(&module.file).and_then(|text| {
                let slice = text.get(body.start_byte..body.end_byte)?;
                Some((slice, text[..body.start_byte].matches('\n').count()))
            });
            if let Some((slice, base)) = slice {
                self.lexes += 1;
                invocations(slice, base, &module.file.to_string(), &mut names);
            }
        } else {
            for file in self.source.files().filter(|file| {
                file.reached
                    .iter()
                    .any(|reach| reach.module_path == module.path)
            }) {
                self.lexes += 1;
                invocations(&file.text, 0, &file.path.to_string(), &mut names);
            }
        }
        names
    }
}

/// Add to `names` each identifier held by a macro invocation at the item level
/// of `text`, with the `file:line` of the first invocation that holds it.
/// `base` is the number of lines before the text. Text that does not lex holds
/// none.
fn invocations(text: &str, base: usize, file: &str, names: &mut BTreeMap<String, String>) {
    let Ok(stream) = text.parse::<TokenStream>() else {
        return;
    };
    let mut tokens: Vec<TokenTree> = stream.into_iter().collect();
    // An inline module's text is its braced body: read the items inside.
    let inner = match tokens.as_slice() {
        [TokenTree::Group(group)] if group.delimiter() == Delimiter::Brace => {
            Some(group.stream().into_iter().collect::<Vec<_>>())
        }
        _ => None,
    };
    if let Some(inner) = inner {
        tokens = inner;
    }
    for (index, token) in tokens.iter().enumerate() {
        let (TokenTree::Punct(bang), Some(TokenTree::Group(args))) = (token, tokens.get(index + 1))
        else {
            continue;
        };
        // `path ! (..)`: an invocation. `macro_rules ! name {..}`, `#![..]`
        // and `impl !Send` do not have this shape.
        if bang.as_char() == '!'
            && index > 0
            && matches!(tokens.get(index - 1), Some(TokenTree::Ident(_)))
        {
            let line = base + tokens[index - 1].span().start().line;
            collect(&args.stream(), &format!("{file}:{line}"), names);
        }
    }
}

/// Add every identifier in `stream` to `names`, keeping the first site.
fn collect(stream: &TokenStream, site: &str, names: &mut BTreeMap<String, String>) {
    for token in stream.clone() {
        match token {
            TokenTree::Ident(ident) => {
                names
                    .entry(ident.to_string())
                    .or_insert_with(|| site.to_string());
            }
            TokenTree::Group(group) => collect(&group.stream(), site, names),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use raf_source::{MemoryTree, Workspace};

    use crate::Workplan;

    use super::run;

    /// Many bindings through one large module lex it once.
    #[test]
    fn the_invocations_of_a_module_are_listed_once_however_many_bindings_miss() {
        let mut lib = String::from("macro_rules! make { ($n:ident) => { pub fn $n() {} }; }\n");
        for index in 0..2000 {
            writeln!(lib, "make!(made_{index});").unwrap();
        }
        let source = Workspace::load(
            &MemoryTree::from([
                (
                    "Cargo.toml",
                    "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
                ),
                ("src/lib.rs", lib.as_str()),
            ]),
            &["."],
        )
        .unwrap();
        let paths = (0..300)
            .map(|index| format!("demo::made_{}", index * 3))
            .collect::<Vec<_>>()
            .join(", ");
        let mut sections = String::new();
        for name in [
            "Purpose",
            "Contract",
            "Usage",
            "Examples",
            "Constraints",
            "Explanation",
            "Verification",
        ] {
            write!(sections, "## {name}\n\nText for {name}.\n\n").unwrap();
        }
        let doc = format!(
            "---\ntype: Capability\nid: cap:many\ntitle: Many\ndescription: Many misses.\n\
             state: implementing\nbindings:\n  code: [{paths}]\n  tests: []\n  examples: []\n\
             depends: []\n---\n\n# Many\n\n{sections}"
        );
        let docs = MemoryTree::from([("docs/capabilities/many.md", doc.as_str())]);
        let plan = Workplan::load(&docs, &source);
        let (findings, lexes) = run(plan.graph(), plan.capabilities(), &source);
        // The names are missing: the macro defines them, the index does not.
        assert_eq!(findings.len(), 300, "{:#?}", plan.check());
        assert!(findings.iter().all(|f| f.kind == "binding-macro-generated"));
        assert_eq!(lexes, 1);
    }
}
