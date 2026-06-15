## False-Positive Guidance

- SDKs may have default retries, timeouts, and retry budgets.
- Gateways, service meshes, queue brokers, and orchestrators may enforce controls outside the repo.
- Small CLIs and local tools may not need SLOs or load shedding.
- Some manual recovery is acceptable for low-frequency, low-impact workflows.
- Fallback may be legally required, but it still needs testing and limits.

