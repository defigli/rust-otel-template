// SPDX-License-Identifier: MIT
//! Telemetry initialization helpers (traces always, optional console logs & OTLP logs).
//!
//! This module provides a thin wrapper around OpenTelemetry + `tracing` setup for
//! a typical service. The public API is intentionally minimal:
//!
//! * [`TelemetryConfig`] – user configurable endpoint & resource metadata.
//! * [`init_telemetry`] – builds providers and installs a global tracer.
//! * [`TelemetryHandle`] – allows explicit synchronous shutdown/flush.
//! * [`shutdown`] – legacy no-op kept only for backwards compatibility.
//!
//! Feature flags (Cargo features) influence behavior:
//!
//! * `console-log` – add a compact console formatting layer.
//! * `otlp-log` – enable OTLP log exporter + tracing bridge (converts tracing events to logs).
//! * (future) `metrics` – would enable metrics exporters (currently unused here).
//!
//! # Example (traces only – default build)
//! ```no_run
//! use rust_otel_template::telemetry::{init_telemetry, TelemetryConfig};
//! fn main() -> anyhow::Result<()> {
//!     let handle = init_telemetry(TelemetryConfig::default())?;
//!     // ... application logic ...
//!     handle.shutdown()?; // ensure final spans exported
//!     Ok(())
//! }
//! ```
//!
//! # Example (console + OTLP logs)
//! ```bash
//! cargo run --no-default-features --features console-log,otlp-log
//! ```
//!
//! # Shutdown
//! Call [`TelemetryHandle::shutdown`] before exiting the Tokio runtime to flush any remaining batches.
//!
//! # Error Handling
//! Shutdown aggregates exporter errors; if any occur an `anyhow::Error` is returned.
//!
//! # Threading Model
//! Batch exporters spawn worker threads (using the blocking HTTP client). No explicit async runtime handle
//! is required beyond constructing telemetry inside a Tokio context.
use anyhow::Result;
use opentelemetry::{global, KeyValue};
#[cfg(feature = "otlp-log")]
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
#[cfg(feature = "otlp-log")]
use opentelemetry_otlp::LogExporter;
use opentelemetry_otlp::{Protocol, SpanExporter, WithExportConfig};
#[cfg(feature = "otlp-log")]
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use tracing_opentelemetry::OpenTelemetryLayer;
#[cfg(feature = "console-log")]
use tracing_subscriber::fmt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Configuration used when initializing telemetry.
///
/// Values are sourced from environment variables if available:
/// * `OTEL_EXPORTER_OTLP_ENDPOINT` – base endpoint (e.g. `http://localhost:4318`).
/// * `OTEL_SERVICE_NAME` – service name resource attribute.
/// * `RUST_ENV` – deployment environment (added as `deployment.environment`).
///
/// Defaults are used when variables are absent. All fields are owned strings to simplify
/// passing across threads and avoiding lifetime issues.
#[derive(Clone, Debug)]
pub struct TelemetryConfig {
    /// Base OTLP endpoint (without per-signal suffix). Example: `http://localhost:4318`.
    pub endpoint: String,
    /// Service name reported in resource attributes (`service.name`).
    pub service_name: String,
    /// Service version reported in resource attributes (`service.version`).
    pub service_version: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4318".to_string()),
            service_name: std::env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "rust-otel-template".to_string()),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Handle allowing explicit synchronous shutdown of telemetry providers.
///
/// Dropping the handle without calling [`TelemetryHandle::shutdown`] may result in
/// losing final batches, depending on exporter internals. Always call `shutdown()`
/// at a controlled point (typically just before process exit) to ensure flush.
pub struct TelemetryHandle {
    tracer_provider: SdkTracerProvider,
    #[cfg(feature = "otlp-log")]
    logger_provider: SdkLoggerProvider,
}

impl TelemetryHandle {
    /// Flush and shutdown all configured telemetry providers.
    ///
    /// Returns `Ok(())` if every provider shutdown cleanly. If one or more providers
    /// report an error, a combined `anyhow::Error` including messages for each failing
    /// component is returned.
    ///
    /// # Examples
    /// ```no_run
    /// # use rust_otel_template::telemetry::{init_telemetry, TelemetryConfig};
    /// # fn main() -> anyhow::Result<()> {
    /// let handle = init_telemetry(TelemetryConfig::default())?;
    /// // work...
    /// handle.shutdown()?;
    /// # Ok(()) }
    /// ```
    pub fn shutdown(self) -> Result<()> {
        let mut errs = Vec::new();
        if let Err(e) = self.tracer_provider.shutdown() {
            errs.push(format!("tracer: {e}"));
        }
        #[cfg(feature = "otlp-log")]
        if let Err(e) = self.logger_provider.shutdown() {
            errs.push(format!("logger: {e}"));
        }
        if errs.is_empty() {
            Ok(())
        } else {
            anyhow::bail!(errs.join(", "))
        }
    }
}

/// Initialize tracing (and optionally logging) telemetry for the application.
///
/// This installs a global tracer provider and configures a subscriber registry
/// composed of layers (console formatting, OTLP log bridge, OpenTelemetry span layer)
/// depending on enabled Cargo features.
///
/// # Parameters
/// * `cfg` – [`TelemetryConfig`] specifying endpoint and resource attributes.
///
/// # Features
/// * `console-log` – adds a compact console formatting layer.
/// * `otlp-log` – enables OTLP log exporter and tracing-to-log bridge.
///
/// # Returns
/// A [`TelemetryHandle`] which must be explicitly shutdown to flush export queues.
///
/// # Errors
/// Returns an error if any exporter builder fails (e.g. invalid endpoint URL).
///
/// # Examples
/// Basic initialization:
/// ```no_run
/// use rust_otel_template::telemetry::{init_telemetry, TelemetryConfig};
/// let handle = init_telemetry(TelemetryConfig::default()).expect("init");
/// // ... run logic ...
/// handle.shutdown().expect("shutdown");
/// ```
pub fn init_telemetry(cfg: TelemetryConfig) -> Result<TelemetryHandle> {
    // Shared resource
    let resource = Resource::builder()
        .with_service_name(cfg.service_name.clone())
        .with_attributes([
            KeyValue::new("service.version", cfg.service_version.clone()),
            KeyValue::new(
                "deployment.environment",
                std::env::var("RUST_ENV").unwrap_or_else(|_| "dev".into()),
            ),
        ])
        .build();

    // Build exporters (HTTP binary OTLP)
    let base = cfg.endpoint.trim_end_matches('/');
    let span_exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(format!("{}/v1/traces", base))
        .build()?;

    #[cfg(feature = "otlp-log")]
    let log_exporter = LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(format!("{}/v1/logs", base))
        .build()?;

    // Providers (batch exporter convenience builder handles spawn threads)
    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .with_resource(resource.clone())
        .build();
    global::set_tracer_provider(tracer_provider.clone());

    #[cfg(feature = "otlp-log")]
    let logger_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(resource.clone())
        .build();

    #[cfg(feature = "otlp-log")]
    let bridge_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    let otel_trace_layer = OpenTelemetryLayer::new(global::tracer("rust-otel-template"));

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // Console formatting: plain compact single-line output.
    #[cfg(feature = "console-log")]
    let fmt_layer_plain = fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .compact();

    #[cfg(all(feature = "console-log", feature = "otlp-log"))]
    Registry::default()
        .with(filter)
        .with(fmt_layer_plain)
        .with(bridge_layer)
        .with(otel_trace_layer)
        .init();

    #[cfg(all(feature = "console-log", not(feature = "otlp-log")))]
    Registry::default()
        .with(filter)
        .with(fmt_layer_plain)
        .with(otel_trace_layer)
        .init();

    #[cfg(all(not(feature = "console-log"), feature = "otlp-log"))]
    Registry::default()
        .with(filter)
        .with(bridge_layer)
        .with(otel_trace_layer)
        .init();

    #[cfg(all(not(feature = "console-log"), not(feature = "otlp-log")))]
    Registry::default()
        .with(filter)
        .with(otel_trace_layer)
        .init();

    #[cfg(feature = "otlp-log")]
    return Ok(TelemetryHandle {
        tracer_provider,
        logger_provider,
    });

    #[cfg(not(feature = "otlp-log"))]
    return Ok(TelemetryHandle { tracer_provider });
}

// Legacy convenience function retained for existing callers (no-op now that handle manages shutdown).
/// Legacy no-op retained for backwards compatibility with earlier versions that
/// exposed a free `shutdown()` function. Prefer [`TelemetryHandle::shutdown`].
pub fn shutdown() {}
