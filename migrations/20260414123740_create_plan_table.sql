-- Add migration script here
CREATE TABLE plans (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      tenant_id UUID NOT NULL REFERENCES tenants(id),
      name VARCHAR(50) NOT NULL,
      monthly_request_quota INTEGER,  -- NULL = 無制限
      created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
      updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
      UNIQUE (tenant_id, name)
);

 CREATE INDEX idx_plans_tenant_id ON plans(tenant_id);
