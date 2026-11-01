// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Rustdoc-specific admission of one HTML input operand, separate from option values.

use super::{DiagramError, refused};

const SWITCHES: &[&str] = &[
    "--verbose",
    "--document-private-items",
    "--document-hidden-items",
    "--extern-html-root-takes-precedence",
    "--sort-modules-by-appearance",
    "--display-doctest-warnings",
    "--markdown-no-toc",
    "--enable-index-page",
    "--show-type-layout",
    "--html-no-source",
    "--generate-link-to-definition",
    "--generate-macro-expansion",
    "--disable-minification",
    "--generate-redirect-map",
];

const VALUES: &[&str] = &[
    "--output-format",
    "--output",
    "--out-dir",
    "--crate-name",
    "--crate-type",
    "--library-path",
    "--cfg",
    "--check-cfg",
    "--extern",
    "--extern-html-root-url",
    "--codegen",
    "--target",
    "--markdown-css",
    "--html-in-header",
    "--html-before-content",
    "--html-after-content",
    "--markdown-before-content",
    "--markdown-after-content",
    "--markdown-playground-url",
    "--extend-css",
    "--sysroot",
    "--playground-url",
    "--crate-version",
    "--default-theme",
    "--default-setting",
    "--theme",
    "--resource-suffix",
    "--edition",
    "--color",
    "--error-format",
    "--diagnostic-width",
    "--json",
    "--allow",
    "--warn",
    "--force-warn",
    "--deny",
    "--forbid",
    "--cap-lints",
    "--remap-path-prefix",
    "--index-page",
    "--static-root-path",
    "--emit",
    "--remap-path-scope",
    "--with-examples",
];

pub(in crate::diagram::cargo) struct Input<'a> {
    pub(in crate::diagram::cargo) source: &'a str,
    pub(in crate::diagram::cargo) crate_name: &'a str,
    pub(in crate::diagram::cargo) terminator: Option<usize>,
    pub(in crate::diagram::cargo) target: Option<&'a str>,
    pub(in crate::diagram::cargo) cap_lints: bool,
    pub(in crate::diagram::cargo) force_warn: bool,
    pub(in crate::diagram::cargo) include_namespace_override: bool,
}

pub(in crate::diagram::cargo) fn parse(arguments: &[String]) -> Result<Input<'_>, DiagramError> {
    let mut source = None;
    let mut crate_name = None;
    let mut target = None;
    let mut cap_lints = false;
    let mut force_warn = false;
    let mut include_namespace_override = false;
    let mut words = arguments
        .iter()
        .enumerate()
        .map(|(index, word)| (index, word.as_str()));
    let mut operands = false;
    let mut terminator = None;
    while let Some((index, word)) = words.next() {
        if word.starts_with('@') {
            return Err(refused(
                "Rustdoc response files require separately observed input evidence",
            ));
        }
        if !operands && word == "--" {
            operands = true;
            terminator = Some(index);
            continue;
        }
        if !operands && word.starts_with('-') && word != "-" {
            let Some((key, inline)) = option(word)? else {
                continue;
            };
            let value = inline
                .or_else(|| words.next().map(|(_, value)| value))
                .ok_or_else(|| refused("Rustdoc option has no required value"))?;
            if value.starts_with('@') {
                return Err(refused("Rustdoc response-file expansion is unsupported"));
            }
            admit_value(key, value)?;
            if key == "--target" && target.replace(value).is_some() {
                return Err(refused("Rustdoc target is repeated"));
            }
            cap_lints |= key == "--cap-lints";
            force_warn |= key == "--force-warn";
            if key == "--extern" {
                let head = value.split('=').next().unwrap_or(value);
                include_namespace_override |= head
                    .split(':')
                    .any(|name| matches!(name.trim_start_matches("r#"), "std" | "core"));
            }
            if key == "--crate-name" && crate_name.replace(value).is_some() {
                return Err(refused("Rustdoc crate-name is repeated"));
            }
        } else if word == "-" || source.replace(word).is_some() {
            return Err(refused(
                "Rustdoc observations require exactly one file input, without standard input",
            ));
        }
    }
    Ok(Input {
        target,
        cap_lints,
        force_warn,
        include_namespace_override,
        terminator,
        source: source.ok_or_else(|| refused("Rustdoc invocation has no input operand"))?,
        crate_name: crate_name
            .filter(|name| !name.is_empty())
            .ok_or_else(|| refused("Rustdoc invocation has no crate-name"))?,
    })
}

fn option(word: &str) -> Result<Option<(&str, Option<&str>)>, DiagramError> {
    if word == "-v" {
        return Ok(None);
    }
    if word.starts_with("--") {
        let (key, inline) = word
            .split_once('=')
            .map_or((word, None), |(key, value)| (key, Some(value)));
        if SWITCHES.contains(&key) && inline.is_none() {
            return Ok(None);
        }
        if VALUES.contains(&key) {
            return Ok(Some((key, inline)));
        }
        return Err(refused(&format!(
            "unsupported Rustdoc HTML observation option {key}"
        )));
    }
    let key = match word.as_bytes().get(1) {
        Some(b'w') => "--output-format",
        Some(b'o') => "--out-dir",
        Some(b'L') => "--library-path",
        Some(b'C') => "--codegen",
        Some(b'e') => "--extend-css",
        Some(b'Z') => "--unstable-option",
        Some(b'A') => "--allow",
        Some(b'W') => "--warn",
        Some(b'D') => "--deny",
        Some(b'F') => "--forbid",
        _ => {
            return Err(refused(&format!(
                "unsupported Rustdoc HTML observation option {word}"
            )));
        }
    };
    Ok(Some((key, (!word[2..].is_empty()).then_some(&word[2..]))))
}

fn admit_value(key: &str, value: &str) -> Result<(), DiagramError> {
    if key == "--output-format" && value != "html" {
        return Err(refused(
            "Rustdoc invocation observations require HTML output",
        ));
    }
    if key == "--emit" && !value.split(',').any(|part| part == "html-non-static-files") {
        return Err(refused(
            "Rustdoc selected-library observations require non-static HTML emission",
        ));
    }
    if key == "--unstable-option" && value != "unstable-options" && !value.starts_with("threads=") {
        return Err(refused(
            "Rustdoc unstable mode requires its own supported HTML observation contract",
        ));
    }
    Ok(())
}
