// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The limits and the grouping scope of one census run.

use std::collections::BTreeSet;

use raf_source::{RelPath, TargetKind};

use crate::Error;

/// The default smallest body, in value-form tokens, that has a `body`
/// fingerprint.
pub const DEFAULT_MIN_TOKENS: usize = 20;

/// The default cap on one source file: 8 MiB.
pub const DEFAULT_MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;

/// The options of one census run.
///
/// The options are `#[non_exhaustive]`: build them from [`Default`] and the
/// setters, which refuse a value the census cannot honour.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CensusOptions {
    /// A body with fewer value-form tokens has no `body` fingerprint.
    pub min_tokens: usize,
    /// The most bytes one source file may hold; a larger file is a load error.
    pub max_file_bytes: u64,
    /// The units that body and shim grouping leaves out. The default leaves
    /// out none.
    pub scope: GroupingScope,
}

impl Default for CensusOptions {
    fn default() -> Self {
        Self {
            min_tokens: DEFAULT_MIN_TOKENS,
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
            scope: GroupingScope::default(),
        }
    }
}

/// What duplicate grouping leaves out of one census run.
///
/// Both exclusions apply only to what a library, a binary or a build script
/// reaches. An integration test, a bench or an example keeps every item it
/// reaches in scope, so helpers copied between test files still group. A unit
/// stays in scope while any one reach of it does. An excluded unit is still a
/// unit: detectors, jobs, patterns, homes, sanctions and the unit index see it
/// unchanged; only body and shim groups, and the group debt they carry, omit it.
///
/// The default excludes nothing and leaves every result unchanged.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct GroupingScope {
    exclude_test_items: bool,
    exclude_dirs: BTreeSet<String>,
}

impl GroupingScope {
    /// Leaves test-only items out of body and shim grouping: an item whose own
    /// `cfg`, or an enclosing module's, holds only in a test build (`test`,
    /// `all(.., test, ..)` or an `any(..)` whose every arm does), and a
    /// `#[test]` function.
    #[must_use]
    pub const fn exclude_test_items(mut self, exclude: bool) -> Self {
        self.exclude_test_items = exclude;
        self
    }

    /// Leaves the units of every file under a directory named `name`, at any
    /// depth below the workspace root, out of body and shim grouping. They stay
    /// units, and a job still counts a copy among them.
    ///
    /// # Errors
    ///
    /// [`Error::Options`] when `name` is not one plain directory name: empty,
    /// `.` or `..`, or holding a `/` or a `\`.
    pub fn exclude_dir(mut self, name: impl Into<String>) -> Result<Self, Error> {
        let name = name.into();
        if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']) {
            return Err(Error::Options(format!(
                "excluded directory {name:?} is not one plain directory name"
            )));
        }
        self.exclude_dirs.insert(name);
        Ok(self)
    }

    /// Whether test-only items are left out.
    #[must_use]
    pub const fn excludes_test_items(&self) -> bool {
        self.exclude_test_items
    }

    /// The excluded directory names, in lexical order.
    pub fn excluded_dirs(&self) -> impl Iterator<Item = &str> {
        self.exclude_dirs.iter().map(String::as_str)
    }

    /// True when the scope leaves nothing out.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        !self.exclude_test_items && self.exclude_dirs.is_empty()
    }

    /// Whether one reach of an item stays in the grouping scope: a reach from
    /// a test, bench or example target always does; one from a library,
    /// binary or build script does unless an exclusion names it.
    pub(crate) fn keeps(&self, kind: TargetKind, test_only: bool, file: &RelPath) -> bool {
        !kind.is_shipping()
            || !((self.exclude_test_items && test_only) || self.under_excluded_dir(file))
    }

    fn under_excluded_dir(&self, file: &RelPath) -> bool {
        let dirs = file.segments().count().saturating_sub(1);
        file.segments()
            .take(dirs)
            .any(|dir| self.exclude_dirs.contains(dir))
    }
}

impl CensusOptions {
    /// Sets the smallest body that has a `body` fingerprint.
    ///
    /// # Errors
    ///
    /// [`Error::Options`] when `min_tokens` is 0, which would fingerprint
    /// every empty body.
    pub fn min_tokens(mut self, min_tokens: usize) -> Result<Self, Error> {
        if min_tokens == 0 {
            return Err(Error::Options(
                "min_tokens is 0; it must be at least 1".to_owned(),
            ));
        }
        self.min_tokens = min_tokens;
        Ok(self)
    }

    /// Sets the cap on the size of one source file.
    #[must_use]
    pub fn max_file_bytes(mut self, max_file_bytes: u64) -> Self {
        self.max_file_bytes = max_file_bytes;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_twenty_tokens_and_eight_mebibytes() {
        let options = CensusOptions::default();
        assert_eq!(options.min_tokens, 20);
        assert_eq!(options.max_file_bytes, 8 * 1024 * 1024);
    }

    #[test]
    fn zero_min_tokens_is_refused_and_one_is_accepted() {
        let refused = CensusOptions::default()
            .min_tokens(0)
            .expect_err("0 is refused");
        assert!(matches!(refused, Error::Options(_)));
        let accepted = CensusOptions::default()
            .min_tokens(1)
            .expect("1 is accepted");
        assert_eq!(accepted.min_tokens, 1);
    }

    #[test]
    fn max_file_bytes_is_set() {
        assert_eq!(
            CensusOptions::default().max_file_bytes(10).max_file_bytes,
            10
        );
    }

    /// An excluded directory is matched by exact segment, so only one plain
    /// name can be honoured; a path or a dot name would silently match nothing.
    #[test]
    fn excluded_directories_are_plain_names_matched_only_above_the_file() {
        for refused in ["", ".", "..", "a/b", "a\\b"] {
            let error = GroupingScope::default()
                .exclude_dir(refused)
                .expect_err("not a plain directory name");
            assert!(matches!(error, Error::Options(_)), "{refused:?}");
        }
        let scope = GroupingScope::default()
            .exclude_dir("generated")
            .expect("a plain name");
        assert!(!scope.is_complete());
        let path = |text: &str| RelPath::new(text).expect("relative path");
        assert!(!scope.keeps(TargetKind::Lib, false, &path("a/generated/x.rs")));
        assert!(scope.keeps(TargetKind::Lib, false, &path("a/src/generated.rs")));
        assert!(scope.keeps(TargetKind::Lib, false, &path("a/generated2/x.rs")));
        assert!(scope.keeps(TargetKind::Test, false, &path("a/generated/x.rs")));
    }
}
