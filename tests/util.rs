use std::time::Duration;

// NB: Must stay in sync with src/main.rs
pub const IO_ERROR: i32 = 1;
pub const RETRIES_EXHAUSTED: i32 = 3;
pub const STOPPED: i32 = 4;

pub const TEST_TIMEOUT: Duration = Duration::from_secs(1);

pub fn unsigned_percent_error(measured: f32, expected: f32) -> f32 {
    100. * (measured - expected).abs() / expected
}

pub fn assert_percent_error(measured: f32, expected: f32, threshold: f32) {
    let pct_err = unsigned_percent_error(measured, expected);
    if pct_err >= threshold {
        panic!(
            "Error is too high (measured {}% threshold {}%)",
            pct_err, threshold
        );
    }
}

/// Assert the unsigned percentage error of a measurement given by func. Attempts to tolerate
/// up to 2 outlier measurements by generating 5 datapoints and discarding the 2 with the highest
/// absolute deviation.
pub fn assert_average_percent_error<F: Fn() -> f32>(func: F, expected: f32, threshold: f32) {
    let mut samples: [f32; 5] = Default::default();
    for (i, v) in [func(), func(), func(), func(), func()]
        .into_iter()
        .enumerate()
    {
        if !v.is_finite() {
            panic!("invalid sample ({})", v)
        };
        samples[i] = v;
    }

    let avg = samples.into_iter().sum::<f32>() / 5.;
    samples.sort_by_key(|s| ((s - avg).abs() * 1_000_000.).round().min(u64::MAX as f32) as u64);
    let measured = samples.into_iter().take(3).sum::<f32>() / 3.;

    assert_percent_error(measured, expected, threshold)
}
