# Policy controls

Policy controls define the conditions under whether `attempt` decides to retry or stop retrying a
command. They use predicates that examine the command's exit status, output, or signals.

Each predicate has two variants: retry and stop. Retry predicates cause `attempt` to retry the
command, while stop predicates cause us to terminate.

## Usage

Retry predicates are used to identify temporary error conditions, such as network timeouts or rate
limiting errors, which may be resolved by retrying. `attempt` will try to rerun the child command if
a retry predicate is matched. It will may be prevented from doing so if the maximum number of
attempts has been reached, or if a stop predicate is matched.

Stop predicates are used to identify permanent error conditions, such as authentication errors or
malformed content errors, which will never be resolved by retrying. If a stop predicate matches,
`attempt` will imediately cease retrying the child command.

## Precedence

Stop predicates always have precedence over retry predicates.

```bash
> attempt --stop-if-status 1 --retry-if-status 1 /bin/false
Command exited: exit status: 1.
Stop: Status matches.
Terminated: Command has failed, but cannot be retried.
>
```
