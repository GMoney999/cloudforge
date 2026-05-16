# CloudForge Agent Instructions

## Repository Structure
- Rust workspace with crates in `crates/` directory
- Main crate: `linear-mcp-client` (MCP client for Linear integration)
- Workspace defined in root `Cargo.toml`

## Development Commands
- Build: `cargo build`
- Test: `cargo test`
- Run tests for specific crate: `cd crates/linear-mcp-client && cargo test`
- Check formatting: `cargo fmt -- --check`
- Clippy lint: `cargo clippy -- -D warnings`

## Workspace Features
- `linear-mcp-client` features:
  - `bearer-auth` (default)
  - `oauth` (requires base64, sha2, rand)
  - `otel` (OpenTelemetry tracing)
  - `test-utils` (Wiremock, tracing-subscriber)

## Important Notes
- Uses Rust 2024 edition
- MCP SDK via `rmcp` crate
- Async runtime with Tokio
- HTTP client with reqwest (rustls-tls)
- Tracing/subscription available via features
- Generated files: none identified
- No special dev server or codegen steps

## Verification Order
1. `cargo fmt -- --check` (formatting)
2. `cargo clippy -- -D warnings` (linting)
3. `cargo build` (compilation)
4. `cargo test` (testing)