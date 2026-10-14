// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The walks of a load cost the caller's stack nothing per level: discovery,
//! the expansion of `workspace.members` patterns and the module walk each keep
//! their own stack. Each fixture runs on a small thread, so a walk that
//! recursed on the caller's stack would abort the test binary instead of
//! ending in the typed outcome asserted here.

mod support;

use std::thread;

use raf_source::{Budget, LoadError, LoadErrorKind, MemoryTree, Tree, Workspace};

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// Run `work` on a thread with a stack of `bytes`.
fn on_stack<T: Send>(bytes: usize, work: impl FnOnce() -> T + Send) -> T {
    thread::scope(|scope| {
        thread::Builder::new()
            .name("small-stack".to_owned())
            .stack_size(bytes)
            .spawn_scoped(scope, work)
            .expect("spawn the thread")
            .join()
            .expect("the work does not panic")
    })
}

fn load(tree: &(dyn Tree + Sync), roots: &[&str], stack: usize) -> Result<Workspace, LoadError> {
    on_stack(stack, || Workspace::load(tree, roots))
}

fn depth_refusal(error: &LoadError) -> bool {
    matches!(
        error.kind,
        LoadErrorKind::Budget {
            budget: Budget::DirectoryDepth,
            limit: 128
        }
    )
}

/// `a/` repeated `levels` times.
fn chain(levels: usize) -> String {
    "a/".repeat(levels)
}

#[test]
fn a_memory_chain_of_twenty_thousand_directories_is_refused_typed_on_a_two_mib_thread() {
    let deep = format!("{}x.txt", chain(20_000));
    let tree = MemoryTree::try_from_iter([(deep.as_str(), "")]).expect("tree");
    let error = load(&tree, &["."], 2 << 20).expect_err("too deep");
    assert!(depth_refusal(&error), "{error}");
    // The directory named is the first one past the limit.
    assert_eq!(error.file.as_str(), chain(129).trim_end_matches('/'));
    assert!(error.to_string().contains("directory nesting"), "{error}");
}

#[test]
fn a_directory_at_the_depth_limit_is_walked_and_one_below_it_is_refused() {
    let at = format!("{}x.txt", chain(128));
    let tree = MemoryTree::try_from_iter([(at.as_str(), "")]).expect("tree");
    let ws = load(&tree, &["."], 2 << 20).expect("at the limit");
    assert_eq!(ws.usage().max_directory_depth, 128);
    let over = format!("{}x.txt", chain(129));
    let tree = MemoryTree::try_from_iter([(over.as_str(), "")]).expect("tree");
    let error = load(&tree, &["."], 2 << 20).expect_err("one over");
    assert!(depth_refusal(&error), "{error}");
    // The depth counts below the root the walk starts from.
    let ws = load(&tree, &["a"], 2 << 20).expect("one level less below `a`");
    assert_eq!(ws.usage().max_directory_depth, 128);
}

/// A workspace whose member sits `levels` directories below it, named by a
/// literal `workspace.members` pattern, and inherits its edition. The load is
/// rooted at the member, so discovery walks no deeper than the member itself
/// and only the member expansion walks the chain.
fn deep_member(levels: usize) -> MemoryTree {
    let member = chain(levels);
    let pattern = member.trim_end_matches('/');
    let workspace = format!(
        "[workspace]\nmembers = [\"{pattern}\"]\n[workspace.package]\nedition = \"2021\"\n"
    );
    let manifest = format!("{member}Cargo.toml");
    let lib = format!("{member}src/lib.rs");
    MemoryTree::try_from_iter([
        ("Cargo.toml", workspace.as_str()),
        (
            manifest.as_str(),
            "[package]\nname = \"deep\"\nversion = \"0.1.0\"\nedition.workspace = true\n",
        ),
        (lib.as_str(), "pub fn f() {}\n"),
    ])
    .expect("tree")
}

#[test]
fn a_member_pattern_at_the_depth_limit_expands_and_one_deeper_is_refused_typed() {
    let at = deep_member(128);
    let root = chain(128);
    let ws = load(&at, &[root.trim_end_matches('/')], 512 << 10).expect("at the limit");
    assert_eq!(ws.usage().max_directory_depth, 128);
    let over = deep_member(129);
    let root = chain(129);
    let error = load(&over, &[root.trim_end_matches('/')], 512 << 10).expect_err("one over");
    assert!(depth_refusal(&error), "{error}");
    assert_eq!(error.file.as_str(), chain(129).trim_end_matches('/'));
}

/// A package whose library declares `levels` nested modules, inline or each
/// in its own file.
fn nested_modules(levels: usize, inline: bool) -> MemoryTree {
    let mut files = vec![("Cargo.toml".to_owned(), DEMO.to_owned())];
    if inline {
        let text = format!(
            "{}pub struct S;{}\n",
            "pub mod m { ".repeat(levels),
            " }".repeat(levels)
        );
        files.push(("src/lib.rs".to_owned(), text));
    } else {
        files.push(("src/lib.rs".to_owned(), "pub mod m;\n".to_owned()));
        for level in 1..=levels {
            let path = format!("src/{}m.rs", "m/".repeat(level - 1));
            let text = if level == levels {
                "pub struct S;\n".to_owned()
            } else {
                "pub mod m;\n".to_owned()
            };
            files.push((path, text));
        }
    }
    files.into_iter().collect()
}

#[test]
fn one_hundred_twenty_eight_nested_modules_load_on_a_five_hundred_twelve_kib_thread() {
    for inline in [true, false] {
        let tree = nested_modules(128, inline);
        let ws = load(&tree, &["."], 512 << 10)
            .unwrap_or_else(|error| panic!("inline {inline}: {error}"));
        let deepest = format!("demo{}::S", "::m".repeat(128));
        assert_eq!(ws.resolve(&deepest).expect("resolves").path, deepest);
        // One level more is refused typed on the same stack: by the module
        // walk, or for inline modules already by the delimiter pre-scan.
        let error =
            load(&nested_modules(129, inline), &["."], 512 << 10).expect_err("one level more");
        let expected = if inline {
            Budget::Delimiters
        } else {
            Budget::ModuleDepth
        };
        assert!(
            matches!(error.kind, LoadErrorKind::Budget { budget, .. } if budget == expected),
            "inline {inline}: {error}"
        );
    }
}

#[cfg(unix)]
#[test]
fn a_disk_chain_of_directories_is_refused_typed_on_a_two_mib_thread() {
    use raf_source::DiskTree;
    use support::Scratch;

    let scratch = Scratch::new();
    // 2100 levels. No path to the deep end fits in one system call, so a chain
    // of 1100 is moved to the bottom of a chain of 1000, each path short enough.
    scratch.file(&format!("lower/{}x.txt", chain(1100)), "");
    scratch.dir(&format!("root/{}", chain(999)));
    std::fs::rename(
        scratch.path("lower/a"),
        scratch.path(&format!("root/{}a", chain(999))),
    )
    .expect("move the lower chain");
    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    let error = load(&tree, &["."], 2 << 20).expect_err("too deep");
    assert!(depth_refusal(&error), "{error}");
    assert_eq!(error.file.as_str(), chain(129).trim_end_matches('/'));
    // The chain is too deep for a removal that holds one handle per level.
    let removed = std::process::Command::new("rm")
        .arg("-rf")
        .arg(scratch.path("root"))
        .status()
        .expect("run rm");
    assert!(removed.success(), "remove the chain");
    // The same chain at the limit is walked.
    let at = Scratch::new();
    at.file(&format!("root/{}x.txt", chain(128)), "");
    let tree = DiskTree::new(at.path("root")).expect("tree");
    let ws = load(&tree, &["."], 2 << 20).expect("at the limit");
    assert_eq!(ws.usage().max_directory_depth, 128);
}

/// A library that imports `use_path`, beside the module the path names.
fn importing(use_path: &str) -> MemoryTree {
    let lib = format!("pub mod a {{ pub fn b() {{}} }}\npub use {use_path};\n");
    [("Cargo.toml", DEMO.to_owned()), ("src/lib.rs", lib)]
        .into_iter()
        .collect()
}

/// A path of `separators` `::`s.
fn long_path(separators: usize) -> String {
    format!("{}b", "a::".repeat(separators))
}

/// A path of `levels` nested groups, each after a `::`.
fn nested_groups(levels: usize) -> String {
    format!("{}b{}", "a::{".repeat(levels), "}".repeat(levels))
}

#[test]
fn a_use_tree_at_the_recursion_limit_loads_resolves_and_drops_on_a_five_hundred_twelve_kib_thread()
{
    let limit = Budget::Recursion.limit();
    // A separator counts sixteen, and a group its `{` and the level it opens.
    for (at, over) in [
        (long_path(limit / 16), long_path(limit / 16 + 1)),
        (nested_groups(limit / 18), nested_groups(limit / 18 + 1)),
    ] {
        let tree = importing(&at);
        let unresolved = on_stack(512 << 10, || {
            let workspace = Workspace::load(&tree, &["."]).expect("loads");
            let unresolved = workspace.resolve("demo::b").is_err();
            drop(workspace);
            unresolved
        });
        assert!(unresolved, "the path names nothing the index holds");
        let over = importing(&over);
        let error = on_stack(512 << 10, || Workspace::load(&over, &["."])).expect_err("refused");
        assert!(
            matches!(
                error.kind,
                LoadErrorKind::Budget {
                    budget: Budget::Recursion,
                    ..
                }
            ),
            "{error}"
        );
    }
}
