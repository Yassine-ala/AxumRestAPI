CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE patients (
                          id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                          first_name TEXT NOT NULL,
                          last_name  TEXT NOT NULL,
                          birth_date DATE NOT NULL,
                          email      TEXT UNIQUE,

                          created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                          updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);