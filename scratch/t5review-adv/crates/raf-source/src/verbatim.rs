// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The items that rustc parses and syn has no node for, returned as tokens.
//!
//! syn gives `Item::Verbatim` or `ImplItem::Verbatim` for an item it parses
//! but has no node for. Three such shapes are read here, from the tokens syn
//! kept, so each token keeps the span it has in the file:
//!
//! * a `use` whose tree is a braced group with a member that starts with `::`
//!   (`pub use { ::std::fmt::Debug as D, crate::a::f };`), which is read as one
//!   `use` per member, each with the attributes and visibility of the whole;
//! * a function with no body (`#[ref_cast_custom] pub fn f(x: &T) -> &Self;`),
//!   the input of an attribute macro, at item level or in an impl block;
//! * a type with no definition (`#[ty] type bool;`), likewise.
//!
//! Every other verbatim shape is not read, and the caller refuses it.
//!
//! The tokens of one item are parsed again after the parse of its file, which
//! built and dropped the same nodes before it kept the tokens, so what the
//! second parse holds at once stays within what the first held. The facts
//! it gives are charged as those of the plain items they stand for.

use proc_macro2::TokenStream;
use syn::parse::{ParseStream, Parser as _};
use syn::{Attribute, ForeignItemFn, ForeignItemType, Token, UseTree, Visibility, braced, token};

/// A verbatim item of a module that the index reads.
pub(crate) enum Verbatim {
    /// A braced `use` group, each member read as its own declaration.
    UseGroup(UseGroup),
    /// An item with no body.
    Bodiless(Bodiless),
}

/// `#[attrs] vis use { member, ... };`.
pub(crate) struct UseGroup {
    /// The attributes of the whole declaration.
    pub(crate) attrs: Vec<Attribute>,
    /// The visibility of the whole declaration.
    pub(crate) vis: Visibility,
    /// The members, in order.
    pub(crate) members: Vec<UseMember>,
}

/// One member of a `use` group.
pub(crate) struct UseMember {
    /// True when the member starts with `::`.
    pub(crate) leading_colon: bool,
    /// The tree after any leading `::`.
    pub(crate) tree: UseTree,
}

/// An item with no body, of a module or an impl block.
pub(crate) enum Bodiless {
    /// A function with no body.
    Fn(Box<ForeignItemFn>),
    /// A type with no definition.
    Type(Box<ForeignItemType>),
}

/// Read the tokens of a verbatim item of a module, or `None` for a shape that
/// is not read.
pub(crate) fn item(tokens: &TokenStream) -> Option<Verbatim> {
    match split_use_group.parse2(tokens.clone()) {
        Ok(group) => Some(Verbatim::UseGroup(group)),
        Err(_) => member(tokens).map(Verbatim::Bodiless),
    }
}

/// Read the tokens of a verbatim member of an impl block, which holds no
/// `use`, or `None` for a shape that is not read.
pub(crate) fn member(tokens: &TokenStream) -> Option<Bodiless> {
    if let Ok(f) = syn::parse2::<ForeignItemFn>(tokens.clone()) {
        return Some(Bodiless::Fn(Box::new(f)));
    }
    syn::parse2::<ForeignItemType>(tokens.clone())
        .ok()
        .map(|t| Bodiless::Type(Box::new(t)))
}

/// `#[attrs] vis use { member, ... };`, each member an optional leading `::`
/// and a use tree, or a braced group of such members, which binds what its
/// members bind. A group under a path is read by syn as usual, so a leading
/// `::` inside it is not accepted, as rustc does not accept it either.
fn split_use_group(input: ParseStream<'_>) -> syn::Result<UseGroup> {
    let attrs = input.call(Attribute::parse_outer)?;
    let vis: Visibility = input.parse()?;
    input.parse::<Token![use]>()?;
    let content;
    braced!(content in input);
    input.parse::<Token![;]>()?;
    let mut members = Vec::new();
    split_members(&content, &mut members)?;
    Ok(UseGroup {
        attrs,
        vis,
        members,
    })
}

/// The members of the braced group `content`, with those of every bare
/// group in it, in order. The recursion is as deep as the nesting of the
/// braces, which the pre-scan bounds and syn has already parsed.
fn split_members(content: ParseStream<'_>, members: &mut Vec<UseMember>) -> syn::Result<()> {
    while !content.is_empty() {
        if content.peek(token::Brace) {
            let inner;
            braced!(inner in content);
            split_members(&inner, members)?;
        } else {
            let leading_colon = content.parse::<Option<Token![::]>>()?.is_some();
            let tree: UseTree = content.parse()?;
            members.push(UseMember {
                leading_colon,
                tree,
            });
        }
        if content.is_empty() {
            break;
        }
        content.parse::<Token![,]>()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Bodiless, Verbatim, item, member};

    fn tokens(text: &str) -> proc_macro2::TokenStream {
        text.parse().expect("tokens")
    }

    #[test]
    fn a_use_group_splits_into_one_use_per_member() {
        let Some(Verbatim::UseGroup(group)) = item(&tokens(
            "#[cfg(x)] pub use { ::std::fmt::Debug as D, crate::a::{f, g}, };",
        )) else {
            panic!("not a use group");
        };
        assert_eq!(group.members.len(), 2);
        assert!(group.members[0].leading_colon);
        assert!(!group.members[1].leading_colon);
        assert_eq!(group.attrs.len(), 1);
        assert!(matches!(group.vis, syn::Visibility::Public(_)));
    }

    #[test]
    fn a_bare_group_in_a_use_group_contributes_its_members() {
        let Some(Verbatim::UseGroup(group)) =
            item(&tokens("use { a::f, { ::b::c, { ::d::e } }, {} };"))
        else {
            panic!("not a use group");
        };
        let colons: Vec<bool> = group.members.iter().map(|m| m.leading_colon).collect();
        assert_eq!(colons, [false, true, true]);
    }

    #[test]
    fn a_use_group_is_not_read_in_an_impl_block() {
        assert!(member(&tokens("use { ::a::b };")).is_none());
    }

    #[test]
    fn bodiless_functions_and_types_are_read() {
        assert!(matches!(
            item(&tokens("#[m] pub(crate) const fn f(x: &str) -> &Self;")),
            Some(Verbatim::Bodiless(Bodiless::Fn(_)))
        ));
        assert!(matches!(
            member(&tokens("#[ty(cast)] type bool;")),
            Some(Bodiless::Type(_))
        ));
    }

    #[test]
    fn other_shapes_are_not_read() {
        for text in [
            "trait Alias = Send;",
            "use { a::{::b} };",
            "use { ::a::b } extra;",
            "use ::{ ::a::b };",
            "const X: u8;",
            "type T: Bound;",
            "impl Tr for T;",
        ] {
            assert!(item(&tokens(text)).is_none(), "{text}");
        }
    }
}
