# Fixed schedule

Fixed delay is the default schedule. It sleeps for a constant amount of time between attempts.

| Attempt | Delay |
| ------- | ----- |
| 0       | 1     |
| 1       | 1     |
| 2       | 1     |

# Example

```bash
# Because it is the default, specifying `fixed` is optional
attempt /bin/false
attempt fixed /bin/false

# Change the wait time from the default of 1 second to 15 seconds
attempt fixed -w 15 /bin/false
attempt fixed --wait 15 /bin/false
```

# Arguments

## `-w --wait <SECONDS>`

The amount of time to sleep between attempts.
