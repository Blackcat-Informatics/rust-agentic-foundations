// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Read-only cached object acquisition for one exact pinned definition experiment.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::ops::Range;
use std::process::{Command, Stdio};

use raf_source::{
    DiskTree, GitObject, GitObjectId, GitObjectKind, GitObjects, GitPackageRoots, LoadOptions,
    LockedCatalog, LockedEdge, LockedManifest, LockedPackageId, Namespace, RelPath, TreeError,
    VerifiedGitTree, Workspace,
};

const PIN: &str = "5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43";
const URL: &str = "https://github.com/Blackcat-Informatics/purrdf.git";
type Failure = Box<dyn Error>;

struct Captured {
    bytes: Vec<u8>,
    objects: BTreeMap<GitObjectId, (GitObjectKind, Range<usize>)>,
    paths: BTreeMap<String, GitObjectId>,
}

impl GitObjects for Captured {
    fn read_object(&self, id: GitObjectId, max_bytes: u64) -> Result<GitObject<'_>, TreeError> {
        let (kind, span) = self.objects.get(&id).ok_or_else(|| TreeError::NotFound {
            path: id.to_string(),
        })?;
        let body = &self.bytes[span.clone()];
        if u64::try_from(body.len()).unwrap_or(u64::MAX) > max_bytes {
            return Err(TreeError::TooLarge {
                path: id.to_string(),
                limit: max_bytes,
            });
        }
        Ok(GitObject { kind: *kind, body })
    }
}

fn git(repository: &str, args: &[&str], input: Option<&[u8]>) -> Result<Vec<u8>, Failure> {
    let stdin = if let Some(input) = input {
        let path = std::env::var("RAF_GIT_PROBE_STDIN")?;
        std::fs::write(&path, input)?;
        Stdio::from(std::fs::File::open(path)?)
    } else {
        Stdio::null()
    };
    let mut command = Command::new("git");
    command
        .args(["--no-replace-objects", "-C", repository])
        .args(args)
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let result = command.output()?;
    if !result.status.success() {
        return Err(format!("git {args:?}: {}", String::from_utf8_lossy(&result.stderr)).into());
    }
    Ok(result.stdout)
}

fn id(text: &str) -> Result<GitObjectId, Failure> {
    let mut bytes = [0; 20];
    if purrdf_hash::hex::decode_to_slice(text, &mut bytes)? != 20 {
        return Err("not a complete SHA-1 object ID".into());
    }
    Ok(GitObjectId::from_sha1(bytes))
}

fn collect(
    text: &[u8],
    ids: &mut BTreeSet<GitObjectId>,
    paths: &mut BTreeMap<String, GitObjectId>,
) -> Result<(), Failure> {
    for line in std::str::from_utf8(text)?.lines() {
        let (header, path) = line.split_once('\t').ok_or("invalid ls-tree framing")?;
        let fields: Vec<_> = header.split_ascii_whitespace().collect();
        if fields.len() != 3 {
            return Err("incomplete ls-tree entry".into());
        }
        let oid = id(fields[2])?;
        if fields[1] == "tree"
            || (path.ends_with(".rs")
                && (path.starts_with("crates/lex/") || path.starts_with("crates/hash/")))
            || path == "Cargo.toml"
            || path.ends_with("/Cargo.toml")
        {
            ids.insert(oid);
            paths.insert(path.to_owned(), oid);
        }
    }
    Ok(())
}

fn capture(repository: &str, max_object: u64, artifact: &str) -> Result<Captured, Failure> {
    let mut ids = BTreeSet::new();
    let mut paths = BTreeMap::new();
    ids.insert(id(PIN)?);
    let root = git(repository, &["rev-parse", &format!("{PIN}^{{tree}}")], None)?;
    ids.insert(id(std::str::from_utf8(&root)?.trim())?);
    collect(
        &git(repository, &["ls-tree", "-r", "-t", PIN], None)?,
        &mut ids,
        &mut paths,
    )?;
    let requests: String = ids.iter().map(|id| format!("{id}\n")).collect();
    let sizes = git(
        repository,
        &["cat-file", "--batch-check"],
        Some(requests.as_bytes()),
    )?;
    let mut expected = 0_usize;
    for line in std::str::from_utf8(&sizes)?.lines() {
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        if fields.len() != 3 {
            return Err(format!("required cached object unavailable: {line}").into());
        }
        let size: usize = fields[2].parse()?;
        if u64::try_from(size)? > max_object {
            return Err(format!("object {} exceeds acquisition limit", fields[0]).into());
        }
        expected = expected
            .checked_add(size)
            .ok_or("object input size overflow")?;
    }
    if expected > 64 * 1024 * 1024 {
        return Err("captured object set exceeds explicit 64 MiB host input bound".into());
    }
    let bytes = git(
        repository,
        &["cat-file", "--batch"],
        Some(requests.as_bytes()),
    )?;
    let mut objects = BTreeMap::new();
    let mut cursor = 0_usize;
    for requested in ids {
        let end = cursor
            + bytes[cursor..]
                .iter()
                .position(|byte| *byte == b'\n')
                .ok_or("missing batch header")?;
        let fields: Vec<_> = std::str::from_utf8(&bytes[cursor..end])?
            .split_ascii_whitespace()
            .collect();
        if fields.len() != 3 || id(fields[0])? != requested {
            return Err("wrong batch object header".into());
        }
        let kind = match fields[1] {
            "commit" => GitObjectKind::Commit,
            "tree" => GitObjectKind::Tree,
            "blob" => GitObjectKind::Blob,
            "tag" => GitObjectKind::Tag,
            _ => return Err("unknown object kind".into()),
        };
        let size: usize = fields[2].parse()?;
        cursor = end + 1;
        let end = cursor.checked_add(size).ok_or("object range overflow")?;
        if bytes.get(end) != Some(&b'\n') {
            return Err("incomplete batch object body".into());
        }
        objects.insert(requested, (kind, cursor..end));
        cursor = end + 1;
    }
    if cursor != bytes.len() {
        return Err("extra batch output".into());
    }
    std::fs::write(artifact, &bytes)?;
    println!(
        "captured {} objects, {} body bytes",
        objects.len(),
        expected
    );
    Ok(Captured {
        bytes,
        objects,
        paths,
    })
}

fn row(catalog: &LockedCatalog, name: &str) -> Result<LockedPackageId, Failure> {
    let candidates: Vec<_> = catalog
        .packages()
        .filter(|row| {
            row.name() == name
                && row.version() == "3.0.1"
                && row.source().is_some_and(|source| {
                    source.repository() == URL && source.revision() == Some(PIN)
                })
        })
        .collect();
    if candidates.len() != 1 {
        return Err(format!("{name} has no unique exact pinned row").into());
    }
    Ok(candidates[0].id())
}

fn load(
    catalog: &LockedCatalog,
    captured: &Captured,
    first: &DiskTree,
) -> Result<Workspace, Failure> {
    let options = LoadOptions::default();
    let lex = row(catalog, "purrdf-lex")?;
    let hash = row(catalog, "purrdf-hash")?;
    let own: Vec<_> = catalog
        .packages()
        .filter(|row| row.name() == "raf-source" && row.source().is_none())
        .collect();
    if own.len() != 1 {
        return Err("no unique first-party Source row".into());
    }
    let tree = VerifiedGitTree::new(catalog, lex, captured, &options)?;
    let own_edges = [LockedEdge::new("dependencies.purrdf-lex", None)];
    let local = [LockedManifest::new(
        own[0].id(),
        RelPath::new("crates/raf-source/Cargo.toml")?,
        &own_edges,
    )];
    let lex_edges = [LockedEdge::new("dependencies.purrdf-hash", None)];
    let native = [
        LockedManifest::new(lex, RelPath::new("crates/lex/Cargo.toml")?, &lex_edges),
        LockedManifest::new(hash, RelPath::new("crates/hash/Cargo.toml")?, &[]),
    ];
    Ok(Workspace::load_locked_sources(
        catalog,
        first,
        &local,
        &[GitPackageRoots::new(&tree, &native)],
        &options,
    )?)
}

fn main() -> Result<(), Failure> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("supply existing cached repository and stage raw object artifact".into());
    }
    let options = LoadOptions::default();
    let mut captured = capture(&args[1], options.max_file_bytes, &args[2])?;
    let first = DiskTree::new(".")?;
    let catalog = LockedCatalog::load(&first, &RelPath::new("Cargo.lock")?, &options)?;
    {
        let ws = load(&catalog, &captured, &first)?;
        let own = ws
            .package_ids()
            .find(|id| {
                ws.package(*id)
                    .is_some_and(|package| package.name == "raf-source")
            })
            .ok_or("Source package missing")?;
        for (written, canonical, file) in [
            (
                "Memory",
                "purrdf_lex::allocation::Memory",
                "crates/lex/src/allocation.rs",
            ),
            (
                "purrdf_lex::allocation::Shared",
                "purrdf_lex::allocation::shared::Shared",
                "crates/lex/src/allocation/shared.rs",
            ),
        ] {
            let resolved = ws.resolve_from_package_id_in(
                own,
                "raf_source::workspace",
                written,
                Namespace::Type,
            )?;
            assert_eq!(resolved.path, canonical);
            for site in resolved.sites {
                let item = ws.item(site).ok_or("definition missing")?;
                assert_eq!(item.file.as_str(), file);
                assert_eq!(item.file.authority().index(), 1);
                let target = ws.target(item.target).ok_or("target missing")?;
                let binding = ws
                    .locked_binding(target.package_id)
                    .ok_or("binding missing")?;
                assert_eq!(binding.row(), row(&catalog, "purrdf-lex")?);
                assert_eq!(
                    binding.source().and_then(|source| source.revision()),
                    Some(PIN)
                );
                println!(
                    "{written} -> {} at {}:{}; package {} {}, commit {}",
                    item.path,
                    item.file,
                    item.name_line,
                    binding.name(),
                    binding.version(),
                    PIN
                );
            }
        }
        println!("joined usage: {:?}", ws.usage());
    }
    let blob = captured
        .paths
        .get("crates/lex/src/allocation.rs")
        .ok_or("actual Memory blob absent")?;
    let (_, span) = captured
        .objects
        .get(blob)
        .ok_or("actual Memory span absent")?;
    captured.bytes[span.start] ^= 1;
    let error = load(&catalog, &captured, &first)
        .expect_err("edited native source must refuse before binding");
    assert!(error.to_string().contains("hash does not match"), "{error}");
    println!("edited native object refused: {error}");
    Ok(())
}
