attempt - a CLI for retrying fallible commands
----------------------------------------------

`attempt` allows you to retry fallible commands with a delay.

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

# Installation

TBD

# What can I do with it?

- Wait for a service to start when you aren't [restricted to bash](https://github.com/vishnubob/wait-for-it)
- Improve the robustness of scripts accessing resources which may become temporarily unavailable
- Bodge flaky programs into working

# How do I use it?

`attempt [STRATEGY] [OPTIONS] [COMMAND]...`

```bash
# Rerun database migrations if the server was not ready
attempt --retry-if-contains "server not ready" sqlx migrate
```

## Documentation

- Read the [usage instructions](docs/usage.md) to get a quick start
- See [here](docs/scripting.md) for advice for using attempt in scripts
- Run `attempt --help` for full documentation
    - Note that you need to specify a strategy to see it's parameters,
        eg `attempt exponential --help`

# Known issues

- `attempt` assumes that the child command's output will be UTF-8 encoded.
    If any output predicates are used on a program that outputs invalid
    UTF-8, `attempt` will crash.
- `attempt --help` will show the strategy as coming after the child command.
    This is incorrect, the child command comes last.
