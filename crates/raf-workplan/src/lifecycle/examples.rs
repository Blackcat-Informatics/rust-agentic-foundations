// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `example-failing`: a bound example target or documentation test of a
//! capability the change lists that does not build and run.

use raf_okf::CapabilitySection;

use crate::BindingKind;
use crate::bound::Executor;
use crate::exec::Runner;
use crate::lookup::capability_path;
use crate::obligation::{Needed, Obligation, ObligationKind, Part, Place, RunProblem, Subject};

use super::Impact;
use super::session::LifecycleError;

/// The `example-failing` obligations of a change: each binding of `examples`
/// of a listed capability that resolves in the source index is run through
/// the executor, and each that is not a passing run is owed.
///
/// Test bindings are not run here, and neither is a binding that does not
/// resolve: it is a `binding-missing` obligation already, and nothing could
/// be passed to `cargo` for it. A run that cannot be started is an error,
/// because it has no verdict.
pub(super) fn raise<R: Runner>(
    executor: &Executor<'_, R>,
    impact: &Impact,
) -> Result<Vec<Obligation>, LifecycleError> {
    let mut owed = Vec::new();
    for binding in impact
        .bindings
        .iter()
        .filter(|binding| binding.kind() == BindingKind::Example && binding.resolution().is_ok())
    {
        let evidence = executor.run(binding)?;
        let Some(problem) = RunProblem::of(&evidence) else {
            continue;
        };
        owed.push(Obligation {
            kind: ObligationKind::ExampleFailing,
            subject: Subject::Capability(binding.capability().to_string()),
            section: Part::Section(CapabilitySection::Examples),
            location: Place {
                path: capability_path(binding.capability()).unwrap_or_default(),
                line: None,
            },
            evidence: Needed::PassingRun {
                binding: binding.path().to_string(),
                problem,
            },
        });
    }
    Ok(owed)
}
