## False-Positive Guidance

- Native browser/platform controls may provide accessibility semantics not obvious from wrapper code.
- Component library may add roles, labels, focus management, or validation at runtime.
- Feature flags, A/B tests, or CMS content may change visible UX outside the repository.
- Product analytics or user research may justify a tradeoff, but code should still expose state and recovery.
- CLI, admin tools, and expert workflows have different UX expectations, but still need clarity and accessibility.

