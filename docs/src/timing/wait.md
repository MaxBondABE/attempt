# Wait control

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

## `-j --jitter <DURATION>`

For a jitter value of `n`, adds a value in the interval `[0, n]` to the delay time. This is useful
for preventing "thundering herd" issues. Jitter is always added last, so even if the delay was
rounded by `--wait-min/--wait-max`, it will still be randomized.

## `-m --wait-min <DURATION>`

Round any delay smaller than the specified minimum up to that minimum.

## `-M --wait-max <DURATION>`

Round any delay larger than the specified maximum up to that maximum. This is useful when using the
linear or exponential strategies, to ensure that you do not sleep for an unbounded amount of time.

## `--stagger <DURATION>`

Stagger the delay by a random amount in the interval `[0, n]`. This is useful for desyncronizing
multiple concurrent instances of `attempt` which start at the same time.
