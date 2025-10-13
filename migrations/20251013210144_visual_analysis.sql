-- Ensure the vector extension is available.
CREATE EXTENSION IF NOT EXISTS vector;

-- A record for a single visual analysis run.
CREATE TABLE visual_analysis
(
    id            BIGSERIAL PRIMARY KEY,
    media_item_id VARCHAR(10) NOT NULL REFERENCES media_item (id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    embedding     VECTOR(1024)
);
CREATE INDEX idx_visual_analysis_media_item_id ON visual_analysis (media_item_id);
CREATE INDEX ON visual_analysis USING hnsw (embedding vector_cosine_ops);

CREATE TABLE ocr_data
(
    id                 BIGSERIAL PRIMARY KEY,
    visual_analysis_id BIGINT  NOT NULL REFERENCES visual_analysis (id) ON DELETE CASCADE,
    has_legible_text   BOOLEAN NOT NULL,
    ocr_text           TEXT
);
CREATE INDEX idx_ocr_data_visual_analysis_id ON ocr_data (visual_analysis_id);


-- CHANGE: `position` is now `position_x` and `position_y`.
CREATE TABLE ocr_box
(
    id          BIGSERIAL PRIMARY KEY,
    ocr_data_id BIGINT NOT NULL REFERENCES ocr_data (id) ON DELETE CASCADE,
    text        TEXT   NOT NULL,
    position_x  REAL   NOT NULL,
    position_y  REAL   NOT NULL,
    width       REAL   NOT NULL,
    height      REAL   NOT NULL,
    confidence  REAL   NOT NULL
);
CREATE INDEX idx_ocr_box_ocr_data_id ON ocr_box (ocr_data_id);


CREATE TABLE face
(
    id                 BIGSERIAL PRIMARY KEY,
    visual_analysis_id BIGINT      NOT NULL REFERENCES visual_analysis (id) ON DELETE CASCADE,
    position_x         REAL        NOT NULL,
    position_y         REAL        NOT NULL,
    width              REAL        NOT NULL,
    height             REAL        NOT NULL,
    confidence         REAL        NOT NULL,
    age                INT         NOT NULL,
    sex                TEXT        NOT NULL,
    mouth_left_x       REAL        NOT NULL,
    mouth_left_y       REAL        NOT NULL,
    mouth_right_x      REAL        NOT NULL,
    mouth_right_y      REAL        NOT NULL,
    nose_tip_x         REAL        NOT NULL,
    nose_tip_y         REAL        NOT NULL,
    eye_left_x         REAL        NOT NULL,
    eye_left_y         REAL        NOT NULL,
    eye_right_x        REAL        NOT NULL,
    eye_right_y        REAL        NOT NULL,
    embedding          VECTOR(512) NOT NULL
);
CREATE INDEX idx_face_visual_analysis_id ON face (visual_analysis_id);
CREATE INDEX ON face USING hnsw (embedding vector_cosine_ops);


CREATE TABLE detected_object
(
    id                 BIGSERIAL PRIMARY KEY,
    visual_analysis_id BIGINT NOT NULL REFERENCES visual_analysis (id) ON DELETE CASCADE,
    position_x         REAL   NOT NULL,
    position_y         REAL   NOT NULL,
    width              REAL   NOT NULL,
    height             REAL   NOT NULL,
    confidence         REAL   NOT NULL,
    label              TEXT   NOT NULL
);
CREATE INDEX idx_detected_object_visual_analysis_id ON detected_object (visual_analysis_id);
CREATE INDEX idx_detected_object_label ON detected_object (label);


-- Stores image quality metrics. (No changes here)
CREATE TABLE quality_data
(
    visual_analysis_id BIGINT PRIMARY KEY REFERENCES visual_analysis (id) ON DELETE CASCADE,
    blurriness         DOUBLE PRECISION NOT NULL,
    noisiness          DOUBLE PRECISION NOT NULL,
    exposure           DOUBLE PRECISION NOT NULL,
    quality_score      DOUBLE PRECISION NOT NULL
);


-- CHANGE: themes is now an array of JSONB (JSONB[]).
CREATE TABLE color_data
(
    visual_analysis_id BIGINT PRIMARY KEY REFERENCES visual_analysis (id) ON DELETE CASCADE,
    themes             JSONB[], -- Array of JSONB objects
    prominent_colors   TEXT[],
    average_hue        REAL NOT NULL,
    average_saturation REAL NOT NULL,
    average_lightness  REAL NOT NULL,
    histogram          JSONB
);


-- CHANGE: All boolean flags are now explicitly marked as NOT NULL.
CREATE TABLE caption_data
(
    visual_analysis_id   BIGINT PRIMARY KEY REFERENCES visual_analysis (id) ON DELETE CASCADE,
    default_caption      TEXT,
    main_subject         TEXT,
    contains_pets        BOOLEAN NOT NULL,
    contains_vehicle     BOOLEAN NOT NULL,
    contains_landmarks   BOOLEAN NOT NULL,
    contains_people      BOOLEAN NOT NULL,
    contains_animals     BOOLEAN NOT NULL,
    is_indoor            BOOLEAN NOT NULL,
    is_food_or_drink     BOOLEAN NOT NULL,
    is_event             BOOLEAN NOT NULL,
    is_document          BOOLEAN NOT NULL,
    is_landscape         BOOLEAN NOT NULL,
    is_cityscape         BOOLEAN NOT NULL,
    is_activity          BOOLEAN NOT NULL,
    setting              TEXT,
    pet_type             TEXT,
    animal_type          TEXT,
    food_or_drink_type   TEXT,
    vehicle_type         TEXT,
    event_type           TEXT,
    landmark_name        TEXT,
    document_type        TEXT,
    people_count         INT,
    people_mood          TEXT,
    photo_type           TEXT,
    activity_description TEXT
);
CREATE INDEX idx_caption_data_contains_people ON caption_data (contains_people) WHERE contains_people = true;
CREATE INDEX idx_caption_data_contains_pets ON caption_data (contains_pets) WHERE contains_pets = true;
CREATE INDEX idx_caption_data_is_landscape ON caption_data (is_landscape) WHERE is_landscape = true;
CREATE INDEX idx_caption_data_contains_landmarks ON caption_data (contains_landmarks) WHERE contains_landmarks = true;

CREATE INDEX idx_caption_data_landmark_name ON caption_data (landmark_name);
CREATE INDEX idx_caption_data_setting ON caption_data (setting);