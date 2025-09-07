use std::{io, process::Child, time::Duration};

const FIXED_DELAY: f32 = 60.; // 1m
const EXP_MULTIPLIER: f32 = 0.010; // 10ms
const EXP_MAX_DELAY: f32 = 15.; // 15s

pub trait Pollable {
    /// Poll in a non-blocking manner. Return true if the result is ready.
    fn poll(&mut self) -> Result<bool, io::Error>;
}

impl Pollable for Child {
    fn poll(&mut self) -> Result<bool, io::Error> {
        Ok(self.try_wait()?.is_some())
    }
}

/// Poll a child process for completion with exponential backoff. This allows us to
/// poll aggressively in the beginning (as most commands should finish pretty quickly)
/// while not overwhelming the system with I/O for commands that take a long time.
/// Saturates at a MAX_DELAY seconds.
pub fn poll_child<P: Pollable>(
    pollable: &mut P,
    timeout: Duration,
    expected_runtime: Option<Duration>,
) -> Result<bool, io::Error> {
    #[cfg(test)]
    use super::mock_sleep::fake_sleep_for_polling as sleep;
    #[cfg(not(test))]
    use std::thread::sleep;

    #[cfg(test)]
    use super::mock_sleep::MockInstant as Instant;
    #[cfg(not(test))]
    use std::time::Instant;

    let start = Instant::now();

    if let Some(expected) = expected_runtime.map(|rt| rt.min(timeout)) {
        // Use a fixed delay schedule until the expected runtime is exhausted
        // We want to poll during this time in case the child returns early
        // (eg if it crashes), but we want to poll slowly to minimize overhead.
        // We don't check timeouts until the after this block, so we can't wait longer
        // than that.
        let fixed_delays = (expected.as_secs_f32() / FIXED_DELAY).floor();
        if fixed_delays < 1. {
            if pollable.poll()? {
                return Ok(true);
            }
            sleep(expected);
        } else {
            for _ in 0..(fixed_delays as usize) {
                if pollable.poll()? {
                    return Ok(true);
                }
                sleep(Duration::from_secs_f32(FIXED_DELAY));
            }

            if pollable.poll()? {
                return Ok(true);
            }
            if let Some(remaining) = expected.checked_sub(start.elapsed()).and_then(|r| {
                if r > Duration::ZERO {
                    Some(r)
                } else {
                    None
                }
            }) {
                sleep(remaining);
            }
        }
    }

    // Use an exponential delay schedule
    let mut i: i32 = 0;
    loop {
        if pollable.poll()? {
            return Ok(true);
        }
        if let Some(remaining) = timeout.checked_sub(start.elapsed()) {
            let delay = Duration::try_from_secs_f32(
                (EXP_MULTIPLIER * 2f32.powi(i))
                    .min(EXP_MAX_DELAY)
                    .min(remaining.as_secs_f32()),
            )
            .unwrap_or(Duration::from_secs_f32(
                EXP_MAX_DELAY.min(remaining.as_secs_f32()),
            ));
            // This try_from/unwrap_or protects us from NaN, inf, etc.

            sleep(delay);
        } else {
            break;
        }

        i = i.saturating_add(1);
    }

    Ok(false)
}

#[cfg(test)]
mod test {
    use crate::util::mock_sleep::{MockInstant, MockSleep};

    use super::*;

    struct PollableTrue;
    impl Pollable for PollableTrue {
        fn poll(&mut self) -> Result<bool, io::Error> {
            Ok(true)
        }
    }

    struct PollableFalse;
    impl Pollable for PollableFalse {
        fn poll(&mut self) -> Result<bool, io::Error> {
            Ok(false)
        }
    }

    struct SucceedAfterDuration(MockInstant, Duration);
    impl SucceedAfterDuration {
        pub fn new(duration: Duration) -> Self {
            Self(MockInstant::now(), duration)
        }
    }
    impl Pollable for SucceedAfterDuration {
        fn poll(&mut self) -> Result<bool, io::Error> {
            Ok(self.0.elapsed() >= self.1)
        }
    }

    #[test]
    fn poll_returns_immediately_if_result_is_ready() {
        let token = MockSleep::start();
        poll_child(&mut PollableTrue, Duration::from_secs(1), None).unwrap();

        let (poll_delays, _) = MockSleep::take(token);
        assert_eq!(poll_delays.len(), 0);

        let token = MockSleep::start();
        poll_child(
            &mut PollableTrue,
            Duration::from_secs(1),
            Some(Duration::from_secs(1)),
        )
        .unwrap();

        let (poll_delays, _) = MockSleep::take(token);
        assert_eq!(poll_delays.len(), 0);
    }

    #[test]
    fn poll_runs_until_timeout() {
        let expected = Duration::from_secs(1);
        let token = MockSleep::start();
        poll_child(&mut PollableFalse, expected, None).unwrap();

        let (poll_delays, _) = MockSleep::take(token);
        assert_eq!(poll_delays.into_iter().sum::<Duration>(), expected);

        let token = MockSleep::start();
        poll_child(
            &mut PollableFalse,
            Duration::from_secs(1),
            Some(Duration::from_secs_f32(0.5)),
        )
        .unwrap();

        let (poll_delays, _) = MockSleep::take(token);
        assert_eq!(poll_delays.into_iter().sum::<Duration>(), expected);
    }

    #[test]
    fn poll_runs_at_the_very_end() {
        let duration = Duration::from_millis(15);
        let expected = Duration::from_millis(5);
        let token = MockSleep::start();
        // We should wait 10ms, and then wait for the 5ms remainder
        poll_child(&mut PollableFalse, duration, None).unwrap();

        let (mut poll_delays, _) = MockSleep::take(token);
        assert!(poll_delays.len() > 0);
        if *poll_delays.last().unwrap() == Duration::from_nanos(0) {
            // We'll have a 0ns wait when we've reached the timeout; it is an artefact
            // of testing, so throw it away.
            poll_delays.pop();
        }
        assert_eq!(*poll_delays.last().unwrap(), expected);
    }

    #[test]
    fn exponential_wait_time_saturates_at_max_delay() {
        let expected = Duration::from_secs_f32(EXP_MAX_DELAY);
        let attempts = (EXP_MAX_DELAY / 0.010).log2().ceil() + 1.;
        // Number of attempts to exceed a 15s wait time, were we not to saturate at 15s
        let timeout: f32 = (0..=(attempts as i32)).map(|n| 0.010 * 2f32.powi(n)).sum();
        // Timeout to supply to obtain the above number of attempts
        let token = MockSleep::start();

        poll_child(&mut PollableFalse, Duration::from_secs_f32(timeout), None).unwrap();

        let (poll_delays, _) = MockSleep::take(token);
        assert_eq!(poll_delays.into_iter().max().unwrap(), expected);
    }

    #[test]
    fn expected_runtime_less_than_1m() {
        // If the expected runtime is less than 1m, we should wait for that period of time
        let token = MockSleep::start();

        poll_child(
            &mut PollableFalse,
            Duration::from_secs(2),
            Some(Duration::from_secs(1)),
        )
        .unwrap();

        let (poll_delays, _) = MockSleep::take(token);
        assert_eq!(poll_delays.first(), Some(&Duration::from_secs(1)));
        assert_eq!(
            poll_delays.into_iter().sum::<Duration>(),
            Duration::from_secs(2)
        );
    }

    #[test]
    fn expected_runtime_greater_than_1m() {
        // If the expected runtime is > 1m, we should wait in 1m increments
        let token = MockSleep::start();
        let expected_delay = Duration::from_secs(60);
        let expected_minutes = 5;

        poll_child(
            &mut SucceedAfterDuration::new(Duration::from_secs(expected_minutes * 60)),
            Duration::from_secs(1200),
            Some(Duration::from_secs(600)),
        )
        .unwrap();

        let (poll_delays, _) = MockSleep::take(token);
        assert_eq!(poll_delays.len(), expected_minutes as usize);
        for (i, delay) in poll_delays.into_iter().enumerate() {
            assert_eq!(delay, expected_delay, "Delay {i} has the wrong duration")
        }
    }

    #[test]
    fn expected_runtime_not_multiple_of_1m() {
        // We should wait in 1m increments, and then whatever the remainder is
        let token = MockSleep::start();

        poll_child(
            &mut PollableFalse,
            Duration::from_secs(120),
            Some(Duration::from_secs(90)),
        )
        .unwrap();

        let (poll_delays, _) = MockSleep::take(token);
        assert_eq!(poll_delays[0], Duration::from_secs(60));
        assert_eq!(poll_delays[1], Duration::from_secs(30));
    }

    #[test]
    fn timeout_overrides_expected_runtime() {
        let expected = Duration::from_secs(0);
        let token = MockSleep::start();

        poll_child(&mut PollableFalse, Duration::ZERO, Some(expected)).unwrap();

        let (poll_delays, _) = MockSleep::take(token);
        assert_eq!(poll_delays.into_iter().sum::<Duration>(), expected);
    }
}
