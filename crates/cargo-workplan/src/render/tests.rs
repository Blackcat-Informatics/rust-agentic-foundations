// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Sink failures, quiet filtering, typed output and exit policy.

use super::*;

fn printed(report: &Report, format: Format, quiet: bool) -> (String, String) {
    let (mut out, mut diagnostics) = (Vec::new(), Vec::new());
    report
        .print(format, quiet, &mut out, &mut diagnostics)
        .unwrap();
    (
        String::from_utf8(out).unwrap(),
        String::from_utf8(diagnostics).unwrap(),
    )
}
fn rewritten() -> Record {
    Record::outcome(
        CommandResult::Rewritten {
            path: "a.md".into(),
        },
        "rewrote a.md",
    )
}

#[test]
fn diagnostic_write_failure_is_propagated_and_quiet_does_not_write_notes() {
    struct Refuse;
    impl Write for Refuse {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "diagnostic sink closed",
            ))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut report = Report::default();
    report.note("a note");
    let error = report
        .print(Format::Text, false, &mut Vec::new(), &mut Refuse)
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(
        printed(&report, Format::Text, false).1,
        "cargo-workplan: a note\n"
    );
    report
        .print(Format::Text, true, &mut Vec::new(), &mut Refuse)
        .unwrap();
    assert_eq!(report.exit(), Exit::Clean);
}

#[test]
fn diagnostic_flush_failure_propagates_and_quiet_never_touches_that_sink() {
    #[derive(Default)]
    struct RefuseFlush {
        bytes: Vec<u8>,
        flushes: usize,
    }
    impl Write for RefuseFlush {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "diagnostic flush failed",
            ))
        }
    }
    let mut report = Report::of([rewritten()]).command("docs sync");
    report.note("a buffered note");
    for format in [Format::Text, Format::Turtle, Format::Ntriples] {
        let mut diagnostics = RefuseFlush::default();
        let error = report
            .print(format, false, &mut Vec::new(), &mut diagnostics)
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(diagnostics.bytes, b"cargo-workplan: a buffered note\n");
        assert_eq!(diagnostics.flushes, 1);
        assert_eq!(
            printed(&report, format, false).1,
            "cargo-workplan: a buffered note\n"
        );
        let mut quiet = RefuseFlush::default();
        report
            .print(format, true, &mut Vec::new(), &mut quiet)
            .unwrap();
        assert_eq!(quiet.bytes, [] as [u8; 0]);
        assert_eq!(quiet.flushes, 0);
    }
}

#[test]
fn quiet_filters_results_without_changing_exit_and_machine_output_stays_a_graph() {
    let refusal = Record::finding(
        CommandResult::Refused {
            failures: 1,
            message: "refused".into(),
            evidence: Vec::new(),
        },
        "refused",
    );
    let mut report = Report::of([rewritten(), refusal]).command("change complete");
    report.note("a note");
    assert_eq!(
        printed(&report, Format::Text, false).0,
        "rewrote a.md\nrefused\n"
    );
    assert_eq!(
        printed(&report, Format::Text, true),
        ("refused\n".into(), String::new())
    );
    assert_eq!(report.exit(), Exit::Findings);
    let turtle = printed(&report, Format::Turtle, true).0;
    let nt = printed(&report, Format::Ntriples, true).0;
    let graph = raf_rdf::Graph::from_text(&turtle, raf_rdf::Format::Turtle).unwrap();
    assert_eq!(
        graph.snapshot().unwrap(),
        raf_rdf::Graph::from_text(&nt, raf_rdf::Format::NTriples)
            .unwrap()
            .snapshot()
            .unwrap()
    );
    assert!(!turtle.contains("Rewritten"));
    let clean = Report::of([rewritten()]);
    assert_eq!(printed(&clean, Format::Text, true).0, "");
    assert!(
        raf_rdf::Graph::from_text(
            &printed(&clean, Format::Turtle, true).0,
            raf_rdf::Format::Turtle
        )
        .is_ok()
    );
    assert_eq!(clean.exit(), Exit::Clean);
}
