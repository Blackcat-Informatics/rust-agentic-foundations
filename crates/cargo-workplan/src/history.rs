// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `history <id> [--since <day>]`: `Workplan::history`, printed as a human table or a typed RDF graph.

use std::path::Path;

use raf_workplan::Gate;
use raf_workplan::exec::{Programs, SystemRunner};
use raf_workplan::history::{HistoryOptions, render};

use crate::cli::HistoryArgs;
use crate::exit::Failure;
use crate::render::{Class, Report};
use crate::result::Result as CommandResult;

/// `history`: the commits of the document or record, newest first. A history
/// is an outcome, so the run exits 0 when it has an answer and `-q` hides it.
/// It starts `git` only and publishes no document or record; Git may fetch
/// missing objects into its metadata in a partial clone.
pub(crate) fn history(root: &Path, args: &HistoryArgs) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let mut options = HistoryOptions::new();
    if let Some(since) = &args.since {
        options = options.with_since(since.clone());
    }
    let runner = SystemRunner::new(Programs::new(root));
    let history = plan.history(&args.id, &options, &runner)?;
    Ok(Report::block(
        Class::Outcome,
        render::table(&history),
        CommandResult::History(history),
    ))
}
