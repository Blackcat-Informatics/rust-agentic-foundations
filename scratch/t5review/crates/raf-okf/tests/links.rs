// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Link forms, split links, code spans, HTML attributes and target
//! resolution.

use raf_okf::{Body, ErrorKind, Link, LinkError, LinkForm, LinkTarget};

fn body(text: &str) -> Body {
    Body::parse(text).unwrap_or_else(|error| panic!("{error} in {text:?}"))
}

fn targets(text: &str) -> Vec<String> {
    body(text)
        .links()
        .iter()
        .map(|l| l.target().to_string())
        .collect()
}

fn split(text: &str) -> Vec<(LinkForm, usize)> {
    body(text)
        .split_links()
        .iter()
        .map(|l| (l.form(), l.line()))
        .collect()
}

fn forms(text: &str) -> Vec<(LinkForm, String, usize)> {
    body(text)
        .links()
        .iter()
        .map(|l| (l.form(), l.target().to_string(), l.line()))
        .collect()
}

#[test]
fn every_link_form_is_read_with_its_target_anchor_and_line() {
    let text = "[inline](a.md#top)\n![image](pic.png \"title\")\n[ref]: ./c.md#sec\n<a href=\"d.md\">x</a> <img src='e.svg'>\n";
    assert_eq!(
        forms(text),
        [
            (LinkForm::Inline, "a.md#top".to_string(), 1),
            (LinkForm::Image, "pic.png".to_string(), 2),
            (LinkForm::ReferenceDefinition, "./c.md#sec".to_string(), 3),
            (LinkForm::Href, "d.md".to_string(), 4),
            (LinkForm::Src, "e.svg".to_string(), 4),
        ]
    );
    let parsed = body(text);
    assert_eq!(parsed.links()[0].anchor(), Some("top"));
    assert_eq!(parsed.links()[1].anchor(), None);
    assert_eq!(parsed.links()[2].anchor(), Some("sec"));
}

#[test]
fn reference_definitions_take_angle_targets_and_skip_footnotes() {
    assert_eq!(
        targets("[a]: <dir/with space.md> \"t\"\n"),
        ["dir/with space.md"]
    );
    assert_eq!(targets("[^1]: A note.\n"), Vec::<String>::new());
    assert_eq!(targets("    [a]: indented.md\n"), Vec::<String>::new());
    assert_eq!(targets("   [a]: ok.md\n"), ["ok.md"]);
    assert_eq!(targets("text [a]: not-at-start.md\n"), Vec::<String>::new());
}

#[test]
fn links_inside_code_spans_of_any_length_and_fences_are_not_links() {
    assert_eq!(targets("`[a](b.md)`\n"), Vec::<String>::new());
    assert_eq!(targets("``[a](b.md)``\n"), Vec::<String>::new());
    assert_eq!(targets("```[a](b.md)```\n"), Vec::<String>::new());
    assert_eq!(targets("`[a](b.md)` and [c](d.md)\n"), ["d.md"]);
    assert_eq!(targets("`unclosed [a](b.md)\n"), ["b.md"]);
    assert_eq!(targets("``a ` b`` [c](d.md)\n"), ["d.md"]);
    assert_eq!(
        targets("```\nv[0](x)\n[a](b.md)\n```\n"),
        Vec::<String>::new()
    );
    assert_eq!(
        targets("[x](y.md)\n```\nv[0](x)\n```\n[z](w.md)\n"),
        ["y.md", "w.md"]
    );
}

#[test]
fn links_inside_html_comments_are_not_links() {
    assert_eq!(targets("<!-- [a](b.md) -->\n[c](d.md)\n"), ["d.md"]);
    assert_eq!(targets("<!--\n[a](b.md)\n-->\n[c](d.md)\n"), ["d.md"]);
}

#[test]
fn an_escaped_closing_parenthesis_belongs_to_the_destination() {
    assert_eq!(targets("[a](b\\).md)\n"), ["b).md"]);
    assert_eq!(targets("[a](b(c).md)\n"), ["b(c).md"]);
    assert_eq!(targets("\\[a](b.md)\n"), Vec::<String>::new());
    assert_eq!(targets("[a\\](b.md)\n"), Vec::<String>::new());
}

#[test]
fn a_space_between_the_text_and_the_parenthesis_is_not_a_link() {
    assert_eq!(targets("[a] (b.md)\n"), Vec::<String>::new());
    assert_eq!(targets("[a](b.md)\n"), ["b.md"]);
    assert_eq!(targets("[a]\t(b.md)\n"), Vec::<String>::new());
    assert!(matches!(split("[a]\n(b.md)\n").as_slice(), []));
}

#[test]
fn only_ascii_space_and_controls_end_a_destination_not_unicode_space() {
    assert_eq!(targets("[a](b\u{a0}c.md)\n"), ["b\u{a0}c.md"]);
    assert_eq!(targets("[a](b\u{3000}c.md)\n"), ["b\u{3000}c.md"]);
    assert_eq!(targets("[a](b.md \"title\")\n"), ["b.md"]);
    assert_eq!(targets("[a](b.md 'title')\n"), ["b.md"]);
    assert_eq!(targets("[a](b.md (title))\n"), ["b.md"]);
    assert_eq!(targets("[a](  b.md\t)\n"), ["b.md"]);
    assert_eq!(targets("[a](<b c.md>)\n"), ["b c.md"]);
    assert_eq!(targets("[a]()\n"), [""]);
    // Two bare words are not a destination and a title.
    assert_eq!(targets("[a](b.md c)\n"), Vec::<String>::new());
    // An unclosed parenthesis is no link.
    assert_eq!(targets("[a](b.md\n"), Vec::<String>::new());
}

#[test]
fn nested_brackets_images_in_links_and_stray_brackets_are_read() {
    assert_eq!(
        targets("[![alt](pic.png)](page.md)\n"),
        ["pic.png", "page.md"]
    );
    assert_eq!(targets("array[0] then [a](b.md)\n"), ["b.md"]);
    assert_eq!(targets("[a [b](c.md) d](e.md)\n"), ["c.md"]);
    assert_eq!(targets("[a][ref] and [b](c.md)\n"), ["c.md"]);
    assert_eq!(targets("[a](b.md)[c](d.md)\n"), ["b.md", "d.md"]);
}

#[test]
fn a_link_split_across_lines_is_recorded_and_not_read() {
    let text_split = "intro\n[text\nmore](a.md)\n";
    assert_eq!(split(text_split), [(LinkForm::Inline, 2)]);
    assert!(matches!(targets(text_split).as_slice(), []));

    let destination_split = "[a](\nb.md)\n";
    assert_eq!(split(destination_split), [(LinkForm::Inline, 1)]);
    assert!(matches!(targets(destination_split).as_slice(), []));

    assert_eq!(split("![alt\ntext](p.png)\n"), [(LinkForm::Image, 1)]);
    assert_eq!(split("[a](b.md\n\"title\")\n"), [(LinkForm::Inline, 1)]);
    assert_eq!(split("> [quoted\n> more](a.md)\n"), [(LinkForm::Inline, 1)]);
}

#[test]
fn split_links_are_not_recorded_in_fences_code_spans_or_for_one_line_links() {
    assert!(matches!(
        split("```\n[text\nmore](a.md)\n```\n").as_slice(),
        []
    ));
    assert!(matches!(split("`[text`\n`more](a.md)`\n").as_slice(), []));
    assert!(matches!(split("[text](a.md)\n").as_slice(), []));
    assert!(matches!(split("[one](a.md)\n[two](b.md)\n").as_slice(), []));
    assert_eq!(targets("[one](a.md)\n[two](b.md)\n"), ["a.md", "b.md"]);
    // A blank line ends the paragraph, so the pieces never join.
    assert!(matches!(split("[text\n\nmore](a.md)\n").as_slice(), []));
    // A heading is a block of its own.
    assert!(matches!(split("## [head\n[x](a.md)\n").as_slice(), []));
}

#[test]
fn a_split_link_does_not_hide_a_link_after_it() {
    let text = "[text\nmore](a.md) and [ok](b.md)\n";
    assert_eq!(split(text), [(LinkForm::Inline, 1)]);
    assert_eq!(targets(text), ["b.md"]);
}

#[test]
fn a_destination_or_title_of_exactly_4096_bytes_is_read_and_one_more_is_refused() {
    let at = "x".repeat(4096);
    let over = "x".repeat(4097);
    let bound = ErrorKind::LinkPartTooLong { max: 4096 };
    // Bare destination.
    assert_eq!(targets(&format!("[a]({at})\n")), std::slice::from_ref(&at));
    let error = Body::parse(&format!("para\n\n[a]({over})\n")).expect_err("destination");
    assert_eq!(error.kind(), &bound);
    assert_eq!(error.line(), 3);
    // Angle-bracket destination, the delimiters not counted.
    assert_eq!(
        targets(&format!("[a](<{at}>)\n")),
        std::slice::from_ref(&at)
    );
    assert_eq!(
        Body::parse(&format!("[a](<{over}>)\n"))
            .expect_err("angle")
            .kind(),
        &bound
    );
    // Title, the delimiters not counted.
    assert_eq!(targets(&format!("[a](b.md \"{at}\")\n")), ["b.md"]);
    assert_eq!(targets(&format!("[a](b.md '{at}')\n")), ["b.md"]);
    assert_eq!(targets(&format!("[a](b.md ({at}))\n")), ["b.md"]);
    let title = Body::parse(&format!("x\n[a](b.md \"{over}\")\n")).expect_err("title");
    assert_eq!(title.kind(), &bound);
    assert_eq!(title.line(), 2);
}

#[test]
fn a_link_split_across_lines_inside_a_block_quote_is_recorded() {
    assert_eq!(split("> [a](\n> b.md)\n"), [(LinkForm::Inline, 1)]);
    assert_eq!(
        split("> > [text\n> > more](a.md)\n"),
        [(LinkForm::Inline, 1)]
    );
    assert_eq!(split("> ![alt\n> text](p.png)\n"), [(LinkForm::Image, 1)]);
    assert!(matches!(targets("> [a](\n> b.md)\n").as_slice(), []));
    // One line in a quote is a link, and two quoted one-line links are links.
    assert_eq!(targets("> [a](b.md)\n"), ["b.md"]);
    assert_eq!(targets("> [a](b.md)\n> [c](d.md)\n"), ["b.md", "d.md"]);
    assert!(matches!(split("> [a](b.md)\n> [c](d.md)\n").as_slice(), []));
}

#[test]
fn html_attributes_are_read_with_spacing_case_and_unquoted_values() {
    assert_eq!(
        forms(
            "<a href = \"docs/m.md\">x</a>\n<img src\t=  docs/s.svg>\n<a HREF=\"up.md\">y</a>\n<img src=docs/t.svg />\n"
        ),
        [
            (LinkForm::Href, "docs/m.md".to_string(), 1),
            (LinkForm::Src, "docs/s.svg".to_string(), 2),
            (LinkForm::Href, "up.md".to_string(), 3),
            (LinkForm::Src, "docs/t.svg".to_string(), 4),
        ]
    );
    let prose = targets(
        "<a data-href=\"a.md\" x-src='b.svg' data_src=c.svg>x</a>\nUse the href attribute and src = nothing here.\n",
    );
    assert!(matches!(prose.as_slice(), []));
}

#[test]
fn an_html_tag_split_across_lines_yields_its_attributes_with_their_lines() {
    for (text, expected) in [
        (
            "<img\n  src=\"docs/missing.svg\"\n  alt=\"x\">\n",
            ("docs/missing.svg", LinkForm::Src, 2),
        ),
        (
            "<img\n  alt=\"x\"\n  src=docs/missing.svg />\n",
            ("docs/missing.svg", LinkForm::Src, 3),
        ),
        (
            "<img\nalt=\"x\"\nsrc = 'docs/missing.svg'\nwidth=\"2\">\n",
            ("docs/missing.svg", LinkForm::Src, 3),
        ),
        (
            "<a\n  href=\"docs/missing.md\">gone</a>\n",
            ("docs/missing.md", LinkForm::Href, 2),
        ),
        (
            "<a title=\"x>y\" href=\"./nope.md\">gone</a>\n",
            ("./nope.md", LinkForm::Href, 1),
        ),
        (
            "<a title=\"two\nlines>\" href=\"./nope.md\">gone</a>\n",
            ("./nope.md", LinkForm::Href, 2),
        ),
    ] {
        assert_eq!(
            forms(text),
            [(expected.1, expected.0.to_string(), expected.2)],
            "{text:?}"
        );
    }
}

#[test]
fn an_attribute_value_that_starts_on_the_next_line_is_read() {
    assert_eq!(
        forms("<img src=\n  \"docs/missing.svg\" alt=\"x\">\n"),
        [(LinkForm::Src, "docs/missing.svg".to_string(), 2)]
    );
    assert_eq!(
        forms("<a href =\n  docs/missing.md>gone</a>\n"),
        [(LinkForm::Href, "docs/missing.md".to_string(), 2)]
    );
    // The same words outside a tag are prose.
    assert!(matches!(
        targets("src =\n  \"docs/missing.svg\"\n").as_slice(),
        []
    ));
}

#[test]
fn tag_state_does_not_survive_a_blank_line_or_a_fence() {
    let broken = "<img src=\"./nope.png\">\n";
    for prose in [
        "a<b it's fine\n\n",
        "a<b \"quoted\n\n",
        "a<b it's fine\n\n```text\ncode\n```\n",
    ] {
        assert_eq!(
            targets(&format!("{prose}{broken}")),
            ["./nope.png"],
            "{prose:?}"
        );
    }
}

#[test]
fn the_tag_scan_spares_prose_autolinks_and_closed_tags() {
    let text = "Write src = nothing or href = nothing; if a < b then x > y.\n<https://example.invalid/?src=a&href=b>\n<p align=\"center\">\n  <img src=\"docs/overview.md\" alt=\"ok\">\n</p>\nAfter the tag, src = nothing is prose again.\n";
    assert_eq!(
        forms(text),
        [(LinkForm::Src, "docs/overview.md".to_string(), 4)]
    );
}

fn link(text: &str) -> Link {
    body(text).links()[0].clone()
}

#[test]
fn targets_resolve_against_the_document_path() {
    let from = "docs/capabilities/a.md";
    let cases = [
        (
            "[l](b.md)",
            LinkTarget::Relative {
                path: "docs/capabilities/b.md".into(),
                anchor: None,
            },
        ),
        (
            "[l](./b.md#x)",
            LinkTarget::Relative {
                path: "docs/capabilities/b.md".into(),
                anchor: Some("x".into()),
            },
        ),
        (
            "[l](../architecture/c.md)",
            LinkTarget::Relative {
                path: "docs/architecture/c.md".into(),
                anchor: None,
            },
        ),
        (
            "[l](../../README.md)",
            LinkTarget::Relative {
                path: "README.md".into(),
                anchor: None,
            },
        ),
        (
            "[l](sub//d.md)",
            LinkTarget::Relative {
                path: "docs/capabilities/sub/d.md".into(),
                anchor: None,
            },
        ),
        ("[l](#top)", LinkTarget::Fragment("top".into())),
        ("[l](#)", LinkTarget::Fragment(String::new())),
        (
            "[l](https://example.invalid/x)",
            LinkTarget::External("https://example.invalid/x".into()),
        ),
        (
            "[l](mailto:a@b.c)",
            LinkTarget::External("mailto:a@b.c".into()),
        ),
        (
            "[l](//cdn.invalid/x)",
            LinkTarget::External("//cdn.invalid/x".into()),
        ),
    ];
    for (text, expected) in cases {
        assert_eq!(link(text).target_in(from), Ok(expected), "{text}");
    }
    assert_eq!(
        link("[l](b.md)").target_in("a.md"),
        Ok(LinkTarget::Relative {
            path: "b.md".into(),
            anchor: None
        })
    );
}

#[test]
fn targets_that_leave_the_root_or_are_empty_or_absolute_are_refused() {
    let from = "docs/capabilities/a.md";
    assert_eq!(
        link("[l](../../../outside.md)").target_in(from),
        Err(LinkError::EscapesRoot {
            target: "../../../outside.md".into()
        })
    );
    assert_eq!(
        link("[l](../../..)").target_in(from),
        Err(LinkError::EscapesRoot {
            target: "../../..".into()
        })
    );
    assert_eq!(
        link("[l](../x.md)").target_in("a.md"),
        Err(LinkError::EscapesRoot {
            target: "../x.md".into()
        })
    );
    assert_eq!(
        link("[l](/etc/passwd)").target_in(from),
        Err(LinkError::Absolute {
            target: "/etc/passwd".into()
        })
    );
    assert_eq!(link("[l]()").target_in(from), Err(LinkError::Empty));
    // The neighbour that stays inside the root resolves.
    assert!(link("[l](../../x.md)").target_in(from).is_ok());
}
