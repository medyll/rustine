# ADR-05 — Logging & Observability

Status: proposed

Context
---------
The codebase currently emits errors via `eprintln!` with no structured logging. As implementation moves into the Implementation phase we need predictable, configurable logs for debugging, local troubleshooting, and optional file-based capture for releases.

Decision
---------
Adopt the `log` facade crate for logging calls and initialize a small runtime logger by default:

- Use `log` for logging macros (`error!`, `warn!`, `info!`, `debug!`, `trace!`).
- Use `env_logger` as the default logger (respect `RUST_LOG` for verbosity control).
- Provide an optional feature `file_logging` that enables `flexi_logger` (or similar) for file rotation in packaged releases.

Consequences
------------
- Pros: consistent logging API across crates, easy to control output during development with `RUST_LOG`, optional file capture for releases, better diagnostics than `eprintln!`.
- Cons: small dependency addition and a few code changes to replace `eprintln!` calls.

Implementation Notes
--------------------
- Add dependencies: `log = "0.4"`, `env_logger = "0.9"`. Optional: `flexi_logger = { version = "*", optional = true }` behind a `file_logging` feature.
- Initialize the logger early in `main.rs` (e.g. `env_logger::init();`), or use `flexi_logger` setup when `file_logging` feature is active.
- Replace `eprintln!` with `log::error!`/`info!` as appropriate; avoid logging sensitive user data.

Acceptance Criteria
-------------------
- `main.rs` initializes a logger and `RUST_LOG` controls log level during run.
- No calls to `eprintln!` remain for error reporting (transitional exceptions OK with TODOs).
- README documents how to enable file logging and how to use `RUST_LOG`.

Next Steps / Follow-ups
----------------------
- Create a small dev story to add dependencies and init code, then a follow-up PR that replaces `eprintln!` usage across modules.

(end)
