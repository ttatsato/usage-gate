ALTER TABLE usage_records ADD COLUMN project_id UUID NOT NULL REFERENCES projects(id);
CREATE INDEX idx_usage_records_project_id ON usage_records(project_id);
