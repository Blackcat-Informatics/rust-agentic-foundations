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

use clap::{ArgAction, ArgGroup, Args as ClapArgs, Parser, Subcommand, ValueEnum};
use raf_okf::{Actor, AssessmentVerdict, DocType, Measure, is_kebab};
use raf_workplan::assessment::parse_subject;
use raf_workplan::evidence::parse_limit;
use raf_workplan::exec::{Commit, Date};
use raf_workplan::history::HISTORY_KINDS;

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

    /// Judge under strict mode, whatever the coverage record says: `check`,
    /// `change reconcile` and `change complete` judge the repository strict,
    /// `coverage lines` shows the ceilings as enforced, `debt --write` writes
    /// the debt of strict mode, and the other commands print the same.
    #[arg(long, global = true)]
    pub(crate) strict: bool,

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
    /// Print the commits that shaped a document or record and, for a
    /// capability, its bound code, newest first. Reads Git; writes nothing.
    History(HistoryArgs),
    /// Evidence records: run a command or a bound test and record its
    /// verdict, or list the claims and records that rest on one another.
    #[command(subcommand)]
    Evidence(Evidence),
    /// Prose assessment of one section: print what an assessor reads, record
    /// a verdict, or list each assessment and whether it is current.
    Assess(AssessArgs),
    /// The coverage reports of the whole workspace: the public items or the
    /// tests that no capability binds, the claims of the delivered
    /// capabilities and the tests that their markers name, or the lines that
    /// no capability explains.
    #[command(subcommand)]
    Coverage(Coverage),
    /// Write the debt record `work/debt/default.md`: the findings of strict
    /// mode and the orphan lines the repository holds as debt. The write
    /// creates the record when there is none, refuses debt that grew, and
    /// lowers it when debt was paid.
    Debt(DebtArgs),
}

/// The arguments of `debt`.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
pub(crate) struct DebtArgs {
    /// Write the record. Required: the command has no other mode.
    #[arg(long, required = true)]
    pub(crate) write: bool,

    /// The copyright text of the first SPDX line of a record that is created
    /// or replaced. Given with `--licence`, or not at all; a record that
    /// exists keeps its own lines when neither is given.
    #[arg(long, value_name = "text", requires = "licence", value_parser = word)]
    pub(crate) copyright: Option<String>,

    /// The licence identifier of the second SPDX line of the record. Given
    /// with `--copyright`, or not at all.
    #[arg(long, value_name = "id", requires = "copyright", value_parser = word)]
    pub(crate) licence: Option<String>,
}

/// The arguments of `history`.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
pub(crate) struct HistoryArgs {
    /// The id of a loaded document or record: `cap:`, `change:`, `arch:`,
    /// `evidence:` or `assessment:` and a slug.
    #[arg(value_name = "id", value_parser = history_id)]
    pub(crate) id: String,

    /// Only the commits whose author date is from 00:00:00 UTC of this day on.
    #[arg(long, value_name = "YYYY-MM-DD", value_parser = since)]
    pub(crate) since: Option<Date>,
}

/// The commands under `evidence`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Evidence {
    /// Run the command after `--`, or a bound test or example, and write its
    /// verdict as the evidence record `work/evidence/<id>.md`.
    Record(RecordArgs),
    /// Print the statements that rest on an evidence record.
    Claims(EvidenceIdArgs),
    /// Print the evidence records that support a capability.
    For(CapabilityArgs),
}

/// Who records an evidence record or an assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum By {
    /// A person.
    Human,
    /// An agent.
    Agent,
}

impl By {
    /// The actor of the record.
    pub(crate) fn actor(self) -> Actor {
        match self {
            Self::Human => Actor::Human,
            Self::Agent => Actor::Agent,
        }
    }
}

/// The verdict of an assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum Verdict {
    /// The section states the delivered contract clearly.
    Accepted,
    /// It does not.
    Rejected,
}

impl Verdict {
    /// The verdict of the record.
    pub(crate) fn verdict(self) -> AssessmentVerdict {
        match self {
            Self::Accepted => AssessmentVerdict::Accepted,
            Self::Rejected => AssessmentVerdict::Rejected,
        }
    }
}

/// The arguments of `evidence record`.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
pub(crate) struct RecordArgs {
    /// The slug of the record, `work/evidence/<id>.md`.
    #[arg(long, value_name = "slug", value_parser = slug)]
    pub(crate) id: String,

    /// A capability the evidence covers, `cap:<slug>`. Give the option once
    /// for each capability.
    #[arg(
        long = "for",
        id = "covers",
        value_name = "cap:<slug>",
        required = true,
        action = ArgAction::Append,
        value_parser = capability_id
    )]
    pub(crate) covers: Vec<String>,

    /// A limit on a value the command prints, `name=number`. Give the option
    /// once for each limit.
    #[arg(long, value_name = "name=number", action = ArgAction::Append, value_parser = limit)]
    pub(crate) limit: Vec<Measure>,

    /// The commit the execution ran at, 40 or 64 lowercase hexadecimal
    /// digits. Stored for readers only.
    #[arg(long, value_name = "commit", value_parser = revision)]
    pub(crate) revision: Option<String>,

    /// Replace the record of the id instead of creating a new one.
    #[arg(long)]
    pub(crate) update: bool,

    /// Who records it: `human` or `agent`.
    #[arg(long, value_enum, value_name = "human|agent", required = true)]
    pub(crate) by: By,

    /// The copyright text of the first SPDX line of the record. Given with
    /// `--licence` or not at all.
    #[arg(long, value_name = "text", requires = "licence", value_parser = word)]
    pub(crate) copyright: Option<String>,

    /// The licence identifier of the second SPDX line of the record. Given
    /// with `--copyright` or not at all.
    #[arg(long, value_name = "id", requires = "copyright", value_parser = word)]
    pub(crate) licence: Option<String>,

    /// Take the verdict from this bound test or example of a capability of
    /// `--for` instead of a command.
    #[arg(long, value_name = "path", value_parser = word, conflicts_with = "command")]
    pub(crate) from_binding: Option<String>,

    /// The command to run, after `--`: the program and its arguments, run
    /// exactly as given, with no shell.
    #[arg(
        last = true,
        value_name = "command",
        num_args = 1..,
        required_unless_present = "from_binding",
        value_parser = clap::value_parser!(OsString)
    )]
    pub(crate) command: Vec<OsString>,
}

/// The arguments of `evidence claims`.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
pub(crate) struct EvidenceIdArgs {
    /// The evidence record, `evidence:<slug>` or the slug.
    #[arg(value_name = "evidence-id", value_parser = evidence_id)]
    pub(crate) id: String,
}

/// The arguments of `evidence for`.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
pub(crate) struct CapabilityArgs {
    /// The capability, `cap:<slug>`.
    #[arg(value_name = "cap:<slug>", value_parser = capability_id)]
    pub(crate) capability: String,
}

/// The arguments of `assess`: one of `--context`, `--verdict` and `--list`.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
#[command(group(ArgGroup::new("mode").required(true).multiple(false)))]
pub(crate) struct AssessArgs {
    /// The section, `cap:<slug>#<section>`. Optional with `--list`, which
    /// then lists every assessment.
    #[arg(
        value_name = "cap:<slug>#<section>",
        value_parser = section,
        required_unless_present = "list"
    )]
    pub(crate) subject: Option<String>,

    /// Print what an assessor reads: the section, the bound symbols that are
    /// its inputs and the evidence its claims name.
    #[arg(long, group = "mode")]
    pub(crate) context: bool,

    /// Record this verdict on the section, replacing its assessment.
    #[arg(
        long,
        value_enum,
        value_name = "accepted|rejected",
        group = "mode",
        requires = "by"
    )]
    pub(crate) verdict: Option<Verdict>,

    /// List each assessment and whether it is current.
    #[arg(long, group = "mode")]
    pub(crate) list: bool,

    /// Who assesses: `human` or `agent`. Required with `--verdict`; the
    /// context is the same for both.
    #[arg(long, value_enum, value_name = "human|agent", conflicts_with = "list")]
    pub(crate) by: Option<By>,

    /// The copyright text of the first SPDX line of the record. Given with
    /// `--licence` and `--verdict`, or not at all.
    #[arg(long, value_name = "text", requires_all = ["licence", "verdict"], value_parser = word)]
    pub(crate) copyright: Option<String>,

    /// The licence identifier of the second SPDX line of the record. Given
    /// with `--copyright` and `--verdict`, or not at all.
    #[arg(long, value_name = "id", requires_all = ["copyright", "verdict"], value_parser = word)]
    pub(crate) licence: Option<String>,
}

/// The commands under `coverage`.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum Coverage {
    /// Print the public items that no capability binds and no exclusion
    /// takes out.
    Code(CoverageArgs),
    /// Print the tests that no capability binds and no exclusion takes out.
    Tests(CoverageArgs),
    /// Print each claim of the delivered capabilities with the tests its
    /// tested-by markers name.
    Claims(ClaimsArgs),
    /// Print the lines of every package that no capability explains, by
    /// crate, module and item: the reverse coverage. `coverage --reverse` is
    /// the same command.
    #[command(long_flag = "reverse")]
    Lines(CoverageArgs),
}

/// The arguments of `coverage claims`.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
pub(crate) struct ClaimsArgs {
    /// Print only the claims of this capability, `cap:<slug>`.
    #[arg(long, value_name = "cap:<slug>", value_parser = capability_id)]
    pub(crate) capability: Option<String>,
}

/// The arguments of a coverage report.
#[derive(Debug, Clone, PartialEq, Eq, ClapArgs)]
pub(crate) struct CoverageArgs {
    /// Print only the report of this package (`raf-okf`, or `raf_okf`).
    #[arg(long = "crate", value_name = "package", value_parser = package)]
    pub(crate) package: Option<String>,
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

/// A value that may not start with `-`: the word is never read as an
/// option, and the separator `--` is never a value.
fn undashed(text: &str, what: &str) -> Result<(), String> {
    if text.starts_with('-') {
        Err(format!("{what} may not start with `-`"))
    } else {
        Ok(())
    }
}

/// A text value of an option: anything but the separator `--`, so that an
/// option never takes the word that starts the command of `evidence record`.
pub(crate) fn word(text: &str) -> Result<String, String> {
    if text == "--" {
        Err("the value may not be `--`, which starts the command".to_string())
    } else {
        Ok(text.to_string())
    }
}

/// A capability id, `cap:<slug>`.
pub(crate) fn capability_id(text: &str) -> Result<String, String> {
    undashed(text, "a capability id")?;
    match text.strip_prefix(DocType::Capability.id_prefix()) {
        Some(slug) if is_kebab(slug) => Ok(text.to_string()),
        _ => Err("a capability id is `cap:` and a slug, such as `cap:workplan-cli`".to_string()),
    }
}

/// An id the history answers for: one of its prefixes and a slug.
pub(crate) fn history_id(text: &str) -> Result<String, String> {
    undashed(text, "an id")?;
    let kebab = HISTORY_KINDS
        .iter()
        .any(|kind| text.strip_prefix(kind.id_prefix()).is_some_and(is_kebab));
    if kebab {
        Ok(text.to_string())
    } else {
        let prefixes: Vec<&str> = HISTORY_KINDS.iter().map(|kind| kind.id_prefix()).collect();
        Err(format!(
            "an id is one of {} and a slug, such as `cap:workplan-cli`",
            prefixes.join(", ")
        ))
    }
}

/// An evidence id, `evidence:<slug>`, or the slug alone.
pub(crate) fn evidence_id(text: &str) -> Result<String, String> {
    undashed(text, "an evidence id")?;
    let slug = text
        .strip_prefix(DocType::Evidence.id_prefix())
        .unwrap_or(text);
    if is_kebab(slug) {
        Ok(text.to_string())
    } else {
        Err("an evidence id is `evidence:` and a slug, or the slug".to_string())
    }
}

/// A `--since` day, `YYYY-MM-DD` from 1970-01-01 to 2099-12-31.
pub(crate) fn since(text: &str) -> Result<Date, String> {
    undashed(text, "a date")?;
    Date::new(text).map_err(|error| error.to_string())
}

/// A `--limit`, `name=number` with no exponent.
pub(crate) fn limit(text: &str) -> Result<Measure, String> {
    undashed(text, "a limit")?;
    parse_limit(text).map_err(|error| error.to_string())
}

/// A `--revision`, a commit id.
pub(crate) fn revision(text: &str) -> Result<String, String> {
    undashed(text, "a revision")?;
    Commit::new(text)
        .map(|commit| commit.as_str().to_string())
        .map_err(|error| error.to_string())
}

/// A section, `cap:<slug>#<section>`.
pub(crate) fn section(text: &str) -> Result<String, String> {
    undashed(text, "a section")?;
    parse_subject(text)
        .map(|_| text.to_string())
        .map_err(|error| error.to_string())
}

/// A package name of `--crate`: letters, digits, `-` and `_`, not starting
/// with `-`.
pub(crate) fn package(text: &str) -> Result<String, String> {
    undashed(text, "a package name")?;
    let named = !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if named {
        Ok(text.to_string())
    } else {
        Err("a package name is letters, digits, `-` and `_`".to_string())
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
    fn strict_is_a_global_option_off_by_default_and_taken_before_or_after_the_command() {
        assert!(!same(&["check"]).strict);
        for words in [
            &["--strict", "check"][..],
            &["check", "--strict"],
            &["change", "reconcile", "grow", "--strict"],
            &["--strict", "change", "complete", "grow"],
            &["status", "--strict", "-q"],
        ] {
            assert!(same(words).strict, "{words:?}");
        }
        assert_eq!(same(&["--strict", "check"]), same(&["check", "--strict"]));
        // The option takes no value.
        assert!(parsed(&["cargo-workplan", "--strict=yes", "check"]).is_err());
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

    /// The usage error of `words` in both shapes: exit 2 each time.
    fn refused(words: &[&str]) {
        let direct: Vec<&str> = std::iter::once("cargo-workplan")
            .chain(words.iter().copied())
            .collect();
        let subcommand: Vec<&str> = ["cargo-workplan", "workplan"]
            .into_iter()
            .chain(words.iter().copied())
            .collect();
        for line in [direct, subcommand] {
            let error = parsed(&line).expect_err("a usage error");
            assert_eq!(error.exit_code(), 2, "{line:?}");
        }
    }

    fn record_args(words: &[&str]) -> RecordArgs {
        match same(words).command {
            Command::Evidence(Evidence::Record(args)) => args,
            other => panic!("not `evidence record`: {other:?}"),
        }
    }

    fn assess_args(words: &[&str]) -> AssessArgs {
        match same(words).command {
            Command::Assess(args) => args,
            other => panic!("not `assess`: {other:?}"),
        }
    }

    fn os(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    #[test]
    fn history_takes_an_id_of_each_kind_and_a_day() {
        for id in [
            "cap:workplan-cli",
            "change:grow",
            "arch:normal-form",
            "evidence:peak-memory",
            "assessment:alpha-contract",
        ] {
            assert_eq!(
                same(&["history", id]).command,
                Command::History(HistoryArgs {
                    id: id.to_string(),
                    since: None
                })
            );
        }
        let dated = same(&["history", "cap:a", "--since", "2026-01-31", "-q"]);
        assert!(dated.quiet);
        assert_eq!(
            dated.command,
            Command::History(HistoryArgs {
                id: "cap:a".to_string(),
                since: Some(Date::new("2026-01-31").expect("a day")),
            })
        );
        assert_eq!(
            same(&["-q", "history", "--since=2026-01-31", "cap:a"]),
            dated
        );
    }

    #[test]
    fn history_refuses_an_id_or_a_day_before_anything_runs() {
        for words in [
            &["history"][..],
            &["history", "-rf"],
            &["history", "--", "-rf"],
            &["history", "--", "--since=x"],
            &["history", "cap:Bad"],
            &["history", "cap:"],
            &["history", "coverage:default"],
            &["history", "cap:a/b"],
            &["history", "workplan-cli"],
            &["history", "cap:a", "--since", "2026-13-01"],
            &["history", "cap:a", "--since", "2026-02-30"],
            &["history", "cap:a", "--since", "1969-12-31"],
            &["history", "cap:a", "--since", "yesterday"],
            &["history", "cap:a", "--since=-1"],
            &["history", "cap:a", "--since"],
            &["history", "cap:a", "cap:b"],
        ] {
            refused(words);
        }
    }

    #[test]
    fn evidence_record_reads_every_option_and_the_command_after_the_separator() {
        let full = record_args(&[
            "evidence",
            "record",
            "--id",
            "peak-memory",
            "--for",
            "cap:a",
            "--for",
            "cap:b",
            "--limit",
            "peak=64",
            "--limit",
            "rss=1.5",
            "--revision",
            "0123456789abcdef0123456789abcdef01234567",
            "--update",
            "--by",
            "agent",
            "--copyright",
            "2026 Example Org",
            "--licence",
            "CC-BY-4.0",
            "--",
            "cargo",
            "bench",
            "--",
            "--save",
        ]);
        assert_eq!(full.id, "peak-memory");
        assert_eq!(full.covers, ["cap:a", "cap:b"]);
        let limits: Vec<(&str, &str)> = full
            .limit
            .iter()
            .map(|m| (m.name.as_str(), m.value.text()))
            .collect();
        assert_eq!(limits, [("peak", "64"), ("rss", "1.5")]);
        assert_eq!(
            full.revision.as_deref(),
            Some("0123456789abcdef0123456789abcdef01234567")
        );
        assert!(full.update);
        assert_eq!(full.by, By::Agent);
        assert_eq!(full.by.actor(), Actor::Agent);
        assert_eq!(full.copyright.as_deref(), Some("2026 Example Org"));
        assert_eq!(full.licence.as_deref(), Some("CC-BY-4.0"));
        assert_eq!(full.from_binding, None);
        // Every word after the first `--` is the command, a second `--`
        // included.
        assert_eq!(full.command, os(&["cargo", "bench", "--", "--save"]));

        let minimal = record_args(&[
            "evidence", "record", "--id", "x", "--for", "cap:a", "--by", "human", "--", "true",
        ]);
        assert_eq!(minimal.by, By::Human);
        assert_eq!(minimal.by.actor(), Actor::Human);
        assert!(!minimal.update && minimal.limit.is_empty() && minimal.revision.is_none());
        assert_eq!(minimal.command, os(&["true"]));

        let bound = record_args(&[
            "evidence",
            "record",
            "--id",
            "x",
            "--for",
            "cap:a",
            "--by",
            "human",
            "--from-binding",
            "demo::tests::passes",
        ]);
        assert_eq!(bound.from_binding.as_deref(), Some("demo::tests::passes"));
        assert_eq!(bound.command, Vec::<OsString>::new());
    }

    #[test]
    fn options_after_the_separator_belong_to_the_command() {
        let args = same(&[
            "--format",
            "json",
            "evidence",
            "record",
            "--id",
            "x",
            "--for",
            "cap:a",
            "--by",
            "agent",
            "--",
            "sh",
            "-q",
            "--format=text",
            "--root",
            "elsewhere",
            "--by",
            "human",
        ]);
        assert_eq!(args.format, Format::Json);
        assert!(!args.quiet);
        assert_eq!(args.root, None);
        let Command::Evidence(Evidence::Record(record)) = args.command else {
            panic!("not `evidence record`");
        };
        assert_eq!(record.by, By::Agent);
        assert_eq!(
            record.command,
            os(&[
                "sh",
                "-q",
                "--format=text",
                "--root",
                "elsewhere",
                "--by",
                "human"
            ])
        );
    }

    #[test]
    fn evidence_record_refuses_a_separator_as_a_value_and_a_record_it_cannot_write() {
        let head = [
            "evidence", "record", "--id", "x", "--for", "cap:a", "--by", "agent",
        ];
        let with = |tail: &[&'static str]| -> Vec<&'static str> {
            head.iter().chain(tail.iter()).copied().collect()
        };
        for words in [
            // An option whose value would be the separator.
            with(&["--copyright", "--", "--licence", "x", "--", "true"]),
            with(&["--copyright=--", "--licence", "x", "--", "true"]),
            with(&["--licence=--", "--copyright", "x", "--", "true"]),
            with(&["--from-binding", "--"]),
            with(&["--from-binding=--"]),
            with(&["--revision", "--", "true"]),
            with(&["--limit", "--", "true"]),
            // No source of the verdict, or both.
            with(&[]),
            with(&["--"]),
            with(&["--from-binding", "demo::t", "--", "true"]),
            // A value that is refused.
            with(&["--limit", "peak=1e-3", "--", "true"]),
            with(&["--limit", "peak", "--", "true"]),
            with(&["--limit", "-1=2", "--", "true"]),
            with(&["--revision", "HEAD", "--", "true"]),
            with(&["--revision", "0123456", "--", "true"]),
            with(&["--copyright", "x", "--", "true"]),
            with(&["--licence", "x", "--", "true"]),
            with(&["--by", "robot", "--", "true"]),
            vec![
                "evidence", "record", "--id", "x", "--for", "cap:a", "--", "true",
            ],
            vec![
                "evidence", "record", "--id", "x", "--by", "agent", "--", "true",
            ],
            vec![
                "evidence", "record", "--for", "cap:a", "--by", "agent", "--", "true",
            ],
            vec![
                "evidence", "record", "--id", "-rf", "--for", "cap:a", "--by", "agent", "--",
                "true",
            ],
            vec![
                "evidence",
                "record",
                "--id",
                "x",
                "--for",
                "workplan-cli",
                "--by",
                "agent",
                "--",
                "true",
            ],
            vec![
                "evidence", "record", "--id", "x", "--for", "--", "--by", "agent", "--", "true",
            ],
        ] {
            refused(&words);
        }
    }

    #[test]
    fn evidence_claims_and_for_take_one_id_each() {
        for id in ["evidence:peak", "peak"] {
            assert_eq!(
                same(&["evidence", "claims", id]).command,
                Command::Evidence(Evidence::Claims(EvidenceIdArgs { id: id.to_string() }))
            );
        }
        assert_eq!(
            same(&["evidence", "for", "cap:a"]).command,
            Command::Evidence(Evidence::For(CapabilityArgs {
                capability: "cap:a".to_string()
            }))
        );
        for words in [
            &["evidence"][..],
            &["evidence", "claims"],
            &["evidence", "claims", "-rf"],
            &["evidence", "claims", "--", "-rf"],
            &["evidence", "claims", "evidence:Peak"],
            &["evidence", "claims", "cap:a"],
            &["evidence", "claims", "a", "b"],
            &["evidence", "for"],
            &["evidence", "for", "a"],
            &["evidence", "for", "--", "-rf"],
            &["evidence", "frobnicate"],
        ] {
            refused(words);
        }
    }

    #[test]
    fn assess_takes_a_section_and_exactly_one_mode() {
        let context = assess_args(&["assess", "cap:a#usage", "--context"]);
        assert!(context.context && !context.list && context.verdict.is_none());
        assert_eq!(context.subject.as_deref(), Some("cap:a#usage"));
        assert_eq!(context.by, None);
        let by = assess_args(&["assess", "cap:a#usage", "--context", "--by", "agent"]);
        assert_eq!(by.by, Some(By::Agent));
        let verdict = assess_args(&[
            "assess",
            "--verdict",
            "rejected",
            "--by",
            "human",
            "cap:a#contract",
            "--copyright",
            "2026 Example Org",
            "--licence",
            "CC-BY-4.0",
        ]);
        assert_eq!(verdict.verdict, Some(Verdict::Rejected));
        assert_eq!(Verdict::Rejected.verdict(), AssessmentVerdict::Rejected);
        assert_eq!(Verdict::Accepted.verdict(), AssessmentVerdict::Accepted);
        assert_eq!(verdict.by, Some(By::Human));
        assert_eq!(verdict.copyright.as_deref(), Some("2026 Example Org"));
        let list = assess_args(&["assess", "--list"]);
        assert!(list.list && list.subject.is_none());
        let one = assess_args(&["assess", "--list", "cap:a#usage"]);
        assert_eq!(one.subject.as_deref(), Some("cap:a#usage"));
        for words in [
            &["assess"][..],
            &["assess", "cap:a#usage"],
            &["assess", "--context"],
            &["assess", "cap:a#usage", "--context", "--list"],
            &[
                "assess",
                "cap:a#usage",
                "--context",
                "--verdict",
                "accepted",
                "--by",
                "agent",
            ],
            &["assess", "cap:a#usage", "--verdict", "accepted"],
            &[
                "assess",
                "cap:a#usage",
                "--verdict",
                "fine",
                "--by",
                "agent",
            ],
            &[
                "assess",
                "cap:a#usage",
                "--verdict",
                "accepted",
                "--by",
                "robot",
            ],
            &["assess", "--list", "--by", "agent"],
            &["assess", "cap:a#intro", "--context"],
            &["assess", "cap:a", "--context"],
            &["assess", "a#usage", "--context"],
            &["assess", "--context", "--", "-rf"],
            &[
                "assess",
                "cap:a#usage",
                "--context",
                "--copyright",
                "x",
                "--licence",
                "y",
            ],
            &[
                "assess",
                "cap:a#usage",
                "--verdict",
                "accepted",
                "--by",
                "agent",
                "--copyright",
                "x",
            ],
        ] {
            refused(words);
        }
    }

    #[test]
    fn debt_takes_write_and_the_spdx_pair_together_or_not_at_all() {
        let args = |copyright: Option<&str>, licence: Option<&str>| {
            Command::Debt(DebtArgs {
                write: true,
                copyright: copyright.map(str::to_string),
                licence: licence.map(str::to_string),
            })
        };
        assert_eq!(same(&["debt", "--write"]).command, args(None, None));
        assert_eq!(
            same(&[
                "debt",
                "--write",
                "--copyright",
                "2026 Example",
                "--licence",
                "CC-BY-4.0"
            ])
            .command,
            args(Some("2026 Example"), Some("CC-BY-4.0"))
        );
        for words in [
            &["debt"][..],
            &["debt", "--copyright", "x", "--licence", "y"],
            &["debt", "--write", "--copyright", "x"],
            &["debt", "--write", "--licence", "y"],
            &["debt", "--write", "--copyright", "--", "--licence", "y"],
            &["debt", "--write", "extra"],
        ] {
            refused(words);
        }
    }

    #[test]
    fn coverage_takes_a_direction_and_an_optional_package() {
        assert_eq!(
            same(&["coverage", "code"]).command,
            Command::Coverage(Coverage::Code(CoverageArgs { package: None }))
        );
        assert_eq!(
            same(&["coverage", "tests", "--crate", "raf_okf"]).command,
            Command::Coverage(Coverage::Tests(CoverageArgs {
                package: Some("raf_okf".to_string())
            }))
        );
        for words in [
            &["coverage"][..],
            &["coverage", "docs"],
            &["coverage", "code", "--crate"],
            &["coverage", "code", "--crate", "-x"],
            &["coverage", "code", "--crate=-x"],
            &["coverage", "code", "--crate", "a b"],
            &["coverage", "code", "--crate", "../x"],
            &["coverage", "code", "extra"],
        ] {
            refused(words);
        }
    }

    #[test]
    fn coverage_lines_takes_an_optional_package_and_reverse_is_the_same_command() {
        assert_eq!(
            same(&["coverage", "lines"]).command,
            Command::Coverage(Coverage::Lines(CoverageArgs { package: None }))
        );
        for args in [
            &["coverage", "lines", "--crate", "raf-okf"][..],
            &["coverage", "--reverse", "--crate", "raf-okf"],
        ] {
            assert_eq!(
                same(args).command,
                Command::Coverage(Coverage::Lines(CoverageArgs {
                    package: Some("raf-okf".to_string())
                })),
                "{args:?}"
            );
        }
        assert_eq!(
            same(&["coverage", "--reverse"]).command,
            same(&["coverage", "lines"]).command
        );
        for args in [
            &["coverage", "lines", "--crate"][..],
            &["coverage", "lines", "--crate", "-x"],
            &["coverage", "lines", "--crate", "../x"],
            &["coverage", "lines", "extra"],
            &["coverage", "--reverse", "lines"],
            &["coverage", "--reverse", "--capability", "cap:a"],
        ] {
            refused(args);
        }
    }

    #[test]
    fn coverage_claims_takes_an_optional_capability() {
        assert_eq!(
            same(&["coverage", "claims"]).command,
            Command::Coverage(Coverage::Claims(ClaimsArgs { capability: None }))
        );
        assert_eq!(
            same(&["coverage", "claims", "--capability", "cap:workplan-cli"]).command,
            Command::Coverage(Coverage::Claims(ClaimsArgs {
                capability: Some("cap:workplan-cli".to_string())
            }))
        );
        for words in [
            &["coverage", "claims", "--capability"][..],
            &["coverage", "claims", "--capability", "workplan-cli"],
            &["coverage", "claims", "--capability", "cap:Bad"],
            &["coverage", "claims", "--capability", "-x"],
            &["coverage", "claims", "--crate", "demo"],
            &["coverage", "claims", "cap:workplan-cli"],
        ] {
            refused(words);
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
