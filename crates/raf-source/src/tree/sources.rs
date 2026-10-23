// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Dispatch independently rooted trees by typed source authority.

use super::{Entry, EntryKind, FileId, Opened, Tree};
use crate::{RelPath, SourceAuthorityId, TreeError};
use std::cell::Cell;

/// A first-party tree and borrowed, independently rooted dependency trees.
///
/// This provides physical source separation, not package or revision proof.
/// The caller guarantees each supplied tree's source identity; a locked catalog
/// must validate that correspondence before publishing grounded dependencies.
/// Authority handles are local to this collection. Relative spelling, module
/// traversal and filesystem containment remain within the selected tree.
/// Each supplied backend accepts unqualified paths and returns authority-zero
/// file identities. Already-scoped backend identities are refused, never flattened.
pub struct SourceTrees<'a> {
    first_party: &'a dyn Tree,
    dependencies: &'a [&'a dyn Tree],
}

impl<'a> SourceTrees<'a> {
    /// Borrow source trees without copying or retaining their contents.
    #[must_use]
    pub const fn new(first_party: &'a dyn Tree, dependencies: &'a [&'a dyn Tree]) -> Self {
        Self {
            first_party,
            dependencies,
        }
    }

    /// The load-local authorities, starting with the first-party tree.
    pub fn authorities(&self) -> impl Iterator<Item = SourceAuthorityId> {
        std::iter::once(SourceAuthorityId::FIRST_PARTY)
            .chain((0..self.dependencies.len()).map(|index| SourceAuthorityId(index + 1)))
    }

    /// The root of an available source, or `None` for an unknown authority.
    #[must_use]
    pub fn root(&self, authority: SourceAuthorityId) -> Option<RelPath> {
        self.source(authority)
            .map(|_| RelPath::root().in_source(authority))
    }

    fn source(&self, authority: SourceAuthorityId) -> Option<&dyn Tree> {
        match authority.index() {
            0 => Some(self.first_party),
            index => self.dependencies.get(index - 1).copied(),
        }
    }

    fn selected(&self, path: &RelPath) -> Result<(&dyn Tree, RelPath), TreeError> {
        let tree = self
            .source(path.authority())
            .ok_or_else(|| TreeError::WrongAuthority {
                path: path.to_string(),
                authority: path.authority(),
            })?;
        Ok((tree, path.in_source(SourceAuthorityId::FIRST_PARTY)))
    }
}

impl Tree for SourceTrees<'_> {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        let (tree, local) = self.selected(path)?;
        tree.read(&local, max_bytes)
    }

    fn read_unseen(
        &self,
        path: &RelPath,
        max_bytes: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        let (tree, local) = self.selected(path)?;
        let authority = path.authority();
        let wrong = Cell::new(None);
        let result = tree.read_unseen(&local, max_bytes, &|id| {
            if wrong.get().is_some() {
                return false;
            }
            if id.authority() != SourceAuthorityId::FIRST_PARTY {
                wrong.set(Some(id.authority()));
                return false;
            }
            seen(&id.clone().in_source(authority))
        });
        if let Some(authority) = wrong.get() {
            return Err(TreeError::BackendAuthority {
                path: path.to_string(),
                authority,
            });
        }
        let opened = result?;
        let id = match &opened {
            Opened::Seen(id) | Opened::Read { id, .. } => id,
        };
        if id.authority() != SourceAuthorityId::FIRST_PARTY {
            return Err(TreeError::BackendAuthority {
                path: path.to_string(),
                authority: id.authority(),
            });
        }
        Ok(match opened {
            Opened::Seen(id) => Opened::Seen(id.in_source(authority)),
            Opened::Read { id, text } => Opened::Read {
                id: id.in_source(authority),
                text,
            },
        })
    }

    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        let (tree, local) = self.selected(path)?;
        tree.list(&local)
    }

    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        let (tree, local) = self.selected(path)?;
        tree.entry(&local)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryTree;

    /// A backend exposing explicit identity tags to exercise the mount boundary.
    struct TaggedBackend {
        tree: MemoryTree,
        authority: SourceAuthorityId,
    }

    impl Tree for TaggedBackend {
        fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
            self.tree.read(path, max_bytes)
        }

        fn read_unseen(
            &self,
            path: &RelPath,
            max_bytes: u64,
            seen: &dyn Fn(&FileId) -> bool,
        ) -> Result<Opened, TreeError> {
            Ok(
                match self.tree.read_unseen(path, max_bytes, &|id| {
                    seen(&id.clone().in_source(self.authority))
                })? {
                    Opened::Seen(id) => Opened::Seen(id.in_source(self.authority)),
                    Opened::Read { id, text } => Opened::Read {
                        id: id.in_source(self.authority),
                        text,
                    },
                },
            )
        }

        fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
            self.tree.list(path)
        }
        fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
            self.tree.entry(path)
        }
    }

    #[test]
    fn scoped_backend_ids_are_refused_before_seen_and_preserve_the_first_cause() {
        let first = MemoryTree::default();
        let bad = TaggedBackend {
            tree: MemoryTree::from([("lib.rs", "source")]),
            authority: SourceAuthorityId(7),
        };
        let good = TaggedBackend {
            tree: MemoryTree::from([("lib.rs", "source")]),
            authority: SourceAuthorityId::FIRST_PARTY,
        };
        let dependencies: [&dyn Tree; 2] = [&bad, &good];
        let sources = SourceTrees::new(&first, &dependencies);
        let ids: Vec<_> = sources.authorities().collect();
        let local = RelPath::new("lib.rs").expect("path");
        // Even a later byte-limit failure cannot replace the backend-identity refusal.
        let error = sources
            .read_unseen(&local.in_source(ids[1]), 0, &|_| {
                panic!("an invalid backend ID must not reach the caller")
            })
            .expect_err("already-scoped backend ID");
        assert!(matches!(
            error,
            TreeError::BackendAuthority {
                authority: SourceAuthorityId(7),
                ..
            }
        ));
        assert!(
            matches!(sources.read_unseen(&local.in_source(ids[2]), 0, &|_| true),
            Ok(Opened::Seen(id)) if id.authority() == ids[2])
        );
        assert_eq!(
            sources
                .read(&local.in_source(ids[2]), 6)
                .expect("valid backend"),
            "source"
        );
    }

    #[test]
    fn equal_relative_paths_and_backend_keys_remain_separate_sources() {
        let first = MemoryTree::from([("src/lib.rs", "first")]);
        let second = MemoryTree::from([("src/lib.rs", "second")]);
        let dependencies: [&dyn Tree; 1] = [&second];
        let sources = SourceTrees::new(&first, &dependencies);
        let authorities: Vec<_> = sources.authorities().collect();
        let plain = RelPath::new("src/lib.rs").expect("relative path");
        let other = plain.in_source(authorities[1]);
        assert_ne!(plain, other);
        assert_eq!(plain.as_str(), other.as_str());
        assert_eq!(sources.read(&plain, 5).expect("first"), "first");
        assert_eq!(sources.read(&other, 6).expect("second"), "second");
        assert!(matches!(
            sources.read(&other, 5),
            Err(TreeError::TooLarge { limit: 5, .. })
        ));
        let first_id = sources.identity(&plain).expect("first identity");
        let second_id = sources.identity(&other).expect("second identity");
        assert_ne!(first_id, second_id);
        assert_eq!(second_id.authority(), authorities[1]);
        assert!(
            matches!(sources.read_unseen(&other, 0, &|id| id == &second_id),
            Ok(Opened::Seen(id)) if id == second_id)
        );
        assert!(
            matches!(sources.read_unseen(&other, 6, &|id| id == &first_id),
            Ok(Opened::Read { text, .. }) if text == "second")
        );
        let root = sources.root(authorities[1]).expect("source root");
        assert_eq!(sources.list(&root).expect("directory").len(), 1);
        assert_eq!(sources.entry(&other).expect("entry"), Some(EntryKind::File));
        let unknown = other.in_source(SourceAuthorityId(2));
        assert!(matches!(
            sources.entry(&unknown),
            Err(TreeError::WrongAuthority { .. })
        ));
        assert!(sources.root(unknown.authority()).is_none());
    }

    #[test]
    fn parent_join_and_real_directory_normalization_keep_the_selected_root() {
        let first = MemoryTree::from([("src/lib.rs", "first")]);
        let second = MemoryTree::from([("src/lib.rs", "second"), ("src/sub/item.rs", "item")]);
        let dependencies: [&dyn Tree; 1] = [&second];
        let sources = SourceTrees::new(&first, &dependencies);
        let authority = sources.authorities().nth(1).expect("dependency");
        let path = sources
            .root(authority)
            .expect("root")
            .join("src/sub/../lib.rs")
            .expect("join");
        assert_eq!(path.authority(), authority);
        assert_eq!(path.parent().expect("parent").authority(), authority);
        let tidy = crate::modtree::tidy(&sources, &path).expect("real directory normalization");
        assert_eq!(tidy.as_str(), "src/lib.rs");
        assert_eq!(tidy.authority(), authority);
        assert_eq!(sources.read(&tidy, 6).expect("selected source"), "second");
        let inside = sources
            .root(authority)
            .expect("root")
            .join("src/../src/lib.rs")
            .expect("inside");
        assert_eq!(sources.read(&inside, 6).expect("inside neighbor"), "second");
        assert!(
            sources
                .root(authority)
                .expect("root")
                .join("../src/lib.rs")
                .is_err()
        );
        // No source handle is a textual prefix that `..` could remove.
        assert_eq!(
            path.source_root(),
            sources.root(authority).expect("same root")
        );
    }

    #[test]
    fn standalone_memory_tree_refuses_another_sources_tag_before_seen_callback() {
        let tree = MemoryTree::from([("src/lib.rs", "source")]);
        let local = RelPath::new("src/lib.rs").expect("path");
        let foreign = local.in_source(SourceAuthorityId(1));
        assert!(matches!(
            tree.read(&foreign, 6),
            Err(TreeError::WrongAuthority { .. })
        ));
        assert!(matches!(
            tree.entry(&foreign),
            Err(TreeError::WrongAuthority { .. })
        ));
        assert!(matches!(
            tree.list(&foreign.source_root()),
            Err(TreeError::WrongAuthority { .. })
        ));
        assert!(matches!(
            tree.identity(&foreign),
            Err(TreeError::WrongAuthority { .. })
        ));
        assert!(matches!(
            tree.read_unseen(&foreign, 6, &|_| panic!("must refuse before callback")),
            Err(TreeError::WrongAuthority { .. })
        ));
        assert_eq!(tree.read(&local, 6).expect("ordinary source"), "source");
    }
}
