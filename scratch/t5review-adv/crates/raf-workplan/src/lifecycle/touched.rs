// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `contract-effect-understated`: under strict mode, a change that says
//! `contract_effect: none` and touches a bound public item.
//!
//! A bound public item is an item of the public surface of a library or binary
//! target ([`Workspace::exported_items`]) that a `code` binding of any loaded
//! capability covers, itself or as an item below it, as two-way coverage reads
//! it ([`Bound::covers`]). "Touch" is read literally: any part of the item's
//! span, its signature, its body, its attributes, its visibility and its doc
//! comments.
//!
//! Each file the rule reads is read at the merge base with the row
//! `git cat-file blob <merge-base>:<prefix><path>` that `section-untouched`
//! uses, at the same merge base, within the 1 MiB bound of a blob, so no new
//! command is asked of `git`. Each side is parsed from its whole file through
//! [`parse_source`], and every item of the file at every depth that a path can
//! name (the items of inline modules, impl blocks, traits and extern blocks
//! included) is cut from the text by its span and read as a `proc_macro2`
//! token stream. A doc comment arrives as a `#[doc]` attribute and counts;
//! whitespace and comments that are not doc comments are not tokens and do not.
//! CRLF line endings read as LF, as `rustc` reads them.
//!
//! * A candidate whose stream is not among the streams of the items of its file
//!   at the merge base is touched. So an item moved within its file is not, an
//!   item moved to another file is, and a new bound public item is; a file that
//!   is absent at the merge base makes every candidate in it touched.
//! * An item of the merge base that the working tree lacks, removed or renamed,
//!   is touched when its path is bound: the path of an item at the merge base
//!   is the module path of its file in the working tree, the names of the inline
//!   modules around it, and, for a member of an impl block or a trait, the last
//!   segment of the self type or the name of the trait. The files read for this
//!   are every file that a library or binary target of a package holding a
//!   bound item reaches outside test code; an item is counted only when it is
//!   declared `pub`, is a `pub` member of an inherent impl, or is a member of a
//!   `pub` trait.
//!
//! A file at the merge base that is not UTF-8 or does not parse is an error and
//! not a pass, and so is a surface the source index cannot list.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use proc_macro2::{Delimiter, LineColumn, Spacing, TokenStream, TokenTree};
use raf_source::syn::spanned::Spanned;
use raf_source::syn::{self, Ident, ImplItem, Item, TraitItem, Visibility};
use raf_source::{LoadError, RelPath, TargetKind, Workspace, parse_source};

use crate::Workplan;
use crate::coverage::{Bound, covering, covering_path};
use crate::exec::Runner;
use crate::git::{MergeBase, Prefix};
use crate::graph::BindingKind;
use crate::obligation::{Needed, Obligation, ObligationKind, Part, Place, Subject, Touch, Touched};

use super::session::{LifecycleError, Side};
use super::untouched::{blob, merge_base};

/// The `contract-effect-understated` obligations of the change `change`, one
/// for each touched bound public item, on the change record.
///
/// The caller asks only under strict mode and for a change with
/// `contract_effect: none`. Nothing is asked of `git` when no `code` binding
/// covers an item; otherwise the merge base is found once, kept in `base` for
/// the other kinds, and each file is read once at it.
pub(super) fn raise(
    plan: &Workplan<'_>,
    runner: &dyn Runner,
    prefix: &Prefix,
    change: &str,
    base: &mut Option<MergeBase>,
) -> Result<Vec<Obligation>, LifecycleError> {
    let record = plan.change(change)?;
    let source = plan.source();
    let packages = bound_packages(plan);
    if packages.is_empty() {
        return Ok(Vec::new());
    }
    let candidates = candidates(plan, &packages)?;
    let files = files(source, &packages, &candidates);
    if files.is_empty() {
        return Ok(Vec::new());
    }
    let base = merge_base(base, runner)?;
    let mut touched = Vec::new();
    for (path, modules) in &files {
        let wanted: Vec<&raf_source::Item> = candidates
            .get(path)
            .map_or_else(Vec::new, |items| items.values().copied().collect());
        touched.extend(judge_file(
            plan, runner, base, prefix, path, modules, &wanted,
        )?);
    }
    Ok(touched
        .into_iter()
        .map(|touched| Obligation {
            kind: ObligationKind::ContractEffectUnderstated,
            subject: Subject::Change(record.value().header.id.clone()),
            section: Part::Document,
            location: Place {
                path: record.path().to_string(),
                line: None,
            },
            evidence: Needed::StatedEffect(touched),
        })
        .collect())
}

/// The packages that hold an item a `code` binding resolves to.
fn bound_packages<'p>(plan: &'p Workplan<'_>) -> BTreeSet<&'p str> {
    let source = plan.source();
    plan.graph()
        .bindings()
        .iter()
        .filter(|binding| binding.kind() == BindingKind::Code)
        .flat_map(|binding| binding.items().iter())
        .filter_map(|id| source.item(*id).and_then(|item| source.target(item.target)))
        .map(|target| target.package.as_str())
        .collect()
}

/// The bound public items of `packages`, by file and by the byte their span
/// starts at, so an item that two targets reach is one candidate.
type Candidates<'s> = BTreeMap<RelPath, BTreeMap<usize, &'s raf_source::Item>>;

fn candidates<'s>(
    plan: &'s Workplan<'_>,
    packages: &BTreeSet<&str>,
) -> Result<Candidates<'s>, LifecycleError> {
    let source = plan.source();
    let bound = Bound::of(plan.graph(), source);
    let mut found: Candidates<'s> = BTreeMap::new();
    for id in source.target_ids() {
        let Some(target) = source.target(id) else {
            continue;
        };
        if !matches!(target.kind, TargetKind::Lib | TargetKind::Bin)
            || !packages.contains(target.package.as_str())
        {
            continue;
        }
        let exports = source
            .exported_items(id)
            .map_err(|error| LifecycleError::Surface {
                package: target.package.clone(),
                target: target.crate_path.clone(),
                reason: error.to_string(),
            })?;
        for export in &exports.items {
            if !bound.covers(source, export.item) {
                continue;
            }
            if let Some(item) = source.item(export.item) {
                found
                    .entry(item.file.clone())
                    .or_default()
                    .entry(item.span.start_byte)
                    .or_insert(item);
            }
        }
    }
    Ok(found)
}

/// The files the rule reads, each with the module paths it has in the library
/// and binary targets of `packages`: every file such a target reaches outside
/// test code, and every file that holds a candidate.
fn files(
    source: &Workspace,
    packages: &BTreeSet<&str>,
    candidates: &Candidates<'_>,
) -> BTreeMap<RelPath, Vec<String>> {
    let mut files: BTreeMap<RelPath, Vec<String>> = BTreeMap::new();
    for file in source.files() {
        let mut modules: Vec<String> = file
            .reached
            .iter()
            .filter(|reach| {
                !reach.test
                    && source.target(reach.target).is_some_and(|target| {
                        matches!(target.kind, TargetKind::Lib | TargetKind::Bin)
                            && packages.contains(target.package.as_str())
                    })
            })
            .map(|reach| reach.module_path.to_string())
            .collect();
        modules.sort();
        modules.dedup();
        if !modules.is_empty() || candidates.contains_key(&file.path) {
            files.insert(file.path.clone(), modules);
        }
    }
    files
}

/// The touched items of one file.
fn judge_file(
    plan: &Workplan<'_>,
    runner: &dyn Runner,
    base: &MergeBase,
    prefix: &Prefix,
    path: &RelPath,
    modules: &[String],
    wanted: &[&raf_source::Item],
) -> Result<Vec<Touched>, LifecycleError> {
    let source = plan.source();
    let document = |side, reason: String| LifecycleError::Document {
        path: path.to_string(),
        side,
        reason,
    };
    let text = source
        .source(path)
        .ok_or_else(|| document(Side::WorkingTree, "is not in the source index".to_string()))?;
    let spans: Vec<Range<usize>> = wanted
        .iter()
        .map(|item| item.span.start_byte..item.span.end_byte)
        .collect();
    let now =
        scan(path, text, &spans).map_err(|error| document(Side::WorkingTree, error.to_string()))?;
    let then = match blob(runner, base, prefix, path.as_str())? {
        None => None,
        Some(bytes) => {
            let text = String::from_utf8(bytes)
                .map_err(|_| document(Side::MergeBase, "is not UTF-8 text".to_string()))?;
            Some(
                scan(path, &text, &[])
                    .map_err(|error| document(Side::MergeBase, error.to_string()))?,
            )
        }
    };

    let mut touched = Vec::new();
    let streams: BTreeSet<&str> = then
        .iter()
        .flat_map(|scan| scan.units.iter().map(|unit| unit.stream.as_str()))
        .collect();
    for (item, stream) in wanted.iter().zip(&now.wanted) {
        if streams.contains(stream.as_str()) {
            continue;
        }
        if let Some(binding) = covering(plan.graph(), source, item.id) {
            touched.push(Touched {
                item: item.path.to_string(),
                file: path.to_string(),
                line: item.name_line,
                capability: binding.capability().to_string(),
                how: Touch::Tokens,
            });
        }
    }
    let Some(then) = then else {
        return Ok(touched);
    };
    let present: BTreeSet<&str> = now
        .units
        .iter()
        .filter_map(|unit| unit.name.as_deref())
        .collect();
    for unit in &then.units {
        let Some(name) = unit.name.as_deref() else {
            continue;
        };
        if !unit.public || present.contains(name) {
            continue;
        }
        let bound = modules.iter().find_map(|module| {
            let full = format!("{module}::{name}");
            covering_path(plan.graph(), source, &full).map(|binding| (full, binding))
        });
        if let Some((full, binding)) = bound {
            touched.push(Touched {
                item: full,
                file: path.to_string(),
                line: unit.line,
                capability: binding.capability().to_string(),
                how: Touch::Removed,
            });
        }
    }
    Ok(touched)
}

/// One item of a file, as the rule compares it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Unit {
    /// The token stream of the item's span, rendered as [`render`] writes it.
    stream: String,
    /// The path of the item below the module of its file, for an item a path
    /// can name: `name`, `inner::name`, `Type::method` or `Trait::item`.
    name: Option<String>,
    /// Whether the item is of the public kind the removal rule counts.
    public: bool,
    /// The 1-based line of its name, or of its first token.
    line: usize,
}

/// What [`scan`] reads from a file: its units, and the stream of each span the
/// caller asked for, in its order.
#[derive(Debug)]
struct Scan {
    units: Vec<Unit>,
    wanted: Vec<String>,
}

/// Parse `text` through [`parse_source`] and read every unit of it, and the
/// stream of each byte range of `wanted`.
///
/// The spans and the streams are read on the thread that parsed, which holds
/// the location data of the parse, and leave it as plain text.
fn scan(path: &RelPath, text: &str, wanted: &[Range<usize>]) -> Result<Scan, LoadError> {
    parse_source(path, text, |file| {
        let reader = Reader::new(text);
        let mut units = Vec::new();
        reader.items(&file.items, "", &mut units);
        let wanted = wanted
            .iter()
            .map(|range| reader.stream_of(range.clone()))
            .collect();
        Scan { units, wanted }
    })
}

/// The text of a file with the byte offset where each line starts.
struct Reader<'t> {
    text: &'t str,
    starts: Vec<usize>,
}

impl<'t> Reader<'t> {
    fn new(text: &'t str) -> Self {
        let starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(at, _)| at + 1))
            .collect();
        Self { text, starts }
    }

    /// The byte offset of a line and column of the parse: the line is 1-based
    /// and the column counts characters from 0.
    fn byte(&self, at: LineColumn) -> Option<usize> {
        let start = *self.starts.get(at.line.checked_sub(1)?)?;
        let line = self.text.get(start..)?;
        let line = line.split('\n').next().unwrap_or(line);
        let offset = line
            .char_indices()
            .map(|(offset, _)| offset)
            .chain(std::iter::once(line.len()))
            .nth(at.column)?;
        Some(start + offset)
    }

    /// The rendered stream of the bytes `range`.
    fn stream_of(&self, range: Range<usize>) -> String {
        let Some(slice) = self.text.get(range.clone()) else {
            // A span that is not a range of the text names no item of it.
            return format!("\u{0}no text at {}..{}", range.start, range.end);
        };
        let slice = slice.replace("\r\n", "\n");
        match slice.parse::<TokenStream>() {
            Ok(stream) => {
                let mut out = String::new();
                render(stream, &mut out);
                out
            }
            // The parse accepted the file, so its items lex; a slice that does
            // not is compared by its text, which no stream equals.
            Err(_) => format!("\u{0}{slice}"),
        }
    }

    /// The rendered stream of the whole span of `node`.
    fn stream_of_node(&self, node: &impl Spanned) -> String {
        let span = node.span();
        match (self.byte(span.start()), self.byte(span.end())) {
            (Some(start), Some(end)) if start <= end => self.stream_of(start..end),
            _ => format!("\u{0}no span at line {}", span.start().line),
        }
    }

    /// Read the items `items` of a module whose path below the module of the
    /// file is `module` (empty for the file's own).
    fn items(&self, items: &[Item], module: &str, out: &mut Vec<Unit>) {
        for item in items {
            let stream = self.stream_of_node(item);
            let named = |ident: &Ident, vis: &Visibility| {
                (
                    Some(joined(module, &ident.to_string())),
                    is_pub(vis),
                    ident_line(ident),
                )
            };
            let (name, public, line) = match item {
                Item::Fn(item) => named(&item.sig.ident, &item.vis),
                Item::Const(item) => named(&item.ident, &item.vis),
                Item::Static(item) => named(&item.ident, &item.vis),
                Item::Struct(item) => named(&item.ident, &item.vis),
                Item::Enum(item) => named(&item.ident, &item.vis),
                Item::Union(item) => named(&item.ident, &item.vis),
                Item::Type(item) => named(&item.ident, &item.vis),
                Item::TraitAlias(item) => named(&item.ident, &item.vis),
                Item::Trait(item) => {
                    let trait_path = joined(module, &item.ident.to_string());
                    let public = is_pub(&item.vis);
                    for member in &item.items {
                        self.trait_item(member, &trait_path, public, out);
                    }
                    named(&item.ident, &item.vis)
                }
                Item::Mod(item) => {
                    let inner = joined(module, &item.ident.to_string());
                    if let Some((_, content)) = &item.content {
                        self.items(content, &inner, out);
                    }
                    named(&item.ident, &item.vis)
                }
                Item::Impl(item) => {
                    let key = self_type_key(&item.self_ty).map(|key| joined(module, &key));
                    let inherent = item.trait_.is_none();
                    for member in &item.items {
                        self.impl_item(member, key.as_deref(), inherent, out);
                    }
                    (None, false, item.span().start().line)
                }
                Item::ForeignMod(item) => {
                    for member in &item.items {
                        let member_stream = self.stream_of_node(member);
                        let (name, public, line) = match member {
                            syn::ForeignItem::Fn(member) => named(&member.sig.ident, &member.vis),
                            syn::ForeignItem::Static(member) => named(&member.ident, &member.vis),
                            syn::ForeignItem::Type(member) => named(&member.ident, &member.vis),
                            other => (None, false, other.span().start().line),
                        };
                        out.push(Unit {
                            stream: member_stream,
                            name,
                            public,
                            line,
                        });
                    }
                    (None, false, item.span().start().line)
                }
                Item::Macro(item) => (
                    item.ident
                        .as_ref()
                        .map(|ident| joined(module, &ident.to_string())),
                    false,
                    item.ident
                        .as_ref()
                        .map_or_else(|| item.span().start().line, ident_line),
                ),
                other => (None, false, other.span().start().line),
            };
            out.push(Unit {
                stream,
                name,
                public,
                line,
            });
        }
    }

    /// Read a member of an impl block whose self type has the path `key`
    /// below the module of the file, when the parse can name it.
    fn impl_item(&self, member: &ImplItem, key: Option<&str>, inherent: bool, out: &mut Vec<Unit>) {
        let stream = self.stream_of_node(member);
        let (ident, vis) = match member {
            ImplItem::Fn(member) => (Some(&member.sig.ident), Some(&member.vis)),
            ImplItem::Const(member) => (Some(&member.ident), Some(&member.vis)),
            ImplItem::Type(member) => (Some(&member.ident), Some(&member.vis)),
            _ => (None, None),
        };
        let name = key.zip(ident).map(|(key, ident)| format!("{key}::{ident}"));
        out.push(Unit {
            stream,
            name,
            public: inherent && vis.is_some_and(is_pub),
            line: ident.map_or_else(|| member.span().start().line, ident_line),
        });
    }

    /// Read a member of the trait whose path below the module of the file is
    /// `trait_path`.
    fn trait_item(&self, member: &TraitItem, trait_path: &str, public: bool, out: &mut Vec<Unit>) {
        let stream = self.stream_of_node(member);
        let ident = match member {
            TraitItem::Fn(member) => Some(&member.sig.ident),
            TraitItem::Const(member) => Some(&member.ident),
            TraitItem::Type(member) => Some(&member.ident),
            _ => None,
        };
        out.push(Unit {
            stream,
            name: ident.map(|ident| format!("{trait_path}::{ident}")),
            public,
            line: ident.map_or_else(|| member.span().start().line, ident_line),
        });
    }
}

/// `module::name`, or `name` for the module of the file itself.
fn joined(module: &str, name: &str) -> String {
    if module.is_empty() {
        name.to_string()
    } else {
        format!("{module}::{name}")
    }
}

fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn ident_line(ident: &Ident) -> usize {
    ident.span().start().line
}

/// The last segment of the path of a self type, without its generic
/// arguments: `Frame` for `Frame<T>` and for `crate::render::Frame`. A self
/// type that is not a path (a reference, a tuple, a slice) has none.
fn self_type_key(self_ty: &syn::Type) -> Option<String> {
    match self_ty {
        syn::Type::Path(path) if path.qself.is_none() => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        _ => None,
    }
}

/// Write `stream` as a comparison key: each identifier and literal as its
/// text between spaces, each punctuation character followed by a space unless
/// it is joined to the next, and each group between its delimiters. Spans,
/// whitespace and comments that are not doc comments leave no trace.
fn render(stream: TokenStream, out: &mut String) {
    for tree in stream {
        match tree {
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    Delimiter::Parenthesis => ('(', ')'),
                    Delimiter::Brace => ('{', '}'),
                    Delimiter::Bracket => ('[', ']'),
                    Delimiter::None => ('\u{2}', '\u{3}'),
                };
                out.push(open);
                render(group.stream(), out);
                out.push(close);
            }
            TokenTree::Ident(ident) => {
                out.push(' ');
                out.push_str(&ident.to_string());
                out.push(' ');
            }
            TokenTree::Punct(punct) => {
                out.push(punct.as_char());
                if punct.spacing() == Spacing::Alone {
                    out.push(' ');
                }
            }
            TokenTree::Literal(literal) => {
                out.push(' ');
                out.push_str(&literal.to_string());
                out.push(' ');
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use raf_source::RelPath;

    use super::{Scan, scan};

    fn units(text: &str) -> Scan {
        scan(&RelPath::new("src/lib.rs").expect("a path"), text, &[]).expect("it parses")
    }

    fn stream(text: &str, name: &str) -> String {
        units(text)
            .units
            .into_iter()
            .find(|unit| unit.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("no {name}"))
            .stream
    }

    #[test]
    fn whitespace_and_plain_comments_leave_the_stream_and_a_doc_comment_or_a_token_changes_it() {
        let base = "/// Adds.\npub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n";
        let same = [
            "/// Adds.\npub fn add(a: u32, b: u32) -> u32 { a + b }\n",
            "/// Adds.\npub fn add(\n    a: u32,\n    b: u32\n) -> u32 {\n    // plain\n    a + b /* block */\n}\n",
            "/// Adds.\r\npub fn add(a: u32, b: u32) -> u32 {\r\n    a + b\r\n}\r\n",
        ];
        for text in same {
            assert_eq!(stream(text, "add"), stream(base, "add"), "{text}");
        }
        let different = [
            "/// Adds two.\npub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
            "/** Adds. */\npub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
            "/// Adds.\npub fn add(a: u32, b: u32) -> u32 {\n    b + a\n}\n",
            "/// Adds.\n#[must_use]\npub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
            "/// Adds.\npub(crate) fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n",
            "/// Adds.\npub fn add(a: u64, b: u32) -> u32 {\n    a + b\n}\n",
            "/// Adds.\npub fn add(a: u32, b: u32,) -> u32 {\n    a + b\n}\n",
        ];
        for text in different {
            assert_ne!(stream(text, "add"), stream(base, "add"), "{text}");
        }
    }

    #[test]
    fn the_items_of_modules_impls_and_traits_are_named_below_the_file_and_a_private_one_is_not_counted()
     {
        let text = "pub mod inner {\n    pub fn go() {}\n    fn helper() {}\n}\npub struct Frame;\nimpl Frame {\n    pub fn push(&self) {}\n    fn grow(&self) {}\n}\nimpl Clone for Frame {\n    fn clone(&self) -> Self { Frame }\n}\npub trait Shape {\n    fn area(&self) -> u32;\n}\nmacro_rules! make { () => {} }\n";
        let named: Vec<(String, bool)> = units(text)
            .units
            .into_iter()
            .filter_map(|unit| unit.name.map(|name| (name, unit.public)))
            .collect();
        assert_eq!(
            named,
            [
                ("inner::go".to_string(), true),
                ("inner::helper".to_string(), false),
                ("inner".to_string(), true),
                ("Frame".to_string(), true),
                ("Frame::push".to_string(), true),
                ("Frame::grow".to_string(), false),
                ("Frame::clone".to_string(), false),
                ("Shape::area".to_string(), true),
                ("Shape".to_string(), true),
                ("make".to_string(), false),
            ]
        );
    }

    #[test]
    fn a_wanted_span_is_read_as_the_stream_of_its_text_and_a_file_that_does_not_parse_is_an_error()
    {
        let text = "pub fn a() {}\npub fn b() { 1 }\n";
        let start = text.find("pub fn b").expect("b");
        let read = scan(
            &RelPath::new("src/lib.rs").expect("a path"),
            text,
            &[Range {
                start,
                end: text.len() - 1,
            }],
        )
        .expect("it parses");
        assert_eq!(read.wanted, [stream(text, "b")]);
        assert!(
            scan(
                &RelPath::new("src/lib.rs").expect("a path"),
                "pub fn (",
                &[]
            )
            .is_err()
        );
    }
}
