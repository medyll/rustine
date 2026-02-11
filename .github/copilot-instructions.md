<!-- .github/copilot-instructions.md: guidance for AI coding agents working on this repository -->
# Copilot instructions — medyll/rustine

Purpose
- Help AI coding agents become immediately productive in this repository.

Quick summary
- Repo currently contains only [LICENSE](LICENSE). There is no Rust manifest (`Cargo.toml`) or `src/` tree detected. Before making broad changes, verify whether the repository is intentionally empty or if files are missing from the clone.

Big picture (what to look for)
- Look for `Cargo.toml`, `Cargo.lock`, `src/`, and `tests/` to determine this is a Rust crate. If those files are present, the project follows standard Cargo layout. If they are absent, do not assume structure — ask the user before scaffolding.
- Check for CI config files (`.github/workflows/`), README, and license. These indicate build and publish expectations.

Developer workflows (explicit commands)
- Standard Rust workflow (when Cargo files exist):
  - Build: `cargo build --workspace --all-targets`
  - Test: `cargo test --workspace`
  - Lint: `cargo clippy --all-targets -- -D warnings`
  - Format: `cargo fmt --all`
  - Run examples / bench: `cargo run --example <name>` or `cargo bench`
- If `Cargo.toml` is missing and the user asks to initialize the project, propose creating a new Cargo workspace with `cargo init` or `cargo new --lib` and confirm before applying.

Project-specific conventions
- Because no source files are present, prefer the following conservative defaults until told otherwise:
  - Use 2021 edition unless the repo indicates otherwise in `Cargo.toml`.
  - Use idiomatic module layout under `src/` (`lib.rs` + `mod` files) for libraries and `src/main.rs` for binaries.
  - Run `cargo fmt` before committing; run `cargo clippy` and fix warnings where feasible.

Integration points & external dependencies
- Inspect `Cargo.toml` for external crates and versions. If absent, ask the user for their intended dependencies before adding them.
- If CI workflows exist, follow those steps — they are authoritative for how the project is built and tested remotely.

When editing or scaffolding
- Always ask before adding a new `Cargo.toml` or initializing a Rust project. Provide a short summary of proposed files (example `Cargo.toml`, `src/lib.rs`, basic tests) and wait for confirmation.
- When implementing code, include unit tests alongside production code in the same crate (e.g., `#[cfg(test)] mod tests`) and prefer small, focused tests.

PR & commit guidance for agents
- Keep diffs small and focused. Use conventional commit-style messages describing the change.
- Run `cargo fmt` and `cargo clippy` locally and include the output summary in the PR description.

Helpful queries to run locally (examples to ask the user or to run when allowed)
- "May I scaffold a minimal Cargo project (Cargo.toml + src/lib.rs) to get started?"
- "Do you want workspace layout (multiple crates) or a single crate?"
- Run: `git status --porcelain` and `git rev-parse --abbrev-ref HEAD` to ensure you're on the intended branch before edits.

Notes and limitations
- This guidance is created from the repository contents at the time of writing (only [LICENSE](LICENSE) present). If additional files appear, re-check `Cargo.toml`, `src/` and `.github/workflows/` and update or ask clarifying questions.

If anything here is unclear, tell me which area (scaffolding, build, tests, CI) you want expanded.
