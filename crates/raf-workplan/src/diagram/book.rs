// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The documented mdBook preprocessor protocol over shared named diagrams.

use super::{DiagramError, documentation::static_fragment};
use crate::Workplan;
use pulldown_cmark::{Event, Parser, Tag};
use purrdf_lex::json::{Object, Value};
use std::ops::Range;

const DIRECTIVE: &str = "{{#workplan-diagram ";
const MAX_CHAPTERS: usize = 4096;
const MAX_DEPTH: usize = 64;
const MAX_REFERENCES: usize = 256;
const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;
const MAX_OUTPUT_BYTES: usize = 64 * 1024 * 1024;

/// Transform an mdBook `[context, book]` request into the output book alone.
///
/// An unindented standalone `{{#workplan-diagram cap:slug#name}}` line embeds
/// the same named bundle's packaged static fragment. Architecture references
/// use `arch:slug#name`. [CommonMark](https://spec.commonmark.org/) byte offsets leave inline code,
/// including multiline spans, fenced and indented code, and raw HTML blocks and
/// tags unchanged.
/// A directive must form its own top-level paragraph, outside other markup.
/// All unrelated book/chapter fields, section variants and their order survive.
///
/// # Errors
/// Refuses malformed protocol shapes, duplicate members, non-HTML renderers,
/// inline or malformed visible directives, unavailable named views and
/// Markdown/rendering failures. Books are bounded to 4096 chapters, 64 nested
/// chapter levels, 256 references, 16 MiB source and 64 MiB transformed content.
/// Each chapter is limited to the native document bound of 1 MiB, with no more
/// than 64 nested Markdown containers. The sole parser follows that specification.
/// A refusal returns no partially transformed book.
pub fn preprocess(plan: &Workplan<'_>, mut request: Value) -> Result<Value, DiagramError> {
    let pair = request
        .as_array_mut()
        .ok_or_else(|| refused("input must be the standard [context, book] array"))?;
    let [context, mut book] = <[Value; 2]>::try_from(std::mem::take(pair))
        .map_err(|_| refused("input must contain exactly context and book"))?;
    let context = object(&context, "context")?;
    if context.get("renderer").and_then(Value::as_str) != Some("html") {
        return Err(refused("renderer must be html"));
    }
    object(&book, "book")?;
    let sections = book
        .get_mut("sections")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| refused("book.sections must be an array"))?;
    chapters(plan, sections, 0, &mut Budget::default())?;
    Ok(book)
}

#[derive(Default)]
struct Budget {
    chapters: usize,
    references: usize,
    source_bytes: usize,
    output_bytes: usize,
}

fn refused(reason: &str) -> DiagramError {
    DiagramError::Refused {
        reason: format!("mdBook preprocessor: {reason}"),
    }
}

fn object<'a>(value: &'a Value, name: &str) -> Result<&'a Object, DiagramError> {
    let object = value
        .as_object()
        .ok_or_else(|| refused(&format!("{name} must be an object")))?;
    if let Some(key) = object.first_duplicate() {
        return Err(refused(&format!("{name} has duplicate member `{key}`")));
    }
    Ok(object)
}

fn chapters(
    plan: &Workplan<'_>,
    sections: &mut [Value],
    depth: usize,
    budget: &mut Budget,
) -> Result<(), DiagramError> {
    if depth >= MAX_DEPTH {
        return Err(refused("chapter nesting exceeds 64 levels"));
    }
    for item in sections {
        if item.as_str() == Some("Separator") {
            continue;
        }
        let section = object(item, "section")?;
        if section.len() != 1 {
            return Err(refused(
                "each section must name one standard BookItem variant",
            ));
        }
        if let Some(part) = section.get("PartTitle") {
            if part.as_str().is_none() {
                return Err(refused("PartTitle must be a string"));
            }
            continue;
        }
        let chapter = item.get_mut("Chapter").ok_or_else(|| {
            refused("unsupported BookItem variant; expected Chapter or PartTitle")
        })?;
        object(chapter, "chapter")?;
        budget.chapters += 1;
        if budget.chapters > MAX_CHAPTERS {
            return Err(refused("book exceeds 4096 chapters"));
        }
        let content = chapter
            .get_mut("content")
            .ok_or_else(|| refused("chapter.content is required"))?;
        let source = content
            .as_str()
            .ok_or_else(|| refused("chapter.content must be a string"))?;
        let transformed = markdown(plan, source, budget)?;
        *content = Value::String(transformed);
        let children = chapter
            .get_mut("sub_items")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| refused("chapter.sub_items must be an array"))?;
        if !children.is_empty() {
            chapters(plan, children, depth + 1, budget)?;
        }
    }
    Ok(())
}

fn markdown(
    plan: &Workplan<'_>,
    source: &str,
    budget: &mut Budget,
) -> Result<String, DiagramError> {
    budget.source_bytes = budget.source_bytes.saturating_add(source.len());
    if budget.source_bytes > MAX_SOURCE_BYTES {
        return Err(refused("book source exceeds 16 MiB"));
    }
    if source.len() > raf_okf::MAX_TEXT_BYTES {
        return Err(refused("chapter source exceeds 1 MiB"));
    }
    let spans = markdown_spans(source)?;
    let mut protected_index = 0;
    let mut paragraph_index = 0;
    let mut output = String::with_capacity(source.len());
    let mut line_start = 0;
    for line in source.split_inclusive('\n') {
        let original = line.trim_end_matches(['\r', '\n']);
        let mut replacement = None;
        for (offset, _) in line.match_indices("{{#workplan-diagram") {
            let offset = line_start + offset;
            if containing(&spans.protected, &mut protected_index, offset).is_some() {
                continue;
            }
            let paragraph = containing(&spans.paragraphs, &mut paragraph_index, offset)
                .ok_or_else(|| refused("diagram references require a top-level prose paragraph"))?;
            if source[paragraph.clone()].trim_end_matches(['\r', '\n']) != original {
                return Err(refused(
                    "diagram references must form their own top-level paragraph",
                ));
            }
            replacement = Some(fragment(plan, original, budget)?);
        }
        let text = replacement.as_deref().unwrap_or(line);
        budget.output_bytes = budget.output_bytes.saturating_add(text.len());
        if budget.output_bytes > MAX_OUTPUT_BYTES {
            return Err(refused("transformed book content exceeds 64 MiB"));
        }
        output.push_str(text);
        line_start += line.len();
    }
    Ok(output)
}

struct MarkdownSpans {
    protected: Vec<Range<usize>>,
    paragraphs: Vec<Range<usize>>,
}

/// The host's Markdown parser alone decides which bytes are literal content.
fn markdown_spans(source: &str) -> Result<MarkdownSpans, DiagramError> {
    let mut spans = MarkdownSpans {
        protected: Vec::new(),
        paragraphs: Vec::new(),
    };
    let mut depth = 0;
    for (event, range) in Parser::new(source).into_offset_iter() {
        match event {
            Event::Start(tag) => {
                if depth == MAX_DEPTH {
                    return Err(refused("Markdown nesting exceeds 64 containers"));
                }
                if tag == Tag::Paragraph && depth == 0 {
                    spans.paragraphs.push(range.clone());
                }
                if matches!(tag, Tag::CodeBlock(_) | Tag::HtmlBlock) {
                    spans.protected.push(range);
                }
                depth += 1;
            }
            Event::End(_) => depth -= 1,
            Event::Code(_) | Event::Html(_) | Event::InlineHtml(_) => {
                spans.protected.push(range);
            }
            _ => {}
        }
    }
    Ok(spans)
}

/// Parser ranges arrive in source order; advancing cursors avoids rescanning
/// the chapter for each directive, including literal examples of directives.
fn containing<'a>(
    spans: &'a [Range<usize>],
    cursor: &mut usize,
    byte: usize,
) -> Option<&'a Range<usize>> {
    while spans.get(*cursor).is_some_and(|span| span.end <= byte) {
        *cursor += 1;
    }
    spans.get(*cursor).filter(|span| span.contains(&byte))
}

fn fragment(
    plan: &Workplan<'_>,
    original: &str,
    budget: &mut Budget,
) -> Result<String, DiagramError> {
    let reference = original
        .strip_prefix(DIRECTIVE)
        .and_then(|text| text.strip_suffix("}}"))
        .filter(|text| !text.is_empty() && !text.contains(char::is_whitespace))
        .ok_or_else(|| refused("use exactly {{#workplan-diagram cap:slug#name}} or an arch reference on its own line"))?;
    budget.references += 1;
    if budget.references > MAX_REFERENCES {
        return Err(refused("book exceeds 256 diagram references"));
    }
    let bundle = plan.named_diagram(reference)?;
    static_fragment(&bundle).map_err(DiagramError::data)
}
