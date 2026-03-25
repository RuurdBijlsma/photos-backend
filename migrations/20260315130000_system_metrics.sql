-- Create a table to store system-wide metrics and cached values.
CREATE TABLE system_metrics
(
    key        TEXT PRIMARY KEY,
    vector     VECTOR(768),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Text embedding cache to speed up search
CREATE TABLE text_embedding_cache
(
    id              UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    model_id        TEXT        NOT NULL,
    text TEXT        NOT NULL,
    embedding       vector(768) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (model_id, text)
);

CREATE INDEX idx_embedding_cache_lookup ON text_embedding_cache (model_id, text);
