// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `history <id> [--since <day>]`: `Workplan::history`, printed as a human table or a typed RDF graph.

use std::path::Path;

use raf_workplan::Gate;
use raf_workplan::git::Repository;
use raf_workplan::history::{HistoryOptions, render};

use crate::cli::HistoryArgs;
use crate::exit::Failure;
use crate::render::{Class, Report};
use crate::result::Result as CommandResult;

/// `history`: the commits of the document or record, newest first. A history
/// is an outcome, so the run exits 0 when it has an answer and `-q` hides it.
/// Reads repository objects natively, without fetching or changing Git state.
pub(crate) fn history(root: &Path, args: &HistoryArgs) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let mut options = HistoryOptions::new();
    if let Some(since) = &args.since {
        options = options.with_since(since.clone());
    }
    let repository = Repository::open(root)?;
    let history = plan.history(&args.id, &options, &repository)?;
    Ok(Report::block(
        Class::Outcome,
        render::table(&history),
        CommandResult::History(history),
    ))
}
