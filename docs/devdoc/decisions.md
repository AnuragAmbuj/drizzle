# Drizzle Gateway - Architectural Decisions Log (ADRs)

This file documents important decisions made during the design and development of IronLattice Gateway.

---

## ADR 001: Programming Language & Runtime
- **Decision**: Rust with Pingora as the async networking engine.
- **Rationale**: Rust provides safety + performance, Pingora gives a proven async proxy foundation.
- **Status**: Accepted ✅

---

## ADR 002: Monorepo Structure
- **Decision**: Use a Cargo workspace with separate crates for bounded contexts (gateway-core, routing, authn, etc.).
- **Rationale**: Keeps separation of concerns, avoids restructuring later, aligns with DDD principles.
- **Status**: Accepted ✅

---

## ADR 003: Multi-Tenancy Model
- **Decision**: Tenant-aware routing and policy enforcement at the gateway level.
- **Rationale**: Enables SaaS-like use cases where multiple customers share the same infrastructure.
- **Status**: Proposed 🔄

---

## ADR 004: Zero-Trust Integration
- **Decision**: All service-to-service communication requires mTLS; per-tenant AuthZ enforced by policies.
- **Rationale**: Enterprise-grade security and compliance (HIPAA, SOC2, etc.).
- **Status**: Proposed 🔄

---

## ADR 005: Observability Stack
- **Decision**: Logs + metrics exposed via Prometheus/OTel; traces via OpenTelemetry integration.
- **Rationale**: Industry-standard observability; easy integration with Grafana/ELK.
- **Status**: Proposed 🔄

---

## ADR 006: UI Framework
- **Decision**: React (Next.js) with ShadCN UI components.
- **Rationale**: User prefers ShadCN; modern, minimalistic UI fits IronLattice Labs branding.
- **Status**: Proposed 🔄
