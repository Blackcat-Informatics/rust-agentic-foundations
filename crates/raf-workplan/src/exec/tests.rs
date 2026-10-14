// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The walk over every variant of [`Arg`]: whatever text a variant is given,
//! a value that fails its pattern never becomes an argument.

use super::{
    Arg, Commit, DocFilter, ExampleName, Filter, Literal, Package, Pattern, PatternError, RefName,
    SourcePath, Target, TargetName,
};

/// The name of a variant. The match has no wildcard arm, so a variant added to
/// [`Arg`] stops this file compiling until it is walked below.
fn variant(arg: &Arg) -> &'static str {
    match arg {
        Arg::Literal(_) => "literal",
        Arg::Package(_) => "package",
        Arg::Target(Target::Lib) => "target-lib",
        Arg::Target(Target::Bin(_)) => "target-bin",
        Arg::Target(Target::Test(_)) => "target-test",
        Arg::Example(_) => "example",
        Arg::Filter(_) => "filter",
        Arg::DocFilter(_) => "doc-filter",
        Arg::RefCommit(_) => "ref-commit",
        Arg::Commit(_) => "commit",
        Arg::RevPath { .. } => "rev-path",
    }
}

/// Every name [`variant`] can return.
const VARIANTS: [&str; 11] = [
    "literal",
    "package",
    "target-lib",
    "target-bin",
    "target-test",
    "example",
    "filter",
    "doc-filter",
    "ref-commit",
    "commit",
    "rev-path",
];

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

type Make = fn(&str) -> Result<Arg, PatternError>;

/// How each value-bearing variant is built from a piece of text, with the
/// pattern the text must match. `rev-path` appears twice, once for each of
/// its fields.
fn builders() -> Vec<(&'static str, Pattern, Make)> {
    vec![
        ("package", Pattern::TargetName, |v| {
            Package::new(v).map(Arg::Package)
        }),
        ("target-bin", Pattern::TargetName, |v| {
            TargetName::new(v).map(|name| Arg::Target(Target::Bin(name)))
        }),
        ("target-test", Pattern::TargetName, |v| {
            TargetName::new(v).map(|name| Arg::Target(Target::Test(name)))
        }),
        ("example", Pattern::TargetName, |v| {
            ExampleName::new(v).map(Arg::Example)
        }),
        ("filter", Pattern::Name, |v| Filter::new(v).map(Arg::Filter)),
        ("doc-filter", Pattern::ItemPath, |v| {
            DocFilter::new(v).map(Arg::DocFilter)
        }),
        ("ref-commit", Pattern::RefName, |v| {
            RefName::new(v).map(Arg::RefCommit)
        }),
        ("commit", Pattern::Commit, |v| {
            Commit::new(v).map(Arg::Commit)
        }),
        ("rev-path", Pattern::SourcePath, |v| {
            SourcePath::new(v).map(|path| Arg::RevPath {
                rev: Commit::new(COMMIT).expect("a valid commit id"),
                path,
            })
        }),
        ("rev-path", Pattern::Commit, |v| {
            Commit::new(v).map(|rev| Arg::RevPath {
                rev,
                path: SourcePath::new("docs/a.md").expect("a valid path"),
            })
        }),
    ]
}

/// Text to build an argument from, with every pattern it matches.
fn corpus() -> Vec<(String, Vec<Pattern>)> {
    use Pattern::{Commit, ItemPath, Name, RefName, SourcePath, TargetName};
    let all = |patterns: &[Pattern]| patterns.to_vec();
    let mut values: Vec<(String, Vec<Pattern>)> = vec![
        (
            "alpha".into(),
            all(&[Name, TargetName, ItemPath, SourcePath]),
        ),
        ("a_b".into(), all(&[Name, TargetName, ItemPath, SourcePath])),
        ("_x".into(), all(&[Name, TargetName, ItemPath, SourcePath])),
        ("9lives".into(), all(&[TargetName, SourcePath])),
        ("my-crate".into(), all(&[TargetName, SourcePath])),
        ("a::b::c".into(), all(&[Name, ItemPath])),
        ("Wrapper<T>::get".into(), all(&[ItemPath])),
        ("Wrapper<&'a T>::get".into(), all(&[])),
        ("Wrapper<&'a_T>::get".into(), all(&[ItemPath])),
        ("src/lib.rs".into(), all(&[SourcePath])),
        ("docs/capabilities/a-b.md".into(), all(&[SourcePath])),
        (
            "refs/remotes/origin/main".into(),
            all(&[RefName, SourcePath]),
        ),
        ("refs/remotes/origin/--output=x".into(), all(&[RefName])),
        (COMMIT.into(), all(&[Commit, TargetName, SourcePath])),
        // Hostile and malformed.
        (String::new(), all(&[])),
        ("-Zunstable-options".into(), all(&[])),
        ("--exact".into(), all(&[])),
        ("--include-ignored".into(), all(&[])),
        ("-rf".into(), all(&[])),
        ("-".into(), all(&[])),
        ("a;b".into(), all(&[])),
        ("a b".into(), all(&[])),
        (" a".into(), all(&[])),
        ("a ".into(), all(&[])),
        ("a\nb".into(), all(&[])),
        ("a\0b".into(), all(&[])),
        ("a=b".into(), all(&[])),
        ("$(touch x)".into(), all(&[])),
        ("`id`".into(), all(&[])),
        ("a|b".into(), all(&[])),
        ("a&b".into(), all(&[])),
        ("*".into(), all(&[])),
        ("a::".into(), all(&[])),
        ("::a".into(), all(&[])),
        ("a:::b".into(), all(&[])),
        ("a:b".into(), all(&[])),
        ("é".into(), all(&[])),
        ("naïve".into(), all(&[])),
        ("名前".into(), all(&[])),
        ("a\u{202e}b".into(), all(&[])),
        ("../x".into(), all(&[])),
        ("a/../b".into(), all(&[])),
        ("/etc/passwd".into(), all(&[])),
        ("refs/heads/../x".into(), all(&[])),
        (
            "main".into(),
            all(&[Name, TargetName, ItemPath, SourcePath]),
        ),
    ];
    values.sort_by(|a, b| a.0.cmp(&b.0));
    values.dedup_by(|a, b| a.0 == b.0);
    values
}

#[test]
fn every_variant_is_walked() {
    let samples = [
        Arg::Literal(Literal::new("--frozen")),
        Arg::Package(Package::new("p").unwrap()),
        Arg::Target(Target::Lib),
        Arg::Target(Target::Bin(TargetName::new("b").unwrap())),
        Arg::Target(Target::Test(TargetName::new("t").unwrap())),
        Arg::Example(ExampleName::new("e").unwrap()),
        Arg::Filter(Filter::new("f").unwrap()),
        Arg::DocFilter(DocFilter::new("d").unwrap()),
        Arg::RefCommit(RefName::new("refs/heads/main").unwrap()),
        Arg::Commit(Commit::new(COMMIT).unwrap()),
        Arg::RevPath {
            rev: Commit::new(COMMIT).unwrap(),
            path: SourcePath::new("a.md").unwrap(),
        },
    ];
    let mut names: Vec<&str> = samples.iter().map(variant).collect();
    names.sort_unstable();
    let mut expected = VARIANTS.to_vec();
    expected.sort_unstable();
    assert_eq!(names, expected);
    // Every value-bearing variant has a builder in the walk below.
    let built: std::collections::BTreeSet<&str> =
        builders().iter().map(|(name, _, _)| *name).collect();
    for name in VARIANTS {
        if !matches!(name, "literal" | "target-lib") {
            assert!(built.contains(name), "no builder for `{name}`");
        }
    }
}

#[test]
fn a_rejected_value_never_becomes_an_argument() {
    let mut accepted = 0;
    let mut rejected = 0;
    for (name, pattern, make) in builders() {
        for (text, matching) in corpus() {
            let expected = matching.contains(&pattern);
            let built = make(&text);
            assert_eq!(
                built.is_ok(),
                expected,
                "`{name}` built from {text:?} must be {} by {pattern:?}",
                if expected { "accepted" } else { "rejected" },
            );
            match built {
                Ok(arg) => {
                    accepted += 1;
                    assert_eq!(variant(&arg), name);
                    let rendered = arg.to_string();
                    assert!(
                        rendered.contains(&text),
                        "`{rendered}` must carry its value {text:?} unchanged"
                    );
                    assert!(!text.starts_with('-'), "{text:?} was accepted");
                    assert!(
                        text.chars().all(|c| c.is_ascii_graphic())
                            && !text.contains([';', '|', '$', '`', '*', '(', ')', '"']),
                        "{text:?} holds a character no pattern allows"
                    );
                }
                Err(error) => {
                    rejected += 1;
                    assert_eq!(error.pattern(), pattern);
                    assert_ne!(error.to_string(), "");
                }
            }
        }
    }
    // Both directions are exercised, so neither the accepting nor the
    // rejecting half can vanish unseen.
    assert!(accepted > 30, "only {accepted} values were accepted");
    assert!(rejected > 150, "only {rejected} values were rejected");
}

#[test]
fn a_literal_is_the_only_free_text_and_it_is_static() {
    // `Literal` takes a `&'static str`: text read at run time cannot be one.
    let arg = Arg::Literal(Literal::new("--frozen"));
    assert_eq!(arg.to_string(), "--frozen");
    assert_eq!(variant(&arg), "literal");
}

/// The real runner with arguments that only a crate-private literal can carry:
/// every one reaches the process as one element, and no shell reads them.
#[cfg(unix)]
mod no_shell {
    use std::fs;
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;

    use super::super::{Invocation, Limits, Program, Programs, Runner, SystemRunner};
    use super::{Arg, Literal};

    #[test]
    fn every_argument_is_one_element_and_no_shell_reads_it() {
        let root =
            std::env::temp_dir().join(format!("raf-workplan-exec-unit-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let stub = root.join("record");
        {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o755)
                .open(&stub)
                .unwrap();
            write!(
                file,
                "#!/bin/sh\nprintf '%s\\0' \"$@\" > \"${{0%/*}}/argv\"\n"
            )
            .unwrap();
        }
        let hostile = "a b;touch pwned $(touch pwned) `touch pwned` *";
        let args = [
            "--",
            hostile,
            "",
            "line\nbreak",
            "~",
            "$HOME",
            "-Zunstable-options",
        ];
        let invocation = Invocation::new(
            Program::Cargo,
            args.into_iter()
                .map(|text| Arg::Literal(Literal::new(text)))
                .collect(),
        );
        let runner = SystemRunner::new(Programs::with_cargo_var(&root, Some(stub.clone().into())));
        let output = runner.run(&invocation, &Limits::query()).unwrap();
        assert!(output.success(), "{output:?}");
        let bytes = fs::read(root.join("argv")).unwrap();
        let mut seen: Vec<&[u8]> = bytes.split(|b| *b == 0).collect();
        assert_eq!(seen.pop(), Some(&b""[..]));
        let expected: Vec<&[u8]> = args.iter().map(|text| text.as_bytes()).collect();
        assert_eq!(seen, expected);
        assert!(!root.join("pwned").exists());
        let _ = fs::remove_dir_all(&root);
    }
}
