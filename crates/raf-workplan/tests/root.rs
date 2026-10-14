// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Root discovery against the real `cargo` in workspaces written to a scratch
//! directory, and against stub programs on Unix: the root is the workspace
//! root from a member and from a subdirectory, a directory outside any
//! workspace is an error, and the `build.rustc`, `build.rustc-wrapper` and
//! `build.rustc-workspace-wrapper` programs of the workspace's cargo
//! configuration are not run by discovery.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use raf_workplan::exec::{ExecError, Programs, SystemRunner};
use raf_workplan::root::{RootError, discover};

/// A stub is written and started while other tests start `cargo`; a forked
/// child briefly holding the descriptor that wrote the file makes the start
/// fail with "text file busy". Every test holds this lock from its first
/// process to its last.
static SPAWN: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SPAWN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A scratch directory, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!(
            "raf-workplan-root-{}-{nanos}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("a scratch directory");
        Self(fs::canonicalize(dir).expect("a canonical scratch directory"))
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn put(&self, path: &str, text: &str) {
        let target = self.0.join(path);
        fs::create_dir_all(target.parent().expect("a file has a directory"))
            .expect("the directory of a file");
        fs::write(target, text).expect("a file");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A workspace of one package at its root, with its own `[workspace]` and no
/// dependency, so that nothing is fetched and no enclosing workspace is read.
fn single(scratch: &Scratch) {
    scratch.put(
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
    );
    scratch.put(
        "Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    );
    scratch.put("src/lib.rs", "");
}

/// A virtual workspace of two members.
fn virtual_workspace(scratch: &Scratch) {
    scratch.put(
        "Cargo.toml",
        "[workspace]\nresolver = \"2\"\nmembers = [\"crates/one\", \"crates/two\"]\n",
    );
    for name in ["one", "two"] {
        scratch.put(
            &format!("crates/{name}/Cargo.toml"),
            &format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        );
        scratch.put(&format!("crates/{name}/src/lib.rs"), "");
    }
    scratch.put(
        "Cargo.lock",
        "version = 4\n\n[[package]]\nname = \"one\"\nversion = \"0.1.0\"\n\n[[package]]\nname = \"two\"\nversion = \"0.1.0\"\n",
    );
}

fn discovered(dir: &Path) -> Result<PathBuf, RootError> {
    discover(&SystemRunner::new(Programs::new(dir)))
}

fn same_path(left: &Path, right: &Path) -> bool {
    fs::canonicalize(left).expect("left exists") == fs::canonicalize(right).expect("right exists")
}

#[test]
fn the_root_is_found_from_the_root_a_member_and_a_subdirectory() {
    let _lock = serial();
    let single_scratch = Scratch::new();
    single(&single_scratch);
    let root = discovered(single_scratch.path()).expect("a root");
    assert!(same_path(&root, single_scratch.path()), "{root:?}");
    assert!(root.is_absolute());
    // From a directory below the root that is not a package.
    single_scratch.put("src/deep/note.txt", "");
    let from_below = discovered(&single_scratch.path().join("src/deep")).expect("a root");
    assert!(
        same_path(&from_below, single_scratch.path()),
        "{from_below:?}"
    );

    let members = Scratch::new();
    virtual_workspace(&members);
    for start in ["", "crates/one", "crates/two/src"] {
        let found = discovered(&members.path().join(start)).expect("a root");
        assert!(same_path(&found, members.path()), "{start}: {found:?}");
    }
}

#[test]
fn a_directory_outside_any_workspace_is_an_error_that_says_what_cargo_said() {
    let _lock = serial();
    let scratch = Scratch::new();
    scratch.put("note.txt", "no manifest here\n");
    let error = discovered(scratch.path()).expect_err("no workspace");
    assert!(
        matches!(&error, RootError::Failed { stderr, .. } if stderr.contains("Cargo.toml")),
        "{error:?}"
    );
    let message = error.to_string();
    assert!(message.contains("cargo metadata"), "{message}");
}

#[test]
fn a_cargo_that_does_not_exist_is_an_error() {
    let _lock = serial();
    let scratch = Scratch::new();
    single(&scratch);
    let programs = Programs::new(scratch.path()).with_cargo(scratch.path().join("no-such-cargo"));
    let error = discover(&SystemRunner::new(programs)).expect_err("no cargo");
    assert!(
        matches!(error, RootError::Exec(ExecError::Spawn { .. })),
        "{error:?}"
    );
}

// --------------------------------------------- the cargo configuration is inert

/// The programs of a workspace's `.cargo/config.toml` are run by a build and
/// not by discovery.
#[cfg(unix)]
mod configured_programs {
    use std::fmt::Write as _;
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    use std::process::Command;

    use super::*;

    /// A script that appends its name to the marker file and then runs the
    /// rest of its command line.
    fn script(scratch: &Scratch, name: &str, marker: &Path, body: &str) -> PathBuf {
        let path = scratch.path().join("tools").join(name);
        fs::create_dir_all(path.parent().expect("a directory")).expect("a tools directory");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o755)
            .open(&path)
            .expect("a script file");
        write!(
            file,
            "#!/bin/sh\necho {name} >> '{}'\n{body}\n",
            marker.display()
        )
        .expect("a script body");
        path
    }

    /// A workspace whose configuration names a script for each of the keys
    /// given (`rustc`, `rustc-wrapper`, `rustc-workspace-wrapper`). Returns the
    /// marker file the scripts write to.
    fn configured(scratch: &Scratch, keys: &[&str]) -> PathBuf {
        single(scratch);
        let marker = scratch.path().join("ran.txt");
        let mut config = String::from("[build]\n");
        for key in keys {
            let body = if *key == "rustc" {
                "exec rustc \"$@\""
            } else {
                "exec \"$@\""
            };
            let tool = script(scratch, key, &marker, body);
            writeln!(config, "{key} = \"{}\"", tool.display()).expect("a string");
        }
        scratch.put(".cargo/config.toml", &config);
        marker
    }

    /// A `cargo` that runs the real one with the environment variables that
    /// override the configured programs removed, so that the configuration of
    /// the fixture is what decides.
    fn clean_cargo(scratch: &Scratch) -> PathBuf {
        let real = Programs::new(scratch.path()).cargo;
        let path = scratch.path().join("tools").join("cargo");
        fs::create_dir_all(path.parent().expect("a directory")).expect("a tools directory");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o755)
            .open(&path)
            .expect("a script file");
        write!(
            file,
            "#!/bin/sh\nunset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER \\\n  CARGO_BUILD_RUSTC CARGO_BUILD_RUSTC_WRAPPER CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER\nexec '{}' \"$@\"\n",
            real.display()
        )
        .expect("a script body");
        path
    }

    fn marker_text(marker: &Path) -> String {
        fs::read_to_string(marker).unwrap_or_default()
    }

    /// Run `cargo check` in the workspace, which runs the programs of the
    /// configuration, and return the names that were recorded.
    fn names_run_by_a_build(scratch: &Scratch, marker: &Path) -> Vec<String> {
        let build = Command::new(clean_cargo(scratch))
            .args(["check", "--offline", "--quiet"])
            .current_dir(scratch.path())
            .env("CARGO_TARGET_DIR", scratch.path().join("target"))
            .output()
            .expect("cargo runs");
        let ran = marker_text(marker);
        assert!(
            !ran.is_empty(),
            "a build runs a program of the configuration; if it ran none, \
             the fixture cannot show a program running; cargo said: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        fs::remove_file(marker).expect("the marker is there");
        ran.lines().map(str::to_string).collect()
    }

    #[test]
    fn discovery_does_not_run_the_programs_of_the_cargo_configuration() {
        let _lock = serial();
        // The controls: a build of the same workspace does run them, so the
        // fixture is able to show a program running. `rustc-workspace-wrapper`
        // takes over from `rustc-wrapper` for the workspace's own crates, so
        // each is shown in a workspace of its own.
        let all = Scratch::new();
        let marker = configured(&all, &["rustc", "rustc-wrapper", "rustc-workspace-wrapper"]);
        let ran = names_run_by_a_build(&all, &marker);
        for name in ["rustc", "rustc-workspace-wrapper"] {
            assert!(ran.iter().any(|line| line == name), "{name} in {ran:?}");
        }
        let wrapped = Scratch::new();
        let wrapper_marker = configured(&wrapped, &["rustc-wrapper"]);
        let ran = names_run_by_a_build(&wrapped, &wrapper_marker);
        assert!(ran.iter().any(|line| line == "rustc-wrapper"), "{ran:?}");

        // Discovery gives the root and runs none of them, in either workspace.
        for (scratch, marker) in [(&all, &marker), (&wrapped, &wrapper_marker)] {
            let programs =
                Programs::new(scratch.path()).with_cargo(scratch.path().join("tools/cargo"));
            let root = discover(&SystemRunner::new(programs)).expect("a root");
            assert!(same_path(&root, scratch.path()), "{root:?}");
            assert!(
                !marker.exists(),
                "discovery ran a configured program: {:?}",
                marker_text(marker)
            );
        }
    }
}

// ------------------------------------------------------------- stub `cargo`

#[cfg(unix)]
mod stubs {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;

    use raf_workplan::exec::METADATA_OUTPUT_BOUND;

    use super::*;

    /// A stub `cargo` in a scratch directory, with the body given.
    fn stub(scratch: &Scratch, body: &str) -> PathBuf {
        let path = scratch.path().join("cargo");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o755)
            .open(&path)
            .expect("a stub file");
        write!(file, "#!/bin/sh\n{body}\n").expect("a stub body");
        path
    }

    fn runner(scratch: &Scratch, cargo: &Path) -> SystemRunner {
        fs::create_dir_all(scratch.path().join("work")).expect("a work directory");
        SystemRunner::new(Programs::new(scratch.path().join("work")).with_cargo(cargo))
    }

    #[test]
    fn exactly_one_process_runs_with_the_documented_argv_directory_and_locale() {
        let _lock = serial();
        let scratch = Scratch::new();
        let record = scratch.path().join("record");
        let cargo = stub(
            &scratch,
            &format!(
                "printf '%s\\037' \"$@\" >> '{record}'\nprintf '\\n' >> '{record}'\npwd -P >> '{record}'\nprintf '%s\\n' \"${{LC_ALL-<unset>}}\" >> '{record}'\nread -r line && echo stdin-open >> '{record}'\necho '{{\"workspace_root\":\"/the/root\"}}'",
                record = record.display()
            ),
        );
        let root = discover(&runner(&scratch, &cargo)).expect("a root");
        assert_eq!(root, Path::new("/the/root"));
        let text = fs::read_to_string(&record).expect("a record");
        let work = fs::canonicalize(scratch.path().join("work")).expect("a work directory");
        assert_eq!(
            text,
            format!(
                "metadata\u{1f}--format-version\u{1f}1\u{1f}--no-deps\u{1f}--frozen\u{1f}\n{}\nC\n",
                work.display()
            ),
            "one record: the argv, the directory, the locale, and a closed standard input"
        );
    }

    #[test]
    fn the_top_level_member_wins_over_a_nested_one_with_the_same_name() {
        let _lock = serial();
        let scratch = Scratch::new();
        let cargo = stub(
            &scratch,
            r#"cat <<'JSON'
{"packages":[{"name":"evil","metadata":{"workspace_root":"/evil","note":"\"workspace_root\":\"/evil2\""}}],"workspace_root":"/real","metadata":{"workspace_root":"/evil3"}}
JSON"#,
        );
        assert_eq!(
            discover(&runner(&scratch, &cargo)).expect("a root"),
            Path::new("/real")
        );
    }

    #[test]
    fn hostile_output_is_an_error_and_never_a_guess() {
        let _lock = serial();
        let scratch = Scratch::new();
        for (index, output) in [
            "",
            "not json",
            r#"{"workspace_root":"/a",}"#,
            r#"{"workspace_root":"/a"} {"workspace_root":"/b"}"#,
            r#"{"workspace_root":["/a"]}"#,
            r#"{"workspace_root":"relative"}"#,
            r#"{"x":{"workspace_root":"/a"}}"#,
        ]
        .into_iter()
        .enumerate()
        {
            let dir = Scratch::new();
            let cargo = stub(&dir, &format!("cat <<'JSON'\n{output}\nJSON"));
            let error = discover(&runner(&dir, &cargo)).expect_err(output);
            assert!(
                matches!(error, RootError::Json(_) | RootError::NotAbsolute { .. }),
                "{index} {output:?}: {error:?}"
            );
        }
        // The neighbour of them all passes.
        let cargo = stub(&scratch, "echo '{\"workspace_root\":\"/a\"}'");
        assert_eq!(
            discover(&runner(&scratch, &cargo)).expect("a root"),
            Path::new("/a")
        );
    }

    #[test]
    fn output_is_bounded_at_64_mebibytes() {
        let _lock = serial();
        // Exactly the bound passes: the document and then spaces to the bound.
        let scratch = Scratch::new();
        let document = "{\"workspace_root\":\"/a\"}";
        let cargo = stub(
            &scratch,
            &format!(
                "printf '%s' '{document}'\nhead -c {} /dev/zero | tr '\\0' ' '",
                METADATA_OUTPUT_BOUND - document.len()
            ),
        );
        assert_eq!(
            discover(&runner(&scratch, &cargo)).expect("a root at the bound"),
            Path::new("/a")
        );

        // One byte more is killed and refused.
        let over = Scratch::new();
        let cargo = stub(
            &over,
            &format!(
                "printf '%s' '{document}'\nhead -c {} /dev/zero | tr '\\0' ' '",
                METADATA_OUTPUT_BOUND - document.len() + 1
            ),
        );
        let error = discover(&runner(&over, &cargo)).expect_err("one byte over");
        assert!(
            matches!(
                &error,
                RootError::Exec(ExecError::OutputTooLarge { limit, .. }) if *limit == METADATA_OUTPUT_BOUND
            ),
            "{error:?}"
        );
    }
}
