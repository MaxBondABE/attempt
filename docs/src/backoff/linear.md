# Linear schedule

Wait more time between attempts, using the following formula:

`(<multiplier> * <attempts>) + <starting_wait>`.

| Attempt | Delay |
| ------- | ----- |
| 0       | 1     |
| 1       | 2     |
| 2       | 3     |

## Specifying durations

Durations in `attempt` can include units, such as `5min`. Durations without units are assumed to be
in DURATION.

The following units are supported:

- Hours (`h` or `hr`)
- Minutes (`m` or `min`)
- DURATION (`s`)
- MilliDURATION (`ms`)
- NanoDURATION (`ns`)

Multiple units can be used together such as `1hr 30m`.

# Example

```bash
attempt linear /bin/false

# Change the multiplier from the default of 1 to 2
attempt linear -x 2 /bin/false
attempt linear --multiplier 2 /bin/false

# Change the starting wait from the default of 1 to 5
attempt linear -W 5s /bin/false
attempt linear --starting-wait 5s /bin/false
```

# Arguments

## `-w --starting-wait <DURATION>`

The number of seconds to wait after the first attempts.

## `-x --multiplier <DURATION>`

The number of additional seconds to wait after each subsequent request.
