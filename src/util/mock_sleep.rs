use std::{
    mem,
    sync::{Mutex, MutexGuard},
    time::Duration,
};

// FIXME Remove this dependency once lazy_cell is stabilized
// https://github.com/rust-lang/rust/issues/109736
use once_cell::sync::Lazy;

const MIN_SLEEP: Duration = Duration::from_nanos(1);

// The context lock serializes tests using the mock, so that they don't see
// each other's writes
static CONTEXT: Mutex<()> = Mutex::new(());
static MOCK_SLEEP: Lazy<Mutex<MockSleep>> = Lazy::new(|| Mutex::new(Default::default()));
static MOCK_INSTANT_GLOBAL_TIME: Lazy<Mutex<Duration>> =
    Lazy::new(|| Mutex::new(Default::default()));

#[allow(dead_code)]
pub struct ContextToken<'a>(MutexGuard<'a, ()>);

/// Global state for mocked sleep
pub struct MockSleep {
    poll_delays: Vec<Duration>,
    attempt_delays: Vec<Duration>,
}
impl MockSleep {
    fn take_inner(&mut self) -> (Vec<Duration>, Vec<Duration>) {
        let mut poll_delays = Vec::with_capacity(16);
        let mut attempt_delays = Vec::with_capacity(16);
        mem::swap(&mut poll_delays, &mut self.poll_delays);
        mem::swap(&mut attempt_delays, &mut self.attempt_delays);
        (poll_delays, attempt_delays)
    }
    fn clear(&mut self) {
        self.poll_delays.clear();
        self.attempt_delays.clear();
    }
    pub fn take(token: ContextToken) -> (Vec<Duration>, Vec<Duration>) {
        let inner = MOCK_SLEEP.lock().unwrap().take_inner();
        drop(token);
        inner
    }
    pub fn start<'a>() -> ContextToken<'a> {
        let guard = CONTEXT.lock().unwrap();
        MOCK_SLEEP.lock().unwrap().clear();
        ContextToken(guard)
    }
}
impl Default for MockSleep {
    fn default() -> Self {
        Self {
            poll_delays: Vec::with_capacity(16),
            attempt_delays: Vec::with_capacity(16),
        }
    }
}

/// Mocked std::time::Instant
/// This is inspired by fake_clock/sn_fake_clock, which appears abandoned. The repo was first
/// moved and then deleted.
// FIXME Should be replaced when the community has rallied behind a
// reliable fork of fake_clock
pub struct MockInstant {
    begun: Duration,
}
impl MockInstant {
    /// Mocked std::time::Instant::now()
    pub fn now() -> Self {
        Self {
            begun: MOCK_INSTANT_GLOBAL_TIME.lock().unwrap().clone(),
        }
    }
    /// Mocked std::time::Instant::elapsed()
    pub fn elapsed(&self) -> Duration {
        *MOCK_INSTANT_GLOBAL_TIME.lock().unwrap() - self.begun
    }
}

/// Mocked std::time::sleep() for use by poll_child()
pub fn fake_sleep_for_polling(duration: Duration) {
    MOCK_SLEEP.lock().unwrap().poll_delays.push(duration);
    *MOCK_INSTANT_GLOBAL_TIME.lock().unwrap() += duration.max(MIN_SLEEP);
    // If we sleep 0ns, we'll get caught in an infinite loop, due to the use
    // of Duration::checked_sub. This is an artefact of testing, so we hack
    // around it here rather than adding a special case.
}

/// Mocked std::time::sleep() for use by attempt()
pub fn fake_sleep_for_attempt(duration: Duration) {
    MOCK_SLEEP.lock().unwrap().attempt_delays.push(duration);
    *MOCK_INSTANT_GLOBAL_TIME.lock().unwrap() += duration.max(MIN_SLEEP);
    // If we sleep 0ns, we'll get caught in an infinite loop, due to the use
    // of Duration::checked_sub. This is an artefact of testing, so we hack
    // around it here rather than adding a special case.
}
