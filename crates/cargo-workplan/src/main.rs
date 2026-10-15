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
//! `docs sync`, `docs preview`, `change new`, `change impact`) prints nothing on
//! standard output under `-q`,
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
//! | `obligation` | `obligation` (one of the seven kinds of `ObligationKind`), `subject`, `section`, `path`, `line` (or `null`), `evidence` |
//! | `refused` | `failures` (the number of `refusal` objects that follow), `message` (the first line of the text form) |
//! | `refusal` | `condition` (`obligations`, `outcomes`, `proposals`, `generated-blocks`, `bound-runs`, `after-removal` or `changed-during-run`), `number` (the number of the condition in the Contract, 1 to 5, or `null`), `source` (the kind of the obligation or finding, or `generated-rewrite`, `bound-test`, `bound-example` or `changed-during-run`), `message`, `repeated` (true when another failure prints the same line, so the text form leaves it out) |
//! | `completed` | `path` (the record removed) |
//! | `binding` | `outcome` (`passed`, `failed` or `unresolved`), `capability`, `list`, `path`, `proof` (the counts), `reason` (or `null`), `message` (the line of the text form) |
//!
//! The kinds that are not rules never equal one, which a test of this crate
//! proves against the full list. A refusal of `change complete` is a `refused`
//! object followed by one `refusal` object for each failure, so a failure that
//! two conditions both state is listed under each; a completion is a
//! `completed` object and a `rewrote` object for each file rewritten. A run that cannot run prints nothing on
//! standard output (a diagnostic on standard error and exit 2), so standard
//! output is either a complete array or empty.
//!
//! The members of a nested object (a capability, a change, a gap) are in
//! alphabetical order; JSON object members carry no meaning by position.
//!
//! # Exit codes
//!
//! * 0: success. No findings, and the command did its work.
//! * 1: findings. `check` found a violation; `change reconcile` printed an
//!   obligation; `change complete` refused; `verify` found a binding that does
//!   not pass.
//! * 2: the tool could not run: a usage error (an unknown command or option, a
//!   slug that is not a slug, one that starts with `-`, `--copyright` without
//!   `--licence` or the other way round), a directory that is not a repository,
//!   an unreadable file, a `git` that cannot run or is older than 2.30 (for
//!   `change reconcile` and `change complete`, which ask `git --version` before
//!   any other process of theirs), a slug that has no change record (except for
//!   `change new`, which refuses one that has), a change record that does not
//!   load (for instance an `outcome` that is not one), a sync that refuses a file
//!   or needs an SPDX field that was not given. A usage error is detected before
//!   any process is started.
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
//! `check`, `status`, `docs preview`, `change impact` and `change reconcile`
//! leave the tree byte-identical. Under `-q` the outcomes (`change new`,
//! `change impact`, a completion, the passing bindings of `verify`) print
//! nothing; the obligations, the refusal and the failing bindings are findings
//! and stay.

mod change;
mod cli;
mod command;
mod exit;
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
