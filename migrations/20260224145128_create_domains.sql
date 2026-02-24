CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- 1) Create domains table (no defaults except id + timestamps)
CREATE TABLE domains (
                         id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                         label TEXT NOT NULL,
                         country TEXT NOT NULL,
                         language TEXT NOT NULL,
                         max_search_identity SMALLINT NOT NULL CHECK (max_search_identity BETWEEN 0 AND 255),
                         created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                         updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 2) Add domain_id to patients (temporarily nullable)
ALTER TABLE patients
    ADD COLUMN domain_id UUID;

-- 3) Create ONE domain and link all existing patients to it
WITH d AS (
INSERT INTO domains (label, country, language, max_search_identity)
VALUES ('main', 'France', 'français', 50)
    RETURNING id
    )
UPDATE patients
SET domain_id = (SELECT id FROM d)
WHERE domain_id IS NULL;

-- 4) Enforce FK + NOT NULL
ALTER TABLE patients
    ALTER COLUMN domain_id SET NOT NULL;

ALTER TABLE patients
    ADD CONSTRAINT fk_patients_domain
        FOREIGN KEY (domain_id)
            REFERENCES domains(id)
            ON DELETE CASCADE;

CREATE INDEX idx_patients_domain_id ON patients(domain_id);