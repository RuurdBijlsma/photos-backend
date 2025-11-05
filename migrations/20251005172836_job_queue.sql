CREATE TYPE job_type AS ENUM ('ingest', 'remove', 'analysis', 'scan', 'cleandb', 'cluster');
CREATE TYPE job_status AS ENUM ('queued', 'running', 'failed', 'done', 'cancelled');

CREATE TABLE jobs
(
    id                  BIGSERIAL PRIMARY KEY,
    relative_path       TEXT,                                 -- references files table
    user_id             INT REFERENCES app_user (id) ON DELETE CASCADE,
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
    last_heartbeat      TIMESTAMPTZ         DEFAULT now(),
    last_error          TEXT
);

-- For the job claiming worker
CREATE INDEX idx_jobs_claim_queued ON jobs (priority, relative_path, scheduled_at, created_at);
CREATE INDEX idx_jobs_claim_running ON jobs (priority, relative_path, last_heartbeat);

-- For general application queries
CREATE INDEX jobs_active_relative_path_idx ON jobs (relative_path);
CREATE INDEX idx_jobs_user_id ON jobs (user_id);

-- For monitoring/dashboarding that doesn't exist yet
CREATE INDEX jobs_status_priority_idx ON jobs (status, priority, scheduled_at, created_at);