ALTER TABLE consumers ADD COLUMN project_id UUID NOT NULL REFERENCES projects(id);
CREATE INDEX idx_consumers_project_id ON consumers(project_id);
