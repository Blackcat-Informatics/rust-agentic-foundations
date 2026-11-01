// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The real receiving doc attribute and exact documented public spellings.

use super::{DiagramError, IncludeRequest, refused};
use crate::diagram::rustdoc;
use raf_source::{Item, RelPath, SymbolKind, TargetId, Workspace, syn};
use syn::ext::IdentExt as _;
use syn::spanned::Spanned as _;
use syn::visit::Visit as _;

fn includes(attributes: &[syn::Attribute]) -> Vec<String> {
    attributes
        .iter()
        .filter_map(|attribute| {
            let syn::Meta::NameValue(pair) = &attribute.meta else {
                return None;
            };
            if !pair.path.is_ident("doc") {
                return None;
            }
            let syn::Expr::Macro(expression) = &pair.value else {
                return None;
            };
            let path = &expression.mac.path;
            let names: Vec<_> = path
                .segments
                .iter()
                .map(|part| part.ident.unraw().to_string())
                .collect();
            let native = path.is_ident("include_str")
                || names == ["std", "include_str"]
                || names == ["core", "include_str"];
            if !native {
                return None;
            }
            expression
                .mac
                .parse_body::<syn::LitStr>()
                .ok()
                .map(|value| value.value())
        })
        .collect()
}

fn content(attributes: &[syn::Attribute]) -> usize {
    attributes
        .iter()
        .filter(|attribute| {
            matches!(&attribute.meta,
        syn::Meta::NameValue(pair) if pair.path.is_ident("doc"))
        })
        .count()
}

struct Receiver<'a> {
    start: usize,
    end: usize,
    result: &'a mut Vec<String>,
    content: &'a mut usize,
}
impl<'ast> syn::visit::Visit<'ast> for Receiver<'_> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let span = node.span().byte_range();
        if span.start == self.start && span.end == self.end {
            self.result.extend(includes(&node.attrs));
            *self.content += content(&node.attrs);
        }
        syn::visit::visit_item_mod(self, node);
    }
}

fn paths(
    source: &Workspace,
    target: TargetId,
    item: &Item,
    root: bool,
) -> Result<(Vec<(RelPath, String)>, usize), DiagramError> {
    let code = source.code_resolver().map_err(DiagramError::data)?;
    let mut result = Vec::new();
    let mut count = 0;
    for file in source.files() {
        let body = file.reached.iter().any(|reach| {
            reach.target == target && reach.module_path.as_str() == item.path.as_str()
        });
        if file.path != item.file && !body {
            continue;
        }
        let found = code
            .with_configured_syntax(target, &file.path, |_, syntax| {
                let mut values = if body || root {
                    includes(&syntax.attrs)
                } else {
                    Vec::new()
                };
                let mut count = if body || root {
                    content(&syntax.attrs)
                } else {
                    0
                };
                if !root && file.path == item.file {
                    Receiver {
                        start: item.span.start_byte,
                        end: item.span.end_byte,
                        result: &mut values,
                        content: &mut count,
                    }
                    .visit_file(syntax);
                }
                (values, count)
            })
            .map_err(DiagramError::data)?;
        result.extend(found.0.into_iter().map(|value| (file.path.clone(), value)));
        count += found.1;
    }
    Ok((result, count))
}

pub(super) fn admit(
    root: &std::path::Path,
    source: &Workspace,
    target: TargetId,
    request: &IncludeRequest,
) -> Result<(), DiagramError> {
    let selected = source
        .target(target)
        .ok_or_else(|| refused("assembly target is absent"))?;
    let surface = source.exported_items(target).map_err(DiagramError::data)?;
    let receiver = source
        .resolve(&request.module)
        .map_err(DiagramError::data)?;
    let [receiver] = receiver.sites.as_slice() else {
        return Err(refused("receiving module is ambiguous or unresolved"));
    };
    let receiver = source
        .item(*receiver)
        .ok_or_else(|| refused("receiving module has no source identity"))?;
    let (module, _) = rustdoc::spelling(&request.module)?;
    let is_root = module == selected.crate_path;
    if receiver.kind != SymbolKind::Mod
        || receiver.doc_hidden
        || (!is_root
            && !surface
                .items
                .iter()
                .any(|item| item.path == module && item.item == receiver.id))
    {
        return Err(refused(
            "receiving module must be public, documented and visible in the selected library",
        ));
    }
    let expected = root.join(request.include.as_str());
    let parent = expected
        .parent()
        .ok_or_else(|| refused("include has no parent"))?
        .canonicalize()
        .map_err(DiagramError::data)?;
    if !parent.starts_with(root) {
        return Err(refused("include parent leaves the captured workspace"));
    }
    let mut matched = 0;
    let (paths, content) = paths(source, target, receiver, is_root)?;
    if content != 1 {
        return Err(refused(
            "the checked include must be the receiving module's sole doc content attribute, so surrounding prose cannot turn required links into code or raw text",
        ));
    }
    for (file, literal) in paths {
        let actual = root
            .join(file.as_str())
            .parent()
            .ok_or_else(|| refused("module file has no parent"))?
            .join(literal);
        if actual.file_name() == expected.file_name()
            && actual
                .parent()
                .ok_or_else(|| refused("doc include has no parent"))?
                .canonicalize()
                .map_err(DiagramError::data)?
                == parent
        {
            matched += 1;
        }
    }
    if matched != 1 {
        return Err(refused(
            "receiving module requires one actual literal doc = include_str! attribute naming the exact owned fragment; generated include expressions require input evidence",
        ));
    }
    Ok(())
}

#[derive(Default)]
struct Audit {
    issue: Option<String>,
}

fn shadow(tree: &syn::UseTree) -> bool {
    match tree {
        syn::UseTree::Glob(_) => true,
        syn::UseTree::Group(group) => group.items.iter().any(shadow),
        syn::UseTree::Path(path) => shadow(&path.tree),
        syn::UseTree::Name(name) => matches!(
            name.ident.unraw().to_string().as_str(),
            "include_str" | "std" | "core"
        ),
        syn::UseTree::Rename(rename) => matches!(
            rename.rename.unraw().to_string().as_str(),
            "include_str" | "std" | "core"
        ),
    }
}

impl<'ast> syn::visit::Visit<'ast> for Audit {
    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        let known = node.path().get_ident().is_some_and(|ident| {
            matches!(
                ident.unraw().to_string().as_str(),
                "doc"
                    | "cfg"
                    | "cfg_attr"
                    | "allow"
                    | "warn"
                    | "deny"
                    | "forbid"
                    | "expect"
                    | "deprecated"
                    | "must_use"
                    | "non_exhaustive"
                    | "repr"
                    | "inline"
                    | "cold"
                    | "track_caller"
                    | "automatically_derived"
                    | "path"
                    | "no_std"
                    | "no_implicit_prelude"
            )
        });
        if !known {
            self.issue =
                Some("opaque rewriting attributes require compiler-expanded input evidence".into());
        }
        if node.path().is_ident("doc")
            && let syn::Meta::List(list) = &node.meta
        {
            let metadata = list.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            );
            let admitted = metadata.as_ref().is_ok_and(|entries| {
                entries.iter().all(|entry| match entry {
                    syn::Meta::Path(path) => path.is_ident("hidden") || path.is_ident("inline"),
                    syn::Meta::NameValue(pair) => {
                        pair.path.is_ident("alias")
                            && matches!(
                                &pair.value,
                                syn::Expr::Lit(syn::ExprLit {
                                    lit: syn::Lit::Str(_),
                                    ..
                                })
                            )
                    }
                    syn::Meta::List(_) => false,
                })
            });
            if !admitted {
                self.issue = Some("unsupported doc metadata, including no_inline or generated content, has no admitted receiving-include semantics".into());
            }
        }
        syn::visit::visit_attribute(self, node);
    }
    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        if !node.mac.path.is_ident("macro_rules")
            || node
                .ident
                .as_ref()
                .is_some_and(|name| name.unraw() == "include_str")
        {
            self.issue = Some("item macro expansion or include_str shadowing requires compiler-expanded input evidence".into());
        }
        syn::visit::visit_item_macro(self, node);
    }
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        if shadow(&node.tree) {
            self.issue = Some("glob or include_str/std/core import shadowing has no admitted builtin include identity".into());
        }
        syn::visit::visit_item_use(self, node);
    }
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if matches!(node.ident.unraw().to_string().as_str(), "std" | "core") {
            self.issue = Some("local modules may replace the builtin include namespace".into());
        }
        syn::visit::visit_item_mod(self, node);
    }
    fn visit_impl_item_macro(&mut self, _node: &'ast syn::ImplItemMacro) {
        self.issue =
            Some("associated item macros require compiler-expanded definition evidence".into());
    }
    fn visit_trait_item_macro(&mut self, _node: &'ast syn::TraitItemMacro) {
        self.issue = Some("trait item macros require compiler-expanded definition evidence".into());
    }
    fn visit_foreign_item_macro(&mut self, _node: &'ast syn::ForeignItemMacro) {
        self.issue =
            Some("foreign item macros require compiler-expanded definition evidence".into());
    }
    fn visit_item_extern_crate(&mut self, node: &'ast syn::ItemExternCrate) {
        let name = node.rename.as_ref().map_or(&node.ident, |(_, name)| name);
        if matches!(
            name.unraw().to_string().as_str(),
            "std" | "core" | "include_str"
        ) {
            self.issue = Some("extern binding may replace the builtin include namespace".into());
        }
        syn::visit::visit_item_extern_crate(self, node);
    }
}

pub(super) fn audit(source: &Workspace, target: TargetId) -> Result<(), DiagramError> {
    let code = source.code_resolver().map_err(DiagramError::data)?;
    for file in source.files() {
        let issue = code
            .with_configured_syntax(target, &file.path, |_, syntax| {
                let mut audit = Audit::default();
                audit.visit_file(syntax);
                audit.issue
            })
            .map_err(DiagramError::data)?;
        if let Some(issue) = issue {
            return Err(refused(format!(
                "{}: {issue}; no checked include was generated",
                file.path
            )));
        }
    }
    Ok(())
}
