---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-entity-history
title: "Entity history"
description: "A history command for a capability or decision and its bindings, built on git log --follow and -L."
state: implementing
bindings:
  code:
    - raf_workplan::history
    - raf_workplan::Workplan::history
    - raf_workplan::git::document_log
    - raf_workplan::git::symbol_log
    - raf_workplan::git::last_change
    - raf_workplan::git::is_shallow
    - raf_workplan::git::AuthorDate
    - raf_workplan::git::LogEntry
    - raf_workplan::git::LastChange
  tests:
    - raf_workplan::test_crate::history::a_binding_that_names_no_single_symbol_is_refused_before_any_process
    - raf_workplan::test_crate::history::a_capability_history_holds_its_document_through_a_move_and_its_bound_code
    - raf_workplan::test_crate::history::a_log_past_sixteen_mebibytes_is_an_error_and_the_bound_itself_is_read
    - raf_workplan::test_crate::history::a_merge_whose_resolution_changed_the_document_is_in_its_history_and_a_clean_one_is_not
    - raf_workplan::test_crate::history::a_module_binding_asks_its_span_and_then_the_log_of_each_of_its_files_in_order
    - raf_workplan::test_crate::history::a_module_binding_lists_the_commits_of_its_files_and_not_of_a_sibling
    - raf_workplan::test_crate::history::a_module_file_that_is_not_committed_names_the_binding_and_the_file_until_it_is
    - raf_workplan::test_crate::history::a_root_commit_that_changed_the_identifier_has_no_parent
    - raf_workplan::test_crate::history::a_shallow_clone_is_refused_with_the_missing_history_named_and_a_full_clone_is_not
    - raf_workplan::test_crate::history::a_subject_with_unit_separators_and_a_hash_is_reported_whole
    - raf_workplan::test_crate::history::a_symbol_log_that_git_refuses_names_the_binding_and_the_file_to_commit
    - raf_workplan::test_crate::history::an_id_or_a_date_that_cannot_be_asked_about_starts_no_process
    - raf_workplan::test_crate::history::an_old_git_or_a_shallow_answer_stops_before_any_history_is_read
    - raf_workplan::test_crate::history::each_kind_of_id_has_the_history_of_its_file
    - raf_workplan::test_crate::history::every_question_is_asked_in_order_as_exactly_its_argv
    - raf_workplan::test_crate::history::since_counts_from_the_start_of_the_day_in_utc_in_every_time_zone
    - raf_workplan::test_crate::history::since_limits_the_logs_and_not_the_removal_lookup
    - raf_workplan::test_crate::history::since_reaches_no_process_and_keeps_each_entry_by_its_author_date
    - raf_workplan::test_crate::history::since_selects_by_the_author_date_and_not_by_the_committer_date
    - raf_workplan::test_crate::history::the_table_and_the_json_carry_the_same_entries_in_the_same_order
    - raf_workplan::test_crate::history::stubs::a_git_reporting_2_29_stops_the_history_before_any_log
    - raf_workplan::test_crate::history::stubs::a_path_named_like_an_option_reaches_git_after_the_separator_or_the_colon
    - raf_workplan::test_crate::history::stubs::a_real_process_that_prints_past_the_bound_is_an_error
    - raf_workplan::git::log::tests::a_document_log_reads_each_record_with_its_paths
    - raf_workplan::git::log::tests::a_log_past_the_bound_is_an_error_that_says_so
    - raf_workplan::git::log::tests::a_since_day_starts_at_midnight_utc_on_the_scale_of_an_author_date
    - raf_workplan::git::log::tests::a_strict_author_date_is_read_and_ordered_by_its_instant
    - raf_workplan::git::log::tests::a_subject_keeps_every_byte_but_a_newline
    - raf_workplan::git::log::tests::a_symbol_log_lists_no_path_and_refuses_any_other_line
    - raf_workplan::git::log::tests::output_that_is_not_a_log_of_the_form_is_an_error
    - raf_workplan::git::log::tests::the_pickaxe_answers_one_record_with_its_parent_or_none
    - raf_workplan::git::log::tests::the_shallow_answer_is_true_or_false
    - raf_workplan::history::render::tests::a_json_string_escapes_quotes_backslashes_and_every_control_character
    - raf_workplan::history::render::tests::a_table_cell_cannot_break_the_table_or_reach_the_terminal
  examples:
    - raf_workplan::Workplan::history
depends: [cap:workplan-document-model, cap:source-index]
---

# Entity history

## Purpose

The current documentation states what applies now and carries no chronology. Some
readers nevertheless need to know how a capability or a decision came to be what it
is. This capability answers that question on request, from Git, so that the answer
costs the asking reader and nobody else.

## Contract

**The command.** `cargo workplan history <id>` takes a capability, change,
architecture, evidence or assessment id (`cap:`, `change:`, `arch:`, `evidence:` or
`assessment:`) and prints the commits that shaped it, newest first. Each entry holds
the commit, its date, its subject, the sources that list it and the paths of the
document and of the bound files it touched. A commit that several sources list is one entry with all of them.
Entries are ordered by the instant of their author date, newest first, and then by
commit id, so the order is the same on every run. `--since <date>` keeps the
entries whose author date is at or after 00:00:00 UTC of that day, so every entry
shown satisfies it by the date it shows and is ordered by.

**Document history.** The history of the document comes from
`git log --follow` on its path, so a move of the file keeps its earlier commits.
`--name-only` lists the name the document had at each commit. `--cc` lists a merge
whose resolution changed the document; a merge whose result equals one of its
parents is not an entry. `--root` lists the commit that created the document even when a user
sets `log.showRoot=false`.

**Binding history.** For each resolved code binding, the report reads the span of
each definition of the symbol (its file and lines) from the
[source index](./source-index.md) and runs `git log -L` over those lines, so the
report lists the commits that changed that symbol and not the whole file. Their
source is the binding's path and their path is the file. A binding that names
more than one symbol, or that the index cannot decide, is an error and no
process starts.

> [!PROPOSED change:workplan-entity-history]
> **Binding history.** For each resolved code binding, the report reads the span of
> each definition of the symbol (its file and lines) from the
> [source index](./source-index.md) and runs `git log -L` over those lines, so the
> report lists the commits that changed that symbol and not the whole file. Their
> source is the binding's path and their path is the file. A binding that names
> more than one symbol, or that the index cannot decide, is an error and no
> process starts. When `git log -L` cannot read the lines of a symbol from `HEAD`,
> as for a bound file that is not committed or that `HEAD` holds fewer lines of,
> the whole history fails with an error (`HistoryError::SymbolLog`) that names the
> binding and the file, says that `git log -L` could not read the lines from `HEAD`
> with what `git` said, and names committing the file with the symbol at the lines
> it has now as one way to read them, not as the cause.

A binding that resolves to a module below its crate root stands for the files of
the module, as its digest does for the evidence and assessments that rest on it
(see [prose assessment](./workplan-prose-assessment.md)): the module's own file
and the file of every module below it in the same target, in the order of their
paths. Besides the log of its own span (the declaration `mod name;`, or the
braces of an inline module), each of those files has the log a document has, so
an edit anywhere in the module, in its own file or in a child module's file, is
listed, and an edit of a sibling module is not. Those commits have the binding's
path as their source and the names `git` lists as their paths. A crate root
stands for nothing below it and has the log of its span only, as every item that
is not a module has; an inline module with no module file below it has the log of
its braces only. A file of the module that no commit holds has an empty log, and
the history fails with an error that names the binding and the file and says to
commit it, rather than report the file as unchanged.

A binding that no longer resolves is reported with the commit that removed it
and that commit's parent as the last commit in which it resolved. The commit is
the newest one that changed how often the last segment of the binding's path
occurs in the files of the directory of its crate (`git log -S`); its first
parent comes from `%P` of the same query, and the first commit of the history
has none. A
crate that no package of the workspace provides has no directory, and the report
says so with no process started.

**Invocation.** Every `git` process is spawned directly with no shell and with its
arguments as an argv vector, in the workspace root and with `LC_ALL=C`. Loading
the repository lists its tracked paths with `git`, as it does for every command of
the [command line](./workplan-cli.md). Then, before any question of its own, the
history runs `git --version` and exits 2 when Git is older than 2.30, the minimum
for the suite, and then `git rev-parse --is-shallow-repository`, and exits 2 when
it answers `true`. The history commands are:

```text
git --no-pager log --no-show-signature --follow --cc --root --format=%x1e%H%x1f%aI%x1f%s --name-only -- <doc-path>
git --no-pager log --no-show-signature -L<start>,<end>:<file> --no-patch --format=%x1e%H%x1f%aI%x1f%s --
git --no-pager log --no-show-signature --root -S<identifier> -n 1 --format=%H%x1f%P%x1f%aI%x1f%s -- <crate-dir>...
```

The first form runs once for the document and once for each file of a bound
module, which is then its `<doc-path>`. The questions run in this order: the log
of the document, the log of each span, by binding path, the log of each file of
a bound module, by binding path and then file path, and the lookup of each
binding that no longer resolves. `<doc-path>`, `<file>` and `<crate-dir>` are
paths relative to the workspace root that the command takes from the loaded
document set and the [source index](./source-index.md), never from the raw command
line; `<crate-dir>` is `.` for a package at the root. In the first and third forms
`--` precedes the paths. In the second the path is embedded after the colon in
the single `-L` argument, so a path that begins with `-` is a path and not an
option. `<identifier>` is the last segment of the binding's path without a
generic argument list or a raw-identifier prefix `r#`, and must match
`[A-Za-z_][A-Za-z0-9_]*`. Each commit id `git` prints, the parents of the third form included, is accepted only when it
matches the commit-id pattern. `--no-show-signature` keeps a user's
`log.showSignature` from adding lines to the output, and `--root` a user's
`log.showRoot=false` from hiding the first commit. The `<id>` argument is matched against the ids of the loaded documents
and is never passed to `git`. `--since <date>` must match `YYYY-MM-DD` and name a
day from 1970-01-01 to 2099-12-31. It counts from 00:00:00 UTC of that day, so the
same date selects the same commits whatever the clock and the time zone. It
reaches no process: Git's own `--since` selects by the committer date, which a
rebase or an amend moves away from the author date, so every log is read whole
and its entries are kept or left out in process by the instant of their author
date. `--since` does not limit the lookup of a removed binding.

Each record of the first two forms opens with `0x1e`, and its first line is split
with at most two splits at `0x1f` (commit, date, then the rest as the subject; the
third form has the parents after the commit and is split at most three times),
because the subject is text an attacker chose and may contain any byte except a
newline. The subject is data and is never interpreted. The lines after it in the
first form are the paths, unquoted when `git` quoted them; any other line is an
error. The command reads at most 16 MiB of each `git` output and fails with an
error that says so when there is more. It runs against the workspace's
repository and Git configuration, which is trusted build input (see
[the security policy](../../SECURITY.md)).

**Output.** The default output is a table: `| Commit | Date | Subject | Source |`,
the commit abbreviated to seven digits, the day of the author date as it is
written in the author's own offset, the subject and the sources, and a second
table for the bindings that no longer resolve. The day shown can therefore be the
day before the `--since` day for a commit authored west of UTC shortly after
00:00:00 UTC.
`--format json` emits the same entries, in the same order, for tools, with the
full commit id and the author date as `git` printed it. The command is read only
and writes no file.

## Usage

A maintainer asks how a design reached its present form:

- `cargo workplan history cap:workplan-reconcile` lists the document commits and the
  commits of each bound symbol.
- `cargo workplan history cap:workplan-reconcile --since 2026-01-01` limits the range
  to the commits authored from that day on.
- `cargo workplan history cap:workplan-current-rationale` lists, for its module
  binding `raf_workplan::rationale`, every commit that changed
  `crates/raf-workplan/src/rationale.rs`.
- `cargo workplan history arch:normal-form` reports the history of a shared decision.

An agent that needs the reasoning behind a contract runs the command in its own
investigation. Nothing in the capability document needs to change for this.

## Examples

A table for one capability with one bound method. The oldest commit wrote both
the document and the method, so it is one entry with both sources:

| Commit | Date | Subject | Source |
| --- | --- | --- | --- |
| 3f9c2a1 | 2026-05-02 | Reject duplicate obligations | document |
| b7d014e | 2026-04-18 | Group obligations by section | `raf_workplan::Workplan::reconcile` |
| 91ac5d0 | 2026-03-30 | Add the reconcile capability | document, `raf_workplan::Workplan::reconcile` |

## Constraints

- The command only reads, from Git and the loaded documents. It never writes the
  working tree or the index. In a partial clone Git itself may fetch missing
  objects from its remote while it answers, which writes into `.git`; that is Git's behaviour for such a clone.
- A history entry always names its commit, so every line is checkable with plain Git.
- The paths of an entry have two bases when the workspace lies below the top of its
  Git work tree. The paths of the document and of the files of a bound module are
  the names `git log --name-only` prints, relative to the top of the work tree
  (relative to the workspace root under a user's `diff.relative=true`); the file of
  a bound symbol's span is relative to the workspace root, as the source index
  names it.
- A shallow clone produces an error that names the missing history, and never
  a partial list presented as complete.
- No `git` process receives text from a document or the command line except as a
  validated path (after `--` or inside the single `-L` argument) or as a validated
  identifier inside the single `-S` argument. A `--since` value reaches no
  process. A path that begins with `-` is treated as a path.
- `--since` selects by the author date and does not narrow what `git` reads: each
  log is read whole, up to its 16 MiB bound, and filtered in process, so a log
  past the bound is an error whatever `--since` says.
- `git log -L` follows the lines of a symbol through a rename that Git detects
  and ends where the lines came from another file, so the history of a symbol
  moved between files starts at the move. The span is read from the working tree
  and traced from `HEAD`, so for a file whose working copy differs from `HEAD`
  the lines are those numbers in `HEAD`. A bound file that is not committed, or
  is shorter in `HEAD`, makes `git log -L` fail, and the whole history fails with
  it rather than leaving the binding out, with an error that names the binding
  and the file and says to commit it.

> [!PROPOSED change:workplan-entity-history]
> - `git log -L` follows the lines of a symbol through a rename that Git detects
>   and ends where the lines came from another file, so the history of a symbol
>   moved between files starts at the move. The span is read from the working tree
>   and traced from `HEAD`, so for a file whose working copy differs from `HEAD`
>   the lines are those numbers in `HEAD`. A bound file that is not committed, or
>   is shorter in `HEAD`, makes `git log -L` fail, and the whole history fails with
>   it rather than leaving the binding out, with an error that names the binding
>   and the file, says that `git log -L` could not read the lines from `HEAD`, and
>   names committing the file as one way to read them.
- The history of a bound module is the history of its whole files: a commit that
  changed any line of one of them, a comment or a unit test included, is listed,
  as the module's digest moves with that edit. The log of each file holds its
  committed versions and follows the file through a rename that Git detects; a
  working copy that differs from `HEAD` is in no commit. A file of the module that
  no commit holds, such as a new file never committed, makes the whole history fail with an
  error that names the binding and the file and says to commit it, because `git
  log` answers such a file with no commit and no error.
- The commit reported as removing a binding is a heuristic. The count is of the
  text anywhere below the crate's directory, which includes any package nested in
  it and, for a package at the root, the whole repository with its documents; so
  a later commit that changes another occurrence of the same word, or of a word
  that holds it, is reported instead. When the symbol left without changing the
  count (a rename to a longer name that holds the old one, a move inside the
  crate) the commit reported can be the one that added it, and the last commit
  in which it resolved one in which it never existed. A removal inside a merge is
  not seen, and a symbol whose name never changed its count there has no commit.
- A table cell writes a control character or a bidirectional formatting
  character of a subject as its `\u{..}` escape, so a subject cannot reorder or
  hide the text of a row.
- The command does not alter what the capability documents say. History is reached on
  request and is never embedded in a document.

## Explanation

Current documentation and history have different readers. The reader who wants to
change or use a capability needs the complete contract that applies and does not need
the proposals that preceded it. The reader who is investigating a surprising decision
needs the sequence. Keeping the sequence in Git, and giving that reader a command to
reach it, lets the first reader work without carrying the second reader's context.

`git log --follow` suits the document because a capability keeps one permanent path,
and the flag keeps the trail through a move. `git log -L` suits the bindings because
the question about a symbol is which commits changed it, and a whole-file log buries
that answer in unrelated edits. Both are commands that a maintainer can repeat by hand,
so the report is a convenience and not a second source of truth.

The distinction between current rationale and historical deliberation (see
[current rationale](./workplan-current-rationale.md)) is what this command completes:
the document keeps the reasons that still govern, and the history command reaches the
sequence of proposals, discarded names and reversals that Git retains.

## Verification

- A fixture repository where a document is renamed: the report includes the commits
  from before the rename.
- A fixture where a function changes in one commit among several that touch its file:
  the report lists that commit and the one that wrote the function, and none of
  the others.
- A module fixture: for a binding of a file module, an edit in the module's own
  file and an edit in a child module's file are listed, and an edit of a sibling
  module and of a function of the crate root are not; a binding of the crate root
  lists the edits of its own file and none of its modules' files, and an inline
  module the edits inside its braces. A scripted runner shows the log of the span
  followed by the log of each file of the module, in the order of their paths,
  and never the sibling's file.
- A module file that no commit holds: the history fails with an error that names
  the binding and the file and says to commit it, and once the file is committed
  the module has its history.
- A binding whose symbol was deleted: the report names the commit that removed it
  and its parent as the last commit in which it resolved.
- A merge fixture: a merge whose conflict resolution changed the document is in
  its history, and a clean merge, whose result equals one of its parents, is not.
- A configuration fixture: with `log.showRoot=false` set in the repository, the
  commit that created the document, and a symbol added in it, are found.
- A time fixture: commits at 23:30 UTC the day before and at 00:30 and 23:30 UTC
  on the day of `--since` give exactly the two of that day, under several `TZ`
  values.
- A date fixture: a commit authored before the `--since` day and committed on it
  is left out, and one authored on the day and committed before it is listed;
  the day before the oldest author date lists both. A scripted log shows that no
  argument of any process carries the date, and that an author date one second
  before 00:00:00 UTC of the day is left out and one at that instant, in another
  offset, is kept.
- A path fixture: a document and a bound symbol whose file is named `-rf.rs` (and
  one named `--output=x.rs`) produce their history and create no file named
  `x.rs`. A recording stub `git`, given to the library as the program it starts
  (`Programs::with_git`), shows `log --follow ... -- -rf.rs` for the document form
  and `-L` carrying the path after the colon for the symbol form.
- An order fixture: the questions are asked in the order above, each as exactly
  its argv; `--since` reaches no `git` process and does not limit the lookup of a
  removed binding.
- A record fixture: a commit subject that contains `0x1f` bytes and text that looks
  like a hash is reported whole as the subject, and output past 16 MiB is an error.
- A Git version fixture: a stub `git` given to the library that reports 2.29 stops
  the history before any log is read; a scripted runner shows that an answer of
  2.29 asks nothing more and a shallow answer nothing after it. Through the
  binary, a recording `git` stub first on `PATH` that reports 2.29 makes
  `history` exit 2 (a fixture of the [command line](./workplan-cli.md)).
- An argument fixture: an `<id>` of another kind or one no document has (`-rf`,
  `--output=x` and `cap:-rf` among them) and a `--since` value that begins with
  `-` or does not match its pattern are refused, and no `git` process starts;
  the command line reports each as a usage error with exit 2.
- A binding that names more than one symbol, or that the index cannot decide, is
  refused before any process starts.
- A shallow clone: the history fails with an error that names the missing
  history, and a full clone of the same origin answers.
- A bound file whose lines `git log -L` cannot read from `HEAD` (as for a file that
  is not committed): the error names the binding and the file and says to commit
  it, while a failing log of the document stays a failure of `git`.

> [!PROPOSED change:workplan-entity-history]
> - A bound file whose lines `git log -L` cannot read from `HEAD`: through the
>   library, the error names the binding and the file, starts with `git log -L`
>   could not read the lines from `HEAD`, quotes what `git` said and ends naming
>   committing the file as one way to read them, while a failing log of the document
>   stays a failure of `git`; through the binary, a file written after the last
>   commit is exit 2 with that message, and has a history once it is committed.
- The JSON output and the table carry the same entries in the same order.
- Each kind of id (`cap:`, `change:`, `arch:`, `evidence:`, `assessment:`) has
  the history of its file.
- The documentation example of `Workplan::history` runs: an id no document has is
  refused before any process starts.
