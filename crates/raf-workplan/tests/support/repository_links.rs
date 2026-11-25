// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Repository navigation through the public immutable publisher.

use super::{
    component_model,
    support::disk::{Scratch, put as write, snapshot},
};
use purrdf_core::BaseIri;
use purrdf_lex::json::{self, record::FromJson};
use purrdf_rdf::viz::VizPresentedExport;
use raf_workplan::{Gate, diagram::publication};
use std::{fs, path::Path};

fn fixture(root: &Path) {
    write(
        root,
        "Cargo.toml",
        "[package]\nname = 'parcel'\nversion = '0.1.0'\nedition = '2024'\n",
    );
    write(
        root,
        "src/lib.rs",
        "#[path = \"escaped # café.rs\"] mod implementation;\npub use implementation::*;\n",
    );
    write(root, "src/escaped # café.rs", component_model::SOURCE);
    write(root, "tests/flow.rs", "#[test]\nfn case() {}\n");
    let model = component_model::MODEL.replace(
        "responsibility: Own parcel delivery",
        "responsibility: Own parcel delivery\n      bindings:\n        capabilities: [cap:empty]",
    );
    let declarations = "diagrams:\n  - name: components\n    kind: components\n    scope: cap:routing\n  - name: artifacts\n    kind: artifacts\n    scope: cap:routing\n  - name: scenario\n    kind: scenario\n    scope: cap:routing#deliver\n  - name: provenance\n    kind: model-provenance\n    scope: cap:routing\n";
    write(
        root,
        "docs/capabilities/routing.md",
        &component_model::document(&model).replacen("model:", &format!("{declarations}model:"), 1),
    );
    let empty = component_model::document("")
        .replace("cap:routing", "cap:empty")
        .replace(
            "code: [parcel::Parcel, parcel::route, parcel::store]",
            "code: []",
        )
        .replace("tests: [parcel::test_crate::flow::case]", "tests: []");
    write(root, "docs/capabilities/empty.md", &empty);
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn repository_links_resolve_from_two_depths_without_changing_cached_views() {
    // The remote-resource check sees a resource whatever the attribute order,
    // quote style or scheme form, and accepts local references.
    for remote in [
        "<script></script><script defer src='https://example.org/app.js'></script>",
        "<link rel=stylesheet href=//example.org/a.css>",
        "<image width=\"1\" href=\"HTTP://example.org/image.png\"/>",
        "<image width=\"1\" xlink:href='https://example.org/image.png'/>",
        "<script>x</script><image href=\"https://example.org/a.png\"/>",
        "<image href=\"&#x68;ttps://example.org/x.png\"/>",
        "<image href='&#104;ttp&colon;//example.org/x.png'/>",
        "<image href=\"ht&Tab;tps&#58;&sol;&sol;example.org/x.png\"/>",
        "<image href=\"&bsol;&bsol;example.org/x.png\"/>",
    ] {
        assert_eq!(remote_resource_urls(remote).len(), 1, "{remote}");
    }
    // Tag-like text inside a script or style body is not markup, and a local
    // or merely ampersand-bearing reference is not remote.
    for local in [
        "<script>a<b</script><a href='docs/x.md'><image width=\"1\" href=\"x.png\"/>",
        "<script>var s = '<image width=\"1\" href=\"https://example.org/x.png\"/>';</script>",
        "<style>a::after { content: '<link href=\"https://example.org/a.css\">' }</style>",
        "<a href=\"docs/x.md?a=1&amp;b=2&unknown;\">",
    ] {
        assert_eq!(remote_resource_urls(local), Vec::<String>::new(), "{local}");
    }
    let scratch = Scratch::new("repository-navigation");
    let root = scratch.path();
    fixture(root);
    let opened = Gate::workspace(root).open().unwrap();
    let plan = opened.workplan();
    for output in ["site # café", "published/nested output"] {
        fs::create_dir_all(root.join(output)).unwrap();
        write(root, &format!("{output}/host.html"), "unrelated host bytes");
        for name in ["components", "artifacts", "scenario", "provenance"] {
            let reference = format!("cap:routing#{name}");
            let cached = plan.named_diagram(&reference).unwrap();
            let before = cached.json().to_owned();
            let directory =
                publication::publish_repository(&opened, &reference, &root.join(output)).unwrap();
            let files = snapshot(root);
            assert_eq!(
                publication::publish_repository(&opened, &reference, &root.join(output)).unwrap(),
                directory
            );
            if output == "site # café" && name == "components" {
                let alias = root.join(output).join("..").join(output);
                assert_eq!(
                    publication::publish_repository(&opened, &reference, &alias).unwrap(),
                    directory
                );
            }
            assert_eq!(snapshot(root), files);
            assert_eq!(plan.named_diagram(&reference).unwrap().json(), before);
            let baseline = publication::publish(&cached, &root.join(output)).unwrap();
            assert_ne!(directory, baseline);
            for current in std::iter::once(cached.as_ref()).chain(cached.details()) {
                check_links(root, &directory, current);
            }
        }
        assert_eq!(
            fs::read_to_string(root.join(output).join("host.html")).unwrap(),
            "unrelated host bytes"
        );
    }
}

/// Returns every `src`, `href` and `xlink:href` value in `markup` that names a
/// remote resource (an absolute `http` or `https` URL, or a protocol-relative
/// one), whatever the attribute order, quoting or element.
fn remote_resource_urls(markup: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut cursor = markup;
    while let Some((_, tag)) = cursor.split_once('<') {
        cursor = tag;
        if !tag.starts_with(|c: char| c.is_ascii_alphabetic()) {
            continue;
        }
        let name_end = tag
            .find(|c: char| c.is_whitespace() || matches!(c, '>' | '/'))
            .unwrap_or(tag.len());
        let (element, mut rest) = tag.split_at(name_end);
        let element = element.to_ascii_lowercase();
        loop {
            rest = rest.trim_start_matches(|c: char| c.is_whitespace() || c == '/');
            if rest.is_empty() || rest.starts_with('>') {
                break;
            }
            let (name, after) = rest.split_at(
                rest.find(|c: char| c.is_whitespace() || matches!(c, '=' | '>' | '/'))
                    .unwrap_or(rest.len()),
            );
            let Some(after) = after.trim_start().strip_prefix('=') else {
                rest = after;
                continue;
            };
            let after = after.trim_start();
            let (value, remainder) = match after.chars().next().filter(|c| matches!(c, '"' | '\''))
            {
                Some(quote) => {
                    let body = after.strip_prefix(quote).unwrap_or(after);
                    body.split_once(quote).unwrap_or((body, ""))
                }
                None => after.split_at(
                    after
                        .find(|c: char| c.is_whitespace() || c == '>')
                        .unwrap_or(after.len()),
                ),
            };
            rest = remainder;
            let lowered = browser_url(value);
            if matches!(
                name.to_ascii_lowercase().as_str(),
                "src" | "href" | "xlink:href"
            ) && ["http:", "https:", "//"]
                .iter()
                .any(|scheme| lowered.starts_with(scheme))
            {
                found.push(value.to_owned());
            }
        }
        cursor = rest;
        // Script and style bodies are text, not markup: skip to the close tag.
        if matches!(element.as_str(), "script" | "style") {
            let close = format!("</{element}");
            let lowered = cursor.to_ascii_lowercase();
            cursor = lowered
                .find(&close)
                .map_or("", |end| cursor.split_at(end).1);
        }
    }
    found
}

/// Returns an attribute value as a browser reads it as a URL: character
/// references decoded, ASCII tab and newlines removed, surrounding control
/// characters and spaces trimmed, `\` read as `/`, and the case folded.
///
/// Numeric references (decimal and hexadecimal, with or without the closing
/// semicolon) are decoded. Named references are decoded for `colon`, `Tab`,
/// `NewLine`, `sol`, `bsol`, `period`, `amp`, `lt`, `gt`, `quot` and `apos`,
/// the ones that can spell a scheme or delimiter.
fn browser_url(value: &str) -> String {
    let mut decoded = String::new();
    let mut rest = value;
    while let Some((head, tail)) = rest.split_once('&') {
        decoded.push_str(head);
        rest = tail;
        if let Some((character, remainder)) = reference(rest) {
            decoded.push(character);
            rest = remainder;
        } else {
            decoded.push('&');
        }
    }
    decoded.push_str(rest);
    decoded
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .map(|c| if c == '\\' { '/' } else { c })
        .collect::<String>()
        .trim_matches(|c: char| c.is_ascii_control() || c == ' ')
        .to_ascii_lowercase()
}

/// Decodes the character reference that follows an ampersand, returning the
/// character and the text after the reference.
fn reference(text: &str) -> Option<(char, &str)> {
    if let Some(numeric) = text.strip_prefix('#') {
        let (radix, digits) = match numeric.strip_prefix(['x', 'X']) {
            Some(hex) => (16, hex),
            None => (10, numeric),
        };
        let count = digits
            .find(|c: char| !c.is_digit(radix))
            .unwrap_or(digits.len());
        let (number, after) = digits.split_at(count);
        let character = u32::from_str_radix(number, radix)
            .ok()
            .and_then(char::from_u32)?;
        return Some((character, after.strip_prefix(';').unwrap_or(after)));
    }
    [
        ("colon;", ':'),
        ("Tab;", '\t'),
        ("NewLine;", '\n'),
        ("sol;", '/'),
        ("bsol;", '\\'),
        ("period;", '.'),
        ("amp;", '&'),
        ("lt;", '<'),
        ("gt;", '>'),
        ("quot;", '"'),
        ("apos;", '\''),
    ]
    .into_iter()
    .find_map(|(name, character)| Some((character, text.strip_prefix(name)?)))
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn check_links(root: &Path, directory: &Path, cached: &raf_workplan::diagram::DiagramBundle) {
    let name = cached.view().name();
    let text = fs::read_to_string(directory.join(format!("{name}.json"))).unwrap();
    let value = json::read(&text).unwrap();
    let contextual = VizPresentedExport::from_json(value.get("export").unwrap()).unwrap();
    let original = cached.presented_export().unwrap();
    assert_eq!(contextual.export.model, original.export.model);
    assert_eq!(contextual.export.layout, original.export.layout);
    assert_eq!(contextual.geometry, original.geometry);
    let page = directory.join(format!("{name}.html"));
    let location = format!(
        "https://example.org/non-root/project/{}",
        purrdf_lex::percent::encode(
            page.strip_prefix(root).unwrap().to_str().unwrap(),
            purrdf_lex::percent::UNRESERVED_SLASH
        )
    );
    let base = BaseIri::parse(&location).unwrap();
    let svg = fs::read_to_string(directory.join(format!("{name}.svg"))).unwrap();
    let mut links = 0;
    for display in &contextual.presentation.displays {
        let Some(navigation) = &display.navigation else {
            continue;
        };
        let href = navigation.as_str();
        assert!(!href.contains("#L"));
        assert!(svg.contains(href));
        let resolved = base.resolve(href).unwrap();
        let expected = if href.contains("escaped") {
            "src/escaped # café.rs"
        } else if href.contains("tests") {
            "tests/flow.rs"
        } else {
            "docs/capabilities/empty.md"
        };
        assert!(root.join(expected).is_file());
        assert_eq!(
            resolved.as_str(),
            format!(
                "https://example.org/non-root/project/{}",
                purrdf_lex::percent::encode(expected, purrdf_lex::percent::UNRESERVED_SLASH)
            )
        );
        links += 1;
    }
    assert!(links > 0);
    let html = fs::read_to_string(page).unwrap();
    // The offline explorer loads nothing: its script is inline and no page or
    // SVG asset names a remote resource, so it works from any directory.
    assert!(html.contains("<script>"));
    assert!(!html.contains("<script src"));
    assert!(!html.contains("<link "));
    assert_eq!(remote_resource_urls(&html), Vec::<String>::new());
    assert_eq!(remote_resource_urls(&svg), Vec::<String>::new());
    for detail in cached.details() {
        assert!(html.contains(&format!("{}.html", detail.view().name())));
        assert!(
            directory
                .join(format!("{}.html", detail.view().name()))
                .is_file()
        );
        assert!(
            fs::read_to_string(directory.join(format!("{}.html", detail.view().name())))
                .unwrap()
                .contains(&format!("{name}.html"))
        );
    }
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn repository_publication_refuses_missing_changed_and_outside_bindings() {
    let scratch = Scratch::new("repository-navigation-refusals");
    let root = scratch.path();
    fixture(root);
    let output = root.join("site");
    fs::create_dir(&output).unwrap();
    let opened = Gate::workspace(root).open().unwrap();
    let reference = "cap:routing#components";
    for rejected in [root.join("missing"), scratch.target()] {
        let before = snapshot(root);
        assert!(publication::publish_repository(&opened, reference, &rejected).is_err());
        assert_eq!(snapshot(root), before);
    }
    let source = root.join("src/escaped # café.rs");
    for content in [
        None,
        Some("pub struct Parcel;\npub fn route() { let changed = 1; }\npub fn store() {}\n"),
    ] {
        if let Some(content) = content {
            fs::write(&source, content).unwrap();
        } else {
            fs::remove_file(&source).unwrap();
        }
        let before = snapshot(root);
        assert!(publication::publish_repository(&opened, reference, &output).is_err());
        assert_eq!(snapshot(root), before);
        fs::write(&source, component_model::SOURCE).unwrap();
    }
    let capability = root.join("docs/capabilities/empty.md");
    let original = fs::read(&capability).unwrap();
    fs::remove_file(&capability).unwrap();
    let before = snapshot(root);
    assert!(publication::publish_repository(&opened, reference, &output).is_err());
    assert_eq!(snapshot(root), before);
    fs::write(capability, original).unwrap();
    #[cfg(target_os = "linux")]
    {
        let outside = scratch.target().join("outside.rs");
        fs::write(&outside, component_model::SOURCE).unwrap();
        fs::remove_file(&source).unwrap();
        std::os::unix::fs::symlink(&outside, &source).unwrap();
        let before = snapshot(root);
        assert!(publication::publish_repository(&opened, reference, &output).is_err());
        assert_eq!(snapshot(root), before);
        fs::remove_file(&source).unwrap();
        fs::write(&source, component_model::SOURCE).unwrap();
    }
    publication::publish_repository(&opened, reference, &output).unwrap();
    let moved = scratch.target().join("moved-workspace");
    fs::rename(root, &moved).unwrap();
    fs::create_dir(root).unwrap();
    let before = snapshot(&moved);
    assert!(publication::publish_repository(&opened, reference, &output).is_err());
    assert_eq!(snapshot(&moved), before);
    assert!(snapshot(root).is_empty());
}
