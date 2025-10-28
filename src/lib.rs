// SPDX-License-Identifier: MIT
//! Crate providing a minimal telemetry setup for Rust services.
//!
//! This library focuses on a pragmatic combination of `tracing` + OpenTelemetry:
//! * Traces always enabled (default build has only spans, no logging layers).
//! * Optional console logging via `console-log` feature.
//! * Optional OTLP log export + tracing bridge via `otlp-log` feature.
//!
//! The primary entry points are found in the [`telemetry`] module: [`telemetry::TelemetryConfig`],
//! [`telemetry::init_telemetry`], and [`telemetry::TelemetryHandle`].
//!
//! # Feature Flags
//! * `console-log` – add a compact console formatter (file/line/thread id).
//! * `otlp-log` – enable an OTLP log exporter and bridge tracing events into logs.
//! * (future) `metrics` – would enable metrics support; not currently wired here.
//!
//! # Quick Start
//! ```no_run
//! use rust_otel_template::telemetry::{init_telemetry, TelemetryConfig};
//! fn main() -> anyhow::Result<()> {
//!     let handle = init_telemetry(TelemetryConfig::default())?;
//!     // business logic
//!     handle.shutdown()?;
//!     Ok(())
//! }
//! ```
pub mod telemetry;

#[cfg(test)]
mod tests {
    use super::telemetry::{init_telemetry, TelemetryConfig};

    #[tokio::test]
    async fn telemetry_init_works() {
        let handle = init_telemetry(TelemetryConfig::default()).expect("telemetry init");
        handle.shutdown().expect("shutdown");
    }
}
