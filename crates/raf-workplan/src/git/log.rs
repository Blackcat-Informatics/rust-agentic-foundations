// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Read-only document, line-range and substring-count history over native objects.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::ops::Range;

use super::{GitError, QUERY_BYTES, Repository, answer};
use crate::identifiers::{Commit, Date, Identifier, LineRange, TreePath};
use raf_source::git::{Commit as NativeCommit, Entry, EntryKind};

/// The author date of a commit as `%aI` prints it, strict ISO 8601:
/// `YYYY-MM-DDTHH:MM:SS` and an offset `+HH:MM`, `-HH:MM` or `Z`.
/// The closed Git subset requires uppercase separators, no fraction, and
/// seconds from 00 through 59; leap-second text is refused.
///
/// Dates are ordered by the instant they name and then by their text, so two
/// commits made at one instant in different zones are ordered the same way on
/// every run.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthorDate {
    text: String,
    seconds: i64,
}

impl AuthorDate {
    /// The date `text` names, when it is of the strict form.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        use purrdf_xsd::rfc3339::{self, LeapSecond, Separator};

        let bytes = text.as_bytes();
        // Git %aI has no fraction, lowercase separator or leap-second field.
        let shape = bytes.get(10) == Some(&b'T')
            && match bytes.len() {
                20 => bytes[19] == b'Z',
                25 => matches!(bytes[19], b'+' | b'-'),
                _ => false,
            };
        if !shape || bytes.get(17..19) == Some(b"60") {
            return None;
        }
        let (seconds, _) =
            rfc3339::parse(text, Separator::Rfc3339Abnf, LeapSecond::MonthEnd).ok()?;
        Some(Self {
            text: text.to_string(),
            seconds,
        })
    }

    /// The date as `git` printed it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The day, `YYYY-MM-DD`, as the date writes it in its own zone.
    #[must_use]
    pub fn day(&self) -> &str {
        &self.text[..10]
    }

    /// The instant, in seconds since 1970-01-01T00:00:00Z.
    #[must_use]
    pub fn seconds(&self) -> i64 {
        self.seconds
    }
}

impl Ord for AuthorDate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.seconds
            .cmp(&other.seconds)
            .then_with(|| self.text.cmp(&other.text))
    }
}

impl PartialOrd for AuthorDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for AuthorDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// The instant 00:00:00 UTC of `date` names, in seconds since
/// 1970-01-01T00:00:00Z: the start of a `--since` day, which the history
/// compares with [`AuthorDate::seconds`]. The pattern of [`Date`] holds its
/// text to `YYYY-MM-DD` digits.
pub(crate) fn start_of_day(date: &Date) -> i64 {
    let (year, month, day) =
        purrdf_xsd::rfc3339::parse_date(date.as_str()).expect("Date retains a validated full-date");
    let days = purrdf_xsd::temporal::days_from_civil(i64::from(year), month, day);
    i64::try_from(days * 86_400).expect("Date is bounded to 1970 through 2099")
}

/// One commit of a log: its id, its author date, its subject and, for the
/// document log, the paths it touched as `git` names them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    commit: Commit,
    date: AuthorDate,
    subject: String,
    paths: Vec<String>,
}

impl LogEntry {
    /// The commit.
    #[must_use]
    pub fn commit(&self) -> &Commit {
        &self.commit
    }

    /// The author date.
    #[must_use]
    pub fn date(&self) -> &AuthorDate {
        &self.date
    }

    /// The subject: data, never interpreted.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// The paths the commit touched, in the order `git` listed them; never
    /// empty in a document log, and empty in the logs that list none.
    #[must_use]
    pub fn paths(&self) -> &[String] {
        &self.paths
    }
}

fn row(commit: &NativeCommit, paths: Vec<String>) -> Result<LogEntry, GitError> {
    Ok(LogEntry {
        commit: Commit::new(&commit.id)
            .map_err(|_| answer("history", "a full commit identifier", &commit.id))?,
        date: AuthorDate::parse(&commit.author_date)
            .ok_or_else(|| answer("history", "a strict author date", &commit.author_date))?,
        subject: commit.subject.clone(),
        paths,
    })
}

fn bytes(repository: &Repository, entry: &Entry) -> Result<Vec<u8>, GitError> {
    if !matches!(entry.kind, EntryKind::Blob | EntryKind::Symlink) {
        return Err(answer(
            "history",
            "a file or symbolic-link blob",
            &entry.path,
        ));
    }
    Ok(repository.blob(&entry.id, QUERY_BYTES)?)
}

fn line_diff<'a>(
    before: &'a [u8],
    after: &'a [u8],
) -> (imara_diff::InternedInput<&'a [u8]>, imara_diff::Diff) {
    let input = imara_diff::InternedInput::new(before, after);
    let mut diff = imara_diff::Diff::compute(imara_diff::Algorithm::Myers, &input);
    diff.postprocess_lines(&input);
    (input, diff)
}

fn similarity(before: &[u8], after: &[u8]) -> usize {
    if before == after {
        return 100;
    }
    if before.is_empty() || after.is_empty() {
        return 0;
    }
    let (input, diff) = line_diff(before, after);
    let shared: usize = input
        .before
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            !diff.is_removed(u32::try_from(*index).expect("bounded input line count"))
        })
        .map(|(_, token)| input.interner[*token].len())
        .sum();
    shared.saturating_mul(100) / before.len().max(after.len())
}

fn previous_file(
    repository: &Repository,
    commit: &str,
    parent: &str,
    path: &str,
    current: &Entry,
) -> Result<(String, Option<Entry>), GitError> {
    if let Some(entry) = repository.entry_at(parent, path)? {
        return Ok((path.into(), Some(entry)));
    }
    let mut candidates = Vec::new();
    for candidate in repository.tree(parent, QUERY_BYTES)? {
        if candidate.kind == current.kind
            && matches!(candidate.kind, EntryKind::Blob | EntryKind::Symlink)
            && repository.entry_at(commit, &candidate.path)?.is_none()
        {
            candidates.push(candidate);
        }
    }
    let exact = candidates
        .iter()
        .filter(|candidate| candidate.id == current.id)
        .min_by(|a, b| a.path.cmp(&b.path));
    if let Some(exact) = exact {
        return Ok((exact.path.clone(), Some(exact.clone())));
    }
    let current_bytes = bytes(repository, current)?;
    let mut best = None;
    let mut best_score = 49;
    for candidate in candidates {
        // A length ratio below one half cannot satisfy the similarity threshold.
        if candidate.size.min(current.size).saturating_mul(2) < candidate.size.max(current.size) {
            continue;
        }
        let score = similarity(&bytes(repository, &candidate)?, &current_bytes);
        if score > best_score
            || (score == best_score
                && best
                    .as_ref()
                    .is_some_and(|old: &Entry| candidate.path < old.path))
        {
            best_score = score;
            best = Some(candidate);
        }
    }
    Ok(best.map_or_else(
        || (path.into(), None),
        |entry| (entry.path.clone(), Some(entry)),
    ))
}

fn project_lines(before: &[u8], after: &[u8], range: Range<usize>) -> (bool, Option<Range<usize>>) {
    let (input, diff) = line_diff(before, after);
    let hunks: Vec<_> = diff.hunks().collect();
    let changed = hunks.iter().any(|hunk| {
        let from = hunk.after.start as usize..hunk.after.end as usize;
        if from.is_empty() {
            range.start < from.start && from.start < range.end
        } else {
            from.start < range.end && range.start < from.end
        }
    });
    let boundary = |point: usize, end: bool| {
        let mut offset = 0isize;
        for hunk in &hunks {
            let from = hunk.after.start as usize..hunk.after.end as usize;
            let to = hunk.before.start as usize..hunk.before.end as usize;
            if from.is_empty() && point == from.start {
                return if end { to.start } else { to.end };
            }
            if point <= from.start {
                break;
            }
            if point < from.end {
                return if end { to.end } else { to.start };
            }
            // INVARIANT: each input is byte-bounded far below isize::MAX.
            offset += isize::try_from(to.len()).expect("bounded parent lines")
                - isize::try_from(from.len()).expect("bounded child lines");
        }
        point.saturating_add_signed(offset)
    };
    let start = boundary(range.start, false).min(input.before.len());
    let end = boundary(range.end, true).min(input.before.len());
    (changed, (start < end).then_some(start..end))
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Follow {
    commit: String,
    path: String,
    lines: Option<(usize, usize)>,
}

fn selected_prefix(repository: &Repository) -> Result<String, GitError> {
    let root = repository
        .work_dir()
        .ok_or_else(|| answer("history", "a worktree", "bare repository"))?;
    let prefix = repository.directory().strip_prefix(root).map_err(|error| {
        answer(
            "history",
            "a selected directory within the worktree",
            &error.to_string(),
        )
    })?;
    let text = prefix.to_str().ok_or_else(|| GitError::NotUtf8 {
        reason: "selected directory prefix".into(),
    })?;
    Ok(if text.is_empty() {
        String::new()
    } else {
        format!("{text}/")
    })
}

fn selected_path(prefix: &str, path: &str) -> String {
    if path == "." {
        if prefix.is_empty() {
            ".".into()
        } else {
            prefix.trim_end_matches('/').to_owned()
        }
    } else {
        format!("{prefix}{path}")
    }
}

fn trace(
    repository: &Repository,
    path: &TreePath,
    lines: Option<LineRange>,
) -> Result<Vec<LogEntry>, GitError> {
    let head = repository.resolve_commit("HEAD")?;
    let prefix = selected_prefix(repository)?;
    let selected = selected_path(&prefix, path.as_str());
    let relative = repository.relative_history_paths()?;
    if let Some(lines) = lines {
        let entry = repository
            .entry_at(&head, &selected)?
            .ok_or_else(|| answer("symbol history", "a file committed in HEAD", path.as_str()))?;
        let content = bytes(repository, &entry)?;
        let length = imara_diff::sources::byte_lines(&content).count();
        if lines.start() > length {
            return Err(answer(
                "symbol history",
                "a line range beginning within the committed file",
                path.as_str(),
            ));
        }
    }
    let pending = vec![Follow {
        commit: head,
        path: selected,
        lines: lines.map(|range| (range.start() - 1, range.end())),
    }];
    let rows = trace_rows(repository, pending)?;
    let mut rows: Vec<_> = rows.into_values().collect();
    rows.sort_unstable_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.commit().as_str().cmp(a.1.commit().as_str()))
    });
    let mut entries: Vec<_> = rows.into_iter().map(|(_, entry)| entry).collect();
    if relative {
        for entry in &mut entries {
            for path in &mut entry.paths {
                if let Some(local) = path.strip_prefix(&prefix) {
                    *path = local.to_owned();
                }
            }
        }
    }
    let size = entries
        .iter()
        .map(|entry| entry.subject.len() + entry.paths.iter().map(String::len).sum::<usize>() + 128)
        .sum::<usize>();
    if size > QUERY_BYTES {
        return Err(GitError::LogTooLarge {
            command: "history results".into(),
            limit: QUERY_BYTES,
        });
    }
    Ok(entries)
}

fn trace_rows(
    repository: &Repository,
    mut pending: Vec<Follow>,
) -> Result<BTreeMap<String, (i64, LogEntry)>, GitError> {
    let mut seen = BTreeSet::new();
    let mut rows: BTreeMap<String, (i64, LogEntry)> = BTreeMap::new();
    let mut acquired = 0usize;
    while let Some(follow) = pending.pop() {
        if !seen.insert(follow.clone()) {
            continue;
        }
        acquired = acquired
            .checked_add(follow.path.len() + 128)
            .filter(|n| *n <= QUERY_BYTES)
            .ok_or_else(|| GitError::LogTooLarge {
                command: "history traversal".into(),
                limit: QUERY_BYTES,
            })?;
        let commit = repository.commit(&follow.commit)?;
        let Some(current) = repository.entry_at(&commit.id, &follow.path)? else {
            let mut parents = Vec::new();
            for parent in &commit.parents {
                parents.push((parent, repository.entry_at(parent, &follow.path)?.is_some()));
            }
            let same = parents.iter().position(|(_, present)| !*present);
            if same.is_none() && !parents.is_empty() {
                rows.entry(commit.id.clone()).or_insert((
                    commit.commit_seconds,
                    row(&commit, vec![follow.path.clone()])?,
                ));
            }
            for (index, (parent, _)) in parents.into_iter().enumerate() {
                if same.is_none_or(|selected| selected == index) {
                    pending.push(Follow {
                        commit: parent.clone(),
                        path: follow.path.clone(),
                        lines: None,
                    });
                }
            }
            continue;
        };
        let mut comparisons = Vec::new();
        for parent in &commit.parents {
            let (previous_path, previous) =
                previous_file(repository, &commit.id, parent, &follow.path, &current)?;
            let same = previous
                .as_ref()
                .is_some_and(|entry| entry.id == current.id && entry.kind == current.kind);
            let (changed, range) = if let Some((start, end)) = follow.lines {
                let after = bytes(repository, &current)?;
                let before = previous
                    .as_ref()
                    .map(|entry| bytes(repository, entry))
                    .transpose()?
                    .unwrap_or_default();
                project_lines(&before, &after, start..end)
            } else {
                (!same || previous_path != follow.path, Some(0..0))
            };
            comparisons.push((parent, previous_path, previous, same, changed, range));
        }
        let unchanged = comparisons
            .iter()
            .position(|(_, path, _, same, changed, _)| {
                !*changed && (follow.lines.is_some() || (*same && *path == follow.path))
            });
        let changed = if commit.parents.is_empty() {
            true
        } else {
            unchanged.is_none() && comparisons.iter().any(|(_, _, _, _, changed, _)| *changed)
        };
        if changed {
            let paths = if follow.lines.is_none() {
                vec![follow.path.clone()]
            } else {
                Vec::new()
            };
            let entry = row(&commit, paths)?;
            rows.entry(commit.id.clone())
                .or_insert((commit.commit_seconds, entry));
        }
        for (index, (parent, previous_path, previous, _, _, range)) in
            comparisons.into_iter().enumerate()
        {
            if unchanged.is_some_and(|selected| index != selected) || previous.is_none() {
                continue;
            }
            if follow.lines.is_some() && range.is_none() {
                continue;
            }
            pending.push(Follow {
                commit: parent.clone(),
                path: previous_path,
                lines: follow
                    .lines
                    .and_then(|_| range.map(|range| (range.start, range.end))),
            });
        }
    }
    Ok(rows)
}

/// Whether a repository's history declares a shallow boundary.
///
/// # Errors
/// Refuses malformed or inaccessible shallow metadata.
pub fn is_shallow(repository: &Repository) -> Result<bool, GitError> {
    Ok(repository.is_shallow()?)
}

/// Read a document's full file history, following deleted-file rename candidates.
///
/// # Errors
/// Refuses missing/corrupt objects, unsupported paths and cumulative history bounds.
pub fn document_log(repository: &Repository, path: &TreePath) -> Result<Vec<LogEntry>, GitError> {
    trace(repository, path, None)
}

/// Trace only the selected committed lines, following file renames, not copies.
///
/// # Errors
/// Refuses absent HEAD files, out-of-range starts, corruption and history bounds.
pub fn symbol_log(
    repository: &Repository,
    lines: LineRange,
    file: &TreePath,
) -> Result<Vec<LogEntry>, GitError> {
    trace(repository, file, Some(lines))
}

/// A count-changing commit and its first parent, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastChange {
    entry: LogEntry,
    parent: Option<Commit>,
}
impl LastChange {
    /// The selected count-changing commit.
    #[must_use]
    pub const fn entry(&self) -> &LogEntry {
        &self.entry
    }
    /// Its first parent; absent for a root commit.
    #[must_use]
    pub const fn parent(&self) -> Option<&Commit> {
        self.parent.as_ref()
    }
}

fn counts(
    repository: &Repository,
    commit: &str,
    identifier: &str,
    dirs: &[TreePath],
) -> Result<BTreeMap<String, (String, usize)>, GitError> {
    let mut result = BTreeMap::new();
    for entry in repository.tree(commit, QUERY_BYTES)? {
        if !matches!(entry.kind, EntryKind::Blob | EntryKind::Symlink) {
            continue;
        }
        if !dirs.is_empty()
            && !dirs.iter().any(|dir| {
                dir.as_str() == "."
                    || entry.path == dir.as_str()
                    || entry
                        .path
                        .strip_prefix(dir.as_str())
                        .is_some_and(|suffix| suffix.starts_with('/'))
            })
        {
            continue;
        }
        let bytes = bytes(repository, &entry)?;
        let text = String::from_utf8_lossy(&bytes);
        let count = text.matches(identifier).count();
        result.insert(entry.path, (entry.id, count));
    }
    Ok(result)
}

fn count_change(
    mut before: BTreeMap<String, (String, usize)>,
    mut after: BTreeMap<String, (String, usize)>,
) -> bool {
    let shared: Vec<String> = before
        .keys()
        .filter(|path| after.contains_key(*path))
        .cloned()
        .collect();
    for path in shared {
        if before.remove(&path).map(|(_, count)| count)
            != after.remove(&path).map(|(_, count)| count)
        {
            return true;
        }
    }
    let old = before.into_values().collect::<Vec<_>>();
    let mut new = after.into_values().collect::<Vec<_>>();
    for (id, count) in old {
        if let Some(index) = new.iter().position(|(other, _)| *other == id) {
            new.remove(index);
        } else if count != 0 {
            return true;
        }
    }
    new.into_iter().any(|(_, count)| count != 0)
}

/// Find the newest per-file substring-count change below selected crate paths.
///
/// # Errors
/// Refuses corrupt/missing objects, noncanonical paths and history byte bounds.
pub fn last_change(
    repository: &Repository,
    identifier: &Identifier,
    dirs: &[TreePath],
) -> Result<Option<LastChange>, GitError> {
    let prefix = selected_prefix(repository)?;
    let dirs = dirs
        .iter()
        .map(|path| {
            let selected = selected_path(&prefix, path.as_str());
            if selected == "." {
                return Ok(TreePath::top());
            }
            TreePath::new(&selected).map_err(|error| {
                answer(
                    "pickaxe",
                    "a repository-relative directory",
                    &error.to_string(),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    for commit in repository.ancestors(QUERY_BYTES)? {
        let current = counts(repository, &commit.id, identifier.as_str(), &dirs)?;
        let changed = if commit.parents.is_empty() {
            current.values().any(|(_, count)| *count != 0)
        } else if commit.parents.len() > 1 {
            false
        } else {
            count_change(
                counts(repository, &commit.parents[0], identifier.as_str(), &dirs)?,
                current,
            )
        };
        if changed {
            return Ok(Some(LastChange {
                entry: row(&commit, Vec::new())?,
                parent: commit
                    .parents
                    .first()
                    .map(|id| Commit::new(id))
                    .transpose()
                    .map_err(|_| answer("history", "a full parent commit", &commit.id))?,
            }));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_lines_follow_shifts_and_own_edits_without_unrelated_history() {
        let before = b"prefix\nfn chosen() {}\nsuffix\n";
        let shifted = b"new\nprefix\nfn chosen() {}\nsuffix\n";
        assert_eq!(project_lines(before, shifted, 2..3), (false, Some(1..2)));
        let edited = b"prefix\nfn chosen() { changed(); }\nsuffix\n";
        assert_eq!(project_lines(before, edited, 1..2), (true, Some(1..2)));
        assert_eq!(
            project_lines(b"prefix\nsuffix\n", before, 1..2),
            (true, None)
        );
        assert_eq!(
            project_lines(before, b"prefix\nsuffix\n", 0..2),
            (true, Some(0..3))
        );
    }

    #[test]
    fn author_dates_keep_strict_zone_and_day_semantics() {
        let utc = AuthorDate::parse("2026-05-02T10:00:00Z").expect("UTC");
        let east = AuthorDate::parse("2026-05-02T12:00:00+02:00").expect("east");
        assert_eq!(utc.seconds(), east.seconds());
        assert_eq!(east.day(), "2026-05-02");
        for refused in [
            "2026-05-02",
            "2026-02-29T00:00:00Z",
            "2026-05-02t10:00:00Z",
            "2016-12-31T23:59:60Z",
            "2026-05-02T10:00:00.0Z",
        ] {
            assert!(AuthorDate::parse(refused).is_none(), "{refused}");
        }
        assert_eq!(start_of_day(&Date::new("1970-01-01").expect("day")), 0);
    }
}
