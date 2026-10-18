// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The handle verifiers, the windows around the open and around each use of a
//! verified handle, and the containment fixtures, run under every strategy
//! the platform can run: on Linux all three, through the test override.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

/// Every strategy this platform can run; on Linux, all three.
const STRATEGIES: &[Strategy] = &[
    #[cfg(target_os = "linux")]
    Strategy::Linux,
    #[cfg(unix)]
    Strategy::OtherUnix,
    #[cfg(any(windows, target_os = "linux"))]
    Strategy::Windows,
];

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

fn write(path: &Path, text: &str) {
    fs::write(path, text).expect("write file");
}

/// A tree at `root` that verifies with `strategy`.
fn tree_with(root: &Path, strategy: Strategy) -> DiskTree {
    let mut tree = DiskTree::new(root).expect("tree");
    tree.strategy = strategy;
    tree
}

fn rel(text: &str) -> RelPath {
    RelPath::new(text).expect("valid path")
}

#[cfg(unix)]
fn make_fifo(path: &Path) {
    let status = std::process::Command::new("mkfifo")
        .arg(path)
        .status()
        .expect("run mkfifo");
    assert!(status.success(), "mkfifo failed");
}

#[cfg(unix)]
fn symlink(target: impl AsRef<Path>, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create symlink");
}

#[test]
fn the_native_strategy_is_the_platforms_own() {
    let scratch = Scratch::new();
    let tree = DiskTree::new(scratch.dir("root")).expect("tree");
    let expected = if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(unix) {
        "OtherUnix"
    } else {
        "Windows"
    };
    assert_eq!(format!("{:?}", tree.strategy), expected);
}

#[cfg(any(target_os = "linux", windows))]
#[test]
fn the_handle_path_verifier_refuses_a_path_outside_the_root_beside_one_inside() {
    let scratch = Scratch::new();
    let root = scratch.dir("root");
    let outside = scratch.dir("outside");

    assert!(verify_handle_path(&root, &root.join("a.rs"), "a.rs").is_ok());
    assert!(verify_handle_path(&root, &root, "").is_ok());
    for refused in [
        outside.join("a.rs"),
        scratch.dir.clone(),
        scratch.dir.join("rootx"),
    ] {
        assert!(
            matches!(
                verify_handle_path(&root, &refused, "a.rs"),
                Err(TreeError::OutsideRoot { .. })
            ),
            "{}",
            refused.display()
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn linux_verifier_refuses_a_handle_link_that_leads_outside_and_accepts_one_inside() {
    let scratch = Scratch::new();
    let root = scratch.dir("root");
    let outside = scratch.dir("outside");
    write(&root.join("in.rs"), "in");
    write(&outside.join("out.rs"), "out");
    symlink(outside.join("out.rs"), &scratch.dir.join("to_out"));
    symlink(root.join("in.rs"), &scratch.dir.join("to_in"));

    assert!(matches!(
        verify_linux_link(&root, &scratch.dir.join("to_out"), "out.rs"),
        Err(TreeError::OutsideRoot { .. })
    ));
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

    let inside = fs::File::open(root.join("in.rs")).expect("open");
    let beyond = fs::File::open(outside.join("out.rs")).expect("open");

    assert!(verify_linux_link(&root, &descriptor_path(&inside), "in.rs").is_ok());
    assert!(matches!(
        verify_linux_link(&root, &descriptor_path(&beyond), "out.rs"),
        Err(TreeError::OutsideRoot { .. })
    ));
}

#[cfg(target_os = "linux")]
#[test]
fn a_missing_descriptor_link_is_an_io_failure_and_never_absence() {
    let scratch = Scratch::new();
    let root = scratch.dir("root");
    let missing = scratch.dir.join("no-proc/self/fd/3");

    let error = verify_linux_link(&root, &missing, "a.rs").expect_err("no link");

    assert!(matches!(error, TreeError::Io { .. }), "{error:?}");
    let mapped = link_read_error("a.rs", std::io::Error::from(std::io::ErrorKind::NotFound));
    assert!(matches!(mapped, TreeError::Io { .. }), "{mapped:?}");
}

#[test]
fn reserved_device_names_and_trailing_dots_or_spaces_are_ambiguous_on_windows_beside_plain_names() {
    for ambiguous in [
        "con",
        "CON",
        "Prn",
        "aux",
        "nul",
        "nul.txt",
        "NUL.tar.gz",
        "aux.rs",
        "nul .txt",
        "nul:",
        "com1",
        "COM9.rs",
        "lpt0",
        "LPT3.txt",
        "com¹",
        "lpt³.rs",
        "conin$",
        "CONOUT$.x",
        "a.",
        "a ",
        "...",
        "x. ",
        "lib.rs:payload",
        "x::$DATA",
        ":",
    ] {
        assert!(is_ambiguous_on_windows(ambiguous), "{ambiguous:?}");
    }
    for plain in [
        "..",
        "console",
        "auxiliary.rs",
        "nul_device",
        "com",
        "com10",
        "lpt",
        "lptx",
        "com1x.rs",
        "a.rs",
        "a b",
        ".hidden",
        "connect.rs",
    ] {
        assert!(!is_ambiguous_on_windows(plain), "{plain:?}");
    }
}

#[cfg(windows)]
mod windows {
    use super::*;

    #[test]
    fn the_final_path_of_a_handle_is_its_canonical_path_and_lies_under_the_root() {
        let scratch = Scratch::new();
        let root = scratch.dir("root");
        scratch.dir("root/sub");
        write(&root.join("sub/inner.rs"), "inner");
        let file = fs::File::open(root.join("sub/inner.rs")).expect("open");

        let path = final_path::of(&file).expect("final path");

        assert_eq!(
            path,
            fs::canonicalize(root.join("sub/inner.rs")).expect("canonical")
        );
        assert!(verify_handle_path(&root, &path, "sub/inner.rs").is_ok());
    }

    #[test]
    fn a_disk_tree_refuses_a_reserved_device_name_or_a_trailing_dot_or_space_beside_a_plain_name() {
        let scratch = Scratch::new();
        let root = scratch.dir("root");
        scratch.dir("root/sub");
        write(&root.join("plain.rs"), "plain");
        write(&root.join("sub/inner.rs"), "inner");
        for &strategy in STRATEGIES {
            refuses_ambiguous_names_beside_plain_ones(&tree_with(&root, strategy));
        }
    }

    fn refuses_ambiguous_names_beside_plain_ones(tree: &DiskTree) {
        for refused in [
            "nul.txt",
            "aux/x.rs",
            "sub/con",
            "CON.rs",
            "a./x.rs",
            "b /x.rs",
            "plain.rs:stream",
        ] {
            assert!(
                matches!(
                    tree.read(&rel(refused), 1024),
                    Err(TreeError::OutsideRoot { .. })
                ),
                "{refused}"
            );
        }
        assert!(matches!(
            tree.entry(&rel("sub/nul")),
            Err(TreeError::OutsideRoot { .. })
        ));
        assert!(matches!(
            tree.list(&rel("lpt1")),
            Err(TreeError::OutsideRoot { .. })
        ));
        assert!(matches!(
            tree.identity(&rel("com1.rs")),
            Err(TreeError::OutsideRoot { .. })
        ));

        assert_eq!(tree.read(&rel("plain.rs"), 1024).expect("plain"), "plain");
        assert_eq!(
            tree.read(&rel("sub/../sub/inner.rs"), 1024).expect("inner"),
            "inner"
        );
        assert_eq!(tree.entry(&rel("sub")).expect("sub"), Some(EntryKind::Dir));
        assert_eq!(tree.list(&rel("sub")).expect("list").len(), 1);
    }
}

#[cfg(unix)]
mod swap {
    use super::*;

    /// A tree with `root/dir/file.rs` and `root/dir/sub/inside.rs`, and an
    /// `outside/dir` that holds a FIFO named `file.rs`, a `sub/leaked.rs` and
    /// a `sub/only_outside`. A FIFO opened for a blocking read without a
    /// writer would block forever, so a test that finishes has not read it.
    fn fixture() -> (Scratch, PathBuf, PathBuf) {
        let scratch = Scratch::new();
        let root = scratch.dir("root");
        scratch.dir("root/dir/sub");
        scratch.dir("outside/dir/sub");
        write(&root.join("dir/file.rs"), "inside");
        write(&root.join("dir/sub/inside.rs"), "inside");
        make_fifo(&scratch.dir.join("outside/dir/file.rs"));
        write(&scratch.dir.join("outside/dir/sub/leaked.rs"), "outside");
        write(&scratch.dir.join("outside/dir/sub/only_outside"), "outside");
        let outside_dir = scratch.dir.join("outside/dir");
        (scratch, root, outside_dir)
    }

    /// A hook that parks `root/dir` beside itself and puts a symlink to
    /// `outside/dir` in its place.
    fn park_and_link(root: &Path, outside_dir: &Path) -> Hook {
        let dir = root.join("dir");
        let parked = root.join("dir.parked");
        let target = outside_dir.to_path_buf();
        Box::new(move || {
            fs::rename(&dir, &parked).expect("park the directory");
            symlink(&target, &dir);
        })
    }

    /// A hook that moves `root/dir` out of the root and puts a symlink to
    /// `outside/dir` in its place, so both the verified handle and the path
    /// lead outside.
    fn move_out_and_link(root: &Path, outside_dir: &Path) -> Hook {
        let dir = root.join("dir");
        let moved = outside_dir.with_file_name("moved");
        let target = outside_dir.to_path_buf();
        Box::new(move || {
            fs::rename(&dir, &moved).expect("move the directory out");
            symlink(&target, &dir);
        })
    }

    fn assert_outside<T: std::fmt::Debug>(result: &Result<T, TreeError>, strategy: Strategy) {
        assert!(
            matches!(result, Err(TreeError::OutsideRoot { .. })),
            "{strategy:?}: {result:?}"
        );
    }

    #[test]
    fn a_directory_swapped_for_an_outside_symlink_after_the_check_is_refused() {
        for &strategy in STRATEGIES {
            let (_scratch, root, outside_dir) = fixture();
            let mut tree = tree_with(&root, strategy);
            tree.before_open = Some(park_and_link(&root, &outside_dir));

            let result = tree.read(&rel("dir/file.rs"), 1024);

            assert_outside(&result, strategy);
        }
    }

    #[test]
    fn the_same_read_without_a_swap_is_accepted() {
        for &strategy in STRATEGIES {
            let (_scratch, root, _outside_dir) = fixture();
            let mut tree = tree_with(&root, strategy);
            tree.before_open = Some(Box::new(|| {}));
            tree.after_verify = Some(Box::new(|| {}));

            let text = tree.read(&rel("dir/file.rs"), 1024);

            assert_eq!(text.expect("read"), "inside", "{strategy:?}");
        }
    }

    #[test]
    fn a_file_whose_directory_is_moved_out_after_verification_is_refused_after_the_read() {
        for &strategy in STRATEGIES {
            for unseen in [false, true] {
                let (_scratch, root, outside_dir) = fixture();
                let mut tree = tree_with(&root, strategy);
                tree.after_verify = Some(move_out_and_link(&root, &outside_dir));

                let path = rel("dir/file.rs");
                let result = if unseen {
                    tree.read_unseen(&path, 1024, &|_| false).map(|_| ())
                } else {
                    tree.read(&path, 1024).map(|_| ())
                };

                assert!(
                    matches!(result, Err(TreeError::OutsideRoot { .. })),
                    "{strategy:?}, unseen: {unseen}: {result:?}"
                );
            }
        }
    }

    #[test]
    fn a_directory_swapped_for_an_outside_symlink_before_a_listing_is_refused() {
        for &strategy in STRATEGIES {
            let (_scratch, root, outside_dir) = fixture();
            let mut tree = tree_with(&root, strategy);
            tree.before_open = Some(park_and_link(&root, &outside_dir));

            let result = tree.list(&rel("dir/sub"));

            assert_outside(&result, strategy);
        }
    }

    #[test]
    fn a_directory_swapped_after_verification_before_the_entries_are_read_is_refused() {
        for &strategy in STRATEGIES {
            let (_scratch, root, outside_dir) = fixture();
            let mut tree = tree_with(&root, strategy);
            tree.after_verify = Some(move_out_and_link(&root, &outside_dir));

            let result = tree.list(&rel("dir/sub"));

            assert_outside(&result, strategy);
        }
    }

    #[test]
    fn the_same_listing_without_a_swap_returns_the_entries() {
        for &strategy in STRATEGIES {
            let (_scratch, root, _outside_dir) = fixture();
            let mut tree = tree_with(&root, strategy);
            tree.before_open = Some(Box::new(|| {}));
            tree.after_verify = Some(Box::new(|| {}));

            let names: Vec<String> = tree
                .list(&rel("dir/sub"))
                .expect("list")
                .into_iter()
                .map(|entry| entry.name)
                .collect();

            assert_eq!(names, ["inside.rs"], "{strategy:?}");
        }
    }

    #[test]
    fn an_entry_whose_directory_is_swapped_for_an_outside_symlink_is_refused() {
        for &strategy in STRATEGIES {
            for before_open in [true, false] {
                let (_scratch, root, outside_dir) = fixture();
                let mut tree = tree_with(&root, strategy);
                if before_open {
                    tree.before_open = Some(park_and_link(&root, &outside_dir));
                } else {
                    tree.after_verify = Some(move_out_and_link(&root, &outside_dir));
                }

                let result = tree.entry(&rel("dir/sub/only_outside"));

                assert!(
                    matches!(result, Err(TreeError::OutsideRoot { .. })),
                    "{strategy:?}, before the open: {before_open}: {result:?}"
                );
            }
        }
    }

    #[test]
    fn the_same_entry_without_a_swap_is_found() {
        for &strategy in STRATEGIES {
            let (_scratch, root, _outside_dir) = fixture();
            let mut tree = tree_with(&root, strategy);
            tree.before_open = Some(Box::new(|| {}));
            tree.after_verify = Some(Box::new(|| {}));

            assert_eq!(
                tree.entry(&rel("dir/sub/inside.rs")).expect("entry"),
                Some(EntryKind::File),
                "{strategy:?}"
            );
            assert_eq!(
                tree.entry(&rel("dir/sub/only_outside")).expect("entry"),
                None,
                "{strategy:?}"
            );
        }
    }

    /// The open events queued on `queue` since it was last read, and none
    /// when the queue is empty.
    #[cfg(target_os = "linux")]
    fn queued_opens(queue: &std::os::fd::OwnedFd) -> Vec<rustix::fs::inotify::ReadFlags> {
        use rustix::fs::inotify::Reader;
        use rustix::io::Errno;

        let mut buffer = [std::mem::MaybeUninit::<u8>::uninit(); 1024];
        let mut reader = Reader::new(queue, &mut buffer);
        let mut events = Vec::new();
        loop {
            match reader.next() {
                Ok(event) => events.push(event.events()),
                Err(Errno::AGAIN) => return events,
                Err(error) => panic!("read the inotify queue: {error}"),
            }
        }
    }

    /// Under the Linux and other-Unix strategies an outside candidate is
    /// never opened; the handle-path strategy opens it and refuses it after.
    #[cfg(target_os = "linux")]
    #[test]
    fn an_outside_fifo_reached_by_a_swap_before_the_open_is_never_opened() {
        for strategy in [Strategy::Linux, Strategy::OtherUnix] {
            outside_fifo_is_never_opened(strategy);
        }
    }

    /// The probe is an inotify watch for `IN_OPEN` on the FIFO's inode. An
    /// `O_PATH` reference must not be reported (the negative control, which the
    /// Linux strategy relies on), and an `O_NONBLOCK` open must be, exactly
    /// once (the positive control, which proves the watch is live).
    #[cfg(target_os = "linux")]
    fn outside_fifo_is_never_opened(strategy: Strategy) {
        use rustix::fs::inotify::{self, CreateFlags, ReadFlags, WatchFlags};
        use std::os::unix::fs::OpenOptionsExt as _;

        let (_scratch, root, outside_dir) = fixture();
        let fifo = outside_dir.join("file.rs");
        let queue = inotify::init(CreateFlags::NONBLOCK | CreateFlags::CLOEXEC)
            .expect("create an inotify queue");
        inotify::add_watch(&queue, &fifo, WatchFlags::OPEN).expect("watch the FIFO for opens");

        let reference = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_PATH)
            .open(&fifo)
            .expect("take an O_PATH reference to the FIFO");
        drop(reference);
        let reported = queued_opens(&queue);
        assert!(
            reported.is_empty(),
            "this kernel reports an O_PATH reference as an open ({reported:?}), so the probe \
             cannot tell a reference from an open of the FIFO"
        );

        let mut tree = tree_with(&root, strategy);
        tree.before_open = Some(park_and_link(&root, &outside_dir));
        let result = tree.read(&rel("dir/file.rs"), 1024);

        assert_outside(&result, strategy);
        let opened = queued_opens(&queue);
        assert!(
            opened.is_empty(),
            "{strategy:?}: the outside FIFO was opened: {opened:?}"
        );
        let flags = tree.recorded_opens();
        assert!(!flags.is_empty(), "{strategy:?}: no open was recorded");
        assert!(flags.iter().all(|&flags| never_blocks(flags)), "{flags:?}");

        let live = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&fifo)
            .expect("open the FIFO without blocking");
        drop(live);
        let reported = queued_opens(&queue);
        assert!(
            matches!(reported.as_slice(), [event] if event.contains(ReadFlags::OPEN)),
            "the probe did not report the one open of the FIFO: {reported:?}"
        );
    }

    /// `root/dir/secret.rs` and `outside/dir/secret.rs`, both regular files,
    /// so a read that reaches the outside one returns its text.
    fn secret_fixture() -> (Scratch, PathBuf, PathBuf) {
        let (scratch, root, outside_dir) = fixture();
        write(&root.join("dir/secret.rs"), "inside");
        write(&outside_dir.join("secret.rs"), "OUTSIDE-SECRET");
        (scratch, root, outside_dir)
    }

    /// A hook that undoes [`park_and_link`]: the symlink goes and the parked
    /// directory returns, so every path names in-root files again.
    fn undo_park(root: &Path) -> Hook {
        let dir = root.join("dir");
        let parked = root.join("dir.parked");
        Box::new(move || {
            fs::remove_file(&dir).expect("remove the symlink");
            fs::rename(&parked, &dir).expect("restore the directory");
        })
    }

    /// The swap is made after the containment check and undone after the
    /// open, before the handle is verified: a check that judges the path
    /// again, and not the handle, would pass and return outside contents.
    #[test]
    fn a_swap_before_the_open_undone_before_the_check_is_refused() {
        type Call = fn(&DiskTree) -> Result<String, TreeError>;
        let calls: [(&str, Call); 4] = [
            ("read", |tree| tree.read(&rel("dir/secret.rs"), 1024)),
            ("read_unseen", |tree| {
                match tree.read_unseen(&rel("dir/secret.rs"), 1024, &|_| false)? {
                    Opened::Read { text, .. } => Ok(text),
                    Opened::Seen(id) => Ok(format!("{id:?}")),
                }
            }),
            ("list", |tree| {
                tree.list(&rel("dir/sub"))
                    .map(|entries| format!("{entries:?}"))
            }),
            ("entry", |tree| {
                tree.entry(&rel("dir/sub/only_outside"))
                    .map(|kind| format!("{kind:?}"))
            }),
        ];
        for &strategy in STRATEGIES {
            for (name, call) in calls {
                let (_scratch, root, outside_dir) = secret_fixture();
                let mut tree = tree_with(&root, strategy);
                tree.before_open = Some(park_and_link(&root, &outside_dir));
                tree.after_open = Some(undo_park(&root));

                let result = call(&tree);

                assert!(
                    matches!(result, Err(TreeError::OutsideRoot { .. })),
                    "{strategy:?} {name}: {result:?}"
                );
            }
        }
    }

    /// The swap is flipped at every point a test can reach: made before the
    /// open, undone after it, and made again after the handle is verified.
    #[test]
    fn a_swap_flipped_at_every_hook_point_is_refused() {
        for &strategy in STRATEGIES {
            let (_scratch, root, outside_dir) = secret_fixture();
            let dir = root.join("dir");
            let parked = root.join("dir.parked");
            let flip = std::sync::Arc::new(move || {
                if fs::symlink_metadata(&dir).is_ok_and(|status| status.file_type().is_symlink()) {
                    fs::remove_file(&dir).expect("remove the symlink");
                    fs::rename(&parked, &dir).expect("restore the directory");
                } else {
                    fs::rename(&dir, &parked).expect("park the directory");
                    symlink(&outside_dir, &dir);
                }
            });
            let mut tree = tree_with(&root, strategy);
            let hook = |flip: &std::sync::Arc<dyn Fn() + Send + Sync>| -> Hook {
                let flip = std::sync::Arc::clone(flip);
                Box::new(move || flip())
            };
            let flip: std::sync::Arc<dyn Fn() + Send + Sync> = flip;
            tree.before_open = Some(hook(&flip));
            tree.after_open = Some(hook(&flip));
            tree.after_verify = Some(hook(&flip));

            let result = tree.read(&rel("dir/secret.rs"), 1024);

            assert_outside(&result, strategy);
        }
    }

    #[test]
    fn the_same_calls_with_the_hooks_and_no_swap_succeed() {
        for &strategy in STRATEGIES {
            let (_scratch, root, _outside_dir) = secret_fixture();
            let mut tree = tree_with(&root, strategy);
            tree.before_open = Some(Box::new(|| {}));
            tree.after_open = Some(Box::new(|| {}));
            tree.after_verify = Some(Box::new(|| {}));

            assert_eq!(
                tree.read(&rel("dir/secret.rs"), 1024).expect("read"),
                "inside",
                "{strategy:?}"
            );
            assert_eq!(
                tree.list(&rel("dir/sub")).expect("list").len(),
                1,
                "{strategy:?}"
            );
            assert_eq!(
                tree.entry(&rel("dir/sub/inside.rs")).expect("entry"),
                Some(EntryKind::File),
                "{strategy:?}"
            );
        }
    }
}

/// The containment fixtures of the disk backend, run under every strategy.
#[cfg(unix)]
mod strategy {
    use super::*;

    /// `root/real.rs` and `outside/sentinel.rs`, a FIFO that must never be
    /// read, with the trees of every strategy over `root`.
    fn fixture() -> (Scratch, PathBuf) {
        let scratch = Scratch::new();
        let root = scratch.dir("root");
        scratch.dir("outside");
        write(&root.join("real.rs"), "real");
        make_fifo(&scratch.dir.join("outside/sentinel.rs"));
        (scratch, root)
    }

    fn read(tree: &DiskTree, path: &str) -> Result<String, TreeError> {
        tree.read(&rel(path), 1024)
    }

    #[test]
    fn every_strategy_refuses_an_outside_symlink_and_reads_in_root_links() {
        for &strategy in STRATEGIES {
            let (scratch, root) = fixture();
            symlink(
                scratch.dir.join("outside/sentinel.rs"),
                &root.join("out.rs"),
            );
            symlink(root.join("real.rs"), &root.join("alias.rs"));
            symlink("real.rs", &root.join("relative.rs"));
            symlink("out.rs", &root.join("chain.rs"));

            for refused in ["out.rs", "chain.rs"] {
                let result = read(&tree_with(&root, strategy), refused);
                assert!(
                    matches!(result, Err(TreeError::OutsideRoot { .. })),
                    "{strategy:?} {refused}: {result:?}"
                );
            }
            let tree = tree_with(&root, strategy);
            for accepted in ["real.rs", "alias.rs", "relative.rs"] {
                assert_eq!(
                    tree.read(&rel(accepted), 1024).expect(accepted),
                    "real",
                    "{strategy:?}"
                );
            }
        }
    }

    #[test]
    fn every_strategy_ends_a_symlink_loop_with_an_error_and_lists_it_without_entering() {
        for &strategy in STRATEGIES {
            let (scratch, root) = fixture();
            scratch.dir("root/real_dir");
            write(&root.join("real_dir/a.rs"), "a");
            symlink("loop_b", &root.join("loop_a"));
            symlink("loop_a", &root.join("loop_b"));
            let tree = tree_with(&root, strategy);

            for call in [
                tree.read(&rel("loop_a"), 1024).map(drop),
                tree.read(&rel("loop_a/x.rs"), 1024).map(drop),
                tree.list(&rel("loop_a/x")).map(drop),
            ] {
                assert!(
                    matches!(call, Err(TreeError::SymlinkLoop { .. })),
                    "{strategy:?}: {call:?}"
                );
            }
            let names: Vec<EntryKind> = tree
                .list(&RelPath::root())
                .expect("list")
                .into_iter()
                .filter(|entry| entry.name.starts_with("loop"))
                .map(|entry| entry.kind)
                .collect();
            assert_eq!(
                names,
                [EntryKind::Symlink, EntryKind::Symlink],
                "{strategy:?}"
            );
        }
    }

    #[test]
    fn every_strategy_follows_an_absolute_in_root_link_spelled_through_a_linked_parent() {
        for &strategy in STRATEGIES {
            let (scratch, root) = fixture();
            // `alias` is another spelling of the scratch directory.
            symlink(&scratch.dir, &scratch.dir.join("alias"));
            let spelled = scratch.dir.join("alias/root/real.rs");
            symlink(&spelled, &root.join("spelled.rs"));
            symlink(scratch.dir.join("alias/root"), &root.join("spelled_root"));
            let tree = tree_with(&root, strategy);

            assert_eq!(
                tree.read(&rel("spelled.rs"), 1024).expect("spelled"),
                "real",
                "{strategy:?}"
            );
            assert_eq!(
                tree.read(&rel("spelled_root/real.rs"), 1024)
                    .expect("through the root"),
                "real",
                "{strategy:?}"
            );
            assert_eq!(
                tree.identity(&rel("spelled.rs")).expect("identity"),
                tree.identity(&rel("real.rs")).expect("identity"),
                "{strategy:?}"
            );
            let climbed = tree.read(&rel("spelled_root/../outside/sentinel.rs"), 1024);
            assert!(
                matches!(climbed, Err(TreeError::OutsideRoot { .. })),
                "{strategy:?}: {climbed:?}"
            );
        }
    }

    /// The answer of each strategy to links into the outside: one that ends at
    /// an outside path that exists or does not, and one that passes an outside
    /// directory that exists or does not and climbs back into the root. The
    /// other-Unix walk answers `OutsideRoot` to all of them, whether the outside
    /// path exists or not. Canonicalisation, which the Linux and Windows
    /// strategies check first, follows the link where it leads: `OutsideRoot`
    /// or `None` for the first pair, the in-root file or `NotFound` for the
    /// second.
    #[test]
    fn links_into_the_outside_are_answered_alike_only_by_the_walk() {
        for &strategy in STRATEGIES {
            let (scratch, root) = fixture();
            scratch.dir("outside/there/deeper");
            scratch.dir("root/sub");
            symlink(scratch.dir.join("outside/there"), &root.join("there"));
            symlink(scratch.dir.join("outside/missing"), &root.join("missing"));
            // The missing component is not the last one the walk takes.
            symlink(
                scratch.dir.join("outside/there/deeper"),
                &root.join("there_deep"),
            );
            symlink(
                scratch.dir.join("outside/missing/deeper"),
                &root.join("missing_deep"),
            );
            // Out of the root and back into it.
            symlink(
                scratch.dir.join("outside/there/../../root/real.rs"),
                &root.join("via_there.rs"),
            );
            symlink(
                scratch.dir.join("outside/missing/../../root/real.rs"),
                &root.join("via_missing.rs"),
            );
            symlink(
                "../outside/there/../../root/real.rs",
                &root.join("rel_there.rs"),
            );
            symlink(
                "../outside/missing/../../root/real.rs",
                &root.join("rel_missing.rs"),
            );
            // A `..` that stays inside the root is followed.
            symlink("..", &root.join("sub/back"));
            let tree = tree_with(&root, strategy);
            let walk = strategy == Strategy::OtherUnix;

            for exists in ["there/x.rs", "there_deep/x.rs"] {
                let result = tree.entry(&rel(exists));
                assert!(
                    matches!(result, Err(TreeError::OutsideRoot { .. })),
                    "{strategy:?} {exists}: {result:?}"
                );
            }
            for missing in ["missing/x.rs", "missing_deep/x.rs"] {
                let result = tree.entry(&rel(missing));
                if walk {
                    assert!(
                        matches!(result, Err(TreeError::OutsideRoot { .. })),
                        "{strategy:?} {missing}: {result:?}"
                    );
                } else {
                    assert_eq!(result.expect(missing), None, "{strategy:?} {missing}");
                }
            }
            for (there, missing) in [
                ("via_there.rs", "via_missing.rs"),
                ("rel_there.rs", "rel_missing.rs"),
            ] {
                let there = tree.read(&rel(there), 1024);
                let missing = tree.read(&rel(missing), 1024);
                if walk {
                    assert!(
                        matches!(there, Err(TreeError::OutsideRoot { .. }))
                            && matches!(missing, Err(TreeError::OutsideRoot { .. })),
                        "{strategy:?}: {there:?} {missing:?}"
                    );
                } else {
                    assert!(
                        matches!(&there, Ok(text) if text == "real")
                            && matches!(missing, Err(TreeError::NotFound { .. })),
                        "{strategy:?}: {there:?} {missing:?}"
                    );
                }
            }
            assert_eq!(
                tree.read(&rel("sub/back/real.rs"), 1024).expect("back"),
                "real",
                "{strategy:?}"
            );
        }
    }

    /// Links to an in-root file whose resolution leaves the root and climbs
    /// back through `..`: the root's own name after a `..`, an absolute target
    /// that climbs out of the root and back, and an outside link to the root
    /// whose own target climbs. Linux and Windows read the file; the other-Unix
    /// walk refuses each with `OutsideRoot`, beside an absolute target through a
    /// linked parent, which every strategy reads.
    #[test]
    fn a_link_that_climbs_back_into_the_root_is_read_only_by_canonicalisation() {
        for &strategy in STRATEGIES {
            let (scratch, root) = fixture();
            scratch.dir("outside/there");
            symlink("../root/real.rs", &root.join("dotdot.rs"));
            symlink(root.join("../root/real.rs"), &root.join("absolute.rs"));
            symlink("there/../../root", &scratch.dir.join("outside/to_root"));
            symlink(
                scratch.dir.join("outside/to_root/real.rs"),
                &root.join("through_link.rs"),
            );
            symlink(&scratch.dir, &scratch.dir.join("alias"));
            symlink(
                scratch.dir.join("alias/root/real.rs"),
                &root.join("spelled.rs"),
            );
            let tree = tree_with(&root, strategy);

            for climbs in ["dotdot.rs", "absolute.rs", "through_link.rs"] {
                let result = tree.read(&rel(climbs), 1024);
                if strategy == Strategy::OtherUnix {
                    assert!(
                        matches!(result, Err(TreeError::OutsideRoot { .. })),
                        "{strategy:?} {climbs}: {result:?}"
                    );
                } else {
                    assert_eq!(result.expect(climbs), "real", "{strategy:?} {climbs}");
                }
            }
            assert_eq!(
                tree.read(&rel("spelled.rs"), 1024).expect("spelled"),
                "real",
                "{strategy:?}"
            );
        }
    }

    /// The open of a step meets a symlink, and the entry is a directory by
    /// the time its status is read: the walk takes the step again and does
    /// not answer `NotFound`.
    #[test]
    fn a_symlink_that_becomes_a_directory_during_a_step_is_stepped_again() {
        for swap in [true, false] {
            let (scratch, root) = fixture();
            scratch.dir("root/real_dir");
            write(&root.join("real_dir/a.rs"), "a");
            symlink("real_dir", &root.join("flap"));
            let mut tree = tree_with(&root, Strategy::OtherUnix);
            let flap = root.join("flap");
            let real = root.join("real_dir");
            tree.mid_step = Some(Box::new(move || {
                let link =
                    fs::symlink_metadata(&flap).is_ok_and(|status| status.file_type().is_symlink());
                if swap && link {
                    fs::remove_file(&flap).expect("remove the symlink");
                    fs::rename(&real, &flap).expect("put the directory in its place");
                }
            }));

            let text = tree.read(&rel("flap/a.rs"), 1024);

            assert_eq!(text.expect("read"), "a", "swap: {swap}");
        }
    }

    /// Run in a child process whose descriptor limit is 64: a path 300
    /// directories deep is read, listed and looked up under every strategy,
    /// beside a `..` near its foot, so no walk holds a descriptor per step.
    #[test]
    fn a_path_three_hundred_deep_is_walked_under_a_limit_of_sixty_four_descriptors() {
        const CHILD: &str = "RAF_SOURCE_DESCRIPTOR_LIMIT_CHILD";
        let name = "tree::disk::tests::strategy::\
                    a_path_three_hundred_deep_is_walked_under_a_limit_of_sixty_four_descriptors";
        if std::env::var_os(CHILD).is_some() {
            deep_walk();
            return;
        }
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg("ulimit -n 64 && exec \"$0\" \"$@\"")
            .arg(std::env::current_exe().expect("the test binary"))
            .args(["--exact", name, "--test-threads=1"])
            .env(CHILD, "1")
            .output()
            .expect("run the child");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success() && stdout.contains("test result: ok. 1 passed"),
            "{}\n{stdout}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn deep_walk() {
        // The limit is in force: opening 64 more descriptors fails.
        let scratch = Scratch::new();
        write(&scratch.dir.join("probe"), "");
        let held: Vec<_> = (0..64)
            .map_while(|_| fs::File::open(scratch.dir.join("probe")).ok())
            .collect();
        assert!(held.len() < 64, "the descriptor limit is not in force");
        drop(held);
        let root = scratch.dir("root");
        let deep = vec!["d"; 300].join("/");
        fs::create_dir_all(root.join(&deep)).expect("create the deep path");
        write(&root.join(&deep).join("f.rs"), "deep");
        for &strategy in STRATEGIES {
            let tree = tree_with(&root, strategy);
            assert_eq!(
                tree.read(&rel(&format!("{deep}/f.rs")), 1024)
                    .expect("deep read"),
                "deep",
                "{strategy:?}"
            );
            assert_eq!(
                tree.read(&rel(&format!("{deep}/../d/f.rs")), 1024)
                    .expect("deep read through .."),
                "deep",
                "{strategy:?}"
            );
            assert_eq!(
                tree.list(&rel(&deep)).expect("deep list").len(),
                1,
                "{strategy:?}"
            );
            assert_eq!(
                tree.entry(&rel(&format!("{deep}/f.rs")))
                    .expect("deep entry"),
                Some(EntryKind::File),
                "{strategy:?}"
            );
        }
    }

    #[test]
    fn every_strategy_refuses_a_parent_segment_after_an_outside_directory_symlink() {
        for &strategy in STRATEGIES {
            let (scratch, root) = fixture();
            scratch.dir("outside/inner");
            make_fifo(&scratch.dir.join("outside/x.rs"));
            symlink(scratch.dir.join("outside/inner"), &root.join("vendored"));
            write(&root.join("x.rs"), "the lexical clamp would read this");

            let result = read(&tree_with(&root, strategy), "vendored/../x.rs");

            assert!(
                matches!(result, Err(TreeError::OutsideRoot { .. })),
                "{strategy:?}: {result:?}"
            );
        }
    }

    #[test]
    fn every_strategy_refuses_a_fifo_and_a_directory_as_a_file_without_blocking() {
        for &strategy in STRATEGIES {
            let (scratch, root) = fixture();
            make_fifo(&root.join("pipe.rs"));
            scratch.dir("root/sub");
            let tree = tree_with(&root, strategy);

            let (pipe, sub, kind, real) = (
                tree.read(&rel("pipe.rs"), 1024),
                tree.identity(&rel("sub")),
                tree.entry(&rel("pipe.rs")),
                tree.read(&rel("real.rs"), 1024),
            );

            assert!(
                matches!(pipe, Err(TreeError::NotRegular { .. })),
                "{strategy:?}: {pipe:?}"
            );
            assert!(
                matches!(sub, Err(TreeError::NotRegular { .. })),
                "{strategy:?}: {sub:?}"
            );
            assert_eq!(kind.expect("entry"), Some(EntryKind::Other), "{strategy:?}");
            assert_eq!(real.expect("regular"), "real", "{strategy:?}");
        }
    }

    /// Every open that resolves a name is an `O_PATH` reference or an
    /// `O_NONBLOCK` open, so none waits on a FIFO. The calls reach a FIFO
    /// inside the root, a FIFO behind an outside link, and a directory; those
    /// that must reach an open record at least one (the Linux strategy refuses
    /// the outside link before it opens anything).
    #[test]
    fn every_open_that_resolves_a_name_is_a_reference_or_does_not_block() {
        for &strategy in STRATEGIES {
            let (scratch, root) = fixture();
            make_fifo(&root.join("pipe.rs"));
            scratch.dir("root/sub");
            write(&root.join("sub/inner.rs"), "inner");
            symlink(
                scratch.dir.join("outside/sentinel.rs"),
                &root.join("out.rs"),
            );
            let tree = tree_with(&root, strategy);

            let mut seen = 0;
            let mut recorded = |call: &str, opens: bool| {
                let flags = tree.recorded_opens();
                assert!(
                    !opens || flags.len() > seen,
                    "{strategy:?} {call}: no open recorded"
                );
                seen = flags.len();
                assert!(
                    flags.iter().all(|&flags| never_blocks(flags)),
                    "{strategy:?} {call}: {flags:?}"
                );
            };
            let _ = tree.read(&rel("pipe.rs"), 1024);
            recorded("read of a FIFO", true);
            let _ = tree.read(&rel("out.rs"), 1024);
            recorded("read through an outside link to a FIFO", false);
            let _ = tree.read_unseen(&rel("pipe.rs"), 1024, &|_| false);
            recorded("read_unseen of a FIFO", true);
            let _ = tree.identity(&rel("pipe.rs"));
            recorded("identity of a FIFO", true);
            let _ = tree.entry(&rel("pipe.rs"));
            recorded("entry of a FIFO", false);
            let _ = tree.entry(&rel("sub/inner.rs"));
            recorded("entry of a file below a directory", true);
            let _ = tree.list(&rel("pipe.rs"));
            recorded("list of a FIFO", false);
            assert_eq!(
                tree.read(&rel("sub/inner.rs"), 1024).expect("read"),
                "inner"
            );
            recorded("read of a regular file", true);
        }
    }

    #[test]
    fn every_strategy_lists_a_directory_and_refuses_a_symlink_a_file_and_an_outside_link() {
        for &strategy in STRATEGIES {
            let (scratch, root) = fixture();
            scratch.dir("root/sub");
            write(&root.join("sub/b.rs"), "b");
            write(&root.join("sub/a.rs"), "a");
            symlink(root.join("sub"), &root.join("selfie"));
            symlink(scratch.dir.join("outside"), &root.join("away"));
            let tree = tree_with(&root, strategy);

            let names: Vec<String> = tree
                .list(&rel("sub"))
                .expect("list")
                .into_iter()
                .map(|entry| entry.name)
                .collect();
            assert_eq!(names, ["a.rs", "b.rs"], "{strategy:?}");
            for refused in ["selfie", "real.rs"] {
                let result = tree.list(&rel(refused));
                assert!(
                    matches!(result, Err(TreeError::NotRegular { .. })),
                    "{strategy:?} {refused}: {result:?}"
                );
            }
            let result = tree.list(&rel("away"));
            assert!(
                matches!(result, Err(TreeError::OutsideRoot { .. })),
                "{strategy:?}: {result:?}"
            );
        }
    }

    #[test]
    fn every_strategy_reports_entry_kinds_and_refuses_an_outside_directory() {
        for &strategy in STRATEGIES {
            let (scratch, root) = fixture();
            scratch.dir("root/sub");
            make_fifo(&root.join("pipe"));
            symlink(scratch.dir.join("outside/sentinel.rs"), &root.join("link"));
            symlink(scratch.dir.join("outside"), &root.join("away"));
            symlink(&root, &root.join("sub/up"));
            let tree = tree_with(&root, strategy);

            for (name, expected) in [
                ("real.rs", Some(EntryKind::File)),
                ("sub", Some(EntryKind::Dir)),
                ("pipe", Some(EntryKind::Other)),
                ("link", Some(EntryKind::Symlink)),
                ("absent", None),
                ("absent/deeper.rs", None),
                ("real.rs/deeper.rs", None),
                ("", Some(EntryKind::Dir)),
                ("sub/..", Some(EntryKind::Dir)),
            ] {
                assert_eq!(
                    tree.entry(&rel(name)).expect(name),
                    expected,
                    "{strategy:?} {name}"
                );
            }
            // A directory that leads outside, whether through a link or a
            // `..` that climbs out of a link to the root, is refused.
            for refused in ["away/sentinel.rs", "sub/up/.."] {
                let result = tree.entry(&rel(refused));
                assert!(
                    matches!(result, Err(TreeError::OutsideRoot { .. })),
                    "{strategy:?} {refused}: {result:?}"
                );
            }
        }
    }

    #[test]
    fn every_strategy_gives_one_identity_for_a_file_its_links_and_a_path_through_a_directory_link()
    {
        for &strategy in STRATEGIES {
            let (scratch, root) = fixture();
            symlink(root.join("real.rs"), &root.join("alias.rs"));
            fs::hard_link(root.join("real.rs"), root.join("hard.rs")).expect("hard link");
            scratch.dir("root/sub");
            write(&root.join("sub/other.rs"), "other");
            symlink(root.join("sub"), &root.join("through"));
            let tree = tree_with(&root, strategy);

            let real = tree.identity(&rel("real.rs")).expect("a file");
            for same in ["alias.rs", "hard.rs", "sub/../real.rs"] {
                assert_eq!(tree.identity(&rel(same)).expect(same), real, "{strategy:?}");
            }
            let other = tree.identity(&rel("sub/other.rs")).expect("other");
            assert_ne!(other, real, "{strategy:?}");
            assert_eq!(
                tree.identity(&rel("through/other.rs")).expect("through"),
                other,
                "{strategy:?}"
            );
            match tree.read_unseen(&rel("alias.rs"), 1024, &|_| false) {
                Ok(Opened::Read { id, text }) => {
                    assert_eq!((id, text.as_str()), (real, "real"), "{strategy:?}");
                }
                other => panic!("{strategy:?}: {other:?}"),
            }
        }
    }
}
