CREATE TABLE policies (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES tenants(id), -- Nullable for global policies
    name TEXT NOT NULL,
    content TEXT NOT NULL, -- Cedar policy source
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_policies_tenant_id ON policies(tenant_id);
