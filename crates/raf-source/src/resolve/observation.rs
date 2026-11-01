// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Optional observations from the ordinary failed native lookup, without a replay.

use super::{Index, Namespace, Resolution, ResolveError, Source, Steps, TargetId};
use crate::error::LoadError;
use crate::meter::{Meter, QueryObservation};

/// One canonical lookup and the actual indexed targets its missing routes read.
/// Target identities are local to this workspace load, not provider authority.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolutionAttempt {
    /// The ordinary resolved definition identity.
    Resolved(Resolution),
    /// No definition resolved. Targets are distinct, in first observation order;
    /// they include private import routes followed by the native resolver.
    NotFound {
        /// Targets whose module scopes the failed lookup actually read.
        attempted_targets: Vec<TargetId>,
    },
}

impl Index {
    pub(crate) fn resolve_observed(
        &self,
        path: &str,
        namespace: Namespace,
        meter: &Meter,
    ) -> Result<ResolutionAttempt, ResolveError> {
        let observation = QueryObservation::new(meter);
        let steps = Steps::Observed(&observation);
        let answer = self.resolve_counted(path, Some(namespace), None, steps);
        match self.query_checked(steps, answer) {
            Ok(resolution) => Ok(ResolutionAttempt::Resolved(resolution)),
            Err(ResolveError::NotFound(_)) => Ok(ResolutionAttempt::NotFound {
                attempted_targets: observation.finish(),
            }),
            Err(error) => Err(error),
        }
    }

    pub(super) fn record_failed_routes<'a>(
        &self,
        sources: impl Iterator<Item = &'a Source>,
        steps: Steps<'_>,
    ) -> Result<(), LoadError> {
        let Steps::Observed(observation) = steps else {
            return Ok(());
        };
        for source in sources {
            let (file, line) = self.module_site(source.module);
            observation.record(self.modules[source.module].target, &file, line)?;
        }
        Ok(())
    }
}
