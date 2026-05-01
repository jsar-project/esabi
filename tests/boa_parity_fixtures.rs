#![cfg(feature = "engine-boa")]

use boa_parity_fixtures::{fixtures, run_fixture};

#[test]
fn boa_parity_fixtures_regressions() {
    for fixture in fixtures() {
        let output = run_fixture(fixture);

        if let Some(expected_error) = fixture.expected_error_contains {
            assert!(
                !output.ok,
                "fixture {} should fail but succeeded with {:?}",
                fixture.id,
                output
            );
            let error = output
                .error
                .as_deref()
                .unwrap_or("<missing error output>");
            assert!(
                error.contains(expected_error),
                "fixture {} error {:?} does not contain {:?}",
                fixture.id,
                error,
                expected_error
            );
            continue;
        }

        assert!(output.ok, "fixture {} failed with {:?}", fixture.id, output);
        assert_eq!(
            output.result.as_deref(),
            fixture.expected_result,
            "fixture {} returned unexpected result",
            fixture.id
        );
        assert_eq!(
            output.logs.iter().map(String::as_str).collect::<Vec<_>>(),
            fixture.expected_logs,
            "fixture {} returned unexpected logs",
            fixture.id
        );
    }
}
