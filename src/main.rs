// SPDX-License-Identifier: MIT
use anyhow::Result;
use rust_otel_template::telemetry::{init_telemetry, TelemetryConfig};
use tracing::{info, instrument};

#[instrument]
async fn simulated_work() {
    info!(task = "simulated_work", "starting task");
    // Placeholder for actual business logic
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    info!(task = "simulated_work", "completed task");
}

#[tokio::main]
async fn main() -> Result<()> {
    let telemetry = init_telemetry(TelemetryConfig::default())?;
    info!("application started");

    // Example scoped span via attribute macro above
    simulated_work().await;

    info!("shutting down");
    telemetry.shutdown()?;
    Ok(())
}
