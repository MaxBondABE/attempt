

# Why did you make this?

I wrote `attempt` for two reasons.

I wanted a more featureful alternative to [wait_for_it.sh](https://github.com/vishnubob/wait-for-it) for
use in Docker Compose files.

I also wanted to experiment with an idea I had that a good way to make a CLI tool is to mimic the API of
an especially good library. In this case, inspiration was the Python library
[tenacity](https://tenacity.readthedocs.io/en/latest/). You can read more about it [here](docs/experiment.md)


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
