# Bibliography

This bibliography presents works for the following uses:

- To fact check claims made in this manual.
- To provide further guidance in cases this manual doesn't address.
- To supply context about `attempt`'s design.

# Tenacity

[Tenacity](https://tenacity.readthedocs.io/en/latest/) is a Python library for retrying. It is the
primary inspiration for `attempt`'s design.

Tenacity forked from [Retrying](https://github.com/rholder/retrying) in 2016 as it's original author
and maintainer, Ray Holder, stopped responding to members of the community who reached out. Holder
wrote Retrying in 2013.

Retrying is currently maintained by Greg Roodt in
[a seperate fork](https://github.com/groodt/retrying). Roodt organized the transfer of the
`retrying` package name name in 2022. He attempting to transfer the name to Tenacity, but while he
succeeded in taking over the name, the issue to transfer ownership to Tenacity was not followed up
on. It remains open at the time of this writing. Roodt's fork recieves periodic updates.

Retrying established the core concepts of the architecture:

- An `@retry(...)` decorator which retries a wrapped function
- Three categories of rules, which together for the retrying strategy
  - Retry rules, determining which circumstances result in a retry
  - Stop rules, determining which cirucmstances terminate retrying
  - Wait rules, determining how long we sleep after an attempt

Retrying baked it's predicates into the arguments of the `@retry(...)` decorator, much like
`attempt`.

Tenacity extended the architecture to support functions as arguments, unlocking arbitrary
predicates, and to use context managers in addition to decorators. It added support for async
contexts. Tenacity has also created a large library of utilities and extensive documentation.

## Additional links

- [Retrying issue 65: Friendly fork?](https://github.com/rholder/retrying/issues/65)
- [Retrying issue 100: Maintenance status](https://github.com/rholder/retrying/issues/100)
- [Retrying issue 97: Transfer ownership to tenacity](https://github.com/rholder/retrying/issues/97)
- [Tenacity issue 356: Publish under retrying on PyPI](https://github.com/jd/tenacity/issues/356)
- [pypi issue 2205: Request: retrying](https://github.com/pypi/support/issues/2205)

# References

- [The Synchronization of Periodic Routing Messages](https://dl.acm.org/doi/pdf/10.1145/166237.166241)
  by Sally Floyd and Van Jacobsen, Lawrence Berkeley Laboratory
- [Exponential backoff and jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)
  by Marc Brooker, AWS
- [Transient fault handling](https://learn.microsoft.com/en-us/azure/architecture/best-practices/transient-faults)
  from Azure Architecture Center
- [Retry pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/retry) from Azure
  Architecture Center

# Case studies

- [How to avoid a self-inflicted DDoS attack: CRE Life Lessons](https://cloud.google.com/blog/products/gcp/how-to-avoid-a-self-inflicted-ddos-attack-cre-life-lessons)
  by Dave Rensin and Adrian Hilton, Google
- [Preventing DB failures - Exponential retries with Jitter](https://medium.com/%40writetokrishna/preventing-db-failures-exponential-retries-with-jitter-34b86e23eda8)
  by Krishnakumar Sathyanarayana
- [Good Retry, Bad Retry: An Incident Story](https://medium.com/yandex/good-retry-bad-retry-an-incident-story-648072d3cee6)
  by Denis Isaev, Yandex
