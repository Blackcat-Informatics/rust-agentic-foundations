// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A read-only view of the active Architecture decision text.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Write as _};
use std::ops::Range;

use raf_okf::{Architecture, Body, DocType, GeneratedBlock, Heading};

use super::{DECISION, supersession};
use crate::lookup::write_findings;
use crate::{Finding, Loaded, Workplan};

/// The Architecture inputs do not support a complete current-design view.
#[derive(Debug)]
pub struct CurrentDesignError {
    /// The existing Workplan findings that identify the refused input.
    pub findings: Vec<Finding>,
}

impl fmt::Display for CurrentDesignError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("fix the Architecture findings before rendering the current design")?;
        write_findings(formatter, &self.findings)
    }
}

impl Error for CurrentDesignError {}

type Sections<'a> = BTreeSet<(&'a str, &'a str)>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Render {
    WithLocations,
    AuthoredText,
}

impl Workplan<'_> {
    /// Render the current Architecture decisions as Markdown in document and
    /// heading order. Each excerpt identifies its original path and heading.
    ///
    /// A named section remains current until any admitted section supersedes
    /// it. Unsectioned decisions remain current. Introductory text accompanies
    /// a decision block that has a current section. Excerpts preserve authored
    /// Markdown, including nested headings and code, inside separate quotes.
    ///
    /// # Errors
    /// Refuses Architecture findings from document loading and invalid
    /// supersession declarations before returning the Markdown.
    pub fn current_design(&self) -> Result<String, CurrentDesignError> {
        let findings = architecture_findings(self);
        if !findings.is_empty() {
            return Err(CurrentDesignError { findings });
        }
        let historical = supersession::historical(self);
        let mut output = String::from("# Current design\n");
        for page in self.architecture() {
            let headings = authored_headings(page.body());
            write_page(
                &mut output,
                page,
                &headings,
                &historical,
                Render::WithLocations,
            );
        }
        Ok(output)
    }
}

pub(crate) fn architecture_findings(plan: &Workplan<'_>) -> Vec<Finding> {
    let directory = DocType::Architecture.directory();
    plan.findings()
        .iter()
        .filter(|finding| {
            finding.file == directory
                || finding
                    .file
                    .strip_prefix(directory)
                    .is_some_and(|suffix| suffix.starts_with('/'))
                || plan.misplaced_architecture.contains(&finding.file)
        })
        .cloned()
        .collect()
}

pub(crate) fn authored_headings(body: &Body) -> Vec<&Heading> {
    supersession::headings_with_authorship(body)
        .filter_map(|(heading, authored)| (authored && heading.level() <= 3).then_some(heading))
        .collect()
}

pub(crate) struct DecisionBlock<'a, 'h> {
    pub(crate) heading: &'h Heading,
    pub(crate) sections: &'a [&'h Heading],
    pub(crate) end: usize,
}

pub(crate) fn decision_blocks<'a, 'h>(
    headings: &'a [&'h Heading],
    body_end: usize,
) -> impl Iterator<Item = DecisionBlock<'a, 'h>> {
    headings
        .iter()
        .enumerate()
        .filter(|(_, heading)| heading.level() == 2 && heading.text() == DECISION)
        .map(move |(index, &heading)| {
            let after = &headings[index + 1..];
            let boundary = after.iter().position(|heading| heading.level() <= 2);
            DecisionBlock {
                heading,
                sections: &after[..boundary.unwrap_or(after.len())],
                end: boundary.map_or(body_end, |index| after[index].line()),
            }
        })
}

pub(crate) fn authored_decision_text(
    page: &Loaded<Architecture>,
    headings: &[&Heading],
    historical: &Sections<'_>,
) -> String {
    let mut output = String::new();
    write_page(
        &mut output,
        page,
        headings,
        historical,
        Render::AuthoredText,
    );
    output
}

/// Normalize significant authored lines for exact retained-text checks.
pub(crate) fn retirement_prose(
    body: &Body,
    ranges: impl Iterator<Item = Range<usize>>,
) -> impl Iterator<Item = String> {
    let mut page = PageText::new(body);
    ranges.map(move |range| normalized_prose(&mut page, range))
}

fn normalized_prose(page: &mut PageText<'_>, lines: Range<usize>) -> String {
    let mut quoted = String::new();
    page.write(lines, &mut quoted, Render::AuthoredText);
    let mut prose = String::from("\n");
    for line in quoted
        .lines()
        .filter_map(|line| line.strip_prefix("> "))
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        prose.push_str(line);
        prose.push('\n');
    }
    prose
}

fn write_page(
    output: &mut String,
    page: &Loaded<Architecture>,
    headings: &[&Heading],
    historical: &Sections<'_>,
    render: Render,
) {
    let body = page.body();
    let id = page.value().header.id.as_str();
    let mut text = PageText::new(body);
    let mut page_heading = false;
    for block in decision_blocks(headings, body.span().lines.end) {
        let named = block.sections;
        let current = |section: &Heading| !historical.contains(&(id, section.anchor()));
        if !named.is_empty() && !named.iter().any(|section| current(section)) {
            continue;
        }
        if !page_heading {
            append(output, format_args!("\n## {id}\n"));
            page_heading = true;
        }
        let end = block.end;
        if render == Render::WithLocations {
            write_source(output, page.path(), block.heading);
        } else {
            output.push_str("\n## Decision\n");
        }
        text.write(
            block.heading.line() + 1..named.first().map_or(end, |heading| heading.line()),
            output,
            render,
        );
        for (index, section) in named
            .iter()
            .enumerate()
            .filter(|(_, section)| current(section))
        {
            append(output, format_args!("\n### {}\n", section.text()));
            if render == Render::WithLocations {
                write_source(output, page.path(), section);
            }
            let next = named.get(index + 1).map_or(end, |heading| heading.line());
            text.write(section.line() + 1..next, output, render);
        }
    }
}

fn write_source(output: &mut String, path: &str, heading: &Heading) {
    append(
        output,
        format_args!(
            "\nSource `{path}#{}` on line {}.\n",
            heading.anchor(),
            heading.line()
        ),
    );
}

#[expect(
    clippy::expect_used,
    reason = "formatting never fails when the destination is a String"
)]
fn append(output: &mut String, args: fmt::Arguments<'_>) {
    output
        .write_fmt(args)
        .expect("INVARIANT: writing to String cannot fail");
}

struct PageText<'a> {
    lines: std::iter::Peekable<std::iter::Enumerate<std::str::SplitInclusive<'a, char>>>,
    start: usize,
    generated: std::iter::Peekable<std::slice::Iter<'a, GeneratedBlock>>,
}

impl<'a> PageText<'a> {
    fn new(body: &'a Body) -> Self {
        Self {
            lines: body.text().split_inclusive('\n').enumerate().peekable(),
            start: body.span().lines.start,
            generated: body.generated_blocks().iter().peekable(),
        }
    }

    fn write(&mut self, range: Range<usize>, output: &mut String, render: Render) {
        let mut started = false;
        let mut last_text = output.len();
        while let Some(&(offset, raw)) = self.lines.peek() {
            let line = self.start + offset;
            if line >= range.end {
                break;
            }
            self.lines.next();
            if line < range.start {
                continue;
            }
            while self
                .generated
                .peek()
                .is_some_and(|block| block.outer().lines.end <= line)
            {
                self.generated.next();
            }
            let raw = raw.trim_end_matches(['\r', '\n']);
            if let Some(block) = self
                .generated
                .peek()
                .filter(|block| block.outer().lines.contains(&line))
            {
                if started && block.outer().lines.start == line {
                    output.push_str(">\n");
                }
                continue;
            }
            if !started && raw.trim().is_empty() {
                continue;
            }
            if !started {
                output.push('\n');
                started = true;
            }
            output.push_str("> ");
            output.push_str(raw);
            output.push('\n');
            if !raw.trim().is_empty() {
                last_text = output.len();
            }
        }
        if render == Render::AuthoredText {
            output.truncate(last_text);
        }
    }
}
