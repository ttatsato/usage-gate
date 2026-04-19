CREATE TABLE quota_counters (
    consumer_id UUID NOT NULL REFERENCES consumers(id),
    period VARCHAR(50) NOT NULL,
    count BIGINT NOT NULL DEFAULT 0,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (consumer_id, period)
);
