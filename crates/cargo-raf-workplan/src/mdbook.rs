// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The mdBook wire protocol, separate from canonical Workplan reports.

use std::io::{self, Read, Write};

use purrdf_lex::json::{self, Limits};
use raf_workplan::{Gate, diagram::book::preprocess};

use crate::{
    cli::{Args, MdbookArgs, MdbookCommand},
    command::workspace_root,
    exit::{Exit, Failure},
    render::Format,
};

const MAX_REQUEST_BYTES: usize = 32 * 1024 * 1024;
const MAX_RESPONSE_BYTES: usize = 128 * 1024 * 1024;

/// A support probe has no output; preprocessing returns the book alone.
pub(crate) enum Response {
    Supports(bool),
    Book(String),
}

impl Response {
    pub(crate) fn write(&self, out: &mut dyn Write) -> io::Result<()> {
        if let Self::Book(book) = self {
            writeln!(out, "{book}")?;
            out.flush()?;
        }
        Ok(())
    }

    pub(crate) fn exit(&self) -> Exit {
        match self {
            Self::Supports(false) => Exit::Findings,
            Self::Supports(true) | Self::Book(_) => Exit::Clean,
        }
    }
}

/// Complete parsing and rendering before emitting any protocol bytes.
pub(crate) fn run(args: &Args, book_args: &MdbookArgs) -> Result<Response, Failure> {
    if args.quiet || args.format != Format::Text {
        return Err(Failure::new(
            "docs mdbook uses its JSON protocol and refuses --quiet and non-text --format",
        ));
    }
    if let Some(MdbookCommand::Supports { renderer }) = &book_args.action {
        return Ok(Response::Supports(renderer == "html"));
    }
    let mut bytes = Vec::new();
    io::stdin()
        .lock()
        .take(u64::try_from(MAX_REQUEST_BYTES + 1)?)
        .read_to_end(&mut bytes)
        .map_err(|error| Failure::new(format!("cannot read the mdBook request: {error}")))?;
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(Failure::new("mdBook request exceeds 32 MiB"));
    }
    // A standard chapter adds three JSON containers per nesting level.
    // This admits the shared preprocessor's 64 chapter levels, with room for
    // the protocol envelope, and refuses duplicate keys throughout the request.
    let request = json::read_slice(
        &bytes,
        Limits {
            max_depth: 256,
            max_values: 1_000_000,
            max_string_bytes: MAX_REQUEST_BYTES,
            unique_members: true,
        },
    )
    .map_err(|error| Failure::new(format!("invalid mdBook request: {error}")))?;
    let root = workspace_root(args.root.as_deref())?;
    let opened = Gate::workspace(root).open()?;
    let book = preprocess(&opened.workplan(), request)?;
    let response = json::write_compact(&book);
    if response.len() > MAX_RESPONSE_BYTES {
        return Err(Failure::new("mdBook response exceeds 128 MiB"));
    }
    Ok(Response::Book(response))
}
