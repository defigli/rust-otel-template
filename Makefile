.PHONY: help build run test clean dev release run-console run-otel-logs run-all docker-up docker-down docker-logs check fmt docs run-otel

# Default target
help:
	@echo "Available commands:"
	@echo "  build          - Build (debug)"
	@echo "  release        - Build optimized release"
	@echo "  run            - Run (traces only)"
	@echo "  dev            - Run with debug log level (traces only)"
	@echo "  run-console    - Run with console formatting layer (feature: console-log)"
	@echo "  run-otel-logs  - Run exporting OTLP logs (feature: otlp-log)"
	@echo "  run-all        - Run with console + OTLP logs (features: console-log,otlp-log)"
	@echo "  test           - Run tests"
	@echo "  clean          - Clean build artifacts"
	@echo "  check          - Lint (check, clippy, fmt --check)"
	@echo "  fmt            - Format code"
	@echo "  docs           - Build documentation"
	@echo "  docker-up      - Start observability stack (Alloy, Loki, Tempo, Prometheus, Grafana)"
	@echo "  docker-down    - Stop observability stack"
	@echo "  docker-logs    - Tail observability stack logs"
	@echo "  run-otel       - (deprecated alias for run-otel-logs)"

# Build the project
build:
	cargo build

# Release build
release:
	cargo build --release

# Run the application
run:
	cargo run

# Run with debug logging (traces only)
dev:
	RUST_LOG=debug cargo run

# Run with console log formatting layer
run-console:
	cargo run --features console-log

# Run exporting OTLP logs (requires otlp-log feature)
run-otel-logs:
	OTEL_EXPORTER_OTLP_ENDPOINT?=http://localhost:4318
	OTEL_EXPORTER_OTLP_ENDPOINT=$(OTEL_EXPORTER_OTLP_ENDPOINT) cargo run --features otlp-log

# Run with both console formatting and OTLP logs
run-all:
	OTEL_EXPORTER_OTLP_ENDPOINT?=http://localhost:4318
	OTEL_EXPORTER_OTLP_ENDPOINT=$(OTEL_EXPORTER_OTLP_ENDPOINT) cargo run --features console-log,otlp-log

# Deprecated alias (backwards compatibility)
run-otel: run-otel-logs

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Check code quality
check:
	cargo check
	cargo clippy -- -D warnings
	cargo fmt -- --check

# Format code
fmt:
	cargo fmt

# Documentation (no dependencies, fast)
docs:
	cargo doc --no-deps

# Start observability stack
docker-up:
	docker-compose up -d
	@echo "Services started:"
	@echo "  Alloy UI: http://localhost:12345"
	@echo "  Loki: http://localhost:3100"
	@echo "  Tempo: http://localhost:3200"
	@echo "  Prometheus: http://localhost:9090"
	@echo "  Grafana: http://localhost:3000 (admin/secret)"

# Stop observability stack
docker-down:
	docker-compose down

# Show logs from observability stack
docker-logs:
	docker-compose logs -f