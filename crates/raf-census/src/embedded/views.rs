// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Borrowed original-source observations from one successful run.

use super::{analysis, pairing};
use std::ops::{Range, RangeInclusive};

/// An actual physical function with every indexed owner retained.
pub struct Function<'run> {
    workspace: &'run raf_source::Workspace,
    file: &'run raf_source::SourceFile,
    owners: &'run [&'run raf_source::Item],
    body: Range<usize>,
}

impl<'run> Function<'run> {
    fn new(input: &'run analysis::Function<'_>) -> Self {
        let (workspace, file, owners) = match &input.origin {
            analysis::Origin::Source {
                workspace,
                file,
                owners,
                ..
            } => (*workspace, *file, owners.as_slice()),
            // Run::analyze only calls the production Workspace collector, which
            // constructs Source origins. Text is confined to private fixtures.
            #[cfg(test)]
            analysis::Origin::Text => unreachable!("public runs require Source provenance"),
        };
        Self {
            workspace,
            file,
            owners,
            body: input.facts.body.start..input.facts.body.end,
        }
    }

    /// The explicit Workspace supplied to the run.
    #[must_use]
    pub const fn workspace(&self) -> &'run raf_source::Workspace {
        self.workspace
    }
    /// Actual Source record, including its authority and original text.
    #[must_use]
    pub const fn file(&self) -> &'run raf_source::SourceFile {
        self.file
    }
    /// Every actual indexed owner of this physical body.
    #[must_use]
    pub const fn owners(&self) -> &'run [&'run raf_source::Item] {
        self.owners
    }
    /// Original, half-open body byte range.
    #[must_use]
    pub fn body(&self) -> Range<usize> {
        self.body.clone()
    }
}

/// A discovery seed anchored in one actual function's atom sequence.
pub struct Seed<'run> {
    function: Function<'run>,
    atom: usize,
}
impl Seed<'_> {
    /// Actual source function.
    #[must_use]
    pub const fn function(&self) -> &Function<'_> {
        &self.function
    }
    /// First atom of the fixed-size discovery gram.
    #[must_use]
    pub const fn atom(&self) -> usize {
        self.atom
    }
}

/// Coarse seed evidence; a matching hash is never a verified clone.
pub struct Candidate<'run> {
    left: Seed<'run>,
    right: Seed<'run>,
    digest: &'run [u8; 32],
    alignment: Option<(Range<usize>, Range<usize>)>,
}
impl<'run> Candidate<'run> {
    pub(super) fn new(
        matches: &'run pairing::Matches<'_>,
        value: &'run pairing::Candidate,
    ) -> Self {
        let seed = |anchor: pairing::Anchor| Seed {
            function: Function::new(&matches.analysis.functions()[anchor.function]),
            atom: anchor.atom,
        };
        Self {
            left: seed(value.left),
            right: seed(value.right),
            digest: &value.seed_digest,
            alignment: value.alignment.map(|alignment| {
                (
                    alignment.left.atom..alignment.left.atom + alignment.len,
                    alignment.right.atom..alignment.right.atom + alignment.len,
                )
            }),
        }
    }
    /// Left seed in production discovery order.
    #[must_use]
    pub const fn left(&self) -> &Seed<'run> {
        &self.left
    }
    /// Right seed in production discovery order.
    #[must_use]
    pub const fn right(&self) -> &Seed<'run> {
        &self.right
    }
    /// Full coarse discovery digest, distinct from verified proof identity.
    #[must_use]
    pub const fn seed_digest(&self) -> &'run [u8; 32] {
        self.digest
    }
    /// Maximal exact coarse alignment, absent for a hash collision.
    #[must_use]
    pub fn alignment(&self) -> Option<(Range<usize>, Range<usize>)> {
        self.alignment.clone()
    }
}

/// A complete admitted original-source syntax region.
pub struct Region<'run> {
    function: Function<'run>,
    site: pairing::Site,
}
impl Region<'_> {
    /// Physical function and its complete actual owner set.
    #[must_use]
    pub const fn function(&self) -> &Function<'_> {
        &self.function
    }
    /// Original half-open byte range, validated by production region admission.
    #[must_use]
    pub const fn bytes(&self) -> Range<usize> {
        self.site.bytes.start..self.site.bytes.end
    }
    /// Inclusive original line range.
    #[must_use]
    pub const fn lines(&self) -> RangeInclusive<usize> {
        self.site.first_line..=self.site.last_line
    }
    /// Half-open atom range within the owning function.
    #[must_use]
    pub const fn atoms(&self) -> Range<usize> {
        self.site.atom_start..self.site.atom_end
    }
    /// Original source slice; inspected as data only.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.function.file.text[self.bytes()]
    }
}

/// A pair established by fresh ordered lexical and binding proof.
pub struct Witness<'run> {
    left: Region<'run>,
    right: Region<'run>,
    digest: &'run [u8; 32],
    candidate: Candidate<'run>,
}
impl<'run> Witness<'run> {
    pub(super) fn new(matches: &'run pairing::Matches<'_>, value: &'run pairing::Witness) -> Self {
        let region = |site: pairing::Site| Region {
            function: Function::new(&matches.analysis.functions()[site.function]),
            site,
        };
        Self {
            left: region(value.left),
            right: region(value.right),
            digest: &value.digest,
            candidate: Candidate::new(matches, &matches.candidates[value.candidate]),
        }
    }
    /// Left verified original-source region.
    #[must_use]
    pub const fn left(&self) -> &Region<'run> {
        &self.left
    }
    /// Right verified original-source region.
    #[must_use]
    pub const fn right(&self) -> &Region<'run> {
        &self.right
    }
    /// Versioned ordered lexical proof identity.
    #[must_use]
    pub const fn digest(&self) -> &'run [u8; 32] {
        self.digest
    }
    /// Actual discovery candidate used by production verification.
    /// Canonical RDF independently chooses stable supporting provenance.
    #[must_use]
    pub const fn candidate(&self) -> &Candidate<'run> {
        &self.candidate
    }
}
