use std::{process::Command, time::Duration};

use clap::{Args, Parser, Subcommand};

use crate::util::create_duration;

#[derive(Parser, Debug)]
pub(crate) struct ArgumentParser {
    #[clap(subcommand)]
    pub backoff: BackoffStrategy,
}

#[derive(Args, Debug)]
pub(crate) struct CommonArguments {
    /// The maximum number of attempts.
    #[clap(long, short, default_value("3"))]
    pub attempts: usize,
    #[clap(flatten)]
    pub wait_params: WaitParameters,
    pub command: Vec<String>,
}

#[derive(Args, Debug, Clone, Copy)]
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
