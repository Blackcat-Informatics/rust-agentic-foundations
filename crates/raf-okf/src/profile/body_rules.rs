// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The rules about the body of a document, and the placeholder rule.

use crate::body::{Body, Section};
use crate::profile::schema::DocType;
use crate::profile::typed::CapabilitySection;
use crate::profile::violation::{Location, Rule, Violation};

/// Unfinished-work tokens that may not appear outside code, lowercase and
/// matched without regard to case, spacing or dots between letters. The text
/// is assembled with `concat!` so this list never holds a token as one
/// literal.
pub(crate) const PLACEHOLDER_TOKENS: [&str; 5] = [
    concat!("t", "bd"),
    concat!("to", "do"),
    concat!("fix", "me"),
    concat!("x", "xx"),
    concat!("to be ", "written"),
];

/// Remove HTML comments, including multi-line ones. A comment that is never
/// closed removes the rest of the text.
fn strip_html_comments(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 4..];
        match after.find("-->") {
            Some(end) => rest = &after[end + 3..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// The content of a section with generated blocks and HTML comments removed.
fn visible_content(body: &Body, section: &Section) -> String {
    let base = body.span().bytes.start;
    let content = &section.content().bytes;
    let text = body.text();
    let mut kept = String::new();
    let mut position = content.start;
    for block in body.generated_blocks() {
        let outer = &block.outer().bytes;
        if outer.start >= content.start && outer.end <= content.end {
            kept.push_str(&text[position - base..outer.start - base]);
            position = outer.end;
        }
    }
    kept.push_str(&text[position - base..content.end - base]);
    strip_html_comments(&kept)
}

fn section_is_empty(body: &Body, section: &Section) -> bool {
    visible_content(body, section).trim().is_empty()
}

/// Rules about the second-level sections and the body as a whole.
pub(crate) fn check_sections(doc_type: DocType, body: &Body, file: &str, out: &mut Vec<Violation>) {
    let mut push = |at: Location, rule: Rule| {
        out.push(Violation {
            file: file.to_string(),
            at,
            rule,
        });
    };
    match doc_type {
        DocType::Capability => {
            let wanted: Vec<&str> = CapabilitySection::ALL
                .iter()
                .map(|section| section.heading())
                .collect();
            let found = body.sections();
            let names: Vec<&str> = found.iter().map(Section::heading).collect();
            for expected in &wanted {
                if !names.contains(expected) {
                    push(
                        Location::Section((*expected).to_string()),
                        Rule::SectionMissing,
                    );
                }
            }
            for name in &names {
                if !wanted.contains(name) {
                    push(
                        Location::Section((*name).to_string()),
                        Rule::SectionUnexpected,
                    );
                }
            }
            let known: Vec<&str> = names
                .iter()
                .copied()
                .filter(|name| wanted.contains(name))
                .collect();
            let ordered: Vec<&str> = wanted
                .iter()
                .copied()
                .filter(|name| known.contains(name))
                .collect();
            if known.len() != ordered.len() {
                push(Location::Document, Rule::SectionDuplicated);
            } else if known != ordered {
                push(
                    Location::Document,
                    Rule::SectionsOutOfOrder {
                        found: known.iter().map(|name| (*name).to_string()).collect(),
                    },
                );
            }
            for section in found {
                if wanted.contains(&section.heading()) && section_is_empty(body, section) {
                    push(
                        Location::Section(section.heading().to_string()),
                        Rule::SectionEmpty,
                    );
                }
            }
        }
        DocType::Change => {
            let summary = body
                .sections()
                .iter()
                .find(|section| section.heading() == "Summary");
            if let Some(section) = summary {
                if section_is_empty(body, section) {
                    push(Location::Section("Summary".to_string()), Rule::SectionEmpty);
                }
            } else {
                push(
                    Location::Section("Summary".to_string()),
                    Rule::SectionMissing,
                );
            }
        }
        DocType::Architecture => {
            if strip_html_comments(body.text()).trim().is_empty() {
                push(Location::Document, Rule::BodyEmpty);
            }
        }
    }
}

/// Rules about PROPOSED blocks and split links.
pub(crate) fn check_blocks(doc_type: DocType, body: &Body, file: &str, out: &mut Vec<Violation>) {
    for block in body.proposed_blocks() {
        let at = Location::Line(block.marker().lines.start);
        let rule = if doc_type != DocType::Capability {
            Some(Rule::ProposedOutsideCapability {
                slug: block.slug().to_string(),
            })
        } else if block.section().is_none() {
            Some(Rule::ProposedOutsideSection {
                slug: block.slug().to_string(),
            })
        } else {
            None
        };
        if let Some(rule) = rule {
            out.push(Violation {
                file: file.to_string(),
                at,
                rule,
            });
        }
    }
    for link in body.split_links() {
        out.push(Violation {
            file: file.to_string(),
            at: Location::Line(link.line()),
            rule: Rule::SplitLink { form: link.form() },
        });
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// True when `token` occurs in `line` as a whole word.
fn has_word(line: &str, token: &str) -> bool {
    line.match_indices(token).any(|(index, _)| {
        let before = line[..index].chars().next_back();
        let after = line[index + token.len()..].chars().next();
        !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
    })
}

/// Lowercase text with whitespace runs collapsed to one space and the dots
/// between letters removed, so `T.B.D.` and `to  be\twritten` normalise to the
/// plain token.
fn normalise(text: &str) -> String {
    let collapsed = text
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let chars: Vec<char> = collapsed.chars().collect();
    let mut out = String::with_capacity(collapsed.len());
    for (index, c) in chars.iter().enumerate() {
        let letter_before = index
            .checked_sub(1)
            .is_some_and(|before| chars[before].is_alphabetic());
        let letter_after = chars
            .get(index + 1)
            .is_some_and(|next| next.is_alphabetic());
        if !(*c == '.' && letter_before && letter_after) {
            out.push(*c);
        }
    }
    out
}

/// The placeholder tokens in `text`, in the order of the token list.
pub(crate) fn placeholders(text: &str) -> Vec<&'static str> {
    let prose = normalise(text);
    PLACEHOLDER_TOKENS
        .iter()
        .copied()
        .filter(|token| has_word(&prose, token))
        .collect()
}

/// The placeholder rule over the prose lines of the body.
pub(crate) fn check_placeholders(body: &Body, file: &str, out: &mut Vec<Violation>) {
    for (line, token) in body.placeholder_lines() {
        out.push(Violation {
            file: file.to_string(),
            at: Location::Line(line),
            rule: Rule::Placeholder {
                token: token.to_string(),
            },
        });
    }
}
