// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The deterministic generator shared by the random families of the tests.

use std::fmt::Write as _;

/// A small deterministic generator, so the random family is the same in every
/// run.
pub(crate) struct Draw(pub(crate) u64);

impl Draw {
    pub(crate) fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// A number below `bound`.
    pub(crate) fn below(&mut self, bound: usize) -> usize {
        let bound = u64::try_from(bound).expect("a small bound");
        usize::try_from(self.next() % bound).expect("below a small bound")
    }

    /// True once in `odds`.
    pub(crate) fn one_in(&mut self, odds: usize) -> bool {
        self.below(odds) == 0
    }
}

/// A random small crate whose carrier `c` holds `carrier`: up to six private
/// modules of `pub`, `pub(crate)` and hidden globs to one another and to the
/// carrier, cycles allowed, binders of `Cell` by a private struct or a
/// private import, hidden modules and a hidden carrier, and a reader at the
/// root or in a `pub` module, sometimes hidden.
pub(crate) fn random_crate(draw: &mut Draw, carrier: &str) -> String {
    let modules = 1 + draw.below(6);
    let target = |draw: &mut Draw, from: Option<usize>| loop {
        let at = draw.below(modules + 1);
        if at == modules {
            return "c".to_owned();
        }
        if Some(at) != from {
            return format!("m{at}");
        }
    };
    let glob = |draw: &mut Draw, to: &str| {
        let hidden = if draw.one_in(4) {
            "#[doc(hidden)] "
        } else {
            ""
        };
        let vis = if draw.one_in(5) { "pub(crate)" } else { "pub" };
        format!("{hidden}{vis} use crate::{to}::*; ")
    };
    let mut lib = String::new();
    let carrier_attr = if draw.one_in(5) {
        "#[doc(hidden)] "
    } else {
        ""
    };
    let module_attr = if draw.one_in(6) {
        "#[doc(hidden)]\n"
    } else {
        ""
    };
    writeln!(lib, "{module_attr}mod c {{ {carrier_attr}{carrier} }}").expect("write");
    for at in 0..modules {
        let mut body = String::new();
        for _ in 0..=draw.below(3) {
            let to = target(draw, Some(at));
            body.push_str(&glob(draw, &to));
        }
        match draw.below(6) {
            0 => body.push_str("struct Cell; "),
            1 => body.push_str("use std::rc::Rc as Cell; "),
            _ => {}
        }
        let module_attr = if draw.one_in(6) {
            "#[doc(hidden)]\n"
        } else {
            ""
        };
        writeln!(lib, "{module_attr}mod m{at} {{ {body}}}").expect("write");
    }
    let mut reader = String::new();
    for _ in 0..=draw.below(3) {
        let to = target(draw, None);
        let hidden = if draw.one_in(3) {
            "#[doc(hidden)] "
        } else {
            ""
        };
        writeln!(reader, "{hidden}pub use crate::{to}::*;").expect("write");
    }
    if draw.one_in(2) {
        let module_attr = if draw.one_in(5) {
            "#[doc(hidden)]\n"
        } else {
            ""
        };
        writeln!(lib, "{module_attr}pub mod r {{\n{reader}}}").expect("write");
    } else {
        lib.push_str(&reader);
    }
    lib
}

impl Draw {
    /// A generator for `seed`, which is mixed so that neighbouring seeds draw
    /// unrelated sequences; the state is never zero.
    pub(crate) fn seeded(seed: u64) -> Self {
        Self(purrdf_hash::mix::splitmix64_step(seed) | 1)
    }

    /// One of `items`, which are not empty.
    pub(crate) fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}
