// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The scanner against the output of `cargo metadata` and against hostile
//! text: what it accepts is exactly one top-level string member, and nothing
//! else is guessed.

use std::fmt::Write as _;

use super::{JsonError, MAX_DEPTH, workspace_root};

fn root(text: &str) -> Result<String, JsonError> {
    workspace_root(text)
}

fn malformed(text: &str) -> bool {
    matches!(root(text), Err(JsonError::Malformed { .. }))
}

/// A cut-down `cargo metadata` output, in the member order cargo writes.
const METADATA: &str = r#"{"packages":[{"name":"demo","version":"0.1.0","id":"path+file:///w#demo@0.1.0","license":null,"dependencies":[],"targets":[{"kind":["lib"],"crate_types":["lib"],"name":"demo","src_path":"/w/src/lib.rs","edition":"2021","doc":true,"test":true}],"features":{},"manifest_path":"/w/Cargo.toml","metadata":null,"publish":null,"authors":[],"rust_version":null}],"workspace_members":["path+file:///w#demo@0.1.0"],"workspace_default_members":["path+file:///w#demo@0.1.0"],"resolve":null,"target_directory":"/w/target","version":1,"workspace_root":"/w","metadata":null}"#;

#[test]
fn the_output_of_cargo_metadata_gives_its_root() {
    assert_eq!(root(METADATA), Ok("/w".to_string()));
    // With whitespace between every token, as a pretty printer writes it.
    let spaced = "{ \t\r\n\"a\" : [ 1 , 2 ] ,\n \"workspace_root\"\n:\n\"/w\" \n}\n";
    assert_eq!(root(spaced), Ok("/w".to_string()));
}

#[test]
fn the_member_is_found_wherever_it_sits_at_the_top_level() {
    for text in [
        r#"{"workspace_root":"/a"}"#,
        r#"{"x":1,"workspace_root":"/a"}"#,
        r#"{"workspace_root":"/a","x":[1,{"y":null}]}"#,
        r#"{"x":{"workspace_root":"/nested"},"workspace_root":"/a","y":{"workspace_root":"/also"}}"#,
        r#"{"x":[{"workspace_root":"/nested"}],"workspace_root":"/a"}"#,
        r#"{"description":"\"workspace_root\":\"/evil\"","workspace_root":"/a"}"#,
        r#"{"workspace_root_not":"/evil","workspace_root":"/a","xworkspace_root":"/evil"}"#,
        r#"{"workspace_root":"/a","workspace_root2":"/evil"}"#,
        r#"{"workspace\u005froot":"/a"}"#,
        r#"{"Workspace_root":"/evil","workspace_root":"/a"}"#,
    ] {
        assert_eq!(root(text), Ok("/a".to_string()), "{text}");
    }
}

#[test]
fn a_member_that_is_only_nested_is_not_the_member() {
    for text in [
        "{}",
        r#"{"x":{"workspace_root":"/nested"}}"#,
        r#"{"x":[{"workspace_root":"/nested"}]}"#,
        r#"{"workspace_root_":"/a"}"#,
        r#"{"WORKSPACE_ROOT":"/a"}"#,
        r#"{"x":"workspace_root"}"#,
        r#"{"x":"\"workspace_root\":\"/evil\""}"#,
    ] {
        assert_eq!(root(text), Err(JsonError::Missing), "{text}");
    }
}

#[test]
fn escapes_are_decoded() {
    assert_eq!(
        root(r#"{"workspace_root":"/h\/o\\me\"q\u0041\u00e9 \b\f\n\r\t"}"#),
        Ok("/h/o\\me\"qA\u{e9} \u{8}\u{c}\n\r\t".to_string())
    );
    // A surrogate pair is one character; raw characters pass through.
    assert_eq!(
        root(r#"{"workspace_root":"/\ud83d\ude00"}"#),
        Ok("/\u{1f600}".to_string())
    );
    assert_eq!(
        root("{\"workspace_root\":\"/home/\u{e9}/\u{540d}\u{1f600}\"}"),
        Ok("/home/\u{e9}/\u{540d}\u{1f600}".to_string())
    );
    assert_eq!(
        root(r#"{"workspace_root":"C:\\Users\\me"}"#),
        Ok("C:\\Users\\me".to_string())
    );
    assert_eq!(root(r#"{"workspace_root":""}"#), Ok(String::new()));
}

#[test]
fn a_value_that_is_not_a_string_is_refused() {
    for text in [
        r#"{"workspace_root":1}"#,
        r#"{"workspace_root":null}"#,
        r#"{"workspace_root":true}"#,
        r#"{"workspace_root":["/a"]}"#,
        r#"{"workspace_root":{"path":"/a"}}"#,
    ] {
        assert_eq!(root(text), Err(JsonError::NotAString), "{text}");
    }
    // A text that is not JSON is that, whatever the member holds.
    assert!(malformed(r#"{"workspace_root":tru}"#));
    assert!(malformed(r#"{"workspace_root":[1,}"#));
}

#[test]
fn a_member_twice_is_refused_even_when_the_values_agree() {
    for text in [
        r#"{"workspace_root":"/a","workspace_root":"/a"}"#,
        r#"{"workspace_root":"/a","workspace_root":"/b"}"#,
        r#"{"workspace_root":"/a","x":1,"workspace\u005froot":"/a"}"#,
    ] {
        assert_eq!(root(text), Err(JsonError::Duplicate), "{text}");
    }
}

#[test]
fn a_top_level_value_that_is_not_an_object_is_refused() {
    for text in ["[]", "[\"/a\"]", "\"/a\"", "1", "null", "true", " [ ] "] {
        assert_eq!(root(text), Err(JsonError::NotAnObject), "{text}");
    }
    for text in ["", " ", "\n", "[", "]", "tru", "-"] {
        assert!(malformed(text), "{text:?}");
    }
}

#[test]
fn text_that_is_not_json_is_refused() {
    for text in [
        // Structure.
        r#"{"workspace_root":"/a",}"#,
        r#"{,"workspace_root":"/a"}"#,
        r#"{"workspace_root":"/a""x":1}"#,
        r#"{"workspace_root" "/a"}"#,
        r#"{"workspace_root":}"#,
        r#"{"workspace_root":"/a""#,
        r#"{"workspace_root":"/a"}}"#,
        r#"{"workspace_root":"/a"}{}"#,
        r#"{"workspace_root":"/a"} x"#,
        r#"{workspace_root:"/a"}"#,
        r"{'workspace_root':'/a'}",
        r#"{"workspace_root":"/a"} // comment"#,
        r#"/* c */{"workspace_root":"/a"}"#,
        r#"{"x":[1,],"workspace_root":"/a"}"#,
        r#"{"x":[,1],"workspace_root":"/a"}"#,
        r#"{"x":[1 2],"workspace_root":"/a"}"#,
        r#"{"x":{"y":1,},"workspace_root":"/a"}"#,
        r#"{"x":{1:2},"workspace_root":"/a"}"#,
        r#"{"x":{"y"},"workspace_root":"/a"}"#,
        "\u{feff}{\"workspace_root\":\"/a\"}",
        "\u{a0}{\"workspace_root\":\"/a\"}",
        // Strings.
        "{\"workspace_root\":\"/a\nb\"}",
        "{\"workspace_root\":\"/a\tb\"}",
        "{\"workspace_root\":\"/a\u{1}b\"}",
        r#"{"workspace_root":"/a\xb"}"#,
        r#"{"workspace_root":"/a\u12"}"#,
        r#"{"workspace_root":"/a\u12g4"}"#,
        r#"{"workspace_root":"/a\u+123"}"#,
        r#"{"workspace_root":"/a\"#,
        r#"{"workspace_root":"/a\"}"#,
        r#"{"x":"a\qb","workspace_root":"/a"}"#,
        "{\"x\":\"a\nb\",\"workspace_root\":\"/a\"}",
        // Numbers.
        r#"{"x":01,"workspace_root":"/a"}"#,
        r#"{"x":1.,"workspace_root":"/a"}"#,
        r#"{"x":.5,"workspace_root":"/a"}"#,
        r#"{"x":-,"workspace_root":"/a"}"#,
        r#"{"x":1e,"workspace_root":"/a"}"#,
        r#"{"x":1e+,"workspace_root":"/a"}"#,
        r#"{"x":+1,"workspace_root":"/a"}"#,
        r#"{"x":0x10,"workspace_root":"/a"}"#,
        r#"{"x":NaN,"workspace_root":"/a"}"#,
        r#"{"x":Infinity,"workspace_root":"/a"}"#,
        r#"{"x":-Infinity,"workspace_root":"/a"}"#,
        // Words.
        r#"{"x":True,"workspace_root":"/a"}"#,
        r#"{"x":nul,"workspace_root":"/a"}"#,
        r#"{"x":undefined,"workspace_root":"/a"}"#,
    ] {
        assert!(malformed(text), "{text:?} -> {:?}", root(text));
    }
}

#[test]
fn numbers_of_the_grammar_pass() {
    for number in [
        "0", "-0", "1", "-1", "10", "1.5", "-1.5e10", "1E+3", "1e-3", "0.0",
    ] {
        let text = format!(r#"{{"x":{number},"workspace_root":"/a"}}"#);
        assert_eq!(root(&text), Ok("/a".to_string()), "{number}");
    }
}

#[test]
fn an_unpaired_surrogate_is_text_nobody_can_use() {
    // In the member it is refused; elsewhere it is valid JSON and passes.
    for text in [
        r#"{"workspace_root":"/\ud800"}"#,
        r#"{"workspace_root":"/\udc00"}"#,
        r#"{"workspace_root":"/\ud800x"}"#,
        r#"{"workspace_root":"/\ud800\u0041"}"#,
        r#"{"workspace_root":"/\udc00\ud800"}"#,
    ] {
        assert_eq!(root(text), Err(JsonError::NotText), "{text}");
    }
    assert_eq!(
        root(r#"{"x":"\ud800","y":"\udc00\u0041","workspace_root":"/a"}"#),
        Ok("/a".to_string())
    );
    // A high surrogate followed by a broken escape is malformed.
    assert!(malformed(r#"{"workspace_root":"/\ud800\u00"}"#));
}

fn nested(depth: usize) -> String {
    format!(
        "{{\"x\":{}{},\"workspace_root\":\"/a\"}}",
        "[".repeat(depth),
        "]".repeat(depth)
    )
}

#[test]
fn nesting_is_bounded_and_never_overflows_the_stack() {
    assert_eq!(root(&nested(MAX_DEPTH)), Ok("/a".to_string()));
    assert!(malformed(&nested(MAX_DEPTH + 1)));
    // Far deeper than any stack could recurse, and unclosed.
    let deep = format!("{{\"x\":{}", "[".repeat(5_000_000));
    assert!(malformed(&deep));
    let deep_objects = format!("{{\"x\":{}", "{\"a\":".repeat(1_000_000));
    assert!(malformed(&deep_objects));
    let at_top = "[".repeat(1_000_000);
    assert!(malformed(&at_top));
}

#[test]
fn a_large_text_is_scanned_to_its_end() {
    // Many members, then the root: found. The same without a root: missing.
    let mut many = String::new();
    for n in 0..100_000 {
        write!(many, "\"k{n}\":{{\"v\":[{n},\"s\",null]}},").expect("a string");
    }
    assert_eq!(
        root(&format!("{{{many}\"workspace_root\":\"/a\"}}")),
        Ok("/a".to_string())
    );
    assert_eq!(root(&format!("{{{many}\"z\":0}}")), Err(JsonError::Missing));
    // A long string is one string.
    let long = "a".repeat(2_000_000);
    assert_eq!(
        root(&format!("{{\"workspace_root\":\"/{long}\"}}")).map(|path| path.len()),
        Ok(2_000_001)
    );
}

#[test]
fn every_prefix_of_a_document_is_refused_and_no_mutation_panics() {
    let text = METADATA;
    for end in 0..text.len() {
        if !text.is_char_boundary(end) {
            continue;
        }
        assert!(root(&text[..end]).is_err(), "prefix of {end} bytes");
    }
    // Replace each byte by each of a set of structural and hostile bytes. A
    // mutation may leave the text valid; the scanner must answer either way.
    let bytes = text.as_bytes();
    for at in 0..bytes.len() {
        for replacement in [
            b'{', b'}', b'[', b']', b'"', b'\\', b',', b':', 0, b'\n', b'-', b'e',
        ] {
            let mut mutated = bytes.to_vec();
            mutated[at] = replacement;
            let mutated = String::from_utf8(mutated).expect("ASCII stays UTF-8");
            let _ = root(&mutated);
        }
    }
}

#[test]
fn random_strings_of_json_punctuation_never_panic() {
    // A fixed linear congruential generator: the same strings on every run.
    let alphabet: &[u8] = b"{}[]\":,\\ \n0123456789-+.eEtrufalsn\x00\x1f";
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(state >> 33).expect("fits")
    };
    for _ in 0..20_000 {
        let length = next() % 64;
        let text: String = (0..length)
            .map(|_| char::from(alphabet[next() % alphabet.len()]))
            .collect();
        let _ = root(&text);
        // Wrapped as the head of an object that carries the member after it.
        let _ = root(&format!("{{\"workspace_root\":\"/a\",{text}"));
        let _ = root(&format!("{{{text},\"workspace_root\":\"/a\"}}"));
    }
}
