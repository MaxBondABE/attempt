# Backoff schedule

The backoff schedule determines how long we wait between successive attempts. The simplest schedule
is a fixed delay, where we wait the same amount of time between all attempts. But it's often a good
idea to wait longer the more failures we encounter to avoid sending requests to an already
overloaded system.

We may want an aggressive backoff schedule with short delays, so that we recover from transient
failures quickly. But longer delays allow systems to self-heal.

The exponential backoff strategy allows us to balance these concerns. In exponential backoff we
double the amount of time we wait after each failure. This allows us to start with an aggressive
delay but to fall back to a long delay if the outage proves persistent.
