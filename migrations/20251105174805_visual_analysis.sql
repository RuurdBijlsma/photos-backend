-- A record for a single visual analysis run.
CREATE TABLE visual_analysis
(
    id            BIGSERIAL PRIMARY KEY,
    media_item_id VARCHAR(10)  NOT NULL REFERENCES media_item (id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    embedding     VECTOR(1024) NOT NULL,
    percentage    INT          NOT NULL
);
CREATE INDEX idx_visual_analysis_media_item_id ON visual_analysis (media_item_id);
CREATE INDEX ON visual_analysis USING hnsw (embedding vector_cosine_ops);


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
    embedding          VECTOR(512) NOT NULL,
    person_id          BIGINT      REFERENCES person (id) ON DELETE SET NULL
);
CREATE INDEX idx_face_visual_analysis_id ON face (visual_analysis_id);
CREATE INDEX ON face USING hnsw (embedding vector_cosine_ops);
CREATE INDEX idx_face_person_id ON face (person_id);


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


-- Stores image quality metrics.
CREATE TABLE quality_data
(
    visual_analysis_id      BIGINT PRIMARY KEY REFERENCES visual_analysis (id) ON DELETE CASCADE,
    exposure                SMALLINT         NOT NULL,
    contrast                SMALLINT         NOT NULL,
    sharpness               SMALLINT         NOT NULL,
    color_accuracy          SMALLINT         NOT NULL,
    composition             SMALLINT         NOT NULL,
    subject_clarity         SMALLINT         NOT NULL,
    visual_impact           SMALLINT         NOT NULL,
    creativity              SMALLINT         NOT NULL,
    color_harmony           SMALLINT         NOT NULL,
    storytelling            SMALLINT         NOT NULL,
    style_suitability       SMALLINT         NOT NULL,
    weighted_score          DOUBLE PRECISION NOT NULL,
    measured_blurriness     DOUBLE PRECISION NOT NULL,
    measured_noisiness      DOUBLE PRECISION NOT NULL,
    measured_exposure       DOUBLE PRECISION NOT NULL,
    measured_weighted_score DOUBLE PRECISION NOT NULL
);


-- CHANGE: themes is now an array of JSONB (JSONB[]).
CREATE TABLE color_data
(
    visual_analysis_id BIGINT PRIMARY KEY REFERENCES visual_analysis (id) ON DELETE CASCADE,
    themes             JSONB[] NOT NULL,
    prominent_colors   TEXT[]  NOT NULL,
    average_hue        REAL    NOT NULL,
    average_saturation REAL    NOT NULL,
    average_lightness  REAL    NOT NULL,
    histogram          JSONB   NOT NULL
);


-- CHANGE: All boolean flags are now explicitly marked as NOT NULL.
CREATE TABLE caption_data
(
    visual_analysis_id   BIGINT PRIMARY KEY REFERENCES visual_analysis (id) ON DELETE CASCADE,
    default_caption      TEXT    NOT NULL,
    main_subject         TEXT    NOT NULL,
    contains_pets        BOOLEAN NOT NULL,
    contains_vehicle     BOOLEAN NOT NULL,
    contains_landmarks   BOOLEAN NOT NULL,
    contains_people      BOOLEAN NOT NULL,
    contains_animals     BOOLEAN NOT NULL,
    contains_text        BOOLEAN NOT NULL,
    is_indoor            BOOLEAN NOT NULL,
    is_food_or_drink     BOOLEAN NOT NULL,
    is_event             BOOLEAN NOT NULL,
    is_document          BOOLEAN NOT NULL,
    is_landscape         BOOLEAN NOT NULL,
    is_cityscape         BOOLEAN NOT NULL,
    is_activity          BOOLEAN NOT NULL,
    setting              TEXT    NOT NULL,
    ocr_text             TEXT,
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