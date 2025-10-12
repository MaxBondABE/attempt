# Exponential schedule

Wait exponentially more time between attempts, using the following formula:

`<multiplier> * (<base> ^ <attempts>)`

The attempt counter starts at 0, so the first wait is is for `<multiplier>` seconds.

| Attempt | Delay |
| ------- | ----- |
| 0       | 1     |
| 1       | 2     |
| 2       | 4     |

## Specifying durations

Durations in `attempt` can include units, such as `5min`. Durations without units are assumed to be
in seconds. `300` would be interpreted as 300 seconds. Multiple units can be used together such as
`1hr30m`. You may include whitespace, so `1hr 30m` is also valid.

The following units are supported:

- Hours (`h` or `hr`)
- Minutes (`m` or `min`)
- Seconds (`s`)
- Milliseconds (`ms`)
- Nanoseconds (`ns`)

# Example

```bash
attempt exponential /bin/false
attempt exp /bin/false

# Change the multiplier from the default of 1s to 2s
attempt exponential -x 2s /bin/false
attempt exponential --multiplier 2s /bin/false

# Change the exponential base from the default of 2 to 5
attempt exponential -b 5 /bin/false
attempt exponential --base 5 /bin/false
```

# Arguments

## `-b --base <BASE>`

The base of the exponential function. The default is 2, corresponding to doubling the wait time
between attempts.

## `-x --multipler <MULTIPLIER>`

Scale of the exponential function. The default is 1.
