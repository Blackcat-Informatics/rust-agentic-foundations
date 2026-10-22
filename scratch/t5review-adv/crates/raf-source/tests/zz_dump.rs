// Review tool: dump library resolution answers of a workspace, for comparing two resolvers.
#![allow(missing_docs, clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use raf_source::{
    DiskTree, ItemId, MemoryTree, Namespace, Resolution, ResolveError, SymbolKind, TargetKind,
    Workspace,
};

fn site(ws: &Workspace, id: ItemId) -> String {
    match ws.item(id) {
        Some(i) => format!("{}@{}:{}", i.path, i.file, i.span.start_byte),
        None => format!("?{}", id.index()),
    }
}

fn show(ws: &Workspace, r: &Result<Resolution, ResolveError>) -> String {
    match r {
        Ok(res) => {
            let sites: Vec<String> = res.sites.iter().map(|id| site(ws, *id)).collect();
            format!(
                "OK {} {:?} glob={} chain={:?} sites={:?}",
                res.path, res.namespace, res.via_glob, res.chain, sites
            )
        }
        Err(ResolveError::Ambiguous(ids)) => {
            let sites: Vec<String> = ids.iter().map(|id| site(ws, *id)).collect();
            format!("AMB {sites:?}")
        }
        Err(e) => format!("ERR {e:?}"),
    }
}

fn dump(ws: &Workspace, out: &mut String) {
    let lib_targets: BTreeSet<_> = ws
        .target_ids()
        .filter(|t| ws.target(*t).is_some_and(|t| t.kind == TargetKind::Lib))
        .collect();
    let items: Vec<_> = ws
        .items()
        .filter(|i| lib_targets.contains(&i.target))
        .collect();
    let mut paths: BTreeSet<String> = BTreeSet::new();
    let mut modules: BTreeSet<String> = BTreeSet::new();
    let mut names_by_crate: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut children: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut roots: BTreeSet<String> = BTreeSet::new();
    for i in &items {
        let p = i.path.to_string();
        paths.insert(p.clone());
        let krate = p.split("::").next().unwrap_or("").to_owned();
        if i.kind == SymbolKind::Mod {
            modules.insert(p.clone());
            if !p.contains("::") {
                roots.insert(p.clone());
            }
        }
        names_by_crate
            .entry(krate)
            .or_default()
            .insert(i.name.to_string());
        if let Some((parent, name)) = p.rsplit_once("::") {
            children
                .entry(parent.to_owned())
                .or_default()
                .insert(name.to_owned());
        }
    }
    for p in &paths {
        writeln!(out, "R {p} => {}", show(ws, &ws.resolve(p))).unwrap();
        for ns in Namespace::ALL {
            writeln!(out, "RI {p} {ns:?} => {}", show(ws, &ws.resolve_in(p, ns))).unwrap();
        }
    }
    for m in &modules {
        let krate = m.split("::").next().unwrap_or("");
        for n in names_by_crate.get(krate).into_iter().flatten() {
            writeln!(out, "F {m} {n} => {}", show(ws, &ws.resolve_from(m, n))).unwrap();
        }
        for n in children.get(m).into_iter().flatten() {
            let q = format!("self::{n}");
            for ns in Namespace::ALL {
                writeln!(
                    out,
                    "FI {m} {q} {ns:?} => {}",
                    show(ws, &ws.resolve_from_in(m, &q, ns))
                )
                .unwrap();
            }
            let child = format!("{m}::{n}");
            if modules.contains(&child) {
                for n2 in children.get(m).into_iter().flatten() {
                    let q = format!("super::{n2}");
                    writeln!(out, "F {child} {q} => {}", show(ws, &ws.resolve_from(&child, &q)))
                        .unwrap();
                }
            }
        }
    }
    for r in &roots {
        for p in &paths {
            writeln!(out, "F {r} {p} => {}", show(ws, &ws.resolve_from(r, p))).unwrap();
            if let Some(rest) = p.strip_prefix(&format!("{r}::")) {
                let q = format!("crate::{rest}");
                writeln!(out, "F {r} {q} => {}", show(ws, &ws.resolve_from(r, &q))).unwrap();
            }
        }
    }
}

#[test]
#[ignore = "review dump"]
fn zz_dump() {
    let out_path = std::env::var("RAF_DUMP_OUT").expect("RAF_DUMP_OUT");
    let mut out = String::new();
    if let Ok(corpus) = std::env::var("RAF_DUMP_CORPUS") {
        let mut names: Vec<_> = fs::read_dir(&corpus)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        let manifest = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
        for name in names {
            let text = fs::read_to_string(Path::new(&corpus).join(&name)).unwrap();
            let tree: MemoryTree = [("Cargo.toml", manifest), ("src/lib.rs", text.as_str())]
                .into_iter()
                .collect();
            writeln!(out, "### {name}").unwrap();
            match Workspace::load(&tree, &["."]) {
                Ok(ws) => dump(&ws, &mut out),
                Err(e) => writeln!(out, "LOAD ERR {e}").unwrap(),
            }
        }
    }
    if let Ok(root) = std::env::var("RAF_DUMP_ROOT") {
        let tree = DiskTree::new(&root).unwrap();
        let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"));
        writeln!(out, "### repo").unwrap();
        dump(&ws, &mut out);
    }
    fs::write(out_path, out).unwrap();
}
