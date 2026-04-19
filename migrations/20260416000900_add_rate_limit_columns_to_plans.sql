ALTER TABLE plans ADD COLUMN daily_request_quota INTEGER;
ALTER TABLE plans ADD COLUMN hourly_request_quota INTEGER;
ALTER TABLE plans ADD COLUMN per_second_request_limit INTEGER;
