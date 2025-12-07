# Drizzle Gateway — Observability Schema
_IronLattice Labs_

This document specifies the **canonical metrics, traces, and logs** for Drizzle Gateway. It is vendor-agnostic and assumes OpenTelemetry (OTel) across the stack.

---

## 0) Design Tenets
- **Low-cardinality by default**; high-cardinality labels (like user ids) are forbidden.
- **Consistent naming** (`drizzle_*`) and units (base SI; durations in **seconds**).
- **Exemplars** on key histograms (latency, upstream RTT) for trace correlation.
- **Redaction** at source; PII/secret-bearing headers are never logged.

---

## 1) Metrics (Prometheus/OTel)

### 1.1 Request Lifecycle
| Name | Type | Unit | Labels | Description |
|------|------|------|--------|-------------|
| `drizzle_requests_total` | counter | 1 | `tenant`,`service`,`route`,`code_class` | Count of requests processed by DP |
| `drizzle_request_duration_seconds` | histogram | seconds | `tenant`,`service`,`route` | End-to-end request time (ingress→egress) |
| `drizzle_active_requests` | gauge | 1 | `tenant`,`service` | Number of in-flight requests |
| `drizzle_request_body_bytes` | histogram | bytes | `tenant`,`service`,`route` | Size of request bodies |
| `drizzle_response_body_bytes` | histogram | bytes | `tenant`,`service`,`route` | Size of response bodies |

**Buckets** (histograms):  
- `drizzle_request_duration_seconds`: `[0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2, 5]`  
- `drizzle_upstream_rtt_seconds`: `[0.001, 0.003, 0.006, 0.012, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2]`  
- `drizzle_*_bytes`: exponential `[256, 512, 1k, 2k, 4k, 8k, 16k, 64k, 256k, 1M, 4M, 16M]`

`code_class` is one of: `1xx`,`2xx`,`3xx`,`4xx`,`5xx`.

### 1.2 AuthN/AuthZ & Security
| Name | Type | Unit | Labels | Description |
|------|------|------|--------|-------------|
| `drizzle_authn_attempts_total` | counter | 1 | `tenant`,`method` | Attempts of AuthN (`mtls`,`jwt`,`apikey`) |
| `drizzle_authn_failures_total` | counter | 1 | `tenant`,`method`,`reason` | Failures grouped by reason (`expired`,`sig`,`missing`,`aud`) |
| `drizzle_authz_decisions_total` | counter | 1 | `tenant`,`policy`,`effect` | Cedar decisions (`allow`,`deny`) |
| `drizzle_mtls_handshakes_total` | counter | 1 | `tenant`,`result` | TLS handshakes attempted vs failed |

### 1.3 Upstream & Retries
| Name | Type | Unit | Labels | Description |
|------|------|------|--------|-------------|
| `drizzle_upstream_requests_total` | counter | 1 | `tenant`,`service`,`route`,`pool` | Requests sent to upstreams |
| `drizzle_upstream_rtt_seconds` | histogram | seconds | `tenant`,`service`,`route`,`pool` | Time to first byte from upstream |
| `drizzle_retries_total` | counter | 1 | `tenant`,`service`,`route`,`reason` | Retry attempts by reason (`connect_failure`,`reset`,`5xx`) |
| `drizzle_hedges_total` | counter | 1 | `tenant`,`service`,`route` | Hedged requests fired |
| `drizzle_circuit_state` | gauge | 0/1 | `tenant`,`service`,`route`,`pool`,`state` | 1 if `open`/`half_open`; 0 when `closed` |

### 1.4 Rate Limiting & Quotas
| Name | Type | Unit | Labels | Description |
|------|------|------|--------|-------------|
| `drizzle_limit_checks_total` | counter | 1 | `tenant`,`policy`,`result` | Limit checks attempted (`ok`,`reject`,`error`) |
| `drizzle_limit_rejects_total` | counter | 1 | `tenant`,`policy`,`scope` | Requests dropped by limiter |
| `drizzle_quota_consumed_total` | counter | tokens | `tenant`,`policy` | Tokens consumed |

### 1.5 Mirrors & Plugins
| Name | Type | Unit | Labels | Description |
|------|------|------|--------|-------------|
| `drizzle_mirrors_total` | counter | 1 | `tenant`,`service`,`route` | Shadow copies dispatched |
| `drizzle_plugin_failures_total` | counter | 1 | `tenant`,`plugin`,`hook` | Plugin errors by hook |
| `drizzle_plugin_duration_seconds` | histogram | seconds | `tenant`,`plugin`,`hook` | Hook execution time |

### 1.6 System
| Name | Type | Unit | Labels | Description |
|------|------|------|--------|-------------|
| `drizzle_snapshot_version_info` | gauge | 1 | `env`,`version`,`schema` | Info metric (value always 1) |
| `drizzle_snapshot_apply_total` | counter | 1 | `result` | Snapshot apply outcomes (`ok`,`invalid_sig`,`compile_error`) |
| `drizzle_worker_queue_depth` | gauge | 1 | `worker` | Pending tasks on worker queues |
| `drizzle_fd_in_use` | gauge | fds |  | File descriptors in use |
| `drizzle_tls_sessions` | gauge | 1 | `state` | TLS sessions by state (`active`,`resumed`) |

**Cardinality policy**
- Label sets are limited to `{tenant,service,route,code_class}` on critical paths.  
- Avoid arbitrary header values as labels.  
- `pool` label limited to small, static sets per service.

---

## 2) Traces (OpenTelemetry)

### 2.1 Span Model
```
drizzle.request
 ├─ drizzle.authn
 ├─ drizzle.authz
 ├─ drizzle.route
 └─ drizzle.upstream (one per attempt; retries create siblings)
```

### 2.2 Span Attributes (selected)
- **`drizzle.request`**
  - `http.method`, `http.target`, `http.route` (normalized)
  - `http.scheme`, `http.flavor` (1.1/2)
  - `net.peer.ip`, `net.peer.port`
  - `drizzle.tenant`, `drizzle.service`, `drizzle.route`
  - `drizzle.snapshot.version`
  - `drizzle.canary.bucket` (if split), `drizzle.hash.key.kind` (cookie/header/ip/principal)
  - `http.status_code`
- **`drizzle.authn`**
  - `drizzle.authn.method` (`mtls`,`jwt`,`apikey`), `drizzle.authn.result` (`ok`/`fail`), `error.type`
- **`drizzle.authz`**
  - `drizzle.policy.name`, `drizzle.authz.effect` (`allow`/`deny`)
- **`drizzle.upstream`**
  - `net.peer.name` (upstream host), `net.peer.port`
  - `drizzle.pool`, `drizzle.attempt`, `drizzle.retry.reason` (if any)

### 2.3 Sampling
- Default **parent-based** with 1–5% head sampling.
- **Tail-sampling** rules (optional): always sample when `http.status_code>=500` or latency > p99.
- Propagation: `traceparent`, `baggage`. Extract `x-b3-*` / `x-amzn-trace-id` if configured.

### 2.4 Exemplars
- Attach trace exemplars to:
  - `drizzle_request_duration_seconds`
  - `drizzle_upstream_rtt_seconds`

---

## 3) Structured Logs

### 3.1 Log Levels
- **INFO**: routing decisions, snapshot applied, normal ops
- **WARN**: policy soft fails, degraded dependencies (Redis/JWKS), retries > threshold
- **ERROR**: upstream failures after retries, snapshot compile errors
- **DEBUG/TRACE**: disabled by default; enable per-route/tenant sampling window

### 3.2 Log Event Shapes

**Access Log** (`info`)
```json
{
  "ts": "2025-08-17T12:34:56.789Z",
  "level": "info",
  "event": "access",
  "request_id": "01J9X...",
  "tenant": "acme",
  "service": "billing",
  "route": "charge",
  "method": "POST",
  "path": "/v1/charge",
  "status": 200,
  "duration_ms": 12.4,
  "bytes_in": 512,
  "bytes_out": 1024,
  "canary_bucket": "stable",
  "upstream": {"pool":"primary","addr":"10.0.1.10:8443","attempts":1}
}
```

**Auth Decision** (`info`)
```json
{
  "ts": "2025-08-17T12:34:56.700Z",
  "level": "info",
  "event": "authz_decision",
  "tenant": "acme",
  "policy": "charge-policy",
  "effect": "allow",
  "request_hash": "sha256:...",
  "principal": {"tenant":"acme","subject":"svc:billing","roles":["svc_billing"]}
}
```

**Error** (`error`)
```json
{
  "ts": "2025-08-17T12:34:56.710Z",
  "level": "error",
  "event": "upstream_error",
  "tenant": "acme",
  "service": "billing",
  "route": "charge",
  "retry_count": 2,
  "reason": "connect_failure",
  "message": "All endpoints failed"
}
```

### 3.3 Redaction & PII Policy
- Drop/Hash: `authorization`, `cookie`, `set-cookie`, `x-api-key`, `x-forwarded-for` (hash), `x-real-ip` (hash).
- Optional allow-list for specific header keys via tenant config.
- Path parameters are logged; query strings are dropped unless allow-listed keys.

### 3.4 Correlation
- `request_id` generated if absent; propagate `x-request-id`/`traceparent` downstream.
- Include `span_id`/`trace_id` in logs when tracing enabled.

---

## 4) Health & Readiness Endpoints
- `GET /healthz` → 200 when process is alive.
- `GET /readyz` → 200 when snapshot loaded & dependencies OK (Redis/JWKS) within thresholds.
- `GET /metrics` → Prometheus plaintext exposition.

---

## 5) Alerts & SLO Guards (starter rules)

### Latency
- **P99** `drizzle_request_duration_seconds` > **200ms** for 5 min → page.
- **P50** drift > 2× baseline for 30 min → warn.

### Error Rate
- `rate(drizzle_requests_total{code_class="5xx"}[5m]) / rate(drizzle_requests_total[5m]) > 2%` → page.
- **Auth failures**: `drizzle_authn_failures_total` surge > 5× baseline → warn.

### Dependency Degradation
- `drizzle_snapshot_apply_total{result="compile_error"} > 0` in 10m → page.
- Redis/JWKS health check failures > threshold → warn.

### Capacity
- `drizzle_fd_in_use / process_max_fds > 0.8` → warn.
- `drizzle_worker_queue_depth` p95 > 100 for 10m → scale up.

---

## 6) OTLP Exporter Configuration (reference)

```yaml
otel:
  service_name: drizzle-gateway
  traces:
    exporter: otlp
    otlp:
      endpoint: "http://otel-collector:4317"
      protocol: grpc
  metrics:
    exporter: otlp
    otlp:
      endpoint: "http://otel-collector:4317"
      protocol: grpc
    temporality: cumulative
  logs:
    exporter: otlp
    otlp:
      endpoint: "http://otel-collector:4317"
      protocol: grpc
  sampling:
    head: 0.05
    tail_rules:
      - match: { attribute: "http.status_code", op: ">=", value: 500 }
        sample: 1.0
      - match: { metric: "drizzle_request_duration_seconds", op: ">", value: "p99" }
        sample: 1.0
```

---

## 7) Dashboards (Panels to include)
- **Traffic**: RPS, code_class stacked bars; per-tenant slices.
- **Latency**: p50/p95/p99 with exemplars; upstream RTT.
- **Errors**: 4xx vs 5xx; top routes; retry reasons.
- **Security**: AuthN failures by reason; AuthZ allow/deny.
- **Limits**: rejects, quota consumption; burst utilization.
- **System**: snapshot version, worker queue depth, TLS sessions.

---

**End of document.**
