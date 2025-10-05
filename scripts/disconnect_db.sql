-- Script to disconnect all connections to the photos db.
-- Run this in pgadmin, while connected to postgres DB.

SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = 'photos'
  AND pid <> pg_backend_pid();