// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared fixtures: a repository written to a scratch directory, the runner
//! of the built binary in both shapes of its argument vector, a snapshot of
//! every byte below a directory, and the documents the cases are made of.

#![allow(
    dead_code,
    reason = "each test binary uses a part of the shared support"
)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use raf_workplan::{Gate, SyncOptions, Workplan};
use serde_json::Value;

#[cfg(unix)]
pub(crate) mod stubs;

/// The built binary.
pub(crate) const BINARY: &str = env!("CARGO_BIN_EXE_cargo-workplan");

/// The copyright text of an index the fixtures create.
pub(crate) const COPYRIGHT: &str = "2026 Example Org <dev@example.org>";
/// The licence identifier of an index the fixtures create.
pub(crate) const LICENCE: &str = "CC-BY-4.0";

/// A unique directory under the system temporary directory, removed on drop.
pub(crate) struct Scratch(PathBuf);

impl Scratch {
    pub(crate) fn new(tag: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "cargo-workplan-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("cannot create {}: {e}", path.display()));
        // A Unix temporary directory may be reached through a link (macOS), and
        // the root `cargo metadata` names is the canonical one. On Windows the
        // canonical form is a verbatim path that not every program takes.
        #[cfg(unix)]
        let path = fs::canonicalize(&path).expect("a canonical scratch directory");
        Self(path)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // The directory is under the temporary directory and only litter.
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A repository on a disk: a Cargo workspace of one package, `demo`, and the
/// documents written into it.
pub(crate) struct Repo {
    scratch: Scratch,
}

impl Repo {
    /// A workspace with the package `demo` and no document.
    pub(crate) fn bare(tag: &str) -> Self {
        let repo = Self {
            scratch: Scratch::new(tag),
        };
        repo.put(
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
        );
        repo.put(
            "Cargo.lock",
            "version = 4\n\n[[package]]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        );
        repo.put(
            "src/lib.rs",
            "/// A documented function.\npub fn real() {}\n",
        );
        repo.put("tests/it.rs", "#[test]\nfn case() {}\n");
        repo
    }

    /// The roadmap repository, with its index created by the library.
    pub(crate) fn synced(tag: &str, missing_binding: bool) -> Self {
        let repo = Self::roadmap(tag, missing_binding);
        repo.sync();
        repo
    }

    /// One capability in each state and one open change, no index.
    ///
    /// * `alpha` is delivered, bound to code and a test.
    /// * `beta` is implementing; its code list holds `demo::real` and, when
    ///   `missing_binding`, `demo::missing`, which does not resolve; its test
    ///   and example lists are empty.
    /// * `gamma` is designed and `delta` is considered, with a PROPOSED block
    ///   of the change `grow` in its Contract.
    /// * `grow` lists `beta`, `gamma` and `delta`.
    pub(crate) fn roadmap(tag: &str, missing_binding: bool) -> Self {
        let repo = Self::bare(tag);
        let code = if missing_binding {
            "[demo::real, demo::missing]"
        } else {
            "[demo::real]"
        };
        repo.put(
            "docs/capabilities/alpha.md",
            &capability(
                "alpha",
                "delivered",
                ["[demo::real]", "[demo::test_crate::it::case]", "[]"],
                "[]",
                &[],
            ),
        );
        repo.put(
            "docs/capabilities/beta.md",
            &capability(
                "beta",
                "implementing",
                [code, "[]", "[]"],
                "[cap:alpha]",
                &[],
            ),
        );
        repo.put(
            "docs/capabilities/gamma.md",
            &capability("gamma", "designed", ["[]", "[]", "[]"], "[cap:alpha]", &[]),
        );
        repo.put(
            "docs/capabilities/delta.md",
            &capability(
                "delta",
                "considered",
                ["[]", "[]", "[]"],
                "[]",
                &[(
                    "Contract",
                    "\n> [!PROPOSED change:grow]\n> Delta reads streams as they arrive.\n>\n> It never buffers a whole document.\n",
                )],
            ),
        );
        repo.put(
            "work/changes/grow.md",
            &change(
                "grow",
                "[cap:beta, cap:gamma, cap:delta]",
                &[
                    "cap:beta#contract",
                    "cap:gamma#contract",
                    "cap:delta#contract",
                ],
            ),
        );
        repo
    }

    pub(crate) fn root(&self) -> &Path {
        self.scratch.path()
    }

    /// Write `text` at `path`, creating the directories.
    pub(crate) fn put(&self, path: &str, text: &str) {
        let target = self.root().join(path);
        fs::create_dir_all(target.parent().expect("a parent"))
            .unwrap_or_else(|e| panic!("cannot create the directory of {path}: {e}"));
        fs::write(&target, text).unwrap_or_else(|e| panic!("cannot write {path}: {e}"));
    }

    /// The text of the file `path`.
    pub(crate) fn read(&self, path: &str) -> String {
        fs::read_to_string(self.root().join(path))
            .unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
    }

    /// Replace the first `from` in the file `path` with `to`.
    pub(crate) fn edit(&self, path: &str, from: &str, to: &str) {
        let text = self.read(path);
        assert!(text.contains(from), "{path} has no `{from}`");
        self.put(path, &text.replacen(from, to, 1));
    }

    /// Sync the generated blocks with the library, creating the index.
    pub(crate) fn sync(&self) {
        let opened = Gate::workspace(self.root()).open().expect("a repository");
        let plan = opened.workplan();
        let report = plan
            .sync(&SyncOptions::new(COPYRIGHT, LICENCE))
            .expect("a sync");
        Workplan::write(&report, self.root()).expect("the sync is written");
    }

    /// Every file below the root.
    pub(crate) fn snapshot(&self) -> BTreeMap<String, Vec<u8>> {
        snapshot(self.root())
    }

    /// The findings of the library over the repository, as the binary prints
    /// them in text.
    pub(crate) fn gate_lines(&self) -> Vec<String> {
        Gate::workspace(self.root())
            .check()
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    /// A command that runs the binary in the root.
    pub(crate) fn run(&self, args: &[&str]) -> Run {
        run_in(self.root(), Shape::Direct, args)
    }

    /// Run the binary in the root in both shapes and require equal results.
    pub(crate) fn both(&self, args: &[&str]) -> Run {
        run_both(self.root(), args)
    }

    /// Run a command that writes in both shapes, each from the same starting
    /// tree, and require the same result and the same tree afterwards. The
    /// tree is left as the command made it.
    pub(crate) fn both_writing(&self, args: &[&str]) -> Run {
        let before = self.snapshot();
        let direct = run_in(self.root(), Shape::Direct, args);
        let written = self.snapshot();
        self.restore(&before);
        let cargo = run_in(self.root(), Shape::Cargo, args);
        assert_eq!(direct, cargo, "the two shapes of {args:?} differ");
        assert_eq!(
            self.snapshot(),
            written,
            "the two shapes of {args:?} wrote differently"
        );
        direct
    }

    /// Put the tree back as a snapshot recorded it.
    pub(crate) fn restore(&self, snapshot: &BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(self.root()).expect("list the root") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                fs::remove_dir_all(&path).expect("remove a directory");
            } else {
                fs::remove_file(&path).expect("remove a file");
            }
        }
        for (path, bytes) in snapshot {
            if let Some(dir) = path.strip_suffix('/') {
                fs::create_dir_all(self.root().join(dir)).expect("a directory");
            } else {
                self.put_bytes(path, bytes);
            }
        }
    }

    fn put_bytes(&self, path: &str, bytes: &[u8]) {
        let target = self.root().join(path);
        fs::create_dir_all(target.parent().expect("a parent")).expect("a directory");
        fs::write(target, bytes).expect("a file");
    }

    /// `--root <root>` followed by `args`, in both shapes.
    pub(crate) fn rooted(&self, args: &[&str]) -> Run {
        let root = self.root().to_string_lossy().into_owned();
        let mut all = vec!["--root", root.as_str()];
        all.extend_from_slice(args);
        run_both(&elsewhere(), &all)
    }
}

/// A directory that is not in any Cargo workspace and is not the cargo
/// directory of the test: where a rooted run starts.
pub(crate) fn elsewhere() -> PathBuf {
    std::env::temp_dir()
}

/// A shape of the argument vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shape {
    /// `cargo-workplan <command>`.
    Direct,
    /// `cargo-workplan workplan <command>`, as cargo runs a subcommand.
    Cargo,
}

/// What a run of the binary did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Run {
    pub(crate) code: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

impl Run {
    /// The lines of standard output.
    pub(crate) fn lines(&self) -> Vec<&str> {
        self.stdout.lines().collect()
    }

    /// Require the exit code.
    pub(crate) fn expect(self, code: i32) -> Self {
        assert_eq!(self.code, code, "{self:#?}");
        self
    }
}

/// The command for the binary: closed standard input, the environment of the
/// test with the cargo variables that would change a nested cargo removed.
pub(crate) fn command(dir: &Path, shape: Shape, args: &[&str]) -> Command {
    let mut command = Command::new(BINARY);
    command
        .current_dir(dir)
        .stdin(Stdio::null())
        .env_remove("CARGO_TARGET_DIR");
    if shape == Shape::Cargo {
        command.arg("workplan");
    }
    command.args(args);
    command
}

/// Run the binary in `dir` in one shape.
pub(crate) fn run_in(dir: &Path, shape: Shape, args: &[&str]) -> Run {
    finish(&mut command(dir, shape, args))
}

/// Run a prepared command to its end.
pub(crate) fn finish(command: &mut Command) -> Run {
    let output = command.output().expect("the binary runs");
    Run {
        code: output.status.code().expect("an exit code, not a signal"),
        stdout: String::from_utf8(output.stdout).expect("UTF-8 on standard output"),
        stderr: String::from_utf8(output.stderr).expect("UTF-8 on standard error"),
    }
}

/// Run the binary in `dir` as `cargo-workplan <args>` and as `cargo-workplan
/// workplan <args>`, require the two to agree in every byte and every code,
/// and return the result.
pub(crate) fn run_both(dir: &Path, args: &[&str]) -> Run {
    let direct = run_in(dir, Shape::Direct, args);
    let cargo = run_in(dir, Shape::Cargo, args);
    assert_eq!(direct, cargo, "the two shapes of {args:?} differ");
    direct
}

/// Every file below `root`: its path with `/` separators and its bytes. A
/// symbolic link is listed by its target text and is not followed.
pub(crate) fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, into: &mut BTreeMap<String, Vec<u8>>) {
        for entry in
            fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()))
        {
            let entry = entry.expect("an entry");
            let path = entry.path();
            let name = path
                .strip_prefix(root)
                .expect("below the root")
                .to_string_lossy()
                .replace('\\', "/");
            let kind = entry.file_type().expect("a file type");
            if kind.is_symlink() {
                let target = fs::read_link(&path).expect("a link target");
                into.insert(
                    format!("{name} -> link"),
                    target.to_string_lossy().into_owned().into_bytes(),
                );
            } else if kind.is_dir() {
                into.insert(format!("{name}/"), Vec::new());
                walk(root, &path, into);
            } else {
                into.insert(name, fs::read(&path).expect("file bytes"));
            }
        }
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    files
}

/// The seven sections of a capability, in order.
const SECTIONS: [&str; 7] = [
    "Purpose",
    "Contract",
    "Usage",
    "Examples",
    "Constraints",
    "Explanation",
    "Verification",
];

/// A capability document. `bindings` is the text after `code:`, `tests:` and
/// `examples:`; `extra` is added after the text of each section, by name.
pub(crate) fn capability(
    slug: &str,
    state: &str,
    bindings: [&str; 3],
    depends: &str,
    extra: &[(&str, &str)],
) -> String {
    let [code, tests, examples] = bindings;
    let sections = SECTIONS
        .iter()
        .map(|name| {
            let added: String = extra
                .iter()
                .filter(|(section, _)| section == name)
                .map(|(_, text)| *text)
                .collect();
            format!("## {name}\n\nText for {name} of {slug}.\n{added}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\n\
         type: Capability\nid: cap:{slug}\ntitle: \"Title of {slug}\"\n\
         description: A description of {slug}.\nstate: {state}\n\
         bindings:\n  code: {code}\n  tests: {tests}\n  examples: {examples}\n\
         depends: {depends}\n---\n\n# {slug}\n\n{sections}"
    )
}

/// A change record listing `changes` (the text after `changes:`) and
/// affecting the given sections, with no item.
pub(crate) fn change(slug: &str, changes: &str, affects: &[&str]) -> String {
    let affects = affects.iter().fold(String::new(), |mut text, entry| {
        let _ = writeln!(text, "  - {entry}");
        text
    });
    format!(
        "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\n\
         type: Change\nid: change:{slug}\ntitle: Change {slug}\n\
         description: Changes {slug}.\nchanges: {changes}\naffects:\n{affects}\
         contract_effect: changes\nitems: []\n---\n\n## Summary\n\nChanges it.\n"
    )
}

/// The sections of a capability that hold the generated blocks of the
/// verification matrix and the relationships, so a sync has something to
/// rewrite when a change record comes or goes.
pub(crate) const BLOCKS: &str = "\n<!-- raf:generated:verification-matrix -->\n<!-- raf:generated:end -->\n\n<!-- raf:generated:relationships -->\n<!-- raf:generated:end -->\n";

/// Run `git` in `dir` as a fixture step, with the variables of an enclosing git
/// command and the user's configuration out of the way. Returns standard
/// output without its last line ending.
pub(crate) fn git(dir: &Path, args: &[&str]) -> String {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.excludesFile=",
        ])
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        // The user's own ignore file (under the config home) must not hide fixture files.
        .env("XDG_CONFIG_HOME", dir)
        .env("HOME", dir)
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid");
    for variable in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_COMMON_DIR",
        "GIT_PREFIX",
    ] {
        command.env_remove(variable);
    }
    let output = command.output().expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("UTF-8 output");
    text.strip_suffix('\n').unwrap_or(&text).to_string()
}

/// A library with a test that passes and one that fails, an example that
/// builds and runs, and one that exits with a failing status: what the
/// fixtures that run the real `cargo` bind.
const LIB: &str = "\
//! Fixture library.

/// Documented.
pub fn real() {}

/// Adds.
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[cfg(test)]
mod tests {
    #[test]
    fn passes() {
        assert_eq!(super::add(1, 2), 3);
    }

    #[test]
    fn fails() {
        panic!(\"this test fails\");
    }
}
";

/// An example that builds and runs.
pub(crate) const GOOD_EXAMPLE: &str = "fn main() {\n    println!(\"fine\");\n}\n";

/// An example that builds and exits with a failing status.
pub(crate) const EXITING_EXAMPLE: &str = "fn main() {\n    std::process::exit(3);\n}\n";

/// The directory the fixtures that run `cargo` build into, one for each test
/// binary and name, so no build writes below a fixture.
pub(crate) fn shared_target(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::create_dir_all(&dir).expect("the shared target directory");
    dir
}

impl Repo {
    /// A workspace that the real `cargo` builds and runs: the package `demo`
    /// with a library of two tests (`demo::tests::passes` and
    /// `demo::tests::fails`) and the examples `good` and `exiting`, building
    /// into `target`.
    pub(crate) fn real(tag: &str, target: &Path) -> Self {
        let repo = Self::bare(tag);
        repo.put("src/lib.rs", LIB);
        repo.put("examples/good.rs", GOOD_EXAMPLE);
        repo.put("examples/exiting.rs", EXITING_EXAMPLE);
        repo.put(
            ".cargo/config.toml",
            &format!("[build]\ntarget-dir = '{}'\n", target.display()),
        );
        repo
    }

    /// Run `git` in the root.
    pub(crate) fn git(&self, args: &[&str]) -> String {
        git(self.root(), args)
    }

    /// Make the root a git checkout on the branch `feature`, forked from
    /// `main` at a commit of the files now there, with `origin/main` at that
    /// commit and `origin/HEAD` naming it, as a clone leaves them.
    pub(crate) fn checkout(&self) {
        self.git(&["init", "-q", "-b", "main"]);
        self.commit("base");
        self.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        self.git(&[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ]);
        self.git(&["checkout", "-q", "-b", "feature"]);
    }

    /// Add everything and commit.
    pub(crate) fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]);
    }
}

/// The one array standard output holds.
pub(crate) fn array(run: &Run) -> Vec<Value> {
    let value: Value = serde_json::from_str(&run.stdout)
        .unwrap_or_else(|e| panic!("standard output is not one JSON value: {e}\n{run:#?}"));
    match value {
        Value::Array(items) => items,
        other => panic!("standard output is not an array: {other}"),
    }
}

/// The member names of each object of the array, in the order written.
///
/// The array is printed one object to a line, so the names are read from the
/// text: `serde_json::Value` sorts them.
pub(crate) fn member_names(run: &Run) -> Vec<Vec<String>> {
    run.stdout
        .lines()
        .filter(|line| line.starts_with('{'))
        .map(|line| {
            let line = line.trim_end_matches(',');
            let mut names = Vec::new();
            let mut depth = 0_i32;
            let mut chars = line.char_indices().peekable();
            while let Some((at, c)) = chars.next() {
                match c {
                    '{' | '[' => depth += 1,
                    '}' | ']' => depth -= 1,
                    '"' => {
                        // A string: find its end, honouring escapes.
                        let start = at;
                        let mut end = at;
                        let mut escaped = false;
                        for (i, d) in chars.by_ref() {
                            end = i;
                            if escaped {
                                escaped = false;
                            } else if d == '\\' {
                                escaped = true;
                            } else if d == '"' {
                                break;
                            }
                        }
                        let is_name = depth == 1 && line[end + 1..].starts_with(':');
                        if is_name {
                            names.push(line[start + 1..end].to_string());
                        }
                    }
                    _ => {}
                }
            }
            names
        })
        .collect()
}

/// The names as owned strings.
pub(crate) fn strings(names: &[&str]) -> Vec<String> {
    names.iter().map(ToString::to_string).collect()
}

/// The files that differ between two snapshots: those added, removed or
/// changed, sorted. A directory is not a file and is left out.
pub(crate) fn changed(
    before: &BTreeMap<String, Vec<u8>>,
    after: &BTreeMap<String, Vec<u8>>,
) -> Vec<String> {
    let mut paths: Vec<String> = before
        .keys()
        .chain(after.keys())
        .filter(|path| !path.ends_with('/') && before.get(*path) != after.get(*path))
        .cloned()
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

/// The snapshot of a work tree: every file but those of `.git`, which `git`
/// itself keeps.
pub(crate) fn worktree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    snapshot(root)
        .into_iter()
        .filter(|(path, _)| path != ".git/" && !path.starts_with(".git/"))
        .collect()
}

impl Repo {
    /// The snapshot of the work tree, without `.git`.
    pub(crate) fn worktree(&self) -> BTreeMap<String, Vec<u8>> {
        worktree(self.root())
    }
}
