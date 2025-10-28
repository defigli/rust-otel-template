# Rust Project Template

![CI](https://github.com/defigli/rust-otel-template/actions/workflows/rust.yml/badge.svg)
![Security Audit](https://github.com/defigli/rust-otel-template/actions/workflows/security-audit.yml/badge.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)

A comprehensive Rust project template featuring logging, telemetry, OpenTelemetry metrics (opt-in), and async runtime with Tokio.

## Features

- **Console Logging**: Structured logging to console using `tracing`
- **HTTP Telemetry**: Distributed tracing via OpenTelemetry OTLP over HTTP
- **Metrics (opt-in)**: OpenTelemetry metrics (enable the `metrics` feature to export metrics)
- **Async Runtime**: Powered by Tokio for high-performance async operations

## Dependencies

- `tokio` - Async runtime
- `tracing` - Structured logging
- `tracing-subscriber` - Log formatting and filtering (optional feature `console-log`)
- `tracing-opentelemetry` - OpenTelemetry integration for traces
- `opentelemetry` - OpenTelemetry SDK (trace-only by default)
- `opentelemetry-otlp` - OTLP exporter for traces (+ logs when `otlp-log` feature enabled). Uses the blocking HTTP client via `reqwest-blocking-client` to avoid reactor panics during shutdown; this ensures batch processors can flush on their own worker threads during shutdown.
- `anyhow` - Error handling

## Quick Start

1. **Run the application**:
   ```bash
   cargo run
   ```

2. **With custom log level**:
   ```bash
   RUST_LOG=debug cargo run
   ```

## Observability Stack (Alloy + Loki + Tempo + Grafana)

Bring everything up:

```bash
docker compose up -d
```

### Feature Flags

- `console-log`: human-readable console output (file & line)
- `otlp-log`: OTLP log exporter + tracing bridge
- Default build enables neither; you opt-in explicitly

### Example Usage

```bash
# Traces only (default)
cargo run

# Console only
cargo run --no-default-features --features console-log

# OTLP logs only
cargo run --no-default-features --features otlp-log

# Console + OTLP logs
cargo run --no-default-features --features console-log,otlp-log
```

### Behavior Matrix

| Features Enabled         | Console Output | OTLP Logs | OTLP Traces |
|-------------------------|:--------------:|:---------:|:-----------:|
| (default, no features)  |      ❌        |    ❌     |     ✅      |
| console-log             |      ✅        |    ❌     |     ✅      |
| otlp-log                |      ❌        |    ✅     |     ✅      |
| console-log, otlp-log   |      ✅        |    ✅     |     ✅      |

### Host Ports

| Component | Purpose                        | Port  |
|-----------|--------------------------------|-------|
| Alloy     | Unified OTLP ingest & pipeline | 12345 |
| Alloy     | OTLP HTTP endpoint             | 4318  |
| Tempo     | OTLP HTTP endpoint (direct)    | 43180 |
| Tempo     | Query / HTTP API (traces UI)   | 3200  |
| Loki      | Log storage                    | 3100  |
| Grafana   | UI                             | 3000  |

Run the app pointing at Alloy (recommended):

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo run
```

Optional explicit per-signal overrides:

```bash
OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://localhost:4318/v1/traces \
OTEL_EXPORTER_OTLP_LOGS_ENDPOINT=http://localhost:4318/v1/logs \
cargo run
```

Alloy routing (see `config.alloy`):
- Logs -> Loki (`http://loki:3100/otlp`)
- Traces -> Tempo (`http://tempo:4318` base, exporter appends /v1/traces)
- Metrics -> Prometheus remote write

### Viewing Logs in Grafana / Loki

1. Open Grafana: [http://localhost:3000](http://localhost:3000)
2. Add Loki datasource (URL `http://loki:3100`) if missing.
3. Explore queries:
    - `{service_name="rust-otel-template"}`
    - If not found, try `{service.name="rust-otel-template"}` or `{resource_service_name="rust-otel-template"}` (label names vary).
4. Span-context logs (from `#[instrument]` spans) can be filtered: `{service_name="rust-otel-template", span_name="simulated_work"}`.

**Troubleshooting missing logs:**
- Ensure you used Alloy's 4318 port, not Tempo's 43180.
- Check Alloy UI ([http://localhost:12345](http://localhost:12345)) to confirm pipeline shows log throughput.
- Increase verbosity: `RUST_LOG=opentelemetry=debug,info OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo run`.
- Connectivity probe: `curl -v http://localhost:4318/v1/logs` (expect 405/404 if method without body; reaching Alloy is key).

### Viewing Traces (Tempo)

Add Tempo datasource (URL `http://tempo:3200`) or the Tempo query API URL used in your docker compose. Use Explore → Trace Search with service name or span name once traces arrive. (Future enhancement: inject trace/span IDs into console JSON.)

### Environment Variables Summary

| Variable                        | Description                                         | Default                  |
|----------------------------------|-----------------------------------------------------|--------------------------|
| OTEL_EXPORTER_OTLP_ENDPOINT      | Base OTLP endpoint (Alloy)                          | http://localhost:4318    |
| OTEL_SERVICE_NAME                | Service name resource attribute                     | rust-otel-template       |
| RUST_LOG                         | Console + subscriber filter                         | info                     |
| RUST_ENV                         | Environment resource attribute                      | dev                      |
| OTEL_EXPORTER_OTLP_TRACES_ENDPOINT | Override traces endpoint                          | unset                    |
| OTEL_EXPORTER_OTLP_LOGS_ENDPOINT | Override logs endpoint                              | unset                    |
| LOG_FORMAT                       | (Deprecated, removed) previously toggled JSON; now use cargo feature | n/a |

### Loki Label Mapping

| Resource Attribute      | Possible Loki Labels                        |
|------------------------|---------------------------------------------|
| service.name           | service_name, service.name, resource_service_name |
| service.version        | service_version, resource_service_version    |
| deployment.environment | deployment_environment                      |

Use Grafana Explore label browser to confirm actual naming.

### Configuration

The application uses the following default configuration:

- **Service Name**: `rust-otel-template`
- **Service Version**: `0.1.0`
- **OTLP Endpoint**: `http://localhost:4318`

You can modify these in the `AppConfig::default()` implementation or add environment variable support.

## Project Structure

```
├── Cargo.toml          # Dependencies and project metadata
├── src/
│   └── main.rs         # Main application with telemetry setup
├── README.md           # This file
└── target/             # Build artifacts
```

## Logging Levels

The application uses structured logging with different levels:

- `INFO`: General application flow
- `WARN`: Potentially problematic situations
- `ERROR`: Error conditions
- `DEBUG`: Detailed information for debugging

## Metrics

The application exports the following metrics (if you enable metrics instrumentation / feature):

- `requests_total`: Counter for total requests processed
- `request_duration_seconds`: Histogram of request processing duration
- `errors_total`: Counter for total errors

## Traces

Each request is traced with:
- Span names and attributes
- Request IDs for correlation
- Timing information
- Error conditions

## Customization

### Adding New Metrics

```rust
let meter = global::meter("your-service-name");
let custom_counter = meter
    .u64_counter("custom_metric")
    .with_description("Description of your metric")
    .init();

custom_counter.add(1, &[KeyValue::new("label", "value")]);
```

### Adding New Traces

```rust
use tracing::{info, span, Level};

let span = span!(Level::INFO, "operation_name", attribute = "value");
let _enter = span.enter();
info!("Operation started");
// Your code here
```

### Custom Configuration

Extend the `AppConfig` struct to add environment variable support:

```rust
impl AppConfig {
    fn from_env() -> Self {
        Self {
            service_name: std::env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "rust-otel-template".to_string()),
            otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4318".to_string()),
            // ... other fields
        }
    }
}
```

## Production Considerations

1. **Error Handling**: Add comprehensive error handling for your business logic
2. **Configuration**: Use environment variables or config files for production settings
3. **Security**: Ensure OTLP endpoints are properly secured
4. **Performance**: Monitor the overhead of telemetry in production
5. **Sampling**: Consider trace sampling for high-volume applications

## Next Steps / Enhancements

- Add metrics exporter usage example (counter + histogram wiring).
- Include trace & span IDs in console JSON output for easier correlation.
- Structured error logging with span status set to error.
- Graceful shutdown improvements: ensure batch processors flush within timeout.

### Shutdown Notes

Telemetry is initialized and shutdown inside the Tokio runtime (`#[tokio::main]`). The batch processors use a blocking HTTP client to ensure final flushes succeed in their worker threads without depending on the Tokio reactor; ensure your runtime awaits shutdown so processors can complete their flush interval.

## License

This template is provided as-is for educational and development purposes.
