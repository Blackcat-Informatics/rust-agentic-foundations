// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Conservative platform guards of existing indexed tests; no item filtering.

use std::fmt;

use purrdf_lex::allocation::Memory;
use syn::{Expr, ExprLit, Lit, Meta};

use crate::cfg::{CfgExpr, parse_list};
use crate::facts::{AttrFact, ModBody};
use crate::index::Index;
use crate::meter::{ItemAdmission, Meter};
use crate::{
    ItemId, LoadError, LoadErrorKind, LoadOptions, RelPath, SourceFile, SymbolKind, Workspace,
};

/// Platform facts printed by the compiler for the actual selected Cargo target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerPlatform {
    os: String,
    families: Vec<String>,
    unix: bool,
    windows: bool,
}

/// Why a compiler configuration cannot supply trustworthy platform facts.
#[derive(Debug)]
#[non_exhaustive]
pub enum CompilerPlatformError {
    /// A printed configuration line is not Rust metadata.
    Syntax {
        /// The original configuration line.
        line: String,
        /// The metadata parser's underlying error.
        source: syn::Error,
    },
    /// A printed key has a value other than a string literal.
    NonStringValue {
        /// The original configuration line.
        line: String,
    },
    /// The output contains a predicate list rather than an atomic fact.
    NonAtomic {
        /// The original configuration line.
        line: String,
    },
    /// No operating-system value was printed.
    MissingOs,
    /// More than one operating-system value was printed.
    MultipleOs,
}

impl fmt::Display for CompilerPlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { line, source } => write!(f, "invalid compiler cfg {line:?}: {source}"),
            Self::NonStringValue { line } => {
                write!(f, "compiler cfg value is not a string: {line:?}")
            }
            Self::NonAtomic { line } => write!(f, "compiler cfg is not atomic: {line:?}"),
            Self::MissingOs => f.write_str("compiler cfg is missing target_os"),
            Self::MultipleOs => f.write_str("compiler cfg contains multiple target_os entries"),
        }
    }
}

impl std::error::Error for CompilerPlatformError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Syntax { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for CompilerPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "target_os={:?}, target_family={:?}, unix={}, windows={}",
            self.os, self.families, self.unix, self.windows
        )
    }
}

impl CompilerPlatform {
    /// Parse the UTF-8 output of `rustc --print=cfg` without inferring bare atoms.
    ///
    /// # Errors
    /// Refuses malformed/non-atomic metadata, missing OS, or multiple OS entries.
    pub fn parse(output: &str) -> Result<Self, CompilerPlatformError> {
        let mut os = None;
        let mut families = Vec::new();
        let mut unix = false;
        let mut windows = false;
        for line in output.lines().filter(|line| !line.trim().is_empty()) {
            let meta: Meta =
                syn::parse_str(line).map_err(|source| CompilerPlatformError::Syntax {
                    line: line.to_owned(),
                    source,
                })?;
            match meta {
                Meta::Path(path) => {
                    unix |= path.is_ident("unix");
                    windows |= path.is_ident("windows");
                }
                Meta::NameValue(pair) => {
                    let Expr::Lit(ExprLit {
                        lit: Lit::Str(value),
                        ..
                    }) = pair.value
                    else {
                        return Err(CompilerPlatformError::NonStringValue {
                            line: line.to_owned(),
                        });
                    };
                    if pair.path.is_ident("target_os") {
                        if os.replace(value.value()).is_some() {
                            return Err(CompilerPlatformError::MultipleOs);
                        }
                    } else if pair.path.is_ident("target_family") {
                        families.push(value.value());
                    }
                }
                Meta::List(_) => {
                    return Err(CompilerPlatformError::NonAtomic {
                        line: line.to_owned(),
                    });
                }
            }
        }
        Ok(Self {
            os: os.ok_or(CompilerPlatformError::MissingOs)?,
            families,
            unix,
            windows,
        })
    }
}

/// A conservative necessary guard borrowed from a test's retained source facts.
///
/// A false result proves platform exclusion. Other results never establish that
/// Cargo lists or runs the test. Ambiguous declaration/path ancestry is unknown.
#[derive(Debug)]
pub struct TestPlatformGuard<'w> {
    attrs: Vec<&'w AttrFact>,
    uncertain: bool,
}

impl TestPlatformGuard<'_> {
    /// Whether an effective guard mentions a supported OS/family condition.
    #[must_use]
    pub fn has_platform_condition(&self) -> bool {
        self.attrs
            .iter()
            .any(|attr| attribute(attr).is_some_and(|meta| mentions_platform(&meta)))
    }

    /// Evaluate only OS/family atoms against the caller's actual target facts.
    ///
    /// Features, custom predicates and `test` remain unknown. This prevents a
    /// nonplatform false condition from becoming a platform-skip certificate.
    #[must_use]
    pub fn evaluate(&self, platform: &CompilerPlatform) -> Option<bool> {
        if self.uncertain {
            return None;
        }
        all(self
            .attrs
            .iter()
            .map(|attr| attribute(attr).and_then(|meta| evaluate_attribute(&meta, platform))))
    }
}

impl fmt::Display for TestPlatformGuard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("all(")?;
        for (index, attr) in self.attrs.iter().enumerate() {
            if index != 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}{}", attr.path, attr.args)?;
        }
        if self.uncertain {
            if !self.attrs.is_empty() {
                f.write_str(", ")?;
            }
            f.write_str("unknown source route")?;
        }
        f.write_str(")")
    }
}

fn attribute(attr: &AttrFact) -> Option<Meta> {
    syn::parse_str(&format!("{}{}", attr.path, attr.args)).ok()
}

fn all(values: impl IntoIterator<Item = Option<bool>>) -> Option<bool> {
    let mut unknown = false;
    for value in values {
        match value {
            Some(false) => return Some(false),
            None => unknown = true,
            Some(true) => {}
        }
    }
    (!unknown).then_some(true)
}

fn any(values: impl IntoIterator<Item = Option<bool>>) -> Option<bool> {
    all(values.into_iter().map(|value| value.map(|value| !value))).map(|value| !value)
}

fn platform_atom(meta: &Meta, platform: &CompilerPlatform) -> Option<bool> {
    match meta {
        Meta::Path(path) if path.is_ident("unix") => Some(platform.unix),
        Meta::Path(path) if path.is_ident("windows") => Some(platform.windows),
        Meta::NameValue(pair) => {
            let Expr::Lit(ExprLit {
                lit: Lit::Str(value),
                ..
            }) = &pair.value
            else {
                return None;
            };
            if pair.path.is_ident("target_os") {
                Some(value.value() == platform.os)
            } else if pair.path.is_ident("target_family") {
                Some(platform.families.contains(&value.value()))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn evaluate(expr: &CfgExpr, platform: &CompilerPlatform) -> Option<bool> {
    match expr {
        CfgExpr::Other(text) => syn::parse_str(text)
            .ok()
            .and_then(|meta| platform_atom(&meta, platform)),
        CfgExpr::Not(inner) => evaluate(inner, platform).map(|value| !value),
        CfgExpr::All(values) if !values.is_empty() => {
            all(values.iter().map(|value| evaluate(value, platform)))
        }
        CfgExpr::Any(values) if !values.is_empty() => {
            any(values.iter().map(|value| evaluate(value, platform)))
        }
        CfgExpr::Boolean(_) | CfgExpr::Test | CfgExpr::All(_) | CfgExpr::Any(_) => None,
    }
}

fn evaluate_attribute(meta: &Meta, platform: &CompilerPlatform) -> Option<bool> {
    let Meta::List(list) = meta else {
        return Some(true);
    };
    if !list.path.is_ident("cfg") && !list.path.is_ident("cfg_attr") {
        return Some(true);
    }
    let parts = crate::cfg::parse_punctuated(&list.tokens).ok()?;
    let condition = CfgExpr::from_meta_configured(parts.first()?).ok()?;
    if list.path.is_ident("cfg") {
        return (parts.len() == 1)
            .then(|| evaluate(&condition, platform))
            .flatten();
    }
    if parts.len() == 1 {
        return parts.trailing_punct().then_some(true);
    }
    // cfg_attr(P, cfg(Q)) is P => Q, not P && Q.
    any([
        evaluate(&condition, platform).map(|value| !value),
        all(parts
            .iter()
            .skip(1)
            .map(|part| evaluate_attribute(part, platform))),
    ])
}

fn mentions_platform(meta: &Meta) -> bool {
    match meta {
        Meta::Path(path) => path.is_ident("unix") || path.is_ident("windows"),
        Meta::NameValue(pair) => {
            pair.path.is_ident("target_os") || pair.path.is_ident("target_family")
        }
        Meta::List(list) => {
            parse_list(&list.tokens).is_ok_and(|parts| parts.iter().any(mentions_platform))
        }
    }
}

struct Builder<'w, 'm, 'a, 'g> {
    attrs: Vec<&'w AttrFact>,
    uncertain: bool,
    memory: &'m mut Memory<'a, ItemAdmission<'g>>,
    meter: &'g Meter,
    file: &'g RelPath,
}

impl<'w> Builder<'w, '_, '_, '_> {
    fn attrs(&mut self, attrs: &'w [AttrFact]) -> Result<(), LoadError> {
        self.meter.work(attrs.len(), self.file, None)?;
        for attr in attrs
            .iter()
            .filter(|attr| matches!(attr.path.as_str(), "cfg" | "cfg_attr"))
        {
            self.meter.work(
                attr.path.len().saturating_add(attr.args.len()),
                self.file,
                None,
            )?;
            let meta = attribute(attr).ok_or_else(|| {
                LoadError::new(
                    self.file,
                    None,
                    LoadErrorKind::Internal {
                        what: "retained cfg attribute is not valid metadata".into(),
                    },
                )
            })?;
            validate_attribute(&meta).map_err(|what| {
                LoadError::new(self.file, None, LoadErrorKind::Internal { what })
            })?;
            self.memory
                .push(&mut self.attrs, attr)
                .map_err(|error| LoadError::new(self.file, None, LoadErrorKind::Storage(error)))?;
        }
        Ok(())
    }

    fn site(&mut self, file: &'w SourceFile, start: usize, module: bool) -> Result<(), LoadError> {
        self.attrs(&file.facts.attrs)?;
        let mut body: &ModBody = &file.facts.body;
        loop {
            self.meter.work(
                body.items.len().saturating_add(body.mods.len()),
                self.file,
                None,
            )?;
            if !module
                && let Some(item) = body.items.iter().find(|item| item.span.start_byte == start)
            {
                return self.attrs(&item.attrs);
            }
            if module
                && let Some(item) = body.mods.iter().find(|item| item.span.start_byte == start)
            {
                self.uncertain |= !item.paths.conditional.is_empty();
                return self.attrs(&item.attrs);
            }
            let mut enclosing = body.mods.iter().filter(|item| {
                item.inline.is_some() && item.span.start_byte <= start && start < item.span.end_byte
            });
            let Some(selected) = enclosing.next() else {
                return Err(LoadError::new(
                    self.file,
                    None,
                    LoadErrorKind::Internal {
                        what: "indexed source item has no retained attribute site".into(),
                    },
                ));
            };
            if enclosing.next().is_some() {
                return Err(LoadError::new(
                    self.file,
                    None,
                    LoadErrorKind::Internal {
                        what: "retained inline module spans overlap".into(),
                    },
                ));
            }
            self.uncertain |= !selected.paths.conditional.is_empty();
            self.attrs(&selected.attrs)?;
            // INVARIANT: the enclosing-module iterator selects only entries
            // whose immutable inline body is Some; no mutation occurs above.
            body = selected.inline.as_ref().expect("selected inline module");
        }
    }
}

fn ordinary(kind: SymbolKind) -> bool {
    matches!(
        kind,
        SymbolKind::Fn
            | SymbolKind::Trait
            | SymbolKind::Struct
            | SymbolKind::Enum
            | SymbolKind::Union
            | SymbolKind::TypeAlias
            | SymbolKind::Const
            | SymbolKind::Static
            | SymbolKind::ExternCrate
            | SymbolKind::Mod
    )
}

pub(crate) fn guard<'w>(
    source: &'w Workspace,
    id: ItemId,
    index: &'w Index,
    options: &LoadOptions,
    used: usize,
) -> Result<TestPlatformGuard<'w>, LoadError> {
    let missing = || {
        LoadError::new(
            &RelPath::root(),
            None,
            LoadErrorKind::Internal {
                what: "platform guard requires a valid indexed item and source context".into(),
            },
        )
    };
    let item = source.item(id).ok_or_else(missing)?;
    let meter = Meter::new(options);
    meter.items(used, &item.file, Some(item.name_line))?;
    let mut admission = ItemAdmission::new(&meter, &item.file);
    let result = (|| {
        let mut memory = Memory::new(&mut admission);
        let mut builder = Builder {
            attrs: Vec::new(),
            uncertain: false,
            memory: &mut memory,
            meter: &meter,
            file: &item.file,
        };
        let file = source.file(&item.file).ok_or_else(missing)?;
        builder
            .meter
            .work(index.root_items.len(), &item.file, Some(item.name_line))?;
        let root = index.root_items.values().any(|root| *root == id);
        let ordinary = ordinary(item.kind);
        if root {
            builder.attrs(&file.facts.attrs)?;
        } else if ordinary {
            builder.site(file, item.span.start_byte, item.kind == SymbolKind::Mod)?;
        } else {
            // Member/foreign/exported-macro guards need exact owner ancestry.
            builder.uncertain = true;
        }
        let parent = item.path.rsplit_once("::").map(|(parent, _)| parent);
        builder
            .meter
            .work(index.modules.len(), &item.file, Some(item.name_line))?;
        let mut module = if root {
            index.target_roots.get(item.target.index()).copied()
        } else {
            index.modules.iter().position(|module| {
                module.target == item.target && Some(module.path.as_str()) == parent
            })
        };
        while let Some(at) = module {
            let Some(parent) = index.modules[at].parent else {
                break;
            };
            builder
                .meter
                .work(index.module_of.len(), &item.file, Some(item.name_line))?;
            let mut sites = index.module_of.iter().filter(|(_, child)| **child == at);
            let Some((&decl, _)) = sites.next() else {
                return Err(missing());
            };
            if sites.next().is_some() {
                builder.uncertain = true;
                break;
            }
            let decl = &index.items[decl.0];
            builder.site(
                source.file(&decl.file).ok_or_else(missing)?,
                decl.span.start_byte,
                true,
            )?;
            module = Some(parent);
        }
        if module.is_none() {
            if ordinary || root {
                return Err(missing());
            }
            builder.uncertain = true;
        }
        if let Some(target) = source.target(item.target) {
            builder.attrs(&source.file(&target.root).ok_or_else(missing)?.facts.attrs)?;
        } else {
            return Err(missing());
        }
        Ok::<_, LoadError>(TestPlatformGuard {
            attrs: builder.attrs,
            uncertain: builder.uncertain,
        })
    })();
    result.map_err(|error| match error.kind {
        LoadErrorKind::Storage(native) => admission.error(native),
        _ => error,
    })
}

fn validate_attribute(meta: &Meta) -> Result<(), String> {
    let Meta::List(list) = meta else {
        return if meta.path().is_ident("cfg") || meta.path().is_ident("cfg_attr") {
            Err("retained cfg attribute requires a predicate list".into())
        } else {
            Ok(())
        };
    };
    if !list.path.is_ident("cfg") && !list.path.is_ident("cfg_attr") {
        return Ok(());
    }
    let parts = crate::cfg::parse_punctuated(&list.tokens)?;
    let first = parts
        .first()
        .ok_or("retained cfg attribute has no predicate")?;
    CfgExpr::from_meta_configured(first)?;
    if list.path.is_ident("cfg") {
        if parts.len() != 1 {
            return Err("retained cfg attribute has several predicates".into());
        }
    } else {
        if parts.len() == 1 && !parts.trailing_punct() {
            return Err("retained cfg_attr requires a comma after its predicate".into());
        }
        for part in parts.iter().skip(1) {
            validate_attribute(part)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryTree;

    fn workspace(files: &[(&str, &str)]) -> Workspace {
        let tree: MemoryTree =
            std::iter::once(("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\n"))
                .chain(files.iter().copied())
                .collect();
        Workspace::load(&tree, &["."]).unwrap()
    }

    fn at<'a>(source: &'a Workspace, path: &str) -> TestPlatformGuard<'a> {
        let item = source
            .items()
            .find(|item| item.path.as_str() == path)
            .unwrap();
        source.test_platform_guard(item.id).unwrap()
    }

    fn linux() -> CompilerPlatform {
        CompilerPlatform::parse("target_os=\"linux\"\ntarget_family=\"unix\"\nunix\n").unwrap()
    }

    fn windows() -> CompilerPlatform {
        CompilerPlatform::parse("target_os=\"windows\"\ntarget_family=\"windows\"\nwindows\n")
            .unwrap()
    }

    #[test]
    fn platform_guard_reference_storage_uses_the_remaining_item_grant() {
        let tree = MemoryTree::from([
            ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\n"),
            ("src/lib.rs", "#[cfg(windows)]\n#[test]\nfn selected() {}\n"),
        ]);
        let source = Workspace::load(&tree, &["."]).unwrap();
        let item = source
            .items()
            .find(|item| item.path.as_str() == "demo::selected")
            .unwrap();
        let item_bytes = source.usage().item_bytes;
        let admitted = source.test_platform_guard(item.id).unwrap();
        // Exactly one retained cfg pointer, so native initial capacity is the
        // only new allocation: no replacement buffer or second growth overlaps.
        assert_eq!(admitted.attrs.len(), 1);
        let pointer_bytes = admitted.attrs.capacity() * size_of::<&AttrFact>();
        assert!(pointer_bytes > 0);
        assert_eq!(admitted.evaluate(&linux()), Some(false));
        drop(admitted);

        let sufficient = LoadOptions {
            max_item_bytes: item_bytes + pointer_bytes,
            ..LoadOptions::default()
        };
        let source = Workspace::load_with(&tree, &["."], &sufficient).unwrap();
        let item = source
            .items()
            .find(|item| item.path.as_str() == "demo::selected")
            .unwrap();
        let admitted = source.test_platform_guard(item.id).unwrap();
        assert_eq!(admitted.attrs.len(), 1);
        assert_eq!(admitted.evaluate(&linux()), Some(false));

        // Both zero remaining storage and one byte less than the native
        // reference buffer allow the actual load, then refuse the real API.
        for remaining in [0, pointer_bytes - 1] {
            let limit = item_bytes + remaining;
            let options = LoadOptions {
                max_item_bytes: limit,
                ..LoadOptions::default()
            };
            let source = Workspace::load_with(&tree, &["."], &options).unwrap();
            assert_eq!(source.usage().item_bytes, item_bytes);
            let item = source
                .items()
                .find(|item| item.path.as_str() == "demo::selected")
                .unwrap();
            let error = source.test_platform_guard(item.id).unwrap_err();
            assert!(
                matches!(error.kind, LoadErrorKind::Budget { budget: crate::Budget::ItemBytes, limit: actual } if actual == limit),
                "{error}"
            );
            assert_eq!(error.file.as_str(), "src/lib.rs");
            assert_eq!(error.line, None);
        }
    }

    #[test]
    fn platform_guard_work_exhaustion_is_typed_beside_a_sufficient_neighbor() {
        let source = workspace(&[("src/lib.rs", "#[cfg(windows)]\n#[test]\nfn selected() {}\n")]);
        let item = source
            .items()
            .find(|item| item.path.as_str() == "demo::selected")
            .unwrap();
        // Exercise the same production helper as the public wrapper, with an
        // exhausted QUERY work account after an independently successful load.
        let options = LoadOptions {
            max_work: 0,
            ..LoadOptions::default()
        };
        let error = guard(
            &source,
            item.id,
            source.index(),
            &options,
            source.usage().item_bytes,
        )
        .unwrap_err();
        assert!(
            matches!(
                error.kind,
                LoadErrorKind::Budget {
                    budget: crate::Budget::Work,
                    limit: 0
                }
            ),
            "{error}"
        );
        assert_eq!(error.file.as_str(), "src/lib.rs");
        assert_eq!(error.line, Some(3));
        let accepted = source.test_platform_guard(item.id).unwrap();
        assert_eq!(accepted.evaluate(&linux()), Some(false));
        assert_eq!(accepted.evaluate(&windows()), Some(true));
    }

    #[test]
    fn parent_platform_guards_do_not_filter_the_source_inventory() {
        let source = workspace(&[
            (
                "src/lib.rs",
                r"
                #[cfg(windows)] mod windows {
                    #[test] fn a_disk_tree_refuses_a_reserved_device_name_or_a_trailing_dot_or_space_beside_a_plain_name() {}
                    #[test] fn the_final_path_of_a_handle_is_its_canonical_path_and_lies_under_the_root() {}
                }
                #[cfg(windows)] mod external;
                #[test] fn plain() {}
            ",
            ),
            ("src/external.rs", "#[test] fn external_test() {}"),
        ]);
        for path in [
            "demo::windows::a_disk_tree_refuses_a_reserved_device_name_or_a_trailing_dot_or_space_beside_a_plain_name",
            "demo::windows::the_final_path_of_a_handle_is_its_canonical_path_and_lies_under_the_root",
            "demo::external::external_test",
        ] {
            let guard = at(&source, path);
            assert!(guard.has_platform_condition());
            assert_eq!(guard.evaluate(&linux()), Some(false));
            assert_eq!(guard.evaluate(&windows()), Some(true));
            assert!(guard.to_string().contains("windows"));
        }
        assert_eq!(at(&source, "demo::plain").evaluate(&linux()), Some(true));
        assert_eq!(source.test_fns().unwrap().len(), 4);
    }

    #[test]
    fn inner_crate_and_external_module_guards_are_both_observed() {
        let source = workspace(&[
            ("src/lib.rs", "#![cfg(target_family=\"unix\")] mod child;"),
            (
                "src/child.rs",
                "#![cfg(target_os=\"linux\")] #[test] fn selected() {}",
            ),
        ]);
        let guard = at(&source, "demo::child::selected");
        assert_eq!(guard.evaluate(&linux()), Some(true));
        assert_eq!(guard.evaluate(&windows()), Some(false));
    }

    #[test]
    fn nonplatform_conditions_and_cfg_attr_do_not_invent_exclusion() {
        let source = workspace(&[(
            "src/lib.rs",
            r#"
            #[cfg(all(not(test), windows))] #[test] fn nonplatform() {}
            #[cfg(all(any(), windows))] #[test] fn constant() {}
            #[cfg(false)] #[test] fn literal_false() {}
            #[cfg(r#false)] #[test] fn raw_false() {}
            #[cfg_attr(unix,)] #[test] fn empty_platform_attr() {}
            #[cfg_attr(custom,)] #[test] fn empty_unknown_attr() {}
            #[cfg(any(feature="extra", windows))] #[test] fn feature() {}
            #[cfg_attr(test, cfg(windows))] #[test] fn unknown_attr() {}
            #[cfg_attr(unix, cfg(target_os="linux"))] #[test] fn implication() {}
            #[cfg(windows)] #[test] fn bare() {}
        "#,
        )]);
        for path in ["demo::nonplatform", "demo::constant"] {
            assert_eq!(at(&source, path).evaluate(&windows()), None);
            assert_eq!(at(&source, path).evaluate(&linux()), Some(false));
        }
        assert_eq!(at(&source, "demo::feature").evaluate(&linux()), None);
        for name in ["demo::literal_false", "demo::raw_false"] {
            assert_eq!(at(&source, name).evaluate(&linux()), None);
            assert_eq!(at(&source, name).evaluate(&windows()), None);
        }
        for name in ["demo::empty_platform_attr", "demo::empty_unknown_attr"] {
            assert_eq!(at(&source, name).evaluate(&linux()), Some(true));
            assert_eq!(at(&source, name).evaluate(&windows()), Some(true));
        }
        for malformed in [
            "cfg_attr(unix)",
            "cfg_attr()",
            "cfg_attr(,)",
            "cfg(::windows)",
            "cfg_attr(::windows,)",
        ] {
            let meta = syn::parse_str(malformed).expect("retained metadata syntax");
            assert!(validate_attribute(&meta).is_err(), "{malformed}");
            assert_eq!(evaluate_attribute(&meta, &linux()), None, "{malformed}");
        }
        for (malformed, correction) in [
            ("cfg(::windows)", "unqualified"),
            ("cfg_attr(::windows,)", "unqualified"),
            ("cfg_attr(unix)", "comma"),
        ] {
            let text = format!("#[{malformed}] #[test] fn malformed() {{}}");
            let invalid = workspace(&[("src/lib.rs", &text)]);
            let item = invalid
                .resolve("demo::malformed")
                .expect("structural item")
                .sites[0];
            let error = invalid
                .test_platform_guard(item)
                .expect_err("malformed guard");
            assert!(error.to_string().contains(correction), "{error}");
        }
        assert_eq!(at(&source, "demo::bare").evaluate(&linux()), Some(false));
        assert_eq!(at(&source, "demo::bare").evaluate(&windows()), Some(true));
        assert_eq!(at(&source, "demo::unknown_attr").evaluate(&linux()), None);
        assert_eq!(
            at(&source, "demo::implication").evaluate(&linux()),
            Some(true)
        );
        assert_eq!(
            at(&source, "demo::implication").evaluate(&windows()),
            Some(true)
        );
        let custom =
            CompilerPlatform::parse("target_os=\"linux\"\ntarget_family=\"unix\"\nwindows\n")
                .unwrap();
        assert_eq!(at(&source, "demo::bare").evaluate(&custom), Some(true));
    }

    #[test]
    fn compiler_platform_refuses_incomplete_or_malformed_output() {
        assert!(matches!(
            CompilerPlatform::parse("unix\n"),
            Err(CompilerPlatformError::MissingOs)
        ));
        assert!(matches!(
            CompilerPlatform::parse("target_os=\"linux\"\ntarget_os=\"linux\"\n"),
            Err(CompilerPlatformError::MultipleOs)
        ));
        assert!(matches!(
            CompilerPlatform::parse("target_os=123\n"),
            Err(CompilerPlatformError::NonStringValue { .. })
        ));
        assert!(matches!(
            CompilerPlatform::parse("target_os=\"linux\"\nall(unix)\n"),
            Err(CompilerPlatformError::NonAtomic { .. })
        ));
        let syntax =
            CompilerPlatform::parse("target_os =\n").expect_err("a missing cfg value is malformed");
        assert!(matches!(syntax, CompilerPlatformError::Syntax { .. }));
        assert!(std::error::Error::source(&syntax).is_some());
        assert!(CompilerPlatform::parse("target_os=\"none\"\n").is_ok());
    }

    #[test]
    fn conditional_module_routes_remain_unknown() {
        let source = workspace(&[
            (
                "src/lib.rs",
                "#[cfg_attr(feature=\"alternate\",path=\"alternate.rs\")] mod child;",
            ),
            ("src/child.rs", "#[cfg(windows)] #[test] fn selected() {}"),
            ("src/alternate.rs", "#[cfg(unix)] #[test] fn selected() {}"),
        ]);
        let alternatives: Vec<_> = source
            .items()
            .filter(|item| item.path.as_str() == "demo::child::selected")
            .collect();
        assert_ne!(alternatives.len(), 0);
        for item in alternatives {
            let guard = source.test_platform_guard(item.id).unwrap();
            assert_eq!(guard.evaluate(&linux()), None);
        }
    }

    #[test]
    fn invalid_item_is_an_actionable_error() {
        let source = workspace(&[("src/lib.rs", "#[test] fn plain() {}")]);
        let error = source.test_platform_guard(ItemId(usize::MAX)).unwrap_err();
        assert!(matches!(error.kind, LoadErrorKind::Internal { .. }));
    }
}
