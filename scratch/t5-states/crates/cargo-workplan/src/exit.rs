// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The exit codes and the failure that ends a run with the third.

use std::error::Error;
use std::fmt;
use std::process::ExitCode;

/// How a run ended: the three exit codes of the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Exit {
    /// Exit 0: nothing to report, or the command did its work.
    Clean,
    /// Exit 1: the run printed findings.
    Findings,
    /// Exit 2: the tool could not run.
    Failed,
}

impl Exit {
    /// The code a process exits with.
    pub(crate) fn code(self) -> u8 {
        match self {
            Self::Clean => 0,
            Self::Findings => 1,
            Self::Failed => 2,
        }
    }
}

impl From<Exit> for ExitCode {
    fn from(exit: Exit) -> Self {
        Self::from(exit.code())
    }
}

/// The tool could not run: a message for standard error and exit 2.
///
/// Every error of the library converts into it by its `Display` text, so a
/// command reports what the library said and adds nothing of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Failure(String);

impl Failure {
    /// A failure with this message.
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<E: Error> From<E> for Failure {
    fn from(error: E) -> Self {
        Self(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_exits_have_the_documented_codes() {
        assert_eq!(Exit::Clean.code(), 0);
        assert_eq!(Exit::Findings.code(), 1);
        assert_eq!(Exit::Failed.code(), 2);
    }

    #[test]
    fn a_library_error_becomes_its_display_text() {
        let error = std::io::Error::other("the disk is gone");
        assert_eq!(Failure::from(error).to_string(), "the disk is gone");
    }
}
