// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The command line: the clap definitions and the two shapes of the argument
//! vector.
//!
//! Cargo runs a subcommand `cargo workplan check` as `cargo-workplan workplan
//! check`, and a person may run `cargo-workplan check` directly. Both reach
//! the same parser: [`parse`] puts the `workplan` word in front when the
//! first argument is not that word, and the parser knows the program as the
//! `workplan` subcommand of `cargo`.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Args as ClapArgs, Parser, Subcommand};
use raf_okf::is_kebab;

use crate::render::Format;

/// The name cargo gives the subcommand.
const SUBCOMMAND: &str = "workplan";

/// The program as cargo knows it: `cargo workplan ...`.
#[derive(Debug, Parser)]
#[command(name = "cargo", bin_name = "cargo", disable_help_subcommand = true)]
enum Cargo {
    /// Plan changes to the documented system and check the documents.
    Workplan(Args),
}

/// The arguments of `cargo workplan`.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
#[command(version, arg_required_else_help = true, disable_help_subcommand = true)]
pub(crate) struct Args {
    /// The workspace root. Without it the root comes from `cargo metadata`,
    /// run in the current directory.
    #[arg(long, global = true, value_name = "dir")]
    pub(crate) root: Option<PathBuf>,

    /// How findings are printed: text, or one JSON array for the run.
    #[arg(
        long,
        global = true,
        value_enum,
        value_name = "format",
        default_value = "text"
    )]
    pub(crate) format: Format,

    /// Print nothing but findings.
    #[arg(short, long, global = true)]
    pub(crate) quiet: bool,

    /// The command to run.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// The commands.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Run every rule of the document model and print each violation.
    Check,
    /// Print the roadmap: capabilities by state, with the gaps of the
    /// implementing ones and the open changes.
    Status,
    /// Generated documentation.
    #[command(subcommand)]
    Docs(Docs),
}

/// The commands under `docs`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Docs {
    /// Rewrite the generated blocks, and create the capability index when it
    /// is missing.
    Sync(SyncArgs),
    /// Print the capabilities of a change as they read once it is
    /// reconciled. Writes nothing.
    Preview(PreviewArgs),
}

/// The arguments of `docs sync`.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
pub(crate) struct SyncArgs {
    /// The copyright text of the first SPDX line of a new capability index.
    /// Read only when the index has to be created, then required.
    #[arg(long, value_name = "text")]
    pub(crate) copyright: Option<String>,

    /// The licence identifier of the second SPDX line of a new capability
    /// index. Read only when the index has to be created, then required.
    #[arg(long, value_name = "id")]
    pub(crate) licence: Option<String>,
}

/// The arguments of `docs preview`.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
pub(crate) struct PreviewArgs {
    /// The slug of the change record.
    #[arg(value_name = "slug", value_parser = slug)]
    pub(crate) slug: String,
}

/// A slug argument: the slug pattern of the profile, so a value that starts
/// with `-`, holds a path separator or is not kebab-case is a usage error
/// before anything is read or started.
pub(crate) fn slug(text: &str) -> Result<String, String> {
    if text.starts_with('-') {
        Err("a slug may not start with `-`".to_string())
    } else if is_kebab(text) {
        Ok(text.to_string())
    } else {
        Err(
            "a slug is lowercase letters and digits in parts joined by single `-`, and starts with a letter or a digit"
                .to_string(),
        )
    }
}

/// The argument vector of a run in the form the parser takes: `cargo-workplan
/// workplan <rest>`. Cargo already passes the `workplan` word; a direct run
/// does not, and gets it here.
fn shaped(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut args: Vec<OsString> = args.into_iter().collect();
    if args.is_empty() {
        args.push(OsString::from("cargo-workplan"));
    }
    if args.get(1).is_none_or(|first| first != SUBCOMMAND) {
        args.insert(1, OsString::from(SUBCOMMAND));
    }
    args
}

/// Parse an argument vector, the program name first, in either shape.
///
/// # Errors
///
/// The error of clap: a usage error (exit code 2), or the request for help or
/// the version (exit code 0), which the caller prints.
pub(crate) fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Args, clap::Error> {
    let Cargo::Workplan(args) = Cargo::try_parse_from(shaped(args))?;
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(args: &[&str]) -> Result<Args, clap::Error> {
        parse(args.iter().map(OsString::from))
    }

    /// The same words in both shapes of the argument vector.
    fn both(words: &[&str]) -> [Args; 2] {
        let direct: Vec<&str> = std::iter::once("cargo-workplan")
            .chain(words.iter().copied())
            .collect();
        let subcommand: Vec<&str> = ["cargo-workplan", "workplan"]
            .into_iter()
            .chain(words.iter().copied())
            .collect();
        [
            parsed(&direct).expect("the direct form parses"),
            parsed(&subcommand).expect("the cargo form parses"),
        ]
    }

    fn same(words: &[&str]) -> Args {
        let [direct, subcommand] = both(words);
        assert_eq!(direct, subcommand, "{words:?}");
        direct
    }

    #[test]
    fn each_command_parses_in_both_shapes() {
        assert_eq!(same(&["check"]).command, Command::Check);
        assert_eq!(same(&["status"]).command, Command::Status);
        assert_eq!(
            same(&["docs", "sync"]).command,
            Command::Docs(Docs::Sync(SyncArgs {
                copyright: None,
                licence: None
            }))
        );
        assert_eq!(
            same(&["docs", "preview", "grow-it"]).command,
            Command::Docs(Docs::Preview(PreviewArgs {
                slug: "grow-it".to_string()
            }))
        );
    }

    #[test]
    fn the_options_default_and_parse_in_both_shapes() {
        let plain = same(&["check"]);
        assert_eq!(plain.root, None);
        assert_eq!(plain.format, Format::Text);
        assert!(!plain.quiet);

        let set = same(&["--root", "some/dir", "--format", "json", "-q", "check"]);
        assert_eq!(set.root, Some(PathBuf::from("some/dir")));
        assert_eq!(set.format, Format::Json);
        assert!(set.quiet);
    }

    #[test]
    fn the_options_may_follow_the_command() {
        let after = same(&["status", "--format", "json", "--root", "r", "--quiet"]);
        let before = same(&["--format", "json", "--root", "r", "--quiet", "status"]);
        assert_eq!(after, before);
        assert_eq!(after.format, Format::Json);
        let nested = same(&["docs", "preview", "grow", "-q", "--format=json"]);
        assert!(nested.quiet && nested.format == Format::Json);
    }

    #[test]
    fn the_sync_text_options_parse() {
        let args = same(&[
            "docs",
            "sync",
            "--copyright",
            "2026 Example Org",
            "--licence",
            "CC-BY-4.0",
        ]);
        assert_eq!(
            args.command,
            Command::Docs(Docs::Sync(SyncArgs {
                copyright: Some("2026 Example Org".to_string()),
                licence: Some("CC-BY-4.0".to_string()),
            }))
        );
    }

    #[test]
    fn a_format_that_is_not_text_or_json_is_a_usage_error() {
        let error = parsed(&["cargo-workplan", "--format", "yaml", "check"]).expect_err("yaml");
        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn a_missing_command_an_unknown_command_and_a_missing_slug_are_usage_errors() {
        for words in [
            &["cargo-workplan"][..],
            &["cargo-workplan", "workplan"],
            &["cargo-workplan", "frobnicate"],
            &["cargo-workplan", "docs"],
            &["cargo-workplan", "docs", "preview"],
            &["cargo-workplan", "check", "extra"],
            &["cargo-workplan", "check", "--no-such-option"],
        ] {
            let error = parsed(words).expect_err("a usage error");
            assert_eq!(error.exit_code(), 2, "{words:?}");
        }
    }

    #[test]
    fn a_slug_that_starts_with_a_dash_is_a_usage_error_however_it_is_given() {
        for slug in ["-rf", "--since=x", "-", "--"] {
            // After `--` the word is a value and the slug parser refuses it.
            let after = parsed(&["cargo-workplan", "docs", "preview", "--", slug]);
            let error = after.expect_err("a slug that starts with a dash");
            assert_eq!(error.exit_code(), 2, "{slug}");
            // Before `--` it is read as an option and refused as one.
            let before = parsed(&["cargo-workplan", "docs", "preview", slug]);
            assert_eq!(before.expect_err("an option").exit_code(), 2, "{slug}");
        }
    }

    #[test]
    fn a_slug_must_be_a_slug() {
        for good in ["grow", "grow-it", "2fa", "a1-b2"] {
            assert_eq!(slug(good).as_deref(), Ok(good));
        }
        for bad in [
            "",
            "Grow",
            "-grow",
            "grow-",
            "a--b",
            "a_b",
            "a/b",
            "../x",
            "change:grow",
            "a b",
        ] {
            assert!(slug(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn help_and_version_exit_zero_in_both_shapes() {
        for words in [
            &["cargo-workplan", "--help"][..],
            &["cargo-workplan", "workplan", "--help"],
            &["cargo-workplan", "--version"],
            &["cargo-workplan", "workplan", "-V"],
            &["cargo-workplan", "docs", "--help"],
        ] {
            let error = parsed(words).expect_err("help is an early exit");
            assert_eq!(error.exit_code(), 0, "{words:?}");
        }
    }
}
