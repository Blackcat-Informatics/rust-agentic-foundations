// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Python source, bytecode and executable archives cannot enter the repository.

use std::path::Path;

use super::{gate_root, read_tracked, tracked_paths};

fn python_interpreter(word: &str) -> bool {
    let name = word.rsplit('/').next().unwrap_or(word);
    name.strip_prefix("python").is_some_and(|suffix| {
        suffix.is_empty()
            || suffix
                .bytes()
                .all(|byte| byte.is_ascii_digit() || byte == b'.')
    }) || matches!(name, "pypy" | "pypy2" | "pypy3")
}

fn python_shebang(bytes: &[u8]) -> bool {
    let first = bytes
        .split(|byte| *byte == b'\n')
        .next()
        .unwrap_or_default();
    let Some(line) = first.strip_prefix(b"#!") else {
        return false;
    };
    let line = String::from_utf8_lossy(line);
    let mut words = line.split_ascii_whitespace();
    let Some(interpreter) = words.next() else {
        return false;
    };
    if python_interpreter(interpreter) {
        return true;
    }
    if interpreter.rsplit('/').next() != Some("env") {
        return false;
    }
    // Find env's command, rather than mistaking a command argument for it.
    while let Some(word) = words.next() {
        if matches!(word, "-u" | "--unset" | "-C" | "--chdir" | "-a" | "--argv0") {
            words.next();
        } else if word == "--" {
            return words.next().is_some_and(python_interpreter);
        } else if !word.starts_with('-') && !word.contains('=') {
            // env -S can quote the command and its arguments as one string.
            return python_interpreter(word.trim_matches(['\'', '"']));
        }
    }
    false
}

fn python_problem(path: &Path, bytes: &[u8]) -> Option<&'static str> {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            ["py", "pyw", "pyi", "pyc", "pyo", "pyz", "pyzw"]
                .iter()
                .any(|banned| ext.eq_ignore_ascii_case(banned))
        })
    {
        Some("Python file extension")
    } else if python_shebang(bytes) {
        Some("Python interpreter shebang")
    } else {
        None
    }
}

#[test]
fn python_tracked_files_are_forbidden() {
    let root = gate_root();
    let paths = tracked_paths();
    assert!(!paths.is_empty(), "the Git index must contain files");
    let mut findings = Vec::new();
    for path in paths {
        if let Some(reason) = python_problem(&path, &read_tracked(&root, &path)) {
            findings.push(format!("{}: {reason}", path.display()));
        }
    }
    assert!(
        findings.is_empty(),
        "Python is forbidden in this repository; remove these files from the commit and implement tooling in Rust:\n{}",
        findings.join("\n")
    );
}

#[test]
fn python_extensions_are_refused_even_for_empty_or_binary_files() {
    for extension in ["py", "pyw", "pyi", "pyc", "pyo", "pyz", "pyzw", "PY", "PyI"] {
        let name = format!("tools/a script.{extension}");
        for bytes in [b"".as_slice(), b"\0binary".as_slice()] {
            assert_eq!(
                python_problem(Path::new(&name), bytes),
                Some("Python file extension"),
                "{name}"
            );
            assert_eq!(python_problem(Path::new("tools/a script.rs"), bytes), None);
        }
    }
}

#[test]
fn python_shebangs_are_refused_without_a_python_extension() {
    for line in [
        "#!/usr/bin/python",
        "#!/usr/local/bin/python3.14 -u",
        "#! /usr/bin/env python3\r",
        "#!/usr/bin/env -S python3 -u",
        "#!/usr/bin/env -S 'python3 -u'",
        "#!/usr/bin/env NAME=value python3",
        "#!/usr/bin/env -u python3 python3",
        "#!/usr/bin/pypy3",
    ] {
        let bytes = format!("{line}\nprint('hello')\n");
        assert_eq!(
            python_problem(Path::new("tools/tool"), bytes.as_bytes()),
            Some("Python interpreter shebang"),
            "{line}"
        );
        let valid = bytes.replace("python", "ruby").replace("pypy", "ruby");
        assert_eq!(
            python_problem(Path::new("tools/tool"), valid.as_bytes()),
            None
        );
    }
}

#[test]
fn python_mentions_and_similar_names_are_allowed() {
    for (name, text) in [
        ("docs/python.md", "Use Rust instead of Python.\n"),
        ("tools/python-helper.rs", "fn main() {}\n"),
        ("tools/check", "#!/bin/sh\necho python3\n"),
        ("tools/check", "#!/usr/bin/env python-helper\n"),
        ("tools/check", "#!/usr/bin/env echo python3\n"),
        ("tools/check", "#!/usr/bin/env -u python3 ruby\n"),
        ("docs/example.md", "```\n#!/usr/bin/python3\n```\n"),
    ] {
        assert_eq!(python_problem(Path::new(name), text.as_bytes()), None);
    }
}
