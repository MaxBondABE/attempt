#![deny(clippy::dbg_macro)]

mod arguments;
mod policy;
mod util;

use std::{io, ops::ControlFlow, process, time::Duration};

use arguments::{parse_arguments, AttemptArguments};
use log::{debug, error, info, trace, warn};
use util::{logger::Logger, poll::poll_child};

// NB: Must stay in sync with tests/util.rs
const SUCCESS: i32 = 0;
const IO_ERROR: i32 = 1;
// 2 is used by clap for invalid args
const RETRIES_EXHAUSTED: i32 = 3;
const STOPPED: i32 = 4;
// 101 is used by Rust during a panic

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    Success,
    RetriesExhausted,
    Stopped,
}
impl From<Outcome> for i32 {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Success => SUCCESS,
            Outcome::RetriesExhausted => RETRIES_EXHAUSTED,
            Outcome::Stopped => STOPPED,
        }
    }
}

fn attempt(args: AttemptArguments) -> Result<Outcome, io::Error> {
    #[cfg(not(test))]
    use std::thread::sleep;
    #[cfg(test)]
    use util::mock_sleep::fake_sleep_for_attempt as sleep;

    if let Some(delay) = args.wait_params.stagger_delay() {
        info!("Staggering by {:.2} seconds", delay.as_secs_f32());
        sleep(delay)
    }

    for (duration, last) in args.backoff() {
        trace!("Starting new attempt...");

        let mut command = args.build_command();
        let mut child = command.spawn()?;
        // NB: The methods on Command to access the output cause the command
        // to be run each time they are called. Only the methods on Child are
        // safe/guarenteed not to rerun the command.

        let mut timed_out = false;
        if let Some(t) = args.timeout {
            let timeout = Duration::from_secs_f32(t);

            trace!("Polling child command...");
            if !poll_child(&mut child, timeout, None)? {
                debug!("Child command has timed out; sending signal...");
                timed_out = true;
                child.kill()?;
            } else {
                trace!("Child command has exited.");
            }
        }

        let (retry, status) = args.evaluate_policy(child, timed_out)?;
        match retry {
            ControlFlow::Break(()) => {
                if status.success() {
                    debug!("Terminated: Success.");
                    return Ok(Outcome::Success);
                } else {
                    debug!("Terminated: Command has failed, but cannot be retried.");
                    return Ok(Outcome::Stopped);
                }
            }
            ControlFlow::Continue(()) if !last => {
                // Only sleep if we have at least 1 more attempt; if we're going to fail,
                // we should fail as fast as possible.
                if duration >= Duration::from_secs(1) {
                    debug!(
                        "Command has failed, retrying in {:.2} seconds...",
                        duration.as_secs_f32()
                    )
                } else {
                    debug!(
                        "Command has failed, retrying in {} milliseconds...",
                        duration.as_millis()
                    )
                }
                sleep(duration);
            }
            _ => (),
        }
    }

    debug!("Terminated: Retries exhausted.");
    Ok(Outcome::RetriesExhausted)
}

fn main() {
    let args = parse_arguments();
    args.validate();

    Logger::new(args.verbose, args.quiet).init().unwrap();

    match attempt(args) {
        Ok(outcome) => process::exit(outcome.into()),
        Err(e) => {
            error!("Failed: {}", e);
            match e.kind() {
                io::ErrorKind::NotFound => {
                    warn!("Does the command exist & is it on the path? Is it spelled correctly?")
                }
                io::ErrorKind::PermissionDenied => {
                    warn!("Does the file have the executable permission set? Are we using the correct user/group?")
                }

                _ => (),
            }
            process::exit(IO_ERROR);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arguments::parse_arguments_from;

    #[test]
    fn happy_path_smoke_test_fixed() {
        let args = parse_arguments_from(["attempt", "/bin/true"]);
        assert_eq!(attempt(args).ok(), Some(Outcome::Success));

        let args = parse_arguments_from(["attempt", "fixed", "/bin/true"]);
        assert_eq!(attempt(args).ok(), Some(Outcome::Success));
    }

    #[test]
    fn happy_path_smoke_test_exp() {
        let args = parse_arguments_from(["attempt", "exponential", "/bin/true"]);
        assert_eq!(attempt(args).ok(), Some(Outcome::Success));
    }

    #[test]
    fn happy_path_smoke_test_linear() {
        let args = parse_arguments_from(["attempt", "linear", "/bin/true"]);
        assert_eq!(attempt(args).ok(), Some(Outcome::Success));
    }

    #[test]
    fn sad_path_smoke_test_fixed() {
        let args = parse_arguments_from(["attempt", "/bin/false"]);
        assert_eq!(attempt(args).ok(), Some(Outcome::RetriesExhausted));

        let args = parse_arguments_from(["attempt", "fixed", "/bin/false"]);
        assert_eq!(attempt(args).ok(), Some(Outcome::RetriesExhausted));
    }

    #[test]
    fn sad_path_smoke_test_exp() {
        let args = parse_arguments_from(["attempt", "exponential", "/bin/false"]);
        assert_eq!(attempt(args).ok(), Some(Outcome::RetriesExhausted));
    }

    #[test]
    fn sad_path_smoke_test_linear() {
        let args = parse_arguments_from(["attempt", "linear", "/bin/false"]);
        assert_eq!(attempt(args).ok(), Some(Outcome::RetriesExhausted));
    }
}
