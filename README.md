attempt - a CLI for retrying fallible commands
----------------------------------------------

`attempt` allows you to retry commands with a delay if they return an exit code
other than 0.

It will return a status code of 0 if the command eventually succeeded and a 1 if
all attempts were exhausted without encoutnering a success.
