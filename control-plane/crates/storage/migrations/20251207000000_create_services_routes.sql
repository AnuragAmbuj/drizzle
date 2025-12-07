-- Create Services Table
CREATE TABLE services (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    hosts TEXT[] NOT NULL, -- Array of strings for upstream hosts
    environment_id UUID NOT NULL, -- Keeping it simpler, no foreign key for now as envs table doesn't exist yet
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_services_tenant_id ON services(tenant_id);

-- Create Routes Table
CREATE TABLE routes (
    id UUID PRIMARY KEY,
    service_id UUID NOT NULL REFERENCES services(id),
    name TEXT NOT NULL,
    priority INT NOT NULL DEFAULT 0,
    match_methods TEXT[], -- Array of strings "GET", "POST", etc.
    match_path JSONB, -- Storing PathMatch enum as JSONB
    match_headers JSONB, -- Storing HashMap<String, String> as JSONB
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_routes_service_id ON routes(service_id);
