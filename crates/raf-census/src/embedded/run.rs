// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One explicit Workspace and the original account for its embedded analysis.

use super::{
    Candidate, Error, Options, OutputFailure, SyntaxExcess, Witness, debt, pairing, report,
    resource, workspace,
};

/// A successful static source-region analysis, independent of whole-body Census.
///
/// Report/debt RDF construction, parsing, validation and rendering charge the
/// original run account, as do Source lexing, parser input, span storage and the
/// parsed syntax tree; selected analysis limits do not establish a total native
/// heap or work guarantee.
pub struct Run<'source> {
    matches: pairing::Matches<'source>,
}

impl<'source> Run<'source> {
    /// Analyze the caller's explicitly loaded Workspace through the production
    /// collector, discovery and ordered verifier. Never runs consumer programs.
    ///
    /// # Errors
    /// Returns native Source errors, located profile refusals and exhausted
    /// analysis/count grants. No partially successful run is returned.
    pub fn analyze(
        source: &'source raf_source::Workspace,
        options: &Options,
    ) -> Result<Self, Error> {
        let analysis = workspace::collect(source, options.max_work, options.max_memory_bytes)?;
        Ok(Self {
            matches: pairing::find(analysis, options.limits())?,
        })
    }

    /// Candidate evidence from actual source functions; hashes alone prove nothing.
    pub fn candidates(&self) -> impl ExactSizeIterator<Item = Candidate<'_>> {
        self.matches
            .candidates
            .iter()
            .map(|value| Candidate::new(&self.matches, value))
    }
    /// Fresh verified physical region pairs after containment suppression.
    pub fn witnesses(&self) -> impl ExactSizeIterator<Item = Witness<'_>> {
        self.matches
            .witnesses
            .iter()
            .map(|value| Witness::new(&self.matches, value))
    }
    /// Syntax trees that held more than their prepaid bound, by file. Each excess
    /// was charged to the run account; an entry means the bound's measured terms
    /// were too low for that input. Empty for every tree within its bound.
    #[must_use]
    pub fn syntax_excesses(&self) -> &[SyntaxExcess<'source>] {
        &self.matches.analysis.excesses
    }
    /// Cumulative metered work and peak admitted run bytes.
    /// Includes admitted report, debt and imported-evidence operations. Source
    /// native lexing, parser input, span storage and syntax trees are charged to it.
    #[must_use]
    pub fn usage(&self) -> (usize, usize) {
        self.matches.analysis.meter().usage()
    }

    /// Produce the separate canonical embedded RDF evidence graph.
    /// The observer sees an unchanged diagnostic while its original grants live.
    pub fn report<'run, R>(
        &'run self,
        observe: impl for<'f> FnOnce(Result<Evidence<'run>, OutputFailure<'f>>) -> R,
    ) -> R {
        report::Claims::produce(&self.matches, |result| {
            observe(result.map(|claims| Evidence {
                claims,
                meter: self.matches.analysis.meter(),
            }))
        })
    }
    /// Import inert evidence claims; never constructs source witnesses or debt.
    /// The observer sees input-size, native RDF and domain refusals while their
    /// original owners live. Successful evidence does not borrow the input.
    pub fn import_evidence<'run, R>(
        &'run self,
        text: &str,
        format: raf_rdf::Format,
        observe: impl for<'f> FnOnce(Result<Evidence<'run>, OutputFailure<'f>>) -> R,
    ) -> R {
        report::Claims::from_text(text, format, self.matches.analysis.meter(), |result| {
            observe(result.map(|claims| Evidence {
                claims,
                meter: self.matches.analysis.meter(),
            }))
        })
    }
    /// Explicit initial allowance adoption. A publisher must require a new owned
    /// destination; existing debt is compared and shrunk instead of reset.
    /// Original source-identity, resource and exact-count refusals remain alive
    /// until the scoped observer has consumed their diagnostics.
    pub fn adopt_debt<'run, R>(
        &'run self,
        observe: impl for<'f> FnOnce(Result<Debt<'run>, OutputFailure<'f>>) -> R,
    ) -> R {
        match debt::Record::adopt(&self.matches) {
            Ok(record) => observe(Ok(Debt { record })),
            Err(cause) => observe(Err(OutputFailure::failure(&cause))),
        }
    }
    /// Import normative allowances into this run's original account, not proof.
    /// Native RDF, descriptor, count, input-size and resource refusals remain
    /// under their original owners during the scoped observation.
    pub fn import_debt<'run, R>(
        &'run self,
        text: &str,
        format: raf_rdf::Format,
        observe: impl for<'f> FnOnce(Result<Debt<'run>, OutputFailure<'f>>) -> R,
    ) -> R {
        debt::Record::from_text(text, format, self.matches.analysis.meter(), |result| {
            observe(result.map(|record| Debt { record }))
        })
    }
    /// Compare fresh verified pair counts with selected allowances.
    /// Observes another-run, growth and original-grant failures in their scope.
    pub fn compare_debt<'run, R>(
        &'run self,
        debt: &Debt<'run>,
        observe: impl for<'f> FnOnce(Result<Comparison, OutputFailure<'f>>) -> R,
    ) -> R {
        match debt.record.compare(&self.matches) {
            Ok(comparison) => observe(Ok(match comparison {
                debt::Comparison::Unchanged => Comparison::Unchanged,
                debt::Comparison::Shrunk { removed } => Comparison::Shrunk { removed },
            })),
            Err(cause) => observe(Err(OutputFailure::failure(&cause))),
        }
    }
    /// Produce exactly the fresh allowances only after the entire comparison
    /// establishes shrink-only behavior. This method writes no files.
    /// Observes another-run, growth and original-grant failures in their scope.
    pub fn shrink_debt<'run, R>(
        &'run self,
        debt: &Debt<'run>,
        observe: impl for<'f> FnOnce(Result<Debt<'run>, OutputFailure<'f>>) -> R,
    ) -> R {
        match debt.record.rewrite_existing(&self.matches) {
            Ok(record) => observe(Ok(Debt { record })),
            Err(cause) => observe(Err(OutputFailure::failure(&cause))),
        }
    }
}

/// Closed RDF evidence claims tied to their selected run; never source proof.
pub struct Evidence<'run> {
    claims: report::Claims<'run>,
    meter: &'run resource::Meter,
}
impl<'run> Evidence<'run> {
    /// Serialize through admitted canonical RDF rendering. Returned bytes retain
    /// their original run grant. Refusals remain under their original accounts
    /// until the observer has consumed their unchanged diagnostic.
    pub fn to_text<R>(
        &self,
        format: raf_rdf::Format,
        observe: impl for<'f> FnOnce(Result<Serialized<'run>, OutputFailure<'f>>) -> R,
    ) -> R {
        self.claims.to_text_admitted(format, self.meter, |result| {
            observe(result.map(|text| Serialized { text }))
        })
    }
}

/// Exact normative allowances owned by the selected run's original account.
pub struct Debt<'run> {
    record: debt::Record<'run>,
}
impl<'run> Debt<'run> {
    /// Serialize the independent embedded debt schema without writing files.
    /// Native RDF, descriptor, size and resource diagnostics stay under their
    /// original grants until observed.
    pub fn to_text<R>(
        &self,
        format: raf_rdf::Format,
        observe: impl for<'f> FnOnce(Result<Serialized<'run>, OutputFailure<'f>>) -> R,
    ) -> R {
        self.record.to_text(format, |result| {
            observe(result.map(|text| Serialized { text }))
        })
    }
}

/// Result of exact count comparison; shrinking must be published explicitly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    /// Every recorded count equals the fresh observed count.
    Unchanged,
    /// Existing allowances exceed fresh verified counts.
    Shrunk {
        /// Number of physical pair allowances removed.
        removed: usize,
    },
}

/// Serialized bytes retaining the original run account until destruction.
/// Borrow the text for publication; there is no payload-detaching operation.
pub struct Serialized<'run> {
    text: resource::Text<'run>,
}
impl Serialized<'_> {
    /// Canonical native RDF text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

#[cfg(test)]
impl<'source> Run<'source> {
    /// A run over an analysis that a test produced under explicit bound terms.
    pub(super) const fn from_matches(matches: pairing::Matches<'source>) -> Self {
        Self { matches }
    }
}
