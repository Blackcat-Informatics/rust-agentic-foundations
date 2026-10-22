// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The documents of the scenario fixtures: a capability document built from
//! its parts, the author's edit of a change record that `change new` wrote,
//! and the readers the assertions use (a section of a document, the links of
//! a document, the rows of a generated matrix).

use std::fmt::Write as _;

use raf_okf::Document;

use super::project::{COPYRIGHT, LICENCE};

/// The headings of a capability, in the order the profile fixes.
pub(crate) const HEADINGS: [&str; 7] = [
    "Purpose",
    "Contract",
    "Usage",
    "Examples",
    "Constraints",
    "Explanation",
    "Verification",
];

/// The index of the section `name` in [`HEADINGS`].
pub(crate) fn index_of(name: &str) -> usize {
    HEADINGS
        .iter()
        .position(|heading| heading.eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("`{name}` is not a section"))
}

/// What a capability binds: the text of each list, one path per entry.
#[derive(Clone, Copy)]
pub(crate) struct Bindings {
    pub(crate) code: &'static [&'static str],
    pub(crate) tests: &'static [&'static str],
    pub(crate) examples: &'static [&'static str],
}

/// The parts of a capability document.
pub(crate) struct Capability<'a> {
    /// The slug, `streaming-parser` for `cap:streaming-parser`.
    pub(crate) slug: &'a str,
    pub(crate) title: &'a str,
    pub(crate) description: &'a str,
    pub(crate) state: &'a str,
    pub(crate) bindings: Bindings,
    /// The text of each section, in the order of [`HEADINGS`], without the
    /// generated blocks of `Verification`.
    pub(crate) sections: [&'a str; 7],
    /// PROPOSED blocks (the marker line and the quoted lines, each ending in a
    /// line break), by the index of the section that holds them.
    pub(crate) blocks: Vec<(usize, String)>,
}

fn list(key: &str, entries: &[&str]) -> String {
    if entries.is_empty() {
        return format!("  {key}: []\n");
    }
    let mut text = format!("  {key}:\n");
    for entry in entries {
        let _ = writeln!(text, "    - {entry}");
    }
    text
}

/// The text of the capability document: the frontmatter, the title, the seven
/// sections, and the two generated blocks of `Verification` empty, for a sync
/// to fill.
pub(crate) fn capability(cap: &Capability<'_>) -> String {
    let Capability {
        slug,
        title,
        description,
        state,
        bindings,
        sections,
        blocks,
    } = cap;
    let mut text = format!(
        "---\n# {}{COPYRIGHT}\n# {}{LICENCE}\ntype: Capability\nid: cap:{slug}\ntitle: \"{title}\"\n\
         description: \"{description}\"\nstate: {state}\nbindings:\n",
        concat!("SPDX-", "FileCopyrightText: "),
        concat!("SPDX-", "License-Identifier: "),
    );
    text.push_str(&list("code", bindings.code));
    text.push_str(&list("tests", bindings.tests));
    text.push_str(&list("examples", bindings.examples));
    text.push_str("depends: []\n---\n\n");
    let _ = writeln!(text, "# {title}");
    for (at, heading) in HEADINGS.iter().enumerate() {
        let _ = writeln!(text, "\n## {heading}\n\n{}", sections[at].trim_end());
        for (section, block) in blocks {
            if *section == at {
                text.push('\n');
                text.push_str(block);
            }
        }
        if *heading == "Verification" {
            text.push_str(
                "\n<!-- raf:generated:verification-matrix -->\n<!-- raf:generated:end -->\n\
                 \n<!-- raf:generated:relationships -->\n<!-- raf:generated:end -->\n",
            );
        }
    }
    text
}

/// A PROPOSED block from the template `change new` printed: the placeholder
/// line of the template is replaced by the proposed text, each line quoted.
pub(crate) fn proposal(template: &str, text: &str) -> String {
    const PLACEHOLDER: &str = "> State the proposed text here.\n";
    assert!(
        template.contains(PLACEHOLDER),
        "the template has no placeholder line: {template}"
    );
    let mut quoted = String::new();
    for line in text.trim_end().lines() {
        let _ = writeln!(quoted, "> {line}");
    }
    template.replacen(PLACEHOLDER, &quoted, 1)
}

/// The change record with the title, the description, the summary and the items
/// an author writes: each item is a claim and, once it is settled, an outcome.
/// `written` is the record `change new` wrote, or an earlier version of this
/// edit.
pub(crate) fn author_record(
    written: &str,
    title: &str,
    description: &str,
    summary: &str,
    items: &[(&str, Option<&str>, Option<&str>)],
) -> String {
    const SUMMARY: &str = "## Summary\n\n";
    let (head, _) = written
        .split_once(SUMMARY)
        .unwrap_or_else(|| panic!("the record has no Summary: {written}"));
    let mut out = String::new();
    let mut lines = head.lines();
    while let Some(line) = lines.next() {
        if line.starts_with("title: ") {
            let _ = writeln!(out, "title: \"{title}\"");
        } else if line.starts_with("description: ") {
            let _ = writeln!(out, "description: \"{description}\"");
        } else if line == "items: []" || line == "items:" {
            if items.is_empty() {
                out.push_str("items: []\n");
            } else {
                out.push_str("items:\n");
                for (claim, outcome, reason) in items {
                    let _ = writeln!(out, "  - claim: \"{claim}\"");
                    if let Some(outcome) = outcome {
                        let _ = writeln!(out, "    outcome: {outcome}");
                    }
                    if let Some(reason) = reason {
                        let _ = writeln!(out, "    reason: \"{reason}\"");
                    }
                }
            }
            // The old items end at the closing fence.
            for next in lines.by_ref() {
                if next == "---" {
                    out.push_str("---\n");
                    break;
                }
            }
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str(SUMMARY);
    out.push_str(summary);
    out.push('\n');
    out
}

/// The text of the section `heading` of a document, without the heading.
pub(crate) fn section_text(document: &str, heading: &str) -> String {
    let parsed = Document::parse(document).unwrap_or_else(|error| panic!("reads: {error}"));
    let body = parsed.body();
    let section = body
        .sections()
        .iter()
        .find(|section| section.heading() == heading)
        .unwrap_or_else(|| panic!("no `{heading}` section"));
    let start = section.content().bytes.start - body.span().bytes.start;
    let end = section.content().bytes.end - body.span().bytes.start;
    body.text()[start..end].to_string()
}

/// The relative link targets of a document, outside fenced code and code
/// spans, in order: the text between `](` and `)`.
pub(crate) fn links_of(document: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut fenced = false;
    for line in document.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let mut rest = line;
        while let Some(start) = rest.find("](") {
            let after = &rest[start + 2..];
            let Some(end) = after.find(')') else { break };
            targets.push(after[..end].to_string());
            rest = &after[end + 1..];
        }
    }
    targets
}

/// The lines of the generated block of `kind` in a document, between its
/// markers.
pub(crate) fn generated_block(document: &str, kind: &str) -> Vec<String> {
    let open = format!("<!-- raf:generated:{kind} -->");
    let mut lines = document.lines().skip_while(|line| *line != open);
    assert!(lines.next().is_some(), "no `{kind}` block in the document");
    lines
        .take_while(|line| *line != "<!-- raf:generated:end -->")
        .map(str::to_string)
        .collect()
}
