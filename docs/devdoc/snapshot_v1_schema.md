# Drizzle Gateway — Snapshot v1 Schema
_IronLattice Labs_

This document defines the **JSON Schema** for Drizzle Gateway configuration snapshots (v1).  
Snapshots are the persisted representation of tenant configuration that the control plane distributes to data-plane instances.

---

## Top-level schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DrizzleGatewaySnapshotV1",
  "type": "object",
  "required": ["version", "tenants"],
  "properties": {
    "version": {
      "type": "string",
      "enum": ["v1"]
    },
    "tenants": {
      "type": "array",
      "items": { "$ref": "#/definitions/Tenant" }
    },
    "generated_at": {
      "type": "string",
      "format": "date-time"
    }
  },
  "definitions": {
    "Tenant": {
      "type": "object",
      "required": ["id", "routes"],
      "properties": {
        "id": { "type": "string" },
        "routes": {
          "type": "array",
          "items": { "$ref": "#/definitions/Route" }
        }
      }
    },
    "Route": {
      "type": "object",
      "required": ["id", "host", "path", "upstreams"],
      "properties": {
        "id": { "type": "string" },
        "host": { "type": "string" },
        "path": { "type": "string" },
        "methods": {
          "type": "array",
          "items": { "type": "string" },
          "uniqueItems": true
        },
        "headers": {
          "type": "object",
          "additionalProperties": { "type": "string" }
        },
        "query": {
          "type": "object",
          "additionalProperties": { "type": "string" }
        },
        "tls_only": { "type": "boolean" },
        "rewrite": { "type": "string" },
        "inject_headers": {
          "type": "object",
          "additionalProperties": { "type": "string" }
        },
        "upstreams": {
          "type": "array",
          "items": { "$ref": "#/definitions/Upstream" },
          "minItems": 1
        }
      }
    },
    "Upstream": {
      "type": "object",
      "required": ["url"],
      "properties": {
        "url": { "type": "string", "format": "uri" },
        "weight": { "type": "integer", "minimum": 0, "default": 1 }
      }
    }
  }
}
```

---

## Notes

- **version** → fixed to `"v1"`. Allows forward migration later.  
- **tenants** → array of tenant-scoped configs. Isolation is strict.  
- **Route precedence** → evaluated in engine, not schema.  
- **Upstreams** → must be ≥1. Supports weighted load balancing.  
- **generated_at** → optional, for debugging/audit trails.  

---

## Example Snapshot

```json
{
  "version": "v1",
  "generated_at": "2025-08-16T09:00:00Z",
  "tenants": [
    {
      "id": "acme",
      "routes": [
        {
          "id": "orders-v1",
          "host": "api.acme.com",
          "path": "/v1/orders/*",
          "methods": ["GET", "POST"],
          "upstreams": [
            { "url": "http://10.0.0.1:8080", "weight": 80 },
            { "url": "http://10.0.0.2:8080", "weight": 20 }
          ]
        }
      ]
    }
  ]
}
```

---

## Validation Strategy

- Schema is validated via `jsonschema` crate (Rust) or AJV (Node).  
- Control plane enforces schema compliance before distributing.  
- Data plane rejects snapshots failing validation.  
