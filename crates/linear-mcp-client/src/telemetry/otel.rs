// src/telemetry/otel.rs

//! OpenTelemetry layer setup, gated behind the `otel` feature flag.
//! Wire this up once at process startup in your agent binary.
//! The library itself only emits `tracing` spans — this module
//! translates them into OTLP exports for any compatible collector.

#[cfg(feature = "otel")]
use opentelemetry::global;
#[cfg(feature = "otel")]
use opentelemetry_otlp::WithExportConfig;
#[cfg(feature = "otel")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[cfg(feature = "otel")]
#[derive(Debug, Clone)]
pub struct OtelConfig {
    /// OTLP collector endpoint, e.g. `http://localhost:4317`
    pub endpoint: String,
    /// Service name reported to the collector.
    pub service_name: String,
    /// Minimum tracing level filter, e.g. `"info,linear_mcp_client=debug"`
    pub env_filter: String,
}

#[cfg(feature = "otel")]
impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4317".into(),
            service_name: "linear-mcp-client".into(),
            env_filter: "info".into(),
        }
    }
}

/// Initialise a global `tracing` subscriber that:
/// - Emits structured logs to stdout (fmt layer)
/// - Exports spans to an OTLP collector (otel layer)
///
/// Call once at binary startup. Returns a shutdown guard — drop it
/// at process exit to flush pending spans.
#[cfg(feature = "otel")]
pub fn init_otel(config: OtelConfig) -> anyhow::Result<OtelShutdownGuard> {
    use opentelemetry::KeyValue;
    use opentelemetry_otlp::SpanExporter;
    use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&config.endpoint)
        .build()?;

    let tracer_provider = sdktrace::TracerProvider::builder()
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            config.service_name.clone(),
        )]))
        .with_batch_exporter(exporter, runtime::Tokio)
        .build();

    let tracer = tracer_provider.tracer(config.service_name.clone());
    global::set_tracer_provider(tracer_provider.clone());

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let fmt_layer = tracing_subscriber::fmt::layer().json();
    let filter = EnvFilter::try_new(&config.env_filter).unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer)
        .init();

    Ok(OtelShutdownGuard { tracer_provider })
}

/// Drop this to flush and shut down the OTel tracer provider cleanly.
#[cfg(feature = "otel")]
pub struct OtelShutdownGuard {
    tracer_provider: opentelemetry_sdk::trace::TracerProvider,
}

#[cfg(feature = "otel")]
impl Drop for OtelShutdownGuard {
    fn drop(&mut self) {
        if let Err(e) = self.tracer_provider.shutdown() {
            eprintln!("[linear-mcp-client] OTel shutdown error: {e}");
        }
    }
}
