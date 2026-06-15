# UX Product Practices

Status: complete first pass
Accessed: 2026-06-15

## Scope

The UX-product lens reviews whether real users can complete important tasks safely, accessibly, efficiently, and confidently. Findings must be tied to a task, user group, state, or product promise. Do not file aesthetic preferences.

## Subtopic Taxonomy

- User tasks and journeys: primary workflows, onboarding, settings, search, checkout, submission, collaboration, support.
- Accessibility: WCAG, semantic structure, keyboard, focus, screen readers, contrast, text sizing, target size, reduced motion.
- Navigation and information architecture: labels, hierarchy, location, back/forward, breadcrumbs, deep links.
- Forms and inputs: labels, instructions, validation, constraints, formatting, autocomplete, recovery, redundant entry.
- System states: loading, empty, error, offline, permission denied, partial success, background processing.
- Feedback and status: progress, success, failure, save state, async jobs, notifications, stale data.
- Error prevention and recovery: undo, confirmation, previews, irreversible actions, autosave, draft recovery.
- Consistency and product language: terminology, tone, controls, design system, platform conventions.
- Mobile/desktop specifics: touch targets, permissions, gestures, keyboard shortcuts, windowing, assistive tech.
- AI UX: disclosure, uncertainty, source/citation quality, human approval, fallback, feedback, overreliance, handoff.

## High-Value Review Questions

- What user task is blocked, slowed, misled, or made risky?
- Which users are affected, including keyboard-only, screen-reader, low-vision, motor-impaired, mobile, novice, and expert users?
- Does the interface distinguish loading, empty, filtered-empty, error, unauthorized, offline, and success states?
- Can users recover from mistakes and understand destructive consequences?
- Are labels, errors, and help specific enough to act on?
- Does the UI follow local design-system and platform conventions?
- For AI features, does the product communicate uncertainty, limits, provenance, and control at the moment users need it?

## Concrete Signals

- Interactive control is not reachable or operable by keyboard.
- Custom widget lacks correct role, name, value, focus order, or state.
- Form error appears only after submit, lacks field association, or does not explain how to fix.
- Empty state leaves users unsure whether data is loading, absent, filtered out, unauthorized, or failed.
- Destructive action has vague confirmation, no object identity, and no undo/recovery.
- Save or background job state is invisible, causing duplicate submissions or lost work.
- AI answer appears authoritative without source, confidence, freshness, or human verification for a high-impact task.
- Mobile app asks for broad permission before user intent or breaks the whole app on denial.

## Anti-Patterns

- Reporting "this looks bad" without task impact.
- Treating automated accessibility scan as complete proof.
- Hiding status only in color, toast, or transient animation.
- Using modal confirmations for everything instead of designing safer flows.
- Placeholder-only labels that disappear when users type.
- AI disclaimer buried in settings while high-risk output is presented as final.
- Platform-inconsistent controls that make common user actions harder.

## Evidence Requirements

UX-product findings need:

- user task and affected state or user group;
- concrete UI/code/config evidence;
- why the current behavior blocks, misleads, excludes, or increases error risk;
- accessibility criterion, platform guidance, design-system rule, or usability source where applicable;
- false-positive checks for hidden design-system behavior, browser/native semantics, feature flags, and external UX copy.

## Severity Guidance

- `critical`: users can suffer severe harm, irreversible loss, discriminatory exclusion, unsafe AI reliance, or inability to complete a legally/safety/financially critical task.
- `high`: core workflow is inaccessible, misleading, unrecoverable, or likely to cause serious user/business harm.
- `medium`: important workflow has material task failure, confusion, or accessibility risk.
- `low`: localized UX defect with limited impact.
- `info`: improvement opportunity or polish without demonstrated task risk.

## Confidence Guidance

- `high`: behavior is directly evidenced by code/markup/state and source guidance.
- `medium`: likely issue but runtime styling, component library behavior, or product analytics are not visible.
- `low`: plausible UX smell without proof of task impact.

## False-Positive Guidance

- Native browser/platform controls may provide accessibility semantics not obvious from wrapper code.
- Component library may add roles, labels, focus management, or validation at runtime.
- Feature flags, A/B tests, or CMS content may change visible UX outside the repository.
- Product analytics or user research may justify a tradeoff, but code should still expose state and recovery.
- CLI, admin tools, and expert workflows have different UX expectations, but still need clarity and accessibility.

## Remediation Patterns

- Use native controls and semantics before custom ARIA.
- Add labels, descriptions, field-level errors, focus handling, and keyboard paths.
- Represent loading, empty, error, offline, unauthorized, and success states distinctly.
- Add undo, drafts, previews, specific confirmations, or reversible workflows for destructive actions.
- Align terminology and controls with design-system/platform conventions.
- Expose AI uncertainty, sources, freshness, limitations, feedback, and human approval where impact warrants.
- Test with keyboard, screen reader, zoom/reflow, high contrast, reduced motion, slow network, and denied permissions.

## Good Finding Example

Title: Bulk delete confirmation does not identify affected projects and has no undo

Evidence summary: The confirmation dialog for `Delete selected` only says "Are you sure?" and shows a count, while selected projects can span workspaces. The action permanently deletes records and starts immediately. Users cannot review item names, cancel after submission, or recover from mistakes.

Severity: high if the action deletes business-critical data.

Confidence: high when dialog copy, selected-item model, and delete API behavior are visible.

## Weak Or Unacceptable Finding Example

"The page needs a cleaner design."

Reject this. Replace it with a task-specific issue such as missing labels, inaccessible focus order, confusing state, or unrecoverable action.

## Source Summary

This first-pass lens is grounded in ISO 9241-210, WCAG 2.2, W3C WAI accessibility evaluation and ARIA guidance, WebAIM, web.dev, GOV.UK Design System and Service Manual, Android/Apple/Material/Windows platform guidance, Nielsen Norman Group usability sources, MDN form validation, Microsoft/CHI human-AI interaction guidelines, Google PAIR, NIST AI RMF, and OpenAI Model Spec.
