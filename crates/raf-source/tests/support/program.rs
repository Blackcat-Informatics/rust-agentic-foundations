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
        if rest.ends_with("::*") {
            return Line::Use {
                vis,
                glob: true,
                binding: None,
            };
        }
        let binding = rest
            .rsplit_once(" as ")
            .map_or_else(|| rest.rsplit("::").next(), |(_, alias)| Some(alias));
        return Line::Use {
            vis,
            glob: false,
            binding: binding.map(str::to_owned),
        };
    }
    Line::Other
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
        let mut source = String::new();
        let mut owners = Vec::new();
        let mut put = |text: &str, owner: Owner| {
            source.push_str(text);
            source.push('\n');
            owners.push(owner);
        };
        let probes_of = |module: &str| -> Vec<usize> {
            include
                .iter()
                .copied()
                .filter(|at| self.probes[*at].module == module)
                .collect()
        };
        let emit = |module: &str, put: &mut dyn FnMut(&str, Owner)| {
            for at in probes_of(module) {
                put(
                    &format!("use {} as probe{at};", self.probes[at].path),
                    Owner::Probe(at),
                );
                for (index, line) in asserts.get(&at).into_iter().flatten().enumerate() {
                    put(line, Owner::Assert(at, index));
                }
            }
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
            if let Line::Use { vis, glob, binding } = line {
                found.push(UseDecl {
                    module: module.to_owned(),
                    vis,
                    glob,
                    binding,
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
    let probes = (0..3 + draw.below(4))
        .map(|_| random_probe(draw, &modules, &items))
        .fold(Vec::<Probe>::new(), |mut all, probe| {
            if !all.contains(&probe) {
                all.push(probe);
            }
            all
        });
    Program {
        lines,
        probes,
        expect: None,
    }
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
