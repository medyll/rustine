---
story_id: PLAN-ARCH
title: Plan Architecture Implementation & ADRs
status: in_progress
assignee: Developer
---

## Why this story

We need a short, actionable architecture plan that resolves open design questions, records Architecture Decision Records (ADRs), and converts audit findings into a prioritized implementation backlog. This reduces ambiguity for the Implementation phase and makes the next development tasks small and reviewable.

## Acceptance Criteria
- A concise architecture plan and a set of ADRs exist and are discoverable under `bmad/artifacts/`.
- The plan lists prioritized dev stories (small, actionable) to address audit findings and open architecture questions.
- `bmad/status.yaml` recommendation is updated to point at this story (so it appears as the next action).

## Implementation Notes
- Create individual ADRs for decisions that affect runtime behavior or distribution (logging, DB file location/ignore, tray feature default, web fetch timeouts, packaging).
- Prefer small follow-up dev stories (1–2 day tasks) for implementation items so CI/tests can validate each change.
- Keep this plan lightweight: the goal is to unblock implementation, not to over-document.

## Tasks
- [ ] Review `bmad/artifacts/architecture.md` and mark unresolved questions
- [ ] Author ADRs for: logging/observability, DB file handling & migrations, webview/timeouts, tray feature default, packaging/distribution
- [ ] Create dev stories for audit fixes (timeouts, .gitignore rustine.db, Tailwind version alignment, CSS build check)
- [ ] Update `bmad/status.yaml` recommendation to reference this story
- [ ] Share plan/ADRs for quick review and iterate

(End of file)
