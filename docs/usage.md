Usage
-----

`attempt [STRATEGY] [OPTIONS] [COMMAND]...`

## Examples

```bash
# Basic example
attempt /bin/false
attempt --verbose /bin/false
```

```bash
# Change the number of attempts from the default of 3 to 10
attempt -a 10 /bin/false
attempt --attempts 10 /bin/false
```

```bash
# Rerun database migrations if the server was not ready
# Useful for `docker-compose` and similar tools (any place
# you'd use `wait-for-it.sh` & aren't restricted to bash).
attempt --retry-if-contains "server not ready" sqlx migrate
```

```bash
# Use `--` to disambiguate between arguments to `attempt` and arguments to
# it's child command
attempt -a 10 -- foo -a bar
```

# Backoff Strategies

The backoff strategy determines how long to wait between attempts. You can also
add random jitter to the wait time (`-j, --jitter`), and constrain the
minimum or maximum wait time (`--wait-min/--wait-max`).

## Fixed Delay

This is the default strategy. The wait time does not change between attempts.

```bash
attempt /bin/false
attempt fixed /bin/false

# Change the wait time from the default of 1 second to 15 seconds
attempt fixed -w 15 /bin/false
attempt fixed --wait 15 /bin/false
```

## Exponential Backoff

Wait exponentially more time between attempts, using the formula <multiplier> * <base> ^ <attempts>. The attempt
counter starts at 0, so the first wait is is for <multiplier> seconds.

```bash
attempt exponential /bin/false
attempt exp /bin/false

# Change the multiplier from the default of 1 to 2
attempt exponential -x 2 /bin/false
attempt exponential --multiplier 2 /bin/false

# Change the exponential base from the default of 2 to 5
attempt exponential -x 5 /bin/false
attempt exponential --base 5 /bin/false
```

## Linear Backoff

Wait more time between attempts, using the formula <multiplier> * <attempts> + <starting_wait>. The attempt
counter starts at 0, so the first wait is is for <starting_wait> seconds.

```bash
attempt linear /bin/false

# Change the multiplier from the default of 1 to 2
attempt linear -m 2 /bin/false
attempt linear --multiplier 2 /bin/false

# Change the starting wait from the default of 1 to 5
attempt linear -s 5 /bin/false
attempt linear --starting-wait 5 /bin/false
```

# Predicates

Predicates control the circumstances under which `attempt` will retry the child
command. Most predicates come with a "retry" and "stop" variant, with the stop
predicates beginning with `--stop-*`. Retry predicates cause a command to be retried,
while stop predicates cause `attempt` to terminate.

Stop predicates always have precedence over retry predicates.

## Default behavior

If no retry predicates are specified, `attempt` will retry a command if it's exit code
isn't equal to 0, or if the command was killed by a [signal](https://en.wikipedia.org/wiki/Signal_(IPC)).

If retry predicates are specified, than `attempt` will not 

### `--retry-always`

Invert `attempt`'s behavior from not retrying by default to retrying by default. You can then use
stop

## Status Predicates

### `-F, --retry-failing-status`

Retry the command if the status code is not equal to 0. This is useful if you'd like to retry
on failing status codes, but you would also like to use another predicate.

### `--retry-if-status <STATUS_CODE>`
### `--stop-if-status <STATUS_CODE>`

Retry or stop the command if it's exit status matches the pattern provided.

Patterns can be composed of:
- Individual codes (ex: `1`)
- A range of codes, specified by two dots (ex: `1..10`)
- Any combination of the previous, seperated by commas (ex: `1,2,3,10..15`)
- Whitespace is allowed between codes (ex: `1, 2, 3`)

## Output Predicates

### `--retry-if-contains <STRING>`
### `--retry-if-matches <REGEX>`
### `--retry-if-stdout-contains <STRING>`
### `--retry-if-stdout-matches <REGEX>`
### `--retry-if-stderr-contains <STRING>`
### `--retry-if-stderr-matches <REGEX>`

### `--stop-if-contains <STRING>`
### `--stop-if-matches <REGEX>`
### `--stop-if-stdout-contains <STRING>`
### `--stop-if-stdout-matches <REGEX>`
### `--stop-if-stderr-contains <STRING>`
### `--stop-if-stderr-matches <REGEX>`

Retry or stop the command if it's output contains a string or matches a regex. The generic version
of these will check both `stdout` and `stderr`, and the more specific versions will only check
the given file descriptor.

These predicates could be the cause of performance issues if the child program outputs a large
amount of text or if the regexes are especially complex. Status code predicates should be
preferred where possible.

See [here](https://docs.rs/regex/latest/regex/#syntax) for documentation of the supported regex
syntax.

## Timeout & signal control

### `--stop-if-timeout`

Don't retry the command if it is killed by a timeout.

### `--stop-if-killed`

Don't retry the command if it is killed be a signal. Because timeouts use
signals, this essentially implies `--stop-if-timeout`.

# Other arguments

### `-a --attempts`

Control the number of times the child command is retried.

### `-U --unlimited-attempts`

This will continue retrying until the command is successful, without limiting
the number of retries.

### `--forever`

This will continue retrying the command forever, regardless of whether it
succeeds or fails. This is useful for services you wish to always to available,
so that they are restarted if they crash.

## Output

### `-v --verbose`

Print additional information to aide in debugging.

```bash
attempt --verbose /bin/false
Command has failed, retrying in 1 seconds...
Command has failed, retrying in 1 seconds...
Terminated: Retries exhausted.
```

## Wait control

### `-j --jitter <SECONDS>`

### `-m --wait-min <SECONDS>`

### `-M --wait-max <SECONDS>`

## Timeout control

### `-t --timeout <SECONDS>`

Kill the child command if it does not complete within the timeout.

### `-9 --force-kill`

Use SIGKILL instead of SIGTERM for a job which has timed out (equivalent to `kill -9`).

### `--stop-if-timeout`

A stop predicate preventing retries in the event of a timeout.

# Scripting

## Exit Codes

The following exit codes are used by `attempt` to indicate whether it failed and
how. Don't write scripts against `attempt`'s output under `--verbose`, use these
exit codes.

0 - Command was run successfully within the allowed number of retries
1 - I/O error (eg, command not found). An error message will be printed.
2 - Invalid arguments. An error message will be printed.
3 - The number of retries has been exhausted without the command ever succeeding
4 - The number of retries has not been exhausted, but the command is no longer
    retryable because of a "no" predicate
101 - `attempt` has crashed

## Best practices

### Use `exponential`

[Exponential backoff](https://en.wikipedia.org/wiki/Exponential_backoff) allows you to retry aggressively
at first while quickly backing off to a significant wait time. This works well because (temporary) errors
are roughly bimodal; many are solved immediately, and the rest take quite a long time.

Don't touch the `base` argument, and use the `multiplier` argument to control how aggressively you retry. Eg,
if you want it to be very aggressive, use a value of `0.050` (50 milliseconds), and if you want it to be very
conservative, use a value of `60` (1 minute). The default of 1 second is a good balance overall, but if
you are accessing public resources, I personally would consider a value of less than 5 seconds to be
rude.

### Use `--wait-max`

Set max wait to a reasonable value, like `900` (15 minutes), so that you do not end up waiting for an
excessive amount of time.

### Use `--jitter`

Random jitter will help you avoid emergent cyclic behavior. This may occur if your script is running on
multiple systems concurrently, or if you are accessing a public resource and many programmers chose similar
constants for their retry logic (eg, if everyone chooses round numbers, then they all will be multiples
of 5, and at some point everyone's retry logic will sync up and make a request at the exact same time). This
helps to mitigate [thundering herd problems](https://en.wikipedia.org/wiki/Thundering_herd_problem).

### Avoid output predicates

Output predicates can create performance issues. Try to use status predicates whenever possible.

If you must use an output predicate, use the specific `stdout` or `stderr` variant. The generic
variants are provided for convinience, but are not as performant.
