# Advice for Scripting

## Exit Codes

The following exit codes are used by `attempt` to indicate whether it failed and
how. Scripts should use these exit codes and not the log messages exposed by
`--verbose`; these exit codes will remain stable, but no such guarantee is made
for the log messages.

0 - Command was run successfully within the allowed number of retries

1 - I/O error (eg, command not found). An error message will be printed.

2 - Invalid arguments. An error message will be printed.

3 - The number of retries has been exhausted without the command ever succeeding

4 - The number of retries has not been exhausted, but the command is no longer
    retryable because of a "stop" predicate.

101 - `attempt` has crashed. The most likely cause is using output predicates on
    data which is not UTF-8 encoded.

## Best practices

### Use `exponential`

[Exponential backoff](https://en.wikipedia.org/wiki/Exponential_backoff) allows you to retry aggressively
at first while quickly backing off to a significant wait time. Many errors are transient, and worked around
by retrying quickly. Other errors may take a long time to resolve, and
[may not resolve](https://en.wikipedia.org/wiki/Thundering_herd_problem) if we stress the system with load.

Leave the `base` argument as it's default of 2, and use the `multiplier` argument to control how aggressively
you retry. If you want it to be very aggressive, use a value of `0.050` (50 milliseconds), and if you want
it to be very conservative, use a value of `60` (1 minute). The default of `1` second is a good balance overall,
but if you are accessing public resources, use a value of `5` seconds or greater.

### Use `--wait-max`

Set max wait to a reasonable value, like `900` (15 minutes), so that you do not end up waiting for an
excessive amount of time.

### Use `--jitter`

Random jitter will help you avoid emergent cyclic behavior. This may occur if your script is running on
multiple systems concurrently, or if you are accessing a public resource and many programmers chose similar
constants for their retry logic (eg, if everyone chooses round numbers, then they all will be multiples
of 5, and at some point everyone's retry logic will sync up and make a request at the exact same time).
A useful metaphor is [metronome synchronization](https://www.youtube.com/watch?v=T58lGKREubo).

See also the [thundering herd problem](https://en.wikipedia.org/wiki/Thundering_herd_problem).

### Avoid output predicates

Output predicates can create performance issues. Try to use status predicates whenever possible.

If you must use an output predicate, use the specific `stdout` or `stderr` variant. The generic
variants are provided for convenience, but are not as performant.
