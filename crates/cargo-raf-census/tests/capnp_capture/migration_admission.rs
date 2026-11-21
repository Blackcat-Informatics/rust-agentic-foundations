// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The shipped migration command admits exact committed pairs under a signed claim.

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

const INPUTS: [&str; 2] = ["old-a.bin", "old-b.bin"];
const OUTPUTS: [&str; 2] = ["new-a.bin", "new -> b.bin"];
const MIGRATOR: &str = "persist_fixture::migrate_packets";
const ENTRY: (&str, &str, &str, &str) = (
    "packet",
    "persist-fixture",
    "persist_fixture::Packet",
    PACKET,
);

struct Fixture {
    repo: TempDir,
    base: String,
    current: String,
    key: SigningKey,
    artifact: String,
    inputs: [Vec<u8>; 2],
    outputs: [Vec<u8>; 2],
}

impl Fixture {
    fn new() -> Self {
        let repo = persistence::repository();
        let root = repo.path();
        let inputs = [payload(17), payload(27)];
        let outputs = [payload(31), payload(41)];
        let mut old_policy = persistence::policy(&[ENTRY]);
        for (path, bytes) in INPUTS.into_iter().zip(&inputs) {
            fs::write(root.join(path), bytes).unwrap();
            text(&mut old_policy, "#packet", "specimenPath", path);
        }
        let base = persistence::commit(root, BASE, old_policy);
        for path in INPUTS.into_iter().chain(OUTPUTS) {
            fs::write(root.join(path), b"current decoy").unwrap();
        }
        let mut new_policy = persistence::policy(&[ENTRY]);
        text(&mut new_policy, "#packet", "migrationEntry", MIGRATOR);
        let current = persistence::commit(root, CHANGED, new_policy);
        Self {
            repo,
            base,
            current,
            key: SigningKey::from_bytes(&[39; 32]),
            artifact: digest(b"expected fixture migration artifact"),
            inputs,
            outputs,
        }
    }

    fn pairs(&self) -> [Pair<'_>; 2] {
        std::array::from_fn(|index| Pair {
            input: INPUTS[index],
            input_bytes: &self.inputs[index],
            output: OUTPUTS[index],
            output_bytes: &self.outputs[index],
        })
    }

    fn publish(
        &self,
        pairs: &[Pair<'_>],
        claim_override: Option<(&str, Option<bool>)>,
        scope_override: Option<(&str, &str)>,
        output_override: Option<(&str, &[u8])>,
    ) -> String {
        let mut graph = Builder::new();
        put(
            &mut graph,
            "",
            vocab::TYPE,
            TermValue::iri(format!("{}CapnpMigrationAdmissionV1", vocab::REPORT)),
        );
        for (field, value) in [
            ("baseCommit", self.base.as_str()),
            ("currentCommit", self.current.as_str()),
            ("rootType", PACKET),
            ("policyFile", "persisted.nt"),
            ("declaration", ENTRY.0),
            ("baseDescriptorFile", "schema.bin"),
            ("baseDescriptorSha256", &digest(BASE)),
            ("currentDescriptorFile", "schema.bin"),
            ("currentDescriptorSha256", &digest(CHANGED)),
            ("comparisonProfile", "strict-schema-history-v1"),
            ("requiredAnnotation", "none"),
            ("migrationEntry", MIGRATOR),
            ("migrationSha256", self.artifact.as_str()),
        ] {
            let value = scope_override
                .filter(|(key, _)| *key == field)
                .map_or(value, |(_, value)| value);
            text(&mut graph, "", field, value);
        }
        for (field, value) in [
            ("acceptsOldShape", true),
            ("producedOutputs", true),
            ("outputsValid", true),
            ("captureVerified", false),
        ] {
            let Some(value) = claim_override
                .filter(|(key, _)| *key == field)
                .map_or(Some(value), |(_, value)| value)
            else {
                continue;
            };
            put(
                &mut graph,
                "",
                &format!("{}{field}", vocab::REPORT),
                TermValue::boolean(value),
            );
        }
        for (index, pair) in pairs.iter().enumerate() {
            let id = format!("#pair-{index}");
            row(
                &mut graph,
                &id,
                "specimenPair",
                "CapnpMigrationSpecimenPair",
            );
            for (field, value) in [
                ("inputFile", pair.input),
                ("inputSha256", &digest(pair.input_bytes)),
                ("outputFile", pair.output),
                ("outputSha256", &digest(pair.output_bytes)),
            ] {
                text(&mut graph, &id, field, value);
            }
            let committed = output_override
                .filter(|(path, _)| *path == pair.output)
                .map_or(pair.output_bytes, |(_, bytes)| bytes);
            fs::write(self.repo.path().join(pair.output), committed).unwrap();
        }
        commit_statement(self.repo.path(), graph, &self.key)
    }

    fn run(&self, evidence: Option<&str>) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-raf-census"));
        command.current_dir(self.repo.path()).args([
            "capnp-migration-admission",
            "--base",
            &self.base,
            "--current",
            &self.current,
            "--base-descriptor",
            "schema.bin",
            "--current-descriptor",
            "schema.bin",
            "--policy",
            "persisted.nt",
            "--declaration",
            ENTRY.0,
        ]);
        if let Some(evidence) = evidence {
            command.args([
                "--evidence-commit",
                evidence,
                "--statement",
                "capture.nt",
                "--signature",
                "capture.sig",
                "--key",
                &purrdf_hash::hex::encode(&self.key.verifying_key().to_bytes()),
                "--migration-sha256",
                &self.artifact,
            ]);
        }
        command.output().unwrap()
    }
}

#[derive(Clone, Copy)]
struct Pair<'a> {
    input: &'a str,
    input_bytes: &'a [u8],
    output: &'a str,
    output_bytes: &'a [u8],
}

#[test]
fn capnp_migration_admits_committed_pairs_and_preserves_strict_findings() {
    // The base supplies the input, the evidence revision supplies the output,
    // and the strict-history finding survives admission byte for byte.
    let fixture = Fixture::new();
    let evidence = fixture.publish(&fixture.pairs(), None, None, None);
    fs::write(fixture.repo.path().join(OUTPUTS[0]), b"checkout decoy").unwrap();
    let strict = persistence::check(fixture.repo.path(), &fixture.base, &fixture.current);
    assert_eq!(strict.status.code(), Some(1), "{strict:?}");
    assert!(String::from_utf8_lossy(&strict.stdout).contains("title: member renamed to heading"));
    let uncovered = fixture.run(None);
    assert_eq!(uncovered.status.code(), Some(1), "{uncovered:?}");
    assert_eq!(
        [
            strict.stdout.as_slice(),
            b"strict findings remain without admitted migration coverage\n",
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
            b"admitted signed migration claim for packet\nold-a.bin\tnew-a.bin\nold-b.bin\tnew -> b.bin\n",
        ]
        .concat(),
        admitted.stdout
    );
}

#[test]
fn capnp_migration_requires_exact_scope_and_output_validity() {
    // An authenticated statement must affirm validity and cover each input
    // selected by the base policy for this exact comparison.
    let fixture = Fixture::new();
    let pairs = fixture.pairs();
    let duplicate = [pairs[0], pairs[0], pairs[1]];
    let extra = [
        pairs[0],
        pairs[1],
        Pair {
            input: "unselected.bin",
            ..pairs[0]
        },
    ];
    for (pairs, claim, scope, message) in [
        (
            &pairs[..],
            Some(("outputsValid", Some(false))),
            None,
            "outputsValid must be true",
        ),
        (
            &pairs[..],
            Some(("outputsValid", None)),
            None,
            "outputsValid",
        ),
        (
            &pairs[..],
            None,
            Some(("currentDescriptorSha256", &digest(BASE)[..])),
            "currentDescriptorSha256 differs",
        ),
        (&pairs[..1], None, None, "exact base specimen set"),
        (&duplicate, None, None, "repeats input path old-a.bin"),
        (&extra, None, None, "exact base specimen set"),
    ] {
        let evidence = fixture.publish(pairs, claim, scope, None);
        assert_refused(fixture.run(Some(&evidence)), message);
    }
}

#[test]
fn capnp_migration_refuses_changed_or_invalid_pair_blobs() {
    // Both roles require the signed identity and the raw canonical framing.
    // Multiple inputs may converge on one output with the same signed digest.
    let fixture = Fixture::new();
    let [first, second] = fixture.pairs();
    let shared = Pair {
        output: first.output,
        output_bytes: first.output_bytes,
        ..second
    };
    let evidence = fixture.publish(&[first, shared], None, None, None);
    let admitted = fixture.run(Some(&evidence));
    assert!(admitted.status.success(), "{admitted:?}");
    assert!(String::from_utf8_lossy(&admitted.stdout).contains("old-b.bin\tnew-a.bin"));

    let conflicting = Pair {
        output_bytes: second.output_bytes,
        ..shared
    };
    let evidence = fixture.publish(&[first, conflicting], None, None, None);
    assert_refused(
        fixture.run(Some(&evidence)),
        "conflicting digests for output new-a.bin",
    );

    let wrong_input = Pair {
        input_bytes: second.input_bytes,
        ..first
    };
    let evidence = fixture.publish(&[wrong_input, second], None, None, None);
    assert_refused(
        fixture.run(Some(&evidence)),
        "base input old-a.bin: SHA-256 differs",
    );

    let evidence = fixture.publish(
        &[first, second],
        None,
        None,
        Some((first.output, second.output_bytes)),
    );
    assert_refused(
        fixture.run(Some(&evidence)),
        "evidence output new-a.bin: SHA-256 differs",
    );

    let malformed = Pair {
        output_bytes: b"malformed",
        ..first
    };
    let evidence = fixture.publish(&[malformed, second], None, None, None);
    assert_refused(
        fixture.run(Some(&evidence)),
        "evidence output new-a.bin: expected a canonical raw",
    );
}
