// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The items of the symbol index.

use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use crate::facts::{FieldShape, SpanFact, TestRole, Vis};
use crate::modtree::TargetId;
use crate::path::RelPath;

/// Text that the records of one load share: an item path, a name, a module
/// path or a written type is stored once and every record that holds it
/// points at that one copy.
///
/// It reads as a `str` and compares with `str` and `String`.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SharedStr(Arc<str>);

impl SharedStr {
    /// The text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for SharedStr {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for SharedStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for SharedStr {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SharedStr {
    fn from(text: &str) -> Self {
        Self(Arc::from(text))
    }
}

impl From<String> for SharedStr {
    fn from(text: String) -> Self {
        Self(Arc::from(text))
    }
}

impl fmt::Display for SharedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for SharedStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&*self.0, f)
    }
}

impl PartialEq<str> for SharedStr {
    fn eq(&self, other: &str) -> bool {
        *self.0 == *other
    }
}

impl PartialEq<&str> for SharedStr {
    fn eq(&self, other: &&str) -> bool {
        *self.0 == **other
    }
}

impl PartialEq<String> for SharedStr {
    fn eq(&self, other: &String) -> bool {
        *self.0 == **other
    }
}

impl PartialEq<SharedStr> for str {
    fn eq(&self, other: &SharedStr) -> bool {
        *self == *other.0
    }
}

impl PartialEq<SharedStr> for &str {
    fn eq(&self, other: &SharedStr) -> bool {
        **self == *other.0
    }
}

impl PartialEq<SharedStr> for String {
    fn eq(&self, other: &SharedStr) -> bool {
        **self == *other.0
    }
}

/// A name space: the three in which a Rust path can name something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Namespace {
    /// Types, traits, modules, aliases and `extern crate` items.
    Type,
    /// Functions, constants, statics, methods and the constructors of unit
    /// and tuple structs.
    Value,
    /// Macros.
    Macro,
}

impl Namespace {
    /// Every name space, in the order `resolve` tries them.
    pub const ALL: [Namespace; 3] = [Namespace::Type, Namespace::Value, Namespace::Macro];
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Type => "type",
            Self::Value => "value",
            Self::Macro => "macro",
        })
    }
}

/// What an indexed item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SymbolKind {
    /// A function.
    Fn,
    /// A struct.
    Struct,
    /// An enum.
    Enum,
    /// A union.
    Union,
    /// A trait.
    Trait,
    /// A type alias.
    TypeAlias,
    /// A constant.
    Const,
    /// A static.
    Static,
    /// A module, inline or in its own file.
    Mod,
    /// A `macro_rules!` definition. A `#[macro_export]` macro is indexed at
    /// the crate root.
    MacroRules,
    /// An `extern crate` item.
    ExternCrate,
    /// A function declared in an `extern` block.
    ForeignFn,
    /// A static declared in an `extern` block.
    ForeignStatic,
    /// A type declared in an `extern` block.
    ForeignType,
    /// A method or associated function of an inherent `impl` block.
    InherentMethod,
    /// A method or associated function of a trait `impl` block.
    TraitImplMethod,
    /// A trait method that has a default body.
    TraitDefaultMethod,
    /// A trait method that has no body.
    TraitRequiredMethod,
    /// An associated constant of a trait or an impl block.
    AssocConst,
    /// An associated type of a trait or an impl block.
    AssocType,
}

/// The identity of an item within one loaded workspace: a dense number that
/// is stable for the life of the index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId(pub(crate) usize);

impl ItemId {
    /// The number of the item.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// How the trait of an impl that is not on a local type is named in a path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum TraitKey {
    /// The one absolute path of the trait: its definition for a local trait,
    /// otherwise the path in the dependency, with `core` and `alloc` written as
    /// `std`.
    Canonical(String),
    /// The path of the trait cannot be told: it is brought in by globs of
    /// dependencies that cannot be looked into, and more than one of them could
    /// provide the name (or the name is also in the standard prelude). The
    /// members are recorded under the written trait path, and every absolute
    /// path the trait could have is listed.
    Unresolved {
        /// The trait path as written, without spaces.
        written: String,
        /// The absolute paths the trait could have, sorted.
        candidates: Vec<String>,
    },
}

/// How the index placed the members of an impl block, by what its self type
/// names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SelfType {
    /// The self type names a struct, enum or union of the workspace, through
    /// any type aliases: the members are recorded under that type.
    Local,
    /// The impl is of a trait and its self type is any other type: the
    /// members are recorded under the key of the trait.
    Trait,
    /// The impl is inherent and its self type names no struct, enum or union
    /// the index can see: a type that a macro generates, a trait object, a
    /// type that does not exist. The members are recorded under the self
    /// path written in the impl's module (see [`UnresolvedImpl::path`]), and
    /// a path that goes through it resolves to
    /// [`ResolveError::UnresolvedSelf`](crate::ResolveError::UnresolvedSelf).
    Unresolved,
}

/// An inherent impl block whose self type names no struct, enum or union the
/// index can see, once for each module path its file is reached under.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnresolvedImpl {
    /// The self path written in the impl's module, which its members are
    /// recorded under: the module path followed by the path the self type is
    /// written with (`crate`, `self` and `super` taken from the module; a path
    /// that starts with `::` from its first segment, alone), or, for a self
    /// type that has no path, by the written self type without spaces.
    pub path: SharedStr,
    /// The self type as written, without spaces.
    pub self_type: SharedStr,
    /// The file that holds the impl.
    pub file: RelPath,
    /// The 1-based line of the `impl` keyword.
    pub line: usize,
    /// The whole impl block.
    pub span: SpanFact,
    /// The target whose module tree holds the impl.
    pub target: TargetId,
    /// The test role of the impl, counting the module tree's test flag.
    pub test: TestRole,
    /// The members of the impl block, in source order.
    pub members: Vec<ItemId>,
}

/// One named item of the workspace.
///
/// An item is identified in its file by the byte offset where its span starts.
/// A file that several module trees reach has one `Item` for each path it is
/// reached under; the items share their file and offset and differ in `path`
/// and `target`.
///
/// A record is at most 256 bytes: its text is shared ([`SharedStr`] and
/// [`RelPath`]), and the trait key of an impl member is shared by the members
/// of its impl block.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Item {
    /// The identity of the item.
    pub id: ItemId,
    /// The full path: the module path and the name, with a method under its
    /// impl type, `<trait>::<method>` for a trait impl on a type that is not
    /// a local struct, enum or union, or `<self path>::<method>` for an
    /// inherent impl whose self type the index cannot see (see
    /// [`SelfType::Unresolved`]).
    pub path: SharedStr,
    /// The name without a raw-identifier prefix.
    pub name: SharedStr,
    /// What the item is.
    pub kind: SymbolKind,
    /// The main name space of the item. Unit and tuple structs are also in
    /// the value name space; see [`Item::in_namespace`].
    pub namespace: Namespace,
    /// The file that defines the item.
    pub file: RelPath,
    /// The 1-based line of the name.
    pub name_line: usize,
    /// The whole item, attributes included.
    pub span: SpanFact,
    /// The braced body, or the function block, if there is one.
    pub body: Option<SpanFact>,
    /// The declared visibility.
    pub visibility: Vis,
    /// True when the item has non-blank documentation: an unconditional
    /// `#[doc]` attribute or doc comment that holds text, or
    /// `#[doc = include_str!(..)]`, where `#[doc(hidden)]` alone does not
    /// count; for a module that has its own file, the file's inner
    /// documentation counts.
    pub documented: bool,
    /// True when the item has `#[doc(hidden)]`.
    pub doc_hidden: bool,
    /// The test role of the item, including the role the module tree gives it:
    /// an item in a file reached only through test code is `CfgTest`.
    pub test: TestRole,
    /// The target whose module tree holds the item.
    pub target: TargetId,
    /// For a member of an impl block, the self type as written, without
    /// spaces.
    pub self_type: Option<SharedStr>,
    /// For a trait impl method, the trait path as written.
    pub trait_path: Option<SharedStr>,
    /// For a trait impl method on a type that is not local, how the trait is
    /// keyed; `None` for the members of impls on local types, which are
    /// recorded under the type.
    pub trait_key: Option<Arc<TraitKey>>,
    /// For a member of an impl block, how its self type placed it; `None` for
    /// every other item.
    pub self_key: Option<SelfType>,
    /// For a struct or union, how its fields are declared.
    pub shape: Option<FieldShape>,
    /// True for a `macro_rules!` macro with an unconditional
    /// `#[macro_export]`, which is indexed at the crate root and which other
    /// crates can name there; false for every other item.
    pub macro_export: bool,
}

impl Item {
    /// True when the item defines a name in `namespace`.
    #[must_use]
    pub fn in_namespace(&self, namespace: Namespace) -> bool {
        self.namespace == namespace
            || (namespace == Namespace::Value
                && self.kind == SymbolKind::Struct
                && matches!(self.shape, Some(FieldShape::Unit | FieldShape::Tuple)))
    }

    /// The identity of the source item: its file and the byte offset where its
    /// span starts.
    #[must_use]
    pub fn source_key(&self) -> (&RelPath, usize) {
        (&self.file, self.span.start_byte)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_item_record_is_at_most_two_hundred_fifty_six_bytes() {
        assert!(
            size_of::<Item>() <= 256,
            "an item record is {} bytes",
            size_of::<Item>()
        );
    }

    #[test]
    fn shared_text_reads_and_compares_as_a_str() {
        let text = SharedStr::from("demo::go");
        assert_eq!(text, "demo::go");
        assert_eq!(text, *"demo::go");
        assert_eq!(text, "demo::go".to_owned());
        assert_eq!("demo::go", text);
        assert_eq!(text.as_str(), "demo::go");
        assert_eq!(format!("{text} {text:?}"), "demo::go \"demo::go\"");
        assert!(text.starts_with("demo"));
    }
}
