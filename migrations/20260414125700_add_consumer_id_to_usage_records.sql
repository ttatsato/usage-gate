-- Add migration script here
ALTER TABLE usage_records ADD COLUMN consumer_id UUID REFERENCES consumers(id);
CREATE INDEX idx_usage_records_consumer_id ON usage_records(consumer_id);
