// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The shipped history command admits only signed, closed compiler captures.

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use capnp::{
    message::{Builder as Message, ReaderOptions},
    schema_capnp::code_generator_request,
    serialize,
};
use purrdf_core::ContentDigest;
use purrdf_ed25519::SigningKey;
use raf_fixtures::git_repo;
use raf_rdf::{Builder, Format, TermValue, Triple, vocab};

const DESCRIPTOR: &[u8] = include_bytes!("fixtures/capnp/capture-closed.bin");
const SOURCES: [(&str, &[u8]); 4] = [
    (
        "capture-entry.capnp",
        include_bytes!("fixtures/capnp/capture-entry.capnp"),
    ),
    (
        "capture-middle.capnp",
        include_bytes!("fixtures/capnp/capture-middle.capnp"),
    ),
    (
        "capture-leaf.capnp",
        include_bytes!("fixtures/capnp/capture-leaf.capnp"),
    ),
    (
        "capture-aux/capture-spare.capnp",
        include_bytes!("fixtures/capnp/capture-aux/capture-spare.capnp"),
    ),
];

fn digest(bytes: &[u8]) -> String {
    purrdf_hash::hex::encode(ContentDigest::of(bytes).as_bytes())
}

fn put(graph: &mut Builder, subject: &str, property: &str, object: TermValue) {
    graph
        .insert(Triple {
            subject: TermValue::iri(subject),
            predicate: TermValue::iri(property),
            object,
        })
        .unwrap();
}

fn text(graph: &mut Builder, subject: &str, key: &str, value: &str) {
    put(
        graph,
        subject,
        &format!("{}{key}", vocab::REPORT),
        TermValue::simple_literal(value),
    );
}

fn publish_capture(
    root: &Path,
    key: &SigningKey,
    descriptor: &[u8],
    sources: &[(&str, &[u8])],
) -> String {
    fs::write(root.join("schema.bin"), descriptor).unwrap();
    let directory = root.join("schemas");
    fs::create_dir_all(&directory).unwrap();
    let mut graph = Builder::new();
    put(
        &mut graph,
        "",
        vocab::TYPE,
        TermValue::iri(format!("{}CapnpCaptureV1", vocab::REPORT)),
    );
    text(&mut graph, "", "compilerVersion", "1.5.0");
    text(
        &mut graph,
        "",
        "compilerSha256",
        &digest(b"test compiler identity"),
    );
    text(&mut graph, "", "descriptorFile", "schema.bin");
    text(&mut graph, "", "descriptorSha256", &digest(descriptor));
    text(&mut graph, "", "schemaRoot", "schemas");
    for (index, (name, bytes)) in sources.iter().enumerate() {
        let path = directory.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
        let id = format!("#source-{index}");
        put(
            &mut graph,
            "",
            &format!("{}schemaSource", vocab::REPORT),
            TermValue::iri(&id),
        );
        put(
            &mut graph,
            &id,
            vocab::TYPE,
            TermValue::iri(format!("{}CapnpSchemaSource", vocab::REPORT)),
        );
        text(&mut graph, &id, "file", name);
        text(&mut graph, &id, "sha256", &digest(bytes));
    }
    let capture = graph.finish().unwrap().to_text(Format::NTriples).unwrap();
    fs::write(
        root.join("capture.sig"),
        key.sign(capture.as_bytes()).to_bytes(),
    )
    .unwrap();
    fs::write(root.join("capture.nt"), capture).unwrap();
    git_repo::commit_all(root)
}

fn history(root: &Path, base: &str, current: &str, key: Option<&SigningKey>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-census"));
    command.current_dir(root).args([
        "capnp-history",
        "--base",
        base,
        "--current",
        current,
        "--base-descriptor",
        "schema.bin",
        "--current-descriptor",
        "schema.bin",
        "--type",
        "0xfe3dcd67079854c3",
    ]);
    if let Some(key) = key {
        let public = purrdf_hash::hex::encode(&key.verifying_key().to_bytes());
        command.args([
            "--capture-key",
            &public,
            "--base-capture",
            "capture.nt",
            "--base-capture-signature",
            "capture.sig",
            "--current-capture",
            "capture.nt",
            "--current-capture-signature",
            "capture.sig",
        ]);
    }
    command.output().unwrap()
}

fn assert_refused(output: Output, expected: &str) {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains(expected), "{diagnostic}");
}

/// Both snapshots need a statement from the selected signer over the exact
/// committed descriptor and source bytes. Working files have no authority.
#[test]
fn capnp_capture_authenticates_the_exact_pinned_inputs() {
    let scratch = purrdf_testkit::TempDir::new_in(Path::new(env!("CARGO_TARGET_TMPDIR"))).unwrap();
    let root = scratch.path();
    git_repo::init(root);
    let key = SigningKey::from_bytes(&[3; 32]);
    let base = publish_capture(root, &key, DESCRIPTOR, &SOURCES);
    let mut changed_sources = SOURCES;
    changed_sources[0] = (
        "capture-entry-renamed.capnp",
        include_bytes!("fixtures/capnp/capture-entry-renamed.capnp"),
    );
    let current = publish_capture(
        root,
        &key,
        include_bytes!("fixtures/capnp/capture-renamed.bin"),
        &changed_sources,
    );

    let wrong_key = SigningKey::from_bytes(&[4; 32]);
    assert_refused(
        history(root, &base, &current, Some(&wrong_key)),
        "capture signature verification failed",
    );
    let signature = root.join("capture.sig");
    let signature_bytes = fs::read(&signature).unwrap();
    fs::write(&signature, [0; 64]).unwrap();
    let bad_signature = git_repo::commit_all(root);
    assert_refused(
        history(root, &base, &bad_signature, Some(&key)),
        "capture signature verification failed",
    );
    fs::write(&signature, signature_bytes).unwrap();

    for name in ["capture-middle.capnp", "capture-leaf.capnp"] {
        let path = root.join("schemas").join(name);
        let source = fs::read(&path).unwrap();
        fs::write(&path, b"different source blob").unwrap();
        let changed = git_repo::commit_all(root);
        assert_refused(
            history(root, &base, &changed, Some(&key)),
            &format!("capture digest differs for schema schemas/{name}"),
        );
        fs::write(path, source).unwrap();
    }
    fs::write(root.join("schema.bin"), b"different descriptor blob").unwrap();
    let bad_descriptor = git_repo::commit_all(root);
    assert_refused(
        history(root, &base, &bad_descriptor, Some(&key)),
        "capture descriptor path or digest differs",
    );

    fs::write(root.join("capture.nt"), b"uncommitted statement").unwrap();
    let comparison = history(root, &base, &current, Some(&key));
    assert_eq!(comparison.status.code(), Some(1), "{comparison:?}");
    assert!(comparison.stderr.is_empty(), "{comparison:?}");
    assert_eq!(
        comparison.stdout,
        b"Cap'n Proto history for 0xfe3dcd67079854c3.nested: member renamed to content\n"
    );
}

/// A signed digest set cannot stand in for the compiler's requested file
/// rows, emitted file nodes, and the names and IDs of every actual import.
#[test]
fn capnp_capture_refuses_an_open_compiler_import_set() {
    let scratch = purrdf_testkit::TempDir::new_in(Path::new(env!("CARGO_TARGET_TMPDIR"))).unwrap();
    let root = scratch.path();
    git_repo::init(root);
    let key = SigningKey::from_bytes(&[3; 32]);
    let base = publish_capture(root, &key, DESCRIPTOR, &SOURCES);
    let transitive_missing = publish_capture(
        root,
        &key,
        include_bytes!("fixtures/capnp/capture-missing-transitive.bin"),
        &[SOURCES[0], SOURCES[1], SOURCES[3]],
    );
    assert_refused(
        history(root, &base, &transitive_missing, Some(&key)),
        "requested schema rows, and file nodes must agree",
    );

    let unused_missing = publish_capture(
        root,
        &key,
        include_bytes!("fixtures/capnp/capture-missing-unused.bin"),
        &SOURCES[..3],
    );
    assert_refused(
        history(root, &base, &unused_missing, Some(&key)),
        "unresolved schema import \"/capture-aux/capture-spare.capnp\"",
    );
    let strict = history(root, &base, &unused_missing, None);
    assert!(strict.status.success(), "{strict:?}");
    assert!(strict.stdout.is_empty(), "{strict:?}");
}

/// A scalar default can depend on a file the compiler omits from its import table.
#[test]
fn capnp_capture_reads_literal_dependencies_missing_from_the_request() {
    let scratch = purrdf_testkit::TempDir::new_in(Path::new(env!("CARGO_TARGET_TMPDIR"))).unwrap();
    let root = scratch.path();
    git_repo::init(root);
    let key = SigningKey::from_bytes(&[3; 32]);
    let user = include_str!("fixtures/capnp/capture-default-user.capnp");
    let defaults = include_bytes!("fixtures/capnp/capture-defaults.capnp");
    let closed = include_bytes!("fixtures/capnp/capture-default-closed.bin");
    let signed_user = ("capture-default-user.capnp", user.as_bytes());
    let signed_defaults = ("capture-defaults.capnp", defaults.as_slice());
    let base = publish_capture(root, &key, closed, &[signed_user, signed_defaults]);

    let clean = history(root, &base, &base, Some(&key));
    assert!(clean.status.success(), "{clean:?}");
    let open = publish_capture(
        root,
        &key,
        include_bytes!("fixtures/capnp/capture-default-open.bin"),
        &[signed_user],
    );
    assert_refused(
        history(root, &base, &open, Some(&key)),
        "unresolved schema import \"capture-defaults.capnp\"",
    );

    for (source, diagnostic) in [
        (
            user.replace(
                "\"capture-defaults.capnp\"",
                r#""capture-defaults\x2ecapnp""#,
            ),
            "must use an unescaped import path",
        ),
        (
            format!("{user}\nconst content :Data = embed \"data.bin\";\n"),
            "uses embed, which the capture profile does not admit",
        ),
        (
            user.replace("capture-defaults.capnp", &"a".repeat(4097)),
            "schema import from capture-default-user.capnp exceeds its length bound",
        ),
        (
            user.replace(
                "\"capture-defaults.capnp\".limit",
                &format!(
                    "({}0)",
                    "import \"capture-defaults.capnp\".limit,".repeat(8193)
                ),
            ),
            "exceeds its 8192 source import allowance",
        ),
    ] {
        let bad = publish_capture(
            root,
            &key,
            closed,
            &[
                ("capture-default-user.capnp", source.as_bytes()),
                signed_defaults,
            ],
        );
        assert_refused(history(root, &base, &bad, Some(&key)), diagnostic);
    }
}

fn edited_descriptor(edit: impl FnOnce(code_generator_request::Builder<'_>)) -> Vec<u8> {
    let mut bytes = DESCRIPTOR;
    let source = serialize::read_message_from_flat_slice(&mut bytes, ReaderOptions::new()).unwrap();
    let mut request = Message::new_default();
    request
        .set_root(
            source
                .get_root::<code_generator_request::Reader<'_>>()
                .unwrap(),
        )
        .unwrap();
    edit(request.get_root().unwrap());
    serialize::write_message_to_words(&request)
}

/// File nodes and import IDs identify exact requested paths inside the signed schema root.
#[test]
fn capnp_capture_refuses_import_identity_mismatch_and_root_escape() {
    const MIDDLE: u64 = 0xe914_47cf_636b_013a;
    const SPARE: u64 = 0xbedd_0c55_85e3_8aff;
    let scratch = purrdf_testkit::TempDir::new_in(Path::new(env!("CARGO_TARGET_TMPDIR"))).unwrap();
    let root = scratch.path();
    git_repo::init(root);
    let key = SigningKey::from_bytes(&[3; 32]);
    let base = publish_capture(root, &key, DESCRIPTOR, &SOURCES);
    for (name, file_id) in [
        ("capture-middle.capnp", SPARE),
        ("../schemas/capture-middle.capnp", MIDDLE),
    ] {
        let invalid = edited_descriptor(|request| {
            let mut import = request
                .get_requested_files()
                .unwrap()
                .get(0)
                .get_imports()
                .unwrap()
                .get(0);
            import.set_name(name);
            import.set_id(file_id);
        });
        let current = publish_capture(root, &key, &invalid, &SOURCES);
        assert_refused(
            history(root, &base, &current, Some(&key)),
            &format!(
                "Cap'n Proto capture capture.nt at {current}: unresolved schema import {name:?}"
            ),
        );
    }
    let inconsistent = edited_descriptor(|request| {
        let mut file = request.get_nodes().unwrap().get(0);
        file.set_display_name("different-entry.capnp");
    });
    let current = publish_capture(root, &key, &inconsistent, &SOURCES);
    assert_refused(
        history(root, &base, &current, Some(&key)),
        "file node name differs from requested schema capture-entry.capnp",
    );
}

/// Capture input limits refuse entire committed blobs before comparison.
#[test]
fn capnp_capture_refuses_oversized_inputs() {
    let scratch = purrdf_testkit::TempDir::new_in(Path::new(env!("CARGO_TARGET_TMPDIR"))).unwrap();
    let root = scratch.path();
    git_repo::init(root);
    let key = SigningKey::from_bytes(&[3; 32]);
    let base = publish_capture(root, &key, DESCRIPTOR, &SOURCES);
    let clean = history(root, &base, &base, Some(&key));
    assert!(clean.status.success(), "{clean:?}");

    fs::write(root.join("capture.nt"), vec![b' '; 128 * 1024 + 1]).unwrap();
    let oversized_statement = git_repo::commit_all(root);
    assert_refused(
        history(root, &base, &oversized_statement, Some(&key)),
        "capture.nt is larger than the limit of 131072 bytes",
    );

    let mut leaf = SOURCES[2].1.to_vec();
    leaf.resize(2 * 1024 * 1024 + 1, b' ');
    let mut oversized_sources: [(&str, &[u8]); 4] = SOURCES;
    oversized_sources[2].1 = &leaf;
    let oversized_schema = publish_capture(root, &key, DESCRIPTOR, &oversized_sources);
    assert_refused(
        history(root, &base, &oversized_schema, Some(&key)),
        "schemas/capture-leaf.capnp is larger than the limit of 2097152 bytes",
    );
}
