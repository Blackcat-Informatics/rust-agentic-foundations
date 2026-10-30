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
//! cargo workplan [--root <dir>] [--format text|turtle|ntriples|json] [-q] <command>
//!
//! check                Run every rule of the document model; print each violation
//! status               Print the roadmap: capabilities by state, gaps, open changes
//! docs sync            Rewrite the generated blocks [--copyright <text> --licence <id>]
//! docs preview <slug>  Print the capabilities of a change as they read once reconciled
//! docs export <path>   Export one authoring document; requires --format turtle|ntriples, refuses -q
//! docs mdbook [supports <renderer>]  Run the mdBook JSON protocol; refuses -q and non-text formats
//! docs build standalone --view <reference> --output-dir <directory>
//!                      Publish an owned offline explorer and static artifacts
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
//! review list|show <review-id>
//!                      List review claims or show one recorded claim
//! review status|changed-since <scope> [--kind <kind>]
//!                      Show source currency or the changes since the latest review
//! review unreviewed|coverage [--crate <package>] [--kind <kind>]
//!                      Show unreviewed items or per-crate coverage
//! review record --id <slug> --scope <scope> --kind <kind> --verdict <verdict>
//!               --date <date> --by human|agent [--revision <commit>]
//!               [--findings <path-or-url>] [--copyright <text> --licence <id>]
//!                      Record the actual source inputs at the reviewed commit
//! coverage code|tests [--crate <package>]
//!                      Print the public items or the tests no capability binds
//! coverage claims [--capability cap:<slug>]
//!                      Print each claim of the delivered capabilities and its tests
//! coverage lines [--crate <package>]   (also: coverage --reverse [--crate <package>])
//!                      Print the lines no capability explains, by crate, module and item
//! debt --write [--copyright <text> --licence <id>]
//!                      Write the debt record of the complete policy, work/debt/default.md
//! ```
//!
//! # Root
//!
//! `--root <dir>` names the workspace root. Without it the root is found by
//! walking ancestor manifests through `raf-source`, without starting a process
//! (`raf_workplan::root::discover`).
//!
//! # Complete policy
//!
//! Every check, reconciliation and completion uses the complete content,
//! binding and line policy. There is no policy flag or typed floor. The
//! closed coverage profile refuses the removed policy key. A policy finding
//! is reported like every other finding, in human text or typed RDF.
//!
//! # Output
//!
//! Findings go to standard output in a stable order; diagnostics go to standard
//! error. `-q` prints nothing but findings: a command with no findings (`status`,
//! `docs sync`, `docs preview`, `change new`, `change impact`, `history`,
//! `evidence claims`, `evidence for`, `assess`) prints nothing on standard output
//! under `-q`,
//! and the notes it would write to standard error are left out. A failure is
//! always written to standard error. Output and diagnostic write failures are
//! run failures, exit 2, including a note that cannot be written.
//!
//! # Machine reports
//!
//! `--format turtle` and `--format ntriples` emit the closed canonical RDF 1.2
//! report profile documented in `docs/capabilities/rdf-report-profile.md`.
//! Both formats encode the same typed graph and snapshot identity. Review commands
//! also accept `--format json`, carrying the same native RDF 1.2 dataset as
//! JSON-LD with triple-term support. Other commands refuse JSON before loading.
//! A successful run with no visible result still emits a
//! canonical Report root, command and reportForm, with no result members.
//! `evidence claims` emits a native semantic dataset for RDF formats: marker
//! occurrences reify unasserted capability/claimedStatement/exact-text triple
//! terms and name stable supporting evidence document IRIs. This does not
//! interpret the prose or assert its truth. RDF quiet mode is refused before
//! repository loading; the ordinary text view still supports quiet mode.
//! `docs export` instead emits the selected document's native authoring dataset,
//! with its stable asserted document identity. It requires an explicit RDF format
//! and refuses quiet mode before root or input loading; it has no Report envelope.
//! `docs mdbook` instead uses mdBook's JSON protocol, including its renderer
//! support probe. It requires the default text carrier and refuses quiet mode;
//! a preprocessing response is the transformed Book alone, without a Report.
//!
//! Every command returns typed domain results. Named rows and zero-based
//! ordinals preserve ordered lists: capability groups, bindings, guidance,
//! references, definitions, measures, inputs, claims, commits, paths and ranges.
//! Optional fields are absent when they do not apply. Findings retain their
//! location variant and fix; histories retain full commits; input observations
//! retain actual digests. Counts and booleans are typed literals; measured
//! decimals remain exact decimal text, without conversion to floating point.
//!
//! Verification includes only the invocation rows returned by the executor,
//! their ordered arguments and process outcomes. A planning refusal has no
//! invocation row. Snapshot-local rows do not claim timestamps or independent
//! execution identifiers. Coverage source-location reifiers describe observed
//! item declarations, without asserting a resolved definition or a test run.
//!
//! # Exit codes
//!
//! * 0: success. No findings, and the command did its work.
//! * 1: findings. `check` found a violation; `change reconcile` printed an
//!   obligation; `change complete` refused; `verify` found a binding that does
//!   not pass; `evidence record` wrote a record whose verdict is `fail`;
//!   `coverage code` or `coverage tests` printed a gap, or `coverage code` a
//!   surface that cannot be listed; `coverage claims` printed a claim that
//!   names no test; `coverage lines` printed an orphan line, or could not count
//!   the lines; `debt --write` printed the findings that stop it, a debt that
//!   would grow or a finding no row can hold, and wrote nothing.
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
//!   that `--crate` does not name, a capability that `--capability` of
//!   `coverage claims` names and no loaded document has, `debt` without
//!   `--write`, and a debt record that cannot be written. A usage error is
//!   detected before any process is started.
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
//!   process but the `git` that lists the tracked files, and publishes no files.
//! * `change reconcile <slug>` prints `Lifecycle::reconcile`, one line for each
//!   obligation in the shape of the reconcile Contract, and exits 1 exactly when
//!   there is one. The tool publishes no files. Building or running trusted bound
//!   examples can write files, including files in the workspace.
//! * `change complete <slug>` prints `Lifecycle::complete`: `refused:` and the
//!   count of each kind of failure with one line for each failure no other line
//!   repeats, or `completed: removed <path>` with a `rewrote <path>` line for each
//!   file whose generated blocks the removal rewrote. A refusal publishes no
//!   tool-owned files; trusted bound programs can have filesystem effects.
//! * `verify [<slug>]` prints `lifecycle::verify`: `pass`, `fail` or
//!   `not-applicable` for each bound
//!   test and example of the change, or of every capability without a slug, with
//!   the counts of its proof or the reason it failed. It asks `git` no question
//!   beyond the listing of tracked files, and publishes no files. Trusted Cargo
//!   builds, tests and examples can write files. A run with no binding exits 0
//!   and says so on standard error.
//!
//! * `history <id> [--since <YYYY-MM-DD>]` prints `Workplan::history` as the
//!   human table or typed RDF graph. The id is a
//!   `cap:`, `change:`, `arch:`, `evidence:` or `assessment:` id of a loaded
//!   document or record. It starts `git` only and publishes no files; Git can
//!   fetch missing objects into its metadata. A history is an outcome and exits 0.
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
//!   `Workplan::coverage_report` as a human table or typed RDF, for the whole workspace or the package
//!   named as cargo names it or as its crate is named. A report that names a
//!   gap (or in the code direction a surface that cannot be listed) is a
//!   finding, printed whole under `-q`, and exits 1; one that names none is an
//!   outcome and exits 0. It writes nothing.
//! * `coverage claims [--capability cap:<slug>]` prints
//!   `Workplan::claims_report` as a human table or typed RDF: each
//!   claim of every delivered capability, or of the one named, with its
//!   section, key, line and the tests its tested-by markers name, sorted by
//!   capability, section and line. A report
//!   with a claim that names no test is a finding, printed whole under `-q`,
//!   and exits 1; one whose every claim names a test is an outcome and exits
//!   0. It writes nothing.
//! * `coverage lines [--crate <package>]`, also spelled `coverage --reverse`,
//!   prints `Workplan::lines_report` as human tables or typed RDF, for every
//!   package or the package named: the counted, covered, excepted and orphan
//!   lines of each package with its ceilings, the modules that hold an orphan
//!   line, and the orphan lines of each unit as merged ranges. Every check
//!   enforces the ceilings. A report with an orphan
//!   line, or one that could not count the lines, is a finding, printed whole
//!   under `-q`, and exits 1; one without is an outcome and exits 0. The two
//!   spellings print the same bytes. It writes nothing.
//! * `debt --write` writes the record `work/debt/default.md` of
//!   `Gate::write_debt`, the findings of the complete policy and the orphan lines of
//!   the packages over a ceiling that the repository holds as debt, and prints
//!   `created:`, `replaced:` or `unchanged:` with the path and the rows the
//!   record holds. The write creates the record when there is none, refuses
//!   debt that grew and a finding that no row can hold (it prints those
//!   findings, exits 1 and writes nothing), and lowers the record when debt
//!   was paid. Two writes of one state write the same bytes. `--copyright` and
//!   `--licence` (both, or neither) open a record it creates or replaces with
//!   the two SPDX comment lines; a record that has them keeps them. It writes
//!   that one file and no other.
//!
//! `check`, `status`, `docs preview`, `change impact`, `change reconcile`,
//! `history`, `evidence claims`, `evidence for`, `assess --context`,
//! `assess --list` and `coverage` publish no repository files. The publication
//! guarantees here describe the tool's own writes. Cargo builds, tests, examples
//! and evidence commands are trusted executable input and can write workspace
//! files; they are not sandboxed. Git can fetch missing objects into its metadata
//! in a partial clone without changing working files or the index.
//! Under `-q`
//! the outcomes (`change new`, `change impact`, a completion, the passing
//! bindings of `verify`, a history, a `pass` evidence record, the evidence
//! views, the assessment commands, a coverage report without a gap, a claims
//! report whose every claim names a test, a lines report without an orphan
//! line, a debt record written) print
//! nothing; the obligations, the refusal, the failing bindings, a `fail`
//! evidence record, a coverage report with a gap, a claims report with an
//! uncovered claim and a lines report with an orphan line are findings and
//! stay.

mod assess;
mod change;
mod cli;
mod command;
mod coverage;
mod debt;
mod evidence;
mod exit;
mod history;
mod mdbook;
mod render;
mod result;
mod result_graph;
mod review;

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
            let response = error.print().and_then(|()| {
                if error.use_stderr() {
                    io::stderr().lock().flush()
                } else {
                    io::stdout().lock().flush()
                }
            });
            if let Err(error) = response {
                return ExitCode::from(
                    fail(
                        &Failure::new(format!("cannot write command-line response: {error}")),
                        &mut io::stderr().lock(),
                    )
                    .code(),
                );
            }
            return ExitCode::from(u8::try_from(code).unwrap_or(2));
        }
    };
    ExitCode::from(run(&args, &mut io::stdout().lock(), &mut io::stderr().lock()).code())
}

/// Run the command of `args`, print its report and return how it ended.
fn run(args: &cli::Args, out: &mut dyn Write, diagnostics: &mut dyn Write) -> Exit {
    let report = match command::run(args) {
        Ok(command::Output::Report(report)) => report,
        Ok(command::Output::Mdbook(response)) => {
            return match response.write(out) {
                Ok(()) => response.exit(),
                Err(error) => fail(
                    &Failure::new(format!("cannot write the mdBook response: {error}")),
                    diagnostics,
                ),
            };
        }
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
