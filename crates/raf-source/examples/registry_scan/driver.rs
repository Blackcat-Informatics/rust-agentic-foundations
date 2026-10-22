// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Load the newest version of every crate in a cargo registry source directory
//! and report, for each, whether it loads and whether every item sits under a
//! module path its file was reached by.
//!
//! ```text
//! cargo run --release -p raf-source --example registry_scan -- \
//!     ~/.cargo/registry/src/<index> [--jobs N] [--max-work N] [--max-memory BYTES]
//! ```
//!
//! With no arguments, a small in-memory package demonstrates the same loader,
//! module-placement check and result formatter without a local registry.
//!
//! Each crate directory holds one package, named `<name>-<version>`. Of each
//! name only the highest version is loaded, with [`DiskTree`] rooted at the
//! crate directory and `"."` as the one root. Each load runs in a child
//! process, this example started again with `--one <dir> --max-work N`, so a
//! crate that aborts ends its own line and not the scan. The
//! child is started through `sh -c 'ulimit -v <KiB>; exec "$0" "$@"'`, which
//! limits its address space to `--max-memory` (8 GiB unless given; a number
//! of bytes, or with a `K`, `M` or `G` suffix for binary multiples). The limit
//! is per child, so the scan as a whole may take `--jobs` × `--max-memory`:
//! the scan reads the memory available on the machine (`MemAvailable` in
//! `/proc/meminfo`), runs as many children as fit in it (and no more than the
//! processors) unless `--jobs` says, and refuses a `--jobs` whose product with
//! `--max-memory` passes it, and a `--max-memory` that one child cannot fit.
//!
//! `--max-work N` (a whole number of at least 1; the limit of
//! [`Budget::Work`](raf_source::Budget::Work) unless given) is the most glob
//! table work steps one load may make. A crate that passes it is refused like
//! any other budget, with `ERR Budget`, and the scan goes on. No clock is read
//! and no child is stopped: the bound is a count. A child that spins in code
//! the work counter does not see is not stopped; parsing, the one such stretch
//! the index has, is bounded by the token budget.
//!
//! The counter also does not see the lookups of settling: a module with n globs
//! and n named imports of a name the globs re-export makes a work count of 4n+2
//! and visits about 25n^2 glob reads, so a crate of that shape is not refused by
//! `--max-work`. Such a load can run long.
//!
//! There is no `--timeout` flag and no `timeout` outcome: a bound by time
//! needs a clock and a child stopped from outside, and a count needs neither,
//! so the bound on a load is the count of work steps.
//!
//! One line is printed per crate, sorted by name:
//!
//! ```text
//! name@version | OK items=N files=N misplaced=N total=N item=N fact=N glob=N tokens=N recursion=N depth=N work=N [first misplaced item]
//! name@version | ERR <kind> :: <message>
//! ```
//!
//! `total`, `item`, `fact`, `glob`, `tokens`, `recursion`, `depth` and `work`
//! are the load's [`Usage`](raf_source::Usage): the bytes read, charged to the
//! index and to the facts, the most charged to the glob tables at once, the
//! most tokens of one file, the highest value of the recursion counter in one
//! file, the deepest directory entered and the glob table work steps made.
//! The kind is the [`LoadErrorKind`](raf_source::LoadErrorKind) variant, or
//! `memory` for a child that ran out of its address space and `abort` for a
//! child that ended without a result otherwise. A closing `#` block counts the outcomes and gives the largest
//! value of each usage figure, with the crate that reached it.
//!
//! An item is misplaced when the module part of its path is not a module path
//! of its file: one of the module paths the file was reached by in the item's
//! target, or the path of an inline module the file defines. The module part
//! is the path without its last segment, and without its last two for a
//! member of a trait. A `macro_rules!` macro marked `#[macro_export]` may also
//! sit at its crate root, where the attribute puts it; no other item may.
//! Members of impl blocks are recorded under their self type wherever it is
//! defined, so they are not judged.
//!
//! A complete registry scan needs a local registry. The no-argument example
//! checks only its built-in package; it does not exercise child orchestration
//! or establish that arbitrary registry packages load.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;

use raf_source::{
    Budget, DiskTree, Item, LoadOptions, MemoryTree, SymbolKind, Usage, Workspace, syn,
};

/// The prefix of the one line a child prints.
const RESULT: &str = "result: ";

/// The address space of one child, unless `--max-memory` says: 8 GiB.
const DEFAULT_MAX_MEMORY: u64 = 8 << 30;

/// What the runtime prints when an allocation fails.
const ALLOCATION_FAILED: &str = "memory allocation of";

/// The usage figures of an `OK` line, in the order they are printed.
const USAGE: [&str; 8] = [
    "total",
    "item",
    "fact",
    "glob",
    "tokens",
    "recursion",
    "depth",
    "work",
];

pub(super) fn run() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match parse_one(&args) {
        Some((dir, max_work)) => {
            println!("{RESULT}{}", load_one(Path::new(dir), max_work));
            Ok(())
        }
        None if args.is_empty() => sample(),
        None => parse_args(&args).and_then(|options| scan(&options)),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("registry_scan: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Load a written package and demonstrate the registry report's core checks.
fn sample() -> Result<(), Box<dyn Error>> {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='sample'\nversion='0.1.0'\nedition='2024'\n",
        ),
        (
            "src/lib.rs",
            "pub mod inner;\npub trait Named { fn name(&self) -> &str; }\npub struct Entry;\nimpl Named for Entry { fn name(&self) -> &str { inner::name() } }\n",
        ),
        (
            "src/inner.rs",
            "pub fn name() -> &'static str { \"sample\" }\n",
        ),
    ]);
    let workspace = Workspace::load(&tree, &["."])?;
    let misplaced = misplaced(&workspace);
    if !misplaced.is_empty() {
        return Err(format!("built-in package has misplaced items: {misplaced:?}").into());
    }
    println!(
        "sample@0.1.0 | {}",
        ok_line(
            workspace.items().count(),
            workspace.files().count(),
            &workspace.usage(),
            &misplaced,
        )
    );
    Ok(())
}

/// The directory and work limit of a child's arguments, `--one <dir>
/// --max-work <N>`, which the scan builds and never a person.
pub(super) fn parse_one(args: &[String]) -> Option<(&str, usize)> {
    match args {
        [one, dir, flag, limit] if one == "--one" && flag == "--max-work" => {
            Some((dir, unsigned(limit)?))
        }
        _ => None,
    }
}

/// What the scan was asked to do.
pub(super) struct Options {
    registry: PathBuf,
    pub(super) jobs: usize,
    max_work: usize,
    max_memory: u64,
}

/// The example's unsigned decimal subset, including `+` and leading zeroes,
/// narrowed through the native exact integer reader to the requested width.
fn unsigned<T: TryFrom<u128>>(text: &str) -> Option<T> {
    let digits = text.strip_prefix('+').unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let significant = digits.trim_start_matches('0');
    let significant = if significant.is_empty() {
        "0"
    } else {
        significant
    };
    if significant.len() > 39 {
        return None;
    }
    let value = purrdf_xsd::numeric::parse_integer_typed(
        significant,
        purrdf_xsd::XsdDatatype::NonNegativeInteger,
    )
    .ok()?
    .into_bigint()
    .unsigned_abs_u128()?;
    T::try_from(value).ok()
}

/// A whole number of at least 1: the work steps of one load.
pub(super) fn parse_work(text: &str) -> Result<usize, String> {
    let value: usize = unsigned(text).ok_or_else(|| {
        format!("--max-work `{text}` must be an unsigned whole number that fits usize")
    })?;
    if value == 0 {
        return Err("--max-work must be at least 1".to_owned());
    }
    Ok(value)
}

/// A number of bytes, with an optional `K`, `M` or `G` suffix.
pub(super) fn parse_bytes(text: &str) -> Result<u64, String> {
    let (digits, shift) = match text.as_bytes().last() {
        Some(b'K' | b'k') => (&text[..text.len() - 1], 10),
        Some(b'M' | b'm') => (&text[..text.len() - 1], 20),
        Some(b'G' | b'g') => (&text[..text.len() - 1], 30),
        _ => (text, 0),
    };
    let value: u64 = unsigned(digits).ok_or_else(|| {
        format!("--max-memory `{text}` must be an unsigned whole number that fits u64")
    })?;
    value
        .checked_mul(1 << shift)
        .filter(|bytes| *bytes >= 1 << 20)
        .ok_or_else(|| format!("--max-memory `{text}` must be at least 1M and fit in 64 bits"))
}

pub(super) fn parse_args(args: &[String]) -> Result<Options, Box<dyn Error>> {
    parse_args_with_memory(args, machine_memory)
}

pub(super) fn parse_args_with_memory(
    args: &[String],
    memory: impl FnOnce() -> Result<u64, Box<dyn Error>>,
) -> Result<Options, Box<dyn Error>> {
    let usage =
        "usage: registry_scan <registry-src-dir> [--jobs N] [--max-work N] [--max-memory BYTES]";
    let mut registry = None;
    let mut jobs = None;
    let mut max_work = Budget::Work.limit();
    let mut max_memory = DEFAULT_MAX_MEMORY;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--jobs" => {
                let value = rest.next().ok_or(usage)?;
                let count: usize = unsigned(value).ok_or_else(|| {
                    format!("--jobs `{value}` must be an unsigned whole number that fits usize")
                })?;
                if count == 0 {
                    return Err("--jobs must be at least 1".into());
                }
                jobs = Some(count);
            }
            "--max-work" => {
                let value = rest.next().ok_or(usage)?;
                max_work = parse_work(value)?;
            }
            "--max-memory" => {
                let value = rest.next().ok_or(usage)?;
                max_memory = parse_bytes(value)?;
            }
            flag if flag.starts_with("--") => {
                return Err(format!("{usage}\nunknown `{flag}`").into());
            }
            dir if registry.is_none() => registry = Some(PathBuf::from(dir)),
            extra => return Err(format!("{usage}\nunexpected `{extra}`").into()),
        }
    }
    let available = memory()?;
    let fit = usize::try_from(available / max_memory).unwrap_or(usize::MAX);
    if fit == 0 {
        return Err(format!(
            "--max-memory {max_memory} bytes passes the {available} bytes available on the machine"
        )
        .into());
    }
    let jobs = match jobs {
        Some(count) if count > fit => {
            return Err(format!(
                "--jobs {count} × --max-memory {max_memory} bytes passes the {available} bytes available on the machine, which fit {fit}"
            )
            .into());
        }
        Some(count) => count,
        None => {
            let processors = thread::available_parallelism().map_or(1, usize::from);
            processors.min(fit)
        }
    };
    Ok(Options {
        registry: registry.ok_or(usage)?,
        jobs,
        max_work,
        max_memory,
    })
}

/// The memory available on the machine in bytes, from `MemAvailable` in
/// `/proc/meminfo`, so other work on a shared host is left its share.
fn machine_memory() -> Result<u64, Box<dyn Error>> {
    let info = std::fs::read_to_string("/proc/meminfo")
        .map_err(|e| format!("cannot read /proc/meminfo for the memory of the machine: {e}"))?;
    let kib: u64 = info
        .lines()
        .find_map(|line| line.strip_prefix("MemAvailable:"))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(unsigned)
        .ok_or("/proc/meminfo has no readable MemAvailable line")?;
    Ok(kib * 1024)
}

// --- choosing the crates ------------------------------------------------------

/// A version, ordered as semantic versioning orders precedence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Version {
    pub(super) core: [u64; 3],
    pre: Vec<String>,
}

impl Version {
    /// The version written as `major.minor.patch[-pre][+build]`.
    pub(super) fn parse(text: &str) -> Option<Self> {
        let text = text.split_once('+').map_or(text, |(version, _)| version);
        let (core, pre) = text
            .split_once('-')
            .map_or((text, None), |(core, pre)| (core, Some(pre)));
        let mut numbers = core.split('.');
        let mut part = || -> Option<u64> {
            let digits = numbers.next()?;
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            unsigned(digits)
        };
        let parsed = [part()?, part()?, part()?];
        if numbers.next().is_some() {
            return None;
        }
        let pre = match pre {
            Some("") => return None,
            Some(pre) => pre.split('.').map(str::to_owned).collect(),
            None => Vec::new(),
        };
        Some(Self { core: parsed, pre })
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        self.core.cmp(&other.core).then_with(|| {
            match (self.pre.is_empty(), other.pre.is_empty()) {
                (true, true) => Ordering::Equal,
                // A release is above every pre-release of its core.
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => {
                    for (a, b) in self.pre.iter().zip(&other.pre) {
                        let order = match (unsigned::<u64>(a), unsigned::<u64>(b)) {
                            (Some(a), Some(b)) => a.cmp(&b),
                            (Some(_), None) => Ordering::Less,
                            (None, Some(_)) => Ordering::Greater,
                            (None, None) => a.cmp(b),
                        };
                        if order != Ordering::Equal {
                            return order;
                        }
                    }
                    self.pre.len().cmp(&other.pre.len())
                }
            }
        })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A directory name split into crate name and version: the first `-` after
/// which a version follows. A crate name never holds a `.`, so the split is
/// the only one.
fn split_name(dir: &str) -> Option<(&str, Version)> {
    dir.match_indices('-').find_map(|(at, _)| {
        let (name, version) = (&dir[..at], &dir[at + 1..]);
        (!name.is_empty())
            .then(|| Version::parse(version))
            .flatten()
            .map(|version| (name, version))
    })
}

/// One crate to load.
struct Crate {
    label: String,
    dir: PathBuf,
}

/// The newest version of each crate under `registry`, sorted by name.
fn newest(registry: &Path) -> Result<Vec<Crate>, Box<dyn Error>> {
    let mut best: BTreeMap<String, (Version, String)> = BTreeMap::new();
    let entries = std::fs::read_dir(registry)
        .map_err(|e| format!("cannot list {}: {e}", registry.display()))?;
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let file_name = entry.file_name();
        let dir = file_name
            .to_str()
            .ok_or_else(|| format!("{} is not UTF-8", entry.path().display()))?;
        let (name, version) =
            split_name(dir).ok_or_else(|| format!("`{dir}` is not `<name>-<version>`"))?;
        let newer = best.get(name).is_none_or(|(known, _)| version > *known);
        if newer {
            best.insert(name.to_owned(), (version, dir.to_owned()));
        }
    }
    if best.is_empty() {
        return Err(format!("{} holds no crate directory", registry.display()).into());
    }
    Ok(best
        .into_iter()
        .map(|(name, (_, dir))| {
            let version = &dir[name.len() + 1..];
            Crate {
                label: format!("{name}@{version}"),
                dir: registry.join(&dir),
            }
        })
        .collect())
}

// --- one crate, in the child --------------------------------------------------

/// Load one crate and describe the outcome on one line.
pub(super) fn load_one(dir: &Path, max_work: usize) -> String {
    let tree = match DiskTree::new(dir) {
        Ok(tree) => tree,
        Err(error) => return format!("ERR tree :: {}", one_line(&error.to_string())),
    };
    let mut options = LoadOptions::default();
    options.max_work = max_work;
    match Workspace::load_with(&tree, &["."], &options) {
        Ok(ws) => {
            let misplaced = misplaced(&ws);
            ok_line(
                ws.items().count(),
                ws.files().count(),
                &ws.usage(),
                &misplaced,
            )
        }
        Err(error) => format!(
            "ERR {} :: {}",
            variant(&format!("{:?}", error.kind)),
            one_line(&error.to_string())
        ),
    }
}

/// The `OK` line of a loaded crate: its counts, the usage figures in the order
/// of [`USAGE`] and the first misplaced item, if any.
pub(super) fn ok_line(items: usize, files: usize, usage: &Usage, misplaced: &[String]) -> String {
    let mut line = format!(
        "OK items={items} files={files} misplaced={} total={} item={} fact={} glob={} tokens={} recursion={} depth={} work={}",
        misplaced.len(),
        usage.total_bytes,
        usage.item_bytes,
        usage.fact_bytes,
        usage.glob_bytes,
        usage.max_file_tokens,
        usage.max_recursion,
        usage.max_directory_depth,
        usage.work,
    );
    if let Some(first) = misplaced.first() {
        let _ = write!(line, " {first}");
    }
    line
}

/// The variant name at the start of a `Debug` rendering.
fn variant(debug: &str) -> &str {
    let end = debug
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(debug.len());
    &debug[..end]
}

fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// True for a member of a trait or of an impl block.
fn is_member(kind: SymbolKind) -> bool {
    matches!(
        kind,
        SymbolKind::InherentMethod
            | SymbolKind::TraitImplMethod
            | SymbolKind::TraitDefaultMethod
            | SymbolKind::TraitRequiredMethod
            | SymbolKind::AssocConst
            | SymbolKind::AssocType
    )
}

/// `path` without its last `drop` segments, or `None` when it has no more.
fn module_part(path: &str, drop: usize) -> Option<&str> {
    let mut module = path;
    for _ in 0..drop {
        module = module.rsplit_once("::")?.0;
    }
    Some(module)
}

/// The module paths of each file in each target: the paths the file was
/// reached by and the inline modules it defines under them.
fn module_paths(ws: &Workspace) -> BTreeSet<(String, usize, String)> {
    let mut paths = BTreeSet::new();
    for file in ws.files() {
        for reach in &file.reached {
            paths.insert((
                file.path.to_string(),
                reach.target.index(),
                reach.module_path.to_string(),
            ));
        }
    }
    for item in ws.items() {
        if item.kind == SymbolKind::Mod && item.body.is_some() {
            paths.insert((
                item.file.to_string(),
                item.target.index(),
                item.path.to_string(),
            ));
        }
    }
    paths
}

/// True when the `macro_rules!` macro `item` is marked `#[macro_export]`:
/// its text, which its span takes from its first attribute, parses as a macro
/// item with that attribute.
fn is_exported_macro(ws: &Workspace, item: &Item) -> bool {
    ws.source(&item.file)
        .and_then(|text| text.get(item.span.start_byte..item.span.end_byte))
        .and_then(|written| syn::parse_str::<syn::ItemMacro>(written).ok())
        .is_some_and(|parsed| {
            parsed
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("macro_export"))
        })
}

/// The misplaced items of a loaded crate, as `path in file`, sorted.
fn misplaced(ws: &Workspace) -> Vec<String> {
    let modules = module_paths(ws);
    let is_module = |item: &Item, path: &str| {
        modules.contains(&(item.file.to_string(), item.target.index(), path.to_owned()))
    };
    let mut out = Vec::new();
    for item in ws.items() {
        if item.self_type.is_some() {
            continue;
        }
        let crate_path = ws
            .target(item.target)
            .map(|target| target.crate_path.as_str());
        // A crate root item is the module of its target's root file.
        if item.kind == SymbolKind::Mod && crate_path == Some(item.path.as_str()) {
            continue;
        }
        let drop = if is_member(item.kind) { 2 } else { 1 };
        let placed = module_part(&item.path, drop).is_some_and(|module| {
            is_module(item, module)
                || (item.kind == SymbolKind::MacroRules
                    && crate_path == Some(module)
                    && is_exported_macro(ws, item))
        });
        if !placed {
            out.push(format!("{} in {}", item.path, item.file));
        }
    }
    out.sort();
    out
}

// --- the scan, in the parent --------------------------------------------------

/// The arguments the parent gives a child: `--one <dir> --max-work <N>`, the
/// shape [`parse_one`] reads.
pub(super) fn child_args(dir: &Path, max_work: usize) -> Vec<std::ffi::OsString> {
    vec![
        "--one".into(),
        dir.as_os_str().to_owned(),
        "--max-work".into(),
        max_work.to_string().into(),
    ]
}

/// Run one crate in a child process, under an address-space limit of
/// `max_memory` bytes and a work limit of `max_work`, and give its line.
fn run_child(exe: &Path, krate: &Crate, max_work: usize, max_memory: u64) -> String {
    let spawned = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "ulimit -v {}; exec \"$0\" \"$@\"",
            max_memory / 1024
        ))
        .arg(exe)
        .args(child_args(&krate.dir, max_work))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => return format!("ERR spawn :: {error}"),
    };
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let status = match child.wait() {
        Ok(status) => status,
        Err(error) => return format!("ERR wait :: {error}"),
    };
    let stdout = stdout.join().unwrap_or_default();
    let stderr = stderr.join().unwrap_or_default();
    match stdout.lines().find_map(|line| line.strip_prefix(RESULT)) {
        Some(line) if status.success() => line.to_owned(),
        _ => {
            let last = stderr
                .lines()
                .rev()
                .find(|line| !line.trim().is_empty())
                .unwrap_or("no output");
            let kind = if stderr.contains(ALLOCATION_FAILED) {
                "memory"
            } else {
                "abort"
            };
            format!("ERR {kind} :: {status}: {}", one_line(last))
        }
    }
}

/// Read a pipe to its end on a thread of its own.
fn drain<R: std::io::Read + Send + 'static>(pipe: Option<R>) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut pipe) = pipe {
            let mut bytes = Vec::new();
            if pipe.read_to_end(&mut bytes).is_ok() {
                text = String::from_utf8_lossy(&bytes).into_owned();
            }
        }
        text
    })
}

/// The counts of a finished scan.
#[derive(Default)]
pub(super) struct Tally {
    pub(super) ok: usize,
    pub(super) with_misplaced: usize,
    pub(super) misplaced_items: usize,
    pub(super) errors: BTreeMap<String, usize>,
    /// The largest value of each usage figure and the crate that reached it.
    pub(super) largest: BTreeMap<&'static str, (u64, String)>,
}

impl Tally {
    pub(super) fn count(&mut self, label: &str, line: &str) {
        if let Some(rest) = line.strip_prefix("OK ") {
            self.ok += 1;
            for field in rest.split_whitespace() {
                let Some((key, value)) = field.split_once('=') else {
                    continue;
                };
                let (Some(key), Some(value)) = (USAGE.iter().find(|k| **k == key), unsigned(value))
                else {
                    continue;
                };
                let best = self.largest.entry(key).or_insert((0, String::new()));
                if value > best.0 {
                    *best = (value, label.to_owned());
                }
            }
            let misplaced = rest
                .split_whitespace()
                .find_map(|field| field.strip_prefix("misplaced="))
                .and_then(unsigned::<usize>)
                .unwrap_or(0);
            if misplaced > 0 {
                self.with_misplaced += 1;
                self.misplaced_items += misplaced;
            }
        } else {
            let kind = line
                .strip_prefix("ERR ")
                .and_then(|rest| rest.split_whitespace().next())
                .unwrap_or("unknown");
            *self.errors.entry(kind.to_owned()).or_default() += 1;
        }
    }
}

fn scan(options: &Options) -> Result<(), Box<dyn Error>> {
    let crates = newest(&options.registry)?;
    let exe = std::env::current_exe()?;
    let next = AtomicUsize::new(0);
    let (send, receive) = mpsc::channel::<(usize, String)>();
    let mut tally = Tally::default();
    thread::scope(|scope| {
        for _ in 0..options.jobs.min(crates.len()) {
            let send = send.clone();
            let (crates, next, exe) = (&crates, &next, &exe);
            scope.spawn(move || {
                loop {
                    let at = next.fetch_add(1, Ordering::Relaxed);
                    let Some(krate) = crates.get(at) else { break };
                    let line = run_child(exe, krate, options.max_work, options.max_memory);
                    if send.send((at, line)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(send);
        // Print in name order as soon as every earlier crate is done.
        let mut done: BTreeMap<usize, String> = BTreeMap::new();
        let mut printed = 0;
        for (at, line) in receive {
            done.insert(at, line);
            while let Some(line) = done.remove(&printed) {
                println!("{} | {line}", crates[printed].label);
                tally.count(&crates[printed].label, &line);
                printed += 1;
            }
        }
    });
    println!("# crates={}", crates.len());
    println!("# ok={}", tally.ok);
    println!("# ok-with-misplaced={}", tally.with_misplaced);
    println!("# misplaced-items={}", tally.misplaced_items);
    for (kind, count) in &tally.errors {
        println!("# err {kind}={count}");
    }
    for key in USAGE {
        match tally.largest.get(key) {
            Some((value, label)) if *value > 0 => println!("# largest {key}={value} ({label})"),
            _ => println!("# largest {key}=0"),
        }
    }
    Ok(())
}
