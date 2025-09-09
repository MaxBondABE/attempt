attempt - a CLI for retrying fallible commands
----------------------------------------------

`attempt` allows you to retry fallible commands with a delay.

# How do I use it?

`attempt [BACKOFF] [OPTIONS] [COMMAND]...`

```bash
# Rerun database migrations if the server was not ready
attempt --retry-if-contains "server not ready" sqlx migrate

# Use an exponential backoff
attempt exponential --retry-if-contains "server not ready" sqlx migrate
```

# What can I do with it?

- Wait for a service to start when you aren't [restricted to bash](https://github.com/vishnubob/wait-for-it)
- Write robust scripts accessing resources which may become temporarily unavailable
- Bodge flaky programs into working

## Key features

- **Simple.**
    - The featureset is small but flexible enough to covers most usecases.
    - The codebase can be audited in an afternoon.
- **Robust.**
    - It provides the tools you need to implement fault tolerance, like
        timeouts, jitter, and exponential backoff.
    - The test suite is extensive and contains both end-to-end and unit tests.
- **Free forever.**
    - The codebase is in the public domain.
    - All future versions will be released under the same license.

## Documentation

- See the [usage instructions](https://maxbondabe.github.io/attempt/usage.html) to get a quick start
- The [scripting guide](https://maxbondabe.github.io/attempt/appendix/scripting.html) has advice about using attempt in scripts
- Read the [user manual](https://maxbondabe.github.io/attempt/intro.html) for full documentation
- Run `attempt --help` for quick access to documentation
    - Note that you need to specify a strategy to see it's parameters,
        eg `attempt exponential --help`

# Known issues

- `attempt` assumes that the child command's output will be UTF-8 encoded.
    If any output predicates are used on a program that outputs invalid
    UTF-8, `attempt` will crash.
- `attempt --help` will not reflect that the `fixed` strategy will be used if
    no strategy is specified.

# Installation

`cargo install attempt-cli`
