# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Setup & Infrastructure
- `./scripts/init_db.sh` - Start Postgres Docker container (port 5442) and run sqlx migrations.
- `docker ps` - Check if `drizzle-postgres` container is running.

### Running Services
- `./scripts/run_admin.sh` - Run Control Plane (Admin API) on port 3000.
- `./scripts/run_gateway.sh` - Run Data Plane (Gateway) on port 6188.

### Testing
- `./scripts/test_domain.sh` - Run domain unit tests.
- `./scripts/test_snapshot.sh` - Run snapshot unit tests.
- `cargo test -p storage` - Run storage integration tests (requires DB).
- `./scripts/verify_e2e.sh` - Run E2E health check (requires services running).
- `make build` - Build all workspace members.
- `make test` - Run all workspace tests.

### Code Quality
- `make fmt` - Format all code.
- `make clippy` - Run clippy with warnings as errors.

## Architecture Overview

### Project Structure
This is a Rust workspace implementing **Drizzle Gateway**, an enterprise API gateway built on Pingora.

### Key Components

**Data Plane (gateway/):**
- `gateway/bin/gatewayd` - Main gateway daemon. Integrates Pingora + Poller.
- `gateway/crates/proxy` - Proxy logic (`GatewayProxy`) and `SnapshotPoller`.
- `gateway/crates/snapshot` - Shared configuration structs (`Snapshot`, `Tenant`).

**Control Plane (control-plane/):**
- `control-plane/crates/domain` - DDD core entities (`Tenant`, `Service`, `Route`).
- `control-plane/crates/storage` - Postgres persistence via `sqlx`.
- `control-plane/services/admin-api` - Axum-based REST API for management.

### Data Flow
1. **Admin API** receives config changes -> Persists to **Postgres**.
2. **Gateway Poller** polls Admin API (`GET /snapshot`) every 10s.
3. **Gateway** updates internal `ArcSwap<Snapshot>` and applies new config to traffic.

### Development Notes
- **Persistence**: Using `sqlx` with Postgres. Requires Docker.
- **Concurrency**: Gateway uses Pingora (multithreaded). Poller runs in a dedicated Tokio background thread.
- **Snapshotting**: Configuration is versioned and hashed (SHA256) for integrity.