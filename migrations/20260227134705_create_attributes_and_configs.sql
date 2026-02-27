CREATE TABLE IF NOT EXISTS attributes (
                                          id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                          key TEXT NOT NULL UNIQUE,
                                          label TEXT NOT NULL,
                                          description TEXT,
                                          data_type TEXT NOT NULL,

                                          created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                                          updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS attribute_configs (
                                                 id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                                 domain_id UUID NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
                                                 attribute_id UUID NOT NULL REFERENCES attributes(id) ON DELETE CASCADE,

                                                 search_weight SMALLINT NOT NULL DEFAULT 1 CHECK (search_weight BETWEEN 0 AND 10),
                                                 search_index SMALLINT NOT NULL,

                                                 appears_in_banner BOOLEAN NOT NULL DEFAULT false,
                                                 mandatory_in_form BOOLEAN NOT NULL DEFAULT false,
                                                 appears_in_search BOOLEAN NOT NULL DEFAULT false,
                                                 mandatory_in_search BOOLEAN NOT NULL DEFAULT false,

                                                 created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                                                 updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

                                                 UNIQUE(domain_id, attribute_id)
);
