# Drizzle Gateway by IronLattice Labs

**Drizzle Gateway** is an enterprise-grade, multi-tenant, programmable API gateway and zero-trust access proxy built on [Pingora](https://github.com/cloudflare/pingora).  
It is designed for extensibility, security, and observability from day one.

---

## 🚀 Features
- Multi-tenant API gateway with programmable routing
- Built-in **Zero Trust** access proxy (mTLS, AuthN, AuthZ)
- Extensible plugin system (SDK + host runtime)
- Test-driven & domain-driven design (DDD-first approach)
- Observability with metrics, tracing, and logging
- Control Plane with Admin API + CLI
- Web-based Monitoring & Configuration UI

---

## 📂 Project Structure
```
drizzle/
├── gateway/               # Data plane (fast path)
│   ├── crates/
│   │   ├── gateway-core   # Core proxy engine
│   │   ├── routing        # Routing rules
│   │   ├── authn          # Authentication
│   │   ├── authz          # Authorization
│   │   ├── limits         # Rate limiting / quotas
│   │   ├── observability  # Logs / metrics / tracing
│   │   └── plugin-*       # Extensible plugins
│   └── bin/
│       └── gatewayd       # Gateway daemon
│
├── control-plane/         # Management plane
│   ├── crates/
│   │   ├── domain         # DDD domain models
│   │   ├── storage        # Persistence layer
│   │   ├── contracts      # API contracts / schemas
│   │   └── events         # Event-driven interactions
│   └── bin/
│       ├── admin-api      # REST/gRPC admin API
│       ├── distributor    # Config distributor
│       ├── idp            # Identity provider
│       └── secrets        # Secrets manager
│
├── ui/                    # Monitoring & Config UI
│   └── dashboard/         # React/Next.js + ShadCN
│
└── Docs/                  # Documentation
    ├── blueprint.md       # Full architectural blueprint
    ├── tasks.md           # Task & milestone tracker
    └── decisions.md       # ADR (decision log)
```

---

## 📘 Documentation
- [Blueprint](docs/devdoc/blueprint.md) – high-level system architecture
- [Tasks](docs/devdoc/tasks.md) – current sprint & milestones
- [Decisions](docs/devdoc/decisions.md) – architectural decision log

---

## 🛠️ Development
### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Cargo](https://doc.rust-lang.org/cargo/)
- [Make](https://www.gnu.org/software/make/) (for build/test helpers)

### Commands
```bash
# Build everything
make build

# Run tests
make test

# Run gateway daemon
cargo run -p gatewayd
```

---

## 🧭 Roadmap
- [ ] MVP Data Plane (Pingora integration, basic routing)
- [ ] Control Plane (Admin API, storage)
- [ ] Zero Trust Layer (mTLS, policy enforcement)
- [ ] Monitoring UI
- [ ] Plugin Ecosystem

---

## 🏢 About IronLattice Labs
**IronLattice Labs** is committed to building secure, performant, and programmable infrastructure for the next generation of enterprise APIs.
