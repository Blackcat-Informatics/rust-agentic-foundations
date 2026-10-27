// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual objects in test-owned repositories, without source-authority flags.

use crate::support::Scratch;
use purrdf_hash::hex;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Copy)]
pub(crate) struct GitObjectId(pub(crate) [u8; 20]);
impl GitObjectId {
    pub(crate) fn sha1(&self) -> &[u8; 20] {
        &self.0
    }
}
impl fmt::Display for GitObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        hex::Lower(&self.0).fmt(f)
    }
}
#[derive(Clone, Copy)]
pub(crate) enum GitObjectKind {
    Blob,
    Tree,
    Commit,
}
pub(crate) struct Objects(Scratch);
impl Default for Objects {
    fn default() -> Self {
        let scratch = Scratch::new();
        let status = command()
            .args(["init", "--quiet"])
            .arg(scratch.base())
            .status()
            .expect("required Git");
        assert!(status.success(), "initialize owned fixture");
        Self(scratch)
    }
}
impl Objects {
    pub(crate) fn command(&self) -> Command {
        let mut command = command();
        command.arg("-C").arg(self.path());
        command
    }
    pub(crate) fn path(&self) -> &Path {
        self.0.base()
    }
    pub(crate) fn object_path(&self, id: GitObjectId) -> PathBuf {
        let id = id.to_string();
        self.path()
            .join(".git/objects")
            .join(&id[..2])
            .join(&id[2..])
    }
    pub(crate) fn insert(&mut self, kind: GitObjectKind, body: Vec<u8>) -> GitObjectId {
        let kind = match kind {
            GitObjectKind::Blob => "blob",
            GitObjectKind::Tree => "tree",
            GitObjectKind::Commit => "commit",
        };
        let mut child = self
            .command()
            .args(["hash-object", "--literally", "-w", "-t", kind, "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("required Git");
        std::io::copy(
            &mut std::io::Cursor::new(body),
            &mut child.stdin.take().expect("stdin"),
        )
        .expect("write owned fixture");
        let output = child.wait_with_output().expect("owned Git");
        assert!(output.status.success(), "fixture object write failed");
        let text = std::str::from_utf8(&output.stdout).expect("hex").trim();
        let mut id = [0; 20];
        hex::decode_to_slice(text, &mut id).expect("full ID");
        let id = GitObjectId(id);
        assert!(self.object_path(id).is_file(), "actual stored object");
        id
    }
    pub(crate) fn commit(&mut self, tree: GitObjectId) -> GitObjectId {
        self.insert(GitObjectKind::Commit,format!("tree {tree}\nauthor Example <example@example.test> 0 +0000\ncommitter Example <example@example.test> 0 +0000\n\nfixture\n").into_bytes())
    }
}

fn command() -> Command {
    let mut command = Command::new("git");
    for name in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_INDEX_FILE",
    ] {
        command.env_remove(name);
    }
    command
}
pub(crate) fn entry(mode: &str, name: &str, id: GitObjectId) -> Vec<u8> {
    let mut output = format!("{mode} {name}\0").into_bytes();
    output.extend_from_slice(id.sha1());
    output
}
