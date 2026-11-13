// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Canonical source loading through bounded native parallel workers.

#![cfg(target_os = "linux")]

use std::fmt;
use std::io;
use std::num::NonZeroUsize;
use std::panic::{self, AssertUnwindSafe};

use raf_source::{
    Entry, EntryKind, FileId, LoadErrorKind, LoadOptions, MemoryTree, Opened, ParallelOptions,
    ParallelTree, RelPath, Tree, TreeError, Workspace,
};

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const GRANT: usize = 16 * 1024 * 1024;

fn options(workers: usize) -> ParallelOptions {
    ParallelOptions::new(NonZeroUsize::new(workers).expect("nonzero workers"), GRANT)
        .expect("scheduler fits")
}

#[test]
fn nested_inline_alternative_alias_and_target_loads_keep_the_serial_index_and_usage() {
    let tree = MemoryTree::from([
        ("Cargo.toml", MANIFEST),
        (
            "src/lib.rs",
            "mod a; mod b; mod inline { mod nested; } #[path=\"a.rs\"] mod alias; #[cfg_attr(test,path=\"other.rs\")] mod selected;",
        ),
        ("src/a.rs", "mod child; pub fn a() {}"),
        ("src/a/child.rs", "pub fn child() {}"),
        ("src/child.rs", "pub fn child() {}"),
        ("src/b.rs", "pub fn b() {}"),
        ("src/inline/nested.rs", "pub fn nested() {}"),
        ("src/selected.rs", "pub fn selected() {}"),
        ("src/other.rs", "pub fn other() {}"),
        ("src/main.rs", "fn main() {}"),
        ("tests/case.rs", "#[test] fn case() {}"),
    ]);
    let load = LoadOptions::default();
    let serial = Workspace::load_with(&tree, &["."], &load).expect("serial loads");
    for workers in [1, 2, 4] {
        let parallel =
            Workspace::par_load(&tree, &["."], &load, &options(workers)).expect("parallel loads");
        assert_eq!(serial, parallel);
        assert_eq!(serial.usage(), parallel.usage());
        let usage = parallel
            .parallel_usage()
            .expect("parallel counts finalized");
        assert!(usage.peak_live_jobs <= workers);
        assert!(usage.peak_scheduler_bytes <= GRANT);
        assert!(usage.peak_worker_stack_bytes <= workers * raf_source::PARSE_STACK_BYTES);
    }
    // MemoryTree accepts long components without creating any host path. A
    // plain module file retains this concrete stem as its child-directory context.
    let stem = "m".repeat(64 * 1024);
    let tree: MemoryTree = [
        ("Cargo.toml".to_owned(), MANIFEST.to_owned()),
        ("src/lib.rs".to_owned(), format!("mod {stem};")),
        (format!("src/{stem}.rs"), "mod child;".to_owned()),
        (
            format!("src/{stem}/child.rs"),
            "pub fn child() {}".to_owned(),
        ),
    ]
    .into_iter()
    .collect();
    let serial = Workspace::load_with(&tree, &["."], &load).expect("valid memory paths");
    let workers = NonZeroUsize::new(1).expect("nonzero");
    let below = ParallelOptions::new(workers, 32 * 1024)
        .expect("fixed scheduler fits below known stem size");
    let refused =
        Workspace::par_load(&tree, &["."], &load, &below).expect_err("retained stem exceeds grant");
    assert!(matches!(
        refused.kind,
        LoadErrorKind::Storage(purrdf_lex::allocation::StorageError::AdmissionFailed)
    ));
    let above = ParallelOptions::new(workers, 256 * 1024).expect("larger scheduler grant");
    let valid = Workspace::par_load(&tree, &["."], &load, &above).expect("known context fits");
    assert_eq!(serial, valid);
    assert_eq!(serial.usage(), valid.usage());
}

#[derive(Debug)]
struct OriginalCause;

impl fmt::Display for OriginalCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("original backend cause")
    }
}

impl std::error::Error for OriginalCause {}

struct FailingTree {
    tree: MemoryTree,
    panic: bool,
    alias: bool,
}

impl Tree for FailingTree {
    fn read(&self, path: &RelPath, maximum: u64) -> Result<String, TreeError> {
        self.tree.read(path, maximum)
    }

    fn read_unseen(
        &self,
        path: &RelPath,
        maximum: u64,
        seen: &dyn Fn(&FileId) -> bool,
    ) -> Result<Opened, TreeError> {
        if self.alias && matches!(path.as_str(), "src/a.rs" | "src/b.rs") {
            let id = FileId::from_key("shared");
            if seen(&id) {
                return Ok(Opened::Seen(id));
            }
            if path.as_str() == "src/a.rs" {
                return self
                    .tree
                    .read(path, maximum)
                    .map(|text| Opened::Read { id, text });
            }
        }
        if path.as_str() == "src/b.rs" {
            if self.panic {
                panic::panic_any(OriginalCause);
            }
            return Err(TreeError::Io {
                path: path.to_string(),
                source: io::Error::other(OriginalCause),
            });
        }
        self.tree.read_unseen(path, maximum, seen)
    }

    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> {
        self.tree.list(path)
    }

    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> {
        self.tree.entry(path)
    }
}

impl ParallelTree for FailingTree {}

fn failing_tree(early: &str, panic: bool) -> FailingTree {
    FailingTree {
        tree: MemoryTree::from([
            ("Cargo.toml", MANIFEST),
            ("src/lib.rs", "mod a; mod b;"),
            ("src/a.rs", early),
            ("src/a/child.rs", "fn broken("),
            ("src/b.rs", "pub fn b() {}"),
        ]),
        panic,
        alias: false,
    }
}

#[test]
fn an_earlier_canonical_descendant_error_wins_over_a_speculative_backend_panic() {
    let tree = failing_tree("mod child;", true);
    let load = LoadOptions::default();
    let serial = Workspace::load_with(&tree, &["."], &load).expect_err("invalid descendant");
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        Workspace::par_load(&tree, &["."], &load, &options(2))
    }));
    let parallel = result
        .expect("unreached panic discarded")
        .expect_err("invalid descendant");
    assert_eq!(serial.to_string(), parallel.to_string());
    assert_eq!(parallel.file.as_str(), "src/a/child.rs");
}

#[test]
fn reached_backend_failures_keep_original_typed_causes_and_panic_payloads() {
    let load = LoadOptions::default();
    let tree = failing_tree("pub fn a() {}", false);
    let error =
        Workspace::par_load(&tree, &["."], &load, &options(2)).expect_err("backend refuses");
    let LoadErrorKind::Tree(source) = error.kind else {
        panic!("original Tree cause required")
    };
    let TreeError::Io { source, .. } = *source else {
        panic!("original I/O cause required")
    };
    assert!(
        source
            .get_ref()
            .expect("custom source")
            .is::<OriginalCause>()
    );

    let tree = failing_tree("pub fn a() {}", true);
    let payload = panic::catch_unwind(AssertUnwindSafe(|| {
        Workspace::par_load(&tree, &["."], &load, &options(2))
    }))
    .expect_err("reached backend panic resumes");
    assert!(payload.is::<OriginalCause>());
}

#[test]
fn canonical_fact_allowances_refuse_the_exact_serial_position_beside_the_valid_limit() {
    let text = format!("/// {}\npub fn payload() {{}}\n", "d".repeat(4096));
    let tree = MemoryTree::from([
        ("Cargo.toml", MANIFEST),
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a.rs", text.as_str()),
        ("src/b.rs", text.as_str()),
    ]);
    let serial = Workspace::load(&tree, &["."]).expect("unlimited facts");
    let mut load = LoadOptions::default();
    load.max_fact_bytes = serial.usage().fact_bytes;
    let valid = Workspace::par_load(&tree, &["."], &load, &options(2)).expect("exact limit");
    assert_eq!(serial, valid);
    assert_eq!(serial.usage(), valid.usage());
    load.max_fact_bytes -= 1;
    let serial = Workspace::load_with(&tree, &["."], &load).expect_err("one byte below");
    let parallel =
        Workspace::par_load(&tree, &["."], &load, &options(2)).expect_err("one byte below");
    assert_eq!(serial.to_string(), parallel.to_string());
    assert_eq!(parallel.file.as_str(), "src/b.rs");
    assert!(parallel.line.is_some());
    assert!(matches!(
        parallel.kind,
        LoadErrorKind::Budget {
            budget: raf_source::Budget::FactBytes,
            ..
        }
    ));
}

#[test]
fn a_committed_physical_alias_overrides_provisional_unseen_read_errors_and_panics() {
    for panic in [false, true] {
        let tree = FailingTree {
            tree: MemoryTree::from([
                ("Cargo.toml", MANIFEST),
                ("src/lib.rs", "mod a; mod b;"),
                ("src/a.rs", "pub fn shared() {}"),
                ("src/b.rs", "pub fn shared() {}"),
            ]),
            panic,
            alias: true,
        };
        let serial = Workspace::load(&tree, &["."]).expect("alias was already read");
        let parallel = panic::catch_unwind(AssertUnwindSafe(|| {
            Workspace::par_load(&tree, &["."], &LoadOptions::default(), &options(2))
        }))
        .expect("provisional alias panic is not canonical")
        .expect("committed alias needs no unreadable second observation");
        assert_eq!(serial, parallel);
        assert_eq!(serial.usage(), parallel.usage());
    }
}
