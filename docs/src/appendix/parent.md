attempt/docs/src/appendix/parent.md

# Using attempt as a parent process

`attempt` is primarily a CLI retry tool, but many people want to use it as a simple
parent/supervisor for short-lived services or to restart a process inside a container. The following
guidance covers the most important considerations and recommended patterns when running `attempt` as
a parent process.

## What `attempt` is (and isn't)

- `attempt` is a retry wrapper: it runs a child command, observes its exit status and output
  according to predicates, and optionally retries with a configurable backoff.
- It is not a full init system or process supervisor. It does not attempt to implement all PID1
  responsibilities (signal handling, reaping complicated process trees, service metadata, graceful
  shutdown orchestration, etc.). If you need a robust init/supervisor, prefer dedicated tools
  (systemd, runit, s6, supervisord) or a tiny init for containers (like `tini` or `dumb-init`) and
  use `attempt` for restart/retry behavior when appropriate.

## Signals & shutdown

- If your environment relies on signal forwarding (for example, your container runtime sends SIGTERM
  to PID 1), be aware that `attempt` may not forward every signal exactly like a dedicated init
  would. If proper signal forwarding and reaping is required, run `attempt` under a minimal init
  such as `tini` rather than as PID 1 directly.
- Use `--stop-if-killed` to avoid retrying when the child is killed by a signal; otherwise `attempt`
  may consider the child retryable. Note that `--stop-if-timeout` implies that a timed-out child
  (which is killed by the timeout mechanism) will not be retried.
- If you rely on `--timeout`, `attempt` will kill the child when the timeout expires. Use
  `--stop-if-timeout` if you want that situation to stop further retries.

## Long-running services

- For long-running services you want to restart if they crash, use:
  - `--forever` (restart regardless of success/failure), or
  - `-U/--unlimited-attempts` with an appropriate retry predicate.
- Example: keep a service running, restarting on any crash:

```docs/usage.md#L1-20
attempt --forever /usr/local/bin/myservice --serve
```

- If you want some backoff between restarts, configure a backoff strategy. A gentle exponential with
  jitter helps avoid rapid restart loops and thundering-herd problems.

## Recommended strategy settings for parent-like use

- Use exponential backoff with a sensible multiplier and a maximum wait time to avoid unbounded
  sleeps:

```docs/usage.md#L1-20
attempt exponential --multiplier 2 --base 2 --wait-max 900 --jitter 5 --forever /usr/local/bin/myservice
```

- Add `--jitter` to avoid synchronized retries when many instances are managed externally.
- Use `--stagger` if you launch many `attempt` parents at once (for example, from an orchestrator)
  to randomize initial starts.

## Timeouts & expected-runtime

- Always set `--timeout` for commands that might hang or deadlock. Without a timeout you risk
  `attempt` waiting indefinitely on a hung child.
- Use `--expected-runtime` to tell `attempt` how long a normal run should take; during that period
  it will poll the child less aggressively (reducing system load). This is useful for services that
  perform occasional long-running initialization.

## Predicates & performance

- Prefer status code predicates (like `--retry-if-status`) over output predicates
  (`--retry-if-contains`, `--retry-if-matches`) because output predicates require capturing and
  scanning stdout/stderr and may cause performance issues for verbose processes.
- If you must inspect output, prefer the specific `--retry-if-stdout-*` or `--retry-if-stderr-*`
  variants rather than the generic versions.
- Remember that "stop" predicates take precedence over "retry" predicates. Use `--stop-if-*` to
  abort retries for unrecoverable situations.

## Exit codes (for supervising processes)

`attempt` uses stable exit codes your supervisor or orchestration tooling can rely on. The important
ones:

- `0` — child eventually ran successfully within allowed retries
- `1` — I/O error (e.g. command not found)
- `2` — invalid arguments (bad invocation)
- `3` — retries exhausted without success
- `4` — stopped early because a "stop" predicate triggered
- `101` — internal crash (e.g. encountering non-UTF-8 when using output predicates)

Use these exit codes to distinguish between transient failures (which may be retriable by the
supervisor) and configuration or internal errors that merit operator attention.

## When used in containers

- Do not use `attempt` as a drop-in PID 1 if you need proper signal handling or reaping. Instead:
  - Run a tiny init (e.g. `tini`) as PID 1 and run `attempt` as a child under that init; or
  - Use your container orchestration restart features and rely on `attempt` only for retry/backoff
    logic inside the container if you understand its limits.
- Example with a minimal init:

```docs/usage.md#L1-20
tini -- attempt --forever --stagger 5 --jitter 3 /usr/local/bin/myservice --serve
```

## Practical examples

- Restart a service on crash, but give it 1s, 2s, 4s between restarts with jitter and a max wait:

```docs/usage.md#L1-20
attempt exponential --multiplier 1 --base 2 --wait-max 60 --jitter 2 --forever /usr/local/bin/myservice
```

- Start multiple small workers at once but avoid thundering herd at boot:

```docs/usage.md#L1-20
attempt --stagger 10 --jitter 3 --forever /usr/local/bin/worker --do-work
```

- Restart only on certain exit codes (useful when your program uses specific codes to signal
  "temporary" failures):

```docs/usage.md#L1-20
attempt --retry-if-status 100..199 --forever /usr/local/bin/task-runner
```

## Summary checklist

- If you need robust init semantics, use a real init and run `attempt` under it.
- Use `--forever` or `-U` for restart-as-parent behavior.
- Configure backoff, jitter, and wait max to avoid hot loops and thundering-herd issues.
- Set `--timeout` and `--expected-runtime` to avoid hangs and reduce polling load.
- Prefer status predicates over output predicates for performance.
- Use exit codes to inform your higher-level supervisor about why `attempt` exited.

If you want, I can add a short example `Dockerfile` or a tiny `systemd` unit showing a recommended
invocation for a container or service manager.
