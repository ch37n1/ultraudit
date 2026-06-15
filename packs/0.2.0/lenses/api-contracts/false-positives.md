## False-Positive Guidance

- API gateways may enforce schemas, auth scopes, pagination limits, or compatibility externally.
- Generated serializers may transform server models before the response leaves the service.
- Monorepos or release trains may deploy consumers and providers atomically.
- Schema registries or brokers may enforce event compatibility outside this repository.
- Public docs may be generated after build, not committed.

