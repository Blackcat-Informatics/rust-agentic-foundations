// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Repository rules and their executable positive/negative preconditions.

use std::collections::BTreeSet;

use raf_source::{RelPath, TargetKind, parse_source};

use crate::{DEFAULT_MAX_FILE_BYTES, Error, FileCtx};

pub(crate) const BUILTIN_RULE_IDS: &[&str] = &[
    "std-default-hasher",
    "hex-format-loop",
    "hex-pair-radix",
    "hex-table",
    "hex-digit-radix",
    "home-literal",
    "sole-name",
    "vocabulary-literal",
    "json-pointer-escape",
    "length-prefix-frame",
    "le-slice-int",
    "align-up-mask",
    "div-ceil-by-hand",
    "xor-first-mismatch",
    "isomorphic",
];

/// One finding emitted by a repository rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleHit {
    /// The one-based source line of the finding.
    pub line: usize,
    /// An actionable explanation.
    pub message: String,
    /// The source-index path of the item that owns this finding.
    pub symbol: String,
}

/// Source snippets proving both what a rule flags and what it leaves alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleFixtures {
    /// Each snippet must produce at least one hit.
    pub positive: Vec<&'static str>,
    /// No snippet may produce a hit.
    pub negative: Vec<&'static str>,
}

/// A repository-specific structural check, executed on the bounded parse thread.
pub trait Rule: Sync {
    /// A lowercase kebab-case id, unique on the gate.
    fn id(&self) -> &'static str;
    /// Inspect one parsed source file and append findings.
    fn check(&self, ctx: &FileCtx, file: &syn::File, out: &mut Vec<RuleHit>);
    /// Positive and negative source cases that must hold before execution.
    fn fixtures(&self) -> RuleFixtures;
}

impl RuleFixtures {
    /// Run every fixture through the real rule and bounded source parser.
    ///
    /// The fixture context is a shipping library in package `fixture`, at
    /// `src/lib.rs`. Both sets are required and every emitted hit must have a
    /// positive line, nonempty message and nonempty owning symbol.
    ///
    /// # Errors
    /// Refuses missing cases, invalid Rust, an oversized fixture, a positive
    /// with no hit, a negative with a hit, or a malformed emitted hit.
    pub fn validate(&self, rule: &dyn Rule) -> Result<(), Error> {
        let id = rule.id();
        if self.positive.is_empty() || self.negative.is_empty() {
            return Err(refusal(
                id,
                "both positive and negative fixtures are required",
            ));
        }
        let file = RelPath::new("src/lib.rs").map_err(|error| refusal(id, &error.to_string()))?;
        let ctx = FileCtx {
            package: "fixture".into(),
            path: file.to_string(),
            shipping: true,
            target_kinds: BTreeSet::from([TargetKind::Lib]),
        };
        for (positive, cases) in [(true, &self.positive), (false, &self.negative)] {
            for (index, source) in cases.iter().enumerate() {
                if source.len() as u64 > DEFAULT_MAX_FILE_BYTES {
                    return Err(refusal(
                        id,
                        &format!("fixture {index} exceeds {DEFAULT_MAX_FILE_BYTES} bytes"),
                    ));
                }
                let hits = parse_source(&file, source, |syntax| {
                    let mut hits = Vec::new();
                    rule.check(&ctx, syntax, &mut hits);
                    hits
                })
                .map_err(|source| Error::Input {
                    file: format!(
                        "rule:{id}:{}-fixture:{index}",
                        if positive { "positive" } else { "negative" }
                    ),
                    source: Box::new(Error::Load(source)),
                })?;
                if hits.iter().any(|hit| {
                    hit.line == 0 || hit.symbol.trim().is_empty() || hit.message.trim().is_empty()
                }) {
                    return Err(refusal(
                        id,
                        &format!("fixture {index} emitted a malformed hit"),
                    ));
                }
                if positive == hits.is_empty() {
                    let kind = if positive {
                        "positive fixture produced no hit"
                    } else {
                        "negative fixture produced a hit"
                    };
                    return Err(refusal(id, &format!("{kind}: {index}")));
                }
            }
        }
        Ok(())
    }
}

fn refusal(id: &str, reason: &str) -> Error {
    Error::Source {
        file: format!("rule:{id}:fixtures"),
        line: 1,
        reason: reason.into(),
    }
}
