# Security

`attempt` is primarily intended to be run in local development environments, where the attack
surface is limited or nonexistant. If you are running `attempt` in a production/untrusted
environment, you should be aware of the potential for vulnerabilities.

# Classes of vulnerability

`attempt` is vulnerable to three primary classes of vulnerability:

- Terminating retries before an application has succeeded
- Retrying an application that has already succeed
- Denial of service attacks

# Potential vulnerabilities

## Manipulating output predicates

If your application logs includes in it's output data from remote hosts or other untrusted sources,
it is likely that attackers will be able to manipulate output predicates. Attackers will likely be
able to trigger retry or stop predicates in inappropriate situations.

## Denial of service attacks on output predicates

Attackers able to influence the output of your application may mount denial of service attacks by
causing it to include invalid UTF-8, which crashes `attempt`. They may also be able to crash
`attempt` by making the output large.

If you are using regex predicates, attackers may be able to mount denial of service attacks by
supplying
[pathological inputs](https://owasp.org/www-community/attacks/Regular_expression_Denial_of_Service_-_ReDoS).

## Sending signals to child processes

An attacker with access to the system `attempt` is running on could manipulate signal predicates by
sending signals to `attempt`'s child process.

# Mitigations

## Write child processes to be idempotent

Whenever possible, child processes should be written such that they tolerate being retried on a
success.

## Avoid output predicates

Prefer the use of status predicates to output predicates. Status predicates are more difficult to
manipulate.

## Setup alerting for failed processing

If you are relying on your application to run,
