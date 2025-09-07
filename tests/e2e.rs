use std::time::Instant;

use assert_cmd::Command;
use predicates::prelude::*;
use util::{assert_average_percent_error, IO_ERROR, RETRIES_EXHAUSTED, STOPPED, TEST_TIMEOUT};

mod util;

// NB: `cmd` is a command in the parlance of `assert_cmd` - it refers to
// `attempt` itself, not to it's child command

#[test]
fn happy_path_smoke_test_fixed() {
    let mut cmd = Command::cargo_bin("attempt").unwrap();
    cmd.arg("--verbose");
    cmd.arg("fixed");
    cmd.arg("/bin/true");

    cmd.timeout(TEST_TIMEOUT);
    cmd.assert().success();
}

#[test]
fn happy_path_smoke_test_exp() {
    let mut cmd = Command::cargo_bin("attempt").unwrap();
    cmd.arg("exponential");
    cmd.arg("/bin/true");

    cmd.timeout(TEST_TIMEOUT);
    cmd.assert().success();
}

#[test]
fn happy_path_smoke_test_linear() {
    let mut cmd = Command::cargo_bin("attempt").unwrap();
    cmd.arg("linear");
    cmd.arg("/bin/true");

    cmd.timeout(TEST_TIMEOUT);
    cmd.assert().success();
}

#[test]
fn sad_path_smoke_test_fixed() {
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

#[test]
fn sad_path_smoke_test_exp() {
    let attempts = 3;
    let multiplier: f32 = 0.050;
    let base: f32 = 2.;
    let expected: f32 = (0..(attempts - 1)).map(|n| multiplier * base.powi(n)).sum();
    // attempts - 1 because we don't sleep for the last attempt

    assert_average_percent_error(
        || {
            let mut cmd = Command::cargo_bin("attempt").unwrap();
            cmd.arg("exponential")
                .arg("--attempts")
                .arg(attempts.to_string())
                .arg("--base")
                .arg(base.to_string())
                .arg("--multiplier")
                .arg(multiplier.to_string());
            cmd.arg("/bin/false");

            cmd.timeout(TEST_TIMEOUT);
            let start = Instant::now();
            cmd.assert().code(predicate::eq(RETRIES_EXHAUSTED));

            start.elapsed().as_secs_f32()
        },
        expected,
        15.,
    );
}

#[test]
fn sad_path_smoke_test_linear() {
    let attempts = 3;
    let multiplier: f32 = 0.050;
    let starting_wait: f32 = 0.250;
    let expected = (0..(attempts - 1))
        .map(|n| multiplier * (n as f32) + starting_wait)
        .sum();
    // attempts - 1 because we don't sleep for the last attempt

    assert_average_percent_error(
        || {
            let mut cmd = Command::cargo_bin("attempt").unwrap();
            cmd.arg("linear")
                .arg("--attempts")
                .arg(attempts.to_string())
                .arg("--starting-wait")
                .arg(starting_wait.to_string())
                .arg("--multiplier")
                .arg(multiplier.to_string());
            cmd.arg("/bin/false");

            cmd.timeout(TEST_TIMEOUT);
            let start = Instant::now();
            cmd.assert().code(predicate::eq(RETRIES_EXHAUSTED));

            start.elapsed().as_secs_f32()
        },
        expected,
        10.,
    );
}

#[test]
fn timeouts_are_respected() {
    let attempts = 1;
    let timeout = 0.500;
    let expected = timeout;

    assert_average_percent_error(
        || {
            let mut cmd = Command::cargo_bin("attempt").unwrap();
            cmd.arg("--attempts")
                .arg(attempts.to_string())
                .arg("--timeout")
                .arg(timeout.to_string());
            cmd.arg("--")
                .arg("/bin/sh")
                .arg("-c")
                .arg("while [ 1 ]; do sleep 0.1; done");
            // FIXME for unclear reasons, something like `sleep 5` doesn't return
            // until the sleep is over, despite being killed. Even with --force-kill.
            // When I test manually I don't reproduce. Using a loop is a workaround.
            // See below.
            //
            // > time cargo --quiet run -- --attempts 1 --timeout 0.5 -- /bin/sh -c "sleep 5"
            //   real	0m0.581s
            //   user	0m0.061s
            //   sys	0m0.021s

            cmd.timeout(TEST_TIMEOUT);
            let start = Instant::now();
            cmd.assert().code(predicate::eq(RETRIES_EXHAUSTED));

            start.elapsed().as_secs_f32()
        },
        expected,
        10.,
    );
}

#[test]
fn command_not_found() {
    let mut cmd = Command::cargo_bin("attempt").unwrap();
    cmd.arg("/not/there");

    cmd.timeout(TEST_TIMEOUT);
    cmd.assert().code(predicate::eq(IO_ERROR));
}

#[test]
fn command_failed() {
    let mut cmd = Command::cargo_bin("attempt").unwrap();
    cmd.arg("--retry-if-status").arg("10");
    cmd.arg("/bin/false");

    cmd.timeout(TEST_TIMEOUT);
    cmd.assert().code(predicate::eq(STOPPED));
}

#[test]
fn staggering() {
    const STAGGER: f32 = 1.0;

    fn with_stagger() -> f32 {
        let mut cmd = Command::cargo_bin("attempt").unwrap();
        cmd.arg("--stagger").arg(STAGGER.to_string());
        cmd.arg("/bin/true");

        cmd.timeout(TEST_TIMEOUT + Duration::from_secs_f32(STAGGER));
        let start = Instant::now();
        cmd.assert().success();

        start.elapsed().as_secs_f32()
    }

    fn without_stagger() -> f32 {
        let mut cmd = Command::cargo_bin("attempt").unwrap();
        cmd.arg("/bin/true");

        cmd.timeout(TEST_TIMEOUT);
        let start = Instant::now();
        cmd.assert().success();

        start.elapsed().as_secs_f32()
    }

    fn timing_difference(i: usize) -> f32 {
        // By taking both measurements concurrently, we negate errors caused by
        // varying load on the system over the course of a test
        if (i & 1) == 0 {
            let with = thread::spawn(with_stagger);
            let without = thread::spawn(without_stagger);
            with.join().unwrap() - without.join().unwrap()
        } else {
            let without = thread::spawn(without_stagger);
            let with = thread::spawn(with_stagger);
            with.join().unwrap() - without.join().unwrap()
        }
    }

    fn sample_timing_difference(n: usize) -> f32 {
        let mut samples: Vec<JoinHandle<f32>> = Vec::with_capacity(n);
        for i in 0..n {
            if i > 0 {
                thread::sleep(Duration::from_millis(10))
            }
            samples.push(thread::spawn(move || timing_difference(i)));
        }
        let total: f32 = samples.drain(..).map(|handle| handle.join().unwrap()).sum();
        total / n as f32
    }

    // - The timing difference should be a function of STAGGER
    // - Because the expectation of a uniform distribution is it's middle, many
    //   samples of the timing difference should converge to half of STAGGER
    // - Otherwise, there is a bug (or bad luck - adjust the sensitivity accordingly)
    assert_average_percent_error(|| sample_timing_difference(8), STAGGER / 2., 30.);
}


#[cfg(unix)]
#[test]
fn retry_on_signal() {
    let mut cmd = Command::cargo_bin("attempt").unwrap();
    cmd.arg("--attempts").arg("2");
    cmd.arg("--wait").arg("0.1");
    cmd.arg("--retry-if-signal").arg("9");
    cmd.arg("--").arg("/bin/sh").arg("-c").arg("kill -9 $$");

    cmd.timeout(Duration::from_secs(5));
    cmd.assert().code(predicate::eq(RETRIES_EXHAUSTED));
}

#[cfg(unix)]
#[test]
fn stop_on_signal() {
    let mut cmd = Command::cargo_bin("attempt").unwrap();
    cmd.arg("--attempts").arg("5");
    cmd.arg("--wait").arg("0.1");
    cmd.arg("--stop-if-signal").arg("9");
    cmd.arg("--").arg("/bin/sh").arg("-c").arg("kill -9 $$");

    cmd.timeout(Duration::from_secs(2));
    cmd.assert().code(predicate::eq(STOPPED));
}

