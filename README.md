attempt - a CLI for retrying fallible commands
----------------------------------------------

`attempt` allows you to retry fallible commands with a delay. `attempt`
strives to provide three key benefits:

- **Simple.**
    - The featureset is small but flexible enough to covers most usecases.
    - The codebase can be audited in an afternoon.
- **Robust.**
    - It provides the tools you need to implement fault tolerance, such as
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
- Run `attempt --help` for full documentation
    - Note that you need to specify a strategy to see it's parameters,
        eg `attempt exponential --help`

# Known issues

- `attempt` assumes that the child command's output will be UTF-8 encoded.
    If any output predicates are used on a program that outputs invalid
    UTF-8, `attempt` will crash.
- `attempt --help` will show the strategy as coming after the child command.
    This is incorrect, the child command comes last.

# Why did you make this?

I wrote `attempt` for two reasons.

I wanted a more featureful alternative to [wait_for_it.sh](https://github.com/vishnubob/wait-for-it) for
use in Docker Compose files.

I also wanted to experiment with an idea I had that a good way to make a CLI tool is to mimic the API of
an especially good library. In this case, inspiration was the Python library
[tenacity](https://tenacity.readthedocs.io/en/latest/). `attempt` takes the three central ideas of `tenacity` -
[wait functions,](https://tenacity.readthedocs.io/en/latest/api.html#wait-functions)
[retry functions,](https://tenacity.readthedocs.io/en/latest/api.html#wait-functions) and
[stop functions](https://tenacity.readthedocs.io/en/latest/api.html#stop-functions) -
and translates them into a CLI context.

Command line arguments are not as expressive as Python code, so instead of arbitrary functions,
`attempt` uses predicates for stopping and retrying. Wait functions are translated to backoff
strategies, and limited to a predefined set.

I consider the experiment a success.
