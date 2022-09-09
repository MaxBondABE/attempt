use std::{process::Command, time::Duration};

use clap::{Args, Parser, Subcommand};

use crate::util::create_duration;

#[derive(Parser, Debug)]
pub(crate) struct ArgumentParser {
    #[clap(subcommand)]
    pub backoff: BackoffStrategy,
}

impl ArgumentParser {
    #[allow(unused)]
    pub(crate) fn new(backoff: BackoffStrategy) -> Self {
        Self { backoff }
    }
}

#[derive(Args, Debug, Default)]
pub(crate) struct CommonArguments {
    /// The maximum number of attempts.
    #[clap(long, short, default_value("3"))]
    pub attempts: usize,
    #[clap(flatten)]
    pub wait_params: WaitParameters,
    pub command: Vec<String>,
}

impl CommonArguments {
    #[allow(unused)]
    pub(crate) fn new(attempts: usize, wait_params: WaitParameters, command: Vec<String>) -> Self {
        Self {
            attempts,
            wait_params,
            command,
        }
    }
}

#[derive(Args, Debug, Clone, Copy, Default)]
pub(crate) struct WaitParameters {
    /// Add random jitter to the wait time, in the interval [-n, n].
    #[clap(long, short)]
    pub jitter: Option<f64>,
    /// The minimum amount of time to wait between attempts.
    #[clap(long)]
    pub wait_min: Option<f64>,
    /// The maximum amount of time to wait between attempts.
    #[clap(long)]
    pub wait_max: Option<f64>,
}

impl WaitParameters {
    #[allow(unused)]
    pub(crate) fn new(jitter: Option<f64>, wait_min: Option<f64>, wait_max: Option<f64>) -> Self {
        Self {
            jitter,
            wait_min,
            wait_max,
        }
    }
}

#[derive(Subcommand, Debug)]
pub(crate) enum BackoffStrategy {
    /// Wait a fixed amount of time between attempts.
    Fixed {
        /// The amount of time to wait between attempts.
        #[clap(long, short, default_value("5.0"))]
        wait: f64,

        #[clap(flatten)]
        common: CommonArguments,
    },

    /// Wait exponentially longer between attempts.
    Exponential {
        #[clap(long, short, default_value("2.0"))]
        base: f64,
        #[clap(long, short, default_value("1.0"))]
        multiplier: f64,

        #[clap(flatten)]
        common: CommonArguments,
    },
}
impl BackoffStrategy {
    pub fn command(&self) -> Command {
        let command = match self {
            BackoffStrategy::Fixed { common, .. } => &common.command,
            BackoffStrategy::Exponential { common, .. } => &common.command,
        };
        let mut c = Command::new(&command[0]);
        c.args(&command[1..]);

        c
    }
}
impl IntoIterator for BackoffStrategy {
    type Item = Duration;
    type IntoIter = Box<dyn Iterator<Item = Duration>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            BackoffStrategy::Fixed { wait, common } => Box::new(
                (0..common.attempts).map(move |_| create_duration(wait, common.wait_params)),
            ),
            BackoffStrategy::Exponential {
                base,
                multiplier,
                common,
            } => Box::new((0..common.attempts).map(move |n| {
                create_duration(multiplier * base.powi(n as i32), common.wait_params)
            })),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fixed() {
        let fixed_args = ArgumentParser::new(BackoffStrategy::Fixed {
            wait: 1.0,
            common: CommonArguments::new(3, WaitParameters::default(), Vec::default()),
        });
        let durations = fixed_args.backoff.into_iter().collect::<Vec<_>>();
        assert_eq!(durations.len(), 3);
        for duration in durations {
            assert_eq!(duration, Duration::from_secs(1))
        }
    }

    #[test]
    fn test_fixed_with_jitter() {
        let fixed_args = ArgumentParser::new(BackoffStrategy::Fixed {
            wait: 5.0,
            common: CommonArguments::new(
                3,
                WaitParameters::new(Some(1.0), None, None),
                Vec::default(),
            ),
        });
        let durations = fixed_args.backoff.into_iter().collect::<Vec<_>>();
        assert_eq!(durations.len(), 3);
        for duration in durations {
            assert!(duration >= Duration::from_secs(4) && duration <= Duration::from_secs(6))
        }
    }

    #[test]
    fn test_exponential() {
        // Test base
        let exp_args = ArgumentParser::new(BackoffStrategy::Exponential {
            base: 2.0,
            multiplier: 1.0,
            common: CommonArguments::new(3, WaitParameters::default(), Vec::default()),
        });
        let durations = exp_args.backoff.into_iter().collect::<Vec<_>>();
        assert_eq!(durations.len(), 3);
        assert_eq!(durations[0], Duration::from_secs(1));
        assert_eq!(durations[1], Duration::from_secs(2));
        assert_eq!(durations[2], Duration::from_secs(4));

        // Test multiplier
        let exp_args = ArgumentParser::new(BackoffStrategy::Exponential {
            base: 2.0,
            multiplier: 2.0,
            common: CommonArguments::new(3, WaitParameters::default(), Vec::default()),
        });
        let durations = exp_args.backoff.into_iter().collect::<Vec<_>>();
        assert_eq!(durations.len(), 3);
        assert_eq!(durations[0], Duration::from_secs(2));
        assert_eq!(durations[1], Duration::from_secs(4));
        assert_eq!(durations[2], Duration::from_secs(8));
    }

    #[test]
    fn test_exponential_with_jitter() {
        let exp_args = ArgumentParser::new(BackoffStrategy::Exponential {
            base: 2.0,
            multiplier: 1.0,
            common: CommonArguments::new(
                3,
                WaitParameters::new(Some(1.0), None, None),
                Vec::default(),
            ),
        });
        let durations = exp_args.backoff.into_iter().collect::<Vec<_>>();
        assert_eq!(durations.len(), 3);
        assert!(durations[0] >= Duration::from_secs(0) && durations[0] <= Duration::from_secs(2));
        assert!(durations[1] >= Duration::from_secs(1) && durations[1] <= Duration::from_secs(3));
        assert!(durations[2] >= Duration::from_secs(3) && durations[2] <= Duration::from_secs(5));
    }
}
