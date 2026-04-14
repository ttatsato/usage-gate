CREATE TABLE usage_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    project_id UUID NOT NULL REFERENCES projects(id),
    consumer_id UUID NOT NULL REFERENCES consumers(id),
    api_key_id UUID NOT NULL REFERENCES api_keys(id),
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    status_code SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_usage_records_tenant_id ON usage_records(tenant_id);
CREATE INDEX idx_usage_records_project_id ON usage_records(project_id);
CREATE INDEX idx_usage_records_consumer_id ON usage_records(consumer_id);
CREATE INDEX idx_usage_records_created_at ON usage_records(created_at);
