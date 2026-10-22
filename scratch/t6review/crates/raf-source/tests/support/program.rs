// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Library crates of several modules for the differential test against
//! `rustc`: their text, the questions asked of them (probes), the items they
//! declare with their identity tags, and the generator that draws them.
//!
//! A program is lines. Every declaration is one line, a module is a line
//! `mod name {` up to a line `}`, and a probe is a `use` of a path as seen by
//! a chosen module, written under a unique alias at the end of that module.
//! Every generated item carries a unique tag: a struct has the associated
//! constant `TAG`, and a function, which cannot have one, returns its tag.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use super::random::Draw;

/// The name of the generated crate.
pub(crate) const CRATE: &str = "demo";

/// The names the items and imports of a program draw from; few, so that
/// they collide.
pub(crate) const NAMES: [&str; 3] = ["A", "B", "C"];

/// The prefix of a line of a stored program that names a probe.
const PROBE_PREFIX: &str = "// probe ";

/// The prefix of the header line of a stored program that records what is
/// expected of it.
pub(crate) const EXPECT_PREFIX: &str = "// oracle: ";

/// One question: the `use` path `path`, written in the module `module`
/// (a canonical path such as `demo::m0`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Probe {
    pub(crate) module: String,
    pub(crate) path: String,
}

impl Probe {
    /// The name the path ends with.
    pub(crate) fn name(&self) -> &str {
        self.path.rsplit("::").next().unwrap_or(&self.path)
    }
}

/// A name space of a generated item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Space {
    Type,
    Value,
}

/// The visibility a generated declaration is written with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Vis {
    Private,
    Public,
    Crate,
}

impl Vis {
    fn written(self) -> &'static str {
        match self {
            Self::Private => "",
            Self::Public => "pub ",
            Self::Crate => "pub(crate) ",
        }
    }
}

/// An item a program declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Decl {
    /// The canonical path of the module that declares it.
    pub(crate) module: String,
    pub(crate) name: String,
    pub(crate) space: Space,
    pub(crate) tag: u32,
}

impl Decl {
    /// The canonical path of the item.
    pub(crate) fn path(&self) -> String {
        format!("{}::{}", self.module, self.name)
    }
}

/// A `use` a program declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UseDecl {
    /// The canonical path of the module that declares it.
    pub(crate) module: String,
    pub(crate) vis: Vis,
    /// True for a glob; the binding is then `None`.
    pub(crate) glob: bool,
    /// The name a named import binds.
    pub(crate) binding: Option<String>,
    /// The module path as written, such as `crate::m0` or `super`, that the
    /// glob brings in or the named import takes its name from.
    pub(crate) from: String,
}

/// What a line of a program is.
#[derive(Debug, PartialEq, Eq)]
enum Line {
    Open(String),
    Close,
    Item {
        name: String,
        space: Space,
        tag: u32,
    },
    Use {
        vis: Vis,
        glob: bool,
        binding: Option<String>,
        from: String,
    },
    Other,
}

fn split_vis(line: &str) -> (Vis, &str) {
    if let Some(rest) = line.strip_prefix("pub(crate) ") {
        (Vis::Crate, rest)
    } else if let Some(rest) = line.strip_prefix("pub ") {
        (Vis::Public, rest)
    } else {
        (Vis::Private, line)
    }
}

fn read_line(text: &str) -> Line {
    let text = text.trim();
    if text == "}" {
        return Line::Close;
    }
    let (vis, rest) = split_vis(text);
    if let Some(name) = rest
        .strip_prefix("mod ")
        .and_then(|rest| rest.strip_suffix(" {"))
    {
        return Line::Open(name.to_owned());
    }
    if let Some(rest) = rest.strip_prefix("struct ")
        && let Some((name, tail)) = rest.split_once(" {} impl ")
        && let Some(tag) = tail
            .split_once(" { pub const TAG: u32 = ")
            .and_then(|(_, number)| number.strip_suffix("; }"))
            .and_then(|number| number.parse().ok())
    {
        return Line::Item {
            name: name.to_owned(),
            space: Space::Type,
            tag,
        };
    }
    if let Some(rest) = rest.strip_prefix("const fn ")
        && let Some((name, tail)) = rest.split_once("() -> u32 { ")
        && let Some(tag) = tail
            .strip_suffix(" }")
            .and_then(|number| number.parse().ok())
    {
        return Line::Item {
            name: name.to_owned(),
            space: Space::Value,
            tag,
        };
    }
    if let Some(rest) = rest
        .strip_prefix("use ")
        .and_then(|rest| rest.strip_suffix(';'))
    {
        if let Some(from) = rest.strip_suffix("::*") {
            return Line::Use {
                vis,
                glob: true,
                binding: None,
                from: from.to_owned(),
            };
        }
        let (path, alias) = rest
            .rsplit_once(" as ")
            .map_or((rest, None), |(path, alias)| (path, Some(alias)));
        let (from, last) = path.rsplit_once("::").unwrap_or(("", path));
        return Line::Use {
            vis,
            glob: false,
            binding: Some(alias.unwrap_or(last).to_owned()),
            from: from.to_owned(),
        };
    }
    Line::Other
}

/// A declaration of a program, or a module with what it holds.
enum Node {
    Line(String),
    Module(String, Vec<Node>, String),
}

/// Who wrote a line of the rendered source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Owner {
    /// The line `lines[at]` of the program.
    Decl(usize),
    /// The `use` of the probe.
    Probe(usize),
    /// The assertion `at` of the probe.
    Assert(usize, usize),
    /// A module's opening or closing line.
    Frame,
}

/// A program rendered as the text of a library root.
#[derive(Debug)]
pub(crate) struct Rendered {
    pub(crate) source: String,
    /// The owner of each line, from the first.
    pub(crate) owners: Vec<Owner>,
}

/// What a rendering writes at the end of a module, given its canonical path
/// and where to put each line.
trait Emit: Fn(&str, &mut dyn FnMut(&str, Owner)) {}

impl<F: Fn(&str, &mut dyn FnMut(&str, Owner))> Emit for F {}

/// A program rendered with one probe asked as a question per name space.
#[derive(Debug)]
pub(crate) struct SplitRendered {
    pub(crate) rendered: Rendered,
    /// The line, counted from 1, of the question in the type name space.
    pub(crate) type_line: usize,
    /// The line, counted from 1, of the question in the value name space.
    pub(crate) value_line: usize,
}

/// A library crate and the questions asked of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Program {
    pub(crate) lines: Vec<String>,
    pub(crate) probes: Vec<Probe>,
    /// What a stored program is expected to show (see [`EXPECT_PREFIX`]).
    pub(crate) expect: Option<String>,
}

impl Program {
    /// Read a stored program: header lines for the probes and the
    /// expectation, then the lines of the crate.
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        let mut program = Self {
            lines: Vec::new(),
            probes: Vec::new(),
            expect: None,
        };
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix(PROBE_PREFIX) {
                let (module, path) = rest
                    .split_once(' ')
                    .ok_or_else(|| format!("a probe needs a module and a path: {line}"))?;
                program.probes.push(Probe {
                    module: module.to_owned(),
                    path: path.to_owned(),
                });
            } else if let Some(rest) = line.strip_prefix(EXPECT_PREFIX) {
                program.expect = Some(rest.to_owned());
            } else if !line.trim().is_empty() {
                program.lines.push(line.to_owned());
            }
        }
        Ok(program)
    }

    /// The stored form, which [`Program::parse`] reads back.
    pub(crate) fn to_text(&self) -> String {
        let mut text = String::new();
        if let Some(expect) = &self.expect {
            writeln!(text, "{EXPECT_PREFIX}{expect}").expect("write");
        }
        for probe in &self.probes {
            writeln!(text, "{PROBE_PREFIX}{} {}", probe.module, probe.path).expect("write");
        }
        for line in &self.lines {
            writeln!(text, "{line}").expect("write");
        }
        text
    }

    /// Walk the lines with the canonical path of the module each is in.
    fn walk(&self, mut visit: impl FnMut(usize, &str, Line)) {
        let mut stack = vec![CRATE.to_owned()];
        for (at, text) in self.lines.iter().enumerate() {
            match read_line(text) {
                Line::Open(name) => {
                    let module = stack.last().cloned().unwrap_or_default();
                    visit(at, &module, Line::Open(name.clone()));
                    stack.push(format!("{module}::{name}"));
                }
                Line::Close => {
                    stack.pop();
                    let module = stack.last().cloned().unwrap_or_default();
                    visit(at, &module, Line::Close);
                }
                other => {
                    let module = stack.last().cloned().unwrap_or_default();
                    visit(at, &module, other);
                }
            }
        }
    }

    /// The text of the library root, with the probes in `include` and, under
    /// each, the lines `asserts` holds for it.
    pub(crate) fn render(
        &self,
        include: &[usize],
        asserts: &BTreeMap<usize, Vec<String>>,
    ) -> Rendered {
        let probes_of = |module: &str| -> Vec<usize> {
            include
                .iter()
                .copied()
                .filter(|at| self.probes[*at].module == module)
                .collect()
        };
        self.assemble(&|module, put| {
            for at in probes_of(module) {
                put(
                    &format!("use {} as probe{at};", self.probes[at].path),
                    Owner::Probe(at),
                );
                for (index, line) in asserts.get(&at).into_iter().flatten().enumerate() {
                    put(line, Owner::Assert(at, index));
                }
            }
        })
    }

    /// The text of the library root with the probe `probe` written as two
    /// separate questions in its module: a type alias of the path, which asks
    /// the type name space, and a function that reads the path as an
    /// expression, which asks the value name space. Neither is an import, and
    /// neither is evaluated.
    pub(crate) fn render_by_space(&self, probe: usize) -> SplitRendered {
        let path = &self.probes[probe].path;
        let type_text = format!("type __t = {path};");
        let value_text = format!("fn __v() {{ let _ = {path}; }}");
        let rendered = self.assemble(&|module, put| {
            if self.probes[probe].module == module {
                put(&type_text, Owner::Probe(probe));
                put(&value_text, Owner::Probe(probe));
            }
        });
        let line_of = |text: &str| {
            rendered
                .source
                .lines()
                .position(|line| line == text)
                .map(|at| at + 1)
        };
        SplitRendered {
            type_line: line_of(&type_text).expect("the type question is written"),
            value_line: line_of(&value_text).expect("the value question is written"),
            rendered,
        }
    }

    /// The lines of the program with `emit` writing what goes at the end of
    /// each module, whose canonical path it is given, and of the crate root.
    fn assemble(&self, emit: &dyn Emit) -> Rendered {
        let mut source = String::new();
        let mut owners = Vec::new();
        let mut put = |text: &str, owner: Owner| {
            source.push_str(text);
            source.push('\n');
            owners.push(owner);
        };
        let mut stack = vec![CRATE.to_owned()];
        for (at, text) in self.lines.iter().enumerate() {
            match read_line(text) {
                Line::Open(name) => {
                    let module = stack.last().cloned().unwrap_or_default();
                    stack.push(format!("{module}::{name}"));
                    put(text, Owner::Frame);
                }
                Line::Close => {
                    let module = stack.pop().unwrap_or_default();
                    emit(&module, &mut put);
                    put(text, Owner::Frame);
                }
                _ => put(text, Owner::Decl(at)),
            }
        }
        emit(CRATE, &mut put);
        Rendered { source, owners }
    }

    /// The items the program declares, with their tags.
    pub(crate) fn decls(&self) -> Vec<Decl> {
        let mut found = Vec::new();
        self.walk(|_, module, line| {
            if let Line::Item { name, space, tag } = line {
                found.push(Decl {
                    module: module.to_owned(),
                    name,
                    space,
                    tag,
                });
            }
        });
        found
    }

    /// The `use` declarations of the program.
    pub(crate) fn uses(&self) -> Vec<UseDecl> {
        let mut found = Vec::new();
        self.walk(|_, module, line| {
            if let Line::Use {
                vis,
                glob,
                binding,
                from,
            } = line
            {
                found.push(UseDecl {
                    module: module.to_owned(),
                    vis,
                    glob,
                    binding,
                    from,
                });
            }
        });
        found
    }

    /// The program with every declaration and module `pub` and the probe
    /// `probe` alone: nothing in it is hidden from any module.
    pub(crate) fn all_public(&self, probe: usize) -> Self {
        let lines = self
            .lines
            .iter()
            .map(|text| match read_line(text) {
                Line::Open(name) => format!("pub mod {name} {{"),
                Line::Close | Line::Other => text.clone(),
                Line::Item { .. } | Line::Use { .. } => {
                    format!("pub {}", split_vis(text.trim()).1)
                }
            })
            .collect();
        Self {
            lines,
            probes: vec![self.probes[probe].clone()],
            expect: None,
        }
    }

    /// The program with every module `pub` and the probe `probe` alone, so
    /// that a lint about re-exports, which `rustc` raises only for what the
    /// crate exports, can fire.
    pub(crate) fn with_public_modules(&self, probe: usize) -> Self {
        let lines = self
            .lines
            .iter()
            .map(|text| match read_line(text) {
                Line::Open(name) => format!("pub mod {name} {{"),
                _ => text.clone(),
            })
            .collect();
        Self {
            lines,
            probes: vec![self.probes[probe].clone()],
            expect: None,
        }
    }

    /// The modules of the program, as canonical paths, root first.
    fn modules(&self) -> Vec<String> {
        let mut found = vec![CRATE.to_owned()];
        self.walk(|_, module, line| {
            if let Line::Open(name) = line {
                found.push(format!("{module}::{name}"));
            }
        });
        found
    }

    /// The module a path written in `module` reads its names from: the path
    /// followed through `crate`, `self`, `super` and the modules of the
    /// program as far as it names modules, and no further, since a name after
    /// that may come from a glob.
    fn module_read_by(modules: &BTreeSet<String>, module: &str, path: &str) -> String {
        let mut at = module.to_owned();
        for (index, segment) in path.split("::").enumerate() {
            let next = match segment {
                "crate" if index == 0 => CRATE.to_owned(),
                "self" if index == 0 => at.clone(),
                "super" => at
                    .rsplit_once("::")
                    .map_or(at.clone(), |(up, _)| up.to_owned()),
                name => format!("{at}::{name}"),
            };
            if !modules.contains(&next) {
                break;
            }
            at = next;
        }
        at
    }

    /// Whether the imports of the program, with the probe `probe` taken as
    /// one more import of its module, form a cycle among the modules that the
    /// probe's module reaches. The graph holds an edge from a module to each
    /// module its `use` declarations read, from the probe's module to the
    /// module its path reads, and from each module to its child modules, since
    /// a glob or a path passes through the names of modules. It is built from
    /// the declarations, not from the text of the program, and over-approximates
    /// the cycles `rustc` cannot settle.
    pub(crate) fn import_cycle_reaches(&self, probe: usize) -> bool {
        let modules: BTreeSet<String> = self.modules().into_iter().collect();
        let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for module in &modules {
            if let Some((parent, _)) = module.rsplit_once("::") {
                edges
                    .entry(parent.to_owned())
                    .or_default()
                    .insert(module.clone());
            }
        }
        for decl in self.uses() {
            let read = Self::module_read_by(&modules, &decl.module, &decl.from);
            edges.entry(decl.module).or_default().insert(read);
        }
        let probed = &self.probes[probe];
        let from = probed
            .path
            .rsplit_once("::")
            .map_or("self", |(from, _)| from);
        let read = Self::module_read_by(&modules, &probed.module, from);
        edges.entry(probed.module.clone()).or_default().insert(read);
        // The modules reachable in one step or more from `start`.
        let reached = |start: &str| {
            let mut seen: BTreeSet<String> = BTreeSet::new();
            let mut todo: Vec<String> = edges.get(start).into_iter().flatten().cloned().collect();
            while let Some(next) = todo.pop() {
                if seen.insert(next.clone()) {
                    todo.extend(edges.get(&next).into_iter().flatten().cloned());
                }
            }
            seen
        };
        let from_reader = reached(&probed.module);
        from_reader
            .iter()
            .chain([&probed.module])
            .any(|module| reached(module).contains(module))
    }

    /// The index of the line that closes the module opened at `open`.
    fn close_of(&self, open: usize) -> usize {
        let mut depth = 0_usize;
        for (at, text) in self.lines.iter().enumerate().skip(open) {
            match read_line(text) {
                Line::Open(_) => depth += 1,
                Line::Close => {
                    depth -= 1;
                    if depth == 0 {
                        return at;
                    }
                }
                _ => {}
            }
        }
        self.lines.len() - 1
    }

    /// The program with the probe `probe` alone.
    pub(crate) fn only_probe(&self, probe: usize) -> Self {
        Self {
            lines: self.lines.clone(),
            probes: vec![self.probes[probe].clone()],
            expect: None,
        }
    }

    /// The program with the declarations of the crate root and of every
    /// module in an order drawn from `draw`, each module kept whole.
    pub(crate) fn shuffled(&self, draw: &mut Draw) -> Self {
        let (mut nodes, _) = Self::nodes(&self.lines, 0);
        Self::shuffle(&mut nodes, draw);
        let mut lines = Vec::new();
        Self::flatten(&nodes, &mut lines);
        Self {
            lines,
            probes: self.probes.clone(),
            expect: None,
        }
    }

    /// The declarations from `at` to the end or the close of the module
    /// they are in, as a list of siblings, and where the list ends.
    fn nodes(lines: &[String], mut at: usize) -> (Vec<Node>, usize) {
        let mut list = Vec::new();
        while at < lines.len() {
            match read_line(&lines[at]) {
                Line::Close => return (list, at),
                Line::Open(_) => {
                    let (inside, close) = Self::nodes(lines, at + 1);
                    list.push(Node::Module(
                        lines[at].clone(),
                        inside,
                        lines[close].clone(),
                    ));
                    at = close + 1;
                }
                _ => {
                    list.push(Node::Line(lines[at].clone()));
                    at += 1;
                }
            }
        }
        (list, at)
    }

    fn shuffle(nodes: &mut [Node], draw: &mut Draw) {
        for at in (1..nodes.len()).rev() {
            nodes.swap(at, draw.below(at + 1));
        }
        for node in nodes {
            if let Node::Module(_, inside, _) = node {
                Self::shuffle(inside, draw);
            }
        }
    }

    /// Write `nodes` to `lines`.
    fn flatten(nodes: &[Node], lines: &mut Vec<String>) {
        for node in nodes {
            match node {
                Node::Line(text) => lines.push(text.clone()),
                Node::Module(open, inside, close) => {
                    lines.push(open.clone());
                    Self::flatten(inside, lines);
                    lines.push(close.clone());
                }
            }
        }
    }

    /// The program without the lines `from..=to`.
    pub(crate) fn without(&self, from: usize, to: usize) -> Self {
        let mut lines = self.lines.clone();
        lines.drain(from..=to);
        Self {
            lines,
            probes: self.probes.clone(),
            expect: self.expect.clone(),
        }
    }

    /// The spans `(first, last)` of the lines a reduction may remove at
    /// once: each module with all it holds, then each declaration alone.
    pub(crate) fn removable(&self) -> Vec<(usize, usize)> {
        let mut spans = Vec::new();
        for (at, text) in self.lines.iter().enumerate() {
            match read_line(text) {
                Line::Open(_) => spans.push((at, self.close_of(at))),
                Line::Close => {}
                _ => spans.push((at, at)),
            }
        }
        spans
    }
}

/// The module paths of a program, relative to the crate root, root first.
type ModulePath = Vec<String>;

fn canonical(path: &[String]) -> String {
    let mut text = CRATE.to_owned();
    for segment in path {
        text.push_str("::");
        text.push_str(segment);
    }
    text
}

/// How a module is named from another one, written from `from`.
fn reach(draw: &mut Draw, from: &[String], to: &[String]) -> String {
    let parent = from.split_last().map(|(_, up)| up);
    if parent == Some(to) && draw.one_in(2) {
        return "super".to_owned();
    }
    if to.len() == from.len() + 1 && to.starts_with(from) && draw.one_in(2) {
        return format!("self::{}", to[to.len() - 1]);
    }
    let mut text = "crate".to_owned();
    for segment in to {
        text.push_str("::");
        text.push_str(segment);
    }
    text
}

fn draw_vis(draw: &mut Draw, private_in_ten: usize, public_in_ten: usize) -> Vis {
    let roll = draw.below(10);
    if roll < private_in_ten {
        Vis::Private
    } else if roll < private_in_ten + public_in_ten {
        Vis::Public
    } else {
        Vis::Crate
    }
}

/// The items a module declares: a name, whether it is a struct or a
/// function, its visibility and its tag.
type Items = Vec<(String, Space, Vis, u32)>;

/// Draw a program: library crates of two to four modules with up to two
/// children each, items of three shared names in all visibilities, private,
/// `pub` and `pub(crate)` globs, named imports, and renames that shadow.
pub(crate) fn random_program(draw: &mut Draw) -> Program {
    let tops = 2 + draw.below(3);
    let mut modules: Vec<ModulePath> = vec![Vec::new()];
    for top in 0..tops {
        let name = format!("m{top}");
        modules.push(vec![name.clone()]);
        for child in 0..draw.below(3) {
            modules.push(vec![name.clone(), format!("s{child}")]);
        }
    }
    let mut tag = 0_u32;
    let mut items: BTreeMap<ModulePath, Items> = BTreeMap::new();
    for module in &modules {
        let mut claimed: BTreeSet<(String, Space)> = BTreeSet::new();
        let mut made = Items::new();
        for _ in 0..draw.below(4) {
            let name = (*draw.pick(&NAMES)).to_owned();
            let vis = draw_vis(draw, 3, 4);
            let spaces: &[Space] = match draw.below(10) {
                0..=3 => &[Space::Type],
                4..=7 => &[Space::Value],
                _ => &[Space::Type, Space::Value],
            };
            for space in spaces {
                if claimed.insert((name.clone(), *space)) {
                    tag += 1;
                    made.push((name.clone(), *space, vis, tag));
                }
            }
        }
        items.insert(module.clone(), made);
    }
    let mut body: BTreeMap<ModulePath, Vec<String>> = BTreeMap::new();
    let mut globs: Vec<(ModulePath, ModulePath)> = Vec::new();
    for module in &modules {
        let mut lines: Vec<String> = Vec::new();
        let mut taken: BTreeSet<String> = BTreeSet::new();
        for (name, space, vis, tag) in &items[module] {
            taken.insert(name.clone());
            lines.push(match space {
                Space::Type => format!(
                    "{}struct {name} {{}} impl {name} {{ pub const TAG: u32 = {tag}; }}",
                    vis.written()
                ),
                Space::Value => format!("{}const fn {name}() -> u32 {{ {tag} }}", vis.written()),
            });
        }
        for _ in 0..=draw.below(4) {
            let target = draw.pick(&modules).clone();
            if &target == module {
                continue;
            }
            let how = reach(draw, module, &target);
            if draw.below(100) < 50 {
                let vis = draw_vis(draw, 4, 4);
                lines.push(format!("{}use {how}::*;", vis.written()));
                globs.push((module.clone(), target));
                continue;
            }
            let here: Vec<&String> = items[&target].iter().map(|item| &item.0).collect();
            let name = if here.is_empty() || draw.one_in(4) {
                (*draw.pick(&NAMES)).to_owned()
            } else {
                (*draw.pick(&here)).clone()
            };
            let binding = if draw.one_in(4) {
                (*draw.pick(&NAMES)).to_owned()
            } else {
                name.clone()
            };
            if taken.contains(&binding) {
                continue;
            }
            taken.insert(binding.clone());
            let vis = draw_vis(draw, 3, 4);
            let alias = if binding == name {
                String::new()
            } else {
                format!(" as {binding}")
            };
            lines.push(format!("{}use {how}::{name}{alias};", vis.written()));
        }
        body.insert(module.clone(), lines);
    }
    let mut lines = Vec::new();
    let top = nest(draw, &Vec::new(), &modules, &mut body);
    lines.extend(top);
    let mut probes: Vec<Probe> = Vec::new();
    let drawn = (0..3 + draw.below(4)).map(|_| random_probe(draw, &modules, &items));
    let bare = bare_probes(&lines, &modules, &items, &globs);
    for probe in drawn.chain(bare) {
        if !probes.contains(&probe) {
            probes.push(probe);
        }
    }
    Program {
        lines,
        probes,
        expect: None,
    }
}

/// Probes whose path starts with a bare name, as code and `use` paths of the
/// 2018 edition and later do: the name of a module the reading module holds
/// or that the globs of the reading module bring in, through the globs the
/// program writes and the globs of the modules they reach. They are drawn
/// from a generator seeded by the text of the program, so that the draws
/// that make the program itself are the same with and without them.
fn bare_probes(
    lines: &[String],
    modules: &[ModulePath],
    items: &BTreeMap<ModulePath, Items>,
    globs: &[(ModulePath, ModulePath)],
) -> Vec<Probe> {
    // FNV-1a over the lines.
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in lines.iter().flat_map(|line| line.bytes().chain(*b"\n")) {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    let mut draw = Draw::seeded(hash);
    let children = |module: &ModulePath| -> Vec<ModulePath> {
        modules
            .iter()
            .filter(|child| child.len() == module.len() + 1 && child.starts_with(module))
            .cloned()
            .collect()
    };
    let mut candidates: Vec<(ModulePath, ModulePath)> = Vec::new();
    for reader in modules {
        // The modules whose globs the reader reaches, itself first.
        let mut reached = vec![reader.clone()];
        let mut at = 0;
        while at < reached.len() {
            for (from, to) in globs {
                if from == &reached[at] && !reached.contains(to) {
                    reached.push(to.clone());
                }
            }
            at += 1;
        }
        for module in &reached {
            for child in children(module) {
                candidates.push((reader.clone(), child));
            }
        }
    }
    let mut found = Vec::new();
    if candidates.is_empty() {
        return found;
    }
    for _ in 0..=draw.below(2) {
        let (reader, target) = draw.pick(&candidates).clone();
        let here: Vec<&String> = items[&target].iter().map(|item| &item.0).collect();
        let name = if here.is_empty() || draw.one_in(4) {
            (*draw.pick(&NAMES)).to_owned()
        } else {
            (*draw.pick(&here)).clone()
        };
        let probe = Probe {
            module: canonical(&reader),
            path: format!("{}::{name}", target[target.len() - 1]),
        };
        if !found.contains(&probe) {
            found.push(probe);
        }
    }
    found
}

/// The lines of the module at `path` with its children nested, in an order
/// drawn.
fn nest(
    draw: &mut Draw,
    path: &ModulePath,
    modules: &[ModulePath],
    body: &mut BTreeMap<ModulePath, Vec<String>>,
) -> Vec<String> {
    let mut units: Vec<Vec<String>> = body
        .remove(path)
        .unwrap_or_default()
        .into_iter()
        .map(|line| vec![line])
        .collect();
    let children: Vec<&ModulePath> = modules
        .iter()
        .filter(|module| module.len() == path.len() + 1 && module.starts_with(path))
        .collect();
    for child in children {
        let name = child[child.len() - 1].clone();
        let vis = if draw.one_in(2) { "pub " } else { "" };
        let mut unit = vec![format!("{vis}mod {name} {{")];
        unit.extend(nest(draw, child, modules, body));
        unit.push("}".to_owned());
        units.push(unit);
    }
    for at in (1..units.len()).rev() {
        let other = draw.below(at + 1);
        units.swap(at, other);
    }
    units.into_iter().flatten().collect()
}

fn random_probe(
    draw: &mut Draw,
    modules: &[ModulePath],
    items: &BTreeMap<ModulePath, Items>,
) -> Probe {
    let reader = draw.pick(modules).clone();
    let target = draw.pick(modules).clone();
    let here: Vec<&String> = items[&target].iter().map(|item| &item.0).collect();
    let name = if here.is_empty() || draw.one_in(4) {
        (*draw.pick(&NAMES)).to_owned()
    } else {
        (*draw.pick(&here)).clone()
    };
    let path = match draw.below(10) {
        0..=4 => format!("{}::{name}", reach(draw, &reader, &target)),
        5..=7 => {
            let through = draw.pick(modules).clone();
            let last = draw.pick(modules).clone();
            let mut text = "crate".to_owned();
            for segment in through.iter().chain(last.last()) {
                text.push_str("::");
                text.push_str(segment);
            }
            format!("{text}::{name}")
        }
        _ => format!("self::{name}"),
    };
    Probe {
        module: canonical(&reader),
        path,
    }
}
