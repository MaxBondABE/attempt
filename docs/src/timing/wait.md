# Wait control

## `-j --jitter <SECONDS>`

For a jitter value of `n`, adds a value in the interval `[0, n]` to the delay time. This is useful
for preventing "thundering herd" issues. Jitter is always added last, so even if the delay was
rounded by `--wait-min/--wait-max`, it will still be randomized.

## `-m --wait-min <SECONDS>`

Round any delay smaller than the specified minimum up to that minimum.

## `-M --wait-max <SECONDS>`

Round any delay larger than the specified maximum up to that maximum. This is useful when using the
linear or exponential strategies, to ensure that you do not sleep for an unbounded amount of time.
