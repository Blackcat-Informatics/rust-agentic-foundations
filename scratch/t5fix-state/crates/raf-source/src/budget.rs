// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The budgets of a load, in one table: the recursion budgets of the pre-scan,
//! the bounds of the module walk and of discovery, and the work and memory
//! bounds of one load.

use std::fmt;

/// A budget of a load.
///
/// The first six bound shapes that make `syn` recurse and are enforced by the
/// pre-scan before a file is parsed. The others bound the module walk,
/// discovery, the tokens of one file, the bytes one load reads and keeps and
/// the steps of glob table work one load makes. The limits live in one table,
/// [`Budget::limit`], which the code and the documentation share. The token
/// budget, the four byte budgets and the work budget are defaults that
/// [`LoadOptions`](crate::LoadOptions) can change.
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
    /// The nesting that the parser builds by recursion, counted additively
    /// over every open delimiter level: the counted tokens of each open level
    /// since its count last returned to zero, plus one for each open
    /// delimiter. Parsing a file, reading its facts and dropping its syntax
    /// tree take at most [`RECURSION_UNIT_BYTES`] of the parse stack for each
    /// unit, so the limit keeps them within half of
    /// [`PARSE_STACK_BYTES`](crate::PARSE_STACK_BYTES).
    Recursion,
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
    /// Bytes of the glob scope tables held at once: what each module exports
    /// to glob imports, kept once for every class of importers that see the
    /// same of it and shared by them, the glob imports that read each table,
    /// and the changes waiting to be passed on as the tables grow.
    GlobBytes,
    /// Steps of glob table work in one load, counted as they are made and
    /// never given back: each glob read listed and each name gathered when a
    /// view is built, each name a new reader is given, each reader a change is
    /// passed on to, each glob read read again when what a module's globs
    /// bring in under a name is read again, and each step of the search for the modules that hold an item
    /// apart from a reading module. These are the steps whose number grows
    /// faster than the bytes of the tables on some shapes of globs, so the
    /// budget bounds the work of settling them whatever their bytes.
    Work,
}

/// One mebibyte.
const MIB: usize = 1024 * 1024;

/// The limit of each budget, indexed by the budget's position.
const BUDGETS: [(Budget, usize); 15] = [
    (Budget::Delimiters, 128),
    (Budget::Angles, 64),
    (Budget::PrefixOperators, 256),
    (Budget::ClosureBars, 128),
    (Budget::ElseIf, 256),
    (Budget::Recursion, RECURSION),
    (Budget::ModuleDepth, 128),
    (Budget::ModulePaths, 262_144),
    (Budget::DirectoryDepth, 128),
    (Budget::FileTokens, FILE_TOKENS),
    (Budget::TotalBytes, 256 * MIB),
    (Budget::ItemBytes, 512 * MIB),
    (Budget::FactBytes, 256 * MIB),
    (Budget::GlobBytes, 512 * MIB),
    (Budget::Work, WORK),
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

/// The most stack of the parse thread that one unit of [`Budget::Recursion`]
/// takes in a debug build, rounded up: parsing, reading the facts and dropping
/// the syntax tree, measured on the costliest counted shape, generic arguments
/// nested in a type (`A<A<...>>`), at 16,481 bytes a unit.
pub const RECURSION_UNIT_BYTES: usize = 17 * 1024;

/// The recursion limit, 1927: half of the parse stack over the cost of one
/// unit.
const RECURSION: usize = crate::parse::PARSE_STACK_BYTES / (2 * RECURSION_UNIT_BYTES);

/// The default token limit of one file. It admits the largest generated source
/// files of real crates (a time-zone table of 1.5 million tokens); a caller on
/// a smaller host lowers it through [`LoadOptions`](crate::LoadOptions), to
/// 320,000 tokens to keep the peak below 512 MiB.
const FILE_TOKENS: usize = 2_000_000;

/// The default limit of the glob table work of one load, 2^31 steps: the same
/// on every target. It admits every load of a local registry of 1271 crates,
/// the most work of one being 34343004 steps, with a margin of about 62 times;
/// a hub of 1024 siblings is the first shape of the hub pattern to pass it.
const WORK: usize = 1 << 31;

impl Budget {
    /// Every budget, in table order.
    pub const ALL: [Budget; 15] = [
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
        BUDGETS[12].0,
        BUDGETS[13].0,
        BUDGETS[14].0,
    ];

    /// The most the budget allows; one more is refused. For the token budget,
    /// the four byte budgets and the work budget this is the default of the
    /// matching [`LoadOptions`](crate::LoadOptions) field.
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
            Self::Recursion => "parser recursion",
            Self::ModuleDepth => "module nesting",
            Self::ModulePaths => "module paths",
            Self::DirectoryDepth => "directory nesting",
            Self::FileTokens => "tokens in one file",
            Self::TotalBytes => "bytes read in one load",
            Self::ItemBytes => "bytes of index records",
            Self::FactBytes => "bytes of file facts",
            Self::GlobBytes => "bytes of glob tables",
            Self::Work => "glob table work steps",
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
        assert_eq!(Budget::Recursion.limit(), 1927);
        assert_eq!(
            RECURSION,
            crate::PARSE_STACK_BYTES / (2 * RECURSION_UNIT_BYTES)
        );
        assert_eq!(Budget::ModuleDepth.limit(), 128);
        assert_eq!(Budget::ModulePaths.limit(), 262_144);
        assert_eq!(Budget::DirectoryDepth.limit(), 128);
        assert_eq!(Budget::FileTokens.limit(), FILE_TOKENS);
        assert_eq!(Budget::TotalBytes.limit(), 256 * MIB);
        assert_eq!(Budget::ItemBytes.limit(), 512 * MIB);
        assert_eq!(Budget::FactBytes.limit(), 256 * MIB);
        assert_eq!(Budget::GlobBytes.limit(), 512 * MIB);
        assert_eq!(Budget::Work.limit(), 1 << 31);
        assert_eq!(Budget::Work.name(), "glob table work steps");
        assert_eq!(Budget::ALL.last(), Some(&Budget::Work));
    }
}
