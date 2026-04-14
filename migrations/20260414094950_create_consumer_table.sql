-- Add migration script here
CREATE TABLE consumers (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      tenant_id UUID NOT NULL REFERENCES tenants(id),
      external_id VARCHAR(255),
      created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
      updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
      UNIQUE (tenant_id, external_id)
  );

  CREATE INDEX idx_consumers_tenant_id ON consumers(tenant_id);
