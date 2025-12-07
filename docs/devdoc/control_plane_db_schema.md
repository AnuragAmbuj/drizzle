# Drizzle Gateway — Control Plane DB Schema (PostgreSQL-first)
_IronLattice Labs_ and _The Data League LLP_

This document defines the relational schema for the **Control Plane** (CP). It stores tenants, services, routes, policies, limit configs, certificates/secret refs, rollouts, snapshots, and audits. PostgreSQL is the reference; SQLite may be used for development with minor adjustments.

---

## 0) Design Goals
- Strong **multi-tenant isolation** and referential integrity
- Deterministic, reproducible **snapshots**
- **Auditable** changes with hash chaining
- Efficient read paths for CP APIs and **Distributor** streaming
- Safe deletion via **soft-deletes** (`deleted_at`) where appropriate

---

## 1) Entity Diagram (logical)

```
Tenant ──< Project ──< Environment
  │                      │
  │                      └──< Service ──< Route ──< RouteTransform
  │                                      │
  │                                      ├──< RouteLimitBinding >── LimitPolicy
  │                                      ├──< RouteAuthN
  │                                      ├──< RouteAuthZ  >── Policy
  │                                      └──< UpstreamPool ──< Endpoint
  │
  ├──< IdentityProvider (IdP)
  ├──< CertificateRef
  ├──< SecretRef
  ├──< Policy
  ├──< LimitPolicy
  ├──< Snapshot
  ├──< Rollout
  └──< AuditEvent
```

---

## 2) DDL (PostgreSQL)

> Run migrations in order; wrap each migration in a transaction. Use `uuid-ossp` (or app-side UUID v4).

```sql
-- Enable extensions (optional)
-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Tenants / Orgs
CREATE TABLE tenants (
  id           UUID PRIMARY KEY,
  slug         TEXT UNIQUE NOT NULL CHECK (slug ~ '^[a-z0-9][a-z0-9_-]{1,63}$'),
  display_name TEXT NOT NULL,
  settings     JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at   TIMESTAMPTZ
);

-- Projects (optional namespace under tenant)
CREATE TABLE projects (
  id           UUID PRIMARY KEY,
  tenant_id    UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  slug         TEXT NOT NULL,
  display_name TEXT NOT NULL,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at   TIMESTAMPTZ,
  UNIQUE (tenant_id, slug)
);

-- Environments (dev/stage/prod)
CREATE TABLE environments (
  id           UUID PRIMARY KEY,
  tenant_id    UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  project_id   UUID REFERENCES projects(id) ON DELETE CASCADE,
  name         TEXT NOT NULL,                      -- e.g., prod, stage
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at   TIMESTAMPTZ,
  UNIQUE (tenant_id, project_id, name)
);

-- Services (logical APIs under env)
CREATE TABLE services (
  id           UUID PRIMARY KEY,
  tenant_id    UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  environment_id UUID NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
  name         TEXT NOT NULL CHECK (name ~ '^[a-z0-9][a-z0-9_-]{1,63}$'),
  description  TEXT,
  hosts        JSONB NOT NULL DEFAULT '[]'::jsonb, -- claimed hosts for this service
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at   TIMESTAMPTZ,
  UNIQUE (tenant_id, environment_id, name)
);

-- Host claiming uniqueness per env
CREATE UNIQUE INDEX services_env_host_claim
  ON services(environment_id, (jsonb_array_elements_text(hosts)))
  WHERE deleted_at IS NULL;

-- Routes
CREATE TABLE routes (
  id             UUID PRIMARY KEY,
  service_id     UUID NOT NULL REFERENCES services(id) ON DELETE CASCADE,
  name           TEXT NOT NULL CHECK (name ~ '^[a-z0-9][a-z0-9_-]{1,63}$'),
  priority       INTEGER NOT NULL DEFAULT 0 CHECK (priority >= 0),
  match_methods  TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
  match_path     JSONB NOT NULL,                    -- {type, value}
  match_headers  JSONB NOT NULL DEFAULT '[]'::jsonb,-- array of {key,op,value(s)}
  match_query    JSONB NOT NULL DEFAULT '[]'::jsonb,
  source_cidrs   TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
  implicit_head  BOOLEAN NOT NULL DEFAULT FALSE,
  timeouts       JSONB NOT NULL DEFAULT '{}'::jsonb,
  retries        JSONB NOT NULL DEFAULT '{}'::jsonb,
  transforms     JSONB NOT NULL DEFAULT '{}'::jsonb,
  traffic        JSONB NOT NULL DEFAULT '{}'::jsonb, -- split, stickiness
  mirror         JSONB,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at     TIMESTAMPTZ,
  UNIQUE (service_id, name)
);

-- Upstream pools per service (referenced by route.traffic/upstreamRef)
CREATE TABLE upstream_pools (
  id           UUID PRIMARY KEY,
  service_id   UUID NOT NULL REFERENCES services(id) ON DELETE CASCADE,
  name         TEXT NOT NULL,
  strategy     TEXT NOT NULL CHECK (strategy IN ('weighted_rr','failover','consistent_hash')),
  hash_key     JSONB,
  health       JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at   TIMESTAMPTZ,
  UNIQUE (service_id, name)
);

CREATE TABLE endpoints (
  id            UUID PRIMARY KEY,
  pool_id       UUID NOT NULL REFERENCES upstream_pools(id) ON DELETE CASCADE,
  addr          TEXT NOT NULL,         -- host:port
  weight        INTEGER NOT NULL DEFAULT 1 CHECK (weight > 0),
  metadata      JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Identity Providers (OIDC/SAML)
CREATE TABLE identity_providers (
  id           UUID PRIMARY KEY,
  tenant_id    UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  kind         TEXT NOT NULL CHECK (kind IN ('oidc','saml')),
  name         TEXT NOT NULL,
  config       JSONB NOT NULL,      -- issuer, client_id, jwks_uri, certs etc.
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at   TIMESTAMPTZ,
  UNIQUE (tenant_id, name)
);

-- AuthN per route (depends on identity_providers)
CREATE TABLE route_authn (
  id           UUID PRIMARY KEY,
  route_id     UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
  mode         TEXT NOT NULL DEFAULT 'all' CHECK (mode IN ('all','any')),
  require      TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],  -- mtls/jwt/apikey
  oidc_provider_id UUID REFERENCES identity_providers(id),
  audiences    TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
  UNIQUE (route_id)
);

-- Policies (Cedar)
CREATE TABLE policies (
  id           UUID PRIMARY KEY,
  tenant_id    UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  name         TEXT NOT NULL,
  language     TEXT NOT NULL DEFAULT 'cedar',
  model        JSONB,
  text         TEXT NOT NULL,
  compiled_b64 TEXT,                  -- optional compiled form
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at   TIMESTAMPTZ,
  UNIQUE (tenant_id, name)
);

-- AuthZ per route (depends on policies)
CREATE TABLE route_authz (
  id           UUID PRIMARY KEY,
  route_id     UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
  policy_id    UUID NOT NULL REFERENCES policies(id) ON DELETE RESTRICT,
  context      JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (route_id)
);

-- Limit Policies
CREATE TABLE limit_policies (
  id           UUID PRIMARY KEY,
  tenant_id    UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  name         TEXT NOT NULL,
  mode         TEXT NOT NULL CHECK (mode IN ('token_bucket','fixed_window','concurrency')),
  rate         INTEGER,
  burst        INTEGER,
  window       TEXT,       -- e.g., 60s
  max_concurrency INTEGER,
  key_dim      TEXT NOT NULL CHECK (key_dim IN ('tenant','principal','route')),
  scope        TEXT NOT NULL DEFAULT 'global' CHECK (scope IN ('local','global')),
  fail_mode    TEXT NOT NULL DEFAULT 'closed' CHECK (fail_mode IN ('open','closed')),
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at   TIMESTAMPTZ,
  UNIQUE (tenant_id, name)
);

-- Route ↔ Limit binding
CREATE TABLE route_limit_bindings (
  id           UUID PRIMARY KEY,
  route_id     UUID NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
  limit_policy_id UUID NOT NULL REFERENCES limit_policies(id) ON DELETE RESTRICT,
  amount       INTEGER NOT NULL DEFAULT 1 CHECK (amount > 0),
  UNIQUE (route_id)
);

-- Certificates/Secrets (by reference)
CREATE TABLE certificate_refs (
  id           UUID PRIMARY KEY,
  tenant_id    UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  name         TEXT NOT NULL,
  ref          TEXT NOT NULL,     -- vault://..., kms://..., file:// (dev only)
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at   TIMESTAMPTZ,
  UNIQUE (tenant_id, name)
);

CREATE TABLE secret_refs (
  id           UUID PRIMARY KEY,
  tenant_id    UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  name         TEXT NOT NULL,
  ref          TEXT NOT NULL,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at   TIMESTAMPTZ,
  UNIQUE (tenant_id, name)
);

-- Snapshots (immutable blobs with signatures)
CREATE TABLE snapshots (
  id             UUID PRIMARY KEY,
  tenant_id      UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  environment_id UUID NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
  version        TEXT NOT NULL,            -- monotonic string
  schema         TEXT NOT NULL DEFAULT 'snapshot/v1',
  sha256         TEXT NOT NULL,
  blob           BYTEA NOT NULL,           -- canonical JSON or protobuf
  issuer         TEXT NOT NULL,
  alg            TEXT NOT NULL,
  sig_b64        TEXT NOT NULL,
  issued_at      TIMESTAMPTZ NOT NULL,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, environment_id, version)
);

-- Rollouts
CREATE TABLE rollouts (
  id             UUID PRIMARY KEY,
  tenant_id      UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  environment_id UUID NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
  snapshot_version TEXT NOT NULL,
  canary_percent INTEGER NOT NULL DEFAULT 0 CHECK (canary_percent BETWEEN 0 AND 100),
  status         TEXT NOT NULL CHECK (status IN ('planned','in_progress','rolled_back','completed')),
  started_at     TIMESTAMPTZ,
  completed_at   TIMESTAMPTZ,
  reason         TEXT
);

-- Audit log with hash chaining
CREATE TABLE audit_events (
  id           UUID PRIMARY KEY,
  tenant_id    UUID REFERENCES tenants(id) ON DELETE SET NULL,
  actor        JSONB NOT NULL,     -- {id, kind: user|svc, ip}
  action       TEXT NOT NULL,      -- e.g., service.update
  object_type  TEXT NOT NULL,      -- e.g., route
  object_id    UUID,
  diff         JSONB NOT NULL,     -- json-patch-like
  ts           TIMESTAMPTZ NOT NULL DEFAULT now(),
  prev_hash    TEXT,
  hash         TEXT NOT NULL
);

CREATE INDEX audit_events_tenant_ts ON audit_events(tenant_id, ts DESC);
```

---

## 3) Indices & Performance Notes
- `services_env_host_claim` enforces **host uniqueness** per env and accelerates host lookups.
- Add GIN indices on JSONB fields used for filters:
  - `CREATE INDEX routes_match_path_gin ON routes USING GIN (match_path);`
  - `CREATE INDEX routes_match_headers_gin ON routes USING GIN (match_headers);`
- For large tenants, consider **materialized views** for flattened route tables for the Distributor.

---

## 4) Sample Queries

**Host claim check**
```sql
SELECT s.id, jsonb_array_elements_text(hosts) AS host
FROM services s
WHERE environment_id = $1 AND deleted_at IS NULL;
```

**Fetch full service spec for snapshot build**
```sql
SELECT s.*, r.*, p.*, lp.*, up.*, e.*
FROM services s
LEFT JOIN routes r        ON r.service_id = s.id AND r.deleted_at IS NULL
LEFT JOIN route_authz raz ON raz.route_id  = r.id
LEFT JOIN policies p      ON p.id = raz.policy_id
LEFT JOIN route_limit_bindings rlb ON rlb.route_id = r.id
LEFT JOIN limit_policies lp ON lp.id = rlb.limit_policy_id
LEFT JOIN upstream_pools up ON up.service_id = s.id AND up.deleted_at IS NULL
LEFT JOIN endpoints e ON e.pool_id = up.id
WHERE s.environment_id = $1 AND s.deleted_at IS NULL AND s.tenant_id = $2;
```

**Append to audit log (hash chained)**
```sql
WITH prev AS (
  SELECT hash FROM audit_events WHERE tenant_id = $1 ORDER BY ts DESC LIMIT 1
)
INSERT INTO audit_events(tenant_id, actor, action, object_type, object_id, diff, prev_hash, hash)
VALUES ($1, $2, $3, $4, $5, $6, (SELECT hash FROM prev), $7);
```

---

## 5) Migration Strategy
- Migrations are **idempotent** and forward-only; keep a `schema_migrations` table.
- Use semantic version tags: `V1__bootstrap.sql`, `V2__routes_authn.sql`, etc.
- Each migration **validates** assumptions (e.g., no duplicate host claims).
- Annotate breaking changes with **data backfills** and dual-read periods if needed.

---

## 6) SQLite Notes (dev)
- Replace UUID with `TEXT` (store UUID v4 strings) or `BLOB` (16 bytes).
- Remove partial indexes using JSONB expressions; emulate with helper tables.
- Use `FOREIGN KEY` pragmas and `ON DELETE CASCADE` carefully.

---

## 7) Security & Compliance
- **Row-level security** (optional): enforce `tenant_id` isolation for shared CPs.
- Secrets are **by reference** only; never store raw private keys/certs.
- Audit log is **append-only**; update only to fix metadata with a new audit entry.

---

## 8) Purge & Retention
- Soft-delete via `deleted_at`; periodic reaper hard-deletes after retention window.
- Snapshots retained for **90 days** by default; move older blobs to cold storage.

---

**End of document.**
