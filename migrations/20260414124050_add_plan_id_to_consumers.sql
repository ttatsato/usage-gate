-- Add migration script here
ALTER TABLE consumers ADD COLUMN plan_id UUID REFERENCES plans(id);
CREATE INDEX idx_consumers_plan_id ON consumers(plan_id);
