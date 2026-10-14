use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use raf_okf::{Body, DocType, Document, Index, Profile, RafDocument, Rule, is_capability_index};
use raf_source::{
    DiskTree, LoadOptions, MemoryTree, Namespace, RelPath, TargetKind, TestRole, Tree, Workspace,
};

type R = Result<(), Box<dyn Error>>;

const REPO: &str = "/home/paudley/Active/rust-agentic-foundations/.worktrees/2-shared-crates-raf-okf-raf-source";

fn main() -> R {
    let which = std::env::args().nth(1).unwrap_or_default();
    match which.as_str() {
        "repo-source" => repo_source(),
        "repo-source-dot" => repo_source_dot(),
        "repo-okf" => repo_okf(),
        "disk-escape" => disk_escape(),
        "memory" => memory(),
        "okf-edge" => okf_edge(),
        "cap8" => cap8(),
        other => Err(format!("unknown section {other}").into()),
    }
}

fn tracked(pattern: &str) -> Vec<String> {
    let out = Command::new("git")
        .args(["-C", REPO, "ls-files", pattern])
        .output()
        .expect("git");
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

fn repo_source() -> R {
    let tree = DiskTree::new(REPO)?;
    let ws = Workspace::load(&tree, &["crates"])?;
    println!("packages: {:?}", ws.packages().iter().map(|p| &p.name).collect::<Vec<_>>());
    let targets: Vec<_> = ws.targets().collect();
    println!("targets: {}", targets.len());
    for kind in [TargetKind::Lib, TargetKind::Test] {
        let n = targets.iter().filter(|t| t.kind == kind).count();
        println!("  {kind:?}: {n}");
    }
    let shipping: Vec<_> = targets.iter().filter(|t| t.shipping).map(|t| t.crate_path.clone()).collect();
    println!("shipping targets: {shipping:?}");
    println!("files: {}", ws.files().count());
    println!("items: {}", ws.items().count());
    // Re-exported root paths resolve to their defining module paths.
    for path in [
        "raf_okf::Document",
        "raf_okf::Body",
        "raf_okf::Profile",
        "raf_okf::Document::parse",
        "raf_okf::Profile::validate",
        "raf_source::Workspace::resolve",
        "raf_source::DiskTree",
        "raf_source::Tree::read",
        "raf_source::PARSE_STACK_BYTES",
        "raf_okf::test_crate::profile::an_unknown_key_is_reported_at_that_key",
        "raf_source::tree::disk::tests::swap::a_directory_swapped_for_an_outside_symlink_after_the_check_is_refused",
        "raf_source::error::tests::a_spawn_error_displays_the_file_and_carries_its_cause",
    ] {
        match ws.resolve(path) {
            Ok(r) => {
                let item = ws.item(r.sites[0]).unwrap();
                let target = ws.target(item.target).unwrap();
                println!(
                    "OK  {path}\n    -> {} ns={:?} kind={:?} file={}:{} test={:?} target={} shipping={} via_glob={} chain={:?}",
                    r.path, r.namespace, item.kind, item.file, item.name_line, item.test,
                    target.crate_path, ws.is_shipping(item), r.via_glob, r.chain
                );
            }
            Err(e) => println!("ERR {path}: {e}"),
        }
    }
    for path in ["raf_okf::Nope", "raf_source::syn::File", "raf_okf::Document::nope", "serde::Serialize"] {
        match ws.resolve(path) {
            Ok(r) => println!("UNEXPECTED OK {path} -> {}", r.path),
            Err(e) => println!("refused {path}: {e:?}"),
        }
    }
    // cfg(test) items in this repository are tagged, not dropped.
    let cfg_test = ws.items().filter(|i| i.test == TestRole::CfgTest).count();
    let test_fn = ws.items().filter(|i| i.test == TestRole::TestFn).count();
    println!("items tagged CfgTest: {cfg_test}, TestFn: {test_fn}");
    let fns = ws.test_fns()?;
    println!("test_fns: {}", fns.len());
    // #[path] usage in this repository's crates
    let path_files: Vec<_> = ws
        .files()
        .filter(|f| f.text.contains("#[path"))
        .map(|f| f.path.to_string())
        .collect();
    println!("files with #[path]: {path_files:?}");
    // Every binding of every delivered capability resolves, independently of docs_contract.
    let mut checked = 0;
    for doc in tracked("docs/capabilities/*.md") {
        let text = fs::read_to_string(Path::new(REPO).join(&doc))?;
        let document = Document::parse(&text)?;
        let Ok(RafDocument::Capability(cap)) = Profile.validate(&document, &doc) else {
            return Err(format!("{doc} does not validate").into());
        };
        for b in cap.bindings.code.iter().chain(&cap.bindings.tests).chain(&cap.bindings.examples) {
            checked += 1;
            if let Err(e) = ws.resolve(b) {
                println!("UNRESOLVED binding {doc}: {b}: {e}");
            } else if cap.bindings.tests.contains(b) {
                let r = ws.resolve(b)?;
                let item = ws.item(r.sites[0]).unwrap();
                if item.kind != raf_source::SymbolKind::Fn {
                    println!("test binding is not a fn: {b} {:?}", item.kind);
                }
            }
        }
    }
    println!("bindings resolved: {checked}");
    Ok(())
}

fn repo_source_dot() -> R {
    let tree = DiskTree::new(REPO)?;
    match Workspace::load(&tree, &["."]) {
        Ok(ws) => {
            println!("packages: {:?}", ws.packages().iter().map(|p| (&p.name, p.manifest.to_string())).collect::<Vec<_>>());
            println!("targets: {:?}", ws.targets().map(|t| t.crate_path.clone()).collect::<Vec<_>>().len());
            for p in ["rust_agentic_foundations::test_crate::docs_contract::repository_bindings_resolve_in_the_source_index", "raf_okf::Document"] {
                println!("{p}: {:?}", ws.resolve(p).map(|r| r.path));
            }
        }
        Err(e) => println!("load '.' failed: {e}"),
    }
    Ok(())
}

fn repo_okf() -> R {
    let mut docs = 0;
    for pattern in ["docs/capabilities/*.md", "docs/architecture/*.md", "work/changes/*.md"] {
        for path in tracked(pattern) {
            let text = fs::read_to_string(Path::new(REPO).join(&path))?;
            let document = Document::parse(&text).map_err(|e| format!("{path}: {e}"))?;
            if document.render() != text {
                println!("ROUND-TRIP MISMATCH {path}");
            }
            let reparsed = Document::parse(&document.render())?;
            if reparsed != document {
                println!("PARSE(RENDER) != DOC {path}");
            }
            match Profile.validate(&document, &path) {
                Ok(typed) => {
                    let kind = match typed {
                        RafDocument::Capability(c) => format!("Capability state={:?}", c.state),
                        RafDocument::Change(c) => format!("Change items={}", c.items.len()),
                        RafDocument::Architecture(_) => "Architecture".to_string(),
                    };
                    if path.contains("okf-profile") || path.contains("source-index") {
                        println!("{path}: {kind} sections={} links={} fences={}", document.body().sections().len(), document.body().links().len(), document.body().fenced_blocks().len());
                    }
                }
                Err(v) => println!("INVALID {path}: {v:?}"),
            }
            docs += 1;
        }
    }
    println!("profile documents parsed, rendered byte-identically and validated: {docs}");
    // Every other tracked Markdown goes through Body::parse.
    let mut others = 0;
    for path in tracked("*.md") {
        if path.starts_with("docs/capabilities/") || path.starts_with("docs/architecture/") || path.starts_with("work/changes/") {
            continue;
        }
        let text = fs::read_to_string(Path::new(REPO).join(&path))?;
        let body = Body::parse(&text).map_err(|e| format!("{path}: {e}"))?;
        if !body.split_links().is_empty() {
            println!("split link in {path}");
        }
        others += 1;
        if path == "README.md" {
            println!("README: headings={} links={} first link={:?}", body.headings().len(), body.links().len(), body.links().first().map(|l| (l.target().to_string(), l.line())));
        }
    }
    println!("other markdown files read by Body::parse: {others}");

    // Mutations of real documents.
    let cap_path = "docs/capabilities/okf-profile.md";
    let cap = fs::read_to_string(Path::new(REPO).join(cap_path))?;
    let change_path = tracked("work/changes/*.md")[0].clone();
    let change = fs::read_to_string(Path::new(REPO).join(&change_path))?;
    let arch_path = "docs/architecture/okf-profile.md";
    let arch = fs::read_to_string(Path::new(REPO).join(arch_path))?;
    for (path, text, anchor) in [
        (cap_path, cap.as_str(), "state: delivered\n"),
        (change_path.as_str(), change.as_str(), "contract_effect:"),
        (arch_path, arch.as_str(), "title:"),
    ] {
        let at = text.find(anchor).ok_or("anchor")?;
        for (label, inject) in [
            ("extra key", "extra: 1\n"),
            ("resource", "resource: https://example.org/x\n"),
            ("dup tag", "tags: [a, a]\n"),
            ("bad timestamp", "timestamp: 2026-13-01T00:00:00Z\n"),
            ("ok tags+timestamp", "tags: [b, a]\ntimestamp: 2026-01-01T00:00:00Z\n"),
        ] {
            let mutated = format!("{}{}{}", &text[..at], inject, &text[at..]);
            let doc = Document::parse(&mutated)?;
            let result = Profile.validate(&doc, path);
            let summary = match &result {
                Ok(_) => "valid".to_string(),
                Err(v) => v.iter().map(|v| format!("{} [{:?}]", v, v.rule())).collect::<Vec<_>>().join(" | "),
            };
            println!("{path} + {label}: {summary}");
            if label == "ok tags+timestamp" {
                let rendered = doc.render();
                let sorted = rendered.contains("tags: [a, b]");
                let stable = Document::parse(&rendered)?.render() == rendered;
                println!("   tags rendered sorted: {sorted}, re-render stable: {stable}");
            }
        }
    }
    Ok(())
}

fn okf_edge() -> R {
    // index.md navigation
    let index = "<!-- SPDX-License-Identifier: CC-BY-4.0 -->\n<!-- raf:generated:capability-index -->\n| a | b |\n<!-- raf:generated:end -->\n";
    println!("is_capability_index(docs/capabilities/index.md) = {}", is_capability_index("docs/capabilities/index.md"));
    println!("is_capability_index(docs/index.md) = {}", is_capability_index("docs/index.md"));
    match Index::parse(index) {
        Ok(i) => println!("Index::parse ok blocks={}", i.generated_blocks().len()),
        Err(e) => println!("Index::parse ERR {e}"),
    }
    println!("Document::parse(index) = {:?}", Document::parse(index).err().map(|e| e.to_string()));
    let prose = format!("{index}Some prose.\n");
    println!("Index::parse(with prose) = {:?}", Index::parse(&prose).err().map(|e| e.to_string()));
    // split link refused in a profile document and a frontmatter-less body
    let split = "# T\n\nSee [a](\nb.md) here.\n";
    let body = Body::parse(split)?;
    println!("split links: {:?}", body.split_links().iter().map(|s| s.line()).collect::<Vec<_>>());
    // PROPOSED + generated blocks + splice on a real capability
    let cap = fs::read_to_string(Path::new(REPO).join("docs/capabilities/census-ledger.md"))?;
    let doc = Document::parse(&cap)?;
    println!("census-ledger proposed={} generated={}", doc.body().proposed_blocks().len(), doc.body().generated_blocks().len());
    // build a doc with a generated block and splice it
    let text = "---\ntype: Architecture\nid: arch:x\ntitle: X\ndescription: Y.\n---\n\n# X\n\n<!-- raf:generated:list -->\nold\n<!-- raf:generated:end -->\n\n> [!PROPOSED change:foo]\n> new thing\n";
    let mut d = Document::parse(text)?;
    let g = d.body().generated_blocks()[0].content().clone();
    d.replace_spans(&[(g, "new\n")])?;
    println!("spliced render contains new: {}", d.render().contains("<!-- raf:generated:list -->\nnew\n<!-- raf:generated:end -->"));
    println!("proposed in arch -> {:?}", Profile.validate(&d, "docs/architecture/x.md").err().map(|v| v.iter().map(ToString::to_string).collect::<Vec<_>>()));
    // fences: markers inside a fence are not blocks
    let fenced = "# T\n\n```\n<!-- raf:generated:x -->\n> [!PROPOSED change:y]\n```\n";
    let b = Body::parse(fenced)?;
    println!("fenced markers -> generated={} proposed={} fences={}", b.generated_blocks().len(), b.proposed_blocks().len(), b.fenced_blocks().len());
    // unknown key fixture fails (frontmatter, each type)
    for (dt, p) in [(DocType::Capability, "docs/capabilities/okf-profile.md")] {
        let _ = (dt, p);
    }
    // Rule enum is reachable
    let _ = Rule::EmptyText;
    Ok(())
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn fresh(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("raf-audit-{name}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

fn disk_escape() -> R {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let base = fresh("escape");
    let root = base.join("root");
    fs::create_dir_all(&root)?;
    // A sentinel outside the root that nobody may read: unreadable, so an open
    // would be PermissionDenied (Io), not OutsideRoot.
    write(&base, "outside.rs", "pub fn stolen() {}\n");
    fs::set_permissions(base.join("outside.rs"), fs::Permissions::from_mode(0o000))?;
    fs::create_dir_all(base.join("outdir"))?;
    write(&root, "Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n");
    // a decoy inside the root that a lexical clamp would read
    write(&root, "outside.rs", "pub fn decoy() {}\n");
    symlink(base.join("outdir"), root.join("vendored"))?;
    write(&root, "src/other/x.rs", "pub fn inside() {}\n");
    let cases = [
        ("parent climb", "#[path = \"../../outside.rs\"]\nmod x;\n"),
        ("absolute", &format!("#[path = \"{}\"]\nmod x;\n", base.join("outside.rs").display()) as &str),
        ("dir symlink then ..", "#[path = \"../vendored/../outside.rs\"]\nmod x;\n"),
        ("inline module path", "mod m {\n    #[path = \"../../../outside.rs\"]\n    mod x;\n}\n"),
        ("cfg_attr path", "#[cfg_attr(unix, path = \"../../outside.rs\")]\nmod x;\n"),
        ("in-root redirect (accepted)", "#[path = \"other/x.rs\"]\npub mod x;\n"),
    ];
    let tree = DiskTree::new(&root)?;
    for (label, lib) in cases {
        write(&root, "src/lib.rs", lib);
        match Workspace::load(&tree, &["."]) {
            Ok(ws) => println!("{label}: LOADED; demo::x::inside = {:?}", ws.resolve("demo::x::inside").map(|r| r.path)),
            Err(e) => println!("{label}: refused: {e}\n    kind={:?} origin={:?} file={} line={:?}", e.kind, e.origin, e.file, e.line),
        }
    }
    // symlinked module file to outside
    write(&root, "src/lib.rs", "mod inner;\n");
    symlink(base.join("outside.rs"), root.join("src/inner.rs"))?;
    match Workspace::load(&tree, &["."]) {
        Ok(_) => println!("symlinked module file: LOADED"),
        Err(e) => println!("symlinked module file: refused: {e}"),
    }
    fs::remove_file(root.join("src/inner.rs"))?;
    // Cargo target path that leaves the root
    write(&root, "src/lib.rs", "pub fn ok() {}\n");
    write(&root, "Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"b\"\npath = \"../outside.rs\"\n");
    match Workspace::load(&tree, &["."]) {
        Ok(_) => println!("bin path escape: LOADED"),
        Err(e) => println!("bin path escape: refused: {e}"),
    }
    // over-cap source file through a load (8 MiB + 1) and a file of exactly 8 MiB at the default
    write(&root, "Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n");
    let fs_perm = fs::Permissions::from_mode(0o644);
    fs::set_permissions(base.join("outside.rs"), fs_perm)?;
    Ok(())
}

fn cap8() -> R {
    let base = fresh("cap8");
    let root = base.join("root");
    write(&root, "Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n");
    let head = "pub fn first() {}\n";
    let pad_line = "// padding padding padding padding padding padding padding padding\n";
    let target = 8 * 1024 * 1024;
    let mut text = String::from(head);
    while text.len() + pad_line.len() <= target {
        text.push_str(pad_line);
    }
    while text.len() < target {
        text.push('\n');
    }
    assert_eq!(text.len(), target);
    write(&root, "src/lib.rs", &text);
    let tree = DiskTree::new(&root)?;
    let t0 = std::time::Instant::now();
    match Workspace::load(&tree, &["."]) {
        Ok(ws) => println!("8 MiB source at default: loaded in {:?}; demo::first = {:?}", t0.elapsed(), ws.resolve("demo::first").map(|r| r.path)),
        Err(e) => println!("8 MiB source at default: refused: {e}"),
    }
    text.push('\n');
    write(&root, "src/lib.rs", &text);
    match Workspace::load(&tree, &["."]) {
        Ok(_) => println!("8 MiB + 1 source: LOADED (wrong)"),
        Err(e) => println!("8 MiB + 1 source: refused: {e} kind={:?}", e.kind),
    }
    write(&root, "src/lib.rs", "pub fn first() {}\n// some more bytes here\n");
    let mut low = LoadOptions::default();
    low.max_file_bytes = 30;
    match Workspace::load_with(&tree, &["."], &low) {
        Ok(_) => println!("lowered cap 30: LOADED (wrong)"),
        Err(e) => println!("lowered cap 30: refused: {e}"),
    }
    Ok(())
}

fn memory() -> R {
    let tree = MemoryTree::from([
        ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        ("src/lib.rs", "pub mod a;\npub use a::b::deep as shallow;\npub use hop1::*;\nmod hop1 { pub use crate::hop2::thing; }\nmod hop2 { pub use crate::a::b::deep as thing; }\n#[path = \"gen/generated.rs\"]\npub mod g;\nmod outer {\n    #[path = \"elsewhere.rs\"]\n    pub mod inner;\n}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn unit() {}\n}\n"),
        ("src/a.rs", "pub mod b;\n"),
        ("src/a/b.rs", "pub fn deep() {}\n"),
        ("src/gen/generated.rs", "pub mod child;\n"),
        ("src/gen/child.rs", "pub struct Child;\n"),
        ("src/outer/elsewhere.rs", "pub fn there() {}\n"),
        ("src/main.rs", "fn main() {}\n"),
        ("src/bin/tool.rs", "fn main() {}\n"),
        ("build.rs", "fn main() {}\n"),
        ("tests/it.rs", "#[test]\nfn case() {}\n"),
        ("benches/b.rs", "fn bench() {}\n"),
        ("examples/e.rs", "fn main() {}\nfn helper() {}\n"),
    ]);
    let ws = Workspace::load(&tree, &["."])?;
    for t in ws.targets() {
        println!("target {:?} name={} crate_path={} root={} shipping={}", t.kind, t.name, t.crate_path, t.root, t.shipping);
    }
    for p in [
        "demo::shallow",
        "demo::thing",
        "demo::g::child::Child",
        "demo::outer::inner::there",
        "demo::tests::unit",
        "demo::test_crate::it::case",
        "demo::bench_crate::b::bench",
        "demo::example_crate::e::helper",
        "demo::example_crate::e",
        "demo::bin_crate::tool",
        "demo::build_script",
    ] {
        match ws.resolve(p) {
            Ok(r) => {
                let item = ws.item(r.sites[0]).unwrap();
                println!("OK {p} -> {} file={} test={:?} shipping={} chain={:?} via_glob={}", r.path, item.file, item.test, ws.is_shipping(item), r.chain, r.via_glob);
            }
            Err(e) => println!("ERR {p}: {e}"),
        }
    }
    // escape on a Memory tree, with the outside path present in no form
    let esc = MemoryTree::from([
        ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
        ("src/lib.rs", "#[path = \"../../outside.rs\"]\nmod x;\n"),
    ]);
    match Workspace::load(&esc, &["."]) {
        Ok(_) => println!("memory escape: LOADED (wrong)"),
        Err(e) => println!("memory escape: refused: {e}"),
    }
    // re-export chain Cyclic, Ambiguous
    let amb = MemoryTree::from([
        ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
        ("src/lib.rs", "mod a { pub fn f() {} }\nmod b { pub fn f() {} }\npub use a::*;\npub use b::*;\npub use self::c1 as c2;\npub use self::c2 as c1;\n"),
    ]);
    let ws = Workspace::load(&amb, &["."])?;
    println!("glob vs glob: {:?}", ws.resolve("demo::f").err().map(|e| e.to_string()));
    println!("cycle: {:?}", ws.resolve("demo::c1").err().map(|e| e.to_string()));
    let _ = (Namespace::Type, RelPath::new("x")?, tree.entry(&RelPath::new("src")?)?);
    Ok(())
}
