// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The per-platform handle verifiers and the window between the prefix check
//! and the open. Each verifier is plain code that compiles everywhere, so
//! every one is exercised here whatever the platform.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

/// A fresh directory under the system temporary directory, removed on drop.
struct Scratch {
    dir: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "raf-source-unit-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).expect("create scratch directory");
        Self {
            dir: fs::canonicalize(&dir).expect("canonicalise scratch directory"),
        }
    }

    fn dir(&self, name: &str) -> PathBuf {
        let path = self.dir.join(name);
        fs::create_dir_all(&path).expect("create directory");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Best effort on teardown: a leftover temporary directory fails nothing.
        drop(fs::remove_dir_all(&self.dir));
    }
}

/// Run `work` on its own thread; fail when it has not finished in 10 seconds.
fn within_watchdog<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        // The receiver is gone only when the watchdog already fired.
        drop(sender.send(work()));
    });
    receiver
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("the operation blocked past the watchdog")
}

fn write(path: &Path, text: &str) {
    fs::write(path, text).expect("write file");
}

fn is_outside(result: &Result<(), TreeError>) -> bool {
    matches!(result, Err(TreeError::OutsideRoot { .. }))
}

#[cfg(unix)]
fn id_of(path: &Path) -> FileId {
    file_id(&fs::metadata(path).expect("metadata"), "x").expect("file id")
}

#[cfg(unix)]
#[test]
fn linux_verifier_refuses_a_handle_link_that_leads_outside_and_accepts_one_inside() {
    let scratch = Scratch::new();
    let root = scratch.dir("root");
    let outside = scratch.dir("outside");
    write(&root.join("in.rs"), "in");
    write(&outside.join("out.rs"), "out");
    std::os::unix::fs::symlink(outside.join("out.rs"), scratch.dir.join("to_out")).expect("link");
    std::os::unix::fs::symlink(root.join("in.rs"), scratch.dir.join("to_in")).expect("link");

    assert!(is_outside(&verify_linux_link(
        &root,
        &scratch.dir.join("to_out"),
        "out.rs"
    )));
    assert!(verify_linux_link(&root, &scratch.dir.join("to_in"), "in.rs").is_ok());
}

#[cfg(target_os = "linux")]
#[test]
fn linux_verifier_reads_the_real_descriptor_of_an_open_file() {
    let scratch = Scratch::new();
    let root = scratch.dir("root");
    let outside = scratch.dir("outside");
    write(&root.join("in.rs"), "in");
    write(&outside.join("out.rs"), "out");

    let inside = File::open(root.join("in.rs")).expect("open");
    let beyond = File::open(outside.join("out.rs")).expect("open");
    let inside_link = descriptor_link(&inside, "in.rs").expect("link");
    let beyond_link = descriptor_link(&beyond, "out.rs").expect("link");

    assert!(verify_linux_link(&root, &inside_link, "in.rs").is_ok());
    assert!(is_outside(&verify_linux_link(
        &root,
        &beyond_link,
        "out.rs"
    )));
}

#[cfg(unix)]
#[test]
fn unix_verifier_refuses_differing_identities_and_accepts_equal_ones() {
    let scratch = Scratch::new();
    let root = scratch.dir("root");
    let outside = scratch.dir("outside");
    write(&root.join("a.rs"), "a");
    write(&root.join("b.rs"), "b");
    write(&outside.join("c.rs"), "c");
    let a = id_of(&root.join("a.rs"));
    let b = id_of(&root.join("b.rs"));
    let c = id_of(&outside.join("c.rs"));

    assert!(verify_same_file(&root, &root.join("a.rs"), a, a, "a.rs").is_ok());
    assert!(is_outside(&verify_same_file(
        &root,
        &root.join("a.rs"),
        a,
        b,
        "a.rs"
    )));
    // The path is outside the root even though the identities agree.
    assert!(is_outside(&verify_same_file(
        &root,
        &outside.join("c.rs"),
        c,
        c,
        "c.rs"
    )));
}

#[test]
fn portable_verifier_refuses_a_non_regular_handle_and_an_outside_path_beside_an_accepted_one() {
    let scratch = Scratch::new();
    let root = scratch.dir("root");
    let outside = scratch.dir("outside");
    write(&root.join("a.rs"), "a");
    write(&outside.join("c.rs"), "c");
    let file = fs::metadata(root.join("a.rs")).expect("metadata");
    let directory = fs::metadata(&root).expect("metadata");

    assert!(verify_portable(&root, &file, &root.join("a.rs"), "a.rs").is_ok());
    assert!(matches!(
        verify_portable(&root, &directory, &root.join("a.rs"), "a.rs"),
        Err(TreeError::NotRegular { .. })
    ));
    assert!(is_outside(&verify_portable(
        &root,
        &file,
        &outside.join("c.rs"),
        "c.rs"
    )));
}

#[cfg(unix)]
mod swap {
    use super::*;

    fn make_fifo(path: &Path) {
        let status = std::process::Command::new("mkfifo")
            .arg(path)
            .status()
            .expect("run mkfifo");
        assert!(status.success(), "mkfifo failed");
    }

    /// A tree with `root/dir/file.rs`, and an `outside/dir` holding a FIFO of
    /// the same name. A FIFO opened for reading without a writer would block
    /// a blocking read forever, so a test that finishes has not read it.
    fn fixture() -> (Scratch, PathBuf, PathBuf) {
        let scratch = Scratch::new();
        let root = scratch.dir("root");
        scratch.dir("root/dir");
        scratch.dir("outside/dir");
        write(&root.join("dir/file.rs"), "inside");
        make_fifo(&scratch.dir.join("outside/dir/file.rs"));
        let outside_dir = scratch.dir.join("outside/dir");
        (scratch, root, outside_dir)
    }

    #[test]
    fn a_directory_swapped_for_an_outside_symlink_after_the_check_is_refused() {
        let (_scratch, root, outside_dir) = fixture();
        let mut tree = DiskTree::new(&root).expect("tree");
        let dir = root.join("dir");
        let parked = root.join("dir.parked");
        let target = outside_dir.clone();
        tree.before_open = Some(Box::new(move |_canonical| {
            fs::rename(&dir, &parked).expect("park the directory");
            std::os::unix::fs::symlink(&target, &dir).expect("plant the symlink");
        }));

        let result =
            within_watchdog(move || tree.read(&RelPath::new("dir/file.rs").expect("path"), 1024));

        assert!(
            matches!(result, Err(TreeError::OutsideRoot { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn the_same_read_without_a_swap_is_accepted() {
        let (_scratch, root, _outside_dir) = fixture();
        let mut tree = DiskTree::new(&root).expect("tree");
        tree.before_open = Some(Box::new(|_canonical| {}));

        let text = tree.read(&RelPath::new("dir/file.rs").expect("path"), 1024);

        assert_eq!(text.expect("read"), "inside");
    }

    #[test]
    fn a_directory_swapped_for_an_outside_symlink_before_a_listing_is_refused() {
        let scratch = Scratch::new();
        let root = scratch.dir("root");
        scratch.dir("root/dir/sub");
        scratch.dir("outside/dir/sub");
        write(&root.join("dir/sub/inside.rs"), "inside");
        write(&scratch.dir.join("outside/dir/sub/leaked.rs"), "outside");
        let mut tree = DiskTree::new(&root).expect("tree");
        let dir = root.join("dir");
        let parked = root.join("dir.parked");
        let target = scratch.dir.join("outside/dir");
        tree.before_open = Some(Box::new(move |_canonical| {
            fs::rename(&dir, &parked).expect("park the directory");
            std::os::unix::fs::symlink(&target, &dir).expect("plant the symlink");
        }));

        let result = within_watchdog(move || tree.list(&RelPath::new("dir/sub").expect("path")));

        assert!(
            matches!(result, Err(TreeError::OutsideRoot { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn the_same_listing_without_a_swap_returns_the_entries() {
        let scratch = Scratch::new();
        let root = scratch.dir("root");
        scratch.dir("root/dir/sub");
        write(&root.join("dir/sub/inside.rs"), "inside");
        let mut tree = DiskTree::new(&root).expect("tree");
        tree.before_open = Some(Box::new(|_canonical| {}));

        let names: Vec<String> = tree
            .list(&RelPath::new("dir/sub").expect("path"))
            .expect("list")
            .into_iter()
            .map(|entry| entry.name)
            .collect();

        assert_eq!(names, ["inside.rs"]);
    }
}

#[test]
fn a_missing_descriptor_link_is_an_io_failure_and_never_absence() {
    let scratch = Scratch::new();
    let root = scratch.dir("root");
    let missing = scratch.dir.join("no-proc/self/fd/3");

    let error = verify_linux_link(&root, &missing, "a.rs").expect_err("no link");

    assert!(matches!(error, TreeError::Io { .. }), "{error:?}");
    let mapped = link_read_error("a.rs", io::Error::from(io::ErrorKind::NotFound));
    assert!(matches!(mapped, TreeError::Io { .. }), "{mapped:?}");
}

#[test]
fn portable_directory_verifier_refuses_a_file_and_an_outside_path_beside_an_accepted_one() {
    let scratch = Scratch::new();
    let root = scratch.dir("root");
    let outside = scratch.dir("outside");
    write(&root.join("a.rs"), "a");
    let directory = fs::metadata(&root).expect("metadata");
    let file = fs::metadata(root.join("a.rs")).expect("metadata");

    assert!(verify_portable_directory(&root, &directory, &root, "d").is_ok());
    assert!(matches!(
        verify_portable_directory(&root, &file, &root, "d"),
        Err(TreeError::NotRegular { .. })
    ));
    assert!(is_outside(&verify_portable_directory(
        &root, &directory, &outside, "d"
    )));
}
