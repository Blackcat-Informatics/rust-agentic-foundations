// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

use super::*;
use raf_workplan::exec::{FakeRunner, Output, Program};

fn invoke(output: Output) -> (Result<u8, Error>, FakeRunner) {
    let runner = FakeRunner::new().on(
        Program::Cargo,
        &["run", "--frozen", "--package=demo", "--example=census"],
        Ok(output),
    );
    let result = run(
        &runner,
        &Package::new("demo").unwrap(),
        &ExampleName::new("census").unwrap(),
        &mut Vec::new(),
    );
    (result, runner)
}

#[test]
fn malformed_target_responses_preserve_the_actual_stderr_cause() {
    for stdout in [b"not an RDF response".to_vec(), vec![255]] {
        let (result, runner) = invoke(Output::new(
            Some(1),
            stdout,
            b"original target failure\n".to_vec(),
        ));
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("original target failure"),
            "{error}"
        );
        assert!(error.source().is_some());
        assert_eq!(runner.calls().len(), 1);
    }
}

#[test]
fn process_failure_protocol_disagreement_and_gate_error_remain_distinct() {
    for code in [None, Some(101), Some(2)] {
        let (result, _) = invoke(Output::new(
            code,
            Vec::new(),
            b"no example target named census".to_vec(),
        ));
        let error = result.unwrap_err().to_string();
        assert!(error.contains("target failed with exit"), "{error}");
        assert!(error.contains("no example target"), "{error}");
    }
    let text = RegisteredCheck::from_check(&Err(raf_census::Error::Options("gate cause".into())))
        .unwrap()
        .to_ntriples()
        .unwrap();
    let (result, _) = invoke(Output::new(
        Some(0),
        text.as_bytes().to_vec(),
        b"mismatch diagnostic".to_vec(),
    ));
    let error = result.unwrap_err().to_string();
    assert!(error.contains("disagrees with protocol"), "{error}");
    assert!(error.contains("mismatch diagnostic"), "{error}");
    let (result, _) = invoke(Output::new(Some(1), text.into_bytes(), Vec::new()));
    let error = result.unwrap_err().to_string();
    assert!(error.contains("repository evaluation failed"), "{error}");
    assert!(error.contains("gate cause"), "{error}");
}
