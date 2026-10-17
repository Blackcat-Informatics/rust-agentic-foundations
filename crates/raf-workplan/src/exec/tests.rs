// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The walk over every variant of [`Arg`]: whatever text a variant is given,
//! a value that fails its pattern never becomes an argument.

use super::{
    Arg, Commit, DocFilter, ExampleName, Filter, Identifier, LineRange, Literal, Package, Pattern,
    PatternError, RefName, SourcePath, Target, TargetName, TreePath,
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
        Arg::Path(_) => "path",
        Arg::LineLog { .. } => "line-log",
        Arg::Pickaxe(_) => "pickaxe",
        Arg::Trusted(_) => "trusted",
    }
}

/// Every name [`variant`] can return.
const VARIANTS: [&str; 15] = [
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
    "path",
    "line-log",
    "pickaxe",
    "trusted",
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
        ("path", Pattern::TreePath, |v| {
            TreePath::new(v).map(Arg::Path)
        }),
        ("line-log", Pattern::TreePath, |v| {
            TreePath::new(v).map(|file| Arg::LineLog {
                lines: LineRange::new(1, 2).expect("a valid range"),
                file,
            })
        }),
        ("pickaxe", Pattern::Identifier, |v| {
            Identifier::new(v).map(Arg::Pickaxe)
        }),
    ]
}

/// Text to build an argument from, with every pattern it matches.
fn corpus() -> Vec<(String, Vec<Pattern>)> {
    use Pattern::{
        Commit, Date, Identifier, ItemPath, Name, RefName, SourcePath, TargetName, TreePath,
    };
    let all = |patterns: &[Pattern]| patterns.to_vec();
    let mut values: Vec<(String, Vec<Pattern>)> = vec![
        (
            "alpha".into(),
            all(&[Name, TargetName, ItemPath, SourcePath, TreePath, Identifier]),
        ),
        (
            "a_b".into(),
            all(&[Name, TargetName, ItemPath, SourcePath, TreePath, Identifier]),
        ),
        (
            "_x".into(),
            all(&[Name, TargetName, ItemPath, SourcePath, TreePath, Identifier]),
        ),
        ("9lives".into(), all(&[TargetName, SourcePath, TreePath])),
        ("my-crate".into(), all(&[TargetName, SourcePath, TreePath])),
        ("a::b::c".into(), all(&[Name, ItemPath, TreePath])),
        ("Wrapper<T>::get".into(), all(&[ItemPath, TreePath])),
        ("Wrapper<&'a T>::get".into(), all(&[TreePath])),
        ("Wrapper<&'a_T>::get".into(), all(&[ItemPath, TreePath])),
        ("src/lib.rs".into(), all(&[SourcePath, TreePath])),
        (
            "docs/capabilities/a-b.md".into(),
            all(&[SourcePath, TreePath]),
        ),
        (
            "refs/remotes/origin/main".into(),
            all(&[RefName, SourcePath, TreePath]),
        ),
        (
            "refs/remotes/origin/--output=x".into(),
            all(&[RefName, TreePath]),
        ),
        (
            COMMIT.into(),
            all(&[Commit, TargetName, SourcePath, TreePath]),
        ),
        (
            "2026-02-28".into(),
            all(&[Date, TargetName, SourcePath, TreePath]),
        ),
        (
            "2026-02-30".into(),
            all(&[TargetName, SourcePath, TreePath]),
        ),
        // Hostile and malformed. A tree path takes many of them, because it
        // is only ever placed after `--` or after the colon of `-L`.
        (String::new(), all(&[])),
        ("-Zunstable-options".into(), all(&[TreePath])),
        ("--exact".into(), all(&[TreePath])),
        ("--include-ignored".into(), all(&[TreePath])),
        ("-rf".into(), all(&[TreePath])),
        ("-".into(), all(&[TreePath])),
        ("a;b".into(), all(&[TreePath])),
        ("a b".into(), all(&[TreePath])),
        (" a".into(), all(&[TreePath])),
        ("a ".into(), all(&[TreePath])),
        ("a\nb".into(), all(&[])),
        ("a\0b".into(), all(&[])),
        ("a=b".into(), all(&[TreePath])),
        ("$(touch x)".into(), all(&[TreePath])),
        ("`id`".into(), all(&[TreePath])),
        ("a|b".into(), all(&[TreePath])),
        ("a&b".into(), all(&[TreePath])),
        ("*".into(), all(&[])),
        ("a::".into(), all(&[TreePath])),
        ("::a".into(), all(&[])),
        ("a:::b".into(), all(&[TreePath])),
        ("a:b".into(), all(&[TreePath])),
        ("é".into(), all(&[TreePath])),
        ("naïve".into(), all(&[TreePath])),
        ("名前".into(), all(&[TreePath])),
        ("a\u{202e}b".into(), all(&[TreePath])),
        ("../x".into(), all(&[])),
        ("a/../b".into(), all(&[])),
        ("/etc/passwd".into(), all(&[])),
        ("refs/heads/../x".into(), all(&[])),
        (
            "main".into(),
            all(&[Name, TargetName, ItemPath, SourcePath, TreePath, Identifier]),
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
        Arg::Path(TreePath::new("-rf.rs").unwrap()),
        Arg::LineLog {
            lines: LineRange::new(3, 9).unwrap(),
            file: TreePath::new("src/lib.rs").unwrap(),
        },
        Arg::Pickaxe(Identifier::new("gone").unwrap()),
        Arg::Trusted(
            super::TrustedArgv::new(vec!["true".to_string()])
                .unwrap()
                .words()[0]
                .clone(),
        ),
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
        // A literal is fixed text, `--lib` holds nothing, and a trusted word
        // comes only from the command line of the process.
        if !matches!(name, "literal" | "target-lib" | "trusted") {
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
                    if pattern == Pattern::TreePath {
                        // A tree path may look like an option or hold any
                        // printable character: it is a path after `--`, or
                        // the tail of one `-L<start>,<end>:` argument.
                        assert!(
                            rendered == text || rendered == format!("-L1,2:{text}"),
                            "`{rendered}` places the tree path {text:?} where an option is read"
                        );
                        assert!(
                            !text.chars().any(char::is_control),
                            "{text:?} holds a control character"
                        );
                    } else {
                        assert!(!text.starts_with('-'), "{text:?} was accepted");
                        assert!(
                            text.chars().all(|c| c.is_ascii_graphic())
                                && !text.contains([';', '|', '$', '`', '*', '(', ')', '"']),
                            "{text:?} holds a character no pattern allows"
                        );
                    }
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
