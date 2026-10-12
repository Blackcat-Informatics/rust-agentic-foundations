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
//! * `CITATION.cff` and `[workspace.package]` carry the same version.
//!
//! Each gate is a pure function over text or bytes, and each has a named
//! negative fixture below that breaks exactly one thing. Test names start with
//! `licenses_`, `issue_refs_` or `citation_` so `make licenses` and
//! `make issue-refs` can select a gate. Only the standard library is used.

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
/// of text files scanned; binary files (a NUL byte or invalid UTF-8) are
/// recognised by content and left out of the count.
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
fn issue_refs_rejects_a_colon_prefixed_number_over_four_digits() {
    // Only three and four digits are treated as colours in a colour context.
    assert_banned(&hashed("id:@12"), NUMBER);
    assert_banned(&hashed("id:@12345"), NUMBER);
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
