-- Ensure the vector extension is available.
CREATE EXTENSION IF NOT EXISTS vector;

-- Represents a person, which is a cluster of similar faces.
CREATE TABLE person
(
    id                      BIGSERIAL PRIMARY KEY,
    user_id                 INT         NOT NULL REFERENCES app_user (id) ON DELETE CASCADE,
    name                    TEXT,        -- The name assigned by the user, e.g., "Jane Doe"
    thumbnail_media_item_id VARCHAR(10) REFERENCES media_item (id) ON DELETE SET NULL,
    centroid                VECTOR(512), -- The average face embedding for this cluster
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- A user cannot have two people with the same name.
    CONSTRAINT uq_user_name UNIQUE (user_id, name)
);
CREATE INDEX idx_person_user_id ON person (user_id);