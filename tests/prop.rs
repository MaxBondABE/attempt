use std::time::Instant;

use assert_cmd::Command;
use predicates::prelude::predicate;

mod util;

use util::{assert_average_percent_error, RETRIES_EXHAUSTED, TEST_TIMEOUT};

#[test]
fn foo() {
    let attempts = 3.;
    let wait = 0.050;
    let expected = wait * (attempts - 1.);
    // attempts - 1 because we don't sleep for the last attempt

    assert_average_percent_error(
        || {
            let mut cmd = Command::cargo_bin("attempt").unwrap();
            cmd.arg("fixed")
                .arg("--attempts")
                .arg(attempts.to_string())
                .arg("--wait")
                .arg(wait.to_string());
            cmd.arg("/bin/false");

            cmd.timeout(TEST_TIMEOUT);
            let start = Instant::now();
            cmd.assert().code(predicate::eq(RETRIES_EXHAUSTED));

            start.elapsed().as_secs_f32()
        },
        expected,
        20.,
    );
}
