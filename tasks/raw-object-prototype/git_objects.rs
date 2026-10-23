// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Raw object fixtures, independently checked by the production snapshot reader.

use purrdf_hash::sha1::Sha1;
use raf_source::{GitObject, GitObjectId, GitObjectKind, GitObjects, TreeError};
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct Objects(pub(crate) BTreeMap<GitObjectId, (GitObjectKind, Vec<u8>)>);

impl Objects {
    pub(crate) fn insert(&mut self, kind: GitObjectKind, body: Vec<u8>) -> GitObjectId {
        let name = match kind {
            GitObjectKind::Commit => "commit",
            GitObjectKind::Tree => "tree",
            GitObjectKind::Blob => "blob",
            GitObjectKind::Tag => "tag",
        };
        let mut hash = Sha1::new();
        hash.update(format!("{name} {}\0", body.len()).as_bytes());
        hash.update(&body);
        let id = GitObjectId::from_sha1(hash.finalize());
        self.0.insert(id, (kind, body));
        id
    }
    pub(crate) fn commit(&mut self, tree: GitObjectId) -> GitObjectId {
        self.insert(GitObjectKind::Commit, format!("tree {tree}\nauthor Example <example@example.test> 0 +0000\ncommitter Example <example@example.test> 0 +0000\ngpgsig fixture\n continued\n\nfixture\n").into_bytes())
    }
}

impl GitObjects for Objects {
    fn read_object(&self, id: GitObjectId, max_bytes: u64) -> Result<GitObject<'_>, TreeError> {
        let (kind, body) = self.0.get(&id).ok_or_else(|| TreeError::NotFound {
            path: id.to_string(),
        })?;
        if u64::try_from(body.len()).expect("fixture size") > max_bytes {
            return Err(TreeError::TooLarge {
                path: id.to_string(),
                limit: max_bytes,
            });
        }
        Ok(GitObject { kind: *kind, body })
    }
}

pub(crate) fn entry(mode: &str, name: &str, id: GitObjectId) -> Vec<u8> {
    let mut output = format!("{mode} {name}\0").into_bytes();
    output.extend_from_slice(id.sha1());
    output
}
