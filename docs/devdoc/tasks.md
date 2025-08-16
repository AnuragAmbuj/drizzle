# Drizzle Gateway - Tasks

This file tracks ongoing tasks, milestones, and development steps for IronLattice Gateway.

---

## 🟢 Current Focus
- [ ] Finalize Domain Model (DDD)
- [ ] Implement core crate scaffolding (gateway-core, routing, authn, authz)
- [ ] Setup CI/CD pipeline with Rust tests + linting
- [ ] Define integration test harness with mock services

---

## 📌 Upcoming Milestones
### Milestone 1: MVP Data Plane
- [ ] Pingora integration (basic reverse proxy)
- [ ] Routing crate (tenant-based routes)
- [ ] AuthN stub (JWT validation)
- [ ] Observability crate (basic logs/metrics)

### Milestone 2: Control Plane
- [ ] Domain models (Tenant, Policy, RouteConfig)
- [ ] Storage crate with Postgres/SQLite
- [ ] Admin API crate
- [ ] CLI for managing configs

### Milestone 3: Zero Trust Layer
- [ ] mTLS support in gateway-core
- [ ] AuthZ enforcement rules
- [ ] Per-tenant isolation policies

### Milestone 4: Monitoring UI
- [ ] Dashboard (Next.js + ShadCN UI)
- [ ] Live metrics + charts
- [ ] Route/policy management UI

---

## ✅ Completed
- [ ] Project scaffolded (monorepo structure in place)
- [ ] Core roadmap documented
