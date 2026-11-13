// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Join original native lexical spans with source-owned binding annotations.
//!
//! Native input streams belong to the parsing stage. The flat lexical inventory
//! uses the original analysis account; native parser and lexer storage still
//! require a separate admitted envelope before exposing a global memory claim.

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};

use super::collect::{Annotation, Facts, Span};
use super::coordinates::Coordinates;
use super::proof::Atom;
use super::resource::{Meter, Refusal, Scratch};

/// Parallel buffers permit a borrowed proof view without copying its atoms.
pub(super) struct Tokens<'source> {
    pub(super) atoms: Vec<Atom<'source>>,
    pub(super) spans: Vec<Span>,
}

#[derive(Clone, Copy)]
enum LexicalKind {
    Group {
        end: usize,
        invisible: bool,
    },
    Token {
        identifier: bool,
        punctuation: Option<char>,
        joint: bool,
    },
}

/// Flat original lexical data; no parser-owned span or token escapes.
#[derive(Clone, Copy)]
pub(super) struct Lexeme {
    span: Span,
    kind: LexicalKind,
}

/// Consume a source's original stream once. The inventory shares the original
/// analysis account, and all functions borrow it during this parser callback.
/// The inverse coordinate table shares this account. Native lexer, span
/// construction and TLS storage still need their separate envelope.
pub(super) fn inventory<'meter>(
    original: TokenStream,
    meter: &'meter Meter,
    coordinates: &Coordinates<'_, '_>,
) -> Result<Scratch<'meter, Lexeme>, Refusal> {
    let mut out = Scratch::new(meter);
    inventory_stream(original, 0, &mut out, meter, coordinates)?;
    Ok(out)
}

fn lexical_span(
    native: proc_macro2::Span,
    coordinates: &Coordinates<'_, '_>,
) -> Result<Span, Refusal> {
    let range = coordinates.range(native)?;
    Ok(Span {
        start: range.start,
        end: range.end,
    })
}

fn inventory_token(
    native: proc_macro2::Span,
    identifier: bool,
    punctuation: Option<char>,
    joint: bool,
    out: &mut Scratch<'_, Lexeme>,
    coordinates: &Coordinates<'_, '_>,
) -> Result<(), Refusal> {
    out.push(Lexeme {
        span: lexical_span(native, coordinates)?,
        kind: LexicalKind::Token {
            identifier,
            punctuation,
            joint,
        },
    })
}

fn inventory_stream(
    original: TokenStream,
    depth: usize,
    out: &mut Scratch<'_, Lexeme>,
    meter: &Meter,
    coordinates: &Coordinates<'_, '_>,
) -> Result<(), Refusal> {
    // Source admits at most MAX_NESTING written delimiters. Native lowering
    // adds one bracket group for a doc comment, whose children are terminals.
    if depth > crate::MAX_NESTING + 1 {
        return Err(Refusal::Unsupported(
            "native token nesting exceeds the source profile",
        ));
    }
    for token in original {
        meter.work(1)?;
        match token {
            TokenTree::Group(group) => {
                let at = out.len();
                let delimiter = group.delimiter();
                let span = lexical_span(group.span(), coordinates)?;
                out.push(Lexeme {
                    span,
                    kind: LexicalKind::Group {
                        end: 0,
                        invisible: delimiter == Delimiter::None,
                    },
                })?;
                let punctuation = match delimiter {
                    Delimiter::Parenthesis => Some(('(', ')')),
                    Delimiter::Brace => Some(('{', '}')),
                    Delimiter::Bracket => Some(('[', ']')),
                    Delimiter::None => None,
                };
                let open = group.span_open();
                let close = group.span_close();
                let children = group.stream();
                // The child stream may now be uniquely owned when consumed;
                // retaining the Group would force a native vector clone.
                drop(group);
                if let Some((open_token, _)) = punctuation {
                    inventory_token(open, false, Some(open_token), false, out, coordinates)?;
                }
                inventory_stream(children, depth + 1, out, meter, coordinates)?;
                if let Some((_, close_token)) = punctuation {
                    inventory_token(close, false, Some(close_token), false, out, coordinates)?;
                }
                let end = out.len();
                out.as_mut_slice()[at].kind = LexicalKind::Group {
                    end,
                    invisible: delimiter == Delimiter::None,
                };
            }
            TokenTree::Ident(token) => {
                inventory_token(token.span(), true, None, false, out, coordinates)?;
            }
            TokenTree::Literal(token) => {
                inventory_token(token.span(), false, None, false, out, coordinates)?;
            }
            TokenTree::Punct(token) => inventory_token(
                token.span(),
                false,
                Some(token.as_char()),
                token.spacing() == Spacing::Joint,
                out,
                coordinates,
            )?,
        }
    }
    Ok(())
}

/// Input comes from the native lexer of this complete original source, never
/// an AST-printed stream. All byte ranges are absolute within `source`.
pub(super) fn join<'source>(
    source: &'source str,
    facts: &Facts<'source>,
    original: &[Lexeme],
    meter: &Meter,
) -> Result<Tokens<'source>, Refusal> {
    let body = source
        .get(facts.body.start..facts.body.end)
        .ok_or(Refusal::Invariant(
            "token owner is outside the original UTF-8 source",
        ))?;
    if body.len() < 2 || !body.starts_with('{') || !body.ends_with('}') {
        return Err(Refusal::Invariant(
            "token owner is not an original braced function body",
        ));
    }
    let ordered = ordered_annotations(source, facts, meter)?;
    for opaque in &facts.opaque {
        meter.work(1)?;
        if opaque.start >= opaque.end
            || opaque.start < facts.body.start
            || opaque.end > facts.body.end
            || source.get(opaque.start..opaque.end).is_none()
        {
            return Err(Refusal::Invariant(
                "opaque region has no original body span",
            ));
        }
    }
    let mut writer = Join {
        source,
        body: facts.body,
        ordered: ordered.as_slice(),
        opaque: &facts.opaque,
        next: 0,
        previous: 0,
        meter,
        out: Tokens {
            atoms: Vec::new(),
            spans: Vec::new(),
        },
    };
    writer.lexemes(original)?;
    writer.markers(facts.body.end)?;
    if writer.next != writer.ordered.len() {
        return Err(Refusal::Invariant(
            "a binding annotation has no original lexical token",
        ));
    }
    if writer.out.spans.first()
        != Some(&Span {
            start: facts.body.start,
            end: facts.body.start + 1,
        })
        || writer.out.spans.last()
            != Some(&Span {
                start: facts.body.end - 1,
                end: facts.body.end,
            })
    {
        return Err(Refusal::Invariant(
            "native token stream omits the original body delimiters",
        ));
    }
    Ok(writer.out)
}

fn ordered_annotations<'source, 'meter>(
    source: &str,
    facts: &Facts<'source>,
    meter: &'meter Meter,
) -> Result<Scratch<'meter, Annotation<'source>>, Refusal> {
    let mut ordered = Scratch::new(meter);
    for annotation in &facts.annotations {
        meter.work(1)?;
        let span = annotation.span;
        let anchor = annotation.anchor;
        if span.start > span.end
            || span.start < facts.body.start
            || span.end > facts.body.end
            || source.get(span.start..span.end).is_none()
            || annotation.marker != (span.start == span.end)
            || anchor.start >= anchor.end
            || anchor.start < facts.body.start
            || anchor.end > facts.body.end
            || source.get(anchor.start..anchor.end).is_none()
            || span.start < anchor.start
            || span.end > anchor.end
            || (!annotation.marker && span != anchor)
        {
            return Err(Refusal::Invariant(
                "annotation has no valid original body span",
            ));
        }
        ordered.push(*annotation)?;
        // Stable insertion: original traversal order breaks equal-byte marker
        // ties. Every comparison and physical move consumes original work.
        let values = ordered.as_mut_slice();
        let mut at = values.len() - 1;
        while at > 0 {
            meter.work(1)?;
            let key = |a: &Annotation<'_>| (a.span.start, a.span.end);
            if key(&values[at - 1]) <= key(&values[at]) {
                break;
            }
            meter.work(1)?;
            values.swap(at - 1, at);
            at -= 1;
        }
    }
    let mut end = facts.body.start;
    for annotation in ordered.iter() {
        meter.work(1)?;
        if annotation.marker {
            continue;
        }
        if annotation.span.start < end {
            return Err(Refusal::Invariant("binding annotation spans overlap"));
        }
        end = annotation.span.end;
    }
    Ok(ordered)
}

struct Join<'source, 'borrow> {
    source: &'source str,
    body: Span,
    ordered: &'borrow [Annotation<'source>],
    opaque: &'borrow [Span],
    next: usize,
    previous: usize,
    meter: &'borrow Meter,
    out: Tokens<'source>,
}

impl Join<'_, '_> {
    fn lexemes(&mut self, original: &[Lexeme]) -> Result<(), Refusal> {
        let mut at = 0;
        while let Some(lexeme) = original.get(at) {
            self.meter.work(1)?;
            let range = lexeme.span;
            let mut opaque = false;
            for barrier in self.opaque {
                self.meter.work(1)?;
                if range.start < range.end
                    && barrier.start <= range.start
                    && range.end <= barrier.end
                {
                    opaque = true;
                    break;
                }
            }
            // Doc attributes inside local items may be native desugarings.
            // Omit that entire owned opaque subtree; source-region admission
            // still refuses every candidate intersecting its original span.
            if opaque {
                at = match lexeme.kind {
                    LexicalKind::Group { end, .. } => end,
                    LexicalKind::Token { .. } => at + 1,
                };
                continue;
            }
            match lexeme.kind {
                LexicalKind::Group {
                    invisible: true, ..
                } => {
                    return Err(Refusal::Invariant("synthetic invisible lexical group"));
                }
                LexicalKind::Group { .. } => {}
                LexicalKind::Token {
                    identifier,
                    punctuation,
                    joint,
                } => {
                    self.token(range, identifier, punctuation, joint)?;
                }
            }
            at += 1;
        }
        Ok(())
    }

    fn token(
        &mut self,
        range: Span,
        identifier: bool,
        punctuation: Option<char>,
        joint: bool,
    ) -> Result<(), Refusal> {
        self.meter.work(1)?;
        // The complete source stream also contains other functions and doc
        // attributes. Their synthetic spans cannot attest this body's tokens.
        if range.end <= self.body.start || range.start >= self.body.end {
            return Ok(());
        }
        let spelling = self
            .source
            .get(range.start..range.end)
            .ok_or(Refusal::Invariant("native token has no UTF-8 source span"))?;
        if range.start >= range.end || range.start < self.previous {
            return Err(Refusal::Invariant(
                "native token spans are empty, synthetic or overlapping",
            ));
        }
        self.previous = range.end;
        if let Some(expected) = punctuation
            && (spelling.len() != expected.len_utf8() || !spelling.starts_with(expected))
        {
            return Err(Refusal::Unsupported(
                "desugared or synthetic tokens have no original spelling",
            ));
        }
        if range.start < self.body.start || range.end > self.body.end {
            return Err(Refusal::Invariant(
                "native token crosses its function body boundary",
            ));
        }
        self.markers(range.start)?;
        let span = Span {
            start: range.start,
            end: range.end,
        };
        let occurrence = if let Some(annotation) = self.ordered.get(self.next) {
            self.meter.work(1)?;
            if !annotation.marker && annotation.span.start == range.start {
                if annotation.span != span || !identifier {
                    return Err(Refusal::Invariant(
                        "binding annotation does not cover exactly one original identifier",
                    ));
                }
                self.next += 1;
                Some(annotation.occurrence)
            } else {
                if annotation.span.start < range.end {
                    return Err(Refusal::Invariant(
                        "binding annotation falls inside a lexical token",
                    ));
                }
                None
            }
        } else {
            None
        };
        self.meter.push_retained(
            &mut self.out.atoms,
            Atom {
                spelling,
                occurrence,
                joint,
            },
        )?;
        self.meter.push_retained(&mut self.out.spans, span)?;
        self.markers(range.end)
    }

    fn markers(&mut self, through: usize) -> Result<(), Refusal> {
        while let Some(annotation) = self.ordered.get(self.next) {
            self.meter.work(1)?;
            if !annotation.marker || annotation.span.start > through {
                break;
            }
            // A marker cannot hide inside an already consumed token.
            if annotation.span.start < self.previous && annotation.span.start != through {
                return Err(Refusal::Invariant(
                    "control marker falls inside a lexical token",
                ));
            }
            self.meter.push_retained(
                &mut self.out.atoms,
                Atom {
                    spelling: "",
                    occurrence: Some(annotation.occurrence),
                    joint: false,
                },
            )?;
            self.meter
                .push_retained(&mut self.out.spans, annotation.anchor)?;
            self.next += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tokens_tests.rs"]
mod tests;
