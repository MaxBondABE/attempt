# Retry control

## `-a --attempts`

The number of times the child command is retried.

## `-U --unlimited-attempts`

Continue retrying until the command is successful, without limiting the number of retries.

## `--retry-always`

Always retry, unless a stop predicate is triggered.

## `--forever`

This will continue retrying the command forever, regardless of whether it succeeds or fails. This is
useful for long-running services you wish to restart if they crash.
