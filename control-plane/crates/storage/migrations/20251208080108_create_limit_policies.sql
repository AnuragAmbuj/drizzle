CREATE TABLE limit_policies (
    id UUID PRIMARY KEY,
    tenant_id UUID REFERENCES tenants(id), -- Nullable for global defaults
    name TEXT NOT NULL,
    rate INT NOT NULL, -- Tokens per second
    burst INT NOT NULL -- Max bucket capacity
);

CREATE INDEX idx_limit_policies_tenant_id ON limit_policies(tenant_id);
