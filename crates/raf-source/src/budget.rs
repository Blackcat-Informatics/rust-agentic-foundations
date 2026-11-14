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
    /// Guard token units of one file, counted by the pre-scan. A doc comment
    /// counts as its attribute. A numeric run may fuse up to three native leaves;
    /// [`PreparedSource::native_token_bound`](crate::PreparedSource::native_token_bound)
    /// gives a separate checked native ceiling. This bounds token structure,
    /// not the complete native parser heap or work. [`PARSE_BYTES_PER_TOKEN`]
    /// is an empirical reference for the measured statement shape.
    FileTokens,
    /// Bytes read in one load: every source file and manifest.
    TotalBytes,
    /// Bytes the symbol index keeps: every item record and the index entries
    /// built for each module path, each shared string once, and cumulative
    /// native load traversal spills, and immutable derived glob reach/broken
    /// facts with their shared allocation header.
    ItemBytes,
    /// Bytes of the facts read from the files of one load, counted on the
    /// parse thread as they are built.
    FactBytes,
    /// Bytes of the glob scope tables held at once: what each module exports
    /// to glob imports, kept once for every class of importers that see the
    /// same of it and shared by them, the glob imports that read each table,
    /// and the changes waiting to be passed on as the tables grow. Also
    /// bounds retained named-import reads, queues, subscriptions and graph
    /// scratch used to settle imports, including loads without globs, scratch
    /// for impl self-type and trait-path traversal and derived glob-fact
    /// construction, and
    /// the live scratch of selected-route proofs in a query after the load.
    GlobBytes,
    /// Steps of glob table work in one load, counted as they are made and
    /// never given back: each queue, subscription and import-graph step,
    /// derived glob-fact construction and traversal, admitted buffer growth,
    /// each retained-read traversal, impl self-type and trait-path traversal
    /// and its retained path segments and bytes, each glob read listed and each name gathered when a
    /// view is built, each name a new reader is given, each reader a change is
    /// passed on to, each glob read read again when what a module's globs
    /// bring in under a name is read again, each step of the search for the modules that hold an item
    /// apart from a reading module, and each glob read a lookup of what the
    /// globs of a module bring in looks at, with each view looked at to find
    /// those reads, charged at the next checkpoint of the load. These are the
    /// steps whose number grows faster than the bytes of the tables on some
    /// shapes of globs, so the budget bounds the work of settling them
    /// whatever their bytes. A query after the load counts the steps of its
    /// own lookups and selected-route determinacy proofs against the same budget, with the limit
    /// [`LoadOptions::max_query_work`](crate::LoadOptions::max_query_work).
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

/// Empirical peak bytes per token for the measured empty-statement shape.
///
/// Measured on one function body
/// of 2^k + 1 empty statements, whose list of statements doubles its storage
/// for the last one: 1,602 bytes per token at every k from 14 to 20, counting
/// a grown block as held beside the block it replaces until that is freed.
/// This reference adds about 4% to that measurement. It is not a hard bound
/// for arbitrary accepted Rust: identifier, literal and documentation payloads,
/// native source maps and other transient storage also depend on source bytes.
/// Token count alone does not bound native parsing work, either. The figure
/// spans the whole parse step, lexing included; the syntax tree alone is
/// bounded per node and per byte by [`SYNTAX_BYTES_PER_NODE`].
pub const PARSE_BYTES_PER_TOKEN: usize = 1664;

/// Measured peak bytes of syntax-tree storage per native token-tree node.
///
/// The syntax parse of a function body of 2^k + 1 empty statements, the worst
/// shape of the reference table, peaks at a measured maximum of 939.98 bytes per
/// node, at k = 17 (131,077 nodes); k = 14 gives 939.8 and k = 10 gives 937.5.
/// The residual is taken beyond the measured per-byte rate of 4.000389 bytes per
/// source byte, that is `(peak - 4.000389 * text_bytes) / nodes`, with the
/// counting allocator, which counts a reallocation as growth in place, so the
/// peak is twice the final capacity of the statement list and the replaced block
/// is not counted beside the new one. The constant is 978: a margin of 38.02
/// bytes per node, 4.04% above the maximum. It is measured over a closed table
/// of shapes, not proven for arbitrary accepted Rust. A tree whose retained-tree
/// estimate ([`syntax_held`](crate::syntax_held)) exceeds the bound is detected
/// after the fact and charged; the estimate misses vector capacity, transient
/// parse storage and the heap behind unlisted types. See
/// [`SyntaxBound`](crate::SyntaxBound).
pub const SYNTAX_BYTES_PER_NODE: usize = 978;

/// Numerator of the bytes of syntax-tree storage charged per source byte.
///
/// The per-byte charge is the ratio `SYNTAX_BYTES_PER_BYTE_NUM /
/// SYNTAX_BYTES_PER_BYTE_DEN`, 417/100 = 4.17, taken as `ceil(text_bytes * num /
/// den)` in checked integer arithmetic. A single 1 MiB documentation comment
/// peaks at a measured maximum of 4.000389 bytes per source byte (4,194,764 bytes
/// over 1,048,589), the most of the byte-dense shapes (string literals 3, an
/// identifier 2). The ratio 4.17 is 0.1696 bytes per byte, 4.24%, above that
/// maximum, the same order as the margin of [`SYNTAX_BYTES_PER_NODE`].
pub const SYNTAX_BYTES_PER_BYTE_NUM: usize = 417;

/// Denominator of the per-byte charge; see [`SYNTAX_BYTES_PER_BYTE_NUM`].
pub const SYNTAX_BYTES_PER_BYTE_DEN: usize = 100;

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
/// a smaller host lowers it through [`LoadOptions`](crate::LoadOptions).
/// This limits token structure without establishing a native parser heap bound.
const FILE_TOKENS: usize = 2_000_000;

/// The default limit of import settling and derived graph work in one load,
/// 2^31 steps, and of lookup and proof work in one query after it. The constant
/// is the same on every target; the counted operations are stated by
/// [`Budget::Work`].
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
