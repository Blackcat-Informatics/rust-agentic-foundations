// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Row-scoped claims cannot erase or transfer strict findings.

use std::{
    fs,
    process::{Command, Output},
};

use purrdf_ed25519::SigningKey;
use purrdf_testkit::TempDir;
use raf_rdf::{Builder, TermValue, vocab};

use super::{
    assert_refused, commit_statement, digest,
    persistence::{self, BASE, CHANGED, PACKET},
    put,
    reader_evidence::payload,
    row, text,
};

const SPECIMEN: &str = "old-packet.bin";
const READER: &str = "persist_fixture::read_old";
const POLICY: &str = "persisted.nt";
const DESCRIPTOR: &str = "schema.bin";
const ENTRY: (&str, &str, &str, &str) = (
    "packet",
    "persist-fixture",
    "persist_fixture::Packet",
    PACKET,
);
const MIRROR: (&str, &str, &str, &str) = (
    "mirror",
    "persist-fixture",
    "persist_fixture::Mirror",
    PACKET,
);

struct Fixture {
    repo: TempDir,
    descriptor_bytes: [&'static [u8]; 2],
    base: String,
    current: String,
    artifact: String,
    key: SigningKey,
    sample: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        Self::with_descriptors(BASE, CHANGED)
    }

    fn with_descriptors(base_descriptor: &'static [u8], current_descriptor: &'static [u8]) -> Self {
        let repo = persistence::repository();
        let root = repo.path();
        let sample = payload(17);
        fs::write(root.join(SPECIMEN), &sample).unwrap();
        fs::write(root.join("alias.bin"), BASE).unwrap();
        let mut base_policy = persistence::policy(&[ENTRY, MIRROR]);
        for row in ["#packet", "#mirror"] {
            text(&mut base_policy, row, "specimenPath", SPECIMEN);
            text(&mut base_policy, row, "readerEntry", READER);
        }
        let base = persistence::commit(root, base_descriptor, base_policy);
        fs::write(root.join(SPECIMEN), b"not the committed old sample").unwrap();
        let mut current_policy = persistence::policy(&[ENTRY, MIRROR]);
        for row in ["#packet", "#mirror"] {
            text(&mut current_policy, row, "readerEntry", READER);
        }
        let current = persistence::commit(root, current_descriptor, current_policy);
        Self {
            repo,
            descriptor_bytes: [base_descriptor, current_descriptor],
            base,
            current,
            sample,
            key: SigningKey::from_bytes(&[23; 32]),
            artifact: digest(b"expected fixture reader artifact"),
        }
    }

    fn signed(
        &self,
        shape_wide: bool,
        scope_override: Option<(&str, &str)>,
        samples: &[(&str, &[u8])],
    ) -> String {
        let mut graph = Builder::new();
        let class = if shape_wide {
            "CapnpReaderAdmissionV1"
        } else {
            "CapnpReaderEvidenceV1"
        };
        put(
            &mut graph,
            "",
            vocab::TYPE,
            TermValue::iri(format!("{}{class}", vocab::REPORT)),
        );
        for (key, value) in [
            ("baseCommit", self.base.as_str()),
            ("currentCommit", self.current.as_str()),
            ("rootType", PACKET),
            ("readerEntry", READER),
            ("readerSha256", self.artifact.as_str()),
        ] {
            text(&mut graph, "", key, value);
        }
        if shape_wide {
            let [base_digest, current_digest] = self.descriptor_bytes.map(digest);
            for (key, value) in [
                ("policyFile", POLICY),
                ("declaration", ENTRY.0),
                ("baseDescriptorFile", DESCRIPTOR),
                ("baseDescriptorSha256", &base_digest),
                ("currentDescriptorFile", DESCRIPTOR),
                ("currentDescriptorSha256", &current_digest),
                ("comparisonProfile", "strict-schema-history-v1"),
                ("requiredAnnotation", "none"),
            ] {
                let value = scope_override
                    .filter(|(field, _)| *field == key)
                    .map_or(value, |(_, value)| value);
                text(&mut graph, "", key, value);
            }
            if let Some(("pairedPresenceAnnotation", value)) = scope_override {
                text(&mut graph, "", "pairedPresenceAnnotation", value);
            }
            for (key, value) in [
                ("acceptsOldShape", true),
                ("acceptsSpecimens", true),
                ("captureVerified", false),
            ] {
                let value = scope_override
                    .filter(|(field, _)| *field == key)
                    .map_or(value, |(_, value)| value == "true");
                put(
                    &mut graph,
                    "",
                    &format!("{}{key}", vocab::REPORT),
                    TermValue::boolean(value),
                );
            }
        } else {
            put(
                &mut graph,
                "",
                &format!("{}accepted", vocab::REPORT),
                TermValue::boolean(true),
            );
        }
        for (index, &(path, bytes)) in samples.iter().enumerate() {
            let id = format!("#payload-{index}");
            row(&mut graph, &id, "payload", "CapnpAcceptedPayload");
            text(&mut graph, &id, "file", path);
            text(&mut graph, &id, "sha256", &digest(bytes));
        }
        commit_statement(self.repo.path(), graph, &self.key)
    }

    fn command(
        &self,
        declaration: &str,
        current: &str,
        descriptor: &str,
        evidence: Option<&str>,
        options: &[&str],
    ) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-census"));
        command.current_dir(self.repo.path()).args([
            "capnp-reader-admission",
            "--base",
            &self.base,
            "--current",
            current,
            "--base-descriptor",
            descriptor,
            "--current-descriptor",
            DESCRIPTOR,
            "--policy",
            POLICY,
            "--declaration",
            declaration,
        ]);
        command.args(options);
        if let Some(evidence) = evidence {
            let public = purrdf_hash::hex::encode(&self.key.verifying_key().to_bytes());
            command.args([
                "--evidence-commit",
                evidence,
                "--statement",
                "capture.nt",
                "--signature",
                "capture.sig",
                "--key",
                &public,
                "--reader-sha256",
                &self.artifact,
            ]);
        }
        command.output().unwrap()
    }

    fn run(&self, evidence: Option<&str>) -> Output {
        self.command(ENTRY.0, &self.current, DESCRIPTOR, evidence, &[])
    }
}

/// An admitted reader changes only the separate verdict for the named row.
#[test]
fn capnp_reader_admission_preserves_strict_findings() {
    let fixture = Fixture::new();
    let evidence = fixture.signed(true, None, &[(SPECIMEN, &fixture.sample)]);
    let strict = persistence::check(fixture.repo.path(), &fixture.base, &fixture.current);
    assert_eq!(strict.status.code(), Some(1), "{strict:?}");
    assert!(String::from_utf8_lossy(&strict.stdout).contains("title: member renamed to heading"));
    let uncovered = fixture.run(None);
    assert_eq!(uncovered.status.code(), Some(1), "{uncovered:?}");
    assert_eq!(uncovered.stderr, b"");
    assert_eq!(
        [
            strict.stdout.as_slice(),
            b"strict findings remain without admitted reader coverage\n"
        ]
        .concat(),
        uncovered.stdout
    );
    let admitted = fixture.run(Some(&evidence));
    assert!(admitted.status.success(), "{admitted:?}");
    assert_eq!(admitted.stderr, b"");
    assert_eq!(
        [
            strict.stdout.as_slice(),
            b"admitted signed old-shape reader claim for packet\nold-packet.bin\n"
        ]
        .concat(),
        admitted.stdout
    );
}

/// A signed reader admits only the paired presence annotation selected for its
/// comparison and keeps the strict pairing finding visible.
#[test]
fn capnp_reader_admission_binds_paired_presence_annotation() {
    const MARKER: &str = "0xbfff7293108cab62";
    let fixture = Fixture::with_descriptors(
        include_bytes!("../fixtures/capnp/paired-base.bin"),
        include_bytes!("../fixtures/capnp/paired-current.bin"),
    );
    let samples = [(SPECIMEN, fixture.sample.as_slice())];
    let options = ["--paired-presence-annotation", MARKER];
    let run = |evidence: &str, options: &[&str]| {
        fixture.command(
            ENTRY.0,
            &fixture.current,
            DESCRIPTOR,
            Some(evidence),
            options,
        )
    };
    let matching = fixture.signed(true, Some(("pairedPresenceAnnotation", MARKER)), &samples);
    let admitted = run(&matching, &options);
    assert!(admitted.status.success(), "{admitted:?}");
    assert_eq!(admitted.stderr, b"");
    assert_eq!(
        admitted.stdout,
        concat!(
            "Cap'n Proto history for 0xf1804b9b18ae6505.profile: pointer field became paired with profileSince\n",
            "admitted signed old-shape reader claim for packet\nold-packet.bin\n",
        )
        .as_bytes()
    );
    let omitted = fixture.signed(true, None, &samples);
    let different = fixture.signed(
        true,
        Some(("pairedPresenceAnnotation", "0xbfff7293108cab63")),
        &samples,
    );
    for (evidence, options, diagnostic) in [
        (&omitted, options.as_slice(), "pairedPresenceAnnotation"),
        (
            &different,
            options.as_slice(),
            "signed pairedPresenceAnnotation differs",
        ),
        (
            &matching,
            [].as_slice(),
            "signed pairedPresenceAnnotation must be absent",
        ),
    ] {
        assert_refused(run(evidence, options), diagnostic);
    }
}

/// The same schema ID and bytes cannot transfer an assertion across row or scope.
#[test]
fn capnp_reader_admission_refuses_stale_or_different_claim_scope() {
    let fixture = Fixture::new();
    let samples = [(SPECIMEN, fixture.sample.as_slice())];
    let evidence = fixture.signed(true, None, &samples);
    for (row, current, descriptor, diagnostic) in [
        (
            "mirror",
            fixture.current.as_str(),
            DESCRIPTOR,
            "signed declaration differs",
        ),
        (
            ENTRY.0,
            fixture.base.as_str(),
            DESCRIPTOR,
            "signed currentCommit differs",
        ),
        (
            ENTRY.0,
            fixture.current.as_str(),
            "alias.bin",
            "signed baseDescriptorFile differs",
        ),
    ] {
        assert_refused(
            fixture.command(row, current, descriptor, Some(&evidence), &[]),
            diagnostic,
        );
    }
    let wrong_digest = digest(b"not the compared descriptor");
    for (field, value) in [
        ("currentDescriptorSha256", wrong_digest.as_str()),
        ("comparisonProfile", "different-profile"),
        ("requiredAnnotation", "0xe91447cf636b013a"),
    ] {
        let evidence = fixture.signed(true, Some((field, value)), &samples);
        assert_refused(
            fixture.run(Some(&evidence)),
            &format!("signed {field} differs"),
        );
    }
    let specimen_only = fixture.signed(false, None, &samples);
    assert_refused(fixture.run(Some(&specimen_only)), "CapnpReaderAdmissionV1");
    for (field, value, diagnostic) in [
        (
            "acceptsOldShape",
            "false",
            "signed acceptsOldShape must be true",
        ),
        (
            "acceptsSpecimens",
            "false",
            "signed acceptsSpecimens must be true",
        ),
        (
            "captureVerified",
            "true",
            "signed captureVerified must be false",
        ),
    ] {
        let evidence = fixture.signed(true, Some((field, value)), &samples);
        assert_refused(fixture.run(Some(&evidence)), diagnostic);
    }
}

/// A shape-wide assertion still requires the base policy's exact signed specimens.
#[test]
fn capnp_reader_admission_verifies_exact_specimens_and_preserves_removal() {
    let fixture = Fixture::new();
    let wrong_bytes = payload(31);
    for (path, bytes, diagnostic) in [
        (
            "different.bin",
            fixture.sample.as_slice(),
            "signed payload paths differ",
        ),
        (
            SPECIMEN,
            wrong_bytes.as_slice(),
            "SHA-256 differs from the signed payload digest",
        ),
    ] {
        let evidence = fixture.signed(true, None, &[(path, bytes)]);
        assert_refused(fixture.run(Some(&evidence)), diagnostic);
    }
    let evidence = fixture.signed(true, None, &[(SPECIMEN, &fixture.sample)]);
    let rebound = (ENTRY.0, ENTRY.1, ENTRY.2, "0xe70e425d8d63c329");
    for (rows, diagnostic) in [
        (&[MIRROR][..], "removed protection"),
        (&[MIRROR, rebound][..], "root changed from"),
    ] {
        let changed = persistence::commit(fixture.repo.path(), CHANGED, persistence::policy(rows));
        let output = fixture.command(ENTRY.0, &changed, DESCRIPTOR, Some(&evidence), &[]);
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert_eq!(output.stderr, b"");
        assert!(String::from_utf8_lossy(&output.stdout).contains(diagnostic));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("admitted signed"));
    }
}
