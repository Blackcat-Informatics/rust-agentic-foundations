// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The production Census gate over the selected repository snapshot.

#[path = "../examples/support/census.rs"]
mod policy;

use raf_census::{CensusOptions, LEDGER_FILE, Ledger, MAX_LEDGER_BYTES};
use raf_rdf::{Builder, Format, Graph, TermValue, vocab};
use raf_source::{DiskTree, EntryKind, MemoryTree, RelPath, Tree};

#[test]
fn repository_census_is_clean_and_duplicate_growth_is_refused() {
    let root = rust_agentic_foundations::gate_root();
    let tree = DiskTree::new(&root).expect("open the selected gate snapshot");
    let policy = tree
        .read(
            &RelPath::new(LEDGER_FILE).expect("the native ledger path"),
            u64::try_from(MAX_LEDGER_BYTES).expect("the ledger limit fits a source read"),
        )
        .expect("read the selected snapshot's ledger");
    let ledger = Ledger::from_turtle(&policy).expect("validate the native policy");
    assert_eq!(ledger.roots(), [".", "fuzz"]);
    let defaults = CensusOptions::default();
    assert_eq!(ledger.options().min_tokens, defaults.min_tokens);
    assert_eq!(ledger.options().max_file_bytes, defaults.max_file_bytes);
    let gate = policy::gate(&root);
    let report = policy::locked_report(&gate, &tree)
        .expect("evaluate the real repository through the locked production gate");
    assert!(report.is_clean(), "{}", report.findings().join("\n"));
    assert!(!report.census().units().is_empty(), "the census is empty");
    for package in ["rust-agentic-foundations", "raf-source-fuzz"] {
        assert!(
            report
                .census()
                .units()
                .iter()
                .any(|unit| unit.package == package),
            "the declared roots did not census {package}"
        );
    }

    // Both fixtures use the same policy and the production empty-debt ratchet.
    // Neither test evaluation writes or increases a debt record.
    let structural = structural_policy(&policy);
    let fixture = |source: &str| {
        MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n",
            ),
            ("src/lib.rs", source),
            (
                "fuzz/Cargo.toml",
                "[package]\nname='probe'\nversion='1.0.0'\nedition='2024'\n[workspace]\n",
            ),
            ("fuzz/src/lib.rs", "pub fn separate() {}"),
            (LEDGER_FILE, structural.as_str()),
        ])
    };
    let source = "pub fn original(values: &[u64]) -> u64 { let mut sum = 0; for value in values { sum += value * 3; } sum }";
    let valid = fixture(source);
    let gate = policy::gate(".");
    let before = gate.check_tree(&valid).expect("evaluate the valid fixture");
    assert!(before.is_clean(), "{:?}", before.findings());
    let duplicate = format!(
        "{source}\npub fn repeated(inputs: &[u64]) -> u64 {{ let mut total = 0; for input in inputs {{ total += input * 3; }} total }}"
    );
    let invalid = fixture(&duplicate);
    let after = gate
        .check_tree(&invalid)
        .expect("evaluate duplicate growth");
    assert!(
        !after.debt().groups().is_empty(),
        "the duplicate was not found"
    );
    assert!(
        after
            .findings()
            .iter()
            .any(|finding| finding.contains("growth cannot be recorded")),
        "{:?}",
        after.findings()
    );
    assert_eq!(
        invalid
            .read(&RelPath::new("src/lib.rs").unwrap(), u64::MAX)
            .unwrap(),
        duplicate
    );
    assert!(
        invalid
            .entry(&RelPath::new(raf_census::DEBT_FILE).unwrap())
            .unwrap()
            .is_none()
    );
}

/// The repository policy without its job rows: the roots and limits that a
/// structural check can judge on an in-memory workspace. Rows that name a
/// preferred provider need the locked evaluation of the real repository.
fn structural_policy(policy: &str) -> String {
    let graph = Graph::from_text(policy, Format::Turtle).expect("read the native policy");
    let mut rows = Builder::from_graph(&graph);
    let job = TermValue::iri(format!("{}job", vocab::REPORT));
    for triple in graph.assertions() {
        if triple.subject != TermValue::iri("") || triple.predicate == job {
            assert!(rows.remove(triple.clone()).expect("remove a job row"));
        }
    }
    rows.finish()
        .expect("finish the structural policy")
        .to_text(Format::Turtle)
        .expect("write the structural policy")
}

/// Every UTF-8 file of the gate snapshot that source loading can reach, so a
/// planted edit changes only an in-memory copy of the real repository.
fn snapshot(tree: &DiskTree, directory: &RelPath, files: &mut Vec<(String, String)>) {
    for entry in tree.list(directory).expect("list the gate snapshot") {
        let path = directory.join(&entry.name).expect("join a snapshot path");
        match entry.kind {
            EntryKind::Dir
                if !matches!(
                    entry.name.as_str(),
                    ".git" | ".worktrees" | ".stage" | "target" | "graphify-out"
                ) =>
            {
                snapshot(tree, &path, files);
            }
            EntryKind::File => {
                if let Ok(text) = tree.read(&path, 16 << 20) {
                    files.push((path.to_string(), text));
                }
            }
            _ => {}
        }
    }
}

const DIGEST: &str = "crates/raf-workplan/src/digest.rs";

/// The repository with `edit` applied to the digest module.
fn planted(edit: impl FnOnce(String) -> String) -> MemoryTree {
    let disk = DiskTree::new(rust_agentic_foundations::gate_root()).expect("open the snapshot");
    let mut files = Vec::new();
    snapshot(&disk, &RelPath::root(), &mut files);
    let target = files
        .iter_mut()
        .find(|(path, _)| path == DIGEST)
        .expect("the digest module is in the snapshot");
    target.1 = edit(std::mem::take(&mut target.1));
    files.into_iter().collect()
}

#[test]
fn a_competing_implementation_of_purrdf_functionality_fails_the_repository_gate() {
    // Three competitors beside an accepted thin adapter. None needs a PurRDF
    // import to be found, and an import or a discarded provider call beside
    // one does not excuse it: the detectors judge what the code computes.
    let competitors = "
pub(crate) fn thin_adapter(bytes: &[u8]) -> String {
    blake3_hex(bytes)
}

pub(crate) fn local_table(value: u8) -> char {
    b\"0123456789abcdef\"[usize::from(value & 15)] as char
}

pub(crate) fn imported_beside_a_table(value: u8) -> char {
    let _ = purrdf_hash::hex::encode(&[value]);
    b\"0123456789abcdef\"[usize::from(value >> 4)] as char
}

pub(crate) fn decorative_call_beside_an_initial_state() -> u32 {
    let _ = purrdf_hash::blake3::hash(b\"decoration\");
    0x6A09_E667
}
";
    let tree = planted(|text| format!("{text}{competitors}"));
    let source = tree.read(&RelPath::new(DIGEST).unwrap(), u64::MAX).unwrap();
    let line_of = |needle: &str| {
        source
            .lines()
            .position(|line| line.contains(needle))
            .expect("the planted line exists")
            + 1
    };
    let gate = policy::gate(".");
    let report = policy::locked_report(&gate, &tree).expect("evaluate the planted repository");
    let findings = report.findings().join("\n");
    assert!(!report.is_clean(), "the planted competitors were accepted");
    for (needle, rule) in [
        ("pub(crate) fn local_table", "rule hex-table"),
        ("pub(crate) fn imported_beside_a_table", "rule hex-table"),
    ] {
        let expected = format!(
            "{DIGEST}:{}: job \"purrdf-digest\", {rule}:",
            line_of(needle)
        );
        assert!(
            findings.contains(&expected),
            "{expected} missing from:\n{findings}"
        );
    }
    let constant = format!(
        "{DIGEST}:{}: job \"purrdf-digest\": raf_workplan::digest::decorative_call_beside_an_initial_state is a COPY",
        line_of("pub(crate) fn decorative_call_beside_an_initial_state")
    );
    assert!(
        findings.contains(&constant),
        "{constant} missing from:\n{findings}"
    );
    let thin = format!("{DIGEST}:{}", line_of("pub(crate) fn thin_adapter"));
    assert!(
        !findings.contains(&thin),
        "the thin adapter was refused:\n{findings}"
    );
    // The recorded debt is part of this tree and holds none of it: the planted
    // symbols are failures of the gate and appear in no adoptable debt group.
    let debt = format!("{:?}", report.debt());
    for symbol in [
        "local_table",
        "imported_beside_a_table",
        "decorative_call_beside_an_initial_state",
        "thin_adapter",
    ] {
        assert!(!debt.contains(symbol), "{symbol} entered the debt: {debt}");
    }
}

#[test]
fn a_decorative_provider_call_cannot_stand_in_for_the_delegation() {
    let tree = planted(|text| {
        text.replace(
            "    purrdf_hash::hex::encode(purrdf_hash::blake3::Hash::as_bytes(&digest))",
            "    let _ = purrdf_hash::hex::encode(purrdf_hash::blake3::Hash::as_bytes(&digest));\n    String::new()",
        )
    });
    let gate = policy::gate(".");
    let refusal = policy::locked_report(&gate, &tree)
        .expect_err("a call whose result is discarded is not delegation");
    assert!(
        refusal.to_string().contains("preferred provider"),
        "{refusal}"
    );
}

#[test]
fn an_oversized_cached_archive_is_refused_before_it_is_held_beside_a_valid_one() {
    let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let index = scratch.path().join("index");
    std::fs::create_dir(&index).unwrap();
    std::fs::write(index.join("small-1.0.0.crate"), b"four").unwrap();
    // A sparse file one byte over the budget: no large allocation is made.
    let oversized = std::fs::File::create(index.join("large-1.0.0.crate")).unwrap();
    oversized.set_len(9).unwrap();
    assert_eq!(
        policy::archive(scratch.path(), "small", "1.0.0", 8).unwrap(),
        b"four"
    );
    let refused = policy::archive(scratch.path(), "large", "1.0.0", 8)
        .expect_err("an archive over the budget must be refused");
    let message = refused.to_string();
    assert!(
        message.contains("large 1.0.0") && message.contains("8 byte archive budget"),
        "{message}"
    );
}
