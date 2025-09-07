# Exponential schedule

Wait exponentially more time between attempts, using the following formula:

`<multiplier> * (<base> ^ <attempts>)`

| Attempt | Delay |
| ------- | ----- |
| 0       | 1     |
| 1       | 2     |
| 2       | 4     |

The attempt counter starts at 0, so the first wait is is for `<multiplier>` seconds.

# Example

```bash
attempt exponential /bin/false
attempt exp /bin/false

# Change the multiplier from the default of 1 to 2
attempt exponential -x 2 /bin/false
attempt exponential --multiplier 2 /bin/false

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
