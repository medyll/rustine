# Rustine - Agent Guidelines

## Build Commands

### Development
- `cargo build` - Build debug version
- `cargo build --release` - Build optimized release version
- `cargo run` - Build and run debug version
- `cargo run --release` - Build and run release version

### Testing
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run specific test
- `cargo test -- --nocapture` - Run tests with stdout output

### Code Quality
- `cargo clippy --all-targets` - Run linter (fix warnings)
- `cargo fmt` - Format code (Rustfmt)
- `cargo check` - Quick compile check without building

### Assets
- `pnpm run build:css` - Build Tailwind CSS from assets
- `pnpm run watch:css` - Watch and rebuild CSS

## Project Structure

This is a Dioxus desktop application with:
- `src/main.rs` - Application entry point
- `src/ui.rs` - Dioxus UI components and logic
- `src/db.rs` - SQLite database with async actor pattern
- `src/tray.rs` - System tray icon with menu (feature-gated)
- `src/webview.rs` - Webview utilities (currently using system browser)

## Code Style Guidelines

### Rust Conventions
- Use 2021 edition
- Prefer `anyhow::Result` for error handling
- Use `thiserror` for custom error types when needed
- Follow Rust naming: `snake_case` for functions/variables, `PascalCase` for types
- Use `#[derive(Debug, Clone)]` for data structures
- Prefer `Option<T>` over `null`, `Result<T, E>` over panics

### Imports
- Group imports: std, external crates, local modules
- Use `use` statements at top of file
- Prefer explicit imports over glob (`use dioxus::prelude::*` is acceptable for Dioxus)

### Types and Data Structures
- Use `#[derive(Debug, Clone)]` for data structures
- Prefer owned types (`String` over `&str`) for data persistence
- Use `chrono::Utc` for timestamps
- Use `serde` for JSON serialization with `#[derive(Serialize, Deserialize)]`

### Async Patterns
- Use crossbeam channels for thread communication
- Use `futures::channel::mpsc` for async channel communication
- Use `once_cell::sync::OnceCell` for global state
- Use `use_signal` and `use_effect` hooks in Dioxus components

### Error Handling
- Use `anyhow::Result` for most functions
- Use `?` operator for error propagation
- Provide context with `anyhow!("message: {}", var)`

### Database
- Use rusqlite with bundled SQLite
- Use prepared statements with `params![]`
- Use `Connection::close()` for cleanup
- Use `Mutex` for thread-safe database access

### UI Components (Dioxus)
- Use `use_signal` for reactive state
- Use `use_effect` for side effects and data loading
- Use `use_coroutine` for async operations
- Follow Dioxus component patterns
- Use Tailwind CSS for styling via `assets/src/styles.css`

### File Organization
- Keep related functionality in same module
- Use `mod.rs` for module exports when needed
- Place tests in `#[cfg(test)]` modules alongside code
- Use `src/` for source code, `assets/` for static files

### Feature Flags
- Use `real_tray` feature for tray icon functionality
- Use `optional = true` in Cargo.toml for optional dependencies
- Use `#[cfg(feature = "feature_name")]` to conditionally compile

### Naming Conventions
- Functions: `snake_case`
- Types/Structs: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Files: `snake_case.rs`
- Variables: `snake_case`

### Comments
- Use `//` for single-line comments
- Use `///` for doc comments on public items
- Avoid unnecessary comments, prefer self-documenting code

## Development Workflow

1. Make changes to code
2. Run `cargo check` for quick validation
3. Run `cargo clippy` for linting
4. Run `cargo fmt` for formatting
5. Run `cargo test` to verify tests pass
6. Run `cargo build` to ensure it compiles
7. Test the application manually

## Platform Notes

- Cross-platform desktop application (Windows/macOS/Linux)
- Uses Dioxus desktop backend
- Tray icon functionality is feature-gated
- SQLite database for data persistence
- System browser for URL opening (not embedded webview)

## Dependencies

Key external crates:
- `dioxus` - UI framework
- `rusqlite` - SQLite database
- `anyhow` - Error handling
- `serde` - Serialization
- `chrono` - Date/time
- `url` - URL parsing
- `reqwest` - HTTP requests
- `image` - Image processing
- `scraper` - HTML parsing
- `crossbeam-channel` - Thread communication
- `once_cell` - Global state
- `wry` - Webview backend
- `tray-icon` - System tray (optional)

## Testing Strategy

- Unit tests for database operations
- Integration tests for UI components
- Manual testing for desktop functionality
- No existing test framework configured yet

## Asset Management

- CSS: Tailwind CSS via PostCSS
- Icons: SVG/PNG assets in `assets/icons/`
- Build process: `pnpm run build:css` generates final CSS

## Debugging

- Use `println!` for simple debugging
- Use `log` crate for structured logging
- Use Rust Analyzer for IDE debugging
- Check Cargo.toml features for platform-specific issues
