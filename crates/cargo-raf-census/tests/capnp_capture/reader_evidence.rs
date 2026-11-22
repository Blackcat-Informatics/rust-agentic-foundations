// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The shipped verifier corroborates exact inputs without entering consumer code.

use std::{
    ffi::OsString,
    fs,
    path::Path,
    process::{Command, ExitCode, Output},
};

use capnp::{
    message::{Builder as Message, Reader, ReaderOptions, SegmentArray},
    schema_capnp::node,
    traits::HasTypeId,
};
use purrdf_ed25519::SigningKey;
use purrdf_testkit::TempDir;
use raf_fixtures::git_repo;
use raf_rdf::{Builder, TermValue, vocab};

use super::{assert_refused, commit_statement, digest, put, row, text};

const FIRST: &str = "old-a.bin";
const SECOND: &str = "old-b.bin";
const ENTRY: &str = "reader_fixture::read_node";
const EXPECTED_OUTPUT: &[u8] = b"verified signed reader acceptance claim\nold-a.bin\nold-b.bin\n";

struct Fixture {
    scratch: TempDir,
    base: String,
    current: String,
    key: SigningKey,
    artifact: String,
}

impl Fixture {
    fn new(first: &[u8], second: &[u8]) -> Self {
        let scratch = TempDir::new_in(Path::new(env!("CARGO_TARGET_TMPDIR"))).unwrap();
        let root = scratch.path();
        git_repo::init(root);
        fs::write(root.join(FIRST), first).unwrap();
        fs::write(root.join(SECOND), second).unwrap();
        let base = git_repo::commit_all(root);
        fs::write(root.join(FIRST), b"new bytes in the current commit").unwrap();
        let current = git_repo::commit_all(root);
        Self {
            scratch,
            base,
            current,
            key: SigningKey::from_bytes(&[11; 32]),
            artifact: digest(b"identified reader fixture executable"),
        }
    }

    fn sign(&self, payloads: &[(&str, &[u8])], accepted: bool) -> String {
        let mut graph = Builder::new();
        put(
            &mut graph,
            "",
            vocab::TYPE,
            TermValue::iri(format!("{}CapnpReaderEvidenceV1", vocab::REPORT)),
        );
        for (key, value) in [
            ("baseCommit", self.base.as_str()),
            ("currentCommit", self.current.as_str()),
            ("rootType", &root_type()),
            ("readerEntry", ENTRY),
            ("readerSha256", self.artifact.as_str()),
        ] {
            text(&mut graph, "", key, value);
        }
        put(
            &mut graph,
            "",
            &format!("{}accepted", vocab::REPORT),
            TermValue::boolean(accepted),
        );
        for (index, (path, bytes)) in payloads.iter().enumerate() {
            let id = format!("#payload-{index}");
            row(&mut graph, &id, "payload", "CapnpAcceptedPayload");
            text(&mut graph, &id, "file", path);
            text(&mut graph, &id, "sha256", &digest(bytes));
        }
        commit_statement(self.scratch.path(), graph, &self.key)
    }

    fn args(&self, evidence: &str, payloads: &[&str]) -> Vec<OsString> {
        let root = self.scratch.path();
        let public = purrdf_hash::hex::encode(&self.key.verifying_key().to_bytes());
        let mut args = ["capnp-reader-evidence", "--workspace"]
            .into_iter()
            .map(OsString::from)
            .collect::<Vec<_>>();
        args.push(root.as_os_str().to_owned());
        args.extend(
            [
                "--base",
                &self.base,
                "--current",
                &self.current,
                "--evidence-commit",
                evidence,
                "--statement",
                "capture.nt",
                "--signature",
                "capture.sig",
                "--key",
                &public,
                "--type",
                &root_type(),
                "--reader-entry",
                ENTRY,
                "--reader-sha256",
                &self.artifact,
            ]
            .into_iter()
            .map(OsString::from),
        );
        for payload in payloads {
            args.extend([OsString::from("--payload"), OsString::from(payload)]);
        }
        args
    }
}

fn root_type() -> String {
    format!("0x{:016x}", node::Reader::TYPE_ID)
}

pub(super) fn payload(id: u64) -> Vec<u8> {
    let mut message = Message::new_default();
    message.init_root::<node::Builder<'_>>().set_id(id);
    let mut canonical = Message::new_default();
    canonical
        .set_root_canonical::<node::Owned>(
            message.get_root_as_reader::<node::Reader<'_>>().unwrap(),
        )
        .unwrap();
    let output = canonical.get_segments_for_output();
    assert_eq!(output.len(), 1);
    let segments = [output[0]];
    let reader = Reader::new(SegmentArray::new(&segments), ReaderOptions::new());
    assert_eq!(reader.get_root::<node::Reader<'_>>().unwrap().get_id(), id);
    output[0].to_vec()
}

fn check(args: Vec<OsString>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cargo-raf-census"))
        .args(args)
        .output()
        .unwrap()
}

fn replace(args: &mut [OsString], flag: &str, value: &str) {
    let index = args.iter().position(|arg| arg == flag).unwrap();
    args[index + 1] = OsString::from(value);
}

/// The signer can attest only the exact caller scope, and Gate registration stays inert.
#[test]
fn capnp_reader_evidence_authenticates_exact_scope_without_configuring_gate() {
    let first = payload(17);
    let second = payload(31);
    let fixture = Fixture::new(&first, &second);
    let evidence = fixture.sign(&[(FIRST, &first), (SECOND, &second)], true);
    let args = fixture.args(&evidence, &[SECOND, FIRST]);
    let success = check(args.clone());
    assert!(success.status.success(), "{success:?}");
    assert!(success.stderr.is_empty(), "{success:?}");
    assert_eq!(success.stdout, EXPECTED_OUTPUT);

    let mut output = Vec::new();
    let mut diagnostics = Vec::new();
    let status = cargo_raf_census::run(
        std::iter::once(OsString::from("cargo-raf-census")).chain(args),
        Some(|_| panic!("reader evidence must not configure a consumer Gate")),
        &mut output,
        &mut diagnostics,
    );
    assert_eq!(status, ExitCode::SUCCESS);
    assert_eq!(diagnostics, b"");
    assert_eq!(output, EXPECTED_OUTPUT);

    let wrong_key = SigningKey::from_bytes(&[12; 32]);
    let public = purrdf_hash::hex::encode(&wrong_key.verifying_key().to_bytes());
    let wrong_artifact = digest(b"different reader executable");
    for (flag, value, expected) in [
        ("--key", public.as_str(), "signature verification failed"),
        (
            "--base",
            fixture.current.as_str(),
            "signed baseCommit differs",
        ),
        (
            "--current",
            fixture.base.as_str(),
            "signed currentCommit differs",
        ),
        (
            "--evidence-commit",
            fixture.current.as_str(),
            "select a separate commit for the reader evidence",
        ),
        ("--type", "0xfe3dcd67079854c3", "signed rootType differs"),
        (
            "--reader-entry",
            "other::reader",
            "signed readerEntry differs",
        ),
        (
            "--reader-sha256",
            &wrong_artifact,
            "signed reader SHA-256 differs",
        ),
    ] {
        let mut args = fixture.args(&evidence, &[FIRST, SECOND]);
        replace(&mut args, flag, value);
        assert_refused(check(args), expected);
    }
    assert_refused(
        check(fixture.args(&evidence, &[FIRST])),
        "signed payload paths differ",
    );
    let partial = fixture.sign(&[(FIRST, &first)], true);
    assert_refused(
        check(fixture.args(&partial, &[FIRST, SECOND])),
        "signed payload paths differ",
    );
    assert_refused(
        check(fixture.args(&evidence, &[FIRST; 129])),
        "select one to 128 exact base payload paths",
    );
}

/// Agreement with the statement's digest is insufficient when framing is invalid.
#[test]
fn capnp_reader_evidence_refuses_changed_payloads_and_noncanonical_framing() {
    let first = payload(17);
    let second = payload(31);
    let mut padded = first.clone();
    padded.extend_from_slice(&[0; 8]);
    let fixture = Fixture::new(&padded, &second);
    let stale = fixture.sign(&[(FIRST, &first), (SECOND, &second)], true);
    assert_refused(
        check(fixture.args(&stale, &[FIRST, SECOND])),
        "SHA-256 differs from the signed payload digest",
    );
    let malformed = fixture.sign(&[(FIRST, &padded), (SECOND, &second)], true);
    assert_refused(
        check(fixture.args(&malformed, &[FIRST, SECOND])),
        "expected a canonical raw single Cap'n Proto segment",
    );
    let negative = fixture.sign(&[(FIRST, &padded), (SECOND, &second)], false);
    assert_refused(
        check(fixture.args(&negative, &[FIRST, SECOND])),
        "must affirm acceptance of every payload",
    );

    let oversized = vec![0; 2 * 1024 * 1024 + 1];
    let fixture = Fixture::new(&first, &oversized);
    let evidence = fixture.sign(&[(FIRST, &first), (SECOND, &oversized)], true);
    assert_refused(
        check(fixture.args(&evidence, &[FIRST, SECOND])),
        "old-b.bin is larger than the limit of 2097152 bytes",
    );
}
