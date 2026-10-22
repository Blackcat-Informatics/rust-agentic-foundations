// Review probe: compare Workspace::impls() with an independent syn walk of every file.
#![allow(missing_docs, clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use raf_source::syn::{self, visit::Visit};
use raf_source::{DiskTree, Workspace};

#[derive(Default)]
struct Nested {
    // (line, where)
    found: Vec<(usize, &'static str)>,
    ctx: Vec<&'static str>,
}

impl<'a> Visit<'a> for Nested {
    fn visit_item_impl(&mut self, i: &'a syn::ItemImpl) {
        if !self.ctx.is_empty() {
            self.found
                .push((i.impl_token.span.start().line, self.ctx.last().copied().unwrap()));
        }
        syn::visit::visit_item_impl(self, i);
    }
    fn visit_block(&mut self, b: &'a syn::Block) {
        self.ctx.push("in-block");
        syn::visit::visit_block(self, b);
        self.ctx.pop();
    }
}

fn module_level(items: &[syn::Item], out: &mut Vec<(usize, usize)>) {
    for item in items {
        match item {
            syn::Item::Impl(i) => {
                out.push((i.impl_token.span.start().line, i.items.len()));
            }
            syn::Item::Mod(m) => {
                if let Some((_, content)) = &m.content {
                    module_level(content, out);
                }
            }
            _ => {}
        }
    }
}

fn macro_impls(text: &str) -> usize {
    // impls written inside module-level macro invocations (macro_rules bodies or calls)
    let file = match syn::parse_file(text) {
        Ok(f) => f,
        Err(_) => return 0,
    };
    let mut n = 0;
    fn walk(items: &[syn::Item], n: &mut usize) {
        for item in items {
            match item {
                syn::Item::Macro(m) => {
                    let s = m.mac.tokens.to_string();
                    *n += s.matches("impl").count();
                }
                syn::Item::Mod(m) => {
                    if let Some((_, c)) = &m.content {
                        walk(c, n);
                    }
                }
                _ => {}
            }
        }
    }
    walk(&file.items, &mut n);
    n
}

#[test]
#[ignore = "review probe"]
fn zz_impls() {
    let root = std::env::var("RAF_DUMP_ROOT").expect("root");
    let out_path = std::env::var("RAF_DUMP_OUT").expect("out");
    let tree = DiskTree::new(&root).unwrap();
    let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"));
    let mut out = String::new();
    // impls() by (file, line)
    let mut listed: BTreeMap<(String, usize), Vec<(String, usize)>> = BTreeMap::new();
    let all: Vec<_> = ws.impls().collect();
    let mut prev: Option<(String, usize, String, usize)> = None;
    let mut order_ok = true;
    for (pos, b) in all.iter().enumerate() {
        assert_eq!(b.id.index(), pos);
        assert_eq!(ws.impl_block(b.id).map(|x| x.id), Some(b.id));
        let key = (b.file.to_string(), b.span.start_byte, b.module.to_string(), b.target.index());
        if let Some(p) = &prev {
            if *p >= key {
                order_ok = false;
                writeln!(out, "ORDER {p:?} >= {key:?}").unwrap();
            }
        }
        prev = Some(key);
        // span sanity: text at span starts with '#' or 'impl' or 'unsafe' and ends with '}'
        let text = ws.source(&b.file).unwrap();
        let s = &text[b.span.start_byte..b.span.end_byte];
        let st = s.trim_start();
        if !(st.starts_with('#') || st.starts_with("impl") || st.starts_with("unsafe") || st.starts_with("///") || st.starts_with("//")) || !s.ends_with('}') {
            writeln!(out, "SPAN-ODD {}:{} {:?}", b.file, b.line, &s[..s.len().min(60)]).unwrap();
        }
        let line_of_start = text[..b.span.start_byte].matches('\n').count() + 1;
        if line_of_start != b.span.lines.start {
            writeln!(out, "SPAN-LINE {}:{} {} vs {}", b.file, b.line, line_of_start, b.span.lines.start).unwrap();
        }
        let impl_line_text = text.lines().nth(b.line - 1).unwrap_or("");
        if !impl_line_text.contains("impl") {
            writeln!(out, "LINE-ODD {}:{} {:?}", b.file, b.line, impl_line_text).unwrap();
        }
        listed
            .entry((b.file.to_string(), b.line))
            .or_default()
            .push((b.module.to_string(), b.members.len()));
    }
    writeln!(out, "impls() total {} distinct (file,line) {} order_ok {order_ok}", all.len(), listed.len()).unwrap();
    // syn walk
    let mut syn_level: BTreeSet<(String, usize)> = BTreeSet::new();
    let mut members: BTreeMap<(String, usize), usize> = BTreeMap::new();
    let mut nested_total = 0;
    let mut macro_total = 0;
    for f in ws.files() {
        let file = syn::parse_file(&f.text).unwrap();
        let mut v = Vec::new();
        module_level(&file.items, &mut v);
        for (line, n) in v {
            syn_level.insert((f.path.to_string(), line));
            members.insert((f.path.to_string(), line), n);
        }
        let mut nested = Nested::default();
        nested.visit_file(&file);
        for (line, wh) in nested.found {
            nested_total += 1;
            writeln!(out, "NESTED {}:{} {wh}", f.path, line).unwrap();
        }
        let m = macro_impls(&f.text);
        if m > 0 {
            macro_total += m;
            writeln!(out, "MACRO-IMPL-WORDS {} {m}", f.path).unwrap();
        }
    }
    writeln!(out, "syn module-level {} nested {nested_total} macro words {macro_total}", syn_level.len()).unwrap();
    let listed_keys: BTreeSet<_> = listed.keys().cloned().collect();
    for k in syn_level.difference(&listed_keys) {
        writeln!(out, "MISSING {}:{}", k.0, k.1).unwrap();
    }
    for k in listed_keys.difference(&syn_level) {
        writeln!(out, "EXTRA {}:{}", k.0, k.1).unwrap();
    }
    // member counts: impls() counts only fn/const/type members; syn counts all impl items (macros too)
    for (k, v) in &listed {
        if let Some(n) = members.get(k) {
            for (m, c) in v {
                if c != n {
                    writeln!(out, "MEMBERS {}:{} {m} listed {c} syn {n}", k.0, k.1).unwrap();
                }
            }
        }
    }
    for k in listed.keys() { writeln!(out, "LISTED {}:{}", k.0, k.1).unwrap(); }
    // multiple module paths
    for (k, v) in &listed {
        if v.len() > 1 {
            writeln!(out, "MULTI {}:{} {:?}", k.0, k.1, v.iter().map(|x| &x.0).collect::<Vec<_>>()).unwrap();
        }
    }
    std::fs::write(out_path, out).unwrap();
}
