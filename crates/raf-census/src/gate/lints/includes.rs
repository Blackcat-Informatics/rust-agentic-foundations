// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Additional first-party inputs consumed by Rust's file-inclusion macros.

use crate::Error;
use proc_macro2::{Ident, TokenStream, TokenTree};
use purrdf_core::ContentDigest;
use raf_source::{LoadOptions, RelPath, Tree, Workspace, prepare_source};
use std::collections::{BTreeMap, BTreeSet};

fn file_macro(name: &Ident) -> bool {
    [
        "include",
        "r#include",
        "include_str",
        "r#include_str",
        "include_bytes",
        "r#include_bytes",
    ]
    .iter()
    .any(|spelling| name == *spelling)
}

fn paths(path: &RelPath, text: &str, fragment: bool) -> Result<Vec<(RelPath, bool)>, Error> {
    if !(text.contains("include") || fragment && text.contains("mod")) {
        return Ok(Vec::new());
    }
    let parent = path.parent().unwrap_or_else(RelPath::root);
    prepare_source(path, text)?.with_lexed(|lexed| {
        let mut inputs = Vec::new();
        let mut groups = vec![lexed.tokens().clone()];
        while let Some(group) = groups.pop() {
            collect(group, &parent, path, fragment, &mut groups, &mut inputs)?;
        }
        Ok(inputs)
    })?
}

fn collect(
    stream: TokenStream,
    parent: &RelPath,
    path: &RelPath,
    fragment: bool,
    groups: &mut Vec<TokenStream>,
    inputs: &mut Vec<(RelPath, bool)>,
) -> Result<(), Error> {
    let mut tokens = stream.into_iter().peekable();
    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Group(group) => groups.push(group.stream()),
            TokenTree::Ident(name) if fragment && name == "mod" => {
                if matches!(tokens.peek(), Some(TokenTree::Ident(_))) {
                    tokens.next();
                    if matches!(tokens.peek(), Some(TokenTree::Punct(p)) if p.as_char() == ';') {
                        return Err(Error::Options(format!(
                            "{path}: an included Rust fragment declares a file-backed module whose compiler inputs cannot be established"
                        )));
                    }
                }
            }
            TokenTree::Ident(name) if file_macro(&name) => {
                let colon =
                    |token: &TokenTree| matches!(token, TokenTree::Punct(p) if p.as_char() == ':');
                let grouped_import = tokens.next_if(colon).is_some()
                    && tokens.next_if(colon).is_some()
                    && matches!(tokens.peek(), Some(TokenTree::Group(group)) if group.stream().into_iter().any(|token| matches!(token, TokenTree::Ident(id) if id == "self")));
                if grouped_import
                    || matches!(tokens.peek(), Some(TokenTree::Ident(next)) if next == "as")
                {
                    return Err(Error::Options(format!(
                        "{path}: an aliased {name}! can read files outside the recorded module tree; invoke the file-inclusion macro by its original name"
                    )));
                }
                if !matches!(tokens.peek(), Some(TokenTree::Punct(punct)) if punct.as_char() == '!')
                {
                    continue;
                }
                tokens.next();
                let literal = match tokens.next() {
                    Some(TokenTree::Group(args)) => syn::parse2::<syn::LitStr>(args.stream()).ok(),
                    _ => None,
                }.ok_or_else(|| Error::Options(format!("{path}: cannot attest the input of dynamic {name}!; the build must identify a literal workspace file")))?;
                let relative = parent
                    .join(&literal.value())
                    .map_err(|e| Error::Options(format!("{path}: {name}! input: {e}")))?;
                inputs.push((relative, name == "include" || name == "r#include"));
            }
            _ => (),
        }
    }
    Ok(())
}

pub(super) fn add(
    tree: &impl Tree,
    workspace: &Workspace,
    options: &LoadOptions,
    files: &mut BTreeMap<RelPath, ContentDigest>,
) -> Result<(), Error> {
    let mut parsed = BTreeSet::new();
    let mut pending = Vec::new();
    for file in workspace.files().filter(|file| file.shipping) {
        parsed.insert(file.path.clone());
        pending.extend(paths(&file.path, &file.text, false)?);
    }
    let mut remaining = options.max_total_bytes;
    while let Some((path, rust)) = pending.pop() {
        if (!rust || parsed.contains(&path)) && files.contains_key(&path) {
            continue;
        }
        let bytes = tree
            .read_bytes(&path, options.max_file_bytes.min(remaining as u64))
            .map_err(|source| super::file_error(&path, source))?;
        remaining -= bytes.len();
        files.insert(path.clone(), ContentDigest::of(&bytes));
        if rust && parsed.insert(path.clone()) {
            let text = std::str::from_utf8(&bytes).map_err(|error| {
                Error::Options(format!("{path}: included Rust is not UTF-8: {error}"))
            })?;
            pending.extend(paths(&path, text, true)?);
        }
    }
    Ok(())
}
