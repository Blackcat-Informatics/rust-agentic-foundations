// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Owned domain results. Human text and machine projection share these values.

use raf_workplan::assessment::{AssessContext, Assessed, AssessmentState};
use raf_workplan::bound::Evidence;
use raf_workplan::coverage::{CoverageReport, GapKind, lines::LinesReport};
use raf_workplan::debt::DebtWrite;
use raf_workplan::evidence::{ClaimStatement, Recorded, Support};
use raf_workplan::history::History;
use raf_workplan::lifecycle::{Failure, Impact, NewChange};
use raf_workplan::obligation::Obligation;
use raf_workplan::strict::ClaimsReport;
use raf_workplan::{Finding, OpenChange, RoadmapEntry};

/// A closed command result; nested values retain their domain types and order.
#[derive(Debug, Clone)]
pub(crate) enum Result {
    Finding(Finding),
    State {
        state: String,
        capabilities: Vec<RoadmapEntry>,
    },
    OpenChanges(Vec<OpenChange>),
    Rewritten {
        path: String,
    },
    Preview {
        path: String,
        capability: String,
        text: String,
    },
    NewChange(NewChange),
    Impact(Impact),
    Obligation(Obligation),
    Completed {
        path: String,
    },
    Refused {
        failures: usize,
        message: String,
    },
    Refusal {
        failure: Failure,
        repeated: bool,
    },
    Binding {
        evidence: Evidence,
        message: String,
    },
    EvidenceRecorded(Recorded),
    EvidenceClaim {
        evidence: String,
        claim: ClaimStatement,
    },
    EvidenceSupport {
        capability: String,
        support: Support,
    },
    AssessmentContext(AssessContext),
    AssessmentRecorded(Assessed),
    Assessment(AssessmentState),
    History(History),
    Coverage {
        report: CoverageReport,
        direction: GapKind,
    },
    Claims(ClaimsReport),
    Lines(LinesReport),
    DebtWritten(DebtWrite),
}
