// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Registered Rule and Extension outcomes over an owned source fixture.

#[path = "support/ledger.rs"]
mod policy;
#[path = "support/registration.rs"]
mod registration;

use raf_census::RegisteredCheck;
use raf_source::MemoryTree;
use std::{error::Error, io::Write as _};

fn fixture(source: &str, owner: &str) -> Result<MemoryTree, Box<dyn Error>> {
    Ok(MemoryTree::try_from_iter([
        (
            "Cargo.toml",
            "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n".to_owned(),
        ),
        ("src/lib.rs", source.to_owned()),
        ("helpers-ledger.ttl", policy::ledger(owner)),
    ])?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let gate = registration::registered_gate(".");
    let source = "/// Home.\npub fn home()->u32{1}\n";
    let accepted = gate.check_tree(&fixture(source, "demo")?);
    let response = RegisteredCheck::from_check(&accepted)?;
    assert_eq!(response.exit_code(), 0);
    let text = response.to_ntriples()?;
    assert_eq!(RegisteredCheck::from_ntriples(&text)?.findings().len(), 0);
    let planted = format!("{source}pub fn local_copy()->u32{{2}}\n");
    let refused = gate.check_tree(&fixture(&planted, "demo")?)?;
    assert!(
        refused
            .findings()
            .iter()
            .any(|message| message.contains("rule no-local-copy: use home")),
        "{:?}",
        refused.findings()
    );
    assert!(gate.check_tree(&fixture(source, "other")?).is_err());
    std::io::stdout().lock().write_all(text.as_bytes())?;
    Ok(())
}
