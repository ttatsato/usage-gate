-- Add migration script here
ALTER TABLE api_keys ADD COLUMN consumer_id UUID NOT NULL REFERENCES consumers(id);
CREATE INDEX idx_api_keys_consumer_id ON api_keys(consumer_id);
