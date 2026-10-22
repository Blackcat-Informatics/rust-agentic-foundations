// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Evidence records and claim markers for the fixtures that reconcile them.

use raf_source::Workspace;
use raf_workplan::digest::Digest;

use super::demo_workspace;

/// A digest no text of the fixtures has.
pub(crate) const OTHER_DIGEST: &str =
    "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

/// The digest of the symbol `path` of `source` as a record keeps it.
pub(crate) fn digest_of(source: &Workspace, path: &str) -> String {
    Digest::symbol(source, path)
        .unwrap_or_else(|error| panic!("{path} has a digest: {error}"))
        .to_string()
}

/// The digest of `demo::real` in the demo workspace.
pub(crate) fn real_digest() -> String {
    digest_of(&demo_workspace(), "demo::real")
}

/// The path of the evidence record `slug`.
pub(crate) fn evidence_path(slug: &str) -> String {
    format!("work/evidence/{slug}.md")
}

/// The line of a claim marker naming `slugs`, to follow a paragraph.
pub(crate) fn claim(slugs: &str) -> String {
    format!("<!-- raf:claim evidence={slugs} -->\n")
}

/// What an evidence record of the fixtures holds.
#[derive(Clone)]
pub(crate) struct Evidence {
    pub(crate) slug: &'static str,
    pub(crate) covers: &'static str,
    pub(crate) verdict: &'static str,
    /// The symbol inputs: path and digest.
    pub(crate) inputs: Vec<(String, String)>,
    pub(crate) values: Vec<(String, String)>,
    pub(crate) limits: Vec<(String, String)>,
}

impl Evidence {
    /// A passing record `slug` for `covers` (the text of the `for` list),
    /// made against `demo::real` as it is.
    pub(crate) fn passing(slug: &'static str, covers: &'static str) -> Self {
        Self {
            slug,
            covers,
            verdict: "pass",
            inputs: vec![("demo::real".to_string(), real_digest())],
            values: Vec::new(),
            limits: Vec::new(),
        }
    }

    pub(crate) fn verdict(mut self, verdict: &'static str) -> Self {
        self.verdict = verdict;
        self
    }

    pub(crate) fn inputs(mut self, inputs: Vec<(String, String)>) -> Self {
        self.inputs = inputs;
        self
    }

    pub(crate) fn values(mut self, values: &[(&str, &str)]) -> Self {
        self.values = owned(values);
        self
    }

    pub(crate) fn limits(mut self, limits: &[(&str, &str)]) -> Self {
        self.limits = owned(limits);
        self
    }

    /// The path of the record.
    pub(crate) fn path(&self) -> String {
        evidence_path(self.slug)
    }

    /// The text of the record.
    pub(crate) fn text(&self) -> String {
        let mut inputs = String::new();
        for (name, digest) in &self.inputs {
            inputs.push_str("  - kind: symbol\n    name: ");
            inputs.push_str(name);
            inputs.push_str("\n    digest: ");
            inputs.push_str(digest);
            inputs.push('\n');
        }
        let map = |key: &str, entries: &[(String, String)]| {
            let mut text = String::new();
            if !entries.is_empty() {
                text.push_str(key);
                text.push_str(":\n");
            }
            for (name, value) in entries {
                text.push_str("  ");
                text.push_str(name);
                text.push_str(": ");
                text.push_str(value);
                text.push('\n');
            }
            text
        };
        format!(
            "---\ntype: Evidence\nid: evidence:{}\nsubject: command\ninputs:\n{inputs}\
             verdict: {}\nby: agent\nfor: {}\ncommand: \"cargo bench\"\n{}{}---\n",
            self.slug,
            self.verdict,
            self.covers,
            map("values", &self.values),
            map("limits", &self.limits),
        )
    }
}

/// The pairs as owned text.
fn owned(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
        .collect()
}
