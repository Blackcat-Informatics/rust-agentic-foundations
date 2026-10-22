// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The normal form through its public API: the registered pairs, the
//! monotonicity of the tower, the boundaries, the refusals and the encoding.

mod support;

use std::collections::BTreeMap;

use raf_census::{
    CensusOptions, Divergence, Error, Fingerprint, Level, MAX_NESTING, Normalizer,
    ParseFingerprintError, Slice, TableKind, Tower, VALUE_ADAPTERS,
};
use support::fixtures::{Expect, Fixture, registry};

fn normalizer(min_tokens: usize) -> Normalizer {
    Normalizer::new(
        &CensusOptions::default()
            .min_tokens(min_tokens)
            .expect("a valid min_tokens"),
    )
}

fn tower(fixture: &Fixture) -> Tower {
    Tower::new(&fixture.slice()).unwrap_or_else(|e| panic!("{}: {e}", fixture.id))
}

fn texts(tower: &Tower, level: Level) -> Vec<String> {
    tower.form(level).texts()
}

#[test]
fn every_registered_pair_compares_as_registered() {
    let registry = registry();
    let by_id: BTreeMap<&str, &Fixture> = registry
        .fixtures
        .iter()
        .map(|fixture| (fixture.id.as_str(), fixture))
        .collect();
    assert!(registry.pairs.len() > 60, "the registry holds the pairs");
    for pair in &registry.pairs {
        let (a, b) = (by_id[pair.a.as_str()], by_id[pair.b.as_str()]);
        let divergence = normalizer(1)
            .explain(&a.slice(), &b.slice())
            .unwrap_or_else(|e| panic!("{}: {e}", pair.id));
        match pair.expect {
            Expect::Equal(level) => {
                assert_eq!(divergence.equal_from, Some(level), "{}", pair.id);
                assert_ne!(
                    divergence.differs_through,
                    Some(Level::Value),
                    "{}",
                    pair.id
                );
            }
            Expect::Distinct => {
                assert_eq!(divergence.equal_from, None, "{}", pair.id);
                assert_eq!(
                    divergence.differs_through,
                    Some(Level::Value),
                    "{}",
                    pair.id
                );
                assert!(divergence.first.is_some(), "{}", pair.id);
            }
        }
    }
}

#[test]
fn a_fixture_named_for_a_size_has_that_many_value_tokens() {
    let registry = registry();
    let sized: Vec<&Fixture> = registry
        .fixtures
        .iter()
        .filter(|f| f.tokens.is_some())
        .collect();
    assert!(sized.len() >= 20);
    for fixture in sized {
        assert_eq!(
            Some(tower(fixture).value_tokens()),
            fixture.tokens,
            "{}",
            fixture.id
        );
    }
}

/// For slices with one parameter list, equal at a finer level implies equal at
/// every coarser one, and no pass adds a token, over every pair of registered
/// fixtures.
#[test]
fn the_tower_is_monotone_over_every_registry_pair() {
    let registry = registry();
    let towers: Vec<(&str, Vec<Vec<String>>)> = registry
        .fixtures
        .iter()
        .map(|fixture| {
            let tower = tower(fixture);
            let forms: Vec<Vec<String>> = Level::ALL
                .iter()
                .map(|level| texts(&tower, *level))
                .collect();
            (fixture.id.as_str(), forms)
        })
        .collect();
    for (id, forms) in &towers {
        for window in forms.windows(2) {
            assert!(
                window[1].len() <= window[0].len(),
                "{id}: a pass added a token"
            );
        }
    }
    let mut equal_pairs = 0;
    for (index, (a_id, a)) in towers.iter().enumerate() {
        for (b_id, b) in &towers[index + 1..] {
            let mut seen_equal = false;
            for level in 0..Level::ALL.len() {
                let equal = a[level] == b[level];
                assert!(
                    equal || !seen_equal,
                    "{a_id} and {b_id} are equal at a finer level but differ at {}",
                    Level::ALL[level]
                );
                seen_equal |= equal;
            }
            equal_pairs += usize::from(seen_equal);
        }
    }
    assert!(equal_pairs > 40, "the property is exercised by equal pairs");
}

fn print_of(params: &[&str], text: &str, min_tokens: usize) -> raf_census::Print {
    normalizer(min_tokens)
        .print(&Slice::new("f.rs", 1, text).params(params.iter().copied()))
        .expect("the text lexes")
}

#[test]
fn shim_forms_exist_for_six_to_twenty_nine_tokens() {
    let registry = registry();
    let print = |id: &str| {
        let fixture = registry
            .fixtures
            .iter()
            .find(|f| f.id == id)
            .expect("registered");
        normalizer(1).print(&fixture.slice()).expect("lexes")
    };
    assert_eq!(print("shim_5").tokens, 5);
    assert!(print("shim_5").shim.is_none(), "5 tokens: no shim form");
    assert!(print("shim_6").shim.is_some(), "6 tokens: a shim form");
    assert!(print("shim_29").shim.is_some(), "29 tokens: a shim form");
    assert!(print("shim_30").shim.is_none(), "30 tokens: no shim form");
    for id in [
        "shim_let_body",
        "shim_two_statements",
        "shim_mentions_self",
        "shim_without_call",
    ] {
        assert!(print(id).shim.is_none(), "{id} is not a shim candidate");
        assert!(print(id).body.is_some(), "{id} keeps its body print");
    }
    assert!(
        print("shim_trailing_semicolon").shim.is_some(),
        "a trailing `;` is not a statement"
    );
    assert!(
        print("shim_macro_call").shim.is_some(),
        "a macro call is a call"
    );
}

#[test]
fn a_shim_candidate_keeps_its_body_print_and_the_shim_print_differs_by_callee() {
    let a = print_of(&["x"], "target(x, 1)", 1);
    let b = print_of(&["y"], "target(y, 1)", 1);
    assert_eq!(a, b, "only own parameters are renamed");
    assert!(a.body.is_some() && a.shim.is_some());
    assert_ne!(a, print_of(&["x"], "other(x, 1)", 1));
    assert_ne!(a, print_of(&["x"], "target(x, 2)", 1));
    let item_a = print_of(&[], "target(first, 1)", 1);
    let item_b = print_of(&[], "target(second, 1)", 1);
    assert_ne!(item_a.shim, item_b.shim, "a name no one binds is an item");
    assert_ne!(item_a.body, item_b.body, "and the value forms differ");
}

#[test]
fn the_body_print_needs_min_tokens() {
    let text = "f(a, b, c, d, e)";
    assert_eq!(print_of(&[], text, 12).tokens, 12);
    assert!(print_of(&[], text, 12).body.is_some());
    assert!(print_of(&[], text, 13).body.is_none());
}

#[test]
fn the_literal_tier_boundary_is_thirty_value_tokens() {
    let registry = registry();
    let print = |id: &str| {
        let fixture = registry
            .fixtures
            .iter()
            .find(|f| f.id == id)
            .expect("registered");
        normalizer(1).print(&fixture.slice()).expect("lexes").body
    };
    assert_ne!(print("tier_29.a"), print("tier_29.b"), "29 keeps literals");
    assert_eq!(print("tier_30.a"), print("tier_30.b"), "30 abstracts them");
    assert_ne!(print("tier_25.a"), print("tier_25.b"));
    assert_eq!(print("tier_35.a"), print("tier_35.b"));
}

#[test]
fn the_value_form_keeps_binary_operators_and_drops_borrows() {
    let value = |params: &[&str], text: &str| {
        let slice = Slice::new("f.rs", 1, text).params(params.iter().copied());
        texts(&Tower::new(&slice).expect("lexes"), Level::Value)
    };
    assert_eq!(
        value(&["a", "b"], "g(a & b)"),
        ["g", "(", "$0", "&", "$1", ")"]
    );
    assert_eq!(
        value(&["a", "b"], "g(a * b)"),
        ["g", "(", "$0", "*", "$1", ")"]
    );
    assert_eq!(
        value(&["a", "b"], "g(a && b)"),
        ["g", "(", "$0", "&~", "&", "$1", ")"]
    );
    assert_eq!(value(&["a"], "g(&mut *a)"), ["g", "(", "$0", ")"]);
    assert_eq!(value(&["a"], "g(&&a)"), ["g", "(", "$0", ")"]);
}

#[test]
fn the_value_adapters_are_the_eleven_of_the_page() {
    assert_eq!(
        VALUE_ADAPTERS,
        [
            "as_str",
            "as_ref",
            "as_slice",
            "as_mut",
            "as_deref",
            "borrow",
            "borrow_mut",
            "clone",
            "cloned",
            "copied",
            "to_owned"
        ]
    );
}

#[test]
fn below_the_tier_the_final_sequence_keeps_literals_and_above_it_abstracts_them() {
    let small = Tower::new(&Slice::new("f.rs", 1, "f(\"a\", 7)")).expect("lexes");
    assert_eq!(
        texts(&small, Level::Value),
        ["f", "(", "\"a\"", ",", "7", ")"]
    );
    let big_text = format!("f(\"a\", 7); {}", "g(1); ".repeat(8));
    let big = Tower::new(&Slice::new("f.rs", 1, &big_text)).expect("lexes");
    assert_eq!(texts(&big, Level::Value)[2], "$str");
    assert_eq!(texts(&big, Level::Value)[4], "$int");
}

#[test]
fn parameters_and_locals_are_numbered_together_by_first_appearance() {
    let slice = Slice::new("f.rs", 1, "let t = b + a; t").params(["a", "b"]);
    let tower = Tower::new(&slice).expect("lexes");
    assert_eq!(
        texts(&tower, Level::Shim),
        ["let", "t", "=", "$1", "+", "$0", ";", "t"],
        "the shim form keeps signature order"
    );
    assert_eq!(
        texts(&tower, Level::Value),
        ["let", "$0", "=", "$1", "+", "$2", ";", "$0"]
    );
    let used_one = Slice::new("f.rs", 1, "let t = b; t").params(["a", "b"]);
    assert_eq!(
        texts(&Tower::new(&used_one).expect("lexes"), Level::Value),
        ["let", "$0", "=", "$1", ";", "$0"]
    );
}

#[test]
fn self_is_not_a_parameter_and_a_field_named_like_one_is_kept() {
    let slice = Slice::new("f.rs", 1, "self.x + x").params(["self", "x"]);
    let tower = Tower::new(&slice).expect("lexes");
    assert_eq!(texts(&tower, Level::Value), ["self", ".", "x", "+", "$0"]);
}

#[test]
fn an_unlexable_slice_is_a_source_error_with_a_valid_neighbour() {
    let refused = Tower::new(&Slice::new("src/a.rs", 10, "f(1);\ng(\"unterminated)"))
        .expect_err("an unterminated string does not lex");
    let Error::Source { file, line, reason } = refused else {
        panic!("not a source error");
    };
    assert_eq!(file, "src/a.rs");
    assert!((10..=11).contains(&line), "line {line}");
    assert!(reason.contains("does not lex"), "{reason}");
    let neighbour = Tower::new(&Slice::new("src/a.rs", 10, "f(1);\ng(\"terminated\")"));
    assert!(neighbour.is_ok());
    assert!(normalizer(1).print(&Slice::new("a.rs", 1, "f(")).is_err());
    assert!(normalizer(1).print(&Slice::new("a.rs", 1, "f()")).is_ok());
    assert!(
        normalizer(1)
            .explain(&Slice::new("a.rs", 1, "f()"), &Slice::new("a.rs", 1, "f("))
            .is_err()
    );
}

#[test]
fn nesting_at_the_budget_lexes_and_one_deeper_is_refused() {
    let nest = |depth: usize| purrdf_testkit::text::nested("(", "x", ")", depth);
    assert_eq!(MAX_NESTING, 128);
    let at = Tower::new(&Slice::new("f.rs", 1, &nest(128))).expect("128 lexes");
    assert_eq!(at.value_tokens(), 257);
    let error = Tower::new(&Slice::new("f.rs", 1, &nest(129))).expect_err("129 is refused");
    assert!(error.to_string().contains("128"), "{error}");
}

#[test]
fn explain_names_the_levels_and_the_first_differing_token_with_its_line() {
    let n = normalizer(1);
    let same = n
        .explain(
            &Slice::new("f.rs", 1, "f(1)"),
            &Slice::new("f.rs", 1, "f( 1 ) // c"),
        )
        .expect("lexes");
    assert_eq!(
        same,
        Divergence {
            differs_through: None,
            equal_from: Some(Level::Verbatim),
            first: None
        }
    );
    let renamed = n
        .explain(
            &Slice::new("f.rs", 1, "let a = 1; a"),
            &Slice::new("f.rs", 1, "let b = 1; b"),
        )
        .expect("lexes");
    assert_eq!(renamed.differs_through, Some(Level::Shim));
    assert_eq!(renamed.equal_from, Some(Level::LiteralTier));
    let first = renamed.first.expect("a first difference");
    assert_eq!(
        (
            first.index,
            first.a.map(|t| t.text),
            first.b.map(|t| t.text)
        ),
        (1, Some("a".into()), Some("b".into()))
    );

    let apart = n
        .explain(
            &Slice::new("f.rs", 40, "let x = 1;\nf(x, 2)"),
            &Slice::new("f.rs", 70, "let x = 1;\nf(x, 3)\n"),
        )
        .expect("lexes");
    assert_eq!(apart.differs_through, Some(Level::Value));
    assert_eq!(apart.equal_from, None);
    let first = apart.first.expect("a first difference");
    assert_eq!(first.index, 9);
    let (a, b) = (first.a.expect("a token"), first.b.expect("a token"));
    assert_eq!((a.text.as_str(), a.line), ("2", 41));
    assert_eq!((b.text.as_str(), b.line), ("3", 71));
    assert_eq!(&"let x = 1;\nf(x, 2)"[a.at.clone()], "2");

    let longer = n
        .explain(
            &Slice::new("f.rs", 1, "f(1)"),
            &Slice::new("f.rs", 1, "f(1) + 1"),
        )
        .expect("lexes");
    let first = longer.first.expect("a first difference");
    assert!(first.a.is_none() && first.b.is_some());
}

#[test]
fn the_digest_is_blake3_of_length_prefixed_tokens() {
    // `f(x, 1)` is `f ( $0 , 1 )`. The expected bytes are written out by hand:
    // each token is its UTF-8 length as four little-endian bytes, then the
    // token.
    let mut bytes: Vec<u8> = Vec::new();
    for token in ["f", "(", "$0", ",", "1", ")"] {
        bytes.extend_from_slice(&u32::try_from(token.len()).expect("short").to_le_bytes());
        bytes.extend_from_slice(token.as_bytes());
    }
    assert_eq!(bytes.len(), 6 * 4 + 7);
    let expected = purrdf_hash::blake3::hash(&bytes);
    assert_eq!(
        purrdf_hash::hex::Lower(expected.as_bytes()).to_string(),
        "abac387fdaf62db5bee4e554a8fe7d4fc574f2e61594275430ddab702ef76288",
        "the digest of the hand-written bytes, computed with b3sum"
    );
    let print = print_of(&["x"], "f(x, 1)", 1);
    let body = print.body.expect("a body print");
    assert_eq!(body, Fingerprint::body(expected.as_bytes()));
    assert_eq!(
        body.to_string(),
        "body:abac387fdaf62db5bee4e554a8fe7d4fc574f2e61594275430ddab702ef76288"
    );
    assert_eq!(
        body.to_string(),
        format!("body:{}", purrdf_hash::hex::Lower(expected.as_bytes()))
    );
}

#[test]
fn a_fingerprint_displays_and_parses_back() {
    let accepted = [
        "body:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "shim:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "body:0000000000000000000000000000000000000000000000000000000000000000",
        "table:hex-lower",
        "table:hex-upper",
        "rule:std-default-hasher",
    ];
    for text in accepted {
        let parsed: Fingerprint = text.parse().expect(text);
        assert_eq!(parsed.to_string(), text);
    }
    assert_eq!(
        "table:hex-lower".parse::<Fingerprint>().expect("parses"),
        Fingerprint::Table(TableKind::HexLower)
    );
    assert_eq!(
        "body:0000000000000001000000000000000100000000000000010000000000000001"
            .parse::<Fingerprint>()
            .expect("parses"),
        Fingerprint::Body({
            let mut bytes = [0; 32];
            for index in [7, 15, 23, 31] {
                bytes[index] = 1;
            }
            bytes
        })
    );
}

#[test]
fn human_abbreviations_do_not_replace_complete_identity() {
    let first = [0; 32];
    let mut second = first;
    second[31] = 1;
    let a = Fingerprint::body(&first);
    let b = Fingerprint::body(&second);
    assert_ne!(a, b);
    assert_ne!(a.to_string(), b.to_string());
    assert_eq!(a.abbreviated(), "body:0000000000000000…");
    assert_eq!(a.abbreviated(), b.abbreviated());
    assert!(a.abbreviated().parse::<Fingerprint>().is_err());
    assert!("body:0000000000000000".parse::<Fingerprint>().is_err());
    assert_eq!(a.to_string().parse::<Fingerprint>().expect("canonical"), a);
    assert_eq!(
        Fingerprint::Table(TableKind::HexLower).abbreviated(),
        "table:hex-lower"
    );
}

#[test]
fn fingerprint_refusals_escape_untrusted_controls_beside_valid_names() {
    for text in ["\u{1b}", "\u{1b}:x", "table:\u{1b}", "rule:\u{1b}"] {
        let message = text
            .parse::<Fingerprint>()
            .expect_err("control refused")
            .to_string();
        assert!(!message.contains('\u{1b}'));
        assert!(message.contains("\\u{1b}"), "{message}");
    }
    let bad_hex = format!("body:{}\u{1b}", "0".repeat(63));
    let message = bad_hex
        .parse::<Fingerprint>()
        .expect_err("hex control refused")
        .to_string();
    assert!(!message.contains('\u{1b}'));
    assert!(message.contains("\\u{1b}"));
    assert!(
        format!("body:{}", "0".repeat(64))
            .parse::<Fingerprint>()
            .is_ok()
    );
    assert!("table:hex-lower".parse::<Fingerprint>().is_ok());
    assert!("rule:valid-name".parse::<Fingerprint>().is_ok());
}

#[test]
fn literal_value_introspection_selects_the_production_default() {
    for fixture in registry().fixtures {
        let tower = tower(&fixture);
        let (retained, abstracted) = tower.literal_values();
        assert_eq!(retained.len(), tower.value_tokens(), "{}", fixture.id);
        assert_eq!(abstracted.len(), tower.value_tokens(), "{}", fixture.id);
        let selected = if tower.value_tokens() < raf_census::LITERAL_TIER {
            retained
        } else {
            abstracted
        };
        assert_eq!(
            selected.texts(),
            tower.form(Level::Value).texts(),
            "{}",
            fixture.id
        );
    }
}

#[test]
fn self_callee_query_distinguishes_generic_types_from_calls() {
    let cases = [
        ("take::<Self::Assoc<fn()>>()", false),
        ("take::<Self::Assoc<fn() -> u8>>()", false),
        ("Self::go::<'a>(x)", true),
        ("Self::go::<fn() -> u8>(x)", true),
        ("Self::go::<for<'a> fn(&'a u8) -> u8>(x)", true),
        ("take::<Self::Assoc<for<'a> fn(&'a u8) -> u8>>()", false),
    ];
    let actual = cases
        .iter()
        .map(|(text, _)| {
            let tower = Tower::new(&Slice::new("fixture.rs", 1, text)).expect("lexes");
            (*text, tower.calls_through_self())
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, cases);
}

/// The check a refusal must satisfy.
type Refusal = fn(&ParseFingerprintError) -> bool;

#[test]
fn a_fingerprint_refuses_every_malformed_spelling() {
    let refused: [(&str, Refusal); 11] = [
        ("source:0123456789abcdef", |e| {
            matches!(e, ParseFingerprintError::Kind(_))
        }),
        ("0123456789abcdef", |e| {
            matches!(e, ParseFingerprintError::NoKind(_))
        }),
        ("body:0123456789abcde", |e| {
            matches!(e, ParseFingerprintError::Width(15))
        }),
        (
            "body:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0",
            |e| matches!(e, ParseFingerprintError::Width(65)),
        ),
        (
            "body:0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF",
            |e| matches!(e, ParseFingerprintError::Hex(_)),
        ),
        (
            "shim:0123456789abcdeg0123456789abcdeg0123456789abcdeg0123456789abcdeg",
            |e| matches!(e, ParseFingerprintError::Hex(_)),
        ),
        (
            "body:+123456789abcdef+123456789abcdef+123456789abcdef+123456789abcdef",
            |e| matches!(e, ParseFingerprintError::Hex(_)),
        ),
        ("table:hex-mixed", |e| {
            matches!(e, ParseFingerprintError::Table(_))
        }),
        ("table:", |e| matches!(e, ParseFingerprintError::Table(_))),
        ("rule:", |e| matches!(e, ParseFingerprintError::Rule(_))),
        ("rule:has space", |e| {
            matches!(e, ParseFingerprintError::Rule(_))
        }),
    ];
    for (text, expected) in refused {
        let error = text.parse::<Fingerprint>().expect_err(text);
        assert!(expected(&error), "{text}: {error:?}");
        assert_ne!(error.to_string(), "");
    }
}

#[test]
fn fingerprints_sort_by_kind_then_value() {
    let mut all: Vec<Fingerprint> = [
        "rule:a",
        "table:hex-upper",
        "shim:0000000000000002000000000000000200000000000000020000000000000002",
        "body:0000000000000009000000000000000900000000000000090000000000000009",
        "body:0000000000000001000000000000000100000000000000010000000000000001",
    ]
    .iter()
    .map(|text| text.parse().expect("parses"))
    .collect();
    all.sort();
    let shown: Vec<String> = all.iter().map(ToString::to_string).collect();
    assert_eq!(
        shown,
        [
            "body:0000000000000001000000000000000100000000000000010000000000000001",
            "body:0000000000000009000000000000000900000000000000090000000000000009",
            "shim:0000000000000002000000000000000200000000000000020000000000000002",
            "table:hex-upper",
            "rule:a"
        ]
    );
}

#[test]
fn doc_comments_are_attribute_tokens_while_ordinary_comments_are_erased() {
    let plain =
        Tower::new(&Slice::new("plain.rs", 1, "let value = input + 1; value")).expect("plain body");
    let ordinary = Tower::new(&Slice::new(
        "ordinary.rs",
        1,
        "/* ordinary */ let value = input + 1; // another comment\nvalue",
    ))
    .expect("ordinary comments");
    for level in Level::ALL {
        assert_eq!(texts(&ordinary, level), texts(&plain, level), "{level:?}");
    }
    let documented = Tower::new(&Slice::new(
        "documented.rs",
        1,
        "/// meaning\nlet value = input + 1; value",
    ))
    .expect("documentation comment");
    let changed = Tower::new(&Slice::new(
        "changed.rs",
        1,
        "/// different\nlet value = input + 1; value",
    ))
    .expect("changed documentation comment");
    let value = texts(&documented, Level::Value);
    assert!(value.join(" ").contains("# [ doc = \" meaning\" ]"));
    assert_ne!(value, texts(&plain, Level::Value));
    assert_ne!(value, texts(&changed, Level::Value));
}

#[test]
fn local_binding_names_apply_to_the_whole_body_beside_preserved_unbound_items() {
    let form = |body| {
        texts(
            &Tower::new(&Slice::new("fixture.rs", 1, body)).expect("body lexes"),
            Level::Value,
        )
    };
    assert_eq!(
        form("first(); { let first = seed(); consume(first); } first()"),
        form("second(); { let second = seed(); consume(second); } second()")
    );
    assert_ne!(
        form("first(); { let local = seed(); consume(local); } first()"),
        form("second(); { let local = seed(); consume(local); } second()")
    );
}

#[test]
fn the_names_of_an_attribute_are_kept_beside_a_local_of_the_same_name() {
    let slice = Slice::new(
        "fixture.rs",
        1,
        "/// the total\nlet doc = x + 1; #[allow(unused)] doc",
    )
    .params(["x"]);
    let tower = Tower::new(&slice).expect("lexes");
    let text = texts(&tower, Level::Value).join(" ");
    // The doc comment is the attribute `# [ doc = "..." ]` with `doc` kept, and
    // the local named `doc` is renamed everywhere else.
    assert!(text.contains("# [ doc = \" the total\" ]"), "{text}");
    assert!(text.contains("# [ allow ( unused ) ]"), "{text}");
    assert!(text.contains("let $0 = $1 + 1"), "{text}");
}

#[test]
fn every_threshold_counts_the_value_form() {
    let registry = registry();
    for fixture in &registry.fixtures {
        let tower = tower(fixture);
        assert_eq!(
            tower.value_tokens(),
            tower.form(Level::Value).len(),
            "{}",
            fixture.id
        );
    }
    let adapters = ".clone().as_slice()";
    // The literal tier: 21 value tokens either way, so literals stay in both.
    let tier =
        |adapters: &str| format!("let n = v{adapters}.len(); if n > 3 {{ return 7; }} n + 1");
    let (plain, noisy) = (
        print_of(&["v"], &tier(""), 1),
        print_of(&["v"], &tier(adapters), 1),
    );
    assert_eq!((plain.tokens, noisy.tokens), (21, 21));
    assert_eq!(plain.body, noisy.body);
    // `min_tokens`: 15 value tokens either way.
    let small = |adapters: &str| format!("if v{adapters}.len() > 3 {{ 1 }} else {{ 0 }}");
    for min in [10, 15] {
        let (plain, noisy) = (
            print_of(&["v"], &small(""), min),
            print_of(&["v"], &small(adapters), min),
        );
        assert!(plain.body.is_some() && plain.body == noisy.body, "{min}");
    }
    for min in [16, 20] {
        let (plain, noisy) = (
            print_of(&["v"], &small(""), min),
            print_of(&["v"], &small(adapters), min),
        );
        assert!(plain.body.is_none() && noisy.body.is_none(), "{min}");
    }
    // The shim bounds: 29 value tokens keep a shim form, 5 do not, whatever
    // the adapters around them.
    let upper = registry
        .fixtures
        .iter()
        .find(|f| f.id == "shim_29")
        .expect("registered");
    let noisy_upper = format!("{}{adapters}", upper.text);
    assert!(print_of(&["a"], &upper.text, 1).shim.is_some());
    assert!(print_of(&["a"], &noisy_upper, 1).shim.is_some());
    assert!(print_of(&["a"], "a.f()", 1).shim.is_none());
    assert!(
        print_of(&["a"], &format!("a.f(){adapters}"), 1)
            .shim
            .is_none()
    );
}

#[test]
fn copies_with_reordered_parameters_share_a_body_print_and_forwarders_keep_their_order() {
    let body = "let mut n = 0; for x in xs { if x == k { n += 1; } } n";
    let a = print_of(&["xs", "k"], body, 1);
    let b = print_of(&["k", "xs"], body, 1);
    assert_eq!(a.body, b.body);
    let in_order = print_of(&["x", "y"], "target(x, y, 1)", 1);
    let swapped = print_of(&["x", "y"], "target(y, x, 1)", 1);
    assert_eq!(in_order.body, swapped.body, "the body form numbers by use");
    assert!(in_order.shim.is_some() && swapped.shim.is_some());
    assert_ne!(in_order.shim, swapped.shim, "the shim form keeps the order");
}

#[test]
fn explain_reports_reordered_parameters_as_equal_from_the_literal_tier() {
    let body = "let mut n = 0; for x in xs { if x == k { n += 1; } } n";
    let a = Slice::new("f.rs", 1, body).params(["xs", "k"]);
    let b = Slice::new("f.rs", 1, body).params(["k", "xs"]);
    let divergence = normalizer(1).explain(&a, &b).expect("lexes");
    assert_eq!(divergence.equal_from, Some(Level::LiteralTier));
    assert_eq!(divergence.differs_through, Some(Level::Shim));
}
