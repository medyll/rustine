# Audit Report - full - 2026-03-05

## Project Map

- Type: Single package
- Package detected: rustine (root)
- Stack: Rust 2021, Dioxus Desktop, rusqlite (SQLite bundled), Tailwind CSS (pnpm)
- CI/CD: None detected
- bmad/ present: Yes

## Health Score: 76 / 100

| Severity | Count |
| --- | --- |
| Critical | 0 |
| Major | 2 |
| Minor | 4 |
| Info | 3 |

## Critical Findings

None

## Major Findings

### [AUDIT-001] Missing automated tests

- Module: code
- Package: root
- Issue: No unit or integration tests detected.
- Impact: Regressions are likely when UI/DB behavior changes.
- Fix: Add unit tests for DB actor operations and URL validation; add at least one integration test that exercises the UI state transitions.
- BMAD Action: /dev-story AUDIT-001

### [AUDIT-002] No CI workflow enforcing build/lint/test

- Module: doc
- Package: root
- Issue: No CI configuration detected to run cargo fmt/clippy/test.
- Impact: Quality gates are manual and easy to skip.
- Fix: Add GitHub Actions workflow to run fmt, clippy, test, and build on push/PR.
- BMAD Action: /dev-story AUDIT-002

## Minor Findings

### [AUDIT-003] Blocking HTTP without explicit timeouts

- Module: perf
- Package: root
- File: src/webview.rs
- Issue: reqwest::blocking::get calls do not set a timeout.
- Impact: Fetch threads can hang on slow or stalled responses.
- Fix: Use a reqwest Client with a reasonable timeout.

### [AUDIT-004] Runtime SQLite database committed to repo

- Module: security
- Package: root
- File: rustine.db
- Issue: Repository includes a runtime DB file.
- Impact: Risk of leaking local data and polluting repo history.
- Fix: Add rustine.db to .gitignore and provide a seed/migration path if needed.

### [AUDIT-005] Tailwind CLI version mismatch

- Module: deps
- Package: root
- File: package.json
- Issue: Scripts run tailwindcss@3.4.2 while devDependencies specify ^4.1.18.
- Impact: Inconsistent behavior between local and CI builds.
- Fix: Align scripts with the declared dependency version.

### [AUDIT-006] CSS build output not guaranteed before app launch

- Module: doc
- Package: root
- Issue: UI loads assets/dist/styles.css at runtime, but no build step is enforced.
- Impact: UI can render without styles on fresh checkout.
- Fix: Document the CSS build step in README and consider failing fast if CSS is missing.

## Info

### [AUDIT-007] Feature-flagged tray implementation

- Module: arch
- Package: root
- Issue: real_tray feature is enabled by default.
- Note: Confirm desired default on all platforms and document how to disable.

### [AUDIT-008] Webview uses system browser fallback

- Module: doc
- Package: root
- Issue: Comment indicates system browser usage instead of embedded webview.
- Note: Ensure this is intentional and documented in user-facing docs.

### [AUDIT-009] Tailwind pipeline uses pnpm dlx

- Module: deps
- Package: root
- Issue: pnpm dlx downloads tooling at run time.
- Note: Consider pinning or installing locally for reproducible builds.

## Recommended Next Steps

1. /dev-story AUDIT-001 - Add baseline unit tests for DB and URL validation
2. /dev-story AUDIT-002 - Add CI workflow for fmt/clippy/test/build
3. /product-brief - Capture the product problem statement and goals
