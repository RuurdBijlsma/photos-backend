-- Step 1: Define custom ENUM types for roles and statuses to ensure data integrity.

CREATE TYPE album_role AS ENUM ('owner', 'contributor', 'viewer');
CREATE TYPE invitation_status AS ENUM ('pending', 'accepted', 'rejected');


-- Step 2: Create the core tables for local album management.

-- The main 'album' table.
CREATE TABLE album
(
    id           VARCHAR(10) PRIMARY KEY,
    owner_id     INTEGER     NOT NULL REFERENCES app_user (id) ON DELETE CASCADE,
    name         TEXT        NOT NULL,
    description  TEXT,
    thumbnail_id VARCHAR(10) NULL REFERENCES media_item (id) ON DELETE SET NULL,
    -- This flag enables public, view-only link sharing without requiring a login.
    is_public    BOOLEAN     NOT NULL DEFAULT false,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index to quickly find all albums owned by a specific user.
CREATE INDEX idx_album_owner_id ON album (owner_id);
-- Index to quickly find public albums by their ID.
CREATE INDEX idx_album_is_public ON album (id) WHERE is_public = true;


-- A many-to-many join table connecting albums and media_items.
CREATE TABLE album_media_item
(
    album_id      VARCHAR(10) NOT NULL REFERENCES album (id) ON DELETE CASCADE,
    media_item_id VARCHAR(10) NOT NULL REFERENCES media_item (id) ON DELETE CASCADE,
    -- Tracks which user added the media item to the album. Can be null if the user is deleted.
    added_by_user INT         REFERENCES app_user (id) ON DELETE SET NULL,
    added_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Ensures a media item can only appear once in any given album.
    PRIMARY KEY (album_id, media_item_id)
);

-- Index for quickly retrieving all media items within a specific album.
CREATE INDEX idx_album_media_item_album_id ON album_media_item (album_id);
-- Index for quickly finding which albums a specific media item belongs to.
CREATE INDEX idx_album_media_item_media_item_id ON album_media_item (media_item_id);


-- Step 3: Create the collaborator table, designed to handle BOTH local and remote users.

-- Manages permissions for albums, linking users (local or remote) to albums with a specific role.
CREATE TABLE album_collaborator
(
    id       BIGSERIAL PRIMARY KEY,
    album_id VARCHAR(10) NOT NULL REFERENCES album (id) ON DELETE CASCADE,
    user_id  INTEGER     NOT NULL REFERENCES app_user (id) ON DELETE CASCADE,
    role     album_role  NOT NULL,
    added_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- A user can only have one role per album.
    CONSTRAINT uq_album_local_collaborator UNIQUE (album_id, user_id)
);


-- A table to link imported media to a target album and remote owner before ingestion is complete.
CREATE TABLE pending_album_media_items
(
    relative_path        TEXT PRIMARY KEY,
    album_id             VARCHAR(10) NOT NULL REFERENCES album (id) ON DELETE CASCADE,
    -- The string identity of the original owner (e.g., 'alice@photos.alice.com').
    remote_user_identity TEXT        NOT NULL
);

-- Index for finding all pending items for a specific album import.
CREATE INDEX idx_pending_album_media_items_target_album_id ON pending_album_media_items (album_id);