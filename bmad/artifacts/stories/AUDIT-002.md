---
story_id: AUDIT-002
title: Add CI Workflow for Build, Lint, and Test
status: completed
assignee: Developer
---

## Why this story

The project currently lacks a CI workflow to enforce build, lint, and test quality gates. Adding a GitHub Actions workflow ensures all pushes and PRs are validated automatically, reducing manual errors and improving reliability.

## Acceptance Criteria
- A GitHub Actions workflow file exists at .github/workflows/ci.yaml.
- The workflow runs cargo fmt, cargo clippy, cargo test, and cargo build on push and pull_request events.
- The workflow passes for the current codebase.
- No secrets or sensitive data are committed.

## Implementation Notes
- Use Rust 2021 and stable toolchain in the workflow.
- Add steps for formatting, linting, testing, and building.
- Fail the job if any step fails.
- Document the workflow in README if not already present.

## Tasks
- [ ] Create .github/workflows/ci.yaml with Rust build/lint/test steps
- [ ] Run workflow and fix any failures
- [ ] Update README with CI badge and workflow description
