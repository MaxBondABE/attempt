# Timeout control

## Specifying durations

Durations in `attempt` can include units, such as `5min`. Durations without units are assumed to be
in seconds.

The following units are supported:

- Hours (`h` or `hr`)
- Minutes (`m` or `min`)
- Seconds (`s`)
- Milliseconds (`ms`)
- Nanoseconds (`ns`)

Multiple units can be used together such as `1hr 30m`.

## `-t --timeout <DURATION>`

Kill the child command if it does not complete within the timeout. This prevents `attempt` from
waiting indefinitely on a child that may never exit. For instance, the child could be stuck in an
infinite loop.

The child is polled using an exponential backoff with a base of 2 and a multiplier of 10ms,
saturating at a maximum delay of 15s.

## `-R --expected-runtime <DURATION>`

Specify how much time the command is expected to take. The child command will be polled slowly
during this time (once per minute).

This is useful to reduce load on the system. An assumption made in the design of the timeout feature
that most commands exit quickly, so the child is polled fairly aggressively. This may adversely
impact performance for some use cases.

## `--retry-if-timeout`

Stop the command if it was killed by a signal. This includes timeouts, since timeouts use signals to
terminate processes.

## `--stop-if-timeout`

Stop the command if it was killed by a signal. This includes timeouts, since timeouts use signals to
terminate processes.
