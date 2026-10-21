// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Normal form 1: a body is lexed into a token tree and rewritten by a fixed
//! tower of four pure passes; each level of the tower is a form, and a form is
//! hashed into a fingerprint.
//!
//! The levels, from the finest to the coarsest, are:
//!
//! 1. verbatim: the tokens as lexed, integers in their decimal spelling;
//! 2. shim: after `rename_params`, the function's own parameters renamed;
//! 3. literal tier: after `rename_locals`, every name the body binds and every
//!    parameter renamed, by first appearance;
//! 4. structural: after `kind_literals`, literals abstracted to their kind
//!    unless the value form is under the literal tier;
//! 5. value: after `erase_held`, the spellings that only change how a value is
//!    held dropped.
//!
//! Two bodies with the same parameter list that are equal at one level are equal
//! at every coarser one, and no pass adds a token. The verbatim form is the text
//! alone, so the same text under another parameter list can differ at the shim
//! level and be equal again from the literal tier.

mod binders;
mod fingerprint;
mod form;
mod literal;
mod passes;
mod shim;
mod words;

use std::fmt;

use crate::{CensusOptions, Error};

pub use fingerprint::{ENCODING, Fingerprint, ParseFingerprintError, RuleId, TableKind};
pub use form::{Form, MAX_NESTING, Token};
pub use passes::PASSES;
pub use words::{VALUE_ADAPTERS, WEAK_KEYWORDS};

pub(crate) use form::{Delim, Group, Kind, Node};
pub(crate) use literal::{kind as literal_kind, parse_integer};

/// A body whose value form has fewer tokens keeps its literals verbatim.
pub const LITERAL_TIER: usize = 30;

/// A body whose value form has fewer tokens has no shim form: below it every
/// `{ f(x) }` would be a group.
pub const MIN_SHIM_TOKENS: usize = 6;

/// A body whose value form has this many tokens or more has no shim form.
pub const SHIM_MAX_TOKENS: usize = 30;

/// A level of the tower, from the finest to the coarsest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// The tokens as lexed.
    Verbatim,
    /// The function's own parameters renamed.
    Shim,
    /// Every local name renamed, literals kept.
    LiteralTier,
    /// Literals abstracted to their kind unless the value form is under the tier.
    Structural,
    /// The value-holding spellings dropped.
    Value,
}

impl Level {
    /// Every level, finest first.
    pub const ALL: [Self; 5] = [
        Self::Verbatim,
        Self::Shim,
        Self::LiteralTier,
        Self::Structural,
        Self::Value,
    ];

    /// The id of the level, as the manifest and the page name it.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Verbatim => "verbatim",
            Self::Shim => "shim",
            Self::LiteralTier => "literal-tier",
            Self::Structural => "structural",
            Self::Value => "value",
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

/// A source slice to normalise: the text of a function body (the contents of
/// its braces) or of an initialiser, and the function's parameter names.
#[derive(Clone, Debug)]
pub struct Slice<'a> {
    file: &'a str,
    first_line: usize,
    text: &'a str,
    params: Vec<String>,
}

impl<'a> Slice<'a> {
    /// A slice of `file` whose first byte is on `first_line` (1-based), with no
    /// parameters.
    #[must_use]
    pub fn new(file: &'a str, first_line: usize, text: &'a str) -> Self {
        Self {
            file,
            first_line,
            text,
            params: Vec::new(),
        }
    }

    /// Sets the parameter names in signature order. `self` is not a parameter
    /// name and a repeated name counts once.
    #[must_use]
    pub fn params<I, S>(mut self, params: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.params.clear();
        for name in params {
            let name = name.into();
            if !words::is_keyword(&name) && !self.params.contains(&name) {
                self.params.push(name);
            }
        }
        self
    }
}

/// The five forms of one slice.
#[derive(Clone, Debug)]
pub struct Tower {
    forms: [Form; 5],
    value_tokens: usize,
    shim_candidate: bool,
}

impl Tower {
    /// Lexes `slice` and applies the passes in order.
    ///
    /// # Errors
    ///
    /// [`Error::Source`] when the slice does not lex or nests delimiters
    /// deeper than [`MAX_NESTING`].
    pub fn new(slice: &Slice<'_>) -> Result<Self, Error> {
        if u32::try_from(slice.text.len()).map_or(true, |len| len > u32::MAX / 2) {
            return Err(Error::Source {
                file: slice.file.to_owned(),
                line: slice.first_line,
                reason: "the source slice is larger than 2 GiB".to_owned(),
            });
        }
        let verbatim = Form::lex(slice.file, slice.first_line, slice.text)?;
        let shim_candidate = shim::candidate(&verbatim);
        let shim = passes::rename_params(&verbatim, &slice.params);
        let literal_tier = passes::rename_locals(&shim);
        let value_tokens = passes::held_len(&literal_tier);
        let structural = passes::kind_literals(&literal_tier, value_tokens < LITERAL_TIER);
        let value = passes::erase_held(&structural);
        Ok(Self {
            forms: [verbatim, shim, literal_tier, structural, value],
            value_tokens,
            shim_candidate,
        })
    }

    /// The form at `level`.
    #[must_use]
    pub fn form(&self, level: Level) -> &Form {
        &self.forms[level as usize]
    }

    /// Computes the retained and abstracted literal-value forms, in that order.
    ///
    /// Both start at the existing literal-tier form and apply the production
    /// literal-kind and held-value erasure passes. This on-demand introspection
    /// does not change selection policy or create candidate fingerprints.
    #[must_use]
    pub fn literal_values(&self) -> (Form, Form) {
        let source = self.form(Level::LiteralTier);
        let retained = passes::erase_held(&passes::kind_literals(source, true));
        let abstracted = passes::erase_held(&passes::kind_literals(source, false));
        (retained, abstracted)
    }

    /// The number of tokens of the value form: what every threshold counts. It
    /// is known before the literals are abstracted, so the literal tier is
    /// decided on it too.
    #[must_use]
    pub const fn value_tokens(&self) -> usize {
        self.value_tokens
    }

    /// Whether the body is a thin forwarder: one expression with a call and no
    /// `self`. It has a shim form only when it also has 6 to 29 value
    /// tokens.
    #[must_use]
    pub const fn shim_candidate(&self) -> bool {
        self.shim_candidate
    }

    /// Whether calls in this body go through a `Self::` callee path.
    ///
    /// Such a body may have a shim print but never supplies a grouping key.
    /// The query shares the production unit collector's call scan.
    #[must_use]
    pub fn calls_through_self(&self) -> bool {
        shim::calls_through_self(&self.form(Level::Verbatim).nodes)
    }

    /// The fingerprints this slice has under `min_tokens`.
    #[must_use]
    pub fn print(&self, min_tokens: usize) -> Print {
        let tokens = self.value_tokens();
        let named = |level: Level, make: fn(&[u8; 32]) -> Fingerprint| {
            let digest = fingerprint::digest(self.form(level));
            make(&digest)
        };
        let shim = (self.shim_candidate && (MIN_SHIM_TOKENS..SHIM_MAX_TOKENS).contains(&tokens))
            .then(|| named(Level::Shim, Fingerprint::shim));
        let body = (tokens >= min_tokens).then(|| named(Level::Value, Fingerprint::body));
        Print { tokens, body, shim }
    }
}

/// What the census learns from one slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Print {
    /// The tokens of the value form.
    pub tokens: usize,
    /// The `body` fingerprint of the value form, when the slice has at least
    /// `min_tokens` value tokens. A shim candidate has one too, but it
    /// never joins a body group: only its shim print groups.
    pub body: Option<Fingerprint>,
    /// The `shim` fingerprint of the shim form, when the slice is a shim
    /// candidate of 6 to 29 value tokens.
    pub shim: Option<Fingerprint>,
}

/// The first place two slices part ways.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenDiff {
    /// The index of the token in the flattened form.
    pub index: usize,
    /// The token of the first slice, `None` when its form ended.
    pub a: Option<Token>,
    /// The token of the second slice, `None` when its form ended.
    pub b: Option<Token>,
}

/// How two slices compare across the tower.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Divergence {
    /// The coarsest level at which the two forms still differ; `None` when
    /// they are equal at every level.
    pub differs_through: Option<Level>,
    /// The finest level from which every coarser level is equal too; `None`
    /// when the forms are distinct even in the value form.
    pub equal_from: Option<Level>,
    /// The first differing token of the forms at `differs_through`, with the
    /// source line of each side.
    pub first: Option<TokenDiff>,
}

/// The normaliser: the one entry point of the normal form.
#[derive(Clone, Copy, Debug)]
pub struct Normalizer {
    min_tokens: usize,
}

impl Normalizer {
    /// A normaliser with the options of a run.
    #[must_use]
    pub const fn new(options: &CensusOptions) -> Self {
        Self {
            min_tokens: options.min_tokens,
        }
    }

    /// The tower of a slice.
    ///
    /// # Errors
    ///
    /// [`Error::Source`] as [`Tower::new`].
    pub fn tower(&self, slice: &Slice<'_>) -> Result<Tower, Error> {
        Tower::new(slice)
    }

    /// The fingerprints of a slice.
    ///
    /// # Errors
    ///
    /// [`Error::Source`] as [`Tower::new`].
    pub fn print(&self, slice: &Slice<'_>) -> Result<Print, Error> {
        Ok(Tower::new(slice)?.print(self.min_tokens))
    }

    /// Compares two slices level by level and names where they part.
    ///
    /// # Errors
    ///
    /// [`Error::Source`] as [`Tower::new`], for either slice.
    pub fn explain(&self, a: &Slice<'_>, b: &Slice<'_>) -> Result<Divergence, Error> {
        let (a, b) = (Tower::new(a)?, Tower::new(b)?);
        let same = |level: Level| a.form(level).texts() == b.form(level).texts();
        let differs_through = Level::ALL.into_iter().rev().find(|level| !same(*level));
        let equal_from = Level::ALL
            .into_iter()
            .rev()
            .take_while(|level| same(*level))
            .last();
        let first = differs_through.and_then(|level| {
            let (left, right) = (a.form(level).tokens(), b.form(level).tokens());
            (0..left.len().max(right.len()))
                .find(|index| {
                    left.get(*index).map(|t| &t.text) != right.get(*index).map(|t| &t.text)
                })
                .map(|index| TokenDiff {
                    index,
                    a: left.get(index).cloned(),
                    b: right.get(index).cloned(),
                })
        });
        Ok(Divergence {
            differs_through,
            equal_from,
            first,
        })
    }
}

/// The pinned manifest of normal form 1: the encoding, the ordered passes, the
/// levels, the thresholds and the closed word lists. It is a golden beside the
/// fingerprints: any change to it is a new normal form.
#[must_use]
pub fn manifest() -> String {
    let levels: Vec<&str> = Level::ALL.iter().map(|level| level.id()).collect();
    format!(
        "encoding {ENCODING}\n\
         passes {}\n\
         levels {}\n\
         literal_tier {LITERAL_TIER}\n\
         min_shim_tokens {MIN_SHIM_TOKENS}\n\
         shim_max_tokens {SHIM_MAX_TOKENS}\n\
         max_nesting {MAX_NESTING}\n\
         value_adapters {}\n\
         weak_keywords {}\n",
        PASSES.join(" "),
        levels.join(" "),
        VALUE_ADAPTERS.join(" "),
        WEAK_KEYWORDS.join(" "),
    )
}
