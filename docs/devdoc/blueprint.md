# IronLattice Gateway — DDD Blueprint

## 1) Bounded Contexts

**Tenant & Org**  
- Aggregates: `Tenant`, `Project`, `Environment`, `Member`, `Role`, `ApiKey`.  
- Invariants: tenant-scoped names; role bindings must reference existing members.

**Identity & Policy**  
- Aggregates: `IdentityProvider` (OIDC/SAML), `Policy` (Cedar), `PermissionModel`.  
- Invariants: policies compile; providers have valid JWKS/metadata.

**Gateway Config**  
- Aggregates: `Service`, `Route`, `UpstreamPool`, `Timeouts`, `RetryPolicy`, `Transform`, `Certificate`, `SecretRef`.  
- Invariants: routes are disjoint or explicitly ordered; secrets referenced must exist.

**Traffic Control**  
- Aggregates: `LimitPolicy` (burst, rate, concurrency), `QuotaPlan`, `CircuitBreakerPolicy`.  
- Invariants: non-negative rates; idempotent-only retries honored.

**Distribution**  
- Aggregates: `Snapshot`, `Rollout`, `Approval`.  
- Invariants: snapshot is **read-only**, content-addressed; rollout must pass approval gates.

**Observability & Audit**  
- Aggregates: `AuditEvent`, `SLO`, `AlertRule`.  
- Invariants: audit entries immutable; redaction rules applied.

---

## 2) Ubiquitous Language (core terms)

- **Service**: A logical API surface (e.g., `billing`).  
- **Route**: Match (host/path/method/headers) ⇒ policy bundle (authN/Z, limits, upstream, transforms).  
- **Snapshot**: Versioned, signed config blob the data plane consumes atomically.  
- **Principal**: Caller identity (JWT claims/mTLS subject/API key).  
- **Policy**: Cedar rules deciding *allow/deny* on attributes from request & tenant context.

---

## 3) Aggregates & Key Fields

**Tenant**  
- `id`, `name`, `created_at`, `settings{ default_limits, mTLS_required, … }`

**Service**  
- `id`, `tenant_id`, `env`, `name`, `hosts[]`, `routes[]` (embedded value objects)

**Route (VO)**  
- `name`, `match{ hosts[], path, methods[], headers{key:pattern} }`  
- `authn{ require:[jwt|oidc|mtls|apikey], providerRef }`  
- `authz{ policyRef }`  
- `limits{ policyRef }`  
- `upstream{ strategy, pools[], retries, timeouts }`  
- `transforms{ requestHeaders, responseHeaders, jsonGuards, hmac }`

**LimitPolicy**  
- `mode: token_bucket | leaky | fixed_window`  
- `rate`, `burst`, `key: tenant|principal|route`, `scope: local|global`

**Snapshot**  
- `version`, `sha256`, `schema_version`, `tenants[]`, `services[]`, `policies[]`, `limits[]`, `certs[]`  
- Signed metadata: `sig`, `issued_at`, `issuer`

**AuditEvent**  
- `tenant`, `actor`, `action`, `object_type`, `object_id`, `diff`, `ts`, `hash_chain_prev`

---

## 4) Domain Services

- **SnapshotBuilder**: validate config → normalize → sign → emit `SnapshotPublished`.  
- **PolicyCompiler**: compile Cedar; run test vectors.  
- **LimiterPlanner**: convert `LimitPolicy` to runtime counters (local + Redis keys).  
- **RolloutManager**: canary %, health checks, auto-rollback on SLO burn.

---

## 5) Domain Events

- `TenantCreated`, `ServiceUpdated`, `RouteAdded`, `PolicyUpdated`  
- `SnapshotPublished`, `RolloutStarted`, `RolloutRolledBack`  
- `LimitExceeded`, `CircuitOpened`, `AdminLoginSucceeded/Failed`

---

## 6) Data Plane (Pingora) — Pipeline Hooks

```
on_accept -> parse_request -> guards(size/method/content-type)
 -> authn (jwt/oidc/apikey/mtls)
 -> policy_check (Cedar)
 -> rate_limit / concurrency
 -> transforms (pre)
 -> choose_upstream (LB, health, stickiness)
 -> send_upstream (timeouts/retries/hedge)
 -> transforms (post)
 -> emit_metrics/traces/audit
```

**Extensibility**  
- Hooks: `on_request`, `on_route_decision`, `on_upstream_request`, `on_response`, `on_error`.  
- Plugins: Rust (trait SDK) + WASM (wasmtime, capability-gated).

---

## 7) Control Plane APIs (contract-first)

**OpenAPI v1 (Admin) – key endpoints**  
- `POST /tenants`  
- `POST /tenants/{t}/services`  
- `POST /tenants/{t}/policies` (Cedar text + model)  
- `POST /tenants/{t}/limits`  
- `POST /envs/{env}/snapshots/validate` → `ValidationReport`  
- `POST /envs/{env}/snapshots/publish` → `{version, sha256}`  
- `POST /rollouts/{version}/start` → canary %  
- `GET  /audits?tenant=...`

**Distributor (gRPC)**  
- `rpc Watch(WatchRequest) returns (stream Snapshot)`  
- DP supports `schema_version: v1` and **N/N-1** compatibility.

---

## 8) Policy (Cedar) — Example

```cedar
permit(
  principal, action, resource
)
when {
  principal.tenant == resource.tenant &&
  action in ["read","write"] &&
  resource.route == "billing.charge" &&
  principal.role in ["svc_billing","tenant_admin"]
};
```

Attributes provided by DP:  
`principal{tenant,role,groups}`, `resource{tenant,route,method,path}`, `env{geo,device_posture}`.

---

## 9) Persistence (initial Postgres sketch)

```sql
tenants(id pk, name unique, created_at timestamptz, settings jsonb);

services(id pk, tenant_id fk, env text, name, hosts jsonb, spec jsonb, version int, updated_at);
policies(id pk, tenant_id fk, name, cedar_text text, model jsonb, version int, compiled bytea);
limit_policies(id pk, tenant_id fk, name, spec jsonb);
snapshots(id pk, env text, version text, sha256 text, schema_version text, blob bytea, issued_at, issuer);
audits(id pk, tenant_id fk, actor jsonb, action text, object_type text, object_id text, diff jsonb, ts timestamptz, prev_hash text);
```

**Redis (global)**  
- `ratelimit:{tenant|principal|route}`  
- recent `idempotency_keys`

**Vault/KMS**  
- certs, OIDC secrets → referenced by `SecretRef`.

---

## 10) Testing Strategy (TDD)

**Unit (fast)**  
- Route matching (table-driven), limiter math, retry policies.  
- Cedar policy allow/deny matrices; property tests (`proptest`) for headers/paths.

**Integration**  
- Control plane: migrations + repos (testcontainers-postgres).  
- SnapshotBuilder determinism (same inputs → same `sha256`).  
- DP boot from snapshot + e2e requests (H1/H2) against fake upstreams.

**Fuzzing**  
- HTTP parser, JWT parser (malformed tokens), header normalization.

**Perf smoke**  
- k6 scenarios: pass-through vs. authN+limit+retry; target p50/p99 SLOs.

---

## 11) Snapshot v1 (shape)

```json
{
  "version": "2025-08-16T12:00:00Z#42",
  "schema": "snapshot/v1",
  "tenants": [{ "id": "acme", "settings": { "mtls_required": true } }],
  "services": [{ "tenant":"acme","env":"prod","name":"billing","hosts":["api.acme.com"],"routes":[ ... ]}],
  "policies": [{ "name":"policy-charge","cedar":"..." }],
  "limits":   [{ "name":"tenant-default","mode":"token_bucket","rate":1000,"burst":2000,"key":"tenant","scope":"global" }],
  "certs":    [{ "id":"acme-client", "ref":"vault://pki/acme/client" }],
  "signing":  { "issuer":"control-plane@ironlattice", "sig":"base64...", "alg":"ed25519" }
}
```

---

## 12) Minimal “Definition of Done” per milestone

- **M1 (CP Core)**: CRUD + validate + deterministic signed snapshots; audit on every change.  
- **M2 (DP MVP)**: Load snapshot, route, timeouts/retries, Prometheus, readiness.  
- **M3 (ZTA & Limits)**: JWT/OIDC, mTLS, Cedar allow/deny, global limits in Redis, breaker.  
- **M4 (Console)**: forms + diffs + approvals + dashboards.  
- **M5 (Plugins)**: SDK + WASM host + sample plugins + conformance suite.  
- **M6 (Hardening)**: fuzzing, chaos, perf baselines, SLO alerts, runbooks.

---

## 13) Immediate Next Steps

1. Lock **contract stubs**: `openapi/v1/admin-api.yaml`, `proto/v1/watch.proto`, `schemas/snapshot/v1.json`.  
2. Write **failing domain tests** for: route disjointness, policy compile, snapshot determinism.  
3. Implement **SnapshotBuilder** + signatures; seed a minimal snapshot fixture.  
4. Wire `gatewayd` to **load snapshot** and route a pass-through (no auth yet).
