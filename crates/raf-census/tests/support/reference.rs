// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! An owned path dependency with inherited metadata and a renamed declaration.

use raf_source::MemoryTree;
#[path = "reference_ledger.rs"]
mod reference_ledger;

pub(crate) fn fixture(consumer: &str, reference: &str) -> MemoryTree {
    let ledger = reference_ledger::ledger("helper", "helper::api", "arithmetic");
    MemoryTree::from([
        (
            "Cargo.toml",
            "[workspace]\nmembers=['app','helper']\n[workspace.package]\nversion='1.0.0'\n[workspace.dependencies]\nrenamed={package='helper',path='helper',version='1'}\n",
        ),
        (
            "app/Cargo.toml",
            "[package]\nname='app'\nversion.workspace=true\nedition='2024'\n[dependencies]\nrenamed.workspace=true\n",
        ),
        ("app/src/lib.rs", consumer),
        (
            "helper/Cargo.toml",
            "[package]\nname='helper'\nversion.workspace=true\nedition='2024'\n",
        ),
        ("helper/src/lib.rs", reference),
        (
            "Cargo.lock",
            "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['helper']\n[[package]]\nname='helper'\nversion='1.0.0'\n",
        ),
        ("helpers-ledger.ttl", ledger.as_str()),
    ])
}
