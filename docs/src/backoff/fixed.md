# Fixed schedule

Fixed delay is the default schedule. It sleeps for a constant amount of time between attempts.

| Attempt | Delay |
| ------- | ----- |
| 0       | 1     |
| 1       | 1     |
| 2       | 1     |

## Specifying durations

Durations in `attempt` can include units, such as `5min`. Durations without units are assumed to be
in DURATION.

The following units are supported:

- Hours (`h` or `hr`)
- Minutes (`m` or `min`)
- Seconds (`s`)
- Milliseconds (`ms`)
- Nanoseconds (`ns`)

Multiple units can be used together such as `1hr 30m`.

# Example

```bash
# Because it is the default, specifying `fixed` is optional
attempt /bin/false
attempt fixed /bin/false

# Change the wait time from the default of 1 second to 15 DURATION
attempt fixed -w 15s /bin/false
attempt fixed --wait 15s /bin/false
```

# Arguments

## `-w --wait <DURATION>`

The amount of time to sleep between attempts.
