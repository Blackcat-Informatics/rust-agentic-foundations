// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

use super::*;
use raf_workplan::exec::{FakeRunner, Output, Program};

fn invoke(output: Output) -> (Result<u8, Error>, FakeRunner) {
    invoke_operation(RegisteredOperation::Check, output)
}

fn invoke_operation(
    operation: RegisteredOperation,
    output: Output,
) -> (Result<u8, Error>, FakeRunner) {
    let mut args = vec!["run", "--frozen", "--package=demo", "--example=census"];
    if operation == RegisteredOperation::WriteDebt {
        args.extend(["--", "--census-debt-write-v1"]);
    }
    let runner = FakeRunner::new().on(Program::Cargo, &args, Ok(output));
    let result = run(
        &runner,
        operation,
        &Package::new("demo").unwrap(),
        &ExampleName::new("census").unwrap(),
        &mut Vec::new(),
    );
    (result, runner)
}

#[test]
fn malformed_target_responses_preserve_the_actual_stderr_cause() {
    for stdout in [b"malformed receipt".to_vec(), vec![255]] {
        let (result, runner) = invoke_operation(
            RegisteredOperation::WriteDebt,
            Output::new(Some(0), stdout, b"writer diagnostic".to_vec()),
        );
        let error = result.unwrap_err();
        assert!(error.to_string().contains("writer diagnostic"), "{error}");
        assert!(error.source().is_some());
        assert_eq!(runner.calls().len(), 1);
    }
    let (result, runner) = invoke_operation(
        RegisteredOperation::WriteDebt,
        Output::new(
            Some(0),
            vec![b'x'; MAX_CHECK_RESPONSE_BYTES + 1],
            Vec::new(),
        ),
    );
    let error = result.unwrap_err();
    assert!(error.to_string().contains("wrote more than"), "{error}");
    assert!(error.source().is_some());
    assert_eq!(runner.calls().len(), 1);
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
    let write_error =
        RegisteredDebtWrite::from_write(&Err(raf_census::Error::Options("writer cause".into())))
            .unwrap()
            .to_ntriples()
            .unwrap();
    let check_error =
        RegisteredCheck::from_check(&Err(raf_census::Error::Options("check cause".into())))
            .unwrap()
            .to_ntriples()
            .unwrap();
    for (code, text, cause) in [
        (Some(101), "", "target failed with exit"),
        (None, "", "target failed with exit"),
        (Some(2), "", "target failed with exit"),
        (Some(0), write_error.as_str(), "disagrees with protocol"),
        (Some(1), write_error.as_str(), "writer cause"),
        (Some(0), check_error.as_str(), "RegisteredDebtWrite"),
    ] {
        let (result, runner) = invoke_operation(
            RegisteredOperation::WriteDebt,
            Output::new(
                code,
                text.as_bytes().to_vec(),
                b"actual target stderr".to_vec(),
            ),
        );
        let error = result.unwrap_err().to_string();
        assert!(error.contains(cause), "{error}");
        assert_eq!(runner.calls().len(), 1);
    }
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
