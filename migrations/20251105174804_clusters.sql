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

-- Represents a cluster of visually similar photos, analogous to the 'person' table.
CREATE TABLE photo_cluster
(
    id                      BIGSERIAL PRIMARY KEY,
    user_id                 INT         NOT NULL REFERENCES app_user (id) ON DELETE CASCADE,
    title                    TEXT,        -- Optional auto generated title, e.g., "Beach Sunsets"
    thumbnail_media_item_id VARCHAR(10) REFERENCES media_item (id) ON DELETE SET NULL,
    centroid                VECTOR(1024), -- The average photo embedding (from visual_analysis) for this cluster
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for quickly finding all clusters belonging to a user.
CREATE INDEX idx_photo_cluster_user_id ON photo_cluster (user_id);

CREATE TABLE media_item_photo_cluster
(
    media_item_id    VARCHAR(10) NOT NULL REFERENCES media_item (id) ON DELETE CASCADE,
    photo_cluster_id BIGINT      NOT NULL REFERENCES photo_cluster (id) ON DELETE CASCADE,
    PRIMARY KEY (media_item_id, photo_cluster_id)
);

-- Index for finding all media items in a specific cluster.
CREATE INDEX idx_media_item_photo_cluster_cluster_id ON media_item_photo_cluster (photo_cluster_id);