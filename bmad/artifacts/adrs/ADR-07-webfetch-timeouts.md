# ADR-07 — Web Fetch Timeouts for Metadata

Status: proposed

Context
---------
The app performs blocking HTTP requests in background threads to fetch favicons and site metadata. Currently requests use `reqwest::blocking::get` without explicit timeouts which can cause fetch threads to hang indefinitely on slow or unresponsive hosts (AUDIT-003).

Decision
---------
Create and reuse a `reqwest::blocking::Client` configured with reasonable timeouts and a sensible `User-Agent` header. Use `once_cell::sync::OnceCell` or `lazy_static` to store a shared client instance.

Recommended timeout values (tunable):
- `connect_timeout`: 5 seconds
- `timeout`: 20 seconds (overall request)

Consequences
------------
- Pros: fetch threads will fail-fast on unresponsive hosts and not block indefinitely; easier to reason about background work.
- Cons: some very slow sites may not yield metadata, but core app functionality (opening URLs) remains unaffected.

Implementation Notes
--------------------
- Build the client in `webview` or a small `http` util module:

```rust
let client = reqwest::blocking::Client::builder()
    .connect_timeout(Duration::from_secs(5))
    .timeout(Duration::from_secs(20))
    .user_agent("rustine/0.1 (+https://github.com/medyll/rustine)")
    .build()?;
```

- Use this client for all metadata requests and handle timeout errors gracefully.

Acceptance Criteria
-------------------
- Metadata fetches return an error after the configured timeout.
- No background fetch thread remains blocked waiting on network IO beyond the configured timeout.

Next Steps / Follow-ups
----------------------
- Dev story: replace `reqwest::blocking::get` calls with the shared client; add unit/integration tests that simulate slow servers (using a local test server) to validate timeout behavior.

(end)
