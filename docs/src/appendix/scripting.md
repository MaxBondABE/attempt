# Advice for Scripting

## Use an exponential backoff

[Exponential backoff](https://en.wikipedia.org/wiki/Exponential_backoff) allows you to retry
aggressively at first while quickly backing off to a significant wait time. Many errors are
transient, and worked around by retrying quickly. Other errors may take a long time to resolve, and
[may not resolve](https://en.wikipedia.org/wiki/Thundering_herd_problem) if we stress the system
with load.

Leave the `base` argument as it's default of 2, and use the `multiplier` argument to control how
aggressively you retry. If you want it to be very aggressive, use a value of `0.050` (50
milliseconds), and if you want it to be very conservative, use a value of `60` (1 minute). The
default of 1 second is a good balance overall, but if you are accessing public resources, consider
using a value of 5 seconds or greater as a courtesy.

## Set a max wait time

Set max wait to a reasonable value, like `900` (15 minutes), so that you do not wait an unbounded
amount amount of time. This is especially important when using an exponential backoff.

## Add jitter to the wait time

Random jitter will help you avoid emergent cyclic behavior. This may occur if your script is running
on multiple systems concurrently, or if you are accessing a public resource and many programmers
chose similar constants for their retry logic (eg, if everyone chooses round numbers, then they all
will be multiples of 5, and at some point everyone's retry logic will sync up and make a request at
the exact same time). A useful metaphor is
[metronome synchronization](https://www.youtube.com/watch?v=T58lGKREubo).

See also the [thundering herd problem](https://en.wikipedia.org/wiki/Thundering_herd_problem).

## Set a timeout on the child command

Set a timeout on the child command so that you don't get stuck if there is an infinite loop, dead
lock, or similar issue. Set the

## Avoid output predicates

Output predicates can create performance issues. Try to use status predicates whenever possible.

If you must use an output predicate, use the specific `stdout` or `stderr` variant. The generic
variants are provided for convenience, but are not as performant.
