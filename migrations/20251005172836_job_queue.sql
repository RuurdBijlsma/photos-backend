CREATE TYPE job_type AS ENUM ('INGEST', 'REMOVE');

CREATE TABLE job_queue
(
    id            SERIAL PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    job_type      job_type    NOT NULL,
    priority      INT         NOT NULL DEFAULT 0,
    retry_count   INT                  DEFAULT 0,
    relative_path TEXT,
    CONSTRAINT job_queue_relative_path_key UNIQUE (relative_path)
);
-- Add a unique constraint to relative_path in job_queue
ALTER TABLE job_queue
    ADD CONSTRAINT uq_job_queue_relative_path UNIQUE (relative_path);

CREATE TABLE queue_failures
(
    id            SERIAL PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    relative_path TEXT,
    CONSTRAINT queue_failures_relative_path_key UNIQUE (relative_path)
);
-- Add a unique constraint to relative_path in queue_failures
ALTER TABLE queue_failures
    ADD CONSTRAINT uq_queue_failures_relative_path UNIQUE (relative_path);