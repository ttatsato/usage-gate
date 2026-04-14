ALTER TABLE api_keys ADD COLUMN project_id UUID NOT NULL REFERENCES projects(id);
CREATE INDEX idx_api_keys_project_id ON api_keys(project_id);
