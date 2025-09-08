# Example usage

## Basic usage

```bash
# Basic example
attempt /bin/false
attempt --verbose /bin/false
```

## Disambiguating child arguments

```bash
# Use `--` to disambiguate between arguments to `attempt` and arguments to
# it's child command
attempt -a 10 -- foo -a bar
```

## A `sqlx` example

```bash
# Rerun database migrations if the server was not ready
# Useful for `docker-compose` and similar tools (any place
# you'd use `wait-for-it.sh` & aren't restricted to bash).
attempt --retry-if-contains "server not ready" sqlx migrate
```

## Using an exponential backoff

```bash
# Use exponential backoff
attempt exp /bin/false
attempt exponential /bin/false

# Change the multiplier from 1 second to 100 milliseconds. Instead of starting
# with a 1 second delay (then 2s, 4s, 8s, ...), use a 100 millisecond delay
# (then 200ms, 400ms, 800ms, ...).
attempt exp -x 100ms
attempt exp --multiplier 100ms

# Change the base from 2 to 3. This will triple the delay between every
# attempt, rather than doubling it.
attempt exp -b 3
attempt exp --base 3
```

## Change the number of attempts

```bash
# Change the number of attempts from 3 to 10
attempt -a 10 /bin/false
attempt --attempts 10 /bin/false
```

## Add random jitter to the wait time

```bash
# Add 1 second of random jitter to wait time
attempt -j 1s /bin/false
attempt --jitter 1s /bin/false
```

## Setting a minimum or maximum wait time

```bash
# Set a minimum wait time of 5 seconds between attempts
attempt -m 5s /bin/false
attempt --wait-min 5s /bin/false

# Set a maximum wait time of 15 minutes between attempts
attempt -M 15m exponential /bin/false
attempt --wait-max 15m exponential /bin/false

# Combine min and max wait times
attempt -m 5s -M 15m exponential /bin/false
```

## Setting a timeout on child command runtime

```bash
# Kill the child command if it runs longer than 30 seconds
attempt -t 30s -- sleep 60
attempt --timeout 30s -- sleep 60
```

## Retrying on status codes

```bash
# Retry on any failing status code (non-zero)
attempt -F /bin/false
attempt --retry-failing-status /bin/false

# Retry only on specific status codes
attempt --retry-if-status 1 /bin/false
attempt --retry-if-status "1,2,3" /bin/false # Retry on 1, 2, or 3
attempt --retry-if-status "1..5" /bin/false # Retry on any status between 1 and 5
attempt --retry-if-status "1..5,10,15..20" /bin/false # Combine rules to form complex patterns

# Retry connection timeouts with `curl`
attempt --retry-if-status 28 curl https://example.com

# Stop retrying on specific status codes
attempt --stop-if-status 2 command_with_permanent_errors
```

## Retrying on child command output

```bash
# Retry if output contains a specific string
attempt --retry-if-contains "Connection timed out" curl https://example.com
attempt --retry-if-stderr-contains "Connection timed out" curl https://example.com

# Retry if output matches a regex pattern
attempt --retry-if-matches "error \d+" error_prone_command
attempt --retry-if-stdout-matches "failed.*retry" flaky_service

# Stop retrying if output indicates permanent failure
attempt --stop-if-contains "Authentication failed" secure_command
attempt --stop-if-stderr-contains "Authentication failed" secure_command
```

## As a long-running parent process

```bash
# Always restart the child command if it exits, and do not wait before restarting
attempt --forever --wait 0 long_running_service
```
