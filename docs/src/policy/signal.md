# Timeout & signal control

Signal and timeout predicates control how `attempt` handles commands that are terminated by signals
or exceed time limits. These predicates are particularly useful for managing long-running processes
and handling system-level interruptions.

## Understanding signals and timeouts

When a process is killed by a signal (such as SIGTERM, SIGKILL, or SIGINT), it does not exit with a
normal status code. Instead, the process is terminated externally. Timeouts in `attempt` work by
sending signals to child processes that exceed their time limit.

## Retry predicates

### `--stop-if-killed`

Retry the command if it was killed by a signal. This includes timeouts, since timeouts use signals
to terminate processes.

## Stop predicates

### `--stop-if-killed`

Stop retrying if the command was killed by any signal. This includes timeouts, since timeouts use
signals to terminate processes.

When this option is enabled, any command that is terminated by a signal will not be retried,
regardless of what caused the signal.

### `--stop-if-timeout`

Stop retrying if the command was killed specifically due to a timeout. This requires that the
`--timeout` option is also specified.

This is more specific than `--stop-if-killed` - it only prevents retries when the termination was
caused by `attempt`'s timeout mechanism, not other signals.

## Default behavior

By default, `attempt` will retry commands that are killed by signals, treating them the same as
commands that exit with non-zero status codes. This behavior can be changed using the predicates
above.

## Relationship between predicates

Note that `--stop-if-killed` implies `--stop-if-timeout`, since timeouts work by sending signals to
terminate processes. If you use `--stop-if-killed`, you don't need to also specify
`--stop-if-timeout`.
