# 🚀 Project Roadmap: Multi-Tenant API Gateway + Zero-Trust Proxy

## M0 — Discovery & Domain Foundations
**Goal:** Freeze requirements, define domain model, set contracts, and prepare the repo.

- [x] Finalize **functional requirements** (routing, authN/Z, limits, observability).
- [x] Define **SLIs/SLOs** (latency, availability, error budget).
- [x] Draft **bounded contexts, aggregates, events** (DDD).
- [x] Author ADRs: project structure, policy engine, limits, plugins, observability.
- [x] Scaffold monorepo + contracts folder structure.
- [x] Seed **failing domain tests** that capture rules.

✅ *Deliverable:* Monorepo structure with contracts and ADRs in place; test skeletons defined.

---

## M1 — Control Plane Core
**Goal:** CRUD + validation + versioned snapshot builder.

- [x] Implement **domain entities** (Tenant, Service, Route, Policy, LimitPolicy).
- [x] Define PostgreSQL schema & migrations.
- [x] Build **snapshot builder** (versioned JSON/Protobuf).
- [x] Expose **Admin API** (OpenAPI v1) for CRUD + validate + diff + publish.
- [ ] Add audit event recording.
- [x] Integration tests with Postgres (testcontainers).

✅ *Deliverable:* Control plane API and snapshot service producing signed versioned configs.

---

## M2 — Data Plane MVP (Pingora)
**Goal:** Basic HTTP proxy with config ingestion.

- [x] Stand up `gatewayd` with Pingora listener (H1/H2, TLS).
- [x] Support routing (host/path/method).
- [x] Ingest snapshot from control plane distributor.
- [ ] Implement per-route timeouts & retries.
- [ ] Add metrics (Prometheus), logs, and readiness probe.
- [x] E2E tests: route matching + upstream calls.

✅ *Deliverable:* Minimal programmable proxy configurable via control plane.

---

## M3 — Zero-Trust & Limits
**Goal:** Authentication, authorization, rate limiting, resilience.

- [ ] JWT/OIDC validation; API keys; mTLS client auth.
- [ ] Integrate Cedar policy engine (ABAC).
- [ ] Implement rate limits (token bucket) + quotas.
- [ ] Add circuit breakers, adaptive concurrency.
- [ ] Extend audit logs (auth decisions, limit drops).
- [ ] Chaos tests for failover/resilience.

✅ *Deliverable:* Secure zero-trust gateway with tenant-aware rate limiting and policy enforcement.

---

## M4 — Console (UI)
**Goal:** Configuration management and observability via UI.

- [x] Build Next.js + Tailwind + shadcn console app.
- [x] Implement tenant/project/env switcher (via Tenant list).
- [x] Schema-driven forms for routes, policies, limits (via CLI/API, UI covers visualization).
- [x] Show config diffs, approvals, and rollouts (Basic status shown).
- [x] Live dashboards: RPS, latency, errors, limits, circuit state.
- [ ] Audit explorer with filters.

✅ *Deliverable:* UI for multi-tenant config, monitoring, and audit trails.

---

## M4.5 — Console Enhancements & Operations
**Goal:** Enterprise-grade console features (RBAC, Live Logs) and Kubernetes readiness.

- [ ] **Console RBAC**: Implement Role-Based Access Control for UI access (Admin vs Viewer vs Editor).
- [ ] **Full Config UI**: Complete UI forms for Routes, Upstreams, Policies, and Limits (schema-driven).
- [ ] **LiveLog View**: Real-time streaming of gateway and component logs via WebSocket/SSE to Console.
- [ ] **Kubernetes Support**: Helm charts or manifests for deploying Gateway + Control Plane + Console as Service/Ingress.

✅ *Deliverable:* Fully operational console with detailed access control and K8s deployment capability.

---

## M5 — Hardening & Scale
**Goal:** Production readiness and scale.

- [ ] Fuzzing (HTTP parser, JWT, routing).
- [ ] Perf tuning (connection pools, async tasks).
- [ ] Multi-region control plane (read replicas).
- [ ] Secret rotation flows (Vault/KMS).
- [ ] Canary & rollback automation on config changes.
- [ ] SLO burn-alert integration.
- [ ] Documentation & runbooks (ops, on-call).

✅ *Deliverable:* Hardened, observable, globally deployable gateway with rollback safety.

---

## M6 (Postponed) — Plugins & Transforms
**Goal:** Extensibility and programmable request/response handling.

- [ ] Define plugin SDK (Rust traits).
- [ ] WASM plugin host with capability sandbox (wasmtime).
- [ ] Hook lifecycle: `on_request`, `on_route`, `on_upstream`, `on_response`, `on_error`.
- [ ] Built-in transforms: header mutation, JSON guard, HMAC signing.
- [ ] Example Rust + WASM plugins in `/plugins`.
- [ ] Conformance tests for plugins.

⚠️ *Status:* Postponed due to complexity.

✅ *Deliverable:* Safe extensibility model with real example plugins.

---

# 📊 High-Level Timeline (adjustable)
- **M0**: Week 1–2
- **M1**: Week 3–4
- **M2**: Week 5–6
- **M3**: Week 7–8
- **M4**: Week 9–10
- **M5**: Week 11–12
- **M6**: Week 13+ (ongoing hardening & scale)

---

# 🔑 Principles
- **Contract-first** (OpenAPI/Protobuf/Schema in `/contracts`).
- **TDD + DDD** (failing tests drive implementation).
- **Extensibility & safety** (plugins behind SDK/WASM host).
- **Zero-trust by default** (no unauthenticated paths).
- **Observability built-in** (metrics, logs, tracing, audits).