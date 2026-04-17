ALTER TABLE plans ALTER COLUMN per_second_request_limit SET DEFAULT 10;
UPDATE plans SET per_second_request_limit = 10 WHERE per_second_request_limit IS NULL;
