# Introduction

`attempt` allows you to retry fallible commands with a delay. It accepts the command line of a child
command. This command is run repeatedly, until it either completes without failing or the allowed
number of retries is exhausted.

There are two cases when this is often used:

- In an environment like `docker-compose`, where a service may need to wait on it's dependencies to
  start up (compare to [`wait-for-it.sh`](https://github.com/vishnubob/wait-for-it))
- When writing shell scripts accessing remote resources which may become temporarily unavailable

# Why does retrying work?

## Many failures are transient

Many failures are transient, even flukes, and are unlikely to be rencountered if retried. For
example, a system we integrate with may have a rare bug that impacts some small number of requests
arbitrarily, without regards to their content. Such bugs may go a long time before they are fixed.
In such a case, immediately retrying the request would succeed.

## Many systems self-heal

Many systems are able to provision additional capacity in response to increased load. If given time
to scale, these systems will process our requests successfully.

# Retry strategies

`attempt` accepts various arguments to configure it's behavior and how it responds to failures.
Together these form our retry strategy.

## Retry policy

The retry policy allows us to distinguish between temporary and permanent error condtions, to
determine whether we should try another attempt. We can look at a command's exit status, or we can
inspect it's output for relevant messages.

Failures such as network timeouts or rate limiting errors (like the HTTP status code 429) should be
retried. Failures that stem from the content of our requests (like the HTTP status code 422) will
not succeed on a retry, and so we should terminate.

## Backoff schedule

The backoff schedule determines how long we wait between successive attempts. The simplest schedule
is a fixed delay, where we wait the same amount of time between all attempts. But it's often a good
idea to wait longer the more failures we encounter to avoid sending requests to an already
overloaded system.

We may want an aggressive backoff schedule with short delays, so that we recover from transient
failures quickly. But longer delays allow systems to self-heal.

The exponential backoff strategy allows us to balance these concerns. In exponential backoff we
double the amount of time we wait after each failure. This allows us to start with an aggressive
delay but to fall back to a long delay if the outage proves persistent.

## Retry limit

Retry logic without a retry limit is an infinite loop. We'll need to pick some sort of limit.

There is a tradeoff between how long we keep retrying and how quickly we can act on an irrecoverable
failure. The longer we wait, the more likely we are to recover from a transient failure. But if our
command is never going to succeed, we want to know sooner than later so that we can investigate and
fix the problem. We'll need to set a retry limit that balances these concerns.

## Jitter

An essential element of any retry strategy is random jitter. If we do not add randomness to the
delays produced by our retry strategy, we will encounter emergent cyclic behavior. Concurrent
retriers will syncronize with each other. This will create huge spikes in requests per second where
all clients send requests at once.

Random jitter breaks up these emergent patterns and flattens the RPS curve. This helps systems that
are having trouble or need to scale recover smoothly.

# Lifecycle of an attempt

1. The child command is executed.
1. The retry policy is evaluated against the child command.
1. If a stop rule is matched, we terminate the program immediately.
1. If a retry rule is matched, and we have not reached the allowed number of retries, return to
   step 1.
1. Otherwise, we terminate the program.

## Peaking under the hood with `--verbose`

We can use the `--verbose` argument to expose these steps of the lifecycle. To show all the
information available, we'll use `--verbose --verbose`, shortened to `-vv`.

Let's run `attempt` against `/bin/true`, a program that always succeeds.

```bash
> attempt -vv /bin/true
Starting new attempt...
Evaluating policy...
Command exited: exit status: 0.
Stop: Command was successful.
Terminated: Success.
>
```

`true` exited with a status of `0`, indicating success. Because it was successful, it was not
eligible to be retried. And so, `attempt` exits.

Now let's try running `attempt` against `/bin/false`, a program which always fails.

```bash
> attempt -vv /bin/false
Starting new attempt...
Evaluating policy...
Command exited: exit status: 1.
Retry: Command failed.
Command has failed, retrying in 1.00 seconds...
Starting new attempt...
Evaluating policy...
Command exited: exit status: 1.
Retry: Command failed.
Command has failed, retrying in 1.00 seconds...
Starting new attempt...
Evaluating policy...
Command exited: exit status: 1.
Retry: Command failed.
Terminated: Retries exhausted.
>
```

`false` always exits with a status of `1`, and so `attempt` will always retry it if possible. By
default, `attempt` waits for 1 second between attempts, and will retry a maximum of 3 times. After
the third attempt, the program exits.
