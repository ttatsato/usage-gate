CREATE TABLE consumers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    project_id UUID NOT NULL REFERENCES projects(id),
    plan_id UUID REFERENCES plans(id),
    external_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, external_id)
);

CREATE INDEX idx_consumers_tenant_id ON consumers(tenant_id);
CREATE INDEX idx_consumers_project_id ON consumers(project_id);
CREATE INDEX idx_consumers_plan_id ON consumers(plan_id);
