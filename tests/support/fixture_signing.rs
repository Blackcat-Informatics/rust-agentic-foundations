// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Genuine ephemeral Git signing for fixtures transported through native guards.

use std::path::PathBuf;
use std::process::Command;

pub(crate) fn after_git(args: &[&str], git: impl Fn(&[&str]) -> String) {
    if args.first() == Some(&"init") {
        initialize(&git);
    }
    if args.first() == Some(&"commit") {
        git(&["verify-commit", "HEAD"]);
    }
}

fn initialize(git: &impl Fn(&[&str]) -> String) {
    let observed =
        raf_workplan::git::GitVersion::parse(&git(&["--version"])).expect("Git states its version");
    let minimum = raf_workplan::git::GitVersion::parse("git version 2.34.0")
        .expect("fixed fixture signing minimum");
    assert!(
        observed >= minimum,
        "native signed fixtures require Git 2.34 or newer"
    );
    let admin = PathBuf::from(git(&["rev-parse", "--absolute-git-dir"]).trim_end_matches('\n'));
    let key = admin.join("raf-fixture-ed25519");
    let generated = Command::new("ssh-keygen")
        .args([
            "-q",
            "-t",
            "ed25519",
            "-N",
            "",
            "-C",
            "fixture@example.invalid",
            "-f",
        ])
        .arg(&key)
        .output()
        .expect("native signed fixtures require OpenSSH ssh-keygen on PATH");
    assert!(
        generated.status.success(),
        "cannot create ephemeral fixture signer: {}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let public = std::fs::read_to_string(key.with_extension("pub")).expect("fixture public key");
    let signers = admin.join("raf-fixture-allowed-signers");
    std::fs::write(
        &signers,
        format!("fixture@example.invalid namespaces=\"git\" {public}"),
    )
    .expect("fixture allowed signer");
    for (name, value) in [
        ("gpg.format", "ssh"),
        ("user.signingkey", key.to_str().expect("fixture key path")),
        (
            "gpg.ssh.allowedSignersFile",
            signers.to_str().expect("fixture signer path"),
        ),
        ("commit.gpgsign", "true"),
    ] {
        git(&["config", "--local", name, value]);
    }
}
