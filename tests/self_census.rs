// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The production Census gate over the selected repository snapshot.

use raf_census::{CensusOptions, Gate, LEDGER_FILE, Ledger, MAX_LEDGER_BYTES};
use raf_source::{DiskTree, MemoryTree, RelPath, Tree};

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
    let report = Gate::workspace(&root)
        .check_tree(&tree)
        .expect("evaluate the real repository through the production gate");
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
            (LEDGER_FILE, policy.as_str()),
        ])
    };
    let source = "pub fn original(values: &[u64]) -> u64 { let mut sum = 0; for value in values { sum += value * 3; } sum }";
    let valid = fixture(source);
    let gate = Gate::workspace(".");
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
