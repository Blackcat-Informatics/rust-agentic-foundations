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

use clap::{ArgAction, Args as ClapArgs, Parser, Subcommand};
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
    /// The change records: create, read, reconcile and complete one.
    #[command(subcommand)]
    Change(Change),
    /// Run the bound tests and examples of a change, or of every capability,
    /// and print pass or fail for each binding.
    Verify(VerifyArgs),
}

/// The commands under `change`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Change {
    /// Create the record of a new change, and print the PROPOSED block to
    /// place and the state each listed capability needs.
    New(NewArgs),
    /// Print what the change reaches: its capabilities, sections, bindings,
    /// dependants and linking sections. Writes nothing.
    Impact(SlugArgs),
    /// Print the obligation list of the change, one line each. Writes no file
    /// in the repository.
    Reconcile(SlugArgs),
    /// Run the completion gate and, when every condition holds, remove the
    /// record.
    Complete(SlugArgs),
}

/// The commands under `docs`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Docs {
    /// Rewrite the generated blocks, and create the capability index when it
    /// is missing.
    Sync(SyncArgs),
    /// Print the capabilities of a change as they read once it is
    /// reconciled. Writes nothing.
    Preview(SlugArgs),
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

/// The arguments of a command that takes the slug of a change record.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
pub(crate) struct SlugArgs {
    /// The slug of the change record.
    #[arg(value_name = "slug", value_parser = slug)]
    pub(crate) slug: String,
}

/// The arguments of `verify`.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
pub(crate) struct VerifyArgs {
    /// The slug of the change record. Without it every capability is verified.
    #[arg(value_name = "slug", value_parser = slug)]
    pub(crate) slug: Option<String>,
}

/// The arguments of `change new`.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
pub(crate) struct NewArgs {
    /// The slug of the new change record.
    #[arg(value_name = "slug", value_parser = slug)]
    pub(crate) slug: String,

    /// A capability the change lists, `cap:<name>`. Give the option once for
    /// each capability.
    #[arg(
        long,
        value_name = "cap:<name>",
        required = true,
        action = ArgAction::Append
    )]
    pub(crate) capability: Vec<String>,

    /// A section the change affects, `cap:<name>#<section>`. Give the option
    /// once for each section. Without it the change affects the contract of
    /// each listed capability.
    #[arg(long, value_name = "cap:<name>#<section>", action = ArgAction::Append)]
    pub(crate) affects: Vec<String>,

    /// The copyright text of the first SPDX line of the record. Given with
    /// `--licence` or not at all.
    #[arg(long, value_name = "text", requires = "licence")]
    pub(crate) copyright: Option<String>,

    /// The licence identifier of the second SPDX line of the record. Given
    /// with `--copyright` or not at all.
    #[arg(long, value_name = "id", requires = "copyright")]
    pub(crate) licence: Option<String>,
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
            Command::Docs(Docs::Preview(SlugArgs {
                slug: "grow-it".to_string()
            }))
        );
        for (words, expected) in [
            (
                &["change", "impact", "grow"][..],
                Change::Impact as fn(SlugArgs) -> Change,
            ),
            (&["change", "reconcile", "grow"], Change::Reconcile),
            (&["change", "complete", "grow"], Change::Complete),
        ] {
            assert_eq!(
                same(words).command,
                Command::Change(expected(SlugArgs {
                    slug: "grow".to_string()
                })),
                "{words:?}"
            );
        }
        assert_eq!(
            same(&["verify"]).command,
            Command::Verify(VerifyArgs { slug: None })
        );
        assert_eq!(
            same(&["verify", "grow"]).command,
            Command::Verify(VerifyArgs {
                slug: Some("grow".to_string())
            })
        );
    }

    fn new_args(words: &[&str]) -> NewArgs {
        match same(words).command {
            Command::Change(Change::New(args)) => args,
            other => panic!("not `change new`: {other:?}"),
        }
    }

    #[test]
    fn change_new_takes_a_slug_repeated_capabilities_and_affects_and_the_spdx_pair() {
        let minimal = new_args(&["change", "new", "grow", "--capability", "cap:a"]);
        assert_eq!(
            minimal,
            NewArgs {
                slug: "grow".to_string(),
                capability: vec!["cap:a".to_string()],
                affects: Vec::new(),
                copyright: None,
                licence: None,
            }
        );
        let full = new_args(&[
            "change",
            "new",
            "grow",
            "--capability",
            "cap:a",
            "--capability",
            "cap:b",
            "--affects",
            "cap:a#usage",
            "--affects",
            "cap:b#contract",
            "--copyright",
            "2026 Example Org",
            "--licence",
            "CC-BY-4.0",
        ]);
        assert_eq!(full.capability, ["cap:a", "cap:b"]);
        assert_eq!(full.affects, ["cap:a#usage", "cap:b#contract"]);
        assert_eq!(full.copyright.as_deref(), Some("2026 Example Org"));
        assert_eq!(full.licence.as_deref(), Some("CC-BY-4.0"));
        // The options may come before the slug and the global ones anywhere.
        let mixed = new_args(&[
            "change",
            "new",
            "--capability",
            "cap:a",
            "grow",
            "-q",
            "--format=json",
        ]);
        assert_eq!(mixed, minimal);
        let args = same(&["-q", "change", "new", "grow", "--capability", "cap:a"]);
        assert!(args.quiet);
    }

    #[test]
    fn change_new_needs_a_capability_and_both_or_neither_spdx_option() {
        for words in [
            &["cargo-workplan", "change", "new", "grow"][..],
            &["cargo-workplan", "change", "new", "--capability", "cap:a"],
            &[
                "cargo-workplan",
                "change",
                "new",
                "grow",
                "--capability",
                "cap:a",
                "--copyright",
                "x",
            ],
            &[
                "cargo-workplan",
                "change",
                "new",
                "grow",
                "--capability",
                "cap:a",
                "--licence",
                "x",
            ],
            &[
                "cargo-workplan",
                "change",
                "new",
                "-rf",
                "--capability",
                "cap:a",
            ],
            &["cargo-workplan", "change", "new", "grow", "--capability"],
        ] {
            let error = parsed(words).expect_err("a usage error");
            assert_eq!(error.exit_code(), 2, "{words:?}");
            // The same in the form cargo gives.
            let mut shaped = vec!["cargo-workplan", "workplan"];
            shaped.extend(&words[1..]);
            assert_eq!(parsed(&shaped).expect_err("usage").exit_code(), 2);
        }
    }

    #[test]
    fn a_command_that_takes_a_slug_refuses_one_that_starts_with_a_dash_in_both_forms() {
        for head in [
            &["change", "impact"][..],
            &["change", "reconcile"],
            &["change", "complete"],
            &["verify"],
        ] {
            for slug in ["-rf", "--since=x"] {
                for dashed in [false, true] {
                    let mut words = vec!["cargo-workplan"];
                    words.extend(head);
                    if dashed {
                        words.push("--");
                    }
                    words.push(slug);
                    assert_eq!(
                        parsed(&words).expect_err("a slug").exit_code(),
                        2,
                        "{words:?}"
                    );
                }
            }
        }
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
            &["cargo-workplan", "change"],
            &["cargo-workplan", "change", "frobnicate"],
            &["cargo-workplan", "change", "impact"],
            &["cargo-workplan", "change", "reconcile"],
            &["cargo-workplan", "change", "complete"],
            &["cargo-workplan", "change", "complete", "grow", "extra"],
            &["cargo-workplan", "verify", "grow", "extra"],
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
