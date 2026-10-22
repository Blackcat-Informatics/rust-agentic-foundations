// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The frontmatter value model.
//!
//! The reader and the public constructors both produce these types, and the
//! constructors refuse every value the reader would not read back. Fields are
//! private, so a value that exists can always be written and read again.
//!
//! Equality ignores line numbers, and it compares the entries of a mapping as
//! a map, so key order does not matter. Everything else a value carries takes
//! part: the quoting style of a scalar, the form of a list, the order of list
//! items and the comments attached to an entry, because the writer reproduces
//! all of them.

use std::collections::HashMap;

use crate::error::ConstructError;
use crate::frontmatter::scalar::{choose_style, plain_ok};
use crate::limits::MAX_YAML_DEPTH;

/// How a scalar was written, and how the writer writes it back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarStyle {
    /// Unquoted text.
    Plain,
    /// Text in `'single quotes'`, where `''` stands for one quote.
    SingleQuoted,
    /// Text in `"double quotes"`, with the shared escape set.
    DoubleQuoted,
}

/// A scalar. Numbers, booleans and null are kept as the text that was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scalar {
    pub(crate) text: String,
    pub(crate) style: ScalarStyle,
}

impl Scalar {
    /// A scalar for `text`, in the style the writer needs for it.
    ///
    /// The text is unquoted when [`plain_ok`] holds and a general YAML reader
    /// would read it as a string, double-quoted when it holds control
    /// characters (each of which needs an escape), and single-quoted
    /// otherwise.
    ///
    /// # Errors
    ///
    /// [`ConstructError::Unrepresentable`] when the text holds a control
    /// character with no escape.
    pub fn new(text: impl Into<String>) -> Result<Self, ConstructError> {
        let text = text.into();
        match choose_style(&text) {
            Ok(style) => Ok(Self { text, style }),
            Err(character) => Err(ConstructError::Unrepresentable { character }),
        }
    }

    /// The decoded text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The style the scalar is written in.
    #[must_use]
    pub fn style(&self) -> ScalarStyle {
        self.style
    }
}

/// How a list was written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListForm {
    /// `[a, b]` on the key's line.
    Flow,
    /// `- a` lines under the key.
    Block,
}

/// Comments attached to a line of the frontmatter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Comments {
    pub(crate) leading: Vec<String>,
    pub(crate) trailing: Option<String>,
}

impl Comments {
    /// The lines (full-line comments and blank lines, verbatim and without a
    /// line ending) that precede the line, in order.
    #[must_use]
    pub fn leading(&self) -> &[String] {
        &self.leading
    }

    /// The text that follows the value on the same line, verbatim, starting at
    /// the whitespace before `#`.
    #[must_use]
    pub fn trailing(&self) -> Option<&str> {
        self.trailing.as_deref()
    }
}

/// A value: a scalar, a list or a mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A scalar.
    Scalar(Scalar),
    /// A flow or block list.
    List(List),
    /// A block mapping.
    Map(Map),
}

impl Value {
    /// The scalar, when this value is one.
    #[must_use]
    pub fn as_scalar(&self) -> Option<&Scalar> {
        match self {
            Self::Scalar(scalar) => Some(scalar),
            Self::List(_) | Self::Map(_) => None,
        }
    }

    /// The list, when this value is one.
    #[must_use]
    pub fn as_list(&self) -> Option<&List> {
        match self {
            Self::List(list) => Some(list),
            Self::Scalar(_) | Self::Map(_) => None,
        }
    }

    /// The mapping, when this value is one.
    #[must_use]
    pub fn as_map(&self) -> Option<&Map> {
        match self {
            Self::Map(map) => Some(map),
            Self::Scalar(_) | Self::List(_) => None,
        }
    }

    /// The number of collection levels this value adds: 0 for a scalar.
    pub(crate) fn depth(&self) -> usize {
        match self {
            Self::Scalar(_) => 0,
            Self::List(list) => list.depth,
            Self::Map(map) => map.depth,
        }
    }
}

/// A list of scalars, or of mappings written as `- key: value` lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List {
    pub(crate) form: ListForm,
    pub(crate) items: Vec<ListItem>,
    pub(crate) depth: usize,
}

impl List {
    /// A flow list `[a, b]` of scalars; it may be empty.
    ///
    /// # Errors
    ///
    /// [`ConstructError::EmptyListItem`] for an empty unquoted scalar and
    /// [`ConstructError::NotFlowSafe`] for an unquoted scalar that [`plain_ok`]
    /// refuses.
    pub fn flow(scalars: Vec<Scalar>) -> Result<Self, ConstructError> {
        let mut items = Vec::with_capacity(scalars.len());
        for scalar in scalars {
            if scalar.style == ScalarStyle::Plain {
                if scalar.text.is_empty() {
                    return Err(ConstructError::EmptyListItem);
                }
                if !plain_ok(&scalar.text) {
                    return Err(ConstructError::NotFlowSafe { text: scalar.text });
                }
            }
            items.push(ListItem {
                value: ItemValue::Scalar(scalar),
                line: 0,
                comments: Comments::default(),
            });
        }
        Ok(Self {
            form: ListForm::Flow,
            items,
            depth: 1,
        })
    }

    /// A block list of `- ` items: all scalars, or all mappings.
    ///
    /// # Errors
    ///
    /// [`ConstructError::EmptyBlockList`] for no items,
    /// [`ConstructError::MixedList`] for scalars beside mappings and
    /// [`ConstructError::DepthExceeded`] past the nesting bound.
    pub fn block(items: Vec<ListItem>) -> Result<Self, ConstructError> {
        if items.is_empty() {
            return Err(ConstructError::EmptyBlockList);
        }
        let maps = items
            .iter()
            .filter(|item| matches!(item.value, ItemValue::Map(_)))
            .count();
        if maps != 0 && maps != items.len() {
            return Err(ConstructError::MixedList);
        }
        let depth = 1 + items
            .iter()
            .map(|item| item.value.depth())
            .fold(0, usize::max);
        if depth > MAX_YAML_DEPTH {
            return Err(ConstructError::DepthExceeded {
                max: MAX_YAML_DEPTH,
            });
        }
        Ok(Self {
            form: ListForm::Block,
            items,
            depth,
        })
    }

    /// How the list was written.
    #[must_use]
    pub fn form(&self) -> ListForm {
        self.form
    }

    /// The items, in author order.
    #[must_use]
    pub fn items(&self) -> &[ListItem] {
        &self.items
    }
}

/// The content of a list item: a scalar, or a mapping written as
/// `- key: value` lines. A list never holds a list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemValue {
    /// A scalar.
    Scalar(Scalar),
    /// A block mapping.
    Map(Map),
}

impl ItemValue {
    /// The scalar, when this item is one.
    #[must_use]
    pub fn as_scalar(&self) -> Option<&Scalar> {
        match self {
            Self::Scalar(scalar) => Some(scalar),
            Self::Map(_) => None,
        }
    }

    /// The mapping, when this item is one.
    #[must_use]
    pub fn as_map(&self) -> Option<&Map> {
        match self {
            Self::Map(map) => Some(map),
            Self::Scalar(_) => None,
        }
    }

    /// The number of collection levels this content adds: 0 for a scalar.
    pub(crate) fn depth(&self) -> usize {
        match self {
            Self::Scalar(_) => 0,
            Self::Map(map) => map.depth,
        }
    }
}

/// One list item.
#[derive(Debug, Clone)]
pub struct ListItem {
    pub(crate) value: ItemValue,
    pub(crate) line: usize,
    pub(crate) comments: Comments,
}

impl ListItem {
    /// An item that holds a scalar.
    ///
    /// # Errors
    ///
    /// [`ConstructError::EmptyListItem`] for an empty unquoted scalar.
    pub fn scalar(scalar: Scalar) -> Result<Self, ConstructError> {
        if scalar.style == ScalarStyle::Plain && scalar.text.is_empty() {
            return Err(ConstructError::EmptyListItem);
        }
        Ok(Self {
            value: ItemValue::Scalar(scalar),
            line: 0,
            comments: Comments::default(),
        })
    }

    /// An item that holds a mapping, written as `- key: value` lines.
    ///
    /// Comments before the first entry move to the item, because the first
    /// entry shares its line with the `- `.
    #[must_use]
    pub fn map(mut map: Map) -> Self {
        let leading = match map.entries.first_mut() {
            Some(first) => std::mem::take(&mut first.comments.leading),
            None => Vec::new(),
        };
        Self {
            value: ItemValue::Map(map),
            line: 0,
            comments: Comments {
                leading,
                trailing: None,
            },
        }
    }

    /// The item: a scalar, or a mapping for `- key: value` items.
    #[must_use]
    pub fn value(&self) -> &ItemValue {
        &self.value
    }

    /// The 1-based line of the item in the text it was read from, and 0 for an
    /// item built from parts (the line of the key for a flow list item).
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }

    /// Comments before the item and after a scalar item's value.
    #[must_use]
    pub fn comments(&self) -> &Comments {
        &self.comments
    }
}

impl PartialEq for ListItem {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.comments == other.comments
    }
}

impl Eq for ListItem {}

/// A block mapping with at least one entry and unique keys.
#[derive(Debug, Clone)]
pub struct Map {
    pub(crate) entries: Vec<Entry>,
    pub(crate) depth: usize,
}

impl Map {
    /// A mapping of `entries`, in the order given.
    ///
    /// # Errors
    ///
    /// [`ConstructError::EmptyMap`] for no entries,
    /// [`ConstructError::DuplicateKey`] for a repeated key and
    /// [`ConstructError::DepthExceeded`] past the nesting bound.
    pub fn new(entries: Vec<Entry>) -> Result<Self, ConstructError> {
        if entries.is_empty() {
            return Err(ConstructError::EmptyMap);
        }
        let mut seen = std::collections::HashSet::new();
        for entry in &entries {
            if !seen.insert(entry.key.as_str()) {
                return Err(ConstructError::DuplicateKey {
                    key: entry.key.clone(),
                });
            }
        }
        let depth = 1 + entries
            .iter()
            .map(|entry| entry.value.depth())
            .fold(0, usize::max);
        if depth > MAX_YAML_DEPTH {
            return Err(ConstructError::DepthExceeded {
                max: MAX_YAML_DEPTH,
            });
        }
        Ok(Self { entries, depth })
    }

    /// The entry with this key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.key == key)
    }

    /// The entries, in author order.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
}

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        entries_equal(&self.entries, &other.entries)
    }
}

impl Eq for Map {}

/// One `key: value` entry.
#[derive(Debug, Clone)]
pub struct Entry {
    pub(crate) key: String,
    pub(crate) value: Value,
    pub(crate) line: usize,
    pub(crate) comments: Comments,
}

/// Whether `key` is a non-empty run of `A-Z a-z 0-9 _ -`.
pub(crate) fn is_key(key: &str) -> bool {
    !key.is_empty() && key.chars().all(is_key_char)
}

/// Whether `c` may appear in a key.
pub(crate) fn is_key_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

impl Entry {
    /// An entry with no comments.
    ///
    /// # Errors
    ///
    /// [`ConstructError::InvalidKey`] when the key is empty or holds
    /// characters outside `A-Z a-z 0-9 _ -`.
    pub fn new(key: impl Into<String>, value: Value) -> Result<Self, ConstructError> {
        let key = key.into();
        if !is_key(&key) {
            return Err(ConstructError::InvalidKey { key });
        }
        Ok(Self {
            key,
            value,
            line: 0,
            comments: Comments::default(),
        })
    }

    /// The key.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The value.
    #[must_use]
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// The 1-based line of the key in the text the entry was read from, and 0
    /// for an entry built from parts.
    #[must_use]
    pub fn line(&self) -> usize {
        self.line
    }

    /// Comments before the key and after its value on the key's line.
    #[must_use]
    pub fn comments(&self) -> &Comments {
        &self.comments
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value && self.comments == other.comments
    }
}

impl Eq for Entry {}

/// Compare two entry sequences as maps: keys are unique, so equal length and
/// every entry of one found equal in the other is equality.
fn entries_equal(left: &[Entry], right: &[Entry]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let index: HashMap<&str, &Entry> = right
        .iter()
        .map(|entry| (entry.key.as_str(), entry))
        .collect();
    left.iter().all(|entry| {
        index
            .get(entry.key.as_str())
            .is_some_and(|other| *other == entry)
    })
}

/// The frontmatter: the top-level mapping and the comments after its last
/// entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontmatter {
    pub(crate) map: Map,
    pub(crate) trailing_comments: Vec<String>,
}

impl Frontmatter {
    /// A frontmatter for `map`, with no comments after the last entry.
    #[must_use]
    pub fn new(map: Map) -> Self {
        Self {
            map,
            trailing_comments: Vec::new(),
        }
    }

    /// The top-level mapping.
    #[must_use]
    pub fn map(&self) -> &Map {
        &self.map
    }

    /// The comment and blank lines after the last entry, verbatim and without
    /// a line ending. The writer keeps them last.
    #[must_use]
    pub fn trailing_comments(&self) -> &[String] {
        &self.trailing_comments
    }
}
