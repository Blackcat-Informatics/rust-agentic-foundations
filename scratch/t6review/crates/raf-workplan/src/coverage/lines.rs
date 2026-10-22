// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reverse coverage: the lines of every package of the source workspace that
//! no capability explains.
//!
//! [`Workplan::lines_report`] reads every package the loader reaches, the root
//! package and the packages that no capability binds included, under either
//! strictness, and returns a [`LinesReport`].
//!
//! # Units and lines
//!
//! The units of a file are the units of every target the source index lists:
//! every item except a module and an `extern crate`, every impl block from its
//! first attribute to its closing brace, and every item-level macro invocation
//! and `const _` of a module, found by parsing the file. The counted lines of a
//! file are the union of the lines of its units; a file is counted once, for
//! the package of the first target that reaches it, however many targets and
//! module paths reach it. Every other line (a blank line, a `use` or `mod`
//! declaration, an inner doc comment, the SPDX header, the braces of an inline
//! module) is structural and is not counted. Lines are counted by line breaks,
//! so a file with CRLF endings reads as the same file with LF.
//!
//! # Covered
//!
//! A line is covered when a covered unit holds it. The covered units are the
//! closure of the bound items under [`Workspace::reachable`]
//! (contains, impl-of and references): the roots are the items that a `code`,
//! `tests` or `examples` binding of a loaded capability resolves to, and for an
//! `examples` binding of the crate root of an example target every item and
//! impl block of that target. A unit is covered when any item or block of its
//! source is reached. A macro invocation or a `const _` is covered when one of
//! its module paths is a module that a root opens (a bound module, or one
//! below it), when its target is an example target an `examples` binding
//! names, or, for an invocation, when a covered unit names a path that resolves
//! to nothing from the module of that unit and whose last segment is an
//! identifier inside the invocation; the `macro_rules!` a covered invocation
//! invokes is then a root too, and the closure is taken again until no
//! invocation is newly covered.
//!
//! # Exceptions
//!
//! A line that no covered unit holds is excepted when an exception of the
//! coverage record matches a unit that holds it, or its file: an `item`
//! exception the path of the unit or a path the unit is below (a method below
//! its type, an item below its module, an impl block below its self type and
//! its module), a `<module>::*` prefix every unit below the module, a `file`
//! the file, and a directory every file below it. The first matching exception
//! in the order of the record gives the reason. The exceptions are put into an
//! index once, and each unit looks up its own paths and the modules above them,
//! so the work grows with the units plus the exceptions.
//!
//! Every other counted line is an orphan line, listed under the innermost unit
//! that holds it. A line is a test line when that unit is test code (a
//! `#[cfg(test)]` module or a `#[test]` function) or is in a target that does
//! not ship (integration tests, benches, examples), and a shipping line
//! otherwise.
//!
//! An exception that matches nothing is judged at the load, without counting a
//! line: an `item` names an item of the index, a prefix a module or an item of
//! the index at or below it, a `file` a file of the index and a directory a
//! file of the index below it.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::panic;
use std::str::FromStr;
use std::thread;

use proc_macro2::{Spacing, TokenStream, TokenTree};
use raf_okf::{
    Ceiling, CeilingClass, CeilingLimit, Exception, ExceptionTarget, Location, Number, Strictness,
};
use raf_source::syn::spanned::Spanned;
use raf_source::syn::{self, Item as SynItem};
use raf_source::{
    ImplBlock, ImplId, Item, ItemId, Namespace, Node, PARSE_STACK_BYTES, RelPath, ResolveError,
    SelfType, SourceFile, SymbolKind, TargetId, TargetKind, TestRole, Workspace, parse_source,
};

use crate::finding::kind;
use crate::graph::BindingKind;
use crate::work::{Work, add};
use crate::{Finding, Loaded, Workplan};

/// The lines of one package.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CrateLines {
    package: String,
    enforced: bool,
    counted: usize,
    covered: usize,
    excepted: usize,
    shipping_counted: usize,
    test_counted: usize,
    shipping_orphan: usize,
    test_orphan: usize,
    ceilings: Vec<Ceiling>,
}

impl CrateLines {
    /// The package name.
    #[must_use]
    pub fn package(&self) -> &str {
        &self.package
    }

    /// Whether its ceilings are judged: the workplan is judged under strict
    /// mode.
    #[must_use]
    pub fn enforced(&self) -> bool {
        self.enforced
    }

    /// The counted lines of its files.
    #[must_use]
    pub fn counted(&self) -> usize {
        self.counted
    }

    /// The counted lines a covered unit holds.
    #[must_use]
    pub fn covered(&self) -> usize {
        self.covered
    }

    /// The counted lines that no covered unit holds and an exception takes
    /// out.
    #[must_use]
    pub fn excepted(&self) -> usize {
        self.excepted
    }

    /// The orphan lines: counted, not covered and not excepted.
    #[must_use]
    pub fn orphan(&self) -> usize {
        self.shipping_orphan + self.test_orphan
    }

    /// The counted lines of shipping code.
    #[must_use]
    pub fn shipping_counted(&self) -> usize {
        self.shipping_counted
    }

    /// The counted lines of test code.
    #[must_use]
    pub fn test_counted(&self) -> usize {
        self.test_counted
    }

    /// The orphan lines of shipping code.
    #[must_use]
    pub fn shipping_orphan(&self) -> usize {
        self.shipping_orphan
    }

    /// The orphan lines of test code.
    #[must_use]
    pub fn test_orphan(&self) -> usize {
        self.test_orphan
    }

    /// The rows of `ceilings` of the coverage record that name the package,
    /// in the order of the record.
    #[must_use]
    pub fn ceilings(&self) -> &[Ceiling] {
        &self.ceilings
    }

    /// The ceilings as the report writes them: each row as `<n> lines` or
    /// `<p>%`, after its class unless it is `all`, joined by `, `; `0 lines`,
    /// what strict mode holds a package with no row to, when there is none.
    #[must_use]
    pub fn ceiling_text(&self) -> String {
        if self.ceilings.is_empty() {
            return "0 lines".to_string();
        }
        let rows: Vec<String> = self.ceilings.iter().map(ceiling_text).collect();
        rows.join(", ")
    }

    /// The orphan lines of the package that a debt record holds as debt: none,
    /// since the repository holds no debt record of strict mode.
    #[must_use]
    pub fn debt(&self) -> usize {
        0
    }
}

/// One ceiling row as the report writes it.
fn ceiling_text(ceiling: &Ceiling) -> String {
    let class = match ceiling.of {
        CeilingClass::All => String::new(),
        class => format!("{} ", class.name()),
    };
    match &ceiling.limit {
        CeilingLimit::Lines(lines) => format!("{class}{} lines", lines.text()),
        CeilingLimit::Percent(percent) => format!("{class}{}%", percent.text()),
    }
}

/// The counted and orphan lines of one module that holds an orphan line.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ModuleLines {
    package: String,
    module: String,
    counted: usize,
    orphan: usize,
}

impl ModuleLines {
    /// The package name.
    #[must_use]
    pub fn package(&self) -> &str {
        &self.package
    }

    /// The module path.
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The counted lines whose innermost unit is in the module.
    #[must_use]
    pub fn counted(&self) -> usize {
        self.counted
    }

    /// Its orphan lines.
    #[must_use]
    pub fn orphan(&self) -> usize {
        self.orphan
    }
}

/// The lines of one unit that share a fate: its orphan lines, or the lines
/// one exception takes out, as merged ranges of whole lines.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnitLines {
    package: String,
    module: String,
    item: String,
    kind: &'static str,
    file: String,
    ranges: Vec<(usize, usize)>,
}

impl UnitLines {
    /// The package name.
    #[must_use]
    pub fn package(&self) -> &str {
        &self.package
    }

    /// The module path of the unit.
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The unit: the path of an item (of the first of its paths), an impl
    /// block as `<module>::<impl Type>` or `<module>::<impl Trait for Type>`
    /// and a member of an impl of a trait on a type of another crate below
    /// it, a macro invocation as `<module>::<path>!` and a `const _` as
    /// `<module>::_`.
    #[must_use]
    pub fn item(&self) -> &str {
        &self.item
    }

    /// What the unit is: `fn`, `struct`, `enum`, `union`, `trait`, `type`,
    /// `const`, `static`, `macro_rules`, `impl` or `macro`.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        self.kind
    }

    /// The file, relative to the workspace root.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// The lines, as sorted ranges `(first, last)`, both included, none
    /// adjacent to another.
    #[must_use]
    pub fn ranges(&self) -> &[(usize, usize)] {
        &self.ranges
    }

    /// How many lines the ranges hold.
    #[must_use]
    pub fn lines(&self) -> usize {
        self.ranges
            .iter()
            .map(|(first, last)| last - first + 1)
            .sum()
    }

    /// The ranges as the report writes them: `a-b`, joined by `, `.
    #[must_use]
    pub fn ranges_text(&self) -> String {
        let ranges: Vec<String> = self
            .ranges
            .iter()
            .map(|(first, last)| format!("{first}-{last}"))
            .collect();
        ranges.join(", ")
    }
}

/// The lines of a unit that an exception takes out, with the exception and
/// its reason.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExceptedLines {
    lines: UnitLines,
    exception: String,
    reason: String,
}

impl ExceptedLines {
    /// The unit and the lines.
    #[must_use]
    pub fn lines(&self) -> &UnitLines {
        &self.lines
    }

    /// The exception as the record writes it.
    #[must_use]
    pub fn exception(&self) -> &str {
        &self.exception
    }

    /// The reason of the exception.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// The reverse coverage of the workspace, or of one package of it: a row for
/// each package, the modules that hold an orphan line, the orphan lines of
/// each unit, the lines the exceptions take out, and why the lines could not
/// be counted, if they could not. The rows are sorted by package, module
/// path, file and first line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct LinesReport {
    crates: Vec<CrateLines>,
    modules: Vec<ModuleLines>,
    orphans: Vec<UnitLines>,
    excepted: Vec<ExceptedLines>,
    errors: Vec<String>,
}

impl LinesReport {
    /// Every package, sorted by name.
    #[must_use]
    pub fn crates(&self) -> &[CrateLines] {
        &self.crates
    }

    /// The modules that hold an orphan line.
    #[must_use]
    pub fn modules(&self) -> &[ModuleLines] {
        &self.modules
    }

    /// The orphan lines of each unit that holds one, each line under the
    /// innermost unit that holds it.
    #[must_use]
    pub fn orphans(&self) -> &[UnitLines] {
        &self.orphans
    }

    /// The lines each exception takes out of each unit.
    #[must_use]
    pub fn excepted(&self) -> &[ExceptedLines] {
        &self.excepted
    }

    /// Why the lines could not be counted: a file that does not parse or
    /// lex, or a walk of the index that failed. A report with an error holds
    /// no row, since no line could be judged.
    #[must_use]
    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    /// True when the report holds no orphan line and no error.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.orphans.is_empty() && self.errors.is_empty()
    }

    /// The report of one package, by its name; `None` for a name that is no
    /// package of the report.
    #[must_use]
    pub fn package(&self, name: &str) -> Option<LinesReport> {
        let row = self.crates.iter().find(|row| row.package == name)?;
        Some(Self {
            crates: vec![row.clone()],
            modules: self
                .modules
                .iter()
                .filter(|row| row.package == name)
                .cloned()
                .collect(),
            orphans: self
                .orphans
                .iter()
                .filter(|row| row.package == name)
                .cloned()
                .collect(),
            excepted: self
                .excepted
                .iter()
                .filter(|row| row.lines.package == name)
                .cloned()
                .collect(),
            errors: self.errors.clone(),
        })
    }
}

impl fmt::Display for LinesReport {
    /// The tables of [`super::render::lines_table`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&super::render::lines_table(self))
    }
}

impl Workplan<'_> {
    /// The reverse coverage of every package of the source workspace, judged
    /// against the bindings of the loaded capabilities and the exceptions and
    /// ceilings of the coverage record (none without a record), under either
    /// strictness; see [the module documentation](crate::coverage::lines).
    /// Each package says whether its ceilings are enforced, which they are
    /// under strict mode. Nothing is written and no process is started.
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::Workplan;
    ///
    /// let source = Workspace::load(
    ///     &MemoryTree::from([
    ///         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
    ///         ("src/lib.rs", "pub fn run() {\n    helper();\n}\n\nfn helper() {}\n\nfn unused() {\n}\n"),
    ///     ]),
    ///     &["."],
    /// )?;
    /// let plan = Workplan::load(&MemoryTree::from([("README.md", "# Demo\n")]), &source);
    /// let report = plan.lines_report();
    /// // Nothing is bound, so every counted line is an orphan line.
    /// assert_eq!(report.crates()[0].counted(), 6);
    /// assert_eq!(report.crates()[0].orphan(), 6);
    /// let items: Vec<(&str, String)> = report
    ///     .orphans()
    ///     .iter()
    ///     .map(|unit| (unit.item(), unit.ranges_text()))
    ///     .collect();
    /// assert_eq!(
    ///     items,
    ///     [
    ///         ("demo::run", "1-3".to_string()),
    ///         ("demo::helper", "5-5".to_string()),
    ///         ("demo::unused", "7-8".to_string()),
    ///     ]
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn lines_report(&self) -> LinesReport {
        compute(self, &[])
    }
}

/// What a unit is, with what the index holds of it.
enum What {
    /// An item, by the items of its source.
    Item(Vec<ItemId>),
    /// An impl block, by the blocks of its source.
    Impl(Vec<ImplId>),
    /// A macro invocation or a `const _` of a module.
    Module(ModuleUnit),
}

/// A macro invocation or a `const _` at the item level of a module.
struct ModuleUnit {
    /// Each module path the unit is reached under, with its target and
    /// whether it is test code there.
    copies: Vec<(String, TargetId, bool)>,
    /// The path of the invoked macro, for an invocation.
    invoked: Option<String>,
    /// The identifiers inside the invocation.
    idents: Vec<String>,
}

/// One unit of a file.
struct Unit {
    file: usize,
    first: usize,
    last: usize,
    /// The bytes of an item or an impl block.
    bytes: (usize, usize),
    what: What,
    module: String,
    item: String,
    kind: &'static str,
    shipping: bool,
    covered: bool,
    exception: Option<usize>,
}

/// One file with units.
struct File<'s> {
    path: &'s RelPath,
    package: &'s str,
    exception: Option<usize>,
}

/// The item-level macro invocations and `const _` items one parse of a file
/// found.
struct Found {
    first: usize,
    last: usize,
    /// The inline modules around it, outermost first.
    modules: Vec<String>,
    invoked: Option<String>,
    idents: Vec<String>,
}

/// The macro invocations and `const _` items at the item level of the file,
/// in source order, and of the inline modules in it.
fn module_units(path: &RelPath, text: &str) -> Result<Vec<Found>, String> {
    parse_source(path, text, |tree| {
        let mut out = Vec::new();
        found_in(&tree.items, &mut Vec::new(), &mut out);
        out
    })
    .map_err(|error| error.to_string())
}

fn found_in(items: &[SynItem], modules: &mut Vec<String>, out: &mut Vec<Found>) {
    for item in items {
        match item {
            SynItem::Macro(invocation) if invocation.ident.is_none() => {
                let span = item.span();
                let mut idents = Vec::new();
                idents_of(invocation.mac.tokens.clone(), &mut idents);
                let path: String = quote_path(&invocation.mac.path);
                out.push(Found {
                    first: span.start().line,
                    last: span.end().line,
                    modules: modules.clone(),
                    invoked: Some(path),
                    idents,
                });
            }
            SynItem::Const(constant) if constant.ident == "_" => {
                let span = item.span();
                out.push(Found {
                    first: span.start().line,
                    last: span.end().line,
                    modules: modules.clone(),
                    invoked: None,
                    idents: Vec::new(),
                });
            }
            SynItem::Mod(module) => {
                if let Some((_, content)) = &module.content {
                    let name = module.ident.to_string();
                    modules.push(name.strip_prefix("r#").unwrap_or(&name).to_string());
                    found_in(content, modules, out);
                    modules.pop();
                }
            }
            _ => {}
        }
    }
}

/// A macro path as written, without spaces: `kinds`, `crate::kinds`.
fn quote_path(path: &syn::Path) -> String {
    let mut text = String::new();
    if path.leading_colon.is_some() {
        text.push_str("::");
    }
    let segments: Vec<String> = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    text.push_str(&segments.join("::"));
    text
}

/// Every identifier of `stream`, nested groups included, walked on an
/// explicit stack.
fn idents_of(stream: TokenStream, out: &mut Vec<String>) {
    let mut stack = vec![stream.into_iter()];
    while let Some(tokens) = stack.last_mut() {
        match tokens.next() {
            Some(TokenTree::Ident(ident)) => {
                let name = ident.to_string();
                out.push(name.strip_prefix("r#").unwrap_or(&name).to_string());
            }
            Some(TokenTree::Group(group)) => stack.push(group.stream().into_iter()),
            Some(_) => {}
            None => {
                stack.pop();
            }
        }
    }
}

/// A path run of a file whose last segment is an identifier that a macro
/// invocation holds: the byte where it starts, its text and that segment.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Run {
    start: usize,
    text: String,
    segments: usize,
    last: String,
}

/// The words that are no segment of a path, beside `crate`, `self` and
/// `super`, which start one.
const KEYWORDS: [&str; 49] = [
    "Self", "abstract", "as", "async", "await", "become", "box", "break", "const", "continue",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl",
    "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe", "unsized",
    "use", "virtual", "where", "while", "yield",
];

/// The words after which an identifier is a name being declared or bound,
/// not a path.
const DECLARES: [&str; 10] = [
    "fn", "struct", "enum", "union", "trait", "type", "mod", "const", "static", "let",
];

/// The runs of `text` whose last segment is one of `wanted`, lexed on a
/// thread of its own so that the span data of the lexer dies with it.
fn runs_of(path: &RelPath, text: &str, wanted: &BTreeSet<String>) -> Result<Vec<Run>, String> {
    thread::scope(|scope| {
        let worker = thread::Builder::new()
            .name(path.to_string())
            .stack_size(PARSE_STACK_BYTES)
            .spawn_scoped(scope, move || lex_runs(text, wanted))
            .map_err(|error| format!("{path}: the thread that lexes it cannot start: {error}"))?;
        match worker.join() {
            Ok(result) => result.map_err(|error| format!("{path}: {error}")),
            Err(payload) => panic::resume_unwind(payload),
        }
    })
}

/// [`runs_of`] on the calling thread. A byte order mark and a shebang line
/// are left out, as `syn` leaves them out, and offsets stay those of `text`.
fn lex_runs(text: &str, wanted: &BTreeSet<String>) -> Result<Vec<Run>, String> {
    let mut base = 0;
    let mut body = text;
    if let Some(rest) = body.strip_prefix('\u{feff}') {
        base += '\u{feff}'.len_utf8();
        body = rest;
    }
    if let Some(after) = body.strip_prefix("#!")
        && !after.trim_start().starts_with('[')
    {
        let cut = body.find('\n').unwrap_or(body.len());
        base += cut;
        body = &body[cut..];
    }
    let stream = TokenStream::from_str(body).map_err(|error| error.to_string())?;
    let mut runs = Vec::new();
    let mut stack: Vec<(proc_macro2::token_stream::IntoIter, Scan)> =
        vec![(stream.into_iter(), Scan::default())];
    while let Some((tokens, scan)) = stack.last_mut() {
        let Some(tree) = tokens.next() else {
            scan.close(&mut runs, wanted);
            stack.pop();
            if let Some((_, outer)) = stack.last_mut() {
                outer.push(Tok::Other);
            }
            continue;
        };
        add(Work::LineTokens, 1);
        match tree {
            TokenTree::Group(group) => {
                scan.close(&mut runs, wanted);
                stack.push((group.stream().into_iter(), Scan::default()));
            }
            TokenTree::Ident(ident) => {
                let raw = ident.to_string();
                let start = base + ident.span().byte_range().start;
                scan.ident(&raw, start, &mut runs, wanted);
            }
            TokenTree::Punct(punct) => {
                scan.punct(
                    punct.as_char(),
                    punct.spacing() == Spacing::Joint,
                    &mut runs,
                    wanted,
                );
            }
            TokenTree::Literal(_) => {
                scan.close(&mut runs, wanted);
                scan.push(Tok::Other);
            }
        }
    }
    Ok(runs)
}

/// A token as the run scanner remembers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tok {
    /// An identifier, and whether the identifier after it is declared by it.
    Ident { declares: bool },
    /// A punctuation character and whether the next one joins it.
    Punct { ch: char, joint: bool },
    /// A literal or a group.
    Other,
}

/// The run scanner of one level of delimiters: the last three tokens and the
/// open run.
#[derive(Default)]
struct Scan {
    last: [Option<Tok>; 3],
    open: Option<Run>,
}

impl Scan {
    fn push(&mut self, tok: Tok) {
        self.last = [self.last[1], self.last[2], Some(tok)];
    }

    /// True when the last two tokens are `::`.
    fn colons(&self) -> bool {
        matches!(
            (self.last[1], self.last[2]),
            (
                Some(Tok::Punct {
                    ch: ':',
                    joint: true
                }),
                Some(Tok::Punct { ch: ':', .. })
            )
        )
    }

    /// True when the last token is a `.` that is not part of `..`.
    fn single_dot(&self) -> bool {
        matches!(
            self.last[2],
            Some(Tok::Punct {
                ch: '.',
                joint: false
            })
        ) && !matches!(
            self.last[1],
            Some(Tok::Punct {
                ch: '.',
                joint: true
            })
        )
    }

    /// End the open run, keeping it when its last segment is wanted.
    fn close(&mut self, runs: &mut Vec<Run>, wanted: &BTreeSet<String>) {
        if let Some(run) = self.open.take()
            && wanted.contains(&run.last)
        {
            runs.push(run);
        }
    }

    fn ident(&mut self, raw: &str, start: usize, runs: &mut Vec<Run>, wanted: &BTreeSet<String>) {
        let plain = raw.strip_prefix("r#");
        let word = plain.unwrap_or(raw);
        let path_word = plain.is_none() && matches!(word, "crate" | "self" | "super");
        let keyword = plain.is_none() && KEYWORDS.contains(&word);
        let declares = plain.is_none() && DECLARES.contains(&word);
        if self.colons() && self.open.is_some() && !keyword && !path_word {
            if let Some(run) = self.open.as_mut() {
                run.text.push_str("::");
                run.text.push_str(word);
                run.segments += 1;
                run.last = word.to_string();
            }
        } else {
            let leading = self.colons() && self.open.is_none();
            self.close(runs, wanted);
            let after_declaring = matches!(self.last[2], Some(Tok::Ident { declares: true }));
            if !keyword && !self.single_dot() && !after_declaring {
                let text = if leading {
                    format!("::{word}")
                } else {
                    word.to_string()
                };
                self.open = Some(Run {
                    start,
                    text,
                    segments: 1,
                    last: word.to_string(),
                });
            }
        }
        self.push(Tok::Ident { declares });
    }

    fn punct(&mut self, ch: char, joint: bool, runs: &mut Vec<Run>, wanted: &BTreeSet<String>) {
        let second = matches!(
            self.last[2],
            Some(Tok::Punct {
                ch: ':',
                joint: true
            })
        );
        if ch == ':' && !joint && !second {
            // A single `:`: a run of one segment before it is a name being
            // bound or a field, not a path.
            if self.open.as_ref().is_some_and(|run| run.segments == 1) {
                self.open = None;
            } else {
                self.close(runs, wanted);
            }
        } else if ch != ':' {
            self.close(runs, wanted);
        }
        self.push(Tok::Punct { ch, joint });
    }
}

/// The exceptions of the record by what each names, so that a unit looks up
/// its own paths and the modules above them, and a file its own path and the
/// directories above it, whatever the number of exceptions.
struct ExceptionIndex<'r> {
    items: HashMap<&'r str, Vec<usize>>,
    prefixes: HashMap<&'r str, Vec<usize>>,
    files: HashMap<&'r str, Vec<usize>>,
    directories: HashMap<&'r str, Vec<usize>>,
}

/// `path` and each path it is below, longest first: `a::b::c`, `a::b`, `a`.
fn ancestors(path: &str) -> impl Iterator<Item = &str> {
    std::iter::successors(Some(path), |p| p.rsplit_once("::").map(|(head, _)| head))
}

/// The directories `file` is below, deepest first: `a/b` and `a` for
/// `a/b/c.rs`.
fn directories(file: &str) -> impl Iterator<Item = &str> {
    std::iter::successors(file.rsplit_once('/').map(|(head, _)| head), |p| {
        p.rsplit_once('/').map(|(head, _)| head)
    })
}

impl<'r> ExceptionIndex<'r> {
    fn new(exceptions: &'r [Exception]) -> Self {
        let mut index = Self {
            items: HashMap::new(),
            prefixes: HashMap::new(),
            files: HashMap::new(),
            directories: HashMap::new(),
        };
        for (position, exception) in exceptions.iter().enumerate() {
            add(Work::ExceptionsIndexed, 1);
            let (map, path) = match &exception.target {
                ExceptionTarget::Item(path) => (&mut index.items, path),
                ExceptionTarget::Prefix(path) => (&mut index.prefixes, path),
                ExceptionTarget::File(path) => (&mut index.files, path),
                ExceptionTarget::Directory(path) => (&mut index.directories, path),
            };
            map.entry(path.as_str()).or_default().push(position);
        }
        index
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
            && self.prefixes.is_empty()
            && self.files.is_empty()
            && self.directories.is_empty()
    }

    /// The first exception that matches a unit at the item paths `paths` and
    /// in the modules `modules`: an item exception at a path or one it is
    /// below, a prefix exception at a module a path is below, and either at a
    /// module of `modules` or one it is below.
    fn unit(&self, paths: &[&str], modules: &[&str]) -> Option<usize> {
        if self.items.is_empty() && self.prefixes.is_empty() {
            return None;
        }
        let mut first = None;
        let mut take = |found: Option<&Vec<usize>>| {
            add(Work::ExceptionLookups, 1);
            if let Some(&position) = found.and_then(|positions| positions.first()) {
                first = Some(first.map_or(position, |seen: usize| seen.min(position)));
            }
        };
        for path in paths {
            for (depth, at) in ancestors(path).enumerate() {
                take(self.items.get(at));
                if depth > 0 {
                    take(self.prefixes.get(at));
                }
            }
        }
        for module in modules {
            for at in ancestors(module) {
                take(self.items.get(at));
                take(self.prefixes.get(at));
            }
        }
        first
    }

    /// The first exception that matches the file `file`, by its path or a
    /// directory it is below.
    fn file(&self, file: &str) -> Option<usize> {
        if self.files.is_empty() && self.directories.is_empty() {
            return None;
        }
        add(Work::ExceptionLookups, 1);
        let mut first = self
            .files
            .get(file)
            .and_then(|found| found.first())
            .copied();
        for directory in directories(file) {
            add(Work::ExceptionLookups, 1);
            if let Some(&position) = self
                .directories
                .get(directory)
                .and_then(|found| found.first())
            {
                first = Some(first.map_or(position, |seen| seen.min(position)));
            }
        }
        first
    }
}

/// The positions of the exceptions of `exceptions` that match nothing of the
/// index of `source`: an item exception that names no item, a prefix that
/// names no module or item at or below it, a file that is no file of the
/// index and a directory that holds none. No line is counted to judge it.
pub(crate) fn unmatched_exceptions(source: &Workspace, exceptions: &[Exception]) -> Vec<usize> {
    let mut matched = vec![false; exceptions.len()];
    let mut files: HashMap<&str, Vec<usize>> = HashMap::new();
    let mut dirs: HashMap<&str, Vec<usize>> = HashMap::new();
    for (position, exception) in exceptions.iter().enumerate() {
        add(Work::ExceptionsIndexed, 1);
        match &exception.target {
            ExceptionTarget::Item(path) => {
                add(Work::ExceptionLookups, 1);
                matched[position] = source
                    .items_under(path)
                    .next()
                    .is_some_and(|item| *item.path == **path);
            }
            ExceptionTarget::Prefix(path) => {
                add(Work::ExceptionLookups, 1);
                matched[position] = source.items_under(path).next().is_some();
            }
            ExceptionTarget::File(path) => files.entry(path).or_default().push(position),
            ExceptionTarget::Directory(path) => dirs.entry(path).or_default().push(position),
        }
    }
    if !files.is_empty() || !dirs.is_empty() {
        for file in source.files() {
            let path = file.path.as_str();
            add(Work::ExceptionLookups, 1);
            for position in files.get(path).into_iter().flatten() {
                matched[*position] = true;
            }
            for directory in directories(path) {
                add(Work::ExceptionLookups, 1);
                for position in dirs.get(directory).into_iter().flatten() {
                    matched[*position] = true;
                }
            }
        }
    }
    matched
        .iter()
        .enumerate()
        .filter(|(_, hit)| !**hit)
        .map(|(position, _)| position)
        .collect()
}

/// The name of the kind of an item, as the report writes it.
fn kind_name(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Fn
        | SymbolKind::ForeignFn
        | SymbolKind::InherentMethod
        | SymbolKind::TraitImplMethod
        | SymbolKind::TraitDefaultMethod
        | SymbolKind::TraitRequiredMethod => "fn",
        SymbolKind::Struct => "struct",
        SymbolKind::Enum => "enum",
        SymbolKind::Union => "union",
        SymbolKind::Trait => "trait",
        SymbolKind::TypeAlias | SymbolKind::ForeignType | SymbolKind::AssocType => "type",
        SymbolKind::Const | SymbolKind::AssocConst => "const",
        SymbolKind::Static | SymbolKind::ForeignStatic => "static",
        SymbolKind::MacroRules => "macro_rules",
        _ => "item",
    }
}

/// An impl block as the report names it: `<module>::<impl Type>` or
/// `<module>::<impl Trait for Type>`.
fn impl_name(block: &ImplBlock) -> String {
    match &block.trait_path {
        Some(trait_path) => format!(
            "{}::<impl {trait_path} for {}>",
            block.module, block.self_type
        ),
        None => format!("{}::<impl {}>", block.module, block.self_type),
    }
}

/// `path` without its last segment.
fn parent(path: &str) -> &str {
    path.rsplit_once("::").map_or(path, |(head, _)| head)
}

/// The module of an item: the module of its impl block for a member of one,
/// the module of its trait for a member of a trait, and the module its path
/// names otherwise.
fn item_module(source: &Workspace, item: &Item, block_of: &HashMap<ItemId, ImplId>) -> String {
    if let Some(block) = block_of.get(&item.id).and_then(|id| source.impl_block(*id)) {
        return block.module.to_string();
    }
    match item.kind {
        SymbolKind::TraitRequiredMethod
        | SymbolKind::TraitDefaultMethod
        | SymbolKind::AssocConst
        | SymbolKind::AssocType => parent(parent(&item.path)).to_string(),
        _ => parent(&item.path).to_string(),
    }
}

/// The unit of the items of one source, the item of the first path first.
fn item_unit(
    source: &Workspace,
    file: usize,
    mut copies: Vec<&Item>,
    block_of: &HashMap<ItemId, ImplId>,
) -> Unit {
    copies.sort_by(|a, b| (&a.path, a.id).cmp(&(&b.path, b.id)));
    let first = copies[0];
    let block = block_of
        .get(&first.id)
        .and_then(|id| source.impl_block(*id));
    let item = match (first.self_key, block) {
        (Some(SelfType::Trait), Some(block)) => format!("{}::{}", impl_name(block), first.name),
        _ => first.path.to_string(),
    };
    Unit {
        file,
        first: first.span.lines.start,
        last: first
            .span
            .lines
            .end
            .saturating_sub(1)
            .max(first.span.lines.start),
        bytes: (first.span.start_byte, first.span.end_byte),
        module: item_module(source, first, block_of),
        item,
        kind: kind_name(first.kind),
        shipping: copies.iter().any(|copy| source.is_shipping(copy)),
        what: What::Item(copies.iter().map(|copy| copy.id).collect()),
        covered: false,
        exception: None,
    }
}

/// The unit of the impl blocks of one source.
fn impl_unit(source: &Workspace, file: usize, copies: &[&ImplBlock]) -> Unit {
    let first = copies[0];
    Unit {
        file,
        first: first.span.lines.start,
        last: first
            .span
            .lines
            .end
            .saturating_sub(1)
            .max(first.span.lines.start),
        bytes: (first.span.start_byte, first.span.end_byte),
        module: first.module.to_string(),
        item: impl_name(first),
        kind: "impl",
        shipping: copies.iter().any(|block| {
            block.test == TestRole::None
                && source
                    .target(block.target)
                    .is_some_and(|target| target.shipping)
        }),
        what: What::Impl(copies.iter().map(|block| block.id).collect()),
        covered: false,
        exception: None,
    }
}

/// The unit of a macro invocation or a `const _` that a parse of `file`
/// found, once for each module path the file is reached under.
fn module_unit(source: &Workspace, index: usize, file: &SourceFile, found: Found) -> Unit {
    let suffix = found.modules.join("::");
    let mut copies: Vec<(String, TargetId, bool)> = file
        .reached
        .iter()
        .map(|reach| {
            let module = if suffix.is_empty() {
                reach.module_path.to_string()
            } else {
                format!("{}::{suffix}", reach.module_path)
            };
            // An inline module guarded by `test` makes what it holds test
            // code, as it makes its items.
            let inner_test = !suffix.is_empty()
                && source
                    .items_under(&module)
                    .take_while(|item| *item.path == *module)
                    .any(|item| {
                        item.kind == SymbolKind::Mod
                            && item.target == reach.target
                            && item.test != TestRole::None
                    });
            (module, reach.target, reach.test || inner_test)
        })
        .collect();
    copies.sort();
    copies.dedup();
    let shipping = copies.iter().any(|(_, target, test)| {
        !test && source.target(*target).is_some_and(|target| target.shipping)
    });
    let module = copies
        .first()
        .map_or_else(String::new, |(module, _, _)| module.clone());
    let (item, kind) = match &found.invoked {
        Some(path) => (format!("{module}::{path}!"), "macro"),
        None => (format!("{module}::_"), "const"),
    };
    Unit {
        file: index,
        first: found.first,
        last: found.last.max(found.first),
        bytes: (0, 0),
        module,
        item,
        kind,
        shipping,
        what: What::Module(ModuleUnit {
            copies,
            invoked: found.invoked,
            idents: found.idents,
        }),
        covered: false,
        exception: None,
    }
}

/// The units of every file of the source workspace.
struct Gathered<'s> {
    files: Vec<File<'s>>,
    units: Vec<Unit>,
    /// The units of each file, as a range of `units`.
    of_file: Vec<std::ops::Range<usize>>,
    /// The runs whose last segment a macro invocation holds, each with the
    /// innermost item or impl unit that holds its start.
    runs: Vec<(Run, Option<usize>)>,
}

/// The units of every file of `source`, file by file in the order of the
/// files.
fn collect<'s>(source: &'s Workspace, judged: &BTreeSet<&str>) -> Result<Gathered<'s>, String> {
    let mut block_of: HashMap<ItemId, ImplId> = HashMap::new();
    let mut blocks: BTreeMap<&RelPath, BTreeMap<(usize, usize), Vec<&ImplBlock>>> = BTreeMap::new();
    for block in source.impls() {
        for member in &block.members {
            block_of.insert(*member, block.id);
        }
        blocks
            .entry(&block.file)
            .or_default()
            .entry((block.span.start_byte, block.span.end_byte))
            .or_default()
            .push(block);
    }
    let mut out = Gathered {
        files: Vec::new(),
        units: Vec::new(),
        of_file: Vec::new(),
        runs: Vec::new(),
    };
    for (index, file) in source
        .files()
        .filter(|file| judged.contains(file.package.as_str()))
        .enumerate()
    {
        let begin = out.units.len();
        out.files.push(File {
            path: &file.path,
            package: &file.package,
            exception: None,
        });
        let mut groups: BTreeMap<(usize, usize), Vec<&Item>> = BTreeMap::new();
        for item in source
            .items_in(&file.path)
            .filter(|item| !matches!(item.kind, SymbolKind::Mod | SymbolKind::ExternCrate))
        {
            groups
                .entry((item.span.start_byte, item.span.end_byte))
                .or_default()
                .push(item);
        }
        for copies in groups.into_values() {
            out.units.push(item_unit(source, index, copies, &block_of));
        }
        for copies in blocks
            .get(&file.path)
            .into_iter()
            .flat_map(BTreeMap::values)
        {
            out.units.push(impl_unit(source, index, copies));
        }
        for found in module_units(&file.path, &file.text)? {
            out.units.push(module_unit(source, index, file, found));
        }
        out.of_file.push(begin..out.units.len());
    }
    let wanted: BTreeSet<String> = out
        .units
        .iter()
        .filter_map(|unit| match &unit.what {
            What::Module(module) if module.invoked.is_some() => Some(&module.idents),
            _ => None,
        })
        .flatten()
        .cloned()
        .collect();
    if !wanted.is_empty() {
        for (index, file) in source
            .files()
            .filter(|file| judged.contains(file.package.as_str()))
            .enumerate()
        {
            let runs = runs_of(&file.path, &file.text, &wanted)?;
            let range = out.of_file[index].clone();
            place(&out.units, range, runs, &mut out.runs);
        }
    }
    Ok(out)
}

/// Give each run of a file to the innermost item or impl unit of `range`
/// whose bytes hold its start, in one sweep of both in byte order.
fn place(
    units: &[Unit],
    range: std::ops::Range<usize>,
    mut runs: Vec<Run>,
    out: &mut Vec<(Run, Option<usize>)>,
) {
    let mut spans: Vec<(usize, usize, usize)> = range
        .filter(|at| !matches!(units[*at].what, What::Module(_)))
        .map(|at| (units[at].bytes.0, units[at].bytes.1, at))
        .collect();
    // Outer units first where two start at one byte.
    spans.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
    runs.sort_by_key(|run| run.start);
    let mut open: Vec<(usize, usize, usize)> = Vec::new();
    let mut next = 0;
    for run in runs {
        while let Some(&span) = spans.get(next).filter(|span| span.0 <= run.start) {
            while open.last().is_some_and(|top| top.1 <= span.0) {
                open.pop();
            }
            open.push(span);
            next += 1;
        }
        while open.last().is_some_and(|top| top.1 <= run.start) {
            open.pop();
        }
        let holder = open.last().map(|span| span.2);
        out.push((run, holder));
    }
}

/// The roots of the walk: the bound items, and every item and impl block of
/// an example target an `examples` binding names; the paths of the bound
/// modules, which open what they hold; and those example targets.
struct Roots {
    nodes: BTreeSet<Node>,
    opened: BTreeSet<String>,
    whole: BTreeSet<TargetId>,
}

fn roots(plan: &Workplan<'_>) -> Roots {
    let source = plan.source();
    let mut roots = Roots {
        nodes: BTreeSet::new(),
        opened: BTreeSet::new(),
        whole: BTreeSet::new(),
    };
    for binding in plan.graph().bindings() {
        for id in binding.items() {
            let Some(item) = source.item(*id) else {
                continue;
            };
            let target = source.target(item.target);
            let crate_root = target.is_some_and(|target| *item.path == *target.crate_path);
            if binding.kind() == BindingKind::Example
                && crate_root
                && target.is_some_and(|target| target.kind == TargetKind::Example)
            {
                roots.whole.insert(item.target);
                continue;
            }
            roots.nodes.insert(Node::Item(*id));
            if item.kind == SymbolKind::Mod && !crate_root {
                roots.opened.insert(item.path.to_string());
            }
        }
    }
    if !roots.whole.is_empty() {
        for item in source
            .items()
            .filter(|item| roots.whole.contains(&item.target))
        {
            roots.nodes.insert(Node::Item(item.id));
        }
        for block in source
            .impls()
            .filter(|block| roots.whole.contains(&block.target))
        {
            roots.nodes.insert(Node::Impl(block.id));
        }
    }
    roots
}

/// Mark the covered units: the closure of the roots, the macro invocations
/// and `const _` items of the opened modules and the example targets, and the
/// invocations a covered unit names a generated item of, whose `macro_rules!`
/// join the roots until no invocation is newly covered. `extra` roots are
/// walked beside them.
fn cover(
    source: &Workspace,
    units: &mut Gathered<'_>,
    roots: &mut Roots,
    extra: &[Node],
) -> Result<(), String> {
    let mut unresolved: HashMap<(String, String), bool> = HashMap::new();
    loop {
        add(Work::LineRounds, 1);
        let mut nodes: Vec<Node> = roots.nodes.iter().copied().collect();
        nodes.extend_from_slice(extra);
        let reach = source
            .reachable(&nodes)
            .map_err(|error| format!("the walk of the bound items failed: {error}"))?;
        for unit in &mut units.units {
            unit.covered = match &unit.what {
                What::Item(ids) => ids.iter().any(|id| reach.contains(Node::Item(*id))),
                What::Impl(ids) => ids.iter().any(|id| reach.contains(Node::Impl(*id))),
                What::Module(module) => module.copies.iter().any(|(path, target, _)| {
                    roots.whole.contains(target)
                        || ancestors(path).any(|at| roots.opened.contains(at))
                }),
            };
        }
        let mut named: BTreeSet<String> = BTreeSet::new();
        for (run, holder) in &units.runs {
            add(Work::LineRuns, 1);
            let Some(unit) = holder
                .map(|at| &units.units[at])
                .filter(|unit| unit.covered)
            else {
                continue;
            };
            let key = (unit.module.clone(), run.text.clone());
            let missing = *unresolved.entry(key).or_insert_with(|| {
                matches!(
                    source.resolve_from(&unit.module, &run.text),
                    Err(ResolveError::NotFound(_))
                )
            });
            if missing {
                named.insert(run.last.clone());
            }
        }
        let mut grew = false;
        for unit in &mut units.units {
            let What::Module(module) = &unit.what else {
                continue;
            };
            if !unit.covered && module.idents.iter().any(|ident| named.contains(ident)) {
                unit.covered = true;
            }
            let (true, Some(invoked)) = (unit.covered, &module.invoked) else {
                continue;
            };
            for (path, _, _) in &module.copies {
                for site in invoked_sites(source, path, invoked) {
                    grew |= roots.nodes.insert(Node::Item(site));
                }
            }
        }
        if !grew {
            return Ok(());
        }
    }
}

/// The macros that the invocation of `invoked` in the module `module` names:
/// the path read from the module in the macro name space, and for a name of
/// one segment from each module above it too, as a `macro_rules!` of a parent
/// module is in the textual scope of the modules declared after it.
fn invoked_sites(source: &Workspace, module: &str, invoked: &str) -> Vec<ItemId> {
    let modules: Vec<&str> = if invoked.contains("::") {
        vec![module]
    } else {
        ancestors(module).collect()
    };
    for at in modules {
        if let Ok(found) = source.resolve_from_in(at, invoked, Namespace::Macro) {
            return found.sites;
        }
    }
    Vec::new()
}

/// The paths and the modules a unit is matched against the exceptions by:
/// the paths of an item, the self types and the modules of an impl block, and
/// the modules of a macro invocation or a `const _`.
fn match_paths(source: &Workspace, what: &What) -> (Vec<String>, Vec<String>) {
    match what {
        What::Item(ids) => (
            ids.iter()
                .filter_map(|id| source.item(*id))
                .map(|item| item.path.to_string())
                .collect(),
            Vec::new(),
        ),
        What::Impl(ids) => {
            let blocks: Vec<&ImplBlock> =
                ids.iter().filter_map(|id| source.impl_block(*id)).collect();
            (
                blocks
                    .iter()
                    .flat_map(|block| &block.self_items)
                    .filter_map(|id| source.item(*id))
                    .map(|item| item.path.to_string())
                    .collect(),
                blocks
                    .iter()
                    .map(|block| block.module.to_string())
                    .collect(),
            )
        }
        What::Module(module) => (
            Vec::new(),
            module
                .copies
                .iter()
                .map(|(path, _, _)| path.clone())
                .collect(),
        ),
    }
}

/// What the lines of the units add up to.
#[derive(Default)]
struct Totals {
    crates: BTreeMap<String, CrateLines>,
    modules: BTreeMap<(String, String), (usize, usize)>,
    orphans: BTreeMap<usize, Vec<usize>>,
    excepted: BTreeMap<(usize, usize), Vec<usize>>,
}

/// Judge every counted line of one file: covered when a covered unit holds
/// it, excepted when an exception matches a unit that holds it or the file,
/// and orphan otherwise, each under the innermost unit that holds it.
fn sweep(units: &[Unit], range: std::ops::Range<usize>, file: &File<'_>, totals: &mut Totals) {
    let Some(last) = range.clone().map(|at| units[at].last).max() else {
        return;
    };
    let mut covered = vec![false; last + 1];
    let mut exception: Vec<Option<usize>> = vec![file.exception; last + 1];
    let mut inner: Vec<Option<usize>> = vec![None; last + 1];
    let mut order: Vec<usize> = range.collect();
    // Longer units first, so the innermost unit of a line is the last to mark
    // it, and of two units of one length the first in the file.
    order.sort_by(|a, b| {
        let (a, b) = (&units[*a], &units[*b]);
        (b.last - b.first, b.first, b.bytes.0).cmp(&(a.last - a.first, a.first, a.bytes.0))
    });
    for at in order {
        add(Work::LineUnits, 1);
        let unit = &units[at];
        for line in unit.first..=unit.last {
            add(Work::LineMarks, 1);
            covered[line] |= unit.covered;
            exception[line] = match (exception[line], unit.exception) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
            inner[line] = Some(at);
        }
    }
    for (line, holder) in inner.iter().enumerate() {
        let Some(at) = *holder else {
            continue;
        };
        let unit = &units[at];
        let row = totals
            .crates
            .entry(file.package.to_string())
            .or_insert_with(|| empty_row(file.package));
        let module = totals
            .modules
            .entry((file.package.to_string(), unit.module.clone()))
            .or_default();
        module.0 += 1;
        if unit.shipping {
            row.shipping_counted += 1;
        } else {
            row.test_counted += 1;
        }
        row.counted += 1;
        if covered[line] {
            row.covered += 1;
        } else if let Some(position) = exception[line] {
            row.excepted += 1;
            totals
                .excepted
                .entry((at, position))
                .or_default()
                .push(line);
        } else {
            module.1 += 1;
            if unit.shipping {
                row.shipping_orphan += 1;
            } else {
                row.test_orphan += 1;
            }
            totals.orphans.entry(at).or_default().push(line);
        }
    }
}

/// The row of a package with no line.
fn empty_row(package: &str) -> CrateLines {
    CrateLines {
        package: package.to_string(),
        enforced: false,
        counted: 0,
        covered: 0,
        excepted: 0,
        shipping_counted: 0,
        test_counted: 0,
        shipping_orphan: 0,
        test_orphan: 0,
        ceilings: Vec::new(),
    }
}

/// Sorted lines as merged ranges.
fn ranges(lines: &[usize]) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for line in lines {
        match out.last_mut() {
            Some((_, last)) if *last + 1 == *line => *last = *line,
            _ => out.push((*line, *line)),
        }
    }
    out
}

/// The lines of the unit `unit` of the file `file`, as rows of the report.
fn unit_lines(unit: &Unit, file: &File<'_>, lines: &[usize]) -> UnitLines {
    UnitLines {
        package: file.package.to_string(),
        module: unit.module.clone(),
        item: unit.item.clone(),
        kind: unit.kind,
        file: file.path.to_string(),
        ranges: ranges(lines),
    }
}

/// The order of the rows of units: package, module path, file, first line.
fn unit_order(lines: &UnitLines) -> (&str, &str, &str, usize) {
    (
        &lines.package,
        &lines.module,
        &lines.file,
        lines.ranges.first().map_or(0, |range| range.0),
    )
}

/// The packages the report judges: every package of `source`.
fn judged_packages(source: &Workspace) -> BTreeSet<&str> {
    source
        .packages()
        .iter()
        .map(|package| package.name.as_str())
        .collect()
}

/// The report of `plan`, with `extra` roots walked beside the bound items.
fn compute(plan: &Workplan<'_>, extra: &[Node]) -> LinesReport {
    match judged(plan, extra) {
        Ok(report) => report,
        Err(error) => LinesReport {
            errors: vec![error],
            ..LinesReport::default()
        },
    }
}

fn judged(plan: &Workplan<'_>, extra: &[Node]) -> Result<LinesReport, String> {
    let source = plan.source();
    let record = plan.coverage().map(Loaded::value);
    let exceptions: &[Exception] = record.map_or(&[], |record| &record.exceptions);
    let judged = judged_packages(source);
    let mut units = collect(source, &judged)?;
    let mut roots = roots(plan);
    cover(source, &mut units, &mut roots, extra)?;
    let index = ExceptionIndex::new(exceptions);
    if !index.is_empty() {
        for unit in &mut units.units {
            let (paths, modules) = match_paths(source, &unit.what);
            let paths: Vec<&str> = paths.iter().map(String::as_str).collect();
            let modules: Vec<&str> = modules.iter().map(String::as_str).collect();
            unit.exception = index.unit(&paths, &modules);
        }
        for file in &mut units.files {
            file.exception = index.file(file.path.as_str());
        }
    }
    let mut totals = Totals::default();
    for (file, range) in units.files.iter().zip(&units.of_file) {
        sweep(&units.units, range.clone(), file, &mut totals);
    }
    let enforced = plan.strictness() == Strictness::Strict;
    let mut report = LinesReport::default();
    for package in source
        .packages()
        .iter()
        .filter(|package| judged.contains(package.name.as_str()))
    {
        let mut row = totals
            .crates
            .remove(&package.name)
            .unwrap_or_else(|| empty_row(&package.name));
        row.enforced = enforced;
        row.ceilings = record.map_or_else(Vec::new, |record| {
            record
                .ceilings
                .iter()
                .filter(|ceiling| ceiling.package == package.name)
                .cloned()
                .collect()
        });
        report.crates.push(row);
    }
    report.crates.sort_by(|a, b| a.package.cmp(&b.package));
    report.modules = totals
        .modules
        .into_iter()
        .filter(|(_, (_, orphan))| *orphan > 0)
        .map(|((package, module), (counted, orphan))| ModuleLines {
            package,
            module,
            counted,
            orphan,
        })
        .collect();
    for (at, lines) in &totals.orphans {
        let unit = &units.units[*at];
        report
            .orphans
            .push(unit_lines(unit, &units.files[unit.file], lines));
    }
    report
        .orphans
        .sort_by(|a, b| unit_order(a).cmp(&unit_order(b)));
    let mut excepted: Vec<(usize, ExceptedLines)> = Vec::new();
    for ((at, position), lines) in &totals.excepted {
        let unit = &units.units[*at];
        let exception = &exceptions[*position];
        excepted.push((
            *position,
            ExceptedLines {
                lines: unit_lines(unit, &units.files[unit.file], lines),
                exception: exception.target.written(),
                reason: exception.reason.clone(),
            },
        ));
    }
    excepted.sort_by(|a, b| (unit_order(&a.1.lines), a.0).cmp(&(unit_order(&b.1.lines), b.0)));
    report.excepted = excepted.into_iter().map(|(_, row)| row).collect();
    Ok(report)
}

/// `digits`, a whole number in decimal, times `factor`, in decimal.
fn times(digits: &str, factor: u128) -> String {
    let mut out: Vec<u8> = Vec::with_capacity(digits.len() + 40);
    let mut carry: u128 = 0;
    for digit in digits.bytes().rev() {
        let value = u128::from(digit - b'0') * factor + carry;
        out.push(b'0' + u8::try_from(value % 10).unwrap_or(0));
        carry = value / 10;
    }
    while carry > 0 {
        out.push(b'0' + u8::try_from(carry % 10).unwrap_or(0));
        carry /= 10;
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

/// Whether `orphan` of `counted` lines is within `limit`: at most `n` lines,
/// or `orphan * 100 * scale <= percent * scale * counted` in whole numbers,
/// `scale` being the power of ten that makes the percent whole, so no
/// division rounds and no count overflows.
fn within(orphan: usize, counted: usize, limit: &CeilingLimit) -> bool {
    let number = |text: &str| Number::parse(text);
    match limit {
        CeilingLimit::Lines(lines) => number(&orphan.to_string())
            .is_some_and(|orphan| orphan.compare(lines) != Ordering::Greater),
        CeilingLimit::Percent(percent) => {
            let text = percent.text();
            let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
            let fraction = fraction.trim_end_matches('0');
            let left = format!("{}{}", orphan as u128 * 100, "0".repeat(fraction.len()));
            let right = times(&format!("{whole}{fraction}"), counted as u128);
            match (number(&left), number(&right)) {
                (Some(left), Some(right)) => left.compare(&right) != Ordering::Greater,
                _ => false,
            }
        }
    }
}

/// The findings of strict mode for the ceilings of the coverage record: each
/// class of a package whose orphan lines are over its ceiling is
/// `coverage-lines-over-ceiling` at `ceilings`. A package's shipping and test
/// lines are each judged against the row of their class, held to 0 when no
/// row of their class and no `all` row bounds them, and their sum against an
/// `all` row. Without a coverage record that loads, or while a capability
/// document does not load, nothing is judged: the record's own finding, or the
/// document's, fails the check, and a document that did not load binds
/// nothing. A report that could not count the lines is one finding.
pub(crate) fn over_ceilings(plan: &Workplan<'_>) -> Vec<Finding> {
    let Some(loaded) = plan.coverage() else {
        return Vec::new();
    };
    let capability_unloaded = plan
        .unloaded()
        .iter()
        .any(|id| id.starts_with(raf_okf::DocType::Capability.id_prefix()));
    if capability_unloaded {
        return Vec::new();
    }
    ceiling_findings(loaded, &plan.lines_report())
}

/// The findings of [`over_ceilings`] for `report`, at the record `loaded`.
fn ceiling_findings(
    loaded: &Loaded<raf_okf::CoverageRecord>,
    report: &LinesReport,
) -> Vec<Finding> {
    let finding = |message: String| {
        Finding::new(
            kind::COVERAGE_LINES_OVER_CEILING,
            loaded.path(),
            Location::Key("ceilings".to_string()),
            message,
            "bind or reach the items that explain the orphan lines, add an exception with its reason, or raise the crate's ceiling",
        )
    };
    if let Some(error) = report.errors().first() {
        return vec![finding(format!(
            "the orphan lines of the workspace cannot be counted, so no package is judged under its ceiling: {error}"
        ))];
    }
    let mut out = Vec::new();
    for row in report.crates() {
        let class = |of: CeilingClass| row.ceilings.iter().find(|ceiling| ceiling.of == of);
        let all = class(CeilingClass::All);
        let mut judge =
            |of: CeilingClass, orphan: usize, counted: usize, ceiling: Option<&Ceiling>| {
                let held = ceiling.map_or(orphan == 0, |ceiling| {
                    within(orphan, counted, &ceiling.limit)
                });
                if !held {
                    out.push(finding(over_message(
                        report, row, of, orphan, counted, ceiling,
                    )));
                }
            };
        for (of, orphan, counted) in [
            (
                CeilingClass::Shipping,
                row.shipping_orphan,
                row.shipping_counted,
            ),
            (CeilingClass::Test, row.test_orphan, row.test_counted),
        ] {
            match class(of) {
                Some(ceiling) => judge(of, orphan, counted, Some(ceiling)),
                None if all.is_none() => judge(of, orphan, counted, None),
                None => {}
            }
        }
        if let Some(ceiling) = all {
            judge(CeilingClass::All, row.orphan(), row.counted, Some(ceiling));
        }
    }
    out
}

/// The message of a class of a package over its ceiling.
fn over_message(
    report: &LinesReport,
    row: &CrateLines,
    of: CeilingClass,
    orphan: usize,
    counted: usize,
    ceiling: Option<&Ceiling>,
) -> String {
    let lines = match of {
        CeilingClass::Shipping => "shipping orphan lines",
        CeilingClass::Test => "test orphan lines",
        CeilingClass::All => "orphan lines",
    };
    let bound = ceiling.map_or_else(
        || "the default of 0 lines, since no row of `ceilings` bounds them".to_string(),
        |ceiling| format!("its ceiling of {}", ceiling_text(ceiling)),
    );
    let mut largest: Vec<&UnitLines> = report
        .orphans()
        .iter()
        .filter(|unit| unit.package == row.package)
        .collect();
    largest.sort_by(|a, b| b.lines().cmp(&a.lines()).then(a.item.cmp(&b.item)));
    let named: Vec<String> = largest
        .iter()
        .take(3)
        .map(|unit| format!("`{}` ({})", unit.item, unit.lines()))
        .collect();
    format!(
        "`{}` has {} orphan {} ({} shipping, {} test) of {} counted, and its {lines}, {orphan} of {counted} counted, are over {bound}; the largest orphan items are {}",
        row.package,
        row.orphan(),
        if row.orphan() == 1 { "line" } else { "lines" },
        row.shipping_orphan,
        row.test_orphan,
        row.counted,
        named.join(", ")
    )
}

#[cfg(test)]
mod tests;
