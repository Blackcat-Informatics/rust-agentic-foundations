// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The budgets of a load, in one table: the recursion budgets of the pre-scan,
//! the bounds of the module walk and of discovery, and the work and memory
//! bounds of one load.

use std::fmt;

/// A budget of a load.
///
/// The first five bound shapes that make `syn` recurse and are enforced by the
/// pre-scan before a file is parsed. The others bound the module walk,
/// discovery, the tokens of one file and the bytes one load reads and keeps.
/// The limits live in one table, [`Budget::limit`], which the code and the
/// documentation share. The token budget and the three byte budgets are
/// defaults that [`LoadOptions`](crate::LoadOptions) can change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Budget {
    /// Open delimiters (`(`, `[`, `{`) nested at one point.
    Delimiters,
    /// Open `<` brackets nested in one delimiter level.
    Angles,
    /// Consecutive prefix operators (`!`, `-`, `*`, `&`, `&&`).
    PrefixOperators,
    /// Back-to-back closure heads (`||` or `|...|` with no body between).
    ClosureBars,
    /// Chained `else if` links.
    ElseIf,
    /// Modules nested along one path from a target root, files and inline
    /// modules together. Unlike the others it is counted by the module walk
    /// and not by the pre-scan.
    ModuleDepth,
    /// Module paths recorded in one load: every arrival of the module walk at
    /// a file, counted over all targets. Counted by the module walk.
    ModulePaths,
    /// Directories nested below a root, counted by package discovery and by
    /// the expansion of `workspace.members` patterns.
    DirectoryDepth,
    /// Tokens of one file, counted by the pre-scan. A doc comment counts as
    /// the tokens of the attribute it stands for. Parsing one file holds at
    /// most [`PARSE_BYTES_PER_TOKEN`] bytes per token at its peak, so the limit
    /// bounds that peak.
    FileTokens,
    /// Bytes read in one load: every source file and manifest.
    TotalBytes,
    /// Bytes the symbol index keeps: every item record and the index entries
    /// built for each module path, and each shared string once.
    ItemBytes,
    /// Bytes of the facts read from the files of one load, counted on the
    /// parse thread as they are built.
    FactBytes,
}

/// One mebibyte.
const MIB: usize = 1024 * 1024;

/// The limit of each budget, indexed by the budget's position.
const BUDGETS: [(Budget, usize); 12] = [
    (Budget::Delimiters, 128),
    (Budget::Angles, 64),
    (Budget::PrefixOperators, 256),
    (Budget::ClosureBars, 128),
    (Budget::ElseIf, 256),
    (Budget::ModuleDepth, 128),
    (Budget::ModulePaths, 262_144),
    (Budget::DirectoryDepth, 128),
    (Budget::FileTokens, FILE_TOKENS),
    (Budget::TotalBytes, 256 * MIB),
    (Budget::ItemBytes, 512 * MIB),
    (Budget::FactBytes, 256 * MIB),
];

/// The most memory parsing one file holds at its peak for each of its tokens.
///
/// Measured on the costliest shape the other budgets admit, one function body
/// of 2^k + 1 empty statements, whose list of statements doubles its storage
/// for the last one: 1,602 bytes per token at every k from 14 to 20, counting
/// a grown block as held beside the block it replaces until that is freed.
/// The bound adds about 4% to that. Multiply a token limit by it to bound the
/// peak of one file: about 3.1 GiB at the default of [`Budget::FileTokens`],
/// and 512 MiB at 322,638 tokens.
pub const PARSE_BYTES_PER_TOKEN: usize = 1664;

/// The default token limit of one file. It admits the largest generated source
/// files of real crates (a time-zone table of 1.5 million tokens); a caller on
/// a smaller host lowers it through [`LoadOptions`](crate::LoadOptions), to
/// 320,000 tokens to keep the peak below 512 MiB.
const FILE_TOKENS: usize = 2_000_000;

impl Budget {
    /// Every budget, in table order.
    pub const ALL: [Budget; 12] = [
        BUDGETS[0].0,
        BUDGETS[1].0,
        BUDGETS[2].0,
        BUDGETS[3].0,
        BUDGETS[4].0,
        BUDGETS[5].0,
        BUDGETS[6].0,
        BUDGETS[7].0,
        BUDGETS[8].0,
        BUDGETS[9].0,
        BUDGETS[10].0,
        BUDGETS[11].0,
    ];

    /// The most the budget allows; one more is refused. For the token budget
    /// and the three byte budgets this is the default of the matching
    /// [`LoadOptions`](crate::LoadOptions) field.
    #[must_use]
    pub const fn limit(self) -> usize {
        BUDGETS[self as usize].1
    }

    /// The budget's name as it appears in messages.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Delimiters => "delimiter nesting",
            Self::Angles => "nested `<`",
            Self::PrefixOperators => "prefix operator run",
            Self::ClosureBars => "closure bar chain",
            Self::ElseIf => "`else if` chain",
            Self::ModuleDepth => "module nesting",
            Self::ModulePaths => "module paths",
            Self::DirectoryDepth => "directory nesting",
            Self::FileTokens => "tokens in one file",
            Self::TotalBytes => "bytes read in one load",
            Self::ItemBytes => "bytes of index records",
            Self::FactBytes => "bytes of file facts",
        }
    }
}

impl fmt::Display for Budget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_indexed_by_each_budgets_position() {
        for (index, (budget, _)) in BUDGETS.iter().enumerate() {
            assert_eq!(*budget as usize, index);
        }
        assert_eq!(Budget::ALL.len(), BUDGETS.len());
        assert_eq!(Budget::Delimiters.limit(), 128);
        assert_eq!(Budget::Angles.limit(), 64);
        assert_eq!(Budget::PrefixOperators.limit(), 256);
        assert_eq!(Budget::ClosureBars.limit(), 128);
        assert_eq!(Budget::ElseIf.limit(), 256);
        assert_eq!(Budget::ModuleDepth.limit(), 128);
        assert_eq!(Budget::ModulePaths.limit(), 262_144);
        assert_eq!(Budget::DirectoryDepth.limit(), 128);
        assert_eq!(Budget::FileTokens.limit(), FILE_TOKENS);
        assert_eq!(Budget::TotalBytes.limit(), 256 * MIB);
        assert_eq!(Budget::ItemBytes.limit(), 512 * MIB);
        assert_eq!(Budget::FactBytes.limit(), 256 * MIB);
    }
}
