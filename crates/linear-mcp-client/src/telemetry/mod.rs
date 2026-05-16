mod otel;
// src/telemetry/mod.rs

#[cfg(feature = "otel")]
pub mod otel;

use tracing::Span;

/// Attach common Linear MCP fields to the current span.
/// Call this at the top of any public client method.
pub fn record_client_call(span: &Span, tool: &str) {
    span.record("mcp.tool", tool);
    span.record("otel.library.name", "linear-mcp-client");
}

/// Record a successful operation outcome on the current span.
pub fn record_success(span: &Span) {
    span.record("outcome", "ok");
}

/// Record a failure outcome and attach the error string.
pub fn record_error(span: &Span, err: &dyn std::fmt::Display) {
    span.record("outcome", "error");
    span.record("error.message", &format!("{err}"));
}
