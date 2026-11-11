// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Registry source authentication, immutable reads and refused archive neighbors.

use purrdf_gts::files::{FileEntry, FileEntryKind};
use raf_source::{
    EntryKind, LoadOptions, LockedCatalog, LockedManifest, MemoryTree, Opened, ReferenceTree,
    RegistryArchiveTree, RelPath, Tree, TreeError, Workspace,
};

fn file(path: &str, bytes: &[u8]) -> FileEntry {
    FileEntry {
        path: path.to_owned(),
        data: Some(bytes.to_vec()),
        ..FileEntry::default()
    }
}
fn archive(entries: &[FileEntry]) -> Vec<u8> {
    let encoded = purrdf_gts::files::pack_entries_v2(entries).expect("native fixture files");
    let graph = purrdf_gts::reader::read(&encoded, false, None);
    assert!(graph.diagnostics.is_empty(), "{:?}", graph.diagnostics);
    purrdf_gts::tar::to_tar_vec(&graph, &purrdf_gts::tar::ToTarOptions::default())
        .expect("native fixture tar")
}
fn gzip(bytes: &[u8]) -> Vec<u8> {
    purrdf_deflate::gzip::compress(bytes, purrdf_deflate::Level::DEFAULT)
}
fn catalog(archive: &[u8]) -> LockedCatalog {
    let checksum = purrdf_core::ContentDigest::of(archive);
    let text = format!(
        "version=4\n[[package]]\nname='demo'\nversion='1.0.0'\nsource='registry+https://github.com/rust-lang/crates.io-index'\nchecksum='{checksum}'\n"
    );
    LockedCatalog::load(
        &MemoryTree::from([("Cargo.lock", text.as_str())]),
        &RelPath::new("Cargo.lock").unwrap(),
        &LoadOptions::default(),
    )
    .expect("native registry catalog")
}
fn valid() -> Vec<FileEntry> {
    vec![
        file(
            "demo-1.0.0/Cargo.toml",
            b"[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n",
        ),
        file("demo-1.0.0/src/lib.rs", b"pub fn provided() {}\r\n"),
        file("demo-1.0.0/data.bin", &[0xff, 0]),
    ]
}

#[test]
fn authenticated_registry_contents_feed_the_loader_and_keep_exact_binary_and_path_boundaries() {
    let archive = gzip(&archive(&valid()));
    let catalog = catalog(&archive);
    let row = catalog.packages().next().unwrap().id();
    let tree = RegistryArchiveTree::from_bytes(&catalog, row, &archive, &LoadOptions::default())
        .expect("valid archive");
    let path = RelPath::new("src/lib.rs").unwrap();
    assert_eq!(tree.read(&path, 100).unwrap(), "pub fn provided() {}\r\n");
    let canonical = RelPath::new("src/../src/lib.rs").unwrap();
    assert_eq!(
        tree.identity(&path).unwrap(),
        tree.identity(&canonical).unwrap()
    );
    assert!(matches!(
        tree.read(&RelPath::new("data.bin").unwrap(), 100),
        Err(TreeError::NotUtf8 { .. })
    ));
    assert!(matches!(
        tree.read(&path, 1),
        Err(TreeError::TooLarge { .. })
    ));
    assert!(matches!(
        tree.read_unseen(&path, 0, &|_| true).unwrap(),
        Opened::Seen(_)
    ));
    assert!(
        tree.read(&RelPath::new("data.bin/../Cargo.toml").unwrap(), 100)
            .is_err()
    );
    assert_eq!(
        tree.entry(&RelPath::new("src").unwrap()).unwrap(),
        Some(EntryKind::Dir)
    );
    assert_eq!(
        tree.list(&RelPath::new("src/..").unwrap()).unwrap().len(),
        3
    );
    assert!(
        Workspace::load(&tree, &["."])
            .unwrap()
            .resolve("demo::provided")
            .is_ok()
    );
    let mut changed = archive.clone();
    changed[0] ^= 1;
    assert!(
        RegistryArchiveTree::from_bytes(&catalog, row, &changed, &LoadOptions::default())
            .err()
            .unwrap()
            .to_string()
            .contains("checksum")
    );
}

#[test]
fn wrong_package_links_conflicting_inventory_and_nested_transport_refuse_beside_a_valid_archive() {
    let valid_tar = archive(&valid());
    let mut wrong_prefix = valid();
    wrong_prefix.push(file("other-1.0.0/extra", b"foreign"));
    let missing_manifest = vec![file("demo-1.0.0/src/lib.rs", b"pub fn provided() {}")];
    let mut conflict = valid();
    conflict.push(file("demo-1.0.0/src", b"not a directory"));
    let mut links = valid();
    links.push(FileEntry {
        path: "demo-1.0.0/link".to_owned(),
        kind: FileEntryKind::Symlink,
        link_target: Some("src".to_owned()),
        ..FileEntry::default()
    });
    for (label, tar) in [
        ("prefix", archive(&wrong_prefix)),
        ("manifest", archive(&missing_manifest)),
        ("inventory", archive(&conflict)),
        ("link", archive(&links)),
        ("transport", gzip(&valid_tar)),
        ("invalid tar", b"malformed tar".to_vec()),
    ] {
        let bytes = gzip(&tar);
        let catalog = catalog(&bytes);
        let row = catalog.packages().next().unwrap().id();
        assert!(
            RegistryArchiveTree::from_bytes(&catalog, row, &bytes, &LoadOptions::default())
                .is_err(),
            "{label}"
        );
    }
    let bytes = gzip(&valid_tar);
    let catalog = catalog(&bytes);
    let row = catalog.packages().next().unwrap().id();
    RegistryArchiveTree::from_bytes(&catalog, row, &bytes, &LoadOptions::default())
        .expect("valid neighbor");
    let mut limited = LoadOptions::default();
    limited.max_total_bytes = bytes.len();
    assert!(
        RegistryArchiveTree::from_bytes(&catalog, row, &bytes, &limited).is_err(),
        "decoded archive exceeds its admitted bytes"
    );
    let mut limited = LoadOptions::default();
    limited.max_item_bytes = 0;
    assert!(
        RegistryArchiveTree::from_bytes(&catalog, row, &bytes, &limited).is_err(),
        "storage is admitted before retention"
    );
}

fn raw_member(name: &[u8], body: &[u8]) -> Vec<u8> {
    let mut header = [0u8; 512];
    header[..name.len()].copy_from_slice(name);
    header[100..108].copy_from_slice(b"0000644\0");
    header[108..116].copy_from_slice(b"0000000\0");
    header[116..124].copy_from_slice(b"0000000\0");
    header[124..136].copy_from_slice(format!("{:011o}\0", body.len()).as_bytes());
    header[136..148].copy_from_slice(b"00000000000\0");
    header[148..156].copy_from_slice(b"        ");
    header[156] = b'0';
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    let sum: u32 = header.iter().map(|byte| u32::from(*byte)).sum();
    header[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
    let mut out = header.to_vec();
    out.extend_from_slice(body);
    out.resize(out.len().div_ceil(512) * 512, 0);
    out
}

/// Protects exact tar path identity: the published native inventory keeps raw name
/// bytes, so a non-UTF-8 member name refuses instead of being read as a lossy
/// replacement path that could alias a different file. The valid neighbor proves
/// the same hand-framed archive loads when the name is UTF-8.
#[test]
fn non_utf8_member_names_refuse_instead_of_aliasing_a_replacement_path() {
    let manifest = b"[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n";
    let build = |name: &[u8]| {
        let mut tar = raw_member(b"demo-1.0.0/Cargo.toml", manifest);
        tar.extend(raw_member(name, b"pub fn provided() {}\n"));
        tar.extend([0u8; 1024]);
        gzip(&tar)
    };
    for (label, name, loads) in [
        ("utf8", b"demo-1.0.0/lib.rs".as_slice(), true),
        ("raw byte", b"demo-1.0.0/\xff.rs".as_slice(), false),
    ] {
        let bytes = build(name);
        let catalog = catalog(&bytes);
        let row = catalog.packages().next().unwrap().id();
        let loaded =
            RegistryArchiveTree::from_bytes(&catalog, row, &bytes, &LoadOptions::default());
        assert_eq!(loaded.is_ok(), loads, "{label}");
        if let Err(error) = loaded {
            // The refusal must come from the member name, not from tar framing.
            let chain = std::iter::successors(Some(&error as &dyn std::error::Error), |error| {
                error.source()
            })
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" | ");
            assert!(
                chain.to_ascii_lowercase().contains("utf-8"),
                "{label}: {chain}"
            );
        }
    }
}

/// Protects large valid packages: record validation is linear, so an archive of
/// many small files loads inside the default work grant. A per-record scan of all
/// earlier records exceeds that grant near ten thousand files.
#[test]
fn an_archive_of_many_small_files_loads_within_the_default_work_grant() {
    let mut entries = valid();
    entries.extend((0..12_000).map(|index| {
        file(
            &format!(
                "demo-1.0.0/generated/group{:03}/module_{index:05}.rs",
                index / 100
            ),
            b"",
        )
    }));
    let bytes = gzip(&archive(&entries));
    let catalog = catalog(&bytes);
    let row = catalog.packages().next().unwrap().id();
    let tree = RegistryArchiveTree::from_bytes(&catalog, row, &bytes, &LoadOptions::default())
        .expect("a large valid archive stays inside the default grants");
    assert_eq!(
        tree.list(&RelPath::new("generated/group119").unwrap())
            .unwrap()
            .len(),
        100
    );
}

/// Protects the caller's configured per-file ceiling: a read may not pass a larger
/// `max_bytes` to get beyond `max_file_bytes`. The default-option read of the same
/// file succeeds beside it.
#[test]
fn registry_reads_honor_the_configured_file_limit() {
    let bytes = gzip(&archive(&valid()));
    let catalog = catalog(&bytes);
    let row = catalog.packages().next().unwrap().id();
    let path = RelPath::new("src/lib.rs").unwrap();
    let open = RegistryArchiveTree::from_bytes(&catalog, row, &bytes, &LoadOptions::default())
        .expect("valid archive");
    assert!(open.read(&path, 100).is_ok());
    let mut limited = LoadOptions::default();
    limited.max_file_bytes = 1;
    let tree = RegistryArchiveTree::from_bytes(&catalog, row, &bytes, &limited)
        .expect("the archive itself fits");
    assert!(matches!(
        tree.read(&path, 100),
        Err(TreeError::TooLarge { limit: 1, .. })
    ));
}

/// Protects first-party registry identity: an archive authenticated against one
/// catalog cannot be bound to another catalog's row that merely shares its name,
/// version and source. The same archive under its own catalog loads beside it.
#[test]
fn a_first_party_archive_cannot_bind_to_a_different_catalogs_row() {
    let bytes = gzip(&archive(&valid()));
    let own = catalog(&bytes);
    let own_row = own.packages().next().unwrap().id();
    let tree = RegistryArchiveTree::from_bytes(&own, own_row, &bytes, &LoadOptions::default())
        .expect("valid archive");
    let mut other_files = valid();
    other_files.push(file("demo-1.0.0/extra.txt", b"different archive"));
    let other = catalog(&gzip(&archive(&other_files)));
    let other_row = other.packages().next().unwrap().id();
    let manifest = RelPath::new("Cargo.toml").unwrap();
    let roots = [RelPath::root()];
    let load = |catalog: &LockedCatalog, row| {
        let local = [LockedManifest::new(row, manifest.clone(), &[])];
        Workspace::load_reference_sources_under(
            catalog,
            ReferenceTree::Registry(&tree),
            &roots,
            &local,
            &[],
            &LoadOptions::default(),
        )
    };
    load(&own, own_row).expect("the archive under its own catalog loads");
    let refused = load(&other, other_row)
        .err()
        .expect("another catalog's row refuses");
    assert!(refused.to_string().contains("differs"), "{refused}");
}
