CREATE TYPE job_type AS ENUM ('INGEST', 'REMOVE', 'ANALYSIS');

CREATE TABLE job_queue
(
    id            SERIAL PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    job_type      job_type    NOT NULL,
    priority      INT         NOT NULL DEFAULT 0,
    retry_count   INT                  DEFAULT 0,
    relative_path TEXT        NOT NULL,
    user_id       INT         NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
    CONSTRAINT uq_job_queue_relative_path_job_type UNIQUE (relative_path, job_type)
);

CREATE TABLE queue_failures
(
    id            SERIAL PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    job_type      job_type    NOT NULL,
    relative_path TEXT        NOT NULL,
    user_id       INT         NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
    CONSTRAINT uq_queue_failures_relative_path_job_type UNIQUE (relative_path, job_type)
);