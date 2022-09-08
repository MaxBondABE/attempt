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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_negative_intervals_are_rejected() {
        assert!(duration_from_f64(-1.0).is_none())
    }

    #[test]
    fn test_intervals_that_do_not_fit_in_u64_is_rejected() {
        assert!(duration_from_f64((u64::MAX as f64) + 1.0).is_none())
    }

    #[test]
    fn test_zero_intervals_are_accepted() {
        let maybe_duration = duration_from_f64(0.0);
        assert!(maybe_duration.is_some());
        assert_eq!(maybe_duration.unwrap(), Duration::from_secs(0))
    }

    #[test]
    fn test_postitive_intervals_are_accepted() {
        let maybe_duration = duration_from_f64(1.0);
        assert!(maybe_duration.is_some());
        assert_eq!(maybe_duration.unwrap(), Duration::from_secs(1))
    }

    #[test]
    fn test_min_wait_is_respected() {
        assert_eq!(
            process_wait_params(1.0, WaitParameters::new(None, Some(5.0), None)),
            5.0
        );
    }

    #[test]
    fn test_max_wait_is_respected() {
        assert_eq!(
            process_wait_params(10.0, WaitParameters::new(None, None, Some(5.0))),
            5.0
        );
    }

    #[test]
    fn test_jitter() {
        let outputs = (0..3)
            .map(|_| process_wait_params(10.0, WaitParameters::new(Some(1.0), None, None)))
            .collect::<Vec<_>>();
        assert!(outputs.iter().any(|n| *n != 10.0));
        assert!(outputs.iter().all(|n| *n >= 9.0 && *n <= 11.0));
    }

    #[test]
    fn test_jitter_with_min_max() {
        let outputs = (0..3)
            .map(|_| process_wait_params(1.0, WaitParameters::new(Some(5.0), Some(0.5), Some(3.0))))
            .collect::<Vec<_>>();
        assert!(outputs.iter().all(|n| *n >= 0.5 && *n <= 3.0));
    }
}
