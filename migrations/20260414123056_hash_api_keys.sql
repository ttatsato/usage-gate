-- Add migration script here
ALTER TABLE api_keys DROP COLUMN key;
ALTER TABLE api_keys ADD COLUMN key_hash VARCHAR(64) NOT NULL UNIQUE;
ALTER TABLE api_keys ADD COLUMN key_prefix VARCHAR(12) NOT NULL;

CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash);
