// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Repository gates that used to be shell recipes.
//!
//! Three gates run over the git index, read under `RAF_GATE_ROOT` (default: the
//! package directory):
//!
//! * the Mulan licence texts match their pinned digest, and the Chinese
//!   licensing explanation matches the fingerprint in its back-translation
//!   record;
//! * no tracked text file carries an issue or pull request number or
//!   development-process wording;
//! * `CITATION.cff` and `[workspace.package]` carry the same version;
//! * the fuzz harness workspace carries the same `[workspace.lints]` tables as
//!   the root, and every workspace manifest inherits them;
//! * the fuzz harness `deny.toml` is the root `deny.toml` plus its one
//!   crate-scoped licence exception.
//!
//! Each gate is a pure function over text or bytes, and each has a named
//! negative fixture below that breaks exactly one thing. Test names start with
//! `licenses_`, `issue_refs_`, `citation_`, `lints_` or `deny_`, which names the gate a test
//! belongs to; `make licenses` and `make issue-refs` run the whole binary. Only
//! the standard library is used.

mod support;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use support::{banned_in_line, decode_text, gate_root, read_tracked, tracked_paths};

/// Digest both Mulan texts must match.
const MULAN_SHA256: &str = "eb7a1d713eb919b146787629e22e4c975cb701f529a65d4d7e0fcd417558bf1c";
const MULAN_FILES: [&str; 2] = ["LICENSE-MULAN", "LICENSES/MulanPSL-2.0.txt"];
const BACKTRANSLATION: &str = "docs/LICENSING-BACKTRANSLATION.md";
const ZH_HANS: &str = "docs/LICENSING.zh-Hans.md";
const FINGERPRINT_LABEL: &str = "Source SHA-256";

// ---------------------------------------------------------------------------
// SHA-256 (FIPS 180-4)
// ---------------------------------------------------------------------------

const K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

const H0: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut schedule = [0u32; 64];
    for (slot, chunk) in schedule.iter_mut().zip(block.as_chunks::<4>().0) {
        *slot = u32::from_be_bytes(*chunk);
    }
    for i in 16..64 {
        let x = schedule[i - 15];
        let y = schedule[i - 2];
        let s0 = x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3);
        let s1 = y.rotate_right(17) ^ y.rotate_right(19) ^ (y >> 10);
        schedule[i] = schedule[i - 16]
            .wrapping_add(s0)
            .wrapping_add(schedule[i - 7])
            .wrapping_add(s1);
    }
    // Working variables a..h live in `v[0]..v[7]`.
    let mut v = *state;
    for (k, word) in K.iter().zip(schedule) {
        let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
        let choose = (v[4] & v[5]) ^ (!v[4] & v[6]);
        let t1 = v[7]
            .wrapping_add(s1)
            .wrapping_add(choose)
            .wrapping_add(*k)
            .wrapping_add(word);
        let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
        let majority = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
        v.rotate_right(1);
        v[0] = t1.wrapping_add(s0.wrapping_add(majority));
        v[4] = v[4].wrapping_add(t1);
    }
    for (slot, add) in state.iter_mut().zip(v) {
        *slot = slot.wrapping_add(add);
    }
}

/// The lowercase hexadecimal SHA-256 digest of `data`.
fn sha256_hex(data: &[u8]) -> String {
    let mut state = H0;
    let (blocks, remainder) = data.as_chunks::<64>();
    for block in blocks {
        compress(&mut state, block);
    }
    let mut tail = remainder.to_vec();
    tail.push(0x80);
    while tail.len() % 64 != 56 {
        tail.push(0);
    }
    let bit_length = (data.len() as u64).wrapping_mul(8);
    tail.extend_from_slice(&bit_length.to_be_bytes());
    for block in tail.as_chunks::<64>().0 {
        compress(&mut state, block);
    }
    let mut hex = String::with_capacity(64);
    for word in state {
        let _ = write!(hex, "{word:08x}");
    }
    hex
}

// ---------------------------------------------------------------------------
// Gate functions
// ---------------------------------------------------------------------------

/// A problem with one text, or `None` when its digest is the `pinned` one.
fn digest_problem(name: &str, bytes: &[u8], pinned: &str) -> Option<String> {
    let got = sha256_hex(bytes);
    (got != pinned).then(|| format!("{name}: sha256 {got} != pinned {pinned}"))
}

/// The 64-digit hash on the first line of `record` that names the source.
fn recorded_fingerprint(record: &str) -> Option<String> {
    let line = record.lines().find(|l| l.contains(FINGERPRINT_LABEL))?;
    let bytes = line.as_bytes();
    (0..bytes.len().saturating_sub(63)).find_map(|start| {
        let window = bytes.get(start..start + 64)?;
        let hex = window
            .iter()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b));
        let before_ok = start == 0 || !bytes[start - 1].is_ascii_hexdigit();
        let after_ok = bytes.get(start + 64).is_none_or(|b| !b.is_ascii_hexdigit());
        (hex && before_ok && after_ok).then(|| String::from_utf8_lossy(window).into_owned())
    })
}

/// A problem with the zh-Hans fingerprint, or `None` when it matches.
fn fingerprint_problem(record: &str, document: &[u8]) -> Option<String> {
    let got = sha256_hex(document);
    match recorded_fingerprint(record) {
        None => Some(format!(
            "{BACKTRANSLATION}: no 64-digit hash on a `{FINGERPRINT_LABEL}` line"
        )),
        Some(want) if want != got => Some(format!(
            "{ZH_HANS}: sha256 {got} != recorded {want} in {BACKTRANSLATION}"
        )),
        Some(_) => None,
    }
}

/// One `path:line: kind` entry per banned line in `text`.
fn scan_text(path: &Path, text: &str) -> Vec<String> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            banned_in_line(line).map(|kind| format!("{}:{}: {kind}", path.display(), index + 1))
        })
        .collect()
}

/// Scan every tracked text file under `root`. Returns the hits and the number
/// of text files scanned; binary files (a NUL byte) are recognised by content
/// and left out of the count. Invalid UTF-8 without a NUL byte is scanned
/// lossily.
fn scan_tree(root: &Path, paths: &[PathBuf]) -> (Vec<String>, usize) {
    let mut hits = Vec::new();
    let mut scanned = 0;
    for path in paths {
        let bytes = read_tracked(root, path);
        if let Some(text) = decode_text(&bytes) {
            scanned += 1;
            hits.extend(scan_text(path, &text));
        }
    }
    (hits, scanned)
}

/// The `version` under `[workspace.package]` in a `Cargo.toml` text.
fn workspace_version(manifest: &str) -> Option<String> {
    let mut in_section = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line == "[workspace.package]";
        } else if in_section && let Some(rest) = line.strip_prefix("version") {
            let value = rest.trim_start().strip_prefix('=')?.trim();
            return Some(value.trim_matches('"').to_string());
        }
    }
    None
}

/// The top-level `version:` value in a `CITATION.cff` text.
fn citation_version(citation: &str) -> Option<String> {
    let value = citation.lines().find_map(|l| l.strip_prefix("version:"))?;
    Some(value.trim().trim_matches(['"', '\'']).to_string())
}

/// A problem with version agreement, or `None` when the two agree.
fn version_problem(manifest: &str, citation: &str) -> Option<String> {
    match (workspace_version(manifest), citation_version(citation)) {
        (None, _) => Some("Cargo.toml: no version under [workspace.package]".to_string()),
        (_, None) => Some("CITATION.cff: no top-level version".to_string()),
        (Some(cargo), Some(cff)) if cargo != cff => Some(format!(
            "CITATION.cff: version {cff} != [workspace.package] version {cargo} in Cargo.toml"
        )),
        _ => None,
    }
}

/// The lines of every table whose header starts with `prefix`, headers
/// included, with blank lines and comments dropped.
fn tables_with_prefix(toml: &str, prefix: &str) -> Vec<String> {
    let mut in_section = false;
    let mut lines = Vec::new();
    for line in toml.lines().map(str::trim) {
        if line.starts_with('[') {
            in_section = line.starts_with(prefix);
        }
        if in_section && !line.is_empty() && !line.starts_with('#') {
            lines.push(line.to_string());
        }
    }
    lines
}

/// A problem with the lint tables of the fuzz workspace, or `None` when they
/// equal the root tables.
fn lint_tables_problem(root: &str, fuzz: &str) -> Option<String> {
    let want = tables_with_prefix(root, "[workspace.lints");
    let got = tables_with_prefix(fuzz, "[workspace.lints");
    if want.is_empty() {
        Some("Cargo.toml: no [workspace.lints] tables".to_string())
    } else if want != got {
        Some(format!(
            "fuzz/Cargo.toml: [workspace.lints] tables differ from Cargo.toml: root has {want:?}, fuzz has {got:?}"
        ))
    } else {
        None
    }
}

/// The level of lint `name` in the `[workspace.lints.rust]` table, as written
/// between its quotes, or `None` when the table does not set it as a string.
fn rust_lint_level(manifest: &str, name: &str) -> Option<String> {
    let mut in_table = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_table = line == "[workspace.lints.rust]";
        } else if in_table {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key.trim() == name {
                return value
                    .trim()
                    .strip_prefix('"')
                    .and_then(|rest| rest.strip_suffix('"'))
                    .map(str::to_string);
            }
        }
    }
    None
}

/// The quoted entries of the `members` array of a workspace manifest.
fn workspace_members(manifest: &str) -> Vec<String> {
    let mut in_workspace = false;
    let mut collecting = false;
    let mut members = Vec::new();
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_workspace = line == "[workspace]";
        }
        if in_workspace && line.starts_with("members") {
            collecting = true;
        }
        if collecting {
            members.extend(line.split('"').skip(1).step_by(2).map(str::to_string));
            if line.contains(']') {
                collecting = false;
            }
        }
    }
    members
}

/// True when the manifest has a `[lints]` table holding `workspace = true`.
fn inherits_workspace_lints(manifest: &str) -> bool {
    let mut in_lints = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_lints = line == "[lints]";
        } else if in_lints && line.replace(' ', "") == "workspace=true" {
            return true;
        }
    }
    false
}

/// The manifests in `names` that do not inherit the workspace lints.
fn without_workspace_lints(names: &[String], read: impl Fn(&str) -> String) -> Vec<String> {
    names
        .iter()
        .filter(|name| !inherits_workspace_lints(&read(name)))
        .cloned()
        .collect()
}

/// The tables of a `deny.toml` as lines, comments and blank lines dropped,
/// split at each header.
fn deny_tables(deny: &str) -> Vec<Vec<String>> {
    let mut tables: Vec<Vec<String>> = Vec::new();
    for line in deny.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            tables.push(Vec::new());
        }
        if let Some(table) = tables.last_mut() {
            table.push(line.to_string());
        }
    }
    tables
}

/// The crate-scoped exception the fuzz configuration adds to the root one.
const FUZZ_EXCEPTION: [&str; 3] = [
    "[[licenses.exceptions]]",
    "crate = \"libfuzzer-sys\"",
    "allow = [\"NCSA\"]",
];

/// A problem with the fuzz `deny.toml`, or `None` when it is the root
/// configuration plus exactly the crate-scoped exception.
fn deny_problem(root: &str, fuzz: &str) -> Option<String> {
    let want = deny_tables(root);
    let (extra, shared): (Vec<_>, Vec<_>) = deny_tables(fuzz)
        .into_iter()
        .partition(|table| table.iter().map(String::as_str).eq(FUZZ_EXCEPTION));
    if extra.len() != 1 {
        Some(format!(
            "fuzz/deny.toml: expected exactly one exception for libfuzzer-sys, found {}",
            extra.len()
        ))
    } else if shared != want {
        Some("fuzz/deny.toml: the shared sections differ from deny.toml".to_string())
    } else {
        None
    }
}

fn read_text(root: &Path, name: &str) -> String {
    let bytes = read_tracked(root, Path::new(name));
    decode_text(&bytes)
        .unwrap_or_else(|| panic!("{name} is not UTF-8 text"))
        .to_string()
}

// ---------------------------------------------------------------------------
// Gates on the real tree
// ---------------------------------------------------------------------------

#[test]
fn licenses_mulan_texts_match_the_pinned_digest() {
    let root = gate_root();
    let problems: Vec<String> = MULAN_FILES
        .iter()
        .filter_map(|name| {
            digest_problem(name, &read_tracked(&root, Path::new(name)), MULAN_SHA256)
        })
        .collect();
    assert!(
        problems.is_empty(),
        "licenses:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn licenses_zh_hans_matches_the_recorded_fingerprint() {
    let root = gate_root();
    let record = read_text(&root, BACKTRANSLATION);
    let document = read_tracked(&root, Path::new(ZH_HANS));
    if let Some(problem) = fingerprint_problem(&record, &document) {
        panic!("licenses: {problem}");
    }
}

#[test]
fn issue_refs_tracked_files_are_clean() {
    let root = gate_root();
    let paths = tracked_paths();
    let (hits, scanned) = scan_tree(&root, &paths);
    assert!(scanned > 0, "no tracked text file was scanned");
    assert!(
        hits.is_empty(),
        "issue-refs: {} hits; remove issue/PR tokens and process wording:\n  {}",
        hits.len(),
        hits.join("\n  ")
    );
}

#[test]
fn citation_version_matches_the_workspace() {
    let root = gate_root();
    let problem = version_problem(
        &read_text(&root, "Cargo.toml"),
        &read_text(&root, "CITATION.cff"),
    );
    if let Some(problem) = problem {
        panic!("citation: {problem}");
    }
}

// ---------------------------------------------------------------------------
// Fixtures: SHA-256 and the licence gate
// ---------------------------------------------------------------------------

#[test]
fn licenses_sha256_matches_known_vectors() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // Two blocks, so the padding path that spills into a second block runs.
    assert_eq!(
        sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
    // Exactly one block of data forces a block holding only padding.
    assert_eq!(
        sha256_hex(&[b'a'; 64]),
        "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb"
    );
}

#[test]
fn licenses_rejects_an_altered_mulan_text() {
    let altered = digest_problem("LICENSE-MULAN", b"not the licence", MULAN_SHA256);
    assert!(altered.is_some_and(|p| p.starts_with("LICENSE-MULAN: sha256 ")));
    assert!(digest_problem("LICENSE-MULAN", b"", MULAN_SHA256).is_some());
}

#[test]
fn licenses_accepts_bytes_that_hash_to_the_pin() {
    let pin = sha256_hex(b"the text");
    assert!(digest_problem("x", b"the text", &pin).is_none());
    assert!(digest_problem("x", b"the text.", &pin).is_some());
    assert_eq!(MULAN_SHA256.len(), 64);
}

#[test]
fn licenses_rejects_a_fingerprint_mismatch() {
    let record = format!("- {FINGERPRINT_LABEL}: `{}`.", sha256_hex(b"other"));
    assert!(fingerprint_problem(&record, b"changed text").is_some());
}

#[test]
fn licenses_accepts_a_matching_fingerprint() {
    let record = format!("- {FINGERPRINT_LABEL}: `{}`.", sha256_hex(b"the text"));
    assert!(fingerprint_problem(&record, b"the text").is_none());
}

#[test]
fn licenses_rejects_a_record_without_a_fingerprint_line() {
    let record = format!("- {FINGERPRINT_LABEL}: none recorded.\n- other: `abc`.");
    assert!(fingerprint_problem(&record, b"text").is_some());
    assert!(fingerprint_problem("no label here", b"text").is_some());
}

// ---------------------------------------------------------------------------
// Fixtures: the issue-reference and process-wording gate
// ---------------------------------------------------------------------------

/// Fixtures spell `#` as `@` so the test source does not trip its own gate.
fn hashed(fixture: &str) -> String {
    fixture.replace('@', "#")
}

fn assert_banned(line: &str, kind: &str) {
    assert_eq!(banned_in_line(line), Some(kind), "expected a hit: {line:?}");
}

fn assert_clean(line: &str) {
    assert_eq!(banned_in_line(line), None, "expected clean: {line:?}");
}

const NUMBER: &str = "issue or pull request number";
const PROCESS: &str = "development-process wording";

#[test]
fn issue_refs_rejects_a_hash_number_after_a_space() {
    assert_banned(&hashed("see @123"), NUMBER);
    assert_banned(&hashed("Per @12"), NUMBER);
    assert_banned(&hashed("@7 first"), NUMBER);
}

#[test]
fn issue_refs_rejects_a_hash_number_after_a_word_character() {
    assert_banned(&hashed("org/repo@12"), NUMBER);
    assert_banned(&hashed("repo@12"), NUMBER);
    assert_banned(&hashed("Fixes@12"), NUMBER);
    assert_banned(&hashed("x@1"), NUMBER);
    assert_banned(
        &hashed("see Blackcat-Informatics/rust-agentic-foundations@12 for it"),
        NUMBER,
    );
}

#[test]
fn issue_refs_rejects_a_parenthesised_hash_number() {
    assert_banned(&hashed("(@1234)"), NUMBER);
    assert_banned(&hashed("fixed in (@12345)."), NUMBER);
}

#[test]
fn issue_refs_spares_a_quoted_three_digit_colour() {
    assert_clean(&hashed(r#"fill="@000""#));
}

#[test]
fn issue_refs_spares_a_hex_colour_with_letters() {
    assert_clean(&hashed("stroke='@abc'"));
    assert_clean(&hashed("color: @1f2"));
    assert_clean(&hashed("fill @111214 and @fbbc05"));
}

#[test]
fn issue_refs_spares_an_xml_character_reference() {
    assert_clean(&hashed("&@38;"));
}

#[test]
fn issue_refs_spares_a_six_digit_colour() {
    assert_clean(&hashed("@123456"));
}

#[test]
fn issue_refs_rejects_a_colon_prefixed_number() {
    // Only a quoted three- or four-digit token is a colour; after a colon an
    // all-digit token is a reference.
    assert_banned(&hashed("id:@12"), NUMBER);
    assert_banned(&hashed("id:@12345"), NUMBER);
    assert_banned(&hashed("Fixes:@123"), NUMBER);
    assert_banned(&hashed("id:@1234"), NUMBER);
}

#[test]
fn issue_refs_rejects_an_all_digit_css_colour() {
    // An all-digit CSS colour fails closed; write it with letters or as
    // `@112233`.
    assert_banned(&hashed("color: @123"), NUMBER);
}

#[test]
fn issue_refs_rejects_an_issue_url() {
    assert_banned(concat!("https://example.invalid/", "issues/", "7"), NUMBER);
}

#[test]
fn issue_refs_rejects_a_pull_url() {
    assert_banned(concat!("https://example.invalid/", "pull/", "7"), NUMBER);
}

#[test]
fn issue_refs_rejects_a_pull_request_label() {
    assert_banned(concat!("see PR", " ", "9"), NUMBER);
    assert_banned(concat!("see PR", "9"), NUMBER);
    assert_clean("the PRESS release");
    // The label is case-sensitive, as the original search was: a lowercase
    // `pr` followed by a digit is common in identifiers and must not hit.
    assert_clean(concat!("pr", " 12"));
    assert_clean(concat!("pr", "12 = 3"));
}

#[test]
fn issue_refs_rejects_a_change_number() {
    assert_banned(concat!("CHANGE", "-", "17"), NUMBER);
    assert_clean("CHANGE-me");
}

#[test]
fn issue_refs_rejects_the_word_issue_with_a_number() {
    assert_banned(concat!("Closes issue", " 12"), NUMBER);
    assert_banned(concat!("Issue", " 3 is open"), NUMBER);
    assert_clean(concat!("reissue", " 12 copies"));
}

#[test]
fn issue_refs_rejects_a_github_style_number() {
    assert_banned(concat!("see GH", "-12"), NUMBER);
    assert_clean(concat!("HIGH", "-12 grade"));
}

#[test]
fn issue_refs_rejects_the_words_pull_request_with_a_number() {
    assert_banned(concat!("merged in pull", " request 11"), NUMBER);
    assert_banned(concat!("Pull", " Request 11"), NUMBER);
}

#[test]
fn issue_refs_spares_near_miss_prose() {
    assert_clean("an issue-reference is a token");
    assert_clean("the issue tracker is external");
    assert_clean("pull request templates live in .github");
    assert_clean("issue numbers are not recorded here");
}

#[test]
fn issue_refs_rejects_this_issue() {
    assert_banned(concat!("As noted in this", " issue"), PROCESS);
}

#[test]
fn issue_refs_rejects_a_plan_possessive() {
    assert_banned(concat!("the plan", "'s step"), PROCESS);
    assert_clean("the planet");
}

#[test]
fn issue_refs_rejects_per_the_plan() {
    assert_banned(concat!("Per the", " plan we skip it"), PROCESS);
}

#[test]
fn issue_refs_rejects_this_branch() {
    assert_banned(concat!("As agreed in this", " branch"), PROCESS);
}

#[test]
fn issue_refs_rejects_a_task_number() {
    assert_banned(concat!("Task", " 4 is done"), PROCESS);
    assert_clean("the task list");
}

#[test]
fn issue_refs_rejects_an_acceptance_label() {
    assert_banned(concat!("Criterion AC", "3 is met"), PROCESS);
    assert_clean("the SPAC3 board and AC/DC");
}

#[test]
fn issue_refs_reports_every_hit_as_file_and_line() {
    let text = hashed("clean line\nsee @12\nalso clean\n");
    let hits = scan_text(Path::new("docs/a.md"), &text);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0], format!("docs/a.md:2: {NUMBER}"));
    let both = scan_text(
        Path::new("b.txt"),
        &format!("{}\n{}\n", hashed("x @9"), concat!("Task", " 1")),
    );
    assert_eq!(
        both,
        vec![format!("b.txt:1: {NUMBER}"), format!("b.txt:2: {PROCESS}")]
    );
}

#[test]
fn issue_refs_treats_only_a_nul_byte_as_binary() {
    assert!(decode_text(b"plain text").is_some());
    assert!(decode_text(b"\x89PNG\r\n\x1a\n\x00\x00\x00").is_none());
    assert!(decode_text(b"text\0text").is_none());
}

#[test]
fn issue_refs_flags_a_token_in_a_latin_1_file() {
    // Invalid UTF-8 is not binary: a Latin-1 file is still scanned.
    let mut bytes = b"caf\xe9 ".to_vec();
    bytes.extend_from_slice(hashed("see @12\n").as_bytes());
    assert!(std::str::from_utf8(&bytes).is_err());
    let text = decode_text(&bytes).expect("a file without NUL is text");
    assert_eq!(
        scan_text(Path::new("latin1.txt"), &text),
        vec![format!("latin1.txt:1: {NUMBER}")]
    );
}

#[test]
fn issue_refs_skips_a_nul_containing_file_holding_a_token() {
    let bytes = hashed("see @12\0\n").into_bytes();
    assert!(decode_text(&bytes).is_none());
}

#[test]
fn issue_refs_scans_text_files_and_skips_binary_ones() {
    let dir = std::env::temp_dir().join(format!("raf-gates-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create scratch directory");
    let bad = hashed("see @12\n");
    std::fs::write(dir.join("text.md"), bad).expect("write text fixture");
    std::fs::write(dir.join("image.bin"), [0x89, b'P', 0, b'@', b'1', b'2']).expect("write binary");
    let paths = [PathBuf::from("text.md"), PathBuf::from("image.bin")];
    let (hits, scanned) = scan_tree(&dir, &paths);
    std::fs::remove_dir_all(&dir).expect("remove scratch directory");
    assert_eq!(scanned, 1);
    assert_eq!(hits, vec![format!("text.md:1: {NUMBER}")]);
}

// ---------------------------------------------------------------------------
// Fixtures: version agreement
// ---------------------------------------------------------------------------

const MANIFEST: &str = "[package]\nversion.workspace = true\n\n[workspace.package]\nversion = \"0.1.0\"\nedition = \"2024\"\n";

#[test]
fn citation_accepts_matching_versions() {
    assert!(version_problem(MANIFEST, "title: x\nversion: \"0.1.0\"\n").is_none());
    assert!(version_problem(MANIFEST, "version: 0.1.0\n").is_none());
}

#[test]
fn citation_rejects_a_version_bump_without_a_matching_manifest() {
    let problem = version_problem(MANIFEST, "version: \"0.2.0\"\n");
    assert!(problem.is_some_and(|p| p.contains("0.2.0") && p.contains("0.1.0")));
}

#[test]
fn citation_rejects_a_manifest_bump_without_a_matching_citation() {
    let bumped = MANIFEST.replace("0.1.0", "0.3.0");
    assert!(version_problem(&bumped, "version: \"0.1.0\"\n").is_some());
}

#[test]
fn citation_rejects_a_missing_version() {
    assert!(version_problem(MANIFEST, "title: x\n").is_some());
    assert!(version_problem("[package]\nversion = \"0.1.0\"\n", "version: \"0.1.0\"\n").is_some());
}

// ---------------------------------------------------------------------------
// Gates on the real tree: lints and licence configuration
// ---------------------------------------------------------------------------

#[test]
fn lints_fuzz_workspace_tables_equal_the_root_tables() {
    let root = gate_root();
    let problem = lint_tables_problem(
        &read_text(&root, "Cargo.toml"),
        &read_text(&root, "fuzz/Cargo.toml"),
    );
    assert_eq!(problem, None);
}

#[test]
fn lints_the_root_denies_missing_docs_unsafe_code_and_unreachable_pub() {
    let manifest = read_text(&gate_root(), "Cargo.toml");
    for lint in ["missing_docs", "unsafe_code", "unreachable_pub"] {
        assert_eq!(
            rust_lint_level(&manifest, lint).as_deref(),
            Some("deny"),
            "[workspace.lints.rust] must set {lint} = \"deny\""
        );
    }
}

#[test]
fn lints_every_workspace_manifest_inherits_the_workspace_lints() {
    let root = gate_root();
    let manifest = read_text(&root, "Cargo.toml");
    let mut names: Vec<String> = workspace_members(&manifest)
        .iter()
        .map(|member| format!("{member}/Cargo.toml"))
        .collect();
    assert!(!names.is_empty(), "the root manifest lists no members");
    names.push("Cargo.toml".to_string());
    names.push("fuzz/Cargo.toml".to_string());
    let missing = without_workspace_lints(&names, |name| read_text(&root, name));
    assert_eq!(missing, Vec::<String>::new());
}

#[test]
fn deny_fuzz_configuration_is_the_root_configuration_plus_one_exception() {
    let root = gate_root();
    let problem = deny_problem(
        &read_text(&root, "deny.toml"),
        &read_text(&root, "fuzz/deny.toml"),
    );
    assert_eq!(problem, None);
}

// ---------------------------------------------------------------------------
// Fixtures: lints and licence configuration
// ---------------------------------------------------------------------------

const LINT_ROOT: &str = "[workspace]\nmembers = [\"a\"]\n\n[workspace.lints.rust]\n# why\nmissing_docs = \"deny\"\n\n[workspace.lints.clippy]\nall = \"warn\"\n";

#[test]
fn lints_reads_the_level_of_a_lint_and_sees_a_weakened_one() {
    assert_eq!(
        rust_lint_level(LINT_ROOT, "missing_docs").as_deref(),
        Some("deny")
    );
    let weakened = LINT_ROOT.replace("missing_docs = \"deny\"", "missing_docs = \"warn\"");
    assert_eq!(
        rust_lint_level(&weakened, "missing_docs").as_deref(),
        Some("warn")
    );
    // A lint of another table, or one that is not set, has no level here.
    assert_eq!(rust_lint_level(LINT_ROOT, "all"), None);
    assert_eq!(rust_lint_level(LINT_ROOT, "unsafe_code"), None);
}

#[test]
fn lints_accepts_equal_tables_whatever_the_comments() {
    let fuzz = "[package]\nname = \"f\"\n\n[workspace.lints.rust]\nmissing_docs = \"deny\"\n# other note\n\n[workspace.lints.clippy]\nall = \"warn\"\n";
    assert_eq!(lint_tables_problem(LINT_ROOT, fuzz), None);
}

#[test]
fn lints_rejects_a_weakened_level_in_the_fuzz_tables() {
    let fuzz = LINT_ROOT.replace("\"deny\"", "\"warn\"");
    assert!(lint_tables_problem(LINT_ROOT, &fuzz).is_some());
}

#[test]
fn lints_rejects_an_added_or_removed_lint_in_the_fuzz_tables() {
    let added = format!("{LINT_ROOT}pedantic = \"warn\"\n");
    assert!(lint_tables_problem(LINT_ROOT, &added).is_some());
    let removed = LINT_ROOT.replace("missing_docs = \"deny\"\n", "");
    assert!(lint_tables_problem(LINT_ROOT, &removed).is_some());
    assert!(lint_tables_problem(LINT_ROOT, "[package]\nname = \"f\"\n").is_some());
}

#[test]
fn lints_rejects_a_root_without_lint_tables() {
    assert!(lint_tables_problem("[workspace]\n", "[workspace]\n").is_some());
}

#[test]
fn lints_reads_single_and_multi_line_member_lists() {
    assert_eq!(workspace_members(LINT_ROOT), ["a"]);
    let spread = "[workspace]\nresolver = \"3\"\nmembers = [\n  \"crates/a\",\n  \"crates/b\",\n]\nexclude = [\"fuzz\"]\n";
    assert_eq!(workspace_members(spread), ["crates/a", "crates/b"]);
}

#[test]
fn lints_rejects_a_member_manifest_that_does_not_inherit() {
    let names = [
        "ok/Cargo.toml",
        "plain/Cargo.toml",
        "off/Cargo.toml",
        "other/Cargo.toml",
    ]
    .map(str::to_string);
    let read = |name: &str| match name {
        "ok/Cargo.toml" => "[package]\nname = \"ok\"\n\n[lints]\nworkspace = true\n".to_string(),
        "plain/Cargo.toml" => "[package]\nname = \"plain\"\n".to_string(),
        "off/Cargo.toml" => "[lints]\nworkspace = false\n".to_string(),
        // `workspace = true` under another table is not the lint switch.
        _ => "[package]\nworkspace = true\n\n[lints]\nrust = {}\n".to_string(),
    };
    assert_eq!(
        without_workspace_lints(&names, read),
        ["plain/Cargo.toml", "off/Cargo.toml", "other/Cargo.toml"]
    );
}

const DENY_ROOT: &str = "# note\n[licenses]\nallow = [\"MIT\"]\n\n[[licenses.exceptions]]\ncrate = \"unicode-ident\"\nallow = [\"Unicode-3.0\"]\n\n[bans]\nwildcards = \"deny\"\n";

fn fuzz_deny() -> String {
    DENY_ROOT.replace(
        "\n[bans]",
        "\n# why\n[[licenses.exceptions]]\ncrate = \"libfuzzer-sys\"\nallow = [\"NCSA\"]\n\n[bans]",
    )
}

#[test]
fn deny_accepts_the_root_configuration_plus_the_one_exception() {
    assert_eq!(deny_problem(DENY_ROOT, &fuzz_deny()), None);
}

#[test]
fn deny_rejects_a_shared_section_that_differs() {
    let changed = fuzz_deny().replace("wildcards = \"deny\"", "wildcards = \"warn\"");
    assert!(deny_problem(DENY_ROOT, &changed).is_some());
    let widened = fuzz_deny().replace("allow = [\"MIT\"]", "allow = [\"MIT\", \"GPL-3.0\"]");
    assert!(deny_problem(DENY_ROOT, &widened).is_some());
}

#[test]
fn deny_rejects_a_missing_or_widened_or_repeated_exception() {
    assert!(deny_problem(DENY_ROOT, DENY_ROOT).is_some());
    let widened = fuzz_deny().replace("[\"NCSA\"]", "[\"NCSA\", \"GPL-3.0\"]");
    assert!(deny_problem(DENY_ROOT, &widened).is_some());
    let other_crate = fuzz_deny().replace("libfuzzer-sys", "other");
    assert!(deny_problem(DENY_ROOT, &other_crate).is_some());
    let twice = fuzz_deny().replace(
        "\n[bans]",
        "\n[[licenses.exceptions]]\ncrate = \"libfuzzer-sys\"\nallow = [\"NCSA\"]\n\n[bans]",
    );
    assert!(deny_problem(DENY_ROOT, &twice).is_some());
}

#[test]
fn deny_rejects_a_dropped_root_exception() {
    let dropped = fuzz_deny().replace(
        "[[licenses.exceptions]]\ncrate = \"unicode-ident\"\nallow = [\"Unicode-3.0\"]\n",
        "",
    );
    assert!(deny_problem(DENY_ROOT, &dropped).is_some());
}
