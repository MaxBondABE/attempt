# Output predicates

Output predicates examine the text written to stdout and stderr by the child command to determine
whether to retry or stop. These predicates are useful when exit codes alone don't provide enough
information about the failure condition.

## Performance

Output predicates are often more convinient than status predicates, and users are encouraged to use
them when in circumstances when developer time is the greatest concern (such as in command lines and
for throw-away scripts). However, they are discouraged in scripting or CI use.

Output predicates are the most expensive. They require additional system calls to retrieve the
output and additional computation to search it. They are typically unstable across different
versions of the child process.

Status predicates should be prefferred when possible. If output predicates are used, the more
specific predicates (eg `--retry-if-stdout-contains` vs `--retry-if-contains`) should be preferred.

## Regular expression syntax

The regular expression predicates use the
[`regex` crate's syntax](https://docs.rs/regex/latest/regex/#syntax).

## Retry predicates

### `--retry-if-contains <STRING>`

Retry if either stdout or stderr contains the specified string.

### `--retry-if-matches <REGEX>`

Retry if either stdout or stderr matches the specified regular expression.

### `--retry-if-stdout-contains <STRING>`

Retry if stdout contains the specified string.

### `--retry-if-stdout-matches <REGEX>`

Retry if stdout matches the specified regular expression.

### `--retry-if-stderr-contains <STRING>`

Retry if stderr contains the specified string.

### `--retry-if-stderr-matches <REGEX>`

Retry if stderr matches the specified regular expression.

## Stop predicates

### `--stop-if-contains <STRING>`

Stop retrying if either stdout or stderr contains the specified string.

### `--stop-if-matches <REGEX>`

Stop retrying if either stdout or stderr matches the specified regular expression.

### `--stop-if-stdout-contains <STRING>`

Stop retrying if stdout contains the specified string.

### `--stop-if-stdout-matches <REGEX>`

Stop retrying if stdout matches the specified regular expression.

### `--stop-if-stderr-contains <STRING>`

Stop retrying if stderr contains the specified string.

### `--stop-if-stderr-matches <REGEX>`

Stop retrying if stderr matches the specified regular expression.
