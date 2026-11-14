// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Original analysis grants and discovery count limits.

/// Limits enforced by embedded collection and matching.
///
/// Load the Workspace with its own Source options before analysis. Report,
/// evidence, debt RDF operations, source registrations, root text and lines,
/// Source header preparation, original lexer buffers through cleanup and Syn
/// parser-input storage and the parsed syntax tree share these original grants.
/// The tree is charged by a bound measured over a reference table and checked
/// against the built tree; native parser work remains unmetered, and these
/// limits do not establish a total native heap or work guarantee.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct Options {
    /// Cumulative metered source preparation, analysis and run-bound RDF work.
    pub max_work: usize,
    /// Live admitted source preparation, analysis and RDF storage.
    pub max_memory_bytes: usize,
    /// Total indexed atoms.
    pub max_atoms: usize,
    /// Retained seed postings.
    pub max_postings: usize,
    /// Matching-hash unordered seed pairs, including failed candidates.
    pub max_seed_pairs: usize,
    /// Retained discovery candidates.
    pub max_candidates: usize,
    /// Syntax-region proposals.
    pub max_regions: usize,
    /// Verified pairs before containment suppression.
    pub max_verified_pairs: usize,
}

impl Default for Options {
    fn default() -> Self {
        let limits = super::policy::Limits::default();
        Self {
            max_work: 1 << 31,
            max_memory_bytes: 512 * 1024 * 1024,
            max_atoms: limits.atoms,
            max_postings: limits.postings,
            max_seed_pairs: limits.seed_pairs,
            max_candidates: limits.candidates,
            max_regions: limits.regions,
            max_verified_pairs: limits.verified,
        }
    }
}

impl Options {
    pub(super) const fn limits(self) -> super::policy::Limits {
        super::policy::Limits {
            atoms: self.max_atoms,
            postings: self.max_postings,
            seed_pairs: self.max_seed_pairs,
            candidates: self.max_candidates,
            regions: self.max_regions,
            verified: self.max_verified_pairs,
        }
    }
}
