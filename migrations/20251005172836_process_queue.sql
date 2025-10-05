CREATE TABLE process_queue
(
    id            SERIAL PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    retry_count   INT DEFAULT 0,
    relative_path TEXT,
    CONSTRAINT process_queue_relative_path_key UNIQUE (relative_path)
);
-- Add a unique constraint to relative_path in process_queue
ALTER TABLE process_queue
    ADD CONSTRAINT uq_process_queue_relative_path UNIQUE (relative_path);

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