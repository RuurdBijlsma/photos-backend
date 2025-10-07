-- Create the Location table first as GPS depends on it.
-- Many GPS entries can point to the same Location.
CREATE TABLE location
(
    id           SERIAL PRIMARY KEY,
    name         TEXT,
    admin1       TEXT,
    admin2       TEXT,
    country_code TEXT,
    country_name TEXT
);

CREATE TYPE user_role AS ENUM ('ADMIN', 'USER');

CREATE TABLE app_user
(
    id           SERIAL PRIMARY KEY,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    email        TEXT        NOT NULL,
    password     TEXT        NOT NULL,
    name         TEXT        NOT NULL,
    media_folder TEXT,
    role         user_role   NOT NULL DEFAULT 'USER',
    CONSTRAINT app_user_email_key UNIQUE (email)
);

CREATE TABLE refresh_token
(
    id            SERIAL PRIMARY KEY,
    user_id       INTEGER     NOT NULL REFERENCES app_user (id) ON DELETE CASCADE,
    selector      TEXT        NOT NULL UNIQUE, -- The selector must be unique for lookups
    verifier_hash TEXT        NOT NULL,        -- The hash of the verifier part
    expires_at    TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX idx_refresh_token_selector ON refresh_token (selector);

-- Create the central MediaItem table.
CREATE TABLE media_item
(
    id                  VARCHAR(10) PRIMARY KEY,
    relative_path       TEXT        NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    width               INT         NOT NULL,
    height              INT         NOT NULL,
    is_video            BOOLEAN     NOT NULL,
    data_url            TEXT        NOT NULL,
    duration_ms         BIGINT,
    taken_at_naive      TIMESTAMP, -- Naive timestamp without timezone
    use_panorama_viewer BOOLEAN,
    deleted             BOOLEAN     NOT NULL DEFAULT false,
    user_id             INT REFERENCES app_user (id) ON DELETE CASCADE,
    CONSTRAINT media_item_relative_path_key UNIQUE (relative_path)
);

-- Create the GPS table with a one-to-one relationship to MediaItem
-- and a many-to-one relationship to Location.
-- A MediaItem might not have GPS information, hence the nullable foreign key.
CREATE TABLE gps
(
    media_item_id   VARCHAR(10) PRIMARY KEY REFERENCES media_item (id) ON DELETE CASCADE,
    location_id     INT REFERENCES location (id),
    latitude        DOUBLE PRECISION NOT NULL,
    longitude       DOUBLE PRECISION NOT NULL,
    altitude        DOUBLE PRECISION,
    image_direction DOUBLE PRECISION
);

-- Create the TimeDetails table with a one-to-one relationship to MediaItem.
CREATE TABLE time_details
(
    media_item_id           VARCHAR(10) PRIMARY KEY REFERENCES media_item (id) ON DELETE CASCADE,
    datetime_utc            TIMESTAMPTZ,
    timezone_name           TEXT,
    timezone_offset_seconds INT,
    source                  TEXT,
    source_details          TEXT,
    source_confidence       TEXT
);

-- Create the Weather table with a one-to-one relationship to MediaItem.
-- A MediaItem might not have weather information.
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

-- Create the Details table with a one-to-one relationship to MediaItem.
CREATE TABLE details
(
    media_item_id                       VARCHAR(10) PRIMARY KEY REFERENCES media_item (id) ON DELETE CASCADE,
    is_motion_photo                     BOOLEAN NOT NULL,
    motion_photo_presentation_timestamp BIGINT,
    is_hdr                              BOOLEAN NOT NULL,
    is_burst                            BOOLEAN NOT NULL,
    burst_id                            TEXT,
    capture_fps                         REAL,
    video_fps                           REAL,
    is_nightsight                       BOOLEAN NOT NULL,
    is_timelapse                        BOOLEAN NOT NULL,
    mime_type                           TEXT    NOT NULL,
    size_bytes                          BIGINT  NOT NULL,
    exif                                JSONB
);

-- Create the CaptureDetails table with a one-to-one relationship to MediaItem.
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

-- Create the Panorama table with a one-to-one relationship to MediaItem.
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

-- Add indices for foreign keys and frequently queried columns
CREATE INDEX idx_gps_location_id ON gps (location_id);
CREATE INDEX idx_media_item_created_at ON media_item (created_at);
CREATE INDEX idx_media_item_taken_at_naive ON media_item (taken_at_naive);

-- Add PostGIS spatial index for latitude and longitude
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE INDEX idx_gps_location ON gps USING GIST (ST_MakePoint(longitude, latitude));