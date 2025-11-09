-- Step 1: Define custom ENUM types for roles and statuses to ensure data integrity.

CREATE TYPE album_role AS ENUM ('owner', 'contributor', 'viewer');
CREATE TYPE invitation_status AS ENUM ('pending', 'accepted', 'rejected');


-- Step 2: Create the core tables for local album management.

-- The main 'album' table.
-- We use UUID for the primary key to ensure it's globally unique, which is
-- essential for future federation.
CREATE TABLE album
(
    id          UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    owner_id    INTEGER     NOT NULL REFERENCES app_user (id) ON DELETE CASCADE,
    name        TEXT        NOT NULL,
    description TEXT,
    -- This flag enables public, view-only link sharing without requiring a login.
    is_public   BOOLEAN     NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index to quickly find all albums owned by a specific user.
CREATE INDEX idx_album_owner_id ON album (owner_id);
-- Index to quickly find public albums by their ID.
CREATE INDEX idx_album_is_public ON album (id) WHERE is_public = true;


-- A many-to-many join table connecting albums and media_items.
CREATE TABLE album_media_item
(
    album_id      UUID        NOT NULL REFERENCES album (id) ON DELETE CASCADE,
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


-- Step 3: Create the collaborator table, designed to handle BOTH local and federated users.

-- Manages permissions for albums, linking users (local or remote) to albums with a specific role.
CREATE TABLE album_collaborator
(
    id                BIGSERIAL PRIMARY KEY,
    album_id          UUID        NOT NULL REFERENCES album (id) ON DELETE CASCADE,
    -- For local users on this server. NULL if the collaborator is federated.
    user_id           INTEGER REFERENCES app_user (id) ON DELETE CASCADE,
    -- For remote users. e.g., 'user@other-server.com'. NULL if the collaborator is local.
    federated_user_id TEXT,
    role              album_role  NOT NULL,
    added_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- A user (local or federated) can only have one role per album.
    CONSTRAINT uq_album_local_collaborator UNIQUE (album_id, user_id),
    CONSTRAINT uq_album_federated_collaborator UNIQUE (album_id, federated_user_id),

    -- Enforces that a collaborator must be EITHER local OR federated, but never both.
    CONSTRAINT chk_collaborator_identity
        CHECK (
            (user_id IS NOT NULL AND federated_user_id IS NULL) OR
            (user_id IS NULL AND federated_user_id IS NOT NULL)
            )
);

-- Index for quickly finding all albums a local user collaborates on.
CREATE INDEX idx_album_collaborator_user_id ON album_collaborator (user_id);
-- Index for quickly finding all albums a federated user collaborates on.
CREATE INDEX idx_album_collaborator_federated_user_id ON album_collaborator (federated_user_id);


-- Step 4: Create tables specifically for the federation functionality.
-- These tables will only be actively used if the application is configured with a 'server_domain'.

-- Stores this server's own cryptographic keypair for signing outgoing S2S requests.
CREATE TABLE server_key
(
    id          INTEGER PRIMARY KEY  DEFAULT 1,
    -- IMPORTANT: The private key MUST BE ENCRYPTED by the application before being stored.
    -- Storing it in plaintext would be a critical security vulnerability.
    private_key TEXT        NOT NULL,
    public_key  TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT enforce_single_row CHECK (id = 1)
);

-- Caches the public keys of other trusted servers for verifying incoming requests.
CREATE TABLE federated_server
(
    domain          TEXT PRIMARY KEY,                  -- The domain of the remote server, e.g., 'photos.example.com'.
    public_key      TEXT        NOT NULL,
    last_fetched_at TIMESTAMPTZ NOT NULL DEFAULT now() -- To allow for periodic key refreshing.
);


-- Manages incoming album invitations from other servers.
CREATE TABLE federated_invitation
(
    id                   BIGSERIAL PRIMARY KEY,
    remote_album_id      UUID              NOT NULL,                                            -- The UUID of the album on the REMOTE server.
    album_name           TEXT              NOT NULL,                                            -- Stored for display in the local user's UI.
    inviter_federated_id TEXT              NOT NULL,                                            -- e.g., 'ruurd@photos.server1.com'.
    invitee_user_id      INTEGER           NOT NULL REFERENCES app_user (id) ON DELETE CASCADE, -- The local user being invited.
    role                 album_role        NOT NULL,
    status               invitation_status NOT NULL DEFAULT 'pending',
    created_at           TIMESTAMPTZ       NOT NULL DEFAULT now()
);

-- We need to create a separate unique index for pending invitations.
CREATE UNIQUE INDEX uq_pending_invitation ON federated_invitation (remote_album_id, invitee_user_id) WHERE (status = 'pending');

-- Index for quickly finding all invitations for a specific local user.
CREATE INDEX idx_federated_invitation_invitee_user_id ON federated_invitation (invitee_user_id);