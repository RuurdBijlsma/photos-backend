CREATE TYPE job_type AS ENUM ('ingest', 'remove', 'analysis');
CREATE TYPE job_status AS ENUM ('queued', 'running', 'failed', 'done', 'cancelled');

CREATE TABLE jobs
(
    id                  BIGSERIAL PRIMARY KEY,
    relative_path       TEXT       NOT NULL,                  -- references files table
    job_type            job_type   NOT NULL,
    priority            INT        NOT NULL DEFAULT 100,      -- lower = higher priority
    status              job_status NOT NULL DEFAULT 'queued', -- queued, running, failed, done, cancelled
    attempts            INT        NOT NULL DEFAULT 0,
    dependency_attempts INT        NOT NULL DEFAULT 0,
    max_attempts        INT        NOT NULL DEFAULT 5,
    owner               TEXT,                                 -- worker id that claimed it
    started_at          TIMESTAMPTZ,
    finished_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ         DEFAULT now(),
    scheduled_at        TIMESTAMPTZ         DEFAULT now(),
    last_error          TEXT,
    user_id             INT        NOT NULL REFERENCES app_user (id) ON DELETE CASCADE
);

CREATE INDEX jobs_status_priority_idx ON jobs (status, priority, scheduled_at, created_at);
CREATE INDEX jobs_active_relative_path_idx
    ON jobs (relative_path)
    WHERE status IN ('queued', 'running');
CREATE INDEX jobs_relative_path_idx ON jobs (relative_path);

ALTER TABLE jobs
    ADD CONSTRAINT chk_attempts_nonneg CHECK (attempts >= 0),
    ADD CONSTRAINT chk_priority_positive CHECK (priority >= 0);
