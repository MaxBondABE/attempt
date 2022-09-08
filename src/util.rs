use std::time::Duration;

use rand_distr::{Distribution, Uniform};

use crate::arguments::WaitParameters;

pub(crate) fn duration_from_f64(interval: f64) -> Option<Duration> {
    let millis = 1000.0 * interval;
    if millis >= 0.0 && millis < u64::MAX as f64 {
        Some(Duration::from_millis(millis as u64))
    } else {
        None
    }
}

pub(crate) fn process_wait_params(interval: f64, params: WaitParameters) -> f64 {
    let jitter_seconds = match params.jitter {
        Some(n) => Uniform::new_inclusive(-n, n).sample(&mut rand::thread_rng()),
        None => 0.0,
    };
    (interval + jitter_seconds)
        .max(params.wait_min.unwrap_or(0.0))
        .min(params.wait_max.unwrap_or(f64::MAX))
}

pub(crate) fn create_duration(interval: f64, wait_params: WaitParameters) -> Duration {
    duration_from_f64(process_wait_params(interval, wait_params))
        .expect("Failed to build a duration")
}
