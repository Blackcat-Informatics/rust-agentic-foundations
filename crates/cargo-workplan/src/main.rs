// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `cargo workplan`: the command line of `raf-workplan`.
//!
//! The binary is `cargo-workplan`. Cargo runs it for `cargo workplan <command>`,
//! and it runs directly as `cargo-workplan <command>`; both forms take the same
//! arguments. It parses its arguments, calls one function of `raf-workplan` for
//! the command, and prints what that returned. It holds no rule, no check and no
//! decision of its own, so the findings of `check` are the findings of
//! `Gate::workspace(..).check()` and the two cannot disagree.
//!
//! ```text
//! cargo workplan [--root <dir>] [--format text|json] [-q] <command>
//!
//! check                Run every rule of the document model; print each violation
//! status               Print the roadmap: capabilities by state, gaps, open changes
//! docs sync            Rewrite the generated blocks [--copyright <text> --licence <id>]
//! docs preview <slug>  Print the capabilities of a change as they read once reconciled
//! change new <slug> --capability cap:<name> [--capability ...]
//!                      [--affects cap:<name>#<section> ...] [--copyright <text> --licence <id>]
//!                      Create the record of a change and say what to do next
//! change impact <slug>      Print what the change reaches
//! change reconcile <slug>   Print the obligation list, one line each
//! change complete <slug>    Run the completion gate and remove the record
//! verify [<slug>]           Run the bound tests and examples and print pass or fail for each
//! history <id> [--since <YYYY-MM-DD>]
//!                      Print the commits that shaped a document or record
//! evidence record --id <slug> --for cap:<slug> [--for ...] --by human|agent
//!                 [--limit <name>=<number> ...] [--revision <commit>] [--update]
//!                 [--copyright <text> --licence <id>]
//!                 (-- <program> <arg>... | --from-binding <path>)
//!                      Run the command or the bound test and write its evidence record
//! evidence claims <evidence-id>   Print the statements that rest on an evidence record
//! evidence for cap:<slug>         Print the evidence records that support a capability
//! assess cap:<slug>#<section> --context
//!                      Print what an assessor reads
//! assess cap:<slug>#<section> --verdict accepted|rejected --by human|agent
//!        [--copyright <text> --licence <id>]
//!                      Record the assessment of the section
//! assess [cap:<slug>#<section>] --list
//!                      Print each assessment and whether it is current
//! coverage code|tests [--crate <package>]
//!                      Print the public items or the tests no capability binds
//! ```
//!
//! # Root
//!
//! `--root <dir>` names the workspace root. Without it the root is the
//! `workspace_root` of `cargo metadata --format-version 1 --no-deps --frozen`
//! run in the current directory (`raf_workplan::root::discover`), so the
//! command runs from any directory inside the workspace. The program is the
//! one `$CARGO` names, and `cargo` from `PATH` otherwise.
//!
//! # Output
//!
//! Findings go to standard output in a stable order; diagnostics go to standard
//! error. `-q` prints nothing but findings: a command with no findings (`status`,
//! `docs sync`, `docs preview`, `change new`, `change impact`, `history`,
//! `evidence claims`, `evidence for`, `assess`) prints nothing on standard output
//! under `-q`,
//! and the notes it would write to standard error are left out. A failure is
//! always written to standard error. `--format json` prints one JSON array for
//! the run, as described below.
//!
//! # JSON
//!
//! `--format json` prints one JSON array per run on standard output: one
//! object for each record that is printed, in the order of the text form (a
//! record whose line another record already prints has an object and no line),
//! and `[]` when none is. Every object begins with `kind`, the discriminator, and
//! then holds the fields its kind has, always present and never omitted;
//! `null` stands for a field that does not apply. The member order of an
//! object is fixed. The kinds and their fields are:
//!
//! | `kind` | fields after `kind` |
//! | --- | --- |
//! | a rule of the document model, one of `Finding::KINDS` | `path`, `section` (the heading, or `null`), `line` (or `null`), `key` (the frontmatter key, or `null`), `message`, `fix` |
//! | `rewrote` | `path` |
//! | `state` | `state`, `count`, `capabilities` (objects with `id`, `slug`, `title`, `path`, `gaps` and `open_changes`; a gap has `list`, `binding` or `null`, and `message`) |
//! | `open-changes` | `count`, `changes` (objects with `id`, `slug`, `path` and `capabilities`) |
//! | `preview` | `path`, `capability`, `text` |
//! | `new-change` | `slug`, `path`, `capabilities`, `affects`, `template`, `guidance` (objects with `advice`, `capability` and `state`; the advice is `keep` or `set-implementing`), `sync_needed` |
//! | `impact` | `change`, `capabilities`, `sections`, `bindings` (objects with `capability`, `list`, `path`, `reason` and `resolves`), `dependants` (objects with `capability` and `depends_on`), `linking` (objects with `from` and `target`) |
//! | `obligation` | `obligation` (one of the kinds of `ObligationKind`), `subject`, `section`, `path`, `line` (or `null`), `evidence` |
//! | `refused` | `failures` (the number of `refusal` objects that follow), `message` (the first line of the text form) |
//! | `refusal` | `condition` (`obligations`, `outcomes`, `proposals`, `generated-blocks`, `bound-runs`, `after-removal` or `changed-during-run`), `number` (the number of the condition in the Contract, 1 to 5, or `null`), `source` (the kind of the obligation or finding, or `generated-rewrite`, `bound-test`, `bound-example` or `changed-during-run`), `message`, `repeated` (true when another failure prints the same line, so the text form leaves it out) |
//! | `completed` | `path` (the record removed) |
//! | `binding` | `outcome` (`passed`, `failed` or `unresolved`), `capability`, `list`, `path`, `proof` (the counts), `reason` (or `null`), `message` (the line of the text form) |
//! | `history-entry` | `commit` (the full id), `date` (the author date as `git` printed it), `subject`, `sources` (objects: `{"source":"document"}` or `{"source":"symbol","binding":<path>}`), `paths` |
//! | `history-removed` | `binding`, `identifier`, `dirs`, `lookup` (`found`, `no-commit` or `no-crate`), `commit`, `date`, `subject` and `last_resolved` (each `null` unless found; `last_resolved` also `null` for a first commit) |
//! | `evidence-recorded` | `path`, `id`, `verdict` (`pass` or `fail`), `subject` (`command` or `binding:<path>`), `command` (or `null`), `revision` (or `null`), `by`, `for`, `values` and `limits` (objects with `name` and `value`, the number as its decimal text), `inputs` (objects with `digest`, `kind` and `name`), `failed` (objects with `code`, `names` and `reason`: `exited`, `timed-out`, `over-limit`, `binding`, or `other` for a reason the library gains that this version does not name), `message` (the line of the text form) |
//! | `evidence-claim` | `evidence`, `capability`, `path`, `section` (or `null`), `line` (of the claim marker), `statement` |
//! | `evidence-for` | `capability`, `evidence`, `path`, `verdict` (or `null` for a record that did not load), `listed` (whether its `for` lists the capability), `claims` (the lines of the claims that name it) |
//! | `assess-context` | `subject`, `record`, `record_path`, `document`, `section` (the authored text), `symbols` (objects with `definitions`, objects with `file` and `text`; `named`; `path`; and `unresolved`, or `null`), `evidence` (objects with `id`, `limits`, `path`, `values` and `verdict`), `inputs` (objects with `digest`, `kind`, `name` and `unresolved`) |
//! | `assessment-recorded` | `path`, `id`, `subject`, `verdict` (`accepted` or `rejected`), `by`, `replaced`, `inputs` |
//! | `assessment` | `id`, `path`, `subject`, `verdict`, `by`, `current`, `moved` (objects with `change`: `changed`, `unresolved`, `no-longer-input`, `new-input`, or `other` for a change the library gains that this version does not name; `kind`, `message` and `name`) |
//! | `coverage-gap` | `gap` (`public item`, `unresolved re-export` or `test`), `crate`, `in_scope`, `path`, `item`, `file`, `line` |
//! | `coverage-excluded` | the fields of `coverage-gap`, then `exclusion` and `reason` |
//! | `coverage-error` | `crate`, `in_scope`, `target`, `root`, `failure`, `reexport` (or `null`), `message` |
//!
//! The objects of `history` and `coverage` are the JSON array that
//! `raf_workplan::history::render::json` and `raf_workplan::coverage::render::json`
//! write, printed as they are: the entries of a history and then the bindings
//! that no longer resolve; the gaps of a coverage report, then the exclusion
//! hits, then, for `coverage code`, the surfaces that could not be listed.
//!
//! The kinds that are not rules never equal one, which a test of this crate
//! proves against the full list. A refusal of `change complete` is a `refused`
//! object followed by one `refusal` object for each failure, so a failure that
//! two conditions both state is listed under each; a completion is a
//! `completed` object and a `rewrote` object for each file rewritten. A run that cannot run prints nothing on
//! standard output (a diagnostic on standard error and exit 2), so standard
//! output is either a complete array or empty.
//!
//! The members of a nested object that this crate builds (a capability, a
//! change, a gap, an input, a measure) are in alphabetical order; the nested
//! objects of a history keep the order the library writes. JSON object members
//! carry no meaning by position.
//!
//! # Exit codes
//!
//! * 0: success. No findings, and the command did its work.
//! * 1: findings. `check` found a violation; `change reconcile` printed an
//!   obligation; `change complete` refused; `verify` found a binding that does
//!   not pass; `evidence record` wrote a record whose verdict is `fail`;
//!   `coverage code` or `coverage tests` printed a gap, or `coverage code` a
//!   surface that cannot be listed.
//! * 2: the tool could not run: a usage error (an unknown command or option, a
//!   slug that is not a slug, one that starts with `-`, `--copyright` without
//!   `--licence` or the other way round), a directory that is not a repository,
//!   an unreadable file, a `git` that cannot run or is older than 2.30 (for
//!   `change reconcile` and `change complete`, which ask `git --version` before
//!   any other process of theirs), a slug that has no change record (except for
//!   `change new`, which refuses one that has), a change record that does not
//!   load (for instance an `outcome` that is not one), a sync that refuses a file
//!   or needs an SPDX field that was not given. For the record commands also: an
//!   id, a day, a limit, a revision, a section or a package that is not one or
//!   starts with `-`, an option whose value would be `--`, no command after
//!   `--` and no `--from-binding` (or both), `--verdict` without `--by`, more
//!   or fewer than one of `--context`, `--verdict` and `--list` (usage errors);
//!   an id or a capability no loaded document has (for `assess --list` with a
//!   section too), a shallow clone, a Git older than 2.30 or a bound file whose
//!   lines are not committed for `history`, a program name of `--` or
//!   `--from-binding` beside a `--`, an evidence record that is not written (a
//!   printed value that is not a number, a name printed twice, a limit never
//!   printed, output past its bound, a command that cannot start, a request
//!   refused before the run), a section that cannot be assessed, a package
//!   that `--crate` does not name. A usage error is detected before any
//!   process is started.
//!
//! # Commands
//!
//! * `check` loads the repository through `Gate::open` and prints
//!   `Gate::try_check`, one line for each finding.
//! * `status` prints the roadmap of `Workplan::status`: one row for each state,
//!   in the order delivered, implementing, designed, considered, with the
//!   gaps of each implementing capability and the open changes that list it, and
//!   one row for the open changes. It always exits 0; a gap is not a violation.
//! * `docs sync` writes the generated blocks of `Workplan::sync` through
//!   `Workplan::write` and prints `rewrote <path>` for each file. When the
//!   capability index does not exist it is created from `--copyright` and
//!   `--licence`; they are read only then, and a missing one is exit 2 that
//!   names it. It writes nothing but generated blocks.
//! * `docs preview <slug>` prints `Workplan::preview`: each capability document
//!   of the change as it reads once the change is reconciled, each under a
//!   `==> <path> <==` line. It never writes. An unknown slug is exit 2.
//!
//! * `change new <slug>` writes exactly one file, `work/changes/<slug>.md`, and
//!   prints what `Workplan::new_change` returns: the path, the PROPOSED block to
//!   place, the state each listed capability needs and, when the record makes a
//!   generated block stale, the two steps to take: `git add` the record, then `docs sync`. With `--copyright` and
//!   `--licence` (both, or neither) the record opens with the two SPDX comment
//!   lines of the frontmatter. It reads the repository through `git`, so it
//!   needs `git`; the Git version is not asked.
//! * `change impact <slug>` prints `Workplan::impact`: the capabilities, sections,
//!   bindings, dependants and linking sections of the change. It starts no
//!   process but the `git` that lists the tracked files, and writes nothing.
//! * `change reconcile <slug>` prints `Lifecycle::reconcile`, one line for each
//!   obligation in the shape of the reconcile Contract, and exits 1 exactly when
//!   there is one. It writes no file in the repository; building a bound example
//!   writes into the cargo target directory.
//! * `change complete <slug>` prints `Lifecycle::complete`: `refused:` and the
//!   count of each kind of failure with one line for each failure no other line
//!   repeats, or `completed: removed <path>` with a `rewrote <path>` line for each
//!   file whose generated blocks the removal rewrote. A refusal writes nothing.
//! * `verify [<slug>]` prints `lifecycle::verify`: `pass` or `fail` for each bound
//!   test and example of the change, or of every capability without a slug, with
//!   the counts of its proof or the reason it failed. It asks `git` no question
//!   beyond the listing of tracked files, and writes nothing in the repository. A
//!   run with no binding exits 0 and says so on standard error.
//!
//! * `history <id> [--since <YYYY-MM-DD>]` prints `Workplan::history` as the
//!   table and the JSON array of `raf_workplan::history::render`. The id is a
//!   `cap:`, `change:`, `arch:`, `evidence:` or `assessment:` id of a loaded
//!   document or record. It starts `git` only and writes nothing; a history is
//!   an outcome and exits 0.
//! * `evidence record` runs the command after `--`, or the bound test or
//!   example `--from-binding` names, and writes the record of
//!   `Workplan::record_evidence`, printing `recorded: <path> (<verdict>)` and,
//!   for `fail`, each reason. The command is the `TrustedArgv` that
//!   `raf-workplan` builds from `std::env::args_os()`: every word after the
//!   first `--` of the command line, options and a second `--` included. The
//!   parser reads the same words as its trailing arguments; a run whose two
//!   readings differ is refused before the root is looked for, and an option
//!   whose value would be `--` is a usage error. `--update` replaces the record
//!   of the id; without it a file at the path is refused. A replacement holds
//!   only what its own run gives, its SPDX lines included, so `--update`
//!   without `--copyright` and `--licence` writes a record with none. It
//!   writes exactly that one record and exits 0 for `pass` and 1 for `fail`.
//!   `--from-binding` beside a `--` (even with nothing after it) and a program
//!   name of `--` are refused before the root is looked for.
//! * `evidence claims <evidence-id>` prints `Workplan::evidence_claims`, each
//!   statement as `<path>:<line>: <capability>#<section>` with its text below,
//!   indented by four spaces. `evidence for cap:<slug>` prints
//!   `Workplan::evidence_for`, one line for each record. Both write nothing
//!   and exit 0; an empty answer has a note on standard error.
//! * `assess cap:<slug>#<section> --context` prints `Workplan::assess_context`,
//!   the same whether `--by` says `human`, `agent` or nothing. `--verdict`
//!   writes the record of `Workplan::record_assessment`, one file, and prints
//!   `recorded:` or `replaced:` with its path; a `rejected` verdict exits 0
//!   too; like `--update`, a verdict that replaces a record writes only the
//!   SPDX lines it is given. `--list` prints `Workplan::list_assessments`, or
//!   the assessment of the section given (which must be one `--context`
//!   answers for, so an unknown capability is exit 2), each with whether it is
//!   current and which inputs moved; it exits 0 whatever it shows, since
//!   `change reconcile` lists what a stale or rejected assessment owes.
//! * `coverage code|tests [--crate <package>]` prints one direction of
//!   `Workplan::coverage_report` as the table and the JSON array of
//!   `raf_workplan::coverage::render`, for the whole workspace or the package
//!   named as cargo names it or as its crate is named. A report that names a
//!   gap (or in the code direction a surface that cannot be listed) is a
//!   finding, printed whole under `-q`, and exits 1; one that names none is an
//!   outcome and exits 0. It writes nothing.
//!
//! `check`, `status`, `docs preview`, `change impact`, `change reconcile`,
//! `history`, `evidence claims`, `evidence for`, `assess --context`,
//! `assess --list` and `coverage` leave the tree byte-identical. Under `-q`
//! the outcomes (`change new`, `change impact`, a completion, the passing
//! bindings of `verify`, a history, a `pass` evidence record, the evidence
//! views, the assessment commands, a coverage report without a gap) print
//! nothing; the obligations, the refusal, the failing bindings, a `fail`
//! evidence record and a coverage report with a gap are findings and stay.

mod assess;
mod change;
mod cli;
mod command;
mod coverage;
mod evidence;
mod exit;
mod history;
mod render;

use std::io::{self, Write};
use std::process::ExitCode;

use exit::{Exit, Failure};

fn main() -> ExitCode {
    let args = match cli::parse(std::env::args_os()) {
        Ok(args) => args,
        Err(error) => {
            // Help and the version go to standard output with code 0, a usage
            // error to standard error with code 2.
            let code = error.exit_code();
            let _ = error.print();
            return ExitCode::from(u8::try_from(code).unwrap_or(2));
        }
    };
    ExitCode::from(run(&args, &mut io::stdout().lock(), &mut io::stderr().lock()).code())
}

/// Run the command of `args`, print its report and return how it ended.
fn run(args: &cli::Args, out: &mut dyn Write, diagnostics: &mut dyn Write) -> Exit {
    let report = match command::run(args) {
        Ok(report) => report,
        Err(failure) => return fail(&failure, diagnostics),
    };
    if let Err(error) = report.print(args.format, args.quiet, out, diagnostics) {
        return fail(
            &Failure::new(format!("cannot write the report: {error}")),
            diagnostics,
        );
    }
    report.exit()
}

/// Write a failure to standard error: exit 2.
fn fail(failure: &Failure, diagnostics: &mut dyn Write) -> Exit {
    // A message that cannot be written changes nothing: the exit is 2 either way.
    let _ = writeln!(diagnostics, "cargo-workplan: {failure}");
    Exit::Failed
}
