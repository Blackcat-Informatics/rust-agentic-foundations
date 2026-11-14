// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The storage of a parsed syntax tree: a bound computed before the parse and a
//! retained-tree estimate read from the tree after it.

use std::fmt::{self, Write as _};

use syn::visit::{self, Visit};

use crate::budget::{SYNTAX_BYTES_PER_BYTE, SYNTAX_BYTES_PER_NODE};

/// The per-node and per-byte terms of a syntax-tree storage bound.
///
/// The bound is `nodes * per_node + text_bytes * per_byte`, where `nodes` is the
/// exact native token-tree node count of the lexed stream and `text_bytes` is
/// the source length. It is computed before the parser runs, so a caller can
/// admit it first. The production terms are measured, not proven; see
/// [`SYNTAX_BYTES_PER_NODE`](crate::SYNTAX_BYTES_PER_NODE).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyntaxBound {
    /// Bytes of peak parse storage per native token-tree node.
    pub per_node: usize,
    /// Bytes of peak parse storage per source byte.
    pub per_byte: usize,
}

impl SyntaxBound {
    /// The measured production terms.
    pub const MEASURED: Self = Self {
        per_node: SYNTAX_BYTES_PER_NODE,
        per_byte: SYNTAX_BYTES_PER_BYTE,
    };

    /// The bound for `nodes` native nodes over `text_bytes` source bytes, or
    /// `None` when it does not fit `usize`.
    #[must_use]
    pub const fn bytes(self, nodes: usize, text_bytes: usize) -> Option<usize> {
        let Some(by_node) = nodes.checked_mul(self.per_node) else {
            return None;
        };
        let Some(by_byte) = text_bytes.checked_mul(self.per_byte) else {
            return None;
        };
        by_node.checked_add(by_byte)
    }
}

/// The node enums the retained-tree estimate counts, each with its size and the
/// fewest native token-tree nodes that one element of the enum takes to parse
/// (a statement or expression takes one, an item or a macro member three, a
/// match arm four).
///
/// One list serves the walk and the structural size check of the bound.
pub const SYNTAX_NODE_SIZES: [(&str, usize, usize); 9] = [
    ("Expr", size_of::<syn::Expr>(), 1),
    ("Stmt", size_of::<syn::Stmt>(), 1),
    ("Item", size_of::<syn::Item>(), 3),
    ("Pat", size_of::<syn::Pat>(), 1),
    ("Type", size_of::<syn::Type>(), 1),
    ("ImplItem", size_of::<syn::ImplItem>(), 3),
    ("TraitItem", size_of::<syn::TraitItem>(), 3),
    ("ForeignItem", size_of::<syn::ForeignItem>(), 3),
    ("Arm", size_of::<syn::Arm>(), 4),
];

const EXPR: usize = 0;
const STMT: usize = 1;
const ITEM: usize = 2;
const PAT: usize = 3;
const TYPE: usize = 4;
const IMPL_ITEM: usize = 5;
const TRAIT_ITEM: usize = 6;
const FOREIGN_ITEM: usize = 7;
const ARM: usize = 8;

/// A retained-tree estimate read from a parsed tree.
///
/// It sums `size_of` of each visited node enum plus the spelling bytes of
/// identifiers and of string, byte-string, integer and float literals. It does
/// not see vector capacity or growth slack, transient parse storage, or the heap
/// behind types outside [`SYNTAX_NODE_SIZES`]; it counts a node held inline in
/// another twice. It is an estimate for detecting a wrong bound, not a heap
/// account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyntaxHeld {
    bytes: usize,
    counts: [usize; 9],
}

impl SyntaxHeld {
    /// Estimated retained bytes.
    #[must_use]
    pub const fn bytes(&self) -> usize {
        self.bytes
    }

    /// The most numerous visited node kind, the larger type on a tie, and its
    /// count. An empty tree names `Item` with zero.
    #[must_use]
    pub fn dominant(&self) -> (&'static str, usize) {
        let mut best = ITEM;
        for index in 0..self.counts.len() {
            let better = (self.counts[index], SYNTAX_NODE_SIZES[index].1)
                > (self.counts[best], SYNTAX_NODE_SIZES[best].1);
            if better {
                best = index;
            }
        }
        (SYNTAX_NODE_SIZES[best].0, self.counts[best])
    }
}

/// Read the retained-tree estimate of `tree`. The walk allocates nothing, apart
/// from one transient copy of each counted literal's native spelling, which
/// `syn` hands out only by value and which dies before the next node.
#[must_use]
pub fn syntax_held(tree: &syn::File) -> SyntaxHeld {
    let mut walk = Walk {
        held: SyntaxHeld {
            bytes: 0,
            counts: [0; 9],
        },
    };
    walk.visit_file(tree);
    walk.held
}

/// Counts the bytes of what is written, retaining none of it.
struct Length(usize);

impl fmt::Write for Length {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0 = self.0.saturating_add(text.len());
        Ok(())
    }
}

struct Walk {
    held: SyntaxHeld,
}

impl Walk {
    fn node(&mut self, kind: usize) {
        self.held.bytes = self.held.bytes.saturating_add(SYNTAX_NODE_SIZES[kind].1);
        self.held.counts[kind] = self.held.counts[kind].saturating_add(1);
    }

    fn spelling(&mut self, bytes: usize) {
        self.held.bytes = self.held.bytes.saturating_add(bytes);
    }

    fn token(&mut self, token: &impl fmt::Display) {
        let mut length = Length(0);
        // Writing to a counting sink cannot fail.
        let _ = write!(length, "{token}");
        self.spelling(length.0);
    }
}

impl<'ast> Visit<'ast> for Walk {
    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        self.node(EXPR);
        visit::visit_expr(self, node);
    }
    fn visit_stmt(&mut self, node: &'ast syn::Stmt) {
        self.node(STMT);
        visit::visit_stmt(self, node);
    }
    fn visit_item(&mut self, node: &'ast syn::Item) {
        self.node(ITEM);
        visit::visit_item(self, node);
    }
    fn visit_pat(&mut self, node: &'ast syn::Pat) {
        self.node(PAT);
        visit::visit_pat(self, node);
    }
    fn visit_type(&mut self, node: &'ast syn::Type) {
        self.node(TYPE);
        visit::visit_type(self, node);
    }
    fn visit_impl_item(&mut self, node: &'ast syn::ImplItem) {
        self.node(IMPL_ITEM);
        visit::visit_impl_item(self, node);
    }
    fn visit_trait_item(&mut self, node: &'ast syn::TraitItem) {
        self.node(TRAIT_ITEM);
        visit::visit_trait_item(self, node);
    }
    fn visit_foreign_item(&mut self, node: &'ast syn::ForeignItem) {
        self.node(FOREIGN_ITEM);
        visit::visit_foreign_item(self, node);
    }
    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        self.node(ARM);
        visit::visit_arm(self, node);
    }
    fn visit_ident(&mut self, node: &'ast proc_macro2::Ident) {
        self.token(node);
    }
    fn visit_lit(&mut self, node: &'ast syn::Lit) {
        match node {
            syn::Lit::Str(literal) => self.token(&literal.token()),
            syn::Lit::ByteStr(literal) => self.token(&literal.token()),
            syn::Lit::Int(literal) => self.token(&literal.token()),
            syn::Lit::Float(literal) => self.token(&literal.token()),
            _ => {}
        }
    }
}
