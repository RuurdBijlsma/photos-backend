CREATE TABLE process_queue
(
    id            SERIAL PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    relative_path TEXT,
    CONSTRAINT process_queue_relative_path_key UNIQUE (relative_path)
);

-- Add a unique constraint to relative_path in process_queue
ALTER TABLE process_queue
    ADD CONSTRAINT uq_process_queue_relative_path UNIQUE (relative_path);