// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Lexical candidates only. Standard-macro provenance is decided in the unit's
//! index context, never by a spelling or a failed ordinary path lookup.

use std::collections::BTreeMap;
use std::mem::size_of;

use proc_macro2::{Delimiter, TokenStream, TokenTree};

use crate::cost::{btree_entry, btree_map};
use crate::error::{LoadError, LoadErrorKind};
use crate::{Budget, RelPath};

mod arguments;
mod identity;
pub(super) use identity::Resolver;

#[derive(Debug)]
pub(super) struct Arguments {
    pub(super) captures: Vec<Box<str>>,
    pub(super) bindings: Vec<usize>,
}

#[derive(Debug)]
pub(super) enum Refused {
    Budget {
        budget: Budget,
        limit: usize,
        line: usize,
    },
    Syntax {
        message: String,
        line: usize,
        column: usize,
    },
}

impl Refused {
    pub(super) fn error(&self, file: &RelPath) -> LoadError {
        let (line, kind) = match self {
            Self::Budget {
                budget,
                limit,
                line,
            } => (
                *line,
                LoadErrorKind::Budget {
                    budget: *budget,
                    limit: *limit,
                },
            ),
            Self::Syntax {
                message,
                line,
                column,
            } => (
                *line,
                LoadErrorKind::Syntax {
                    message: message.clone(),
                    column: *column,
                },
            ),
        };
        LoadError::new(file, Some(line), kind)
    }
}

#[derive(Debug)]
pub(super) struct Call {
    pub(super) start: usize,
    scope: usize,
    path: Vec<String>,
    leading: bool,
    pub(super) arguments: Option<Result<Arguments, Refused>>,
}

#[derive(Debug)]
struct Import {
    path: Vec<String>,
    leading: bool,
    uncertain: bool,
}

#[derive(Debug, Default)]
enum ScopeKind {
    Module,
    #[default]
    Block,
}

#[derive(Debug, Default)]
struct Scope {
    kind: ScopeKind,
    parent: Option<usize>,
    enclosing: Option<usize>,
    macros: BTreeMap<String, Vec<usize>>,
    imports: BTreeMap<String, Vec<Import>>,
    globs: Vec<Import>,
    unknown: bool,
    disabled: bool,
    std_bound: bool,
}

#[derive(Debug, Default)]
pub(super) struct Formats {
    scopes: Vec<Scope>,
    pub(super) calls: Vec<Call>,
}

impl Formats {
    /// Charge the actual owned containers and text, including declined candidates.
    pub(super) fn retained_bytes(&self) -> usize {
        let mut held = size_of::<Self>()
            + self.scopes.capacity() * size_of::<Scope>()
            + self.calls.capacity() * size_of::<Call>();
        for scope in &self.scopes {
            held += scope.globs.capacity() * size_of::<Import>()
                + scope
                    .globs
                    .iter()
                    .map(|import| strings_bytes(&import.path))
                    .sum::<usize>();
            if !scope.macros.is_empty() {
                held += btree_map::<String, Vec<usize>>();
            }
            for (name, declarations) in &scope.macros {
                held += btree_entry::<String, Vec<usize>>()
                    + name.capacity()
                    + declarations.capacity() * size_of::<usize>();
            }
            if !scope.imports.is_empty() {
                held += btree_map::<String, Vec<Import>>();
            }
            for (name, imports) in &scope.imports {
                held += btree_entry::<String, Vec<Import>>()
                    + name.capacity()
                    + imports.capacity() * size_of::<Import>();
                held += imports
                    .iter()
                    .map(|import| strings_bytes(&import.path))
                    .sum::<usize>();
            }
        }
        for call in &self.calls {
            held += strings_bytes(&call.path);
            match &call.arguments {
                Some(Ok(args)) => {
                    held += args.captures.capacity() * size_of::<Box<str>>()
                        + args.captures.iter().map(|name| name.len()).sum::<usize>()
                        + args.bindings.capacity() * size_of::<usize>();
                }
                Some(Err(Refused::Syntax { message, .. })) => held += message.capacity(),
                Some(Err(Refused::Budget { .. })) | None => {}
            }
        }
        held
    }
}

fn strings_bytes(strings: &Vec<String>) -> usize {
    strings.capacity() * size_of::<String>() + strings.iter().map(String::capacity).sum::<usize>()
}

struct Frame {
    tokens: Vec<TokenTree>,
    at: usize,
    scope: usize,
    disabled: bool,
    conditional: bool,
    module_pending: bool,
    macro_group: Option<(usize, usize)>,
}

fn punct(token: Option<&TokenTree>, ch: char) -> bool {
    matches!(token, Some(TokenTree::Punct(p)) if p.as_char() == ch)
}

fn word(token: Option<&TokenTree>, name: &str) -> bool {
    matches!(token, Some(TokenTree::Ident(i)) if i == name)
}

/// A second bounded token walk records scopes and literal-leading invocations.
/// It returns all work to the caller; worker-local tallies would be lost.
pub(super) fn scan(
    stream: TokenStream,
    text: &str,
    base: usize,
) -> (Option<Box<Formats>>, usize, usize) {
    let mut formats = Formats {
        scopes: vec![Scope {
            kind: ScopeKind::Module,
            ..Scope::default()
        }],
        calls: Vec::new(),
    };
    let mut frames = vec![Frame {
        tokens: stream.into_iter().collect(),
        at: 0,
        scope: 0,
        disabled: false,
        conditional: false,
        module_pending: false,
        macro_group: None,
    }];
    let (mut tokens, mut bytes) = (0, 0);
    while let Some(frame) = frames.last_mut() {
        if frame.at == frame.tokens.len() {
            frames.pop();
            continue;
        }
        let at = frame.at;
        frame.at += 1;
        tokens += 1;
        if frame.declaration(
            at,
            &mut formats.scopes[frame.scope],
            base,
            &mut tokens,
            &mut bytes,
        ) {
            continue;
        }
        if word(frame.tokens.get(at), "mod") {
            frame.module_pending = true;
        }
        if punct(frame.tokens.get(at), ';') {
            frame.module_pending = false;
        }
        if matches!(frame.tokens.get(at), Some(TokenTree::Ident(_)))
            && let Some((path, leading, bang)) = macro_path(&frame.tokens, at)
            && let Some(TokenTree::Group(group)) = frame.tokens.get(bang + 1)
        {
            let start = base + frame.tokens[at].span().byte_range().start;
            let arguments = arguments::read(group, text, base, &mut bytes);
            frame.macro_group = Some((bang + 1, formats.calls.len()));
            formats.calls.push(Call {
                start,
                scope: frame.scope,
                path,
                leading,
                arguments,
            });
        }
        if let TokenTree::Group(group) = &frame.tokens[at] {
            let scope = formats.scopes.len();
            formats.scopes.push(Scope {
                parent: Some(frame.scope),
                enclosing: frame
                    .macro_group
                    .filter(|(group, _)| *group == at)
                    .map(|(_, call)| call),
                kind: if frame.module_pending && group.delimiter() == Delimiter::Brace {
                    ScopeKind::Module
                } else {
                    ScopeKind::Block
                },
                disabled: frame.disabled,
                ..Scope::default()
            });
            frame.disabled = false;
            frame.conditional = false;
            frame.module_pending = false;
            let next = Frame {
                tokens: group.stream().into_iter().collect(),
                at: 0,
                scope,
                disabled: false,
                conditional: false,
                module_pending: false,
                macro_group: None,
            };
            frames.push(next);
        }
    }
    // The explicit depth-first walk records invocations in source order. Scope
    // links retain these indices, so they must not be reordered independently.
    formats.calls.shrink_to_fit();
    formats.scopes.shrink_to_fit();
    let kept = formats
        .calls
        .iter()
        .any(|call| call.arguments.is_some())
        .then(|| Box::new(formats));
    (kept, tokens, bytes)
}

impl Frame {
    fn declaration(
        &mut self,
        at: usize,
        scope: &mut Scope,
        base: usize,
        tokens: &mut usize,
        bytes: &mut usize,
    ) -> bool {
        if word(self.tokens.get(at), "macro_rules")
            && punct(self.tokens.get(at + 1), '!')
            && let (Some(TokenTree::Ident(name)), Some(TokenTree::Group(body))) =
                (self.tokens.get(at + 2), self.tokens.get(at + 3))
        {
            scope
                .macros
                .entry(name.to_string().trim_start_matches("r#").to_owned())
                .or_default()
                .push(
                    base + if punct(self.tokens.get(at + 4), ';') {
                        self.tokens[at + 4].span().byte_range().end
                    } else {
                        body.span().byte_range().end
                    },
                );
            self.at = at + 4;
            *tokens += 3;
            return true;
        }
        if word(self.tokens.get(at), "use") {
            let end = self.tokens[at..]
                .iter()
                .position(|token| punct(Some(token), ';'))
                .map(|n| at + n + 1);
            if let Some(end) = end {
                read_use(&self.tokens[at..end], scope, self.conditional, bytes);
                self.at = end;
                *tokens += end - at - 1;
                self.conditional = false;
                self.disabled = false;
                return true;
            }
        }
        if (word(self.tokens.get(at), "mod")
            || word(self.tokens.get(at), "type")
            || word(self.tokens.get(at), "struct")
            || word(self.tokens.get(at), "extern"))
            && (word(self.tokens.get(at + 1), "std")
                || (word(self.tokens.get(at), "extern")
                    && word(self.tokens.get(at + 1), "crate")
                    && word(self.tokens.get(at + 2), "std")))
        {
            scope.std_bound = true;
        }
        if punct(self.tokens.get(at), '#') {
            let inner = punct(self.tokens.get(at + 1), '!');
            if let Some(TokenTree::Group(attribute)) =
                self.tokens.get(at + if inner { 2 } else { 1 })
            {
                let attribute = attribute.stream().to_string();
                *bytes += attribute.len();
                let uncertain = attribute.starts_with("no_implicit_prelude")
                    || attribute.starts_with("no_std")
                    || attribute.starts_with("no_core")
                    || attribute.starts_with("macro_use")
                    || (attribute.starts_with("cfg_attr")
                        && attribute
                            .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
                            .any(|s| {
                                matches!(
                                    s,
                                    "no_implicit_prelude" | "no_std" | "no_core" | "macro_use"
                                )
                            }));
                if inner {
                    scope.disabled |= uncertain;
                } else {
                    self.disabled |= uncertain;
                    self.conditional |= attribute.starts_with("cfg");
                }
                self.at = at + if inner { 3 } else { 2 };
                *tokens += if inner { 2 } else { 1 };
                return true;
            }
        }
        false
    }
}

fn macro_path(tokens: &[TokenTree], at: usize) -> Option<(Vec<String>, bool, usize)> {
    if punct(at.checked_sub(1).and_then(|n| tokens.get(n)), '.') {
        return None;
    }
    let leading = at >= 2 && punct(tokens.get(at - 1), ':') && punct(tokens.get(at - 2), ':');
    if leading && at >= 3 && matches!(tokens.get(at - 3), Some(TokenTree::Ident(_))) {
        return None;
    }
    let mut next = at;
    loop {
        let TokenTree::Ident(_) = tokens.get(next)? else {
            return None;
        };
        next += 1;
        if punct(tokens.get(next), '!') {
            let path = (at..next)
                .step_by(3)
                .map(|at| {
                    let TokenTree::Ident(name) = &tokens[at] else {
                        unreachable!("validated path")
                    };
                    name.to_string().trim_start_matches("r#").to_owned()
                })
                .collect();
            return Some((path, leading, next));
        }
        if !punct(tokens.get(next), ':') || !punct(tokens.get(next + 1), ':') {
            return None;
        }
        next += 2;
    }
}

fn read_use(tokens: &[TokenTree], scope: &mut Scope, uncertain: bool, bytes: &mut usize) {
    let stream: TokenStream = tokens.iter().cloned().collect();
    let text = stream.to_string();
    *bytes += text.len();
    let file = RelPath::new("format-import.rs").expect("a fixed relative path");
    if crate::prescan::prescan(&file, &text, Budget::FileTokens.limit()).is_err() {
        scope.unknown = true;
        return;
    }
    let Ok(import) = syn::parse2::<syn::ItemUse>(stream) else {
        scope.unknown = true;
        return;
    };
    let leading = import.leading_colon.is_some();
    let mut stack = vec![(Vec::<String>::new(), &import.tree)];
    while let Some((mut path, tree)) = stack.pop() {
        match tree {
            syn::UseTree::Path(part) => {
                path.push(part.ident.to_string().trim_start_matches("r#").to_owned());
                stack.push((path, &part.tree));
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    stack.push((path.clone(), tree));
                }
            }
            syn::UseTree::Glob(_) => scope.globs.push(Import {
                path,
                leading,
                uncertain,
            }),
            syn::UseTree::Name(name) => {
                let name = name.ident.to_string().trim_start_matches("r#").to_owned();
                path.push(name.clone());
                scope.imports.entry(name).or_default().push(Import {
                    path,
                    leading,
                    uncertain,
                });
            }
            syn::UseTree::Rename(rename) => {
                path.push(rename.ident.to_string().trim_start_matches("r#").to_owned());
                scope
                    .imports
                    .entry(
                        rename
                            .rename
                            .to_string()
                            .trim_start_matches("r#")
                            .to_owned(),
                    )
                    .or_default()
                    .push(Import {
                        path,
                        leading,
                        uncertain,
                    });
            }
        }
    }
}
