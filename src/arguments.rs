use std::{
    ffi::OsString,
    io,
    ops::Not,
    process::{Child, Command, ExitStatus},
    time::Duration,
};

use clap::{error::ErrorKind, Args, CommandFactory, Parser, Subcommand};
use rand_distr::{Distribution, Uniform};
use regex::Regex;

use crate::util::{
    status::StatusCodePattern,
    value_parsing::{f32_gte_0, usize_gte_1},
};

/// Hack to get a "default" subcommand to work. See ImplicitSubcommandArguments.
pub fn parse_arguments() -> AttemptArguments {
    // NB: Both parse_arguments functions MUST be kept in sync.
    // This is NOT protected by a test. If you change this function,
    // you must ensure this manually. Otherwise, unit tests cannot be
    // relied upon.
    match AttemptArguments::try_parse() {
        Ok(args) => args,
        Err(e) => {
            if let Ok(args) = ImplicitSubcommandArguments::try_parse() {
                args.into()
            } else {
                e.exit()
            }
        }
    }
}

#[allow(unused)] // testing utility
/// Hack to get a "default" subcommand to work. See ImplicitSubcommandArguments.
pub fn parse_arguments_from<I: IntoIterator<Item = T> + Copy, T: Into<OsString> + Clone>(
    itr: I,
) -> AttemptArguments {
    // NB: Both parse_arguments functions MUST be kept in sync.
    // This is NOT protected by a test. If you change this function,
    // you must ensure this manually. Otherwise, unit tests cannot be
    // relied upon.
    match AttemptArguments::try_parse_from(itr) {
        Ok(args) => args,
        Err(e) => {
            if let Ok(args) = ImplicitSubcommandArguments::try_parse_from(itr) {
                args.into()
            } else {
                e.exit()
            }
        }
    }
}

#[derive(Parser, Debug)]
pub struct AttemptArguments {
    #[command(subcommand)]
    pub strategy: BackoffStrategy,

    #[command(flatten)]
    pub wait_params: WaitParameters,
    #[command(flatten)]
    pub policy_params: PolicyParameters,

    /// The maximum number of attempts.
    #[arg(long, short, default_value_t = 3, global = true, value_parser=usize_gte_1)]
    pub attempts: usize,
    /// Timeout for an individual attempt of the command.
    #[arg(long, short = 't', global = true, value_parser=f32_gte_0, value_name="SECONDS")]
    pub timeout: Option<f32>,
    /// The amount of time the command is expected to take, in seconds. This will not be
    /// counted against it's timeout.
    #[arg(long, short='R', global=true, value_parser=f32_gte_0, value_name="SECONDS")]
    pub expected_runtime: Option<f32>,

    /// Print human-readable messages. Use -vv to show all messages. Don't write scripts against
    /// this output, use exit codes.
    #[arg(long, short, global = true, action=clap::ArgAction::Count)]
    pub verbose: u8,
    /// Suppress human-readable messages. Use -qq to suppress all messages. (Errors in parsing
    /// arguments will still be printed.)
    #[arg(long, short, global = true, action=clap::ArgAction::Count)]
    pub quiet: u8,

    /// Run until the command succeeds, with no limit on the number of attempts.
    #[arg(long, short = 'U', global = true)]
    pub unlimited_attempts: bool,
    /// Always retry the command, and do not limit the number of attempts. Useful for restarting
    /// long-running applications.
    #[arg(long, short = 'Y', global = true)]
    pub forever: bool,
}

impl AttemptArguments {
    pub fn validate(&self) {
        // NB: The command here in the `clap` parlance - NOT the command we are
        // retrying.
        if self.strategy.command().is_empty() {
            let mut clap_cmd = AttemptArguments::command();
            clap_cmd
                .error(ErrorKind::InvalidValue, "No command specified.")
                .exit();
        }
        if self.timeout.is_none() && self.policy_params.stop_if_timeout {
            let mut clap_cmd = AttemptArguments::command();
            clap_cmd
                .error(
                    ErrorKind::InvalidValue,
                    "--stop-if-timeout requires --timeout.",
                )
                .exit();
        }
        if self.timeout.is_none() && self.expected_runtime.is_some() {
            let mut clap_cmd = AttemptArguments::command();
            clap_cmd
                .error(
                    ErrorKind::InvalidValue,
                    "--expected-runtime requires --timeout.",
                )
                .exit();
        }
    }
    pub fn backoff(&self) -> BackoffIter {
        let unlimited_attempts = self.unlimited_attempts || self.forever;
        BackoffIter {
            params: self.strategy.params(),
            attempts: unlimited_attempts.not().then_some(self.attempts),
            wait_params: self.wait_params,
        }
    }
    pub fn build_command(&self) -> Command {
        let command = self.strategy.command();
        let mut c = Command::new(&command[0]);
        c.args(&command[1..]);

        c
    }
    pub fn evaluate_policy(
        &self,
        child: Child,
        timed_out: bool,
    ) -> Result<(bool, ExitStatus), io::Error> {
        self.policy_params
            .evaluate_policy(child, timed_out, self.forever)
    }
}
impl Default for AttemptArguments {
    fn default() -> Self {
        AttemptArguments::parse_from(["attempt", "fixed", "--wait", "1"])
    }
}

// Hack to get an implicit subcommand to work.
// - We are abusing the subcommand feature of clap to implement a "mode"
//  - Rather than branching between different behaviors, we are doing the same
//    behavior differently.
// - To implicitly used Fixed if no subcommand is specified, we need to accept
//  `command` as an argument.
// - However, subcommands _always_ go last in usage documentation.
// - Because `command` is a list, it _has_ to go last.
// - This means clap will output confusing/wrong usage documentation.
// - Hack: Push `command` down into the subcommands, write _two_ parsers,
//   and fall back to the implicit behavior if the normal, "explicit" subcommand
//   fails.
// NB: We never show help from this parser.
#[derive(Parser, Debug)]
pub struct ImplicitSubcommandArguments {
    #[command(flatten)]
    pub wait_params: WaitParameters,
    #[command(flatten)]
    pub policy_params: PolicyParameters,

    #[arg(long, short, default_value_t = 1.0, value_parser=f32_gte_0, hide = true)]
    wait: f32,

    #[arg(long, short, default_value_t = 3, global = true, value_parser=usize_gte_1)]
    pub attempts: usize,
    #[arg(long, short = 't', global = true, value_parser=f32_gte_0, value_name="SECONDS")]
    pub timeout: Option<f32>,
    #[arg(long, short='R', global=true, value_parser=f32_gte_0, value_name="SECONDS")]
    pub expected_runtime: Option<f32>,

    #[arg(long, short, global = true, action=clap::ArgAction::Count)]
    pub verbose: u8,
    #[arg(long, short, global = true, action=clap::ArgAction::Count)]
    pub quiet: u8,

    #[arg(long, short = 'U', global = true)]
    pub unlimited_attempts: bool,
    #[arg(long, short = 'Y', global = true)]
    pub forever: bool,

    #[arg(global = true)]
    pub command: Vec<String>,
}
impl From<ImplicitSubcommandArguments> for AttemptArguments {
    fn from(value: ImplicitSubcommandArguments) -> Self {
        let ImplicitSubcommandArguments {
            wait_params,
            policy_params,
            wait,
            attempts,
            timeout,
            expected_runtime,
            verbose,
            quiet,
            unlimited_attempts,
            forever,
            command,
        } = value;
        let strategy = BackoffStrategy::Fixed { wait, command };

        Self {
            strategy,
            wait_params,
            policy_params,
            attempts,
            timeout,
            expected_runtime,
            verbose,
            quiet,
            unlimited_attempts,
            forever,
        }
    }
}

#[derive(Args, Debug, Clone, Copy, Default)]
pub struct WaitParameters {
    /// Add random jitter to the wait time, in the interval [-n/2, n/2].
    #[arg(long, short, global = true, value_parser=f32_gte_0, value_name="SECONDS")]
    pub jitter: Option<f32>,
    /// The minimum amount of time to wait between attempts.
    #[arg(long, short = 'm', global = true, value_parser=f32_gte_0, value_name="SECONDS")]
    pub wait_min: Option<f32>,
    /// The maximum amount of time to wait between attempts.
    #[arg(long, short = 'M', global = true, value_parser=f32_gte_0, value_name="SECONDS")]
    pub wait_max: Option<f32>,
}

impl WaitParameters {
    pub fn create_duration(&self, interval: f32) -> Duration {
        let jitter_seconds = match self.jitter {
            Some(n) => Uniform::new_inclusive(-n / 2., n / 2.).sample(&mut rand::thread_rng()),
            None => 0.0,
        };
        Duration::from_secs_f32(
            (interval + jitter_seconds)
                .max(self.wait_min.unwrap_or(0.0))
                .min(self.wait_max.unwrap_or(f32::MAX)),
        )
    }
}

#[derive(Args, Debug, Default)]
pub struct PolicyParameters {
    /// Retry if the program exists with a code other than zero
    #[arg(long, short = 'F', global = true)]
    pub retry_failing_status: bool,

    /// Retry if the program exits with this code, or a pattern
    /// consisting of a range of codes (eg `1..5`), a series of codes
    /// (eg `1,2,3`), or a combination (eg `1..5,10,15..20`).
    #[arg(long, short = 'S', value_name = "STATUS_CODE", global = true)]
    pub retry_if_status: Option<StatusCodePattern>,

    /// Retry if the program's output contains this string
    #[arg(long, short = 's', value_name = "STRING", global = true)]
    pub retry_if_contains: Option<String>,

    /// Retry if the program's output matches this regex
    #[arg(long, short = 'r', value_name = "REGEX", global = true)]
    pub retry_if_matches: Option<Regex>,

    /// Retry if the program's stdout contains this string
    #[arg(long, value_name = "STRING", global = true)]
    pub retry_if_stdout_contains: Option<String>,

    /// Retry if the program's stdout matches this regex
    #[arg(long, value_name = "REGEX", global = true)]
    pub retry_if_stdout_matches: Option<Regex>,

    /// Retry if the program's stderr contains this string
    #[arg(long, value_name = "STRING", global = true)]
    pub retry_if_stderr_contains: Option<String>,

    /// Retry if the program's stderr matches this regex
    #[arg(long, value_name = "REGEX", global = true)]
    pub retry_if_stderr_matches: Option<Regex>,

    /// Always retry the command, whether it succeeded or failed.
    #[arg(long, global = true)]
    pub retry_always: bool,

    /// Stop retrying if the program exits with this code or pattern
    #[arg(long, value_name = "STATUS_CODE", global = true)]
    pub stop_if_status: Option<StatusCodePattern>,

    /// Stop retrying if the program's output contains this string
    #[arg(long, value_name = "STRING", global = true)]
    pub stop_if_contains: Option<String>,

    /// Stop retrying if the program's output matches this regex
    #[arg(long, value_name = "REGEX", global = true)]
    pub stop_if_matches: Option<Regex>,

    /// Stop retrying if the program's stdout contains this string
    #[arg(long, value_name = "STRING", global = true)]
    pub stop_if_stdout_contains: Option<String>,

    /// Stop retrying if the program's stdout matches this regex
    #[arg(long, value_name = "REGEX", global = true)]
    pub stop_if_stdout_matches: Option<Regex>,

    /// Stop retrying if the program's stderr contains this string
    #[arg(long, value_name = "STRING", global = true)]
    pub stop_if_stderr_contains: Option<String>,

    /// Stop retrying if the program's stderr matches this regex
    #[arg(long, value_name = "REGEX", global = true)]
    pub stop_if_stderr_matches: Option<Regex>,

    /// Stop retrying if the program was killed by a signal. Note that this
    /// implies --stop-if-timeout, because timed-out commands will be killed.
    #[arg(long, global = true)]
    pub stop_if_killed: bool,

    /// Stop retrying if the command has timed out
    #[arg(long, global = true)]
    pub stop_if_timeout: bool,
}

impl PolicyParameters {
    pub fn default_behavior(&self) -> bool {
        // NB: This is not protected by a test, it must be manually verified if changed
        self.retry_if_status.is_none()
            && self.retry_if_contains.is_none()
            && self.retry_if_matches.is_none()
            && self.retry_if_stdout_contains.is_none()
            && self.retry_if_stdout_matches.is_none()
            && self.retry_if_stderr_contains.is_none()
            && self.retry_if_stderr_matches.is_none()
            && self.stop_if_status.is_none()
            && self.stop_if_contains.is_none()
            && self.stop_if_matches.is_none()
            && self.stop_if_stdout_contains.is_none()
            && self.stop_if_stdout_matches.is_none()
            && self.stop_if_stderr_contains.is_none()
            && self.stop_if_stderr_matches.is_none()
            && !self.stop_if_killed
            && !self.stop_if_timeout
            && !self.retry_failing_status
            && !self.retry_always
    }
}

#[derive(Subcommand, Clone, Debug)]
#[command(
    subcommand_value_name = "STRATEGY",
    subcommand_help_heading = "Backoff Strategies",
    disable_help_subcommand = true
)]
pub enum BackoffStrategy {
    /// Wait a fixed amount of time between attempts (this is the default).
    Fixed {
        /// The amount of time to wait between attempts.
        // NB: Keep in sync with duplicate in ImplicitSubcommandArguments
        #[arg(long, short, default_value_t = 1.0, value_parser=f32_gte_0)]
        wait: f32,

        /// The command to be attempted. Using `--` to disambiguate arguments between `attempt` and
        /// the child command is recommended.
        #[arg(global = true)]
        command: Vec<String>,
    },

    /// Wait exponentially longer between attempts, using the formula
    /// <multiplier> * <base> ^ <attempts>.
    #[command(alias = "exp")]
    Exponential {
        #[arg(long, short, default_value_t = 2.0, value_parser=f32_gte_0)]
        base: f32,
        #[arg(long, default_value_t = 1.0, short = 'x', value_parser=f32_gte_0)]
        multiplier: f32,

        /// The command to be attempted. Using `--` to disambiguate arguments between `attempt` and
        /// the child command is recommended.
        #[arg(global = true)]
        command: Vec<String>,
    },

    /// Wait linearly longer between attempts, using the formula
    /// <multiplier> * <attempts> + <starting_wait>.
    Linear {
        #[arg(long, default_value_t = 1.0, short = 'x', value_parser=f32_gte_0)]
        multiplier: f32,
        #[arg(long, short='W', default_value_t = 1.0, value_parser=f32_gte_0)]
        starting_wait: f32,

        /// The command to be attempted. Using `--` to disambiguate arguments between `attempt` and
        /// the child command is recommended.
        #[arg(global = true)]
        command: Vec<String>,
    },
}

impl BackoffStrategy {
    fn params(&self) -> BackoffParameters {
        match self {
            BackoffStrategy::Fixed { wait, .. } => BackoffParameters::Fixed { wait: *wait },
            BackoffStrategy::Exponential {
                base, multiplier, ..
            } => BackoffParameters::Exponential {
                base: *base,
                multiplier: *multiplier,
            },
            BackoffStrategy::Linear {
                multiplier,
                starting_wait,
                ..
            } => BackoffParameters::Linear {
                multiplier: *multiplier,
                starting_wait: *starting_wait,
            },
        }
    }
    fn command(&self) -> &Vec<String> {
        match self {
            BackoffStrategy::Fixed { command, .. } => command,
            BackoffStrategy::Exponential { command, .. } => command,
            BackoffStrategy::Linear { command, .. } => command,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum BackoffParameters {
    Fixed { wait: f32 },
    Exponential { base: f32, multiplier: f32 },
    Linear { multiplier: f32, starting_wait: f32 },
}

#[derive(Copy, Clone, Debug)]
pub struct BackoffIter {
    params: BackoffParameters,
    attempts: Option<usize>,
    wait_params: WaitParameters,
}

impl IntoIterator for BackoffIter {
    type Item = (Duration, bool);
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        match self.params {
            BackoffParameters::Fixed { wait } => {
                if let Some(attempts) = self.attempts {
                    let last = attempts - 1;
                    Box::new(
                        (0..attempts)
                            .map(move |n| (self.wait_params.create_duration(wait), n >= last)),
                    )
                } else {
                    Box::new((0..).map(move |_| (self.wait_params.create_duration(wait), false)))
                }
            }
            BackoffParameters::Exponential { base, multiplier } => {
                if let Some(attempts) = self.attempts {
                    let last = attempts - 1;
                    Box::new((0..attempts).map(move |n| {
                        (
                            self.wait_params
                                .create_duration(multiplier * base.powi(n as i32)),
                            n >= last,
                        )
                    }))
                } else {
                    Box::new((0..).map(move |n| {
                        (
                            self.wait_params.create_duration(multiplier * base.powi(n)),
                            false,
                        )
                    }))
                }
            }
            BackoffParameters::Linear {
                multiplier,
                starting_wait,
            } => {
                if let Some(attempts) = self.attempts {
                    let last = attempts - 1;
                    Box::new((0..attempts).map(move |n| {
                        (
                            self.wait_params
                                .create_duration(multiplier * n as f32 + starting_wait),
                            n >= last,
                        )
                    }))
                } else {
                    Box::new((0..).map(move |n| {
                        (
                            self.wait_params
                                .create_duration(multiplier * n as f32 + starting_wait),
                            false,
                        )
                    }))
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fixed() {
        let fixed_args = AttemptArguments {
            strategy: BackoffStrategy::Fixed {
                wait: 1.0,
                command: vec![],
            },
            attempts: 3,
            ..Default::default()
        };

        let durations = fixed_args.backoff().into_iter().collect::<Vec<_>>();
        assert_eq!(durations.len(), 3);
        for (duration, _) in durations {
            assert_eq!(duration, Duration::from_secs(1))
        }
    }

    #[test]
    fn fixed_with_jitter() {
        let fixed_args = AttemptArguments {
            strategy: BackoffStrategy::Fixed {
                wait: 5.0,
                command: vec![],
            },
            attempts: 3,
            wait_params: WaitParameters {
                jitter: Some(1.0),
                ..Default::default()
            },
            ..Default::default()
        };

        let durations = fixed_args.backoff().into_iter().collect::<Vec<_>>();
        assert_eq!(durations.len(), 3);
        for (duration, _) in durations {
            assert!(duration >= Duration::from_secs(4) && duration <= Duration::from_secs(6))
        }
    }

    #[test]
    fn linear() {
        let multiplier = 2.;
        let starting_wait = 1.;
        let linear_args = AttemptArguments {
            strategy: BackoffStrategy::Linear {
                multiplier,
                starting_wait,
                command: vec![],
            },
            attempts: 3,
            ..Default::default()
        };

        let durations = linear_args
            .backoff()
            .into_iter()
            .map(|(duration, _)| duration)
            .collect::<Vec<_>>();
        assert_eq!(durations.len(), 3);
        assert_eq!(durations[0], Duration::from_secs_f32(starting_wait));
        assert_eq!(
            durations[1],
            Duration::from_secs_f32(1. * multiplier + starting_wait)
        );
        assert_eq!(
            durations[2],
            Duration::from_secs_f32(2. * multiplier + starting_wait)
        );
    }

    #[test]
    fn exponential() {
        // Test base
        let exp_args = AttemptArguments {
            strategy: BackoffStrategy::Exponential {
                base: 2.0,
                multiplier: 1.0,
                command: vec![],
            },
            ..Default::default()
        };

        let durations = exp_args
            .backoff()
            .into_iter()
            .map(|(duration, _)| duration)
            .collect::<Vec<_>>();
        assert_eq!(durations.len(), 3);
        assert_eq!(durations[0], Duration::from_secs(1));
        assert_eq!(durations[1], Duration::from_secs(2));
        assert_eq!(durations[2], Duration::from_secs(4));

        // Test multiplier
        let exp_args = AttemptArguments {
            strategy: BackoffStrategy::Exponential {
                base: 2.0,
                multiplier: 2.0,
                command: vec![],
            },
            ..Default::default()
        };

        let durations = exp_args
            .backoff()
            .into_iter()
            .map(|(duration, _)| duration)
            .collect::<Vec<_>>();
        assert_eq!(durations.len(), 3);
        assert_eq!(durations[0], Duration::from_secs(2));
        assert_eq!(durations[1], Duration::from_secs(4));
        assert_eq!(durations[2], Duration::from_secs(8));
    }

    #[test]
    fn exponential_with_jitter() {
        let exp_args = AttemptArguments {
            strategy: BackoffStrategy::Exponential {
                base: 2.0,
                multiplier: 1.0,
                command: vec![],
            },
            wait_params: WaitParameters {
                jitter: Some(1.0),
                ..Default::default()
            },
            ..Default::default()
        };
        let durations = exp_args
            .backoff()
            .into_iter()
            .map(|(duration, _)| duration)
            .collect::<Vec<_>>();
        assert_eq!(durations.len(), 3);
        assert!(durations[0] >= Duration::from_secs(0) && durations[0] <= Duration::from_secs(2));
        assert!(durations[1] >= Duration::from_secs(1) && durations[1] <= Duration::from_secs(3));
        assert!(durations[2] >= Duration::from_secs(3) && durations[2] <= Duration::from_secs(5));
    }

    #[test]
    fn fixed_is_default() {
        let fixed_args = AttemptArguments {
            strategy: BackoffStrategy::Fixed {
                wait: 1.0,
                command: vec![],
            },
            ..Default::default()
        };
        let fixed_durations = fixed_args.backoff().into_iter().collect::<Vec<_>>();

        let default_args = parse_arguments_from(["/bin/true"]);
        let default_durations = default_args.backoff().into_iter().collect::<Vec<_>>();

        assert_eq!(fixed_durations, default_durations);
    }

    #[test]
    fn forever_and_unlimited_override_attempts() {
        let args = AttemptArguments {
            attempts: 1,
            forever: true,
            ..Default::default()
        };
        let mut backoff = args.backoff().into_iter();
        assert!(backoff.next().is_some());
        assert!(backoff.next().is_some());

        let args = AttemptArguments {
            attempts: 1,
            unlimited_attempts: true,
            ..Default::default()
        };
        let mut backoff = args.backoff().into_iter();
        assert!(backoff.next().is_some());
        assert!(backoff.next().is_some());
    }

    #[test]
    fn min_wait_is_respected() {
        let params = WaitParameters {
            wait_min: Some(5.0),
            ..Default::default()
        };
        assert_eq!(params.create_duration(1.0), Duration::from_secs(5));
    }

    #[test]
    fn max_wait_is_respected() {
        let params = WaitParameters {
            wait_max: Some(5.0),
            ..Default::default()
        };
        assert_eq!(params.create_duration(10.0), Duration::from_secs(5));
    }

    #[test]
    fn jitter() {
        let params = WaitParameters {
            jitter: Some(1.0),
            ..Default::default()
        };
        let outputs = (0..3)
            .map(|_| params.create_duration(10.0))
            .collect::<Vec<_>>();
        assert!(outputs.iter().any(|n| n.as_secs_f32() != 10.0));
        assert!(outputs
            .iter()
            .all(|n| n.as_secs_f32() >= 9.0 && n.as_secs_f32() <= 11.0));
    }

    #[test]
    fn jitter_with_min_max() {
        let params = WaitParameters {
            jitter: Some(5.0),
            wait_min: Some(0.5),
            wait_max: Some(3.0),
        };
        let outputs = (0..3)
            .map(|_| params.create_duration(1.0))
            .collect::<Vec<_>>();
        assert!(outputs
            .iter()
            .all(|n| n.as_secs_f32() >= 0.5 && n.as_secs_f32() <= 3.0));
    }
}
