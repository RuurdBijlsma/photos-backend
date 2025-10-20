-- Creates a view that summarizes the number of media items per month for each user.
CREATE OR REPLACE VIEW timeline_summary AS
SELECT
    user_id,
    EXTRACT(YEAR FROM taken_at_local)::INT AS year,
    EXTRACT(MONTH FROM taken_at_local)::INT AS month,
    COUNT(*)::BIGINT AS media_count
FROM
    media_item
WHERE
    deleted = false
GROUP BY
    user_id,
    year,
    month
ORDER BY
    user_id,
    year DESC,
    month DESC;