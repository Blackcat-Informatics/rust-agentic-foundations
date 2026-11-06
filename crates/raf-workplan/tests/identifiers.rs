// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed identifiers used by native data readers.

use raf_workplan::identifiers::{
    Commit, DocFilter, ExampleName, Filter, Package, Pattern, PatternError, RefName, SourcePath,
    TargetName,
};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const COMMIT_256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

// ---------------------------------------------------------------- patterns

/// Values each pattern accepts and refuses. Every refusal is paired with the
/// accepted neighbour it is nearest to.
struct Table {
    pattern: Pattern,
    accepted: &'static [&'static str],
    refused: &'static [&'static str],
}

const HOSTILE: &[&str] = &[
    "",
    "-Zunstable-options",
    "--exact",
    "--include-ignored",
    "-rf",
    "-",
    "a;b",
    "a b",
    " a",
    "a ",
    "a\tb",
    "a\nb",
    "a\0b",
    "$(touch x)",
    "`id`",
    "a|b",
    "a&&b",
    "a>b",
    "a\"b",
    "a'b",
    "a\\b",
    "a*",
    "é",
    "naïve",
    "名前",
    "a\u{202e}b",
    "a\u{ff1a}\u{ff1a}b",
];

const TABLES: [Table; 9] = [
    Table {
        pattern: Pattern::Name,
        accepted: &[
            "a",
            "_",
            "a1",
            "a_b",
            "A::b",
            "a::b::c",
            "_x::_y9",
            "tests::it_works",
        ],
        refused: &[
            "1a",
            "a-b",
            "a::",
            "::a",
            "a:::b",
            "a:b",
            "a::1",
            "a::b-c",
            "a.b",
            "a/b",
            "a<T>",
            "a<T>::get",
            "a::b ",
            "r#type!",
        ],
    },
    Table {
        pattern: Pattern::TargetName,
        accepted: &[
            "a", "1", "_", "_a", "9lives", "my-crate", "a-b-c", "a_b-c", "A-",
        ],
        refused: &[
            "-a",
            "-",
            "a.b",
            "a/b",
            "a::b",
            "a=b",
            "--package=x",
            "a b",
            "a+",
        ],
    },
    Table {
        pattern: Pattern::ItemPath,
        accepted: &[
            "a",
            "a::b",
            "Wrapper<T>::get",
            "Wrapper<T>",
            "Wrapper<>::get",
            "Wrapper<'a>::get",
            "Wrapper<&'a_T>::get",
            "Map<K,V>::insert",
            "a::Wrapper<a::B>::get",
            "a<b>::c<d>::e",
            "a<x:y>",
        ],
        refused: &[
            "<T>",
            "Wrapper<T",
            "Wrapper<T>>",
            "Wrapper<<T>>",
            "Wrapper<T> ::get",
            "Wrapper<T>get",
            "Wrapper<T>::",
            "Wrapper<a b>::get",
            "Wrapper<&'a T>::get",
            "Wrapper<T-U>::get",
            "Wrapper<T;U>::get",
            "Wrapper<T>::-x",
            "-Wrapper<T>",
            "a::b<",
            "a<b>c",
            "1a::b",
            "a::1b",
            "a- b",
        ],
    },
    Table {
        pattern: Pattern::SourcePath,
        accepted: &[
            "a",
            "_",
            "9",
            "src/lib.rs",
            "src/a_b/c-d.rs",
            "docs/capabilities/workplan-cli.md",
            "a.b/c.d",
            "a..b",
        ],
        refused: &[
            "-a",
            ".a",
            ".",
            "..",
            "../a",
            "a/..",
            "a/../b",
            "a/./b",
            "a//b",
            "/a",
            "a b",
            "a:b",
            "a\\b",
            "a?b",
            "a=b",
            "src/é.rs",
            "a/b/",
        ],
    },
    Table {
        pattern: Pattern::RefName,
        accepted: &[
            "refs/remotes/origin/main",
            "refs/remotes/origin/feature/x-1",
            "refs/remotes/origin/--output=x",
            "refs/remotes/origin/-Zunstable-options",
            "refs/remotes/origin/v1.2.3",
            "refs/heads/ünï",
        ],
        refused: &[
            "origin/main",
            "refs",
            "refs/",
            "refs//x",
            "refs/heads/",
            "refs/heads/a..b",
            "refs/heads/.hidden",
            "refs/heads/a.",
            "refs/heads/x.lock",
            "refs/heads/a b",
            "refs/heads/a~1",
            "refs/heads/a^",
            "refs/heads/a:b",
            "refs/heads/a?",
            "refs/heads/a*",
            "refs/heads/a[1]",
            "refs/heads/a\\b",
            "refs/heads/a@{1}",
            "refs/heads/a\nb",
            "refs/heads/a;b",
            "-refs/heads/a",
            "--end-of-options",
        ],
    },
    Table {
        pattern: Pattern::Commit,
        accepted: &[COMMIT, COMMIT_256],
        refused: &[
            "",
            "0123",
            "0123456789ABCDEF0123456789ABCDEF01234567",
            "0123456789abcdef0123456789abcdef0123456",
            "0123456789abcdef0123456789abcdef012345678",
            "0123456789abcdef0123456789abcdef0123456g",
            "HEAD",
            "-0123456789abcdef0123456789abcdef0123456",
            "0123456789abcdef0123456789abcdef01234567 ",
            "0123456789abcdef0123456789abcdef01234567:x",
        ],
    },
    Table {
        pattern: Pattern::TreePath,
        accepted: &[
            "a",
            "src/lib.rs",
            "-rf.rs",
            "--output=x.rs",
            "-",
            "src/-rf.rs",
            "a b.rs",
            "src/\u{e9}t\u{e9}.rs",
            "a:b",
            "a=b",
            "a..b",
            "a;b",
            ".hidden",
        ],
        refused: &[
            "", ".", "..", "../a", "a/..", "a/../b", "a/./b", "a//b", "a/", "/a", ":a", ":(glob)*",
            "a*", "a?", "a[1]", "a\\b", "a\nb", "a\rb", "a\tb", "a\0b", "a\u{1b}b", "a\u{7f}b",
            "a\u{85}b",
        ],
    },
    Table {
        pattern: Pattern::Identifier,
        accepted: &["a", "_", "_a", "gone", "Gone9", "snake_case"],
        refused: &[
            "", "1a", "-a", "a-b", "a::b", "a b", "a<T>", "r#type", "é", "-Sx", "a.b",
        ],
    },
    Table {
        pattern: Pattern::Date,
        accepted: &[
            "2026-01-01",
            "2026-12-31",
            "2028-02-29",
            "2000-02-29",
            "1970-01-01",
            "2099-12-31",
        ],
        refused: &[
            "",
            "-2026-01-01",
            "--all",
            "2026-1-01",
            "2026-01-1",
            "26-01-01",
            "2026/01/01",
            "2026-00-01",
            "2026-13-01",
            "2026-01-00",
            "2026-01-32",
            "2026-02-29",
            "1900-02-29",
            "2026-04-31",
            "2026-01-01 ",
            "2026-01-01T00:00",
            "yesterday",
            "2026-0a-01",
            "1969-12-31",
            "2100-01-01",
            "0001-01-01",
            "2999-01-01",
        ],
    },
];

#[test]
fn every_pattern_accepts_what_it_states() {
    for table in &TABLES {
        for text in table.accepted {
            assert!(
                table.pattern.matches(text),
                "{:?} must accept {text:?}",
                table.pattern
            );
        }
    }
}

#[test]
fn every_pattern_refuses_what_it_does_not_state() {
    for table in &TABLES {
        for text in table.refused {
            assert!(
                !table.pattern.matches(text),
                "{:?} must refuse {text:?}",
                table.pattern
            );
        }
    }
}

#[test]
fn every_pattern_refuses_hostile_values() {
    // The tree path is the one pattern that takes text that looks like an
    // option or holds a shell character: it is placed only after `--` or
    // after the colon of one `-L` argument (see the row tests below), and it
    // still refuses every hostile value that is not printable path text.
    for pattern in Pattern::ALL
        .into_iter()
        .filter(|pattern| *pattern != Pattern::TreePath)
    {
        for text in HOSTILE {
            assert!(
                !pattern.matches(text),
                "{pattern:?} must refuse the hostile value {text:?}"
            );
        }
    }
}

#[test]
fn a_tree_path_refuses_every_hostile_value_that_is_not_printable_path_text() {
    for text in HOSTILE {
        let printable = !text.is_empty()
            && !text
                .chars()
                .any(|c| c.is_control() || matches!(c, '\\' | '*'));
        assert_eq!(
            Pattern::TreePath.matches(text),
            printable,
            "the tree path and {text:?}"
        );
    }
}

#[test]
fn the_patterns_are_the_ones_the_documents_state() {
    assert_eq!(Pattern::TreePath.expression(), "[^/:][^/]*(/[^/]+)*");
    assert_eq!(Pattern::Identifier.expression(), "[A-Za-z_][A-Za-z0-9_]*");
    assert_eq!(
        Pattern::Date.expression(),
        "(19[7-9][0-9]|20[0-9][0-9])-[0-9]{2}-[0-9]{2}"
    );
    assert_eq!(
        Pattern::Name.expression(),
        "[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*"
    );
    assert_eq!(
        Pattern::TargetName.expression(),
        "[A-Za-z0-9_][A-Za-z0-9_-]*"
    );
    assert_eq!(
        Pattern::SourcePath.expression(),
        "[A-Za-z0-9_][A-Za-z0-9_./-]*"
    );
    assert!(
        Pattern::ItemPath
            .expression()
            .contains("<[A-Za-z0-9_,':&]*>")
    );
    // The table above covers every pattern.
    let covered: Vec<Pattern> = TABLES.iter().map(|table| table.pattern).collect();
    for pattern in Pattern::ALL {
        assert!(covered.contains(&pattern), "{pattern:?} has no table");
    }
}

#[test]
fn a_generic_list_holds_exactly_the_characters_of_the_pattern() {
    for allowed in ['a', 'Z', '0', '_', ',', '\'', ':', '&'] {
        let item = format!("W<{allowed}>::get");
        assert!(DocFilter::new(&item).is_ok(), "{item}");
    }
    for refused in [
        '-', ' ', ';', '<', '>', '(', '*', '\\', '"', 'é', '\n', '=', '.',
    ] {
        let item = format!("W<{refused}>::get");
        assert!(DocFilter::new(&item).is_err(), "{item:?}");
    }
}

#[test]
fn each_newtype_is_built_only_by_its_pattern() {
    assert_eq!(Package::new("raf-demo").unwrap().as_str(), "raf-demo");
    assert!(Package::new("-Zunstable-options").is_err());
    assert!(TargetName::new("--exact").is_err());
    assert!(ExampleName::new("chunked-1").is_ok());
    assert!(ExampleName::new("a b").is_err());
    assert!(Filter::new("tests::it_works").is_ok());
    assert!(Filter::new("--include-ignored").is_err());
    assert!(Filter::new("a;b").is_err());
    assert!(DocFilter::new("Wrapper<T>::get").is_ok());
    assert!(DocFilter::new("-x").is_err());
    assert!(SourcePath::new("src/lib.rs").is_ok());
    assert!(SourcePath::new("a/../b").is_err());
    assert!(RefName::new("refs/remotes/origin/main").is_ok());
    assert!(RefName::new("origin/main").is_err());
    assert!(Commit::new(COMMIT).is_ok());
    assert!(Commit::new("HEAD").is_err());
}

#[test]
fn a_refusal_names_the_pattern_and_shows_the_value_safely() {
    let error: PatternError = Filter::new("-Zunstable-options").unwrap_err();
    assert_eq!(error.pattern(), Pattern::Name);
    assert_eq!(error.value(), "-Zunstable-options");
    let text = error.to_string();
    assert!(text.contains("-Zunstable-options"), "{text}");
    assert!(text.contains(Pattern::Name.expression()), "{text}");
    assert!(text.contains("not passed to a process"), "{text}");

    // Control characters are escaped so the message cannot forge a line.
    let text = Package::new("a\nb\u{1b}[31m").unwrap_err().to_string();
    assert!(!text.contains('\n') && !text.contains('\u{1b}'), "{text:?}");

    // A long value is cut.
    let long = "-".repeat(500);
    let error = Package::new(&long).unwrap_err();
    assert!(error.value().chars().count() < 100, "{}", error.value());
    assert!(error.value().ends_with("..."));

    // A neighbour: the value that matches builds.
    assert!(Package::new("raf-demo").is_ok());
}
