// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The recursion budgets of the pre-scan, in one table.

use std::fmt;

/// A recursion budget that the pre-scan enforces before a file is parsed.
///
/// Each budget bounds one shape that makes `syn` recurse. The limits live in
/// one table, [`Budget::limit`], which the scan and the documentation share.
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
}

/// The limit of each budget, indexed by the budget's position.
const BUDGETS: [(Budget, usize); 5] = [
    (Budget::Delimiters, 128),
    (Budget::Angles, 64),
    (Budget::PrefixOperators, 256),
    (Budget::ClosureBars, 128),
    (Budget::ElseIf, 256),
];

impl Budget {
    /// Every budget, in table order.
    pub const ALL: [Budget; 5] = [
        BUDGETS[0].0,
        BUDGETS[1].0,
        BUDGETS[2].0,
        BUDGETS[3].0,
        BUDGETS[4].0,
    ];

    /// The most the budget allows; one more is refused.
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
    }
}
