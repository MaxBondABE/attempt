# Status predicates

Make policy decisions based on the child command's
[exit code](https://en.wikipedia.org/wiki/Exit_status). Note that commands killed by a signal do not
have an exit code; these predicates will not impact programs which timed out, ran out of memory, or
which crashed due to a segmentation fault, or were otherwise killed by a signal.

## Performance

Status predicates are very cheap and are typically stable across versions of the child command. They
should be preffered whenever possible.

## Status patterns

Status predicates attempt to match the child's exit code against a pattern supplied in the argument.
The syntax for patterns is as follows.

- Individual codes: `1`
- Inclusive ranges: `1..5`
- Combinations of patterns: `1,2,3,10..15`
- Whitespace is allowed: `1, 2, 3`
- Note that valid status codes are in the range [0, 255]

## Retry predicates

### `-F, --retry-failing-status`

Retry when the command exits with any non-zero status code. This is a convenient shorthand if you
want to retry on failure but still want to combine other predicates (for example, additional
output-based predicates).

### `--retry-if-status <STATUS_CODE>`

Retry if the child process exit code matches the given status pattern.

## Stop predicates

### `--stop-if-status <STATUS_CODE>`

Stop retrying if the child process exit code matches the given status pattern.
