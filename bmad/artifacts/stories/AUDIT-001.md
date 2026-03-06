---
story_id: AUDIT-001
title: Add Unit Tests for DB Actor & URL Validation
status: planned
assignee: PM
---

## Why this story

The codebase currently lacks automated tests around the internal DB actor and URL handling. This story adds a focused unit test that exercises inserting URLs via the DB actor and retrieving them via list_recent, ensuring the basic data path works as expected.

## Acceptance Criteria
- A unit test exists that inserts a URL through the DB actor and validates retrieval via list_recent.
- Running cargo test should pass for the new test without affecting other tests.
- No regressions introduced to existing DB tests.

## Implementation Notes
- Implement test in src/db.rs under #[cfg(test)] mod tests to reuse the existing DB actor logic.
- The test should insert a URL with a current timestamp and verify the last inserted item matches the URL.
- Keep tests isolated and avoid relying on network access; use in-memory or deterministic timestamps where possible.

## Tasks
- [ ] Add test function test_db_insert_and_list_recent_basic (Rust)
- [ ] Run cargo test and fix any failures
