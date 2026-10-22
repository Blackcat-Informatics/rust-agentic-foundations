// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Make known fixture-constant dependencies explicit in existing format calls.
//! This editor preserves the source bytes outside macro argument insertions.

use std::collections::{BTreeMap, BTreeSet};
use proc_macro2::{TokenStream, TokenTree};

fn fields(text: &str, names: &BTreeSet<String>) -> BTreeSet<String> {
    let mut answer = BTreeSet::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '{' { continue; }
        if chars.peek() == Some(&'{') { chars.next(); continue; }
        let mut name = String::new();
        while let Some(&next) = chars.peek() {
            if !next.is_ascii_alphanumeric() && next != '_' { break; }
            name.push(chars.next().unwrap());
        }
        if names.contains(&name) { answer.insert(name); }
    }
    answer
}

fn edits(stream: TokenStream, names: &BTreeSet<String>, result: &mut BTreeMap<usize, String>) {
    let tokens: Vec<_> = stream.into_iter().collect();
    for (index, token) in tokens.iter().enumerate() {
        if let TokenTree::Group(group) = token { edits(group.stream(), names, result); }
        let TokenTree::Ident(name) = token else { continue; };
        if !["format", "format_args", "write", "writeln", "print", "println", "eprint", "eprintln"].contains(&name.to_string().as_str()) { continue; }
        if !matches!(tokens.get(index + 1), Some(TokenTree::Punct(p)) if p.as_char() == '!') { continue; }
        let Some(TokenTree::Group(group)) = tokens.get(index + 2) else { continue; };
        let args: Vec<_> = group.stream().into_iter().collect();
        let Some(text) = args.iter().find_map(|token| {
            if !matches!(token, TokenTree::Literal(_)) { return None; }
            raf_source::syn::parse_str::<raf_source::syn::LitStr>(&token.to_string()).ok().map(|v| v.value())
        }) else { continue; };
        let mut captured = fields(&text, names);
        for pair in args.windows(2) {
            if matches!(&pair[1], TokenTree::Punct(p) if p.as_char() == '=') {
                if let TokenTree::Ident(name) = &pair[0] { captured.remove(&name.to_string()); }
            }
        }
        if captured.is_empty() { continue; }
        let comma = if matches!(args.last(), Some(TokenTree::Punct(p)) if p.as_char() == ',') { " " } else { ", " };
        let append = captured.iter().map(|name| format!("{name} = {name}")).collect::<Vec<_>>().join(", ");
        assert!(result.insert(group.span_close().byte_range().start, format!("{comma}{append}")).is_none());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let report = std::fs::read_to_string(".stage/raf-workplan-strict-mode-force-coverage/raw/adoption-candidate-second/lines.md")?;
    let mut files = BTreeMap::<String, BTreeSet<String>>::new();
    for line in report.lines().filter(|line| line.starts_with('|')) {
        let cells: Vec<_> = line.split('|').map(str::trim).collect();
        if cells.len() < 7 || cells[3] != "const" { continue; }
        let path = cells[2].trim_matches('`');
        if !path.contains("::test_crate::") && !path.contains("::tests::") { continue; }
        files.entry(cells[4].into()).or_default().insert(path.rsplit("::").next().unwrap().into());
    }
    for (path, names) in files {
        let mut text = std::fs::read_to_string(&path)?;
        let mut changes = BTreeMap::new();
        edits(text.parse()?, &names, &mut changes);
        println!("{path}: {} explicit format argument insertions", changes.len());
        for (at, append) in changes.into_iter().rev() { text.insert_str(at, &append); }
        std::fs::write(path, text)?;
    }
    Ok(())
}
