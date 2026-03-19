-- Create a table to store system-wide metrics and cached values.
CREATE TABLE system_metrics (
    key TEXT PRIMARY KEY,
    vector VECTOR(768),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
