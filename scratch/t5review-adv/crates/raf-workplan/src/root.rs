// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Root discovery: the workspace root of the Cargo workspace the runner's
//! directory is in.
//!
//! [`discover`] starts exactly one process, through the runner:
//!
//! ```text
//! cargo metadata --format-version 1 --no-deps --frozen
//! ```
//!
//! `--no-deps` skips dependency resolution, which finding the root does not
//! need, and `--frozen` forbids network access and any change to `Cargo.lock`.
//! The output is bounded at 64 MiB ([`METADATA_OUTPUT_BOUND`](crate::exec::METADATA_OUTPUT_BOUND)) and the
//! `workspace_root` member of its top-level object is the only thing read.
//! `raf-workplan` takes no JSON dependency, so a small scanner validates the
//! text as JSON and finds that one member; a text that is not JSON, or whose
//! top-level object does not have the member exactly once as a string, is an
//! error and not a guess (see [`RootError`]).
//!
//! No value from a document is placed on the command line. Cargo reads the
//! workspace's `.cargo/config.toml`, and the rustup proxy reads its
//! `rust-toolchain.toml`, so the workspace is trusted build input, as it is for
//! `cargo build`; cargo does not run the `build.rustc` or `build.rustc-wrapper`
//! program of that configuration for this command.

mod json;

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::exec::{ExecError, Invocation, Limits, Runner};

use json::JsonError;

/// Why the workspace root cannot be found.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RootError {
    /// `cargo` did not run to a result: it could not be started, ran too long,
    /// or wrote more than the bound.
    Exec(ExecError),
    /// `cargo metadata` ran and failed, as it does outside a Cargo workspace.
    Failed {
        /// The exit code, or `None` when a signal ended the process.
        code: Option<i32>,
        /// What it wrote to standard error.
        stderr: String,
    },
    /// The output is not UTF-8, so it is not JSON.
    NotUtf8,
    /// The output is not the JSON object the field is read from.
    Json(String),
    /// The field is not an absolute path.
    NotAbsolute {
        /// The value.
        value: String,
    },
}

impl fmt::Display for RootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exec(error) => write!(f, "{error}"),
            Self::Failed { code, stderr } => {
                write!(
                    f,
                    "`cargo metadata --format-version 1 --no-deps --frozen` failed"
                )?;
                match code {
                    Some(code) => write!(f, " (exit {code})")?,
                    None => write!(f, " (ended by a signal)")?,
                }
                if !stderr.is_empty() {
                    write!(f, ": {stderr}")?;
                }
                write!(
                    f,
                    "; run it inside a Cargo workspace, or name the workspace root directly"
                )
            }
            Self::NotUtf8 => write!(
                f,
                "the output of `cargo metadata` is not UTF-8, so it is not JSON; the workspace root is not read"
            ),
            Self::Json(reason) => write!(
                f,
                "the output of `cargo metadata` does not give the workspace root: {reason}"
            ),
            Self::NotAbsolute { value } => write!(
                f,
                "`workspace_root` of `cargo metadata` is `{}`, which is not an absolute path",
                value.escape_debug()
            ),
        }
    }
}

impl Error for RootError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Exec(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ExecError> for RootError {
    fn from(error: ExecError) -> Self {
        Self::Exec(error)
    }
}

impl From<JsonError> for RootError {
    fn from(error: JsonError) -> Self {
        Self::Json(error.to_string())
    }
}

/// The workspace root of the Cargo workspace the runner's directory is in, as
/// the `workspace_root` field of `cargo metadata` gives it.
///
/// # Errors
///
/// A [`RootError`] when `cargo` cannot run or fails, writes more than 64 MiB, or
/// its output does not give the root as the string member `workspace_root` of
/// the top-level JSON object, an absolute path.
pub fn discover(runner: &dyn Runner) -> Result<PathBuf, RootError> {
    let output = runner.run(&Invocation::metadata(), &Limits::metadata())?;
    if !output.success() {
        return Err(RootError::Failed {
            code: output.code,
            stderr: output.stderr_text(),
        });
    }
    let text = std::str::from_utf8(&output.stdout).map_err(|_| RootError::NotUtf8)?;
    from_metadata(text)
}

/// The root from the text of a `cargo metadata` output.
fn from_metadata(text: &str) -> Result<PathBuf, RootError> {
    let value = json::workspace_root(text)?;
    if value.contains('\0') || !Path::new(&value).is_absolute() {
        return Err(RootError::NotAbsolute { value });
    }
    Ok(PathBuf::from(value))
}

#[cfg(test)]
mod tests;
