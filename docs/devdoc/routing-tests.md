# Drizzle Gateway — Routing Tests Matrix
_IronLattice Labs_

This document specifies concrete routing test cases for Drizzle Gateway. Each case includes:
- **Fixture**: minimal snapshot excerpt to compile.
- **Request**: normalized inputs (host, path, method, headers, query).
- **Expect**: matched route & upstream (or code), and notable side-effects.

---

## 1. Host + Path Match

**Fixture**
```yaml
tenants:
  - id: t1
    routes:
      - id: r1
        host: api.acme.com
        path: /v1/orders/*
        upstream: http://acme-orders
```

**Request**
```
Host: api.acme.com
GET /v1/orders/123
```

**Expect**
```
route.id = r1
upstream = http://acme-orders
```

---

## 2. Host mismatch → 404

**Request**
```
Host: foo.com
GET /v1/orders/123
```

**Expect**
```
404 (no route found)
```

---

## 3. Path precedence (exact > prefix > catch-all)

**Fixture**
```yaml
routes:
  - id: exact
    host: api.acme.com
    path: /foo/bar
    upstream: http://svc1
  - id: prefix
    host: api.acme.com
    path: /foo/*
    upstream: http://svc2
  - id: catch
    host: api.acme.com
    path: /*
    upstream: http://svc3
```

**Request** `GET /foo/bar`  
**Expect** → `exact` → `http://svc1`

**Request** `GET /foo/baz`  
**Expect** → `prefix` → `http://svc2`

**Request** `GET /zzz`  
**Expect** → `catch` → `http://svc3`

---

## 4. Method filtering

**Fixture**
```yaml
routes:
  - id: post-only
    host: api.acme.com
    path: /submit
    methods: [POST]
    upstream: http://writer
```

**Request** `POST /submit` → expect `writer`.  
**Request** `GET /submit` → expect `405 Method Not Allowed`.

---

## 5. Query param matching

**Fixture**
```yaml
routes:
  - id: promo
    host: shop.acme.com
    path: /checkout
    query:
      promo: free
    upstream: http://promo-checkout
```

**Request** `/checkout?promo=free` → promo-checkout.  
**Request** `/checkout?promo=paid` → 404.

---

## 6. Header-based routing

**Fixture**
```yaml
routes:
  - id: mobile
    host: app.acme.com
    path: /
    headers:
      X-Client: mobile
    upstream: http://mobile-ui
```

**Request** header `X-Client: mobile` → mobile-ui.  
**Request** without → 404.

---

## 7. Weighted backends (traffic splitting)

**Fixture**
```yaml
routes:
  - id: split
    host: api.acme.com
    path: /exp
    upstreams:
      - url: http://v1
        weight: 80
      - url: http://v2
        weight: 20
```

**Request** `/exp` → random distribution, ~80% v1, 20% v2.

---

## 8. Tenant isolation

Two tenants define same host/path. Ensure tenant scoping enforced.

**Fixture**
```yaml
tenants:
  - id: t1
    routes:
      - host: api.acme.com
        path: /foo
        upstream: http://t1-foo
  - id: t2
    routes:
      - host: api.acme.com
        path: /foo
        upstream: http://t2-foo
```

**Request** with tenant=t1 context → t1-foo.  
**Request** with tenant=t2 context → t2-foo.

---

## 9. TLS-only routes

**Fixture**
```yaml
routes:
  - id: secure
    host: secure.acme.com
    path: /
    tls_only: true
    upstream: http://svc
```

**Request over HTTPS** → svc.  
**Request over HTTP** → 403.

---

## 10. Rewrite + header injection

**Fixture**
```yaml
routes:
  - id: rewrite
    host: api.acme.com
    path: /v1/*
    rewrite: /$1
    upstream: http://legacy
    inject_headers:
      X-Forwarded-For: "{client_ip}"
```

**Request** `GET /v1/abc`  
**Expect**
```
upstream = http://legacy/abc
header[X-Forwarded-For] = <client_ip>
```

---

# 🔍 Test Strategy

- Each case → unit test of `routing::match()` with fixture snapshot.  
- Integration tests with in-memory Pingora server → replay sample requests.  
- Golden snapshots stored under `gateway/tests/fixtures/`.  
