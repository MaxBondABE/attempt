# The default policy

If no policy controls are specified, `attempt` will retry if the child command which exits with a
status other than 0 or that is killed by a signal. It will retry a maximum of 3 times with a fixed
delay of 1 second.

This is equivalent to the following arguments:

```bash
attempt \
    fixed --wait 1s \
    --attempts 3 \
    --retry-failing-status \
    --retry-if-killed \
   /bin/false
```
