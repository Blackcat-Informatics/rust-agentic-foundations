// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The token tree every pass rewrites, and the lexer that builds it.

use std::ops::Range;
use std::str::FromStr;

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};

use crate::Error;

/// The deepest delimiter nesting a slice may have. It is the delimiter budget
/// of `raf-source`, so a slice of a loaded workspace never reaches it.
pub const MAX_NESTING: usize = 128;

/// What a leaf token is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Ident,
    Punct,
    Literal,
}

/// A delimiter pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Delim {
    Paren,
    Brace,
    Bracket,
    /// An invisible group; the lexer never builds one.
    Invisible,
}

impl Delim {
    const fn open(self) -> &'static str {
        match self {
            Self::Paren => "(",
            Self::Brace => "{",
            Self::Bracket => "[",
            Self::Invisible => "«",
        }
    }

    const fn close(self) -> &'static str {
        match self {
            Self::Paren => ")",
            Self::Brace => "}",
            Self::Bracket => "]",
            Self::Invisible => "»",
        }
    }
}

/// Where a token came from: its byte range inside the slice and its 1-based
/// line in the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Place {
    pub(crate) at: Range<usize>,
    pub(crate) line: usize,
}

/// A token that is not a delimiter.
#[derive(Clone, Debug)]
pub(crate) struct Leaf {
    pub(crate) kind: Kind,
    /// An identifier or a literal as written; a punctuation character.
    pub(crate) text: String,
    /// A punctuation character joined to the next one (`-` in `->`).
    pub(crate) joint: bool,
    pub(crate) place: Place,
}

/// A delimited group.
#[derive(Clone, Debug)]
pub(crate) struct Group {
    pub(crate) delim: Delim,
    pub(crate) open: Place,
    pub(crate) close: Place,
    pub(crate) children: Vec<Node>,
}

/// A token tree node.
#[derive(Clone, Debug)]
pub(crate) enum Node {
    Leaf(Leaf),
    Group(Group),
}

impl Node {
    pub(crate) fn as_leaf(&self) -> Option<&Leaf> {
        match self {
            Self::Leaf(leaf) => Some(leaf),
            Self::Group(_) => None,
        }
    }

    pub(crate) fn as_group(&self) -> Option<&Group> {
        match self {
            Self::Group(group) => Some(group),
            Self::Leaf(_) => None,
        }
    }
}

/// One token of a flattened form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    /// The token as it is hashed: a joined punctuation character carries a
    /// trailing `~`, and an opening or closing delimiter is its character.
    pub text: String,
    /// The byte range of the token in the slice that was lexed.
    pub at: Range<usize>,
    /// The 1-based line of the token in its file.
    pub line: usize,
}

/// A token tree: a function body, or an initialiser, at one level of the
/// tower. Every token keeps the place it came from in the source.
#[derive(Clone, Debug)]
pub struct Form {
    pub(crate) nodes: Vec<Node>,
}

impl Form {
    /// Lexes `text`, whose first byte is on `first_line` of `file`.
    ///
    /// # Errors
    ///
    /// [`Error::Source`] when the text does not lex, or nests delimiters
    /// deeper than [`MAX_NESTING`].
    pub(crate) fn lex(file: &str, first_line: usize, text: &str) -> Result<Self, Error> {
        let stream = TokenStream::from_str(text).map_err(|error| Error::Source {
            file: file.to_owned(),
            line: first_line + error.span().start().line.saturating_sub(1),
            reason: format!("the source does not lex: {error}"),
        })?;
        let lexer = Lexer { file, first_line };
        Ok(Self {
            nodes: lexer.nodes(stream, 0)?,
        })
    }

    /// The tokens in order, delimiters included. Every threshold counts these.
    #[must_use]
    pub fn tokens(&self) -> Vec<Token> {
        let mut out = Vec::new();
        flatten(&self.nodes, &mut out);
        out
    }

    /// The number of tokens, delimiters included.
    #[must_use]
    pub fn len(&self) -> usize {
        count(&self.nodes)
    }

    /// Whether the form has no token.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The token texts in order.
    #[must_use]
    pub fn texts(&self) -> Vec<String> {
        self.tokens().into_iter().map(|token| token.text).collect()
    }
}

fn count(nodes: &[Node]) -> usize {
    nodes
        .iter()
        .map(|node| match node {
            Node::Leaf(_) => 1,
            Node::Group(group) => 2 + count(&group.children),
        })
        .sum()
}

fn flatten(nodes: &[Node], out: &mut Vec<Token>) {
    for node in nodes {
        match node {
            Node::Leaf(leaf) => out.push(Token {
                text: if leaf.kind == Kind::Punct && leaf.joint {
                    format!("{}~", leaf.text)
                } else {
                    leaf.text.clone()
                },
                at: leaf.place.at.clone(),
                line: leaf.place.line,
            }),
            Node::Group(group) => {
                out.push(Token {
                    text: group.delim.open().to_owned(),
                    at: group.open.at.clone(),
                    line: group.open.line,
                });
                flatten(&group.children, out);
                out.push(Token {
                    text: group.delim.close().to_owned(),
                    at: group.close.at.clone(),
                    line: group.close.line,
                });
            }
        }
    }
}

struct Lexer<'a> {
    file: &'a str,
    first_line: usize,
}

impl Lexer<'_> {
    fn place(&self, span: proc_macro2::Span) -> Place {
        Place {
            at: span.byte_range(),
            line: self.first_line + span.start().line.saturating_sub(1),
        }
    }

    fn nodes(&self, stream: TokenStream, depth: usize) -> Result<Vec<Node>, Error> {
        let mut nodes = Vec::new();
        for token in stream {
            nodes.push(match token {
                TokenTree::Group(group) => {
                    if depth >= MAX_NESTING {
                        return Err(Error::Source {
                            file: self.file.to_owned(),
                            line: self.place(group.span()).line,
                            reason: format!(
                                "delimiters nest deeper than the budget of {MAX_NESTING}"
                            ),
                        });
                    }
                    Node::Group(Group {
                        delim: match group.delimiter() {
                            Delimiter::Parenthesis => Delim::Paren,
                            Delimiter::Brace => Delim::Brace,
                            Delimiter::Bracket => Delim::Bracket,
                            Delimiter::None => Delim::Invisible,
                        },
                        open: self.place(group.span_open()),
                        close: self.place(group.span_close()),
                        children: self.nodes(group.stream(), depth + 1)?,
                    })
                }
                TokenTree::Ident(ident) => Node::Leaf(Leaf {
                    kind: Kind::Ident,
                    text: ident.to_string(),
                    joint: false,
                    place: self.place(ident.span()),
                }),
                TokenTree::Punct(punct) => Node::Leaf(Leaf {
                    kind: Kind::Punct,
                    text: punct.as_char().to_string(),
                    joint: punct.spacing() == Spacing::Joint,
                    place: self.place(punct.span()),
                }),
                TokenTree::Literal(literal) => Node::Leaf(Leaf {
                    kind: Kind::Literal,
                    text: crate::normalize::literal::respell(&literal.to_string()),
                    joint: false,
                    place: self.place(literal.span()),
                }),
            });
        }
        Ok(nodes)
    }
}
