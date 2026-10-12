// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A tree that lives in memory, for fixtures that need no disk.

use std::collections::BTreeMap;

use super::{Entry, EntryKind, Tree};
use crate::error::TreeError;
use crate::path::RelPath;

/// Why a set of keys is not a tree.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Fault {
    /// A key is absolute or climbs above the root.
    Outside { key: String },
    /// A key is a file and also a directory holding other keys.
    FileAsDirectory { key: String },
    /// A key names a path that an earlier key already holds.
    Duplicate { key: String },
}

impl Fault {
    fn to_error(&self) -> TreeError {
        match self {
            Self::Outside { key } => TreeError::OutsideRoot { path: key.clone() },
            Self::FileAsDirectory { key } => TreeError::NotRegular { path: key.clone() },
            Self::Duplicate { key } => TreeError::DuplicatePath { path: key.clone() },
        }
    }
}

/// A tree of files held in memory: a map of path to text.
///
/// Directories are inferred from the keys. There are no symlinks and no
/// special files. A key is normalised like a [`RelPath`], and a `..` segment
/// is applied to the keys' own directories.
///
/// [`MemoryTree::try_from_iter`] refuses a bad key with a [`TreeError`]. The
/// infallible `From` and `FromIterator` conversions cannot return one, so a
/// tree built from a bad key reports that same error from every call and
/// never serves a partial tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MemoryTree {
    files: BTreeMap<String, String>,
    fault: Option<Fault>,
}

/// The lexical form of a path: `..` applied against its own segments.
fn collapse(path: &RelPath) -> String {
    let mut kept: Vec<&str> = Vec::new();
    for segment in path.segments() {
        if segment == ".." {
            kept.pop();
        } else {
            kept.push(segment);
        }
    }
    kept.join("/")
}

impl MemoryTree {
    /// Build a tree from `(path, text)` pairs.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::OutsideRoot`] naming a key that is absolute or
    /// climbs above the root, and [`TreeError::NotRegular`] naming a key that
    /// is a file and also holds other keys below it, and
    /// [`TreeError::DuplicatePath`] naming a key whose normalised path an
    /// earlier key already holds.
    pub fn try_from_iter<I, K, V>(entries: I) -> Result<Self, TreeError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Into<String>,
    {
        Self::build(entries).map_err(|fault| fault.to_error())
    }

    fn build<I, K, V>(entries: I) -> Result<Self, Fault>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Into<String>,
    {
        let mut files = BTreeMap::new();
        for (key, text) in entries {
            let key = key.as_ref();
            let path = RelPath::new(key).map_err(|_| Fault::Outside {
                key: key.to_owned(),
            })?;
            if files.insert(collapse(&path), text.into()).is_some() {
                return Err(Fault::Duplicate {
                    key: key.to_owned(),
                });
            }
        }
        for key in files.keys() {
            let mut prefix = String::new();
            for segment in key.split('/') {
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(segment);
                if prefix != *key && files.contains_key(&prefix) {
                    return Err(Fault::FileAsDirectory { key: prefix });
                }
            }
        }
        Ok(Self { files, fault: None })
    }

    fn check(&self) -> Result<(), TreeError> {
        self.fault
            .as_ref()
            .map_or(Ok(()), |fault| Err(fault.to_error()))
    }

    /// True when some key lies below `directory` (the root when empty).
    fn is_directory(&self, directory: &str) -> bool {
        directory.is_empty() || self.keys_below(directory).next().is_some()
    }

    fn keys_below<'a>(&'a self, directory: &'a str) -> impl Iterator<Item = &'a str> {
        self.files.keys().filter_map(move |key| {
            if directory.is_empty() {
                Some(key.as_str())
            } else {
                key.strip_prefix(directory)
                    .and_then(|rest| rest.strip_prefix('/'))
            }
        })
    }
}

impl<K: AsRef<str>, V: Into<String>> FromIterator<(K, V)> for MemoryTree {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(entries: I) -> Self {
        match Self::build(entries) {
            Ok(tree) => tree,
            Err(fault) => Self {
                files: BTreeMap::new(),
                fault: Some(fault),
            },
        }
    }
}

impl<const N: usize> From<[(&str, &str); N]> for MemoryTree {
    fn from(entries: [(&str, &str); N]) -> Self {
        entries.into_iter().collect()
    }
}

impl Tree for MemoryTree {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        self.check()?;
        let key = collapse(path);
        match self.files.get(&key) {
            Some(text) => {
                if u64::try_from(text.len()).map_or(true, |len| len > max_bytes) {
                    Err(TreeError::TooLarge {
                        path: path.to_string(),
                        limit: max_bytes,
                    })
                } else {
                    Ok(text.clone())
                }
            }
            None if self.is_directory(&key) => Err(TreeError::NotRegular {
                path: path.to_string(),
            }),
            None => Err(TreeError::NotFound {
                path: path.to_string(),
            }),
        }
    }

    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        self.check()?;
        let key = collapse(path);
        if self.files.contains_key(&key) {
            return Err(TreeError::NotRegular {
                path: path.to_string(),
            });
        }
        if !self.is_directory(&key) {
            return Err(TreeError::NotFound {
                path: path.to_string(),
            });
        }
        let mut found: BTreeMap<&str, EntryKind> = BTreeMap::new();
        for rest in self.keys_below(&key) {
            match rest.split_once('/') {
                Some((name, _)) => found.insert(name, EntryKind::Dir),
                None => found.insert(rest, EntryKind::File),
            };
        }
        Ok(found
            .into_iter()
            .map(|(name, kind)| Entry {
                name: name.to_owned(),
                kind,
            })
            .collect())
    }

    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        self.check()?;
        let key = collapse(path);
        if self.files.contains_key(&key) {
            Ok(Some(EntryKind::File))
        } else if self.is_directory(&key) {
            Ok(Some(EntryKind::Dir))
        } else {
            Ok(None)
        }
    }
}
