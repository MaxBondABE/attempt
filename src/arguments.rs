use std::{process::Command, time::Duration};

use clap::{Args, Parser, Subcommand};

use crate::util::create_duration;

#[derive(Parser, Debug)]
pub(crate) struct ArgumentParser {
    #[clap(subcommand)]
    pub strategy: RetryStrategy,
}

#[derive(Args, Debug)]
pub(crate) struct CommonArguments {
    #[clap(long, short, default_value("3"))]
    pub attempts: usize,
    #[clap(flatten)]
    pub wait_params: WaitParameters,
    pub command: Vec<String>,
}

#[derive(Args, Debug, Clone, Copy)]
pub(crate) struct WaitParameters {
    #[clap(long, short)]
    pub jitter: Option<f64>,
    #[clap(long)]
    pub wait_min: Option<f64>,
    #[clap(long)]
    pub wait_max: Option<f64>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum RetryStrategy {
    Interval {
        #[clap(long, short, default_value("5.0"))]
        wait: f64,

        #[clap(flatten)]
        common: CommonArguments,
    },

    Exponential {
        #[clap(long, short, default_value("2.0"))]
        base: f64,
        #[clap(long, short, default_value("1.0"))]
        multiplier: f64,

        #[clap(flatten)]
        common: CommonArguments,
    },
}
impl RetryStrategy {
    pub fn command(&self) -> Command {
        let command = match self {
            RetryStrategy::Interval { common, .. } => &common.command,
            RetryStrategy::Exponential { common, .. } => &common.command,
        };
        let mut c = Command::new(&command[0]);
        c.args(&command[1..]);

        c
    }
}
impl IntoIterator for RetryStrategy {
    type Item = Duration;
    type IntoIter = Box<dyn Iterator<Item = Duration>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            RetryStrategy::Interval { wait, common } => Box::new(
                (0..common.attempts).map(move |_| create_duration(wait, common.wait_params)),
            ),
            RetryStrategy::Exponential {
                base,
                multiplier,
                common,
            } => Box::new((0..common.attempts).map(move |n| {
                create_duration(multiplier * base.powi(n as i32), common.wait_params)
            })),
        }
    }
}
