use std::{
    cell::OnceCell,
    io,
    process::{self, Child, ExitStatus},
    str::from_utf8,
};

use log::{debug, trace};

use crate::{arguments::PolicyParameters, SUCCESS};

const STRING_ERR_MSG: &str = "Failed to parse command output as UTF-8.";

pub trait OutputShim {
    fn status_code(&self) -> Option<i32>;
    fn stdout(&self) -> &str;
    fn stderr(&self) -> &str;
}

#[derive(Debug)]
pub struct OutputWrapper<'a> {
    output: &'a process::Output,
    stdout: &'a OnceCell<&'a str>,
    stderr: &'a OnceCell<&'a str>,
}
impl OutputShim for OutputWrapper<'_> {
    fn status_code(&self) -> Option<i32> {
        self.output.status.code()
    }

    fn stdout(&self) -> &str {
        self.stdout
            .get_or_init(|| from_utf8(&self.output.stdout).expect(STRING_ERR_MSG))
    }

    fn stderr(&self) -> &str {
        self.stderr
            .get_or_init(|| from_utf8(&self.output.stdout).expect(STRING_ERR_MSG))
    }
}

impl PolicyParameters {
    fn evaluate_stop_predicates(&self, output: impl OutputShim, timed_out: bool) -> bool {
        trace!("Evaluating stop predicates...");

        // Status code & signal control
        if self.stop_if_timeout & timed_out {
            debug!("Stop: Timeout.");
            return true;
        }

        if let Some(code) = output.status_code() {
            if let Some(pattern) = self.stop_if_status.as_ref() {
                if pattern.contains(code) {
                    debug!("Stop: Status matches.");
                    return true;
                }
            }
        } else if self.stop_if_killed {
            debug!("Stop: Command killed by signal.");
            return true;
        }

        // Output
        if let Some(output_str) = self.stop_if_contains.as_ref() {
            if output.stdout().contains(output_str) {
                debug!("Stop: stdout contained string '{}'.", output_str);
                return true;
            }

            if output.stderr().contains(output_str) {
                debug!("Stop: stderr contained string '{}'.", output_str);
                return true;
            }
        }
        if let Some(output_rgx) = self.stop_if_matches.as_ref() {
            if output_rgx.is_match(output.stdout()) {
                debug!("Stop: stdout matched regex '{}'.", output_rgx);
                return true;
            }

            if output_rgx.is_match(output.stderr()) {
                debug!("Stop: stderr matched regex '{}'.", output_rgx);
                return true;
            }
        }
        if let Some(stdout_str) = self.stop_if_stdout_contains.as_ref() {
            if output.stdout().contains(stdout_str) {
                debug!("Stop: stdout contained string '{}'.", stdout_str);
                return true;
            }
        }
        if let Some(stdout_rgx) = self.stop_if_stdout_matches.as_ref() {
            if stdout_rgx.is_match(output.stdout()) {
                debug!("Stop: stdout matched regex '{}'.", stdout_rgx);
                return true;
            }
        }
        if let Some(stderr_str) = self.stop_if_stderr_contains.as_ref() {
            if output.stderr().contains(stderr_str) {
                debug!("Stop: stdout contained string '{}'.", stderr_str);
                return true;
            }
        }
        if let Some(stderr_rgx) = self.stop_if_stderr_matches.as_ref() {
            if stderr_rgx.is_match(output.stderr()) {
                debug!("Stop: stderr matched regex '{}'.", stderr_rgx);
                return true;
            }
        }

        false
    }

    fn evaluate_retry_predicates(&self, output: impl OutputShim, forever: bool) -> bool {
        trace!("Evaluating retry predicates...");

        if self.retry_always || forever {
            debug!("Retry: Retrying by default.");
            return true;
        }

        // Status code
        if let Some(pattern) = self.retry_if_status.as_ref() {
            if let Some(code) = output.status_code() {
                if pattern.contains(code) {
                    debug!("Retry: Status matches.");
                    return true;
                }
            }
        }
        if self.retry_failing_status {
            if let Some(code) = output.status_code() {
                if code != SUCCESS {
                    debug!("Retry: Command exited with failing status.");
                    return true;
                }
            }
        }

        // Output
        if let Some(output_str) = self.retry_if_contains.as_ref() {
            if output.stdout().contains(output_str) {
                debug!("Retry: stdout contained string '{}'.", output_str);
                return true;
            }

            if output.stderr().contains(output_str) {
                debug!("Retry: stderr contained string '{}'.", output_str);
                return true;
            }
        }
        if let Some(output_rgx) = self.retry_if_matches.as_ref() {
            if output_rgx.is_match(output.stdout()) {
                debug!("Retry: stdout matched regex '{}'.", output_rgx);
                return true;
            }

            if output_rgx.is_match(output.stderr()) {
                debug!("Retry: stderr matched regex '{}'.", output_rgx);
                return true;
            }
        }
        if let Some(stdout_str) = self.retry_if_stdout_contains.as_ref() {
            if output.stdout().contains(stdout_str) {
                debug!("Retry: stdout contained string '{}'.", stdout_str);
                return true;
            }
        }
        if let Some(stdout_rgx) = self.retry_if_stdout_matches.as_ref() {
            if stdout_rgx.is_match(output.stdout()) {
                debug!("Retry: stdout matched regex '{}'.", stdout_rgx);
                return true;
            }
        }
        if let Some(stderr_str) = self.retry_if_stderr_contains.as_ref() {
            if output.stderr().contains(stderr_str) {
                debug!("Retry: stderr contained string '{}'.", stderr_str);
                return true;
            }
        }
        if let Some(stderr_rgx) = self.retry_if_stderr_matches.as_ref() {
            if stderr_rgx.is_match(output.stderr()) {
                debug!("Retry: stderr matched regex '{}'.", stderr_rgx);
                return true;
            }
        }

        false
    }

    pub fn evaluate_policy(
        &self,
        mut child: Child,
        timed_out: bool,
        forever: bool,
    ) -> Result<(bool, ExitStatus), io::Error> {
        trace!("Evaluating policy...");

        if self.default_behavior() {
            let status = child.wait()?;
            debug!("Child has exited with {}.", status);
            if status.success() {
                debug!("Stop: Command was successful.");
                return Ok((false, status));
            } else {
                debug!("Retry: Command failed.");
                return Ok((true, status));
            }
        }

        let output = child.wait_with_output()?;
        let stdout: OnceCell<&str> = OnceCell::new();
        let stderr: OnceCell<&str> = OnceCell::new();
        debug!("Command exited with status: {}.", output.status);

        // NB: Stop predicates have precedence over retry predicates
        if self.evaluate_stop_predicates(
            OutputWrapper {
                output: &output,
                stdout: &stdout,
                stderr: &stderr,
            },
            timed_out,
        ) {
            return Ok((false, output.status));
        };

        if self.evaluate_retry_predicates(
            OutputWrapper {
                output: &output,
                stdout: &stdout,
                stderr: &stderr,
            },
            forever,
        ) {
            return Ok((true, output.status));
        };

        debug!("Stop: Stopping by default.");
        Ok((false, output.status))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::status::StatusCodePattern;
    use regex::Regex;

    struct Successful;
    impl OutputShim for Successful {
        fn status_code(&self) -> Option<i32> {
            Some(0)
        }
        fn stdout(&self) -> &str {
            ""
        }
        fn stderr(&self) -> &str {
            ""
        }
    }

    struct FailingStatusCode;
    impl OutputShim for FailingStatusCode {
        fn status_code(&self) -> Option<i32> {
            Some(1)
        }
        fn stdout(&self) -> &str {
            ""
        }
        fn stderr(&self) -> &str {
            ""
        }
    }

    struct Killed;
    impl OutputShim for Killed {
        fn status_code(&self) -> Option<i32> {
            None
        }
        fn stdout(&self) -> &str {
            ""
        }
        fn stderr(&self) -> &str {
            ""
        }
    }

    struct PrintsFooStdout;
    impl OutputShim for PrintsFooStdout {
        fn status_code(&self) -> Option<i32> {
            Some(0)
        }
        fn stdout(&self) -> &str {
            "foo"
        }
        fn stderr(&self) -> &str {
            ""
        }
    }

    struct PrintsFooStderr;
    impl OutputShim for PrintsFooStderr {
        fn status_code(&self) -> Option<i32> {
            Some(0)
        }
        fn stdout(&self) -> &str {
            ""
        }
        fn stderr(&self) -> &str {
            "foo"
        }
    }

    struct PrintsBarStdout;
    impl OutputShim for PrintsBarStdout {
        fn status_code(&self) -> Option<i32> {
            Some(0)
        }
        fn stdout(&self) -> &str {
            "bar"
        }
        fn stderr(&self) -> &str {
            ""
        }
    }

    struct PrintsBarStderr;
    impl OutputShim for PrintsBarStderr {
        fn status_code(&self) -> Option<i32> {
            Some(0)
        }
        fn stdout(&self) -> &str {
            ""
        }
        fn stderr(&self) -> &str {
            "bar"
        }
    }

    #[test]
    fn stop_if_timeout() {
        let policy = PolicyParameters {
            stop_if_timeout: true,
            ..Default::default()
        };

        assert!(policy.evaluate_stop_predicates(Successful, true));
        assert!(!policy.evaluate_stop_predicates(Successful, false));
    }

    #[test]
    fn stop_if_status() {
        let policy = PolicyParameters {
            stop_if_status: Some(StatusCodePattern::only(1)),
            ..Default::default()
        };

        assert!(policy.evaluate_stop_predicates(FailingStatusCode, false));
        assert!(!policy.evaluate_stop_predicates(Successful, false));
    }

    #[test]
    fn stop_if_killed() {
        let policy = PolicyParameters {
            stop_if_killed: true,
            ..Default::default()
        };

        assert!(policy.evaluate_stop_predicates(Killed, false));
        assert!(!policy.evaluate_stop_predicates(Successful, false));
    }

    #[test]
    fn stop_if_contains() {
        let policy = PolicyParameters {
            stop_if_contains: Some("foo".to_string()),
            ..Default::default()
        };

        assert!(policy.evaluate_stop_predicates(PrintsFooStdout, false));
        assert!(policy.evaluate_stop_predicates(PrintsFooStderr, false));
        assert!(!policy.evaluate_retry_predicates(PrintsBarStdout, false));
        assert!(!policy.evaluate_stop_predicates(Successful, false));

        let policy = PolicyParameters {
            stop_if_stdout_contains: Some("foo".to_string()),
            ..Default::default()
        };

        assert!(policy.evaluate_stop_predicates(PrintsFooStdout, false));
        assert!(!policy.evaluate_stop_predicates(PrintsFooStderr, false));
        assert!(!policy.evaluate_retry_predicates(PrintsBarStdout, false));
        assert!(!policy.evaluate_stop_predicates(Successful, false));

        let policy = PolicyParameters {
            stop_if_stderr_contains: Some("foo".to_string()),
            ..Default::default()
        };

        assert!(policy.evaluate_stop_predicates(PrintsFooStderr, false));
        assert!(!policy.evaluate_stop_predicates(PrintsFooStdout, false));
        assert!(!policy.evaluate_retry_predicates(PrintsBarStdout, false));
        assert!(!policy.evaluate_stop_predicates(Successful, false));
    }

    #[test]
    fn stop_if_matches() {
        let policy = PolicyParameters {
            stop_if_matches: Some(Regex::new("foo").unwrap()),
            ..Default::default()
        };

        assert!(policy.evaluate_stop_predicates(PrintsFooStdout, false));
        assert!(policy.evaluate_stop_predicates(PrintsFooStderr, false));
        assert!(!policy.evaluate_stop_predicates(PrintsBarStdout, false));
        assert!(!policy.evaluate_stop_predicates(Successful, false));

        let policy = PolicyParameters {
            stop_if_stdout_matches: Some(Regex::new("foo").unwrap()),
            ..Default::default()
        };

        assert!(policy.evaluate_stop_predicates(PrintsFooStdout, false));
        assert!(!policy.evaluate_stop_predicates(PrintsFooStderr, false));
        assert!(!policy.evaluate_stop_predicates(PrintsBarStdout, false));
        assert!(!policy.evaluate_stop_predicates(Successful, false));

        let policy = PolicyParameters {
            stop_if_stderr_matches: Some(Regex::new("foo").unwrap()),
            ..Default::default()
        };

        assert!(policy.evaluate_stop_predicates(PrintsFooStderr, false));
        assert!(!policy.evaluate_stop_predicates(PrintsFooStdout, false));
        assert!(!policy.evaluate_stop_predicates(PrintsBarStderr, false));
        assert!(!policy.evaluate_stop_predicates(Successful, false));
    }

    #[test]
    fn retry_if_status() {
        let policy = PolicyParameters {
            retry_if_status: Some(StatusCodePattern::only(1)),
            ..Default::default()
        };

        assert!(policy.evaluate_retry_predicates(FailingStatusCode, false));
        assert!(!policy.evaluate_retry_predicates(Successful, false));
    }

    #[test]
    fn retry_failing_status() {
        let policy = PolicyParameters {
            retry_failing_status: true,
            ..Default::default()
        };

        assert!(policy.evaluate_retry_predicates(FailingStatusCode, false));
        assert!(!policy.evaluate_retry_predicates(Successful, false));
    }

    #[test]
    fn retry_always_and_forever() {
        let policy = PolicyParameters {
            retry_always: true,
            ..Default::default()
        };

        assert!(policy.evaluate_retry_predicates(FailingStatusCode, false));
        assert!(policy.evaluate_retry_predicates(Killed, false));
        assert!(policy.evaluate_retry_predicates(Successful, false));

        let policy = PolicyParameters::default();

        assert!(policy.evaluate_retry_predicates(FailingStatusCode, true));
        assert!(policy.evaluate_retry_predicates(Killed, true));
        assert!(policy.evaluate_retry_predicates(Successful, true));
    }

    #[test]
    fn retry_if_contains() {
        let policy = PolicyParameters {
            retry_if_contains: Some("foo".to_string()),
            ..Default::default()
        };

        assert!(policy.evaluate_retry_predicates(PrintsFooStdout, false));
        assert!(policy.evaluate_retry_predicates(PrintsFooStderr, false));
        assert!(!policy.evaluate_retry_predicates(PrintsBarStdout, false));
        assert!(!policy.evaluate_retry_predicates(Successful, false));

        let policy = PolicyParameters {
            retry_if_stdout_contains: Some("foo".to_string()),
            ..Default::default()
        };

        assert!(policy.evaluate_retry_predicates(PrintsFooStdout, false));
        assert!(!policy.evaluate_retry_predicates(PrintsFooStderr, false));
        assert!(!policy.evaluate_retry_predicates(PrintsBarStdout, false));
        assert!(!policy.evaluate_retry_predicates(Successful, false));

        let policy = PolicyParameters {
            retry_if_stderr_contains: Some("foo".to_string()),
            ..Default::default()
        };

        assert!(policy.evaluate_retry_predicates(PrintsFooStderr, false));
        assert!(!policy.evaluate_retry_predicates(PrintsFooStdout, false));
        assert!(!policy.evaluate_retry_predicates(PrintsBarStdout, false));
        assert!(!policy.evaluate_retry_predicates(Successful, false));
    }

    #[test]
    fn retry_if_matches() {
        let policy = PolicyParameters {
            retry_if_matches: Some(Regex::new("foo").unwrap()),
            ..Default::default()
        };

        assert!(policy.evaluate_retry_predicates(PrintsFooStdout, false));
        assert!(policy.evaluate_retry_predicates(PrintsFooStderr, false));
        assert!(!policy.evaluate_retry_predicates(PrintsBarStdout, false));
        assert!(!policy.evaluate_retry_predicates(Successful, false));

        let policy = PolicyParameters {
            retry_if_stdout_matches: Some(Regex::new("foo").unwrap()),
            ..Default::default()
        };

        assert!(policy.evaluate_retry_predicates(PrintsFooStdout, false));
        assert!(!policy.evaluate_retry_predicates(PrintsFooStderr, false));
        assert!(!policy.evaluate_retry_predicates(PrintsBarStdout, false));
        assert!(!policy.evaluate_retry_predicates(Successful, false));

        let policy = PolicyParameters {
            retry_if_stderr_matches: Some(Regex::new("foo").unwrap()),
            ..Default::default()
        };

        assert!(policy.evaluate_retry_predicates(PrintsFooStderr, false));
        assert!(!policy.evaluate_retry_predicates(PrintsFooStdout, false));
        assert!(!policy.evaluate_retry_predicates(PrintsBarStderr, false));
        assert!(!policy.evaluate_retry_predicates(Successful, false));
    }
}
