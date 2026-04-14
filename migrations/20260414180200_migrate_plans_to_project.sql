ALTER TABLE plans DROP CONSTRAINT IF EXISTS plans_tenant_id_fkey;
ALTER TABLE plans DROP CONSTRAINT IF EXISTS plans_tenant_id_name_key;
ALTER TABLE plans DROP COLUMN tenant_id;
ALTER TABLE plans ADD COLUMN project_id UUID NOT NULL REFERENCES projects(id);
ALTER TABLE plans ADD CONSTRAINT plans_project_id_name_unique UNIQUE (project_id, name);
CREATE INDEX idx_plans_project_id ON plans(project_id);
