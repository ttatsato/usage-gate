-- Add migration script here
CREATE TABLE plans (
     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
     name VARCHAR(50) NOT NULL UNIQUE,
     monthly_request_quota INTEGER,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
 );
