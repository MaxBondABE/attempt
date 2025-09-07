# Exit codes

The following exit codes are used by `attempt` to indicate whether it failed and how. Scripts should
use these exit codes and not the log messages exposed by `--verbose`; these exit codes will remain
stable, but no such guarantee is made for the log messages.

| Code number | Description                                                                                                         |
| ----------- | ------------------------------------------------------------------------------------------------------------------- |
| 0           | Command was run successfully within the allowed number of retries.                                                  |
| 1           | I/O error (eg, command not found). An error message will be printed.                                                |
| 2           | Invalid arguments. An error message will be printed.                                                                |
| 3           | The number of retries has been exhausted without the command ever succeeding.                                       |
| 4           | The number of retries has not been exhausted, but the command is no longer retryable because of a "stop" predicate. |
| 101         | `attempt` has crashed. The most likely cause is using output predicates on data which is not UTF-8 encoded.         |
