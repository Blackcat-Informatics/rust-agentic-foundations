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
        let directory = DocType::Architecture.directory();
        let findings: Vec<_> = self
            .findings()
            .iter()
            .filter(|finding| {
                finding.file == directory
                    || finding
                        .file
                        .strip_prefix(directory)
                        .is_some_and(|suffix| suffix.starts_with('/'))
                    || self.misplaced_architecture.contains(&finding.file)
            })
            .cloned()
            .collect();
        if !findings.is_empty() {
            return Err(CurrentDesignError { findings });
        }
        // Workplan has already checked both endpoints and cycles for these
        // declarations. Targets of historical sections remain historical.
        let historical = self
            .architecture()
            .iter()
            .flat_map(|page| &page.value().supersedes)
            .flat_map(raf_okf::DecisionSupersession::targets)
            .map(|target| (target.architecture(), target.section()))
            .collect();
        let mut output = String::from("# Current design\n");
        for page in self.architecture() {
            write_page(&mut output, page, &historical);
        }
        Ok(output)
    }
}

fn write_page(output: &mut String, page: &Loaded<Architecture>, historical: &Sections<'_>) {
    let body = page.body();
    let headings: Vec<_> = supersession::headings_with_authorship(body)
        .filter_map(|(heading, authored)| (authored && heading.level() <= 3).then_some(heading))
        .collect();
    let id = page.value().header.id.as_str();
    let mut text = PageText::new(body);
    let mut page_heading = false;
    for (index, decision) in headings
        .iter()
        .enumerate()
        .filter(|(_, heading)| heading.level() == 2 && heading.text() == DECISION)
    {
        let after = &headings[index + 1..];
        let boundary = after.iter().position(|heading| heading.level() <= 2);
        let named = &after[..boundary.unwrap_or(after.len())];
        let current = |section: &Heading| !historical.contains(&(id, section.anchor()));
        if !named.is_empty() && !named.iter().any(|section| current(section)) {
            continue;
        }
        if !page_heading {
            append(output, format_args!("\n## {id}\n"));
            page_heading = true;
        }
        let end = boundary.map_or(body.span().lines.end, |index| after[index].line());
        write_source(output, page.path(), decision);
        text.write(
            decision.line() + 1..named.first().map_or(end, |heading| heading.line()),
            output,
        );
        for (index, section) in named
            .iter()
            .enumerate()
            .filter(|(_, section)| current(section))
        {
            append(output, format_args!("\n### {}\n", section.text()));
            write_source(output, page.path(), section);
            let next = named.get(index + 1).map_or(end, |heading| heading.line());
            text.write(section.line() + 1..next, output);
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

    fn write(&mut self, range: Range<usize>, output: &mut String) {
        let mut started = false;
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
        }
    }
}
