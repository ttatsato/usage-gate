-- Add migration script here
CREATE TABLE usage_records (
     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
     tenant_id UUID NOT NULL REFERENCES tenants(id),
     api_key_id UUID NOT NULL REFERENCES api_keys(id),
     endpoint VARCHAR(255) NOT NULL,
     method VARCHAR(10) NOT NULL,
     status_code SMALLINT NOT NULL,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
 );

CREATE INDEX idx_usage_records_tenant_id ON usage_records(tenant_id);
CREATE INDEX idx_usage_records_created_at ON usage_records(created_at);
