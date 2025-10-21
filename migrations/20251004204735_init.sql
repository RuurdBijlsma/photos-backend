CREATE TYPE user_role AS ENUM ('admin', 'user');

-- Create the Location table. Many GPS entries can point to one Location.
CREATE TABLE location
(
    id           SERIAL PRIMARY KEY,
    name         TEXT,
    admin1       TEXT,
    admin2       TEXT,
    country_code TEXT NOT NULL,
    country_name TEXT NOT NULL
);
CREATE INDEX idx_location_lookup ON location (name, admin1, country_code);

-- Create the User table.
CREATE TABLE app_user
(
    id           SERIAL PRIMARY KEY,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    email        TEXT        NOT NULL UNIQUE,
    password     TEXT        NOT NULL,
    name         TEXT        NOT NULL,
    media_folder TEXT,
    role         user_role   NOT NULL DEFAULT 'user'
);

-- Create the Refresh Token table for persistent user sessions.
CREATE TABLE refresh_token
(
    id            SERIAL PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    user_id       INTEGER     NOT NULL REFERENCES app_user (id) ON DELETE CASCADE,
    selector      TEXT        NOT NULL UNIQUE,
    verifier_hash TEXT        NOT NULL,
    expires_at    TIMESTAMPTZ NOT NULL
);

-- Create the central MediaItem table.
-- Other tables with specific metadata will link to this one.
CREATE TABLE media_item
(
    id                  VARCHAR(10) PRIMARY KEY,
    hash                TEXT        NOT NULL,
    relative_path       TEXT        NOT NULL UNIQUE,
    user_id             INT         NOT NULL REFERENCES app_user (id) ON DELETE CASCADE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    width               INT         NOT NULL,
    height              INT         NOT NULL,
    is_video            BOOLEAN     NOT NULL,
    duration_ms         BIGINT,
    taken_at_local      TIMESTAMP   NOT NULL,
    taken_at_utc        TIMESTAMPTZ,
    use_panorama_viewer BOOLEAN     NOT NULL,
    deleted             BOOLEAN     NOT NULL DEFAULT false
);

-- The following tables store optional, detailed metadata for a MediaItem.
-- They use a one-to-one relationship where the primary key is also a foreign key
-- referencing media_item.id. This keeps the main media_item table clean.

CREATE TABLE gps
(
    media_item_id   VARCHAR(10) PRIMARY KEY REFERENCES media_item (id) ON DELETE CASCADE,
    location_id     INT REFERENCES location (id), -- A media item can exist without a resolved location.
    latitude        DOUBLE PRECISION NOT NULL,
    longitude       DOUBLE PRECISION NOT NULL,
    altitude        DOUBLE PRECISION,
    image_direction DOUBLE PRECISION
);

CREATE TABLE time_details
(
    media_item_id           VARCHAR(10) PRIMARY KEY REFERENCES media_item (id) ON DELETE CASCADE,
    timezone_name           TEXT,
    timezone_offset_seconds INT,
    source                  TEXT,
    source_details          TEXT,
    source_confidence       TEXT
);

CREATE TABLE weather
(
    media_item_id     VARCHAR(10) PRIMARY KEY REFERENCES media_item (id) ON DELETE CASCADE,
    temperature       REAL,
    dew_point         REAL,
    relative_humidity INT,
    precipitation     REAL,
    snow              INT,
    wind_direction    INT,
    wind_speed        REAL,
    peak_wind_gust    REAL,
    pressure          REAL,
    sunshine_minutes  INT,
    condition         TEXT,
    sunrise           TIMESTAMPTZ,
    sunset            TIMESTAMPTZ,
    dawn              TIMESTAMPTZ,
    dusk              TIMESTAMPTZ,
    is_daytime        BOOLEAN
);

CREATE TABLE details
(
    media_item_id                       VARCHAR(10) PRIMARY KEY REFERENCES media_item (id) ON DELETE CASCADE,
    mime_type                           TEXT    NOT NULL,
    size_bytes                          BIGINT  NOT NULL,
    is_motion_photo                     BOOLEAN NOT NULL,
    motion_photo_presentation_timestamp BIGINT,
    is_hdr                              BOOLEAN NOT NULL,
    is_burst                            BOOLEAN NOT NULL,
    burst_id                            TEXT,
    capture_fps                         REAL,
    video_fps                           REAL,
    is_nightsight                       BOOLEAN NOT NULL,
    is_timelapse                        BOOLEAN NOT NULL,
    exif                                JSONB
);

CREATE TABLE capture_details
(
    media_item_id VARCHAR(10) PRIMARY KEY REFERENCES media_item (id) ON DELETE CASCADE,
    iso           INT,
    exposure_time REAL,
    aperture      REAL,
    focal_length  REAL,
    camera_make   TEXT,
    camera_model  TEXT
);

CREATE TABLE panorama
(
    media_item_id      VARCHAR(10) PRIMARY KEY REFERENCES media_item (id) ON DELETE CASCADE,
    is_photosphere     BOOLEAN NOT NULL,
    projection_type    TEXT,
    horizontal_fov_deg REAL,
    vertical_fov_deg   REAL,
    center_yaw_deg     REAL,
    center_pitch_deg   REAL
);

-- Create indices for foreign keys and frequently queried columns for performance.

-- Index for the foreign key in the gps table.
CREATE INDEX idx_gps_location_id ON gps (location_id);

-- Indices for common sorting/filtering operations on media_item.
CREATE INDEX idx_media_item_created_at ON media_item (created_at);
CREATE INDEX idx_media_item_taken_at_local ON media_item (taken_at_local);
CREATE INDEX idx_media_item_user_id ON media_item (user_id);
CREATE INDEX idx_media_item_user_hash ON media_item (user_id, hash);
