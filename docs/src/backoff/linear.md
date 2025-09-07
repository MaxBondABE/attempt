# Linear schedule

Wait more time between attempts, using the following formula:

`(<multiplier> * <attempts>) + <starting_wait>`.

| Attempt | Delay |
| ------- | ----- |
| 0       | 1     |
| 1       | 2     |
| 2       | 3     |

# Example

```bash
attempt linear /bin/false

# Change the multiplier from the default of 1 to 2
attempt linear -x 2 /bin/false
attempt linear --multiplier 2 /bin/false

# Change the starting wait from the default of 1 to 5
attempt linear -W 5 /bin/false
attempt linear --starting-wait 5 /bin/false
```

# Arguments

## `-w --starting-wait <STARTING_WAIT>`

The number of seconds to wait after the first attempts.

## `-x --multiplier <MULTIPLIER>`

The number of additional seconds to wait after each subsequent request.
