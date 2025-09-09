# Timeout & signal predicates

## Signal patterns

Some signal predicates attempt to match the signal number against a pattern supplied in the
argument. The syntax for patterns is as follows.

- Individual codes: `1`
- Inclusive ranges: `1..5`
- Combinations of patterns: `1,2,3,10..15`
- Whitespace is allowed: `1, 2, 3`
- Note that valid status codes are in the range [0, 255]

## Retry predicates

### `--retry-if-timeout`

Stop retrying if the command was killed specifically due to a timeout. This requires that the
`--timeout` option is also specified.

### `--retry-if-killed`

Retry if the command was killed by any signal. Note this implies `--retry-if-timeout`, because
timeouts use signals to terminate processes.

### `--retry-if-signal <PATTERN>`

Retrying if the command was killed by any signal matching the given pattern.

This is only available on Unix systems.

## Stop predicates

### `--stop-if-timeout`

Stop retrying if the command was killed specifically due to a timeout. This requires that the
`--timeout` option is also specified.

### `--stop-if-killed`

Stop retrying if the command was killed by any signal. Note this implies `--stop-if-timeout`,
because timeouts use signals to terminate processes.

### `--stop-if-signal <PATTERN>`

Stop retrying if the command was killed by any signal matching the given pattern.

This is only available on Unix systems.
