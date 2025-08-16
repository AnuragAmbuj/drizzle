# Drizzle Development Plan

## 1. Domain-Driven Design (DDD) Breakdown

- **Core Domains:**
    - Routing
    - Authentication & Authorization (AuthN/AuthZ)
    - Rate Limits
    - Observability
    - Tenancy
    - Configuration Management

- **Architecture:**
    - Separate control-plane and data-plane concerns
    - Clear bounded contexts

## 2. Bounded Contexts → Crates

| Context | Crate Name | Purpose |
|---------|------------|---------|
| Core | `gateway-core` | Request pipeline + Pingora integration |
| Authentication | `authn` | Authentication services |
| Authorization | `authz` | Authorization services |
| Rate Limiting | `limits` | Rate limiting & quotas |
| Routing | `routing` | Service discovery & tenant-aware routing |
| Observability | `observability` | Logs, metrics, traces |
| Extensibility | `plugin-sdk` | Plugin APIs and extensions |
| Domain Models | `domain` | Control plane models and configs |
| Storage | `storage` | Persistence (Postgres/Etcd/Redis) |

## 3. Zero-Trust Proxy Layer

- **Security Features:**
    - Mutual TLS (mTLS) between services
    - Policy enforcement at every hop (AuthZ rules)
    - Per-tenant isolation
    - Namespace support

## 4. Monitoring & UI

- **Dashboard:**
    - Built with React/Next.js + ShadCN UI
    - Real-time metrics and monitoring
    - Configuration management

- **Metrics:**
    - Queries per second (QPS)
    - Error rates
    - Per-tenant quota usage

## 5. Development Approach

- **Test-Driven Development:**
    - Integration & E2E test harness
    - Property-based testing with `proptest`
    - Mock services for routing/auth

- **CI/CD:**
    - GitHub Actions pipeline
    - Automated testing
    - Deployment workflows